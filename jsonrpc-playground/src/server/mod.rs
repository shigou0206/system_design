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

// 使用 jsonrpc-rust 库的类型定义
use jsonrpc_rust::prelude::*;

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
#[derive(Debug, Clone, serde::Serialize)]
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
    Json(request_value): Json<Value>,
) -> Result<ResponseJson<Value>, StatusCode> {
    let start_time = std::time::Instant::now();
    
    debug!("收到 JsonRPC 请求: {}", serde_json::to_string_pretty(&request_value).unwrap_or_default());
    
    // 解析为 JsonRpcRequest
    let request: JsonRpcRequest = match serde_json::from_value(request_value) {
        Ok(req) => req,
        Err(err) => {
            error!("请求解析错误: {}", err);
            let error_response = JsonRpcResponse::error(
                serde_json::Value::Null,
                JsonRpcError::parse_error("Invalid JSON-RPC request format")
            );
            return Ok(ResponseJson(serde_json::to_value(error_response).unwrap()));
        }
    };
    
    // 处理请求
    let response = process_jsonrpc_request(&state, request).await;
    let duration = start_time.elapsed().as_millis() as u64;
    
    // 记录统计
    state.record_request(response.is_success(), duration).await;
    
    debug!("返回 JsonRPC 响应: {:?}", response);
    
    let response_value = serde_json::to_value(response).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(ResponseJson(response_value))
}

/// 处理JsonRPC请求
async fn process_jsonrpc_request(
    state: &AppState,
    request: JsonRpcRequest,
) -> JsonRpcResponse {
    let method = request.method();
    let params = request.params.clone().unwrap_or(Value::Null);
    let request_id = request.id().cloned().unwrap_or(Value::Null);
    
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
    };
    
    // 返回适当的响应
    match result {
        Ok(result_value) => JsonRpcResponse::success(request_id, result_value),
        Err(err) => {
            error!("方法执行错误: {}", err);
            JsonRpcResponse::error(
                request_id,
                JsonRpcError::internal_error(&format!("Method execution failed: {}", err))
            )
        }
    }
}

/// 获取系统统计信息
async fn get_system_stats(state: &AppState) -> anyhow::Result<Value> {
    let stats = state.stats.read().await.clone();
    let session_count = state.sessions.read().await.len();
    
    Ok(json!({
        "total_requests": stats.total_requests,
        "successful_requests": stats.successful_requests,
        "failed_requests": stats.failed_requests,
        "success_rate": if stats.total_requests > 0 {
            stats.successful_requests as f64 / stats.total_requests as f64 * 100.0
        } else {
            0.0
        },
        "average_response_time_ms": stats.average_response_time_ms,
        "active_sessions": session_count,
        "timestamp": chrono::Utc::now().to_rfc3339()
    }))
}

/// 获取活跃会话信息
async fn get_active_sessions(state: &AppState) -> anyhow::Result<Value> {
    let sessions = state.sessions.read().await;
    let session_list: Vec<_> = sessions.values().cloned().collect();
    
    Ok(json!({
        "count": session_list.len(),
        "sessions": session_list
    }))
}

/// 健康检查处理器
pub async fn health_handler(State(_state): State<AppState>) -> ResponseJson<Value> {
    ResponseJson(json!({
        "status": "ok",
        "service": "JsonRPC Playground",
        "version": env!("CARGO_PKG_VERSION"),
        "jsonrpc_version": jsonrpc_rust::JSONRPC_VERSION,
        "timestamp": chrono::Utc::now().to_rfc3339()
    }))
} 