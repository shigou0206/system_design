//! WebSocket模块
//! 
//! 实现WebSocket双向流通信，展示JsonRPC框架的BidirectionalStream功能

use std::collections::HashMap;
use std::sync::Arc;
use axum::{
    extract::{State, WebSocketUpgrade, ws::{WebSocket, Message}},
    response::Response,
};
use tokio::sync::{RwLock, mpsc};
use futures::{sink::SinkExt, stream::StreamExt};
use serde_json::{Value, json};
use uuid::Uuid;
use tracing::{info, debug, error, warn};

use crate::server::AppState;

/// WebSocket连接管理器
pub type ConnectionManager = Arc<RwLock<HashMap<String, ConnectionInfo>>>;

/// 连接信息
#[derive(Debug, Clone)]
pub struct ConnectionInfo {
    pub id: String,
    pub connected_at: chrono::DateTime<chrono::Utc>,
    pub last_activity: chrono::DateTime<chrono::Utc>,
    pub message_count: u64,
    pub subscriptions: Vec<String>,
}

/// 活跃数据流
#[derive(Debug)]
pub struct DataStream {
    #[allow(dead_code)]
    pub id: String,
    pub connection_id: String,
    #[allow(dead_code)]
    pub interval_ms: u64,
    pub sender: mpsc::UnboundedSender<()>,
}

/// 聊天室
#[derive(Debug, Clone)]
pub struct ChatRoom {
    #[allow(dead_code)]
    pub name: String,
    #[allow(dead_code)]
    pub members: Vec<String>,
    #[allow(dead_code)]
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// 全局WebSocket状态
pub struct WebSocketState {
    pub connections: ConnectionManager,
    pub data_streams: Arc<RwLock<HashMap<String, DataStream>>>,
    pub chat_rooms: Arc<RwLock<HashMap<String, ChatRoom>>>,
}

impl WebSocketState {
    pub fn new() -> Self {
        Self {
            connections: Arc::new(RwLock::new(HashMap::new())),
            data_streams: Arc::new(RwLock::new(HashMap::new())),
            chat_rooms: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

lazy_static::lazy_static! {
    static ref WS_STATE: WebSocketState = WebSocketState::new();
}

/// WebSocket连接处理器
pub async fn websocket_handler(
    ws: WebSocketUpgrade,
    State(app_state): State<AppState>,
) -> Response {
    ws.on_upgrade(move |socket| handle_websocket(socket, app_state))
}

/// 处理WebSocket连接
async fn handle_websocket(socket: WebSocket, app_state: AppState) {
    let connection_id = Uuid::new_v4().to_string();
    info!("新WebSocket连接: {}", connection_id);
    
    // 注册连接
    register_connection(&connection_id).await;
    
    // 分离发送和接收
    let (mut sender, mut receiver) = socket.split();
    
    // 发送欢迎消息
    let welcome_msg = json!({
        "jsonrpc": "2.0",
        "method": "connection.welcome",
        "params": {
            "connection_id": connection_id,
            "timestamp": chrono::Utc::now(),
            "message": "Welcome to JsonRPC Playground WebSocket!"
        }
    });
    
    if let Err(e) = sender.send(Message::Text(welcome_msg.to_string())).await {
        error!("发送欢迎消息失败: {}", e);
        return;
    }
    
    // 处理消息
    while let Some(msg) = receiver.next().await {
        match msg {
            Ok(Message::Text(text)) => {
                update_connection_activity(&connection_id).await;
                
                match handle_text_message(&connection_id, &text, &app_state).await {
                    Ok(Some(response)) => {
                        if let Err(e) = sender.send(Message::Text(response)).await {
                            error!("发送响应失败: {}", e);
                            break;
                        }
                    }
                    Ok(None) => {
                        // 无需响应的消息
                    }
                    Err(e) => {
                        let error_response = json!({
                            "jsonrpc": "2.0",
                            "error": {
                                "code": -32603,
                                "message": "Internal error",
                                "data": e.to_string()
                            },
                            "id": null
                        });
                        
                        if let Err(e) = sender.send(Message::Text(error_response.to_string())).await {
                            error!("发送错误响应失败: {}", e);
                            break;
                        }
                    }
                }
            }
            Ok(Message::Close(_)) => {
                info!("WebSocket连接关闭: {}", connection_id);
                break;
            }
            Ok(Message::Ping(_)) => {
                // 自动处理ping/pong
            }
            Ok(Message::Pong(_)) => {
                // 收到pong响应
            }
            Err(e) => {
                warn!("WebSocket消息错误: {}", e);
                break;
            }
            _ => {
                // 忽略其他消息类型
            }
        }
    }
    
    // 清理连接
    cleanup_connection(&connection_id).await;
    info!("WebSocket连接已清理: {}", connection_id);
}

/// 处理文本消息
async fn handle_text_message(
    connection_id: &str,
    text: &str,
    _app_state: &AppState,
) -> anyhow::Result<Option<String>> {
    debug!("收到WebSocket消息 [{}]: {}", connection_id, text);
    
    let request: Value = serde_json::from_str(text)
        .map_err(|e| anyhow::anyhow!("Invalid JSON: {}", e))?;
    
    let method = request.get("method")
        .and_then(|m| m.as_str())
        .ok_or_else(|| anyhow::anyhow!("Missing method field"))?;
    
    let params = request.get("params").cloned().unwrap_or(Value::Null);
    let id = request.get("id").cloned();
    
    let result = match method {
        // 流式数据服务
        "stream.data" => handle_data_stream(connection_id, params).await,
        
        // 聊天服务
        "stream.chat" => handle_chat_stream(connection_id, params).await,
        
        // 连接管理
        "connection.info" => get_connection_info(connection_id).await,
        "connection.list" => list_connections().await,
        
        // 系统命令
        "system.ping" => Ok(json!({"pong": chrono::Utc::now()})),
        
        _ => Err(anyhow::anyhow!("Unknown WebSocket method: {}", method))
    };
    
    match result {
        Ok(res) => {
            let response = json!({
                "jsonrpc": "2.0",
                "result": res,
                "id": id
            });
            Ok(Some(response.to_string()))
        }
        Err(e) => {
            let error_response = json!({
                "jsonrpc": "2.0",
                "error": {
                    "code": -32603,
                    "message": e.to_string()
                },
                "id": id
            });
            Ok(Some(error_response.to_string()))
        }
    }
}

/// 处理数据流
async fn handle_data_stream(connection_id: &str, params: Value) -> anyhow::Result<Value> {
    let action = params.get("type")
        .and_then(|t| t.as_str())
        .ok_or_else(|| anyhow::anyhow!("Missing type parameter"))?;
    
    match action {
        "start" => {
            let interval_ms = params.get("interval_ms")
                .and_then(|i| i.as_u64())
                .unwrap_or(1000);
            
            start_data_stream(connection_id, interval_ms).await
        }
        "stop" => {
            stop_data_stream(connection_id).await
        }
        _ => Err(anyhow::anyhow!("Invalid data stream action: {}", action))
    }
}

/// 启动数据流
async fn start_data_stream(connection_id: &str, interval_ms: u64) -> anyhow::Result<Value> {
    let stream_id = format!("{}_{}", connection_id, Uuid::new_v4());
    let (tx, mut rx) = mpsc::unbounded_channel();
    
    // 存储流信息
    let stream = DataStream {
        id: stream_id.clone(),
        connection_id: connection_id.to_string(),
        interval_ms,
        sender: tx,
    };
    
    WS_STATE.data_streams.write().await.insert(stream_id.clone(), stream);
    
    // 启动数据生成任务
    let stream_id_clone = stream_id.clone();
    let _connection_id_clone = connection_id.to_string();
    
    tokio::spawn(async move {
        let mut counter = 0u64;
        let mut interval = tokio::time::interval(tokio::time::Duration::from_millis(interval_ms));
        
        loop {
            tokio::select! {
                _ = interval.tick() => {
                    counter += 1;
                    
                    let _data_msg = json!({
                        "jsonrpc": "2.0",
                        "method": "stream.data.update",
                        "params": {
                            "stream_id": stream_id_clone,
                            "counter": counter,
                            "timestamp": chrono::Utc::now(),
                            "random_value": fastrand::f64(),
                            "data": format!("Generated data #{}", counter)
                        }
                    });
                    
                    // 这里应该向连接发送消息，暂时记录日志
                    debug!("数据流 [{}] 生成数据: {}", stream_id_clone, counter);
                }
                _ = rx.recv() => {
                    info!("数据流 [{}] 停止", stream_id_clone);
                    break;
                }
            }
        }
        
        // 清理流信息
        WS_STATE.data_streams.write().await.remove(&stream_id_clone);
    });
    
    Ok(json!({
        "stream_id": stream_id,
        "status": "started",
        "interval_ms": interval_ms,
        "message": "Data stream started successfully"
    }))
}

/// 停止数据流
async fn stop_data_stream(connection_id: &str) -> anyhow::Result<Value> {
    let mut streams = WS_STATE.data_streams.write().await;
    let mut stopped_count = 0;
    
    // 找到并停止所有该连接的流
    let mut to_remove = Vec::new();
    for (stream_id, stream) in streams.iter() {
        if stream.connection_id == connection_id {
            let _ = stream.sender.send(());
            to_remove.push(stream_id.clone());
            stopped_count += 1;
        }
    }
    
    for stream_id in to_remove {
        streams.remove(&stream_id);
    }
    
    Ok(json!({
        "stopped_streams": stopped_count,
        "message": "Data streams stopped successfully"
    }))
}

/// 处理聊天流
async fn handle_chat_stream(connection_id: &str, params: Value) -> anyhow::Result<Value> {
    let action = params.get("action")
        .and_then(|a| a.as_str())
        .ok_or_else(|| anyhow::anyhow!("Missing action parameter"))?;
    
    let room = params.get("room")
        .and_then(|r| r.as_str())
        .unwrap_or("general");
    
    match action {
        "join" => join_chat_room(connection_id, room).await,
        "leave" => leave_chat_room(connection_id, room).await,
        "message" => {
            let message = params.get("message")
                .and_then(|m| m.as_str())
                .ok_or_else(|| anyhow::anyhow!("Missing message parameter"))?;
            send_chat_message(connection_id, room, message).await
        }
        _ => Err(anyhow::anyhow!("Invalid chat action: {}", action))
    }
}

/// 加入聊天室
async fn join_chat_room(connection_id: &str, room_name: &str) -> anyhow::Result<Value> {
    let mut rooms = WS_STATE.chat_rooms.write().await;
    
    let room = rooms.entry(room_name.to_string()).or_insert_with(|| ChatRoom {
        name: room_name.to_string(),
        members: Vec::new(),
        created_at: chrono::Utc::now(),
    });
    
    if !room.members.contains(&connection_id.to_string()) {
        room.members.push(connection_id.to_string());
    }
    
    Ok(json!({
        "room": room_name,
        "action": "joined",
        "member_count": room.members.len(),
        "timestamp": chrono::Utc::now()
    }))
}

/// 离开聊天室
async fn leave_chat_room(connection_id: &str, room_name: &str) -> anyhow::Result<Value> {
    let mut rooms = WS_STATE.chat_rooms.write().await;
    
    if let Some(room) = rooms.get_mut(room_name) {
        room.members.retain(|id| id != connection_id);
        
        Ok(json!({
            "room": room_name,
            "action": "left",
            "member_count": room.members.len(),
            "timestamp": chrono::Utc::now()
        }))
    } else {
        Err(anyhow::anyhow!("Room not found: {}", room_name))
    }
}

/// 发送聊天消息
async fn send_chat_message(connection_id: &str, room_name: &str, message: &str) -> anyhow::Result<Value> {
    let rooms = WS_STATE.chat_rooms.read().await;
    
    if let Some(room) = rooms.get(room_name) {
        // 这里应该向房间所有成员广播消息
        info!("聊天消息 [{}] from {}: {}", room_name, connection_id, message);
        
        Ok(json!({
            "room": room_name,
            "action": "message_sent",
            "message": message,
            "sender": connection_id,
            "timestamp": chrono::Utc::now(),
            "delivered_to": room.members.len()
        }))
    } else {
        Err(anyhow::anyhow!("Room not found: {}", room_name))
    }
}

/// 注册新连接
async fn register_connection(connection_id: &str) {
    let now = chrono::Utc::now();
    let connection = ConnectionInfo {
        id: connection_id.to_string(),
        connected_at: now,
        last_activity: now,
        message_count: 0,
        subscriptions: Vec::new(),
    };
    
    WS_STATE.connections.write().await.insert(connection_id.to_string(), connection);
    info!("注册WebSocket连接: {}", connection_id);
}

/// 更新连接活动
async fn update_connection_activity(connection_id: &str) {
    if let Some(conn) = WS_STATE.connections.write().await.get_mut(connection_id) {
        conn.last_activity = chrono::Utc::now();
        conn.message_count += 1;
    }
}

/// 清理连接
async fn cleanup_connection(connection_id: &str) {
    // 移除连接
    WS_STATE.connections.write().await.remove(connection_id);
    
    // 停止所有数据流
    let _ = stop_data_stream(connection_id).await;
    
    // 从所有聊天室移除
    let mut rooms = WS_STATE.chat_rooms.write().await;
    for room in rooms.values_mut() {
        room.members.retain(|id| id != connection_id);
    }
}

/// 获取连接信息
async fn get_connection_info(connection_id: &str) -> anyhow::Result<Value> {
    let connections = WS_STATE.connections.read().await;
    
    if let Some(conn) = connections.get(connection_id) {
        Ok(json!({
            "id": conn.id,
            "connected_at": conn.connected_at,
            "last_activity": conn.last_activity,
            "message_count": conn.message_count,
            "subscriptions": conn.subscriptions
        }))
    } else {
        Err(anyhow::anyhow!("Connection not found"))
    }
}

/// 列出所有连接
async fn list_connections() -> anyhow::Result<Value> {
    let connections = WS_STATE.connections.read().await;
    let connection_list: Vec<Value> = connections.values()
        .map(|conn| json!({
            "id": conn.id,
            "connected_at": conn.connected_at,
            "last_activity": conn.last_activity,
            "message_count": conn.message_count
        }))
        .collect();
    
    Ok(json!({
        "count": connections.len(),
        "connections": connection_list
    }))
} 