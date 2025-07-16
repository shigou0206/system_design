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
use tracing::{info, debug, error};

// 使用 jsonrpc-rust 库的类型定义
use jsonrpc_rust::prelude::*;

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

/// WebSocket全局状态
struct WebSocketState {
    connections: ConnectionManager,
    data_streams: Arc<RwLock<HashMap<String, DataStream>>>,
    chat_rooms: Arc<RwLock<HashMap<String, ChatRoom>>>,
}

lazy_static::lazy_static! {
    static ref WS_STATE: WebSocketState = WebSocketState {
        connections: Arc::new(RwLock::new(HashMap::new())),
        data_streams: Arc::new(RwLock::new(HashMap::new())),
        chat_rooms: Arc::new(RwLock::new(HashMap::new())),
    };
}

/// WebSocket升级处理器
pub async fn websocket_handler(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
) -> Response {
    ws.on_upgrade(|socket| handle_websocket(socket, state))
}

/// 处理WebSocket连接
async fn handle_websocket(socket: WebSocket, _state: AppState) {
    let connection_id = Uuid::new_v4().to_string();
    info!("WebSocket 连接建立: {}", connection_id);
    
    // 注册连接
    let connection = ConnectionInfo {
        id: connection_id.clone(),
        connected_at: chrono::Utc::now(),
        last_activity: chrono::Utc::now(),
        message_count: 0,
        subscriptions: Vec::new(),
    };
    
    WS_STATE.connections.write().await.insert(connection_id.clone(), connection);
    
    let (mut sender, mut receiver) = socket.split();
    
    // 发送欢迎消息
    let welcome_response = JsonRpcResponse::success(
        serde_json::Value::String("welcome".to_string()),
        json!({
            "message": "WebSocket 连接已建立",
            "connection_id": connection_id,
            "server": "JsonRPC Playground",
            "protocol": "JsonRPC 2.0",
            "timestamp": chrono::Utc::now()
        })
    );
    
    if let Ok(welcome_msg) = serde_json::to_string(&welcome_response) {
        if sender.send(Message::Text(welcome_msg)).await.is_err() {
            error!("发送欢迎消息失败");
            return;
        }
    }
    
    // 处理消息循环
    while let Some(msg) = receiver.next().await {
        match msg {
            Ok(Message::Text(text)) => {
                debug!("收到消息: {}", text);
                
                // 更新连接活动时间
                if let Some(connection) = WS_STATE.connections.write().await.get_mut(&connection_id) {
                    connection.last_activity = chrono::Utc::now();
                    connection.message_count += 1;
                }
                
                // 处理JsonRPC请求
                if let Some(response_text) = handle_jsonrpc_message(&connection_id, &text).await {
                    if sender.send(Message::Text(response_text)).await.is_err() {
                        error!("发送响应失败");
                        break;
                    }
                }
            }
            Ok(Message::Close(_)) => {
                info!("WebSocket 连接关闭: {}", connection_id);
                break;
            }
            Err(e) => {
                error!("WebSocket 错误: {}", e);
                break;
            }
            _ => {}
        }
    }
    
    // 清理连接
    cleanup_connection(&connection_id).await;
}

/// 处理JsonRPC消息
async fn handle_jsonrpc_message(connection_id: &str, text: &str) -> Option<String> {
    // 解析JsonRPC请求
    let request: JsonRpcRequest = match serde_json::from_str(text) {
        Ok(req) => req,
        Err(e) => {
            error!("解析JsonRPC请求失败: {}", e);
            let error_response = JsonRpcResponse::error(
                serde_json::Value::Null,
                JsonRpcError::parse_error("Invalid JSON-RPC request")
            );
            return serde_json::to_string(&error_response).ok();
        }
    };
    
    let response = process_websocket_request(connection_id, request).await;
    serde_json::to_string(&response).ok()
}

/// 处理WebSocket JsonRPC请求
async fn process_websocket_request(connection_id: &str, request: JsonRpcRequest) -> JsonRpcResponse {
    let method = request.method();
    let params = request.params.clone().unwrap_or(Value::Null);
    let request_id = request.id().cloned().unwrap_or(Value::Null);
    
    info!("WebSocket 处理方法: {} [连接: {}]", method, connection_id);
    
    let result = match method {
        // WebSocket特定方法
        "ws.ping" => handle_ping().await,
        "ws.status" => handle_connection_status(connection_id).await,
        "ws.subscribe" => handle_subscription(connection_id, params).await,
        "ws.unsubscribe" => handle_unsubscription(connection_id, params).await,
        
        // 数据流控制
        "stream.data" => handle_data_stream(connection_id, params).await,
        "stream.chat" => handle_chat_stream(connection_id, params).await,
        
        // 实时聊天
        "chat.join" => handle_chat_join(connection_id, params).await,
        "chat.send" => handle_chat_send(connection_id, params).await,
        "chat.leave" => handle_chat_leave(connection_id, params).await,
        
        _ => Err(anyhow::anyhow!("Unknown WebSocket method: {}", method))
    };
    
    match result {
        Ok(result_value) => JsonRpcResponse::success(request_id, result_value),
        Err(err) => {
            error!("WebSocket方法执行错误: {}", err);
            JsonRpcResponse::error(
                request_id,
                JsonRpcError::internal_error(&format!("Method execution failed: {}", err))
            )
        }
    }
}

/// 处理Ping请求
async fn handle_ping() -> anyhow::Result<Value> {
    Ok(json!({"pong": chrono::Utc::now()}))
}

/// 处理连接状态请求
async fn handle_connection_status(connection_id: &str) -> anyhow::Result<Value> {
    let connections = WS_STATE.connections.read().await;
    let connection_info = connections.get(connection_id).ok_or_else(|| anyhow::anyhow!("Connection not found"))?;
    
    Ok(json!({
        "id": connection_info.id,
        "connected_at": connection_info.connected_at,
        "last_activity": connection_info.last_activity,
        "message_count": connection_info.message_count,
        "subscriptions": connection_info.subscriptions
    }))
}

/// 处理订阅
async fn handle_subscription(connection_id: &str, params: Value) -> anyhow::Result<Value> {
    let subscription_type = params.get("type")
        .and_then(|t| t.as_str())
        .ok_or_else(|| anyhow::anyhow!("Missing type parameter"))?;
    
    let subscription_id = Uuid::new_v4().to_string();
    
    match subscription_type {
        "data_stream" => {
            let interval_ms = params.get("interval_ms")
                .and_then(|i| i.as_u64())
                .unwrap_or(1000);
            
                         let stream_id = format!("{}_{}", connection_id, subscription_id);
             let stream_id_clone = stream_id.clone();
             let (tx, mut rx) = mpsc::unbounded_channel();
             
             let stream = DataStream {
                 id: stream_id.clone(),
                 connection_id: connection_id.to_string(),
                 interval_ms,
                 sender: tx,
             };
             
             WS_STATE.data_streams.write().await.insert(stream_id.clone(), stream);
             
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
                 "subscription_id": subscription_id,
                 "stream_id": stream_id,
                 "status": "started",
                 "interval_ms": interval_ms,
                 "message": "Data stream subscription started"
             }))
        }
        "chat_room" => {
            let room_name = params.get("room")
                .and_then(|r| r.as_str())
                .unwrap_or("general");
            
            let room = ChatRoom {
                name: room_name.to_string(),
                members: vec![connection_id.to_string()],
                created_at: chrono::Utc::now(),
            };
            
            WS_STATE.chat_rooms.write().await.insert(room_name.to_string(), room);
            
            Ok(json!({
                "subscription_id": subscription_id,
                "room": room_name,
                "status": "started",
                "message": "Chat room subscription started"
            }))
        }
        _ => Err(anyhow::anyhow!("Unknown subscription type: {}", subscription_type))
    }
}

/// 处理取消订阅
async fn handle_unsubscription(connection_id: &str, params: Value) -> anyhow::Result<Value> {
    let subscription_id = params.get("subscription_id")
        .and_then(|id| id.as_str())
        .ok_or_else(|| anyhow::anyhow!("Missing subscription_id parameter"))?;
    
    let subscription_type = params.get("type")
        .and_then(|t| t.as_str())
        .ok_or_else(|| anyhow::anyhow!("Missing type parameter"))?;
    
    match subscription_type {
        "data_stream" => {
            let stream_id = format!("{}_{}", connection_id, subscription_id);
            let _ = stop_data_stream(&stream_id).await;
            Ok(json!({
                "subscription_id": subscription_id,
                "stream_id": stream_id,
                "status": "stopped",
                "message": "Data stream subscription stopped"
            }))
        }
        "chat_room" => {
            let room_name = params.get("room")
                .and_then(|r| r.as_str())
                .ok_or_else(|| anyhow::anyhow!("Missing room parameter"))?;
            
            let mut rooms = WS_STATE.chat_rooms.write().await;
            if let Some(room) = rooms.get_mut(room_name) {
                room.members.retain(|id| id != connection_id);
                if room.members.is_empty() {
                    rooms.remove(room_name);
                }
            }
            Ok(json!({
                "subscription_id": subscription_id,
                "room": room_name,
                "status": "stopped",
                "message": "Chat room subscription stopped"
            }))
        }
        _ => Err(anyhow::anyhow!("Unknown subscription type for unsubscription: {}", subscription_type))
    }
}

/// 处理数据流
async fn handle_data_stream(connection_id: &str, params: Value) -> anyhow::Result<Value> {
    let action = params.get("action")
        .and_then(|t| t.as_str())
        .ok_or_else(|| anyhow::anyhow!("Missing action parameter"))?;
    
    match action {
        "start" => {
            let interval_ms = params.get("interval_ms")
                .and_then(|i| i.as_u64())
                .unwrap_or(1000);
            
            start_data_stream(connection_id, interval_ms).await
        }
        "stop" => {
            let stream_id = params.get("stream_id")
                .and_then(|id| id.as_str())
                .ok_or_else(|| anyhow::anyhow!("Missing stream_id parameter"))?;
            
            stop_data_stream(stream_id).await
        }
        _ => Err(anyhow::anyhow!("Invalid data stream action: {}", action))
    }
}

/// 处理聊天流
async fn handle_chat_stream(connection_id: &str, params: Value) -> anyhow::Result<Value> {
    let action = params.get("action")
        .and_then(|t| t.as_str())
        .ok_or_else(|| anyhow::anyhow!("Missing action parameter"))?;
    
    match action {
        "start" => {
            let room_name = params.get("room")
                .and_then(|r| r.as_str())
                .unwrap_or("general");
            
            Ok(json!({
                "status": "started",
                "room": room_name,
                "connection_id": connection_id,
                "message": "Chat stream started"
            }))
        }
        "stop" => {
            Ok(json!({
                "status": "stopped",
                "connection_id": connection_id,
                "message": "Chat stream stopped"
            }))
        }
        _ => Err(anyhow::anyhow!("Invalid chat stream action: {}", action))
    }
}

/// 处理加入聊天室
async fn handle_chat_join(connection_id: &str, params: Value) -> anyhow::Result<Value> {
    let room_name = params.get("room")
        .and_then(|r| r.as_str())
        .ok_or_else(|| anyhow::anyhow!("Missing room parameter"))?;
    
    let username = params.get("username")
        .and_then(|u| u.as_str())
        .unwrap_or("Anonymous");
    
    let mut rooms = WS_STATE.chat_rooms.write().await;
    let room = rooms.entry(room_name.to_string()).or_insert_with(|| {
        ChatRoom {
            name: room_name.to_string(),
            members: Vec::new(),
            created_at: chrono::Utc::now(),
        }
    });
    
    if !room.members.contains(&connection_id.to_string()) {
        room.members.push(connection_id.to_string());
    }
    
    Ok(json!({
        "status": "joined",
        "room": room_name,
        "username": username,
        "member_count": room.members.len(),
        "message": format!("{} joined the room", username)
    }))
}

/// 处理发送聊天消息
async fn handle_chat_send(connection_id: &str, params: Value) -> anyhow::Result<Value> {
    let room_name = params.get("room")
        .and_then(|r| r.as_str())
        .ok_or_else(|| anyhow::anyhow!("Missing room parameter"))?;
    
    let message = params.get("message")
        .and_then(|m| m.as_str())
        .ok_or_else(|| anyhow::anyhow!("Missing message parameter"))?;
    
    let username = params.get("username")
        .and_then(|u| u.as_str())
        .unwrap_or("Anonymous");
    
    let rooms = WS_STATE.chat_rooms.read().await;
    let room = rooms.get(room_name)
        .ok_or_else(|| anyhow::anyhow!("Room not found"))?;
    
    if !room.members.contains(&connection_id.to_string()) {
        return Err(anyhow::anyhow!("Not a member of this room"));
    }
    
    Ok(json!({
        "status": "sent",
        "room": room_name,
        "username": username,
        "message": message,
        "timestamp": chrono::Utc::now(),
        "message_id": Uuid::new_v4()
    }))
}

/// 处理离开聊天室
async fn handle_chat_leave(connection_id: &str, params: Value) -> anyhow::Result<Value> {
    let room_name = params.get("room")
        .and_then(|r| r.as_str())
        .ok_or_else(|| anyhow::anyhow!("Missing room parameter"))?;
    
    let username = params.get("username")
        .and_then(|u| u.as_str())
        .unwrap_or("Anonymous");
    
    let mut rooms = WS_STATE.chat_rooms.write().await;
    if let Some(room) = rooms.get_mut(room_name) {
        room.members.retain(|id| id != connection_id);
        if room.members.is_empty() {
            rooms.remove(room_name);
        }
    }
    
    Ok(json!({
        "status": "left",
        "room": room_name,
        "username": username,
        "message": format!("{} left the room", username)
    }))
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