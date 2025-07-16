//! JsonRPC Playground - Interactive Web Testing Platform
//! 
//! A comprehensive web-based playground for testing and demonstrating
//! the JsonRPC-Rust framework capabilities.

use axum::{
    extract::Query,
    routing::{get, post},
    Router,
    response::Html,
    http::StatusCode,
};
use tower::ServiceBuilder;
use tower_http::{
    services::ServeDir,
    cors::CorsLayer,
    trace::TraceLayer,
};
use tracing::{info, Level};
use tracing_subscriber;

mod server;
mod services;
mod websocket;
mod sse;
mod events;

use server::AppState;
use websocket::websocket_handler;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 初始化日志
    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .init();

    info!("🚀 启动 JsonRPC Playground");

    // 创建应用状态
    let app_state = AppState::new().await;

    // 构建路由
    let app = Router::new()
        // 主页
        .route("/", get(index_handler))
        
        // API路由
        .route("/api/jsonrpc", post(server::jsonrpc_handler))
        .route("/api/health", get(server::health_handler))
        
        // SSE路由
        .route("/api/sse", get(sse::sse_handler))
        .route("/api/sse/info", get(sse_info_handler))
        
        // Events API路由
        .route("/api/events/recent", get(events_recent_handler))
        .route("/api/events/stats", get(events_stats_handler))
        .route("/api/events/info", get(events_info_handler))
        
        // WebSocket路由
        .route("/ws", get(websocket_handler))
        
        // 静态文件服务
        .nest_service("/static", ServeDir::new("static"))
        
        // 中间件
        .layer(
            ServiceBuilder::new()
                .layer(TraceLayer::new_for_http())
                .layer(CorsLayer::permissive())
        )
        
        // 共享状态
        .with_state(app_state);

    // 启动服务器
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000").await?;
    
    info!("🌐 JsonRPC Playground 运行在 http://127.0.0.1:3000");
    info!("📡 WebSocket 端点: ws://127.0.0.1:3000/ws");
    info!("🔧 JsonRPC API: http://127.0.0.1:3000/api/jsonrpc");
    
    axum::serve(listener, app).await?;
    
    Ok(())
}

/// 主页处理器
async fn index_handler() -> Result<Html<String>, StatusCode> {
    let html = include_str!("../static/html/index.html");
    Ok(Html(html.to_string()))
}

/// SSE info handler
async fn sse_info_handler() -> axum::Json<serde_json::Value> {
    axum::Json(sse::get_sse_info().await)
}

/// Events recent handler
async fn events_recent_handler(
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> axum::Json<serde_json::Value> {
    let limit = params.get("limit")
        .and_then(|s| s.parse::<usize>().ok());
    
    let events = events::GLOBAL_EVENT_BUS.get_recent_events(limit).await;
    axum::Json(serde_json::json!({
        "events": events,
        "count": events.len()
    }))
}

/// Events stats handler
async fn events_stats_handler() -> axum::Json<serde_json::Value> {
    axum::Json(events::GLOBAL_EVENT_BUS.get_event_stats().await)
}

/// Events info handler
async fn events_info_handler() -> axum::Json<serde_json::Value> {
    axum::Json(events::get_events_info().await)
} 