//! Future and stream abstractions for the JSON-RPC framework
//!
//! This module provides async primitives with advanced features like priority scheduling,
//! backpressure control, cancellation support, and convenient chain operations.

use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};
use std::time::{Duration, Instant};
use std::sync::{Arc, atomic::{AtomicBool, Ordering}};
use std::collections::HashMap;

use futures::{Stream, StreamExt};
// use tokio::sync::{mpsc, oneshot, Semaphore};
use serde::{Deserialize, Serialize};

use crate::core::error::{Error, Result};
use crate::core::types::JsonRpcResponse;

/// Priority levels for futures and streams
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Priority {
    /// Lowest priority (best effort)
    Low = 0,
    /// Normal priority (default)
    Normal = 1,
    /// High priority (time-sensitive)
    High = 2,
    /// Critical priority (system operations)
    Critical = 3,
}

impl Default for Priority {
    fn default() -> Self {
        Priority::Normal
    }
}

impl Priority {
    /// Get numeric priority value (higher number = higher priority)
    pub fn value(self) -> u8 {
        self as u8
    }
    
    /// Check if this priority is higher than another
    pub fn is_higher_than(self, other: Priority) -> bool {
        self.value() > other.value()
    }
    
    /// Get all available priorities in ascending order
    pub fn all() -> &'static [Priority] {
        &[Priority::Low, Priority::Normal, Priority::High, Priority::Critical]
    }
}

/// Spawn policy for controlling how futures are executed
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpawnPolicy {
    /// Priority level for scheduling
    pub priority: Priority,
    /// Maximum execution time before timeout
    pub timeout: Option<Duration>,
    /// Whether to retry on failure
    pub retry_on_failure: bool,
    /// Maximum retry attempts
    pub max_retries: u32,
    /// Retry delay
    pub retry_delay: Duration,
    /// Resource limits
    pub resource_limits: ResourceLimits,
    /// Custom spawn configuration
    pub custom_config: HashMap<String, serde_json::Value>,
}

impl Default for SpawnPolicy {
    fn default() -> Self {
        Self {
            priority: Priority::Normal,
            timeout: Some(Duration::from_secs(30)),
            retry_on_failure: false,
            max_retries: 0,
            retry_delay: Duration::from_millis(100),
            resource_limits: ResourceLimits::default(),
            custom_config: HashMap::new(),
        }
    }
}

impl SpawnPolicy {
    /// Create a new spawn policy with default values
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Set priority level
    pub fn with_priority(mut self, priority: Priority) -> Self {
        self.priority = priority;
        self
    }
    
    /// Set timeout duration
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }
    
    /// Disable timeout
    pub fn without_timeout(mut self) -> Self {
        self.timeout = None;
        self
    }
    
    /// Enable retry on failure
    pub fn with_retry(mut self, max_retries: u32, delay: Duration) -> Self {
        self.retry_on_failure = true;
        self.max_retries = max_retries;
        self.retry_delay = delay;
        self
    }
    
    /// Set resource limits
    pub fn with_resource_limits(mut self, limits: ResourceLimits) -> Self {
        self.resource_limits = limits;
        self
    }
    
    /// Add custom configuration
    pub fn with_config(mut self, key: String, value: serde_json::Value) -> Self {
        self.custom_config.insert(key, value);
        self
    }
    
    /// Create a high-priority policy
    pub fn high_priority() -> Self {
        Self::new()
            .with_priority(Priority::High)
            .with_timeout(Duration::from_secs(10))
    }
    
    /// Create a critical policy
    pub fn critical() -> Self {
        Self::new()
            .with_priority(Priority::Critical)
            .with_timeout(Duration::from_secs(5))
            .with_retry(3, Duration::from_millis(50))
    }
    
    /// Create a background policy
    pub fn background() -> Self {
        Self::new()
            .with_priority(Priority::Low)
            .with_timeout(Duration::from_secs(120))
    }
}

/// Resource limits for spawn policy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceLimits {
    /// Maximum memory usage in bytes
    pub max_memory_bytes: Option<u64>,
    /// Maximum CPU time in milliseconds  
    pub max_cpu_time_ms: Option<u64>,
    /// Maximum number of file descriptors
    pub max_file_descriptors: Option<u32>,
    /// Maximum network bandwidth in bytes/sec
    pub max_network_bps: Option<u64>,
}

impl Default for ResourceLimits {
    fn default() -> Self {
        Self {
            max_memory_bytes: Some(256 * 1024 * 1024), // 256MB
            max_cpu_time_ms: Some(30_000), // 30 seconds
            max_file_descriptors: Some(64),
            max_network_bps: Some(10 * 1024 * 1024), // 10MB/s
        }
    }
}

/// Execution statistics for monitoring
#[derive(Debug, Clone, Default)]
pub struct ExecutionStats {
    /// Start time
    pub started_at: Option<Instant>,
    /// Completion time
    pub completed_at: Option<Instant>,
    /// Number of retry attempts
    pub retry_count: u32,
    /// Memory usage peak in bytes
    pub peak_memory_bytes: u64,
    /// CPU time used in milliseconds
    pub cpu_time_ms: u64,
    /// Network bytes transferred
    pub network_bytes: u64,
}

impl ExecutionStats {
    /// Get execution duration
    pub fn duration(&self) -> Option<Duration> {
        match (self.started_at, self.completed_at) {
            (Some(start), Some(end)) => Some(end.duration_since(start)),
            _ => None,
        }
    }
    
    /// Check if execution is complete
    pub fn is_complete(&self) -> bool {
        self.completed_at.is_some()
    }
    
    /// Record start time
    pub fn start(&mut self) {
        self.started_at = Some(Instant::now());
    }
    
    /// Record completion time
    pub fn complete(&mut self) {
        self.completed_at = Some(Instant::now());
    }
}

/// Enhanced JSON-RPC Future with priority and spawn policy support
pub struct JsonRpcFuture {
    inner: Pin<Box<dyn Future<Output = Result<JsonRpcResponse>> + Send>>,
    policy: SpawnPolicy,
    cancellation_token: Arc<AtomicBool>,
    stats: Arc<std::sync::Mutex<ExecutionStats>>,
}

impl JsonRpcFuture {
    /// Create a new JSON-RPC future
    pub fn new<F>(future: F) -> Self 
    where
        F: Future<Output = Result<JsonRpcResponse>> + Send + 'static,
    {
        Self {
            inner: Box::pin(future),
            policy: SpawnPolicy::default(),
            cancellation_token: Arc::new(AtomicBool::new(false)),
            stats: Arc::new(std::sync::Mutex::new(ExecutionStats::default())),
        }
    }
    
    /// Create future with spawn policy
    pub fn with_policy<F>(future: F, policy: SpawnPolicy) -> Self 
    where
        F: Future<Output = Result<JsonRpcResponse>> + Send + 'static,
    {
        let mut fut = Self::new(future);
        fut.policy = policy;
        fut
    }
    
    /// Set spawn policy
    pub fn set_policy(mut self, policy: SpawnPolicy) -> Self {
        self.policy = policy;
        self
    }
    
    /// Get spawn policy
    pub fn policy(&self) -> &SpawnPolicy {
        &self.policy
    }
    
    /// Get priority
    pub fn priority(&self) -> Priority {
        self.policy.priority
    }
    
    /// Check if cancelled
    pub fn is_cancelled(&self) -> bool {
        self.cancellation_token.load(Ordering::SeqCst)
    }
    
    /// Cancel the future
    pub fn cancel(&self) {
        self.cancellation_token.store(true, Ordering::SeqCst);
    }
    
    /// Get execution statistics
    pub fn stats(&self) -> ExecutionStats {
        self.stats.lock().unwrap().clone()
    }
    
    /// Box the future for type erasure
    pub fn boxed(self) -> Pin<Box<dyn Future<Output = Result<JsonRpcResponse>> + Send>> {
        Box::pin(self)
    }
    
    /// Map the result of the future
    pub fn map<F, T>(self, f: F) -> Pin<Box<dyn Future<Output = T> + Send>>
    where
        F: FnOnce(Result<JsonRpcResponse>) -> T + Send + 'static,
        T: Send + 'static,
    {
        Box::pin(async move {
            let result = self.await;
            f(result)
        })
    }
    
    /// Map the success result only
    pub fn map_ok<F>(self, f: F) -> Pin<Box<dyn Future<Output = Result<JsonRpcResponse>> + Send>>
    where
        F: FnOnce(JsonRpcResponse) -> JsonRpcResponse + Send + 'static,
    {
        Box::pin(async move {
            match self.await {
                Ok(response) => Ok(f(response)),
                Err(e) => Err(e),
            }
        })
    }
    
    /// Map the error result only
    pub fn map_err<F>(self, f: F) -> Pin<Box<dyn Future<Output = Result<JsonRpcResponse>> + Send>>
    where
        F: FnOnce(Error) -> Error + Send + 'static,
    {
        Box::pin(async move {
            match self.await {
                Ok(response) => Ok(response),
                Err(e) => Err(f(e)),
            }
        })
    }
    
    /// Add timeout to the future
    pub fn timeout(self, duration: Duration) -> Pin<Box<dyn Future<Output = Result<JsonRpcResponse>> + Send>> {
        Box::pin(async move {
            match tokio::time::timeout(duration, self).await {
                Ok(result) => result,
                Err(_) => Err(Error::timeout("JsonRpcFuture", duration)),
            }
        })
    }
    
    /// Retry the future on failure
    /// Note: This is a simplified implementation - in practice we'd need to recreate the future
    pub fn retry(self, max_attempts: u32, delay: Duration) -> Pin<Box<dyn Future<Output = Result<JsonRpcResponse>> + Send>> {
        Box::pin(async move {
            // This is a placeholder implementation
            // In a real retry system, we'd need a way to recreate the future
            // For now, just execute once and handle the error
            match self.await {
                Ok(response) => Ok(response),
                Err(e) => {
                    if max_attempts > 1 && e.is_retryable() {
                        tokio::time::sleep(delay).await;
                        // In a real implementation, we'd recreate and retry the future
                        Err(Error::service("Retry not fully implemented yet"))
                    } else {
                        Err(e)
                    }
                }
            }
        })
    }
    
    /// Spawn the future with high priority
    pub fn spawn_high_priority(self) -> tokio::task::JoinHandle<Result<JsonRpcResponse>> {
        tokio::task::spawn(self.set_policy(SpawnPolicy::high_priority()))
    }
    
    /// Spawn the future with critical priority
    pub fn spawn_critical(self) -> tokio::task::JoinHandle<Result<JsonRpcResponse>> {
        tokio::task::spawn(self.set_policy(SpawnPolicy::critical()))
    }
    
    /// Spawn the future in background
    pub fn spawn_background(self) -> tokio::task::JoinHandle<Result<JsonRpcResponse>> {
        tokio::task::spawn(self.set_policy(SpawnPolicy::background()))
    }
}

impl Future for JsonRpcFuture {
    type Output = Result<JsonRpcResponse>;
    
    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        // Check for cancellation
        if self.cancellation_token.load(Ordering::SeqCst) {
            return Poll::Ready(Err(Error::cancelled("JsonRpcFuture")));
        }
        
        // Update stats
        {
            let mut stats = self.stats.lock().unwrap();
            if stats.started_at.is_none() {
                stats.start();
            }
        }
        
        // Poll the inner future
        match self.inner.as_mut().poll(cx) {
            Poll::Ready(result) => {
                // Update completion stats
                {
                    let mut stats = self.stats.lock().unwrap();
                    stats.complete();
                }
                Poll::Ready(result)
            }
            Poll::Pending => Poll::Pending,
        }
    }
}

/// Backpressure signals for flow control
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BackpressureSignal {
    /// Normal flow - no backpressure
    None,
    /// Slow down processing
    SlowDown,
    /// Pause processing temporarily
    Pause,
    /// Drop messages due to overload
    Drop,
}

/// Stream control for managing flow and lifecycle
#[derive(Debug, Clone)]
pub struct StreamControl {
    /// Cancellation token
    pub cancellation_token: Arc<AtomicBool>,
    /// Pause/resume control
    pub pause_token: Arc<AtomicBool>,
    /// Backpressure signal
    pub backpressure: Arc<std::sync::Mutex<BackpressureSignal>>,
    /// Maximum buffer size
    pub max_buffer_size: usize,
    /// Current buffer size
    pub current_buffer_size: Arc<std::sync::atomic::AtomicUsize>,
}

impl Default for StreamControl {
    fn default() -> Self {
        Self {
            cancellation_token: Arc::new(AtomicBool::new(false)),
            pause_token: Arc::new(AtomicBool::new(false)),
            backpressure: Arc::new(std::sync::Mutex::new(BackpressureSignal::None)),
            max_buffer_size: 1000,
            current_buffer_size: Arc::new(std::sync::atomic::AtomicUsize::new(0)),
        }
    }
}

impl StreamControl {
    /// Create new stream control
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Create with custom buffer size
    pub fn with_buffer_size(max_buffer_size: usize) -> Self {
        Self {
            max_buffer_size,
            ..Self::default()
        }
    }
    
    /// Cancel the stream
    pub fn cancel(&self) {
        self.cancellation_token.store(true, Ordering::SeqCst);
    }
    
    /// Check if cancelled
    pub fn is_cancelled(&self) -> bool {
        self.cancellation_token.load(Ordering::SeqCst)
    }
    
    /// Pause the stream
    pub fn pause(&self) {
        self.pause_token.store(true, Ordering::SeqCst);
    }
    
    /// Resume the stream
    pub fn resume(&self) {
        self.pause_token.store(false, Ordering::SeqCst);
    }
    
    /// Check if paused
    pub fn is_paused(&self) -> bool {
        self.pause_token.load(Ordering::SeqCst)
    }
    
    /// Set backpressure signal
    pub fn set_backpressure(&self, signal: BackpressureSignal) {
        *self.backpressure.lock().unwrap() = signal;
    }
    
    /// Get current backpressure signal
    pub fn backpressure(&self) -> BackpressureSignal {
        *self.backpressure.lock().unwrap()
    }
    
    /// Update buffer size
    pub fn update_buffer_size(&self, size: usize) {
        self.current_buffer_size.store(size, Ordering::SeqCst);
        
        // Automatic backpressure based on buffer utilization
        let utilization = size as f64 / self.max_buffer_size as f64;
        let signal = match utilization {
            x if x < 0.5 => BackpressureSignal::None,
            x if x < 0.8 => BackpressureSignal::SlowDown,
            x if x < 0.95 => BackpressureSignal::Pause,
            _ => BackpressureSignal::Drop,
        };
        
        self.set_backpressure(signal);
    }
    
    /// Get buffer utilization (0.0 to 1.0)
    pub fn buffer_utilization(&self) -> f64 {
        let current = self.current_buffer_size.load(Ordering::SeqCst);
        current as f64 / self.max_buffer_size as f64
    }
}

/// Enhanced JSON-RPC Stream with priority and flow control
pub struct JsonRpcStream {
    inner: Pin<Box<dyn Stream<Item = Result<JsonRpcResponse>> + Send>>,
    control: StreamControl,
    policy: SpawnPolicy,
    stats: Arc<std::sync::Mutex<ExecutionStats>>,
}

impl JsonRpcStream {
    /// Create a new JSON-RPC stream
    pub fn new<S>(stream: S) -> Self 
    where
        S: Stream<Item = Result<JsonRpcResponse>> + Send + 'static,
    {
        Self {
            inner: Box::pin(stream),
            control: StreamControl::default(),
            policy: SpawnPolicy::default(),
            stats: Arc::new(std::sync::Mutex::new(ExecutionStats::default())),
        }
    }
    
    /// Create stream with policy
    pub fn with_policy<S>(stream: S, policy: SpawnPolicy) -> Self 
    where
        S: Stream<Item = Result<JsonRpcResponse>> + Send + 'static,
    {
        let mut s = Self::new(stream);
        s.policy = policy;
        s
    }
    
    /// Set spawn policy
    pub fn set_policy(mut self, policy: SpawnPolicy) -> Self {
        self.policy = policy;
        self
    }
    
    /// Get stream control
    pub fn control(&self) -> &StreamControl {
        &self.control
    }
    
    /// Get stream control (mutable)
    pub fn control_mut(&mut self) -> &mut StreamControl {
        &mut self.control
    }
    
    /// Get priority
    pub fn priority(&self) -> Priority {
        self.policy.priority
    }
    
    /// Box the stream for type erasure
    pub fn boxed(self) -> Pin<Box<dyn Stream<Item = Result<JsonRpcResponse>> + Send>> {
        Box::pin(self)
    }
    
    /// Map stream items
    pub fn map<F, T>(self, f: F) -> Pin<Box<dyn Stream<Item = T> + Send>>
    where
        F: FnMut(Result<JsonRpcResponse>) -> T + Send + 'static,
        T: Send + 'static,
    {
        Box::pin(self.inner.map(f))
    }
    
    /// Filter stream items
    pub fn filter<F>(self, f: F) -> Pin<Box<dyn Stream<Item = Result<JsonRpcResponse>> + Send>>
    where
        F: Fn(&Result<JsonRpcResponse>) -> bool + Send + Sync + 'static,
    {
        Box::pin(self.inner.filter(move |item| {
            let result = f(item);
            async move { result }
        }))
    }
    
    /// Take only first N items
    pub fn take(self, n: usize) -> Pin<Box<dyn Stream<Item = Result<JsonRpcResponse>> + Send>> {
        Box::pin(self.inner.take(n))
    }
    
    /// Skip first N items
    pub fn skip(self, n: usize) -> Pin<Box<dyn Stream<Item = Result<JsonRpcResponse>> + Send>> {
        Box::pin(self.inner.skip(n))
    }
    
    /// Add timeout to each stream item
    pub fn timeout_each(self, duration: Duration) -> Pin<Box<dyn Stream<Item = Result<JsonRpcResponse>> + Send>> {
        Box::pin(self.inner.then(move |item| async move {
            match tokio::time::timeout(duration, async move { item }).await {
                Ok(result) => result,
                Err(_) => Err(Error::timeout("JsonRpcStream item", duration)),
            }
        }))
    }
    
    /// Collect all items into a vector
    pub async fn collect_all(mut self) -> Result<Vec<JsonRpcResponse>> {
        let mut results = Vec::new();
        
        while let Some(item) = self.next().await {
            match item {
                Ok(response) => results.push(response),
                Err(e) => return Err(e),
            }
        }
        
        Ok(results)
    }
    
    /// Get execution statistics
    pub fn stats(&self) -> ExecutionStats {
        self.stats.lock().unwrap().clone()
    }
    
    /// Create a throttled stream
    pub fn throttle(self, duration: Duration) -> Pin<Box<dyn Stream<Item = Result<JsonRpcResponse>> + Send>> {
        Box::pin(self.inner.then(move |item| async move {
            tokio::time::sleep(duration).await;
            item
        }))
    }
    
    /// Create a buffered stream
    /// Note: This would require futures that produce JsonRpcResponse
    /// For now, we'll just return the original stream
    pub fn buffer_unordered(self, _n: usize) -> Pin<Box<dyn Stream<Item = Result<JsonRpcResponse>> + Send>> {
        // In a real implementation, this would buffer async operations
        // For now, just return the original stream
        Box::pin(self.inner)
    }
}

impl Stream for JsonRpcStream {
    type Item = Result<JsonRpcResponse>;
    
    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        // Check for cancellation
        if self.control.is_cancelled() {
            return Poll::Ready(None);
        }
        
        // Check for pause
        if self.control.is_paused() {
            cx.waker().wake_by_ref();
            return Poll::Pending;
        }
        
        // Check backpressure
        match self.control.backpressure() {
            BackpressureSignal::Drop => {
                // Skip this item due to backpressure
                cx.waker().wake_by_ref();
                return Poll::Pending;
            }
            BackpressureSignal::Pause => {
                // Temporarily pause
                cx.waker().wake_by_ref();
                return Poll::Pending;
            }
            BackpressureSignal::SlowDown => {
                // Add a small delay - schedule yield for next poll
                // This is a polling context, so we just return Pending to slow down
                return Poll::Pending;
            }
            BackpressureSignal::None => {
                // Normal processing
            }
        }
        
        // Update stats
        {
            let mut stats = self.stats.lock().unwrap();
            if stats.started_at.is_none() {
                stats.start();
            }
        }
        
        // Poll the inner stream
        match self.inner.as_mut().poll_next(cx) {
            Poll::Ready(Some(item)) => {
                // Update buffer size (simulated)
                let current_size = self.control.current_buffer_size.load(Ordering::SeqCst);
                self.control.update_buffer_size(current_size.saturating_sub(1));
                Poll::Ready(Some(item))
            }
            Poll::Ready(None) => {
                // Stream ended - update completion stats
                {
                    let mut stats = self.stats.lock().unwrap();
                    stats.complete();
                }
                Poll::Ready(None)
            }
            Poll::Pending => Poll::Pending,
        }
    }
}

/// Service Stream type alias for compatibility
pub type ServiceStream = JsonRpcStream;

/// Utility functions for creating enhanced futures and streams
impl JsonRpcFuture {
    /// Create from async function
    pub fn from_async<F, Fut>(f: F) -> Self 
    where
        F: FnOnce() -> Fut + Send + 'static,
        Fut: Future<Output = Result<JsonRpcResponse>> + Send + 'static,
    {
        Self::new(async move { f().await })
    }
    
    /// Create ready future with immediate result
    pub fn ready(result: Result<JsonRpcResponse>) -> Self {
        Self::new(async move { result })
    }
    
    /// Create error future
    pub fn error(error: Error) -> Self {
        Self::new(async move { Err(error) })
    }
}

impl JsonRpcStream {
    /// Create from iterator
    pub fn from_iter<I>(iter: I) -> Self 
    where
        I: IntoIterator<Item = Result<JsonRpcResponse>> + Send + 'static,
        I::IntoIter: Send,
    {
        Self::new(futures::stream::iter(iter))
    }
    
    /// Create empty stream
    pub fn empty() -> Self {
        Self::new(futures::stream::empty())
    }
    
    /// Create single-item stream
    pub fn once(item: Result<JsonRpcResponse>) -> Self {
        Self::new(futures::stream::once(async move { item }))
    }
    
    /// Create error stream
    pub fn error(error: Error) -> Self {
        Self::once(Err(error))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::types::JsonRpcResponse;
    use serde_json::json;
    use futures::StreamExt;

    #[tokio::test]
    async fn test_jsonrpc_future_with_policy() {
        let policy = SpawnPolicy::new()
            .with_priority(Priority::High)
            .with_timeout(Duration::from_secs(5));
        
        let future = JsonRpcFuture::with_policy(
            async { 
                Ok(JsonRpcResponse::success(json!(1), json!({"result": "test"})))
            },
            policy
        );
        
        assert_eq!(future.priority(), Priority::High);
        let result = future.await.unwrap();
        assert!(result.is_success());
    }

    #[tokio::test]
    async fn test_service_future_cancellation() {
        let future = JsonRpcFuture::new(async {
            tokio::time::sleep(Duration::from_secs(10)).await;
            Ok(JsonRpcResponse::success(json!(1), json!({"result": "test"})))
        });
        
        // Cancel the future
        future.cancel();
        
        let result = future.await;
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().kind(), crate::core::error::ErrorKind::Cancelled);
    }

    #[tokio::test]
    async fn test_service_future_with_timeout() {
        let future = JsonRpcFuture::new(async {
            tokio::time::sleep(Duration::from_secs(2)).await;
            Ok(JsonRpcResponse::success(json!(1), json!({"result": "test"})))
        });
        
        let result = future.timeout(Duration::from_millis(100)).await;
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().kind(), crate::core::error::ErrorKind::Timeout);
    }

    #[tokio::test]
    async fn test_service_stream_basic() {
        let items = vec![
            Ok(JsonRpcResponse::success(json!(1), json!(1))),
            Ok(JsonRpcResponse::success(json!(2), json!(2))),
            Ok(JsonRpcResponse::success(json!(3), json!(3))),
        ];
        
        let stream = JsonRpcStream::from_iter(items);
        let collected: Vec<_> = stream.collect().await;
        
        assert_eq!(collected.len(), 3);
        assert!(collected.into_iter().all(|r| r.is_ok()));
    }

    #[tokio::test]
    async fn test_stream_control() {
        let control = StreamControl::new();
        
        // Test pause/resume
        assert!(!control.is_paused());
        control.pause();
        assert!(control.is_paused());
        control.resume();
        assert!(!control.is_paused());
        
        // Test cancellation
        assert!(!control.is_cancelled());
        control.cancel();
        assert!(control.is_cancelled());
    }

    #[tokio::test]
    async fn test_stream_from_iter() {
        let data = vec![
            Ok(JsonRpcResponse::success(json!(1), json!("first"))),
            Ok(JsonRpcResponse::success(json!(2), json!("second"))),
            Err(Error::custom("test error")),
        ];
        
        let mut stream = JsonRpcStream::from_iter(data);
        
        // First two should be Ok
        assert!(stream.next().await.unwrap().is_ok());
        assert!(stream.next().await.unwrap().is_ok());
        
        // Third should be Err
        assert!(stream.next().await.unwrap().is_err());
        
        // Stream should end
        assert!(stream.next().await.is_none());
    }

    #[test]
    fn test_priority_ordering() {
        assert!(Priority::High > Priority::Normal);
        assert!(Priority::Critical > Priority::High);
        assert!(Priority::Normal > Priority::Low);
        
        assert!(Priority::High.is_higher_than(Priority::Normal));
        assert!(!Priority::Low.is_higher_than(Priority::Normal));
    }

    #[test]
    fn test_spawn_policy_builder() {
        let policy = SpawnPolicy::new()
            .with_priority(Priority::Critical)
            .with_timeout(Duration::from_secs(10))
            .with_retry(3, Duration::from_millis(100))
            .with_config("custom".to_string(), json!("value"));
        
        assert_eq!(policy.priority, Priority::Critical);
        assert_eq!(policy.timeout, Some(Duration::from_secs(10)));
        assert!(policy.retry_on_failure);
        assert_eq!(policy.max_retries, 3);
        assert_eq!(policy.custom_config.get("custom"), Some(&json!("value")));
    }

    #[test]
    fn test_backpressure_signals() {
        let control = StreamControl::with_buffer_size(100);
        
        // Test automatic backpressure
        control.update_buffer_size(30); // 30% utilization
        assert_eq!(control.backpressure(), BackpressureSignal::None);
        
        control.update_buffer_size(70); // 70% utilization
        assert_eq!(control.backpressure(), BackpressureSignal::SlowDown);
        
        control.update_buffer_size(90); // 90% utilization
        assert_eq!(control.backpressure(), BackpressureSignal::Pause);
        
        control.update_buffer_size(100); // 100% utilization
        assert_eq!(control.backpressure(), BackpressureSignal::Drop);
    }
} 