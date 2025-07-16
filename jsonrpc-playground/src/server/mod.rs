//! 服务器核心模块
//! 
//! 提供应用状态管理、HTTP JsonRPC处理等核心服务器功能

use std::sync::Arc;
use std::collections::HashMap;
use axum::{
    extract::{State, Json},
    response::Json as ResponseJson,
    http::StatusCode,
};
use serde_json::{Value, json};
use tokio::sync::RwLock;
use uuid::Uuid;
use tracing::{info, error, debug};

use crate::services::DemoServices;

/// 应用全局状态
#[derive(Clone)]
pub struct AppState {
    /// 演示服务集合
    pub services: Arc<DemoServices>,
    /// 活跃会话记录
    pub sessions: Arc<RwLock<HashMap<String, SessionInfo>>>,
    /// 请求统计
    pub stats: Arc<RwLock<RequestStats>>,
}

/// 会话信息
#[derive(Debug, Clone)]
pub struct SessionInfo {
    pub id: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub last_activity: chrono::DateTime<chrono::Utc>,
    pub request_count: u64,
}

/// 请求统计
#[derive(Debug, Clone, Default)]
pub struct RequestStats {
    pub total_requests: u64,
    pub successful_requests: u64,
    pub failed_requests: u64,
    pub average_response_time_ms: f64,
}

impl AppState {
    /// 创建新的应用状态
    pub async fn new() -> Self {
        info!("初始化应用状态...");
        
        let services = Arc::new(DemoServices::new().await);
        let sessions = Arc::new(RwLock::new(HashMap::new()));
        let stats = Arc::new(RwLock::new(RequestStats::default()));
        
        info!("应用状态初始化完成");
        
        Self {
            services,
            sessions,
            stats,
        }
    }
    
    /// 创建新会话
    #[allow(dead_code)]
    pub async fn create_session(&self) -> String {
        let session_id = Uuid::new_v4().to_string();
        let now = chrono::Utc::now();
        
        let session = SessionInfo {
            id: session_id.clone(),
            created_at: now,
            last_activity: now,
            request_count: 0,
        };
        
        self.sessions.write().await.insert(session_id.clone(), session);
        debug!("创建新会话: {}", session_id);
        
        session_id
    }

    /// 更新会话活动
    #[allow(dead_code)]
    pub async fn update_session_activity(&self, session_id: &str) {
        if let Some(session) = self.sessions.write().await.get_mut(session_id) {
            session.last_activity = chrono::Utc::now();
            session.request_count += 1;
        }
    }
    
    /// 记录请求统计
    pub async fn record_request(&self, success: bool, response_time_ms: u64) {
        let mut stats = self.stats.write().await;
        stats.total_requests += 1;
        
        if success {
            stats.successful_requests += 1;
        } else {
            stats.failed_requests += 1;
        }
        
        // 简单的移动平均
        stats.average_response_time_ms = 
            (stats.average_response_time_ms * (stats.total_requests - 1) as f64 + response_time_ms as f64) 
            / stats.total_requests as f64;
    }
}

/// HTTP JsonRPC 请求处理器
pub async fn jsonrpc_handler(
    State(state): State<AppState>,
    Json(request): Json<Value>,
) -> Result<ResponseJson<Value>, StatusCode> {
    let start_time = std::time::Instant::now();
    
    debug!("收到 JsonRPC 请求: {}", serde_json::to_string_pretty(&request).unwrap_or_default());
    
    // 解析请求
    let response = match process_jsonrpc_request(&state, request).await {
        Ok(resp) => {
            let duration = start_time.elapsed().as_millis() as u64;
            state.record_request(true, duration).await;
            resp
        }
        Err(err) => {
            error!("JsonRPC 请求处理错误: {}", err);
            let duration = start_time.elapsed().as_millis() as u64;
            state.record_request(false, duration).await;
            
            json!({
                "jsonrpc": "2.0",
                "error": {
                    "code": -32603,
                    "message": "Internal error",
                    "data": err.to_string()
                },
                "id": null
            })
        }
    };
    
    debug!("返回 JsonRPC 响应: {}", serde_json::to_string_pretty(&response).unwrap_or_default());
    
    Ok(ResponseJson(response))
}

/// 处理JsonRPC请求
async fn process_jsonrpc_request(
    state: &AppState,
    request: Value,
) -> anyhow::Result<Value> {
    // 验证JsonRPC格式
    let method = request.get("method")
        .and_then(|m| m.as_str())
        .ok_or_else(|| anyhow::anyhow!("Missing method field"))?;
    
    let params = request.get("params").cloned().unwrap_or(Value::Null);
    let id = request.get("id").cloned().unwrap_or(Value::Null);
    
    info!("处理方法: {} with params: {}", method, params);
    
    // 路由到对应的服务
    let result = match method {
        // 系统方法
        "system.info" => state.services.get_system_info().await,
        "system.stats" => get_system_stats(state).await,
        "system.sessions" => get_active_sessions(state).await,
        
        // 数学计算服务
        "math.add" => state.services.math_add(params).await,
        "math.multiply" => state.services.math_multiply(params).await,
        "math.fibonacci" => state.services.math_fibonacci(params).await,
        
        // 工具服务
        "tools.echo" => state.services.tools_echo(params).await,
        "tools.timestamp" => state.services.tools_timestamp().await,
        "tools.uuid" => state.services.tools_uuid().await,
        
        // 流式服务（这里返回初始响应，实际流式数据通过WebSocket）
        "stream.data" => state.services.stream_data_info().await,
        "stream.chat" => state.services.stream_chat_info().await,
        
        _ => Err(anyhow::anyhow!("Unknown method: {}", method))
    }?;
    
    Ok(json!({
        "jsonrpc": "2.0",
        "result": result,
        "id": id
    }))
}

/// 获取系统统计信息
async fn get_system_stats(state: &AppState) -> anyhow::Result<Value> {
    let stats = state.stats.read().await.clone();
    let session_count = state.sessions.read().await.len();
    
    Ok(json!({
        "total_requests": stats.total_requests,
        "successful_requests": stats.successful_requests,
        "failed_requests": stats.failed_requests,
        "average_response_time_ms": stats.average_response_time_ms,
        "active_sessions": session_count,
        "uptime_seconds": std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
    }))
}

/// 获取活跃会话信息
async fn get_active_sessions(state: &AppState) -> anyhow::Result<Value> {
    let sessions = state.sessions.read().await;
    let session_list: Vec<Value> = sessions.values()
        .map(|session| json!({
            "id": session.id,
            "created_at": session.created_at,
            "last_activity": session.last_activity,
            "request_count": session.request_count
        }))
        .collect();
    
    Ok(json!({
        "count": sessions.len(),
        "sessions": session_list
    }))
}

/// 健康检查处理器
pub async fn health_handler(
    State(state): State<AppState>,
) -> ResponseJson<Value> {
    let stats = state.stats.read().await.clone();
    
    ResponseJson(json!({
        "status": "healthy",
        "timestamp": chrono::Utc::now(),
        "version": env!("CARGO_PKG_VERSION"),
        "requests_handled": stats.total_requests,
        "success_rate": if stats.total_requests > 0 {
            stats.successful_requests as f64 / stats.total_requests as f64 * 100.0
        } else {
            100.0
        }
    }))
} 