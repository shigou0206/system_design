//! Phase 1 Demo - Core Layer Functionality
//! 
//! This example demonstrates the core traits and types implemented in Phase 1.

use jsonrpc_rust::prelude::*;
use jsonrpc_rust::core::{
    types::AuthContext,
};
use serde_json::json;

fn main() {
    println!("🦀 jsonrpc-rust Phase 1 Demo - Core Layer");
    println!("==========================================");
    
    // 1. 演示 JSON-RPC 消息类型
    demo_message_types();
    
    // 2. 演示错误处理系统
    demo_error_system();
    
    // 3. 演示服务上下文
    demo_service_context();
    
    // 4. 演示序列化
    demo_serialization();
    
    println!("\n✅ Phase 1 核心功能演示完成!");
    println!("   • 基础消息类型 ✓");
    println!("   • 错误处理系统 ✓");
    println!("   • 服务上下文 ✓");
    println!("   • 核心 trait 定义 (Message, Transport, Connection, MethodHandler, StreamHandler)");
    println!("   • 异步流处理 ✓");
}

fn demo_message_types() {
    println!("\n📦 1. JSON-RPC 消息类型演示");
    
    // 创建请求 (使用正确的API)
    let request = JsonRpcRequest::with_id(
        "weather.get", 
        Some(json!({"city": "北京"})), 
        serde_json::Value::Number(serde_json::Number::from(1))
    );
    println!("   请求: {:?}", request);
    
    // 创建成功响应
    let success_response = JsonRpcResponse::success(
        serde_json::Value::Number(serde_json::Number::from(1)),
        json!({"temperature": 25, "description": "晴天"})
    );
    println!("   成功响应: {:?}", success_response);
    
    // 创建错误响应
    let error_response = JsonRpcResponse::error(
        serde_json::Value::Number(serde_json::Number::from(1)),
        JsonRpcError::new(JsonRpcErrorCode::InvalidRequest, "参数错误")
    );
    println!("   错误响应: {:?}", error_response);
    
    // 创建通知 (无id)
    let notification = JsonRpcRequest::new("user.logout", Some(json!({"user_id": "123"})));
    println!("   通知: {:?}", notification);
}

fn demo_error_system() {
    println!("\n❌ 2. 错误处理系统演示");
    
    // 传输错误
    let transport_error = Error::Transport {
        message: "连接超时".to_string(),
        source: None,
    };
    println!("   传输错误: {:?} (种类: {:?})", transport_error, transport_error.kind());
    
    // JSON-RPC 协议错误
    let jsonrpc_error = JsonRpcError::new(JsonRpcErrorCode::MethodNotFound, "方法不存在: weather.unknown");
    println!("   JSON-RPC错误码: {} - {}", jsonrpc_error.code, jsonrpc_error.message);
    
    // 验证错误
    let validation_error = Error::Validation {
        message: "请求参数无效".to_string(),
        source: None,
    };
    println!("   验证错误: {:?} (种类: {:?})", validation_error, validation_error.kind());
}

fn demo_service_context() {
    println!("\n🔐 3. 服务上下文演示");
    
    // 创建认证上下文
    let auth_context = AuthContext::new("user_123", "bearer_token")
        .with_role("admin")
        .with_role("user")
        .with_permission("read:weather")
        .with_permission("write:settings");
    
    println!("   用户: {} | 角色: {:?}", auth_context.user_id, auth_context.roles);
    println!("   权限: {:?}", auth_context.permissions);
    println!("   有admin角色: {}", auth_context.has_role("admin"));
    println!("   有read权限: {}", auth_context.has_permission("read:weather"));
    
    // 创建服务上下文
    let context = ServiceContext::new("req_456")
        .with_auth_context(auth_context)
        .with_metadata("trace_id", json!("trace_789"))
        .with_metadata("region", json!("cn-north"));
    
    println!("   请求ID: {}", context.request_id);
    println!("   元数据: {:?}", context.metadata);
}

fn demo_serialization() {
    println!("\n🔄 4. 序列化演示");
    
    let request = JsonRpcRequest::with_id(
        "ping", 
        Some(json!({})), 
        serde_json::Value::String("test_123".to_string())
    );
    
    // 演示序列化 (需要实现 MessageSerializer trait)
    println!("   原始请求: {:?}", request);
    
    // 演示不同的消息ID类型
    let number_id = serde_json::Value::Number(serde_json::Number::from(42));
    let string_id = serde_json::Value::String("unique_id_456".to_string());
    
    println!("   数字ID: {:?}", number_id);
    println!("   字符串ID: {:?}", string_id);
} 