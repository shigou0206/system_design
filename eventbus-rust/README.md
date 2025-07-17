# EventBus JSON-RPC Integration

EventBus的JSON-RPC服务端和客户端集成，基于[jsonrpc-rust](../jsonrpc-rust)框架实现。

## 🚀 功能特点

- **🌐 网络化EventBus**: 将EventBus功能暴露为JSON-RPC网络服务
- **📡 客户端-服务端架构**: 支持远程EventBus操作
- **🔄 完整API支持**: 包含所有EventBus核心功能
- **📦 批量操作**: 支持批量事件发送
- **🔔 订阅管理**: 支持事件订阅和轮询
- **📊 统计信息**: 提供实时服务统计
- **🔒 类型安全**: 强类型的JSON-RPC接口

## 📋 支持的方法

### 事件操作
- `eventbus.emit` - 发送单个事件
- `eventbus.emit_batch` - 批量发送事件
- `eventbus.poll` - 查询事件

### 订阅管理
- `eventbus.subscribe` - 订阅主题
- `eventbus.unsubscribe` - 取消订阅
- `eventbus.get_subscription_events` - 获取订阅事件

### 信息查询
- `eventbus.list_topics` - 列出可用主题
- `eventbus.get_stats` - 获取服务统计

## 🛠️ 使用方法

### 启动服务端

```bash
# 在默认地址启动
cargo run --bin eventbus-server

# 在指定地址启动
cargo run --bin eventbus-server 0.0.0.0:9000
```

### 客户端使用

```rust
use eventbus_rust::prelude::*;
use eventbus_rust::jsonrpc::connect_to_eventbus;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // 连接到EventBus服务器
    let client = connect_to_eventbus("127.0.0.1:8080").await?;

    // 发送事件
    let event = EventEnvelopeBuilder::new()
        .topic("trn:user:demo:tool:my-app:v1.0")
        .source_trn("trn:user:demo:tool:client:v1.0")
        .payload_json(serde_json::json!({"message": "Hello EventBus!"}))
        .metadata(serde_json::json!({"client": "demo"}))
        .build()?;

    client.emit(event).await?;

    // 获取统计信息
    let stats = client.get_stats().await?;
    println!("Events processed: {}", stats.events_processed);

    Ok(())
}
```

### 批量操作示例

```rust
// 批量发送事件
let events = vec![
    EventEnvelopeBuilder::new()
        .topic("trn:user:demo:tool:batch-test:v1.0")
        .source_trn("trn:user:demo:tool:client:v1.0")
        .payload_json(serde_json::json!({"id": 1}))
        .build()?,
    EventEnvelopeBuilder::new()
        .topic("trn:user:demo:tool:batch-test:v1.0")
        .source_trn("trn:user:demo:tool:client:v1.0")
        .payload_json(serde_json::json!({"id": 2}))
        .build()?,
];

let processed_count = client.emit_batch(events).await?;
println!("Processed {} events", processed_count);
```

### 订阅和事件轮询

```rust
// 订阅主题
let subscription = client
    .subscribe("trn:user:demo:tool:notifications:v1.0", Some("my-client".to_string()))
    .await?;

// 轮询订阅的事件
let events = client
    .get_subscription_events(&subscription, Some(10), Some(5000))
    .await?;

for event in events {
    println!("Received event: {}", event.id);
}

// 取消订阅
client.unsubscribe(&subscription).await?;
```

## 📊 JSON-RPC方法参考

### `eventbus.emit`

发送单个事件。

**参数:**
```json
{
  "event": {
    "topic": "trn:user:demo:tool:my-topic:v1.0",
    "payload": {"data": "value"},
    "metadata": {"source": "client"},
    "source_trn": "trn:user:demo:tool:client:v1.0"
  }
}
```

**响应:**
```json
{
  "success": true
}
```

### `eventbus.get_stats`

获取EventBus服务统计信息。

**响应:**
```json
{
  "stats": {
    "events_processed": 1250,
    "topic_count": 15,
    "active_subscriptions": 3,
    "events_per_second": 12.5,
    "uptime_seconds": 3600,
    "memory_usage": {
      "events_in_memory": 1000,
      "estimated_bytes": 512000
    }
  }
}
```

## 🧪 测试

运行集成测试：

```bash
cargo test --test jsonrpc_integration_test
```

运行所有测试：

```bash
cargo test
```

## 🏗️ 架构

```
┌─────────────────┐    JSON-RPC     ┌─────────────────┐
│   EventBus      │◄───────────────►│   EventBus      │
│   Client        │                 │   RPC Server    │
└─────────────────┘                 └─────────────────┘
                                            │
                                            ▼
                                    ┌─────────────────┐
                                    │   EventBus      │
                                    │   Service       │
                                    └─────────────────┘
```

## 🔧 配置

服务端配置通过`ServiceConfig`完成：

```rust
let config = ServiceConfig {
    instance_id: "production-eventbus".to_string(),
    max_memory_events: 10000,
    max_events_per_second: Some(1000),
    enable_metrics: true,
    ..Default::default()
};
```

## ⚠️ 注意事项

1. **TRN格式**: 所有主题(topic)必须使用有效的TRN格式
2. **事件负载**: 事件必须包含payload字段
3. **网络连接**: 目前使用mock transport，实际网络传输需要等待jsonrpc-rust完善
4. **并发限制**: 服务端支持配置最大并发连接数和速率限制

## 🚧 开发状态

- ✅ JSON-RPC方法定义
- ✅ 服务端handler实现  
- ✅ 客户端API实现
- ✅ 基础测试覆盖
- ⏳ 真实网络传输 (等待jsonrpc-rust完善)
- ⏳ WebSocket订阅支持
- ⏳ TLS/SSL支持

## 📝 示例

查看完整示例：
- [客户端示例](examples/jsonrpc_client_demo.rs)
- [服务端启动](src/bin/eventbus-server.rs)
- [集成测试](tests/jsonrpc_integration_test.rs) 