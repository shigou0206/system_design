//! Bidirectional Stream Demo
//! 
//! This example demonstrates how to use the BidirectionalStream feature
//! for real-time bidirectional communication between JSON-RPC peers.

use jsonrpc_rust::prelude::*;
use jsonrpc_rust::core::types::ChannelBidirectionalStream;
use jsonrpc_rust::core::traits::BidirectionalStream;
use serde_json::json;
use tokio::time::{sleep, Duration};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔄 jsonrpc-rust Bidirectional Stream Demo");
    println!("==========================================");
    
    // Create a bidirectional stream pair
    let (mut client_stream, mut server_peer) = ChannelBidirectionalStream::new();
    
    // Add metadata to identify the session
    let client_stream = client_stream.with_metadata("session_id", json!("demo_session_123"));
    
    println!("\n📡 1. Setting up bidirectional communication");
    println!("   Client and server streams created");
    println!("   Session ID: {:?}", client_stream.get_metadata("session_id"));
    
    // Spawn server task to handle incoming requests
    let server_handle = tokio::spawn(async move {
        println!("\n🖥️  Server: Starting to listen for requests...");
        
        while server_peer.is_open() {
            match server_peer.receive_request().await {
                Ok(request) => {
                    println!("🖥️  Server: Received request: {}", request.method);
                    
                    let response = match request.method.as_str() {
                        "ping" => {
                            JsonRpcResponse::success(
                                request.id.clone().unwrap_or(json!(null)),
                                json!({"message": "pong", "timestamp": chrono::Utc::now().timestamp()})
                            )
                        }
                        "echo" => {
                            JsonRpcResponse::success(
                                request.id.clone().unwrap_or(json!(null)),
                                request.params.unwrap_or(json!(null))
                            )
                        }
                        "get_status" => {
                            JsonRpcResponse::success(
                                request.id.clone().unwrap_or(json!(null)),
                                json!({
                                    "status": "healthy",
                                    "uptime": "5 minutes",
                                    "connections": 1
                                })
                            )
                        }
                        _ => {
                            JsonRpcResponse::error(
                                request.id.clone().unwrap_or(json!(null)),
                                JsonRpcError::new(JsonRpcErrorCode::MethodNotFound, "Unknown method")
                            )
                        }
                    };
                    
                    if let Err(e) = server_peer.send_response(response).await {
                        eprintln!("🖥️  Server error sending response: {}", e);
                        break;
                    }
                }
                Err(e) => {
                    println!("🖥️  Server: Error receiving request: {}", e);
                    break;
                }
            }
        }
        
        println!("🖥️  Server: Shutting down");
    });
    
    // Client operations
    let mut client_stream = client_stream;
    
    println!("\n📱 2. Client sending requests");
    
    // Send ping request
    let ping_request = JsonRpcRequest::with_id(
        "ping", 
        Some(json!({"client": "demo"})), 
        json!(1)
    );
    
    client_stream.send(ping_request).await?;
    println!("📱 Client: Sent ping request");
    
    if let Ok(response) = client_stream.receive().await {
        println!("📱 Client: Received ping response: {:?}", response.result);
    }
    
    // Send echo request
    let echo_request = JsonRpcRequest::with_id(
        "echo", 
        Some(json!({"message": "Hello, bidirectional world!", "data": [1, 2, 3]})), 
        json!(2)
    );
    
    client_stream.send(echo_request).await?;
    println!("📱 Client: Sent echo request");
    
    if let Ok(response) = client_stream.receive().await {
        println!("📱 Client: Received echo response: {:?}", response.result);
    }
    
    // Send status request
    let status_request = JsonRpcRequest::with_id(
        "get_status", 
        None, 
        json!(3)
    );
    
    client_stream.send(status_request).await?;
    println!("📱 Client: Sent status request");
    
    if let Ok(response) = client_stream.receive().await {
        println!("📱 Client: Received status response: {:?}", response.result);
    }
    
    // Send unknown method to test error handling
    let unknown_request = JsonRpcRequest::with_id(
        "unknown_method", 
        Some(json!({"test": true})), 
        json!(4)
    );
    
    client_stream.send(unknown_request).await?;
    println!("📱 Client: Sent unknown method request");
    
    if let Ok(response) = client_stream.receive().await {
        if let Some(error) = response.error {
            println!("📱 Client: Received error response: {} - {}", error.code, error.message);
        }
    }
    
    println!("\n⏳ 3. Demonstrating concurrent operations");
    
    // Spawn a task to send multiple requests concurrently
    let client_handle = tokio::spawn(async move {
        for i in 5..8 {
            let request = JsonRpcRequest::with_id(
                "ping", 
                Some(json!({"batch": i})), 
                json!(i)
            );
            
            if client_stream.send(request).await.is_err() {
                break;
            }
            
            if let Ok(response) = client_stream.receive().await {
                println!("📱 Client: Batch response {}: {:?}", i, response.result);
            }
            
            sleep(Duration::from_millis(100)).await;
        }
        
        // Close the client stream
        client_stream.close().await.unwrap();
        println!("📱 Client: Stream closed");
    });
    
    // Wait for both tasks to complete
    let (client_result, server_result) = tokio::join!(client_handle, server_handle);
    
    client_result?;
    server_result?;
    
    println!("\n✅ Bidirectional stream demo completed!");
    println!("   • Real-time bidirectional communication ✓");
    println!("   • Request/response pattern ✓");
    println!("   • Error handling ✓");
    println!("   • Concurrent operations ✓");
    println!("   • Graceful shutdown ✓");
    
    Ok(())
} 