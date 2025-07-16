//! 演示服务模块
//! 
//! 提供各种类型的JsonRPC服务实现，展示框架的功能和用法

use serde_json::{Value, json};
use uuid::Uuid;
use tracing::{info, debug};

/// 演示服务集合
pub struct DemoServices {
    // 这里可以添加服务特定的状态
}

impl DemoServices {
    /// 创建新的演示服务集合
    pub async fn new() -> Self {
        info!("初始化演示服务...");
        
        Self {}
    }
    
    /// 获取系统信息
    pub async fn get_system_info(&self) -> anyhow::Result<Value> {
        Ok(json!({
            "name": "JsonRPC Playground",
            "version": env!("CARGO_PKG_VERSION"),
            "description": "Interactive testing platform for JsonRPC-Rust framework",
            "framework": {
                "name": "jsonrpc-rust",
                "features": [
                    "Bidirectional streaming",
                    "Error handling",
                    "Type safety",
                    "Async support",
                    "WebSocket transport"
                ]
            },
            "available_methods": [
                "system.info",
                "system.stats", 
                "system.sessions",
                "math.add",
                "math.multiply",
                "math.fibonacci",
                "tools.echo",
                "tools.timestamp",
                "tools.uuid",
                "stream.data",
                "stream.chat"
            ],
            "timestamp": chrono::Utc::now()
        }))
    }
    
    // === 数学计算服务 ===
    
    /// 数学加法
    pub async fn math_add(&self, params: Value) -> anyhow::Result<Value> {
        debug!("执行数学加法: {}", params);
        
        let numbers = params.as_array()
            .ok_or_else(|| anyhow::anyhow!("参数必须是数字数组"))?;
        
        let mut sum = 0.0;
        for num in numbers {
            if let Some(n) = num.as_f64() {
                sum += n;
            } else {
                return Err(anyhow::anyhow!("无效的数字: {}", num));
            }
        }
        
        Ok(json!({
            "result": sum,
            "operation": "addition",
            "operands": numbers,
            "timestamp": chrono::Utc::now()
        }))
    }
    
    /// 数学乘法
    pub async fn math_multiply(&self, params: Value) -> anyhow::Result<Value> {
        debug!("执行数学乘法: {}", params);
        
        let obj = params.as_object()
            .ok_or_else(|| anyhow::anyhow!("参数必须是对象"))?;
            
        let a = obj.get("a")
            .and_then(|v| v.as_f64())
            .ok_or_else(|| anyhow::anyhow!("缺少参数 a"))?;
            
        let b = obj.get("b")
            .and_then(|v| v.as_f64())
            .ok_or_else(|| anyhow::anyhow!("缺少参数 b"))?;
        
        let result = a * b;
        
        Ok(json!({
            "result": result,
            "operation": "multiplication",
            "operands": {"a": a, "b": b},
            "timestamp": chrono::Utc::now()
        }))
    }
    
    /// 计算斐波那契数列
    pub async fn math_fibonacci(&self, params: Value) -> anyhow::Result<Value> {
        debug!("计算斐波那契数列: {}", params);
        
        let n = params.get("n")
            .or_else(|| params.as_u64().map(|_| &params))
            .and_then(|v| v.as_u64())
            .ok_or_else(|| anyhow::anyhow!("需要参数 n (正整数)"))?;
        
        if n > 100 {
            return Err(anyhow::anyhow!("n 不能超过 100"));
        }
        
        let sequence = calculate_fibonacci(n as usize);
        let result = if n > 0 { sequence.last().copied().unwrap_or(0) } else { 0 };
        
        Ok(json!({
            "result": result,
            "n": n,
            "sequence": sequence,
            "operation": "fibonacci",
            "timestamp": chrono::Utc::now()
        }))
    }
    
    // === 工具服务 ===
    
    /// 回显服务
    pub async fn tools_echo(&self, params: Value) -> anyhow::Result<Value> {
        debug!("执行回显: {}", params);
        
        Ok(json!({
            "echo": params,
            "timestamp": chrono::Utc::now(),
            "message": "Echo service - returns your input"
        }))
    }
    
    /// 获取时间戳
    pub async fn tools_timestamp(&self) -> anyhow::Result<Value> {
        let now = chrono::Utc::now();
        
        Ok(json!({
            "timestamp": now,
            "unix_timestamp": now.timestamp(),
            "iso8601": now.to_rfc3339(),
            "timezone": "UTC"
        }))
    }
    
    /// 生成UUID
    pub async fn tools_uuid(&self) -> anyhow::Result<Value> {
        let uuid = Uuid::new_v4();
        
        Ok(json!({
            "uuid": uuid.to_string(),
            "version": 4,
            "variant": "RFC 4122",
            "timestamp": chrono::Utc::now()
        }))
    }
    
    // === 流式服务信息 ===
    
    /// 数据流服务信息
    pub async fn stream_data_info(&self) -> anyhow::Result<Value> {
        Ok(json!({
            "service": "stream.data",
            "description": "实时数据流服务",
            "transport": "WebSocket",
            "endpoint": "/ws",
            "message_format": {
                "jsonrpc": "2.0",
                "method": "stream.data",
                "params": {
                    "type": "start|stop",
                    "interval_ms": "数字，数据间隔毫秒数"
                }
            },
            "example": {
                "jsonrpc": "2.0",
                "method": "stream.data",
                "params": {"type": "start", "interval_ms": 1000},
                "id": "stream-1"
            }
        }))
    }
    
    /// 聊天流服务信息
    pub async fn stream_chat_info(&self) -> anyhow::Result<Value> {
        Ok(json!({
            "service": "stream.chat",
            "description": "实时聊天流服务",
            "transport": "WebSocket",
            "endpoint": "/ws",
            "message_format": {
                "jsonrpc": "2.0",
                "method": "stream.chat",
                "params": {
                    "action": "join|leave|message",
                    "room": "房间名",
                    "message": "消息内容 (仅用于message)"
                }
            },
            "example": {
                "jsonrpc": "2.0",
                "method": "stream.chat",
                "params": {"action": "join", "room": "general"},
                "id": "chat-1"
            }
        }))
    }
}

/// 计算斐波那契数列
fn calculate_fibonacci(n: usize) -> Vec<u64> {
    if n == 0 {
        return vec![];
    }
    if n == 1 {
        return vec![0];
    }
    
    let mut sequence = vec![0, 1];
    for i in 2..n {
        let next = sequence[i-1] + sequence[i-2];
        sequence.push(next);
    }
    
    sequence
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_fibonacci() {
        assert_eq!(calculate_fibonacci(0), Vec::<u64>::new());
        assert_eq!(calculate_fibonacci(1), vec![0]);
        assert_eq!(calculate_fibonacci(5), vec![0, 1, 1, 2, 3]);
        assert_eq!(calculate_fibonacci(10), vec![0, 1, 1, 2, 3, 5, 8, 13, 21, 34]);
    }
    
    #[tokio::test]
    async fn test_demo_services() {
        let services = DemoServices::new().await;
        
        // 测试系统信息
        let info = services.get_system_info().await.unwrap();
        assert!(info.get("name").is_some());
        
        // 测试数学加法
        let result = services.math_add(json!([1, 2, 3, 4])).await.unwrap();
        assert_eq!(result.get("result").unwrap().as_f64().unwrap(), 10.0);
    }
} 