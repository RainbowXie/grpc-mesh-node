# grpc-mesh-node API 文档

## 概述

`grpc-mesh-node` 是一个 Rust 实现的反向网关客户端框架，用于将内网服务通过 TLS + Yamux 隧道暴露给外部控制平面调用。

**核心特性：**
- 🔐 TLS 1.3 加密通信
- 🚀 Yamux 多路复用（单连接承载多个逻辑流）
- 🔄 自动断线重连（指数退避）
- 📡 gRPC 服务托管
- 🔑 Token 认证
- 📊 分布式追踪支持

---

## 架构概览

```
┌─────────────────────────────────────────────────────────┐
│  grpc-mesh-node (内网节点)                                    │
│                                                         │
│  ┌──────────────┐    ┌──────────────┐                 │
│  │  Your gRPC   │───▶│  Transport   │                 │
│  │  Service     │    │  Connector   │                 │
│  └──────────────┘    └──────┬───────┘                 │
│                              │                          │
│                              ▼                          │
│                       ┌─────────────┐                  │
│                       │   Yamux     │                  │
│                       │  Connection │                  │
│                       └──────┬──────┘                  │
│                              │                          │
│                              ▼                          │
│                       ┌─────────────┐                  │
│                       │ TLS 1.3     │                  │
│                       │ (rustls)    │                  │
│                       └──────┬──────┘                  │
└──────────────────────────────┼──────────────────────────┘
                               │ TCP (主动出网)
                               ▼
┌─────────────────────────────────────────────────────────┐
│  grpc-mesh-server (公网控制平面 - Go)                       │
│                                                         │
│  ┌──────────────┐    ┌──────────────┐                 │
│  │  gRPC Client │◀───│  Reverse     │                 │
│  │  (业务调用)   │    │  Gateway     │                 │
│  └──────────────┘    └──────────────┘                 │
└─────────────────────────────────────────────────────────┘
```

---

## 快速开始

### 1. 添加依赖

```toml
[dependencies]
grpc-mesh-node = { path = "../grpc-mesh-node" }
tokio = { version = "1.38", features = ["full"] }
tonic = "0.11"
```

### 2. 实现你的 gRPC 服务

```rust
use tonic::{Request, Response, Status};

pub mod hello {
    tonic::include_proto!("hello");
}

use hello::{HelloRequest, HelloResponse};
use hello::greeter_server::{Greeter, GreeterServer};

pub struct MyGreeter;

#[tonic::async_trait]
impl Greeter for MyGreeter {
    async fn say_hello(
        &self,
        request: Request<HelloRequest>,
    ) -> Result<Response<HelloResponse>, Status> {
        let name = request.into_inner().name;
        let reply = HelloResponse {
            message: format!("Hello, {}!", name),
        };
        Ok(Response::new(reply))
    }
}
```

### 3. 启动反向网关

```rust
use wa_emu_rs::gateway::ReverseGateway;
use wa_emu_rs::config::GatewayConfig;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. 加载配置
    let config = GatewayConfig::load("config.json")?;
    
    // 2. 创建网关
    let gateway = ReverseGateway::new(config).await?;
    
    // 3. 注册 gRPC 服务
    let greeter = MyGreeter;
    gateway.add_service(GreeterServer::new(greeter));
    
    // 4. 启动（阻塞直到收到 SIGINT/SIGTERM）
    gateway.serve().await?;
    
    Ok(())
}
```

---

## 核心 API

### 1. `GatewayConfig`

配置结构体，定义连接参数、认证信息和重连策略。

#### 字段说明

```rust
pub struct GatewayConfig {
    /// 服务端地址（格式: "host:port"）
    pub server_address: String,
    
    /// TLS 配置
    pub tls: TlsConfig,
    
    /// 节点配置
    pub node: NodeConfig,
    
    /// 重连策略
    pub reconnect: ReconnectConfig,
}

pub struct TlsConfig {
    /// 服务端域名（用于证书验证）
    pub server_name: String,
    
    /// CA 证书路径（可选，默认使用系统根证书）
    pub ca_cert_path: Option<String>,
}

pub struct NodeConfig {
    /// 节点 ID（"auto" 表示自动生成）
    pub id: String,
    
    /// 认证 Token
    pub token: String,
    
    /// 节点版本
    pub version: String,
    
    /// 支持的功能特性列表
    pub supported_features: Vec<String>,
    
    /// 附加元数据
    pub metadata: HashMap<String, String>,
}

pub struct ReconnectConfig {
    /// 初始重连延迟（秒）
    pub base_delay_secs: u64,
    
    /// 最大重连延迟（秒）
    pub max_delay_secs: u64,
    
    /// 最大重试次数（0 表示无限重试）
    pub max_retries: u32,
    
    /// 退避倍数
    pub backoff_multiplier: f64,
}
```

#### 从文件加载

```rust
// 从 JSON 文件加载
let config = GatewayConfig::load("config.json")?;

// 从 TOML 文件加载
let config = GatewayConfig::load_toml("config.toml")?;

// 从环境变量覆盖
let mut config = GatewayConfig::load("config.json")?;
config.apply_env_overrides();
```

#### 环境变量覆盖

| 环境变量 | 配置字段 | 说明 |
|---------|---------|------|
| `WA_SERVER_ADDRESS` | `server_address` | 服务端地址 |
| `WA_NODE_ID` | `node.id` | 节点 ID |
| `WA_NODE_TOKEN` | `node.token` | 认证 Token |
| `WA_NODE_VERSION` | `node.version` | 节点版本 |
| `WA_TLS_SERVER_NAME` | `tls.server_name` | TLS 服务端域名 |
| `WA_TLS_CA_CERT` | `tls.ca_cert_path` | CA 证书路径 |

#### 配置文件示例

**JSON 格式：**

```json
{
  "server": {
    "address": "gateway.example.com:8443",
    "tls": {
      "server_name": "gateway.example.com",
      "ca_cert_path": "/etc/wa-emu/ca.crt"
    }
  },
  "node": {
    "id": "auto",
    "token": "waemu_your_token_here",
    "version": "1.0.0",
    "supported_features": ["invoke", "health", "echo"],
    "metadata": {
      "region": "us-west-2",
      "env": "production"
    }
  },
  "reconnect": {
    "base_delay_secs": 1,
    "max_delay_secs": 60,
    "max_retries": 0,
    "backoff_multiplier": 2.0
  }
}
```

**TOML 格式：**

```toml
[server]
address = "gateway.example.com:8443"

[server.tls]
server_name = "gateway.example.com"
ca_cert_path = "/etc/wa-emu/ca.crt"

[node]
id = "auto"
token = "waemu_your_token_here"
version = "1.0.0"
supported_features = ["invoke", "health", "echo"]

[node.metadata]
region = "us-west-2"
env = "production"

[reconnect]
base_delay_secs = 1
max_delay_secs = 60
max_retries = 0
backoff_multiplier = 2.0
```

---

### 2. `ReverseGateway`

反向网关核心类，负责连接管理、服务托管和生命周期控制。

#### 创建网关

```rust
use wa_emu_rs::gateway::ReverseGateway;

// 从配置创建
let gateway = ReverseGateway::new(config).await?;

// 带自定义 tracing 层
let gateway = ReverseGateway::builder()
    .config(config)
    .with_tracing_layer(tracing_layer)
    .build()
    .await?;
```

#### 注册 gRPC 服务

```rust
use tonic::transport::server::Router;

// 方式 1：直接添加服务
gateway.add_service(GreeterServer::new(greeter));

// 方式 2：使用 Router
let router = Router::new()
    .add_service(GreeterServer::new(greeter))
    .add_service(HealthServer::new(health));
gateway.set_router(router);

// 方式 3：链式调用
gateway
    .add_service(GreeterServer::new(greeter))
    .add_service(HealthServer::new(health))
    .add_service(MetricsServer::new(metrics));
```

#### 启动网关

```rust
// 阻塞直到收到停止信号
gateway.serve().await?;

// 带超时启动
use tokio::time::{timeout, Duration};
timeout(Duration::from_secs(30), gateway.serve()).await??;

// 手动控制生命周期
let handle = gateway.spawn();
// ... 做其他事情 ...
gateway.shutdown().await?;
```

#### 健康检查

```rust
// 检查连接状态
if gateway.is_connected() {
    println!("Gateway is connected");
}

// 获取连接统计
let stats = gateway.connection_stats();
println!("Reconnect count: {}", stats.reconnect_count);
println!("Uptime: {:?}", stats.uptime);
```

#### 动态更新方法列表

```rust
use wa_emu_rs::method::{MethodDescriptor, MethodMetadata};

let methods = vec![
    MethodDescriptor {
        name: "greeter.SayHello".to_string(),
        description: Some("Greet a user".to_string()),
        tags: vec!["hello".to_string()],
        metadata: MethodMetadata::default(),
    },
];

gateway.update_methods(methods).await?;
```

---

### 3. `TransportConnector`

底层传输层连接器，处理 TCP → TLS → Yamux 连接建立和重连。

#### 创建连接器

```rust
use wa_emu_rs::transport::TransportConnector;

let connector = TransportConnector::new(
    "gateway.example.com:8443",
    tls_config,
    reconnect_config,
).await?;
```

#### 获取 Yamux 连接

```rust
// 获取当前连接（如果已连接）
let connection = connector.connection().await?;

// 打开新流
let stream = connector.open_stream().await?;
```

#### 监听连接事件

```rust
let mut event_rx = connector.subscribe_events();

tokio::spawn(async move {
    while let Some(event) = event_rx.recv().await {
        match event {
            TransportEvent::Connected => {
                println!("Connected to server");
            }
            TransportEvent::Disconnected(reason) => {
                println!("Disconnected: {}", reason);
            }
            TransportEvent::Reconnecting(attempt) => {
                println!("Reconnecting (attempt {})", attempt);
            }
            TransportEvent::Error(err) => {
                eprintln!("Transport error: {}", err);
            }
        }
    }
});
```

---

### 4. 控制流 API

控制流用于握手、心跳和方法注册。

#### 握手

握手在连接建立后自动执行，包含节点认证和元数据交换。

**握手消息格式：**

```json
{
  "type": "handshake",
  "payload": {
    "node_id": "myhost-a1b2c3d4",
    "token": "waemu_your_token_here",
    "version": "1.0.0",
    "features": ["invoke", "health"],
    "metadata": {
      "hostname": "myhost",
      "os": "linux",
      "arch": "x86_64"
    }
  }
}
```

**响应：**

```json
{
  "accepted": true,
  "session_id": "sess_abc123",
  "server_version": "1.0.0"
}
```

#### 心跳

心跳自动发送，默认间隔 30 秒。

```rust
// 自定义心跳间隔
let config = GatewayConfig {
    heartbeat_interval_secs: 15,
    ..Default::default()
};

// 禁用心跳（不推荐）
let config = GatewayConfig {
    heartbeat_enabled: false,
    ..Default::default()
};
```

**心跳消息格式：**

```json
{
  "type": "heartbeat",
  "payload": {
    "node_id": "myhost-a1b2c3d4",
    "timestamp": 1704067200000,
    "metrics": {
      "cpu_usage": 45.2,
      "memory_usage": 1024000000,
      "active_streams": 3
    }
  }
}
```

#### 方法注册

```rust
// 手动触发方法注册
gateway.register_methods().await?;

// 自动注册（启动时）
let gateway = ReverseGateway::builder()
    .config(config)
    .auto_register_methods(true)
    .build()
    .await?;
```

---

## 高级用法

### 1. 自定义中间件

```rust
use tonic::body::BoxBody;
use tower::{Layer, Service};
use std::task::{Context, Poll};

// 定义中间件
#[derive(Clone)]
struct LoggingMiddleware<S> {
    inner: S,
}

impl<S> Service<hyper::Request<BoxBody>> for LoggingMiddleware<S>
where
    S: Service<hyper::Request<BoxBody>>,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = S::Future;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: hyper::Request<BoxBody>) -> Self::Future {
        println!("Request: {:?}", req);
        self.inner.call(req)
    }
}

// 应用中间件
let gateway = ReverseGateway::builder()
    .config(config)
    .layer(LoggingMiddleware)
    .build()
    .await?;
```

### 2. 自定义错误处理

```rust
use wa_emu_rs::error::{GatewayError, ErrorHandler};

struct MyErrorHandler;

impl ErrorHandler for MyErrorHandler {
    fn handle_connection_error(&self, err: &GatewayError) {
        // 发送告警、记录日志等
        eprintln!("Connection error: {}", err);
    }

    fn handle_service_error(&self, method: &str, err: &tonic::Status) {
        eprintln!("Service error in {}: {}", method, err);
    }
}

let gateway = ReverseGateway::builder()
    .config(config)
    .error_handler(MyErrorHandler)
    .build()
    .await?;
```

### 3. 动态证书更新

```rust
// 监听证书文件变化
use notify::{Watcher, RecursiveMode};

let (tx, rx) = tokio::sync::mpsc::channel(1);
let mut watcher = notify::recommended_watcher(move |res| {
    tx.blocking_send(res).ok();
})?;

watcher.watch(Path::new("/etc/wa-emu/ca.crt"), RecursiveMode::NonRecursive)?;

tokio::spawn(async move {
    while let Some(Ok(event)) = rx.recv().await {
        println!("Certificate changed: {:?}", event);
        gateway.reload_tls_config().await?;
    }
});
```

### 4. 指标收集

```rust
use wa_emu_rs::metrics::{MetricsCollector, MetricsSnapshot};

// 获取指标快照
let metrics = gateway.metrics_snapshot();
println!("Total requests: {}", metrics.total_requests);
println!("Error rate: {:.2}%", metrics.error_rate * 100.0);

// 导出 Prometheus 格式
let prometheus_text = gateway.metrics_prometheus();
println!("{}", prometheus_text);

// 注册自定义指标
gateway.register_metric("my_custom_counter", MetricType::Counter);
gateway.increment_metric("my_custom_counter", 1);
```

---

## 错误处理

### 错误类型

```rust
pub enum GatewayError {
    /// 配置错误
    ConfigError(String),
    
    /// 连接错误
    ConnectionError(std::io::Error),
    
    /// TLS 错误
    TlsError(rustls::Error),
    
    /// 认证失败
    AuthenticationFailed(String),
    
    /// 握手失败
    HandshakeError(String),
    
    /// 服务调用错误
    ServiceError(tonic::Status),
    
    /// 超时
    Timeout,
    
    /// 其他错误
    Other(String),
}
```

### 错误处理示例

```rust
use wa_emu_rs::error::GatewayError;

match gateway.serve().await {
    Ok(_) => println!("Gateway stopped gracefully"),
    Err(GatewayError::AuthenticationFailed(msg)) => {
        eprintln!("Authentication failed: {}", msg);
        std::process::exit(1);
    }
    Err(GatewayError::ConnectionError(e)) => {
        eprintln!("Connection error: {}", e);
        // 可能需要重试或告警
    }
    Err(e) => {
        eprintln!("Unexpected error: {}", e);
        std::process::exit(2);
    }
}
```

---

## 最佳实践

### 1. 生产环境配置

```rust
let config = GatewayConfig {
    server_address: "gateway.example.com:8443".to_string(),
    
    tls: TlsConfig {
        server_name: "gateway.example.com".to_string(),
        ca_cert_path: Some("/etc/wa-emu/ca.crt".to_string()),
    },
    
    node: NodeConfig {
        id: std::env::var("WA_NODE_ID")
            .unwrap_or_else(|_| "auto".to_string()),
        token: std::env::var("WA_NODE_TOKEN")
            .expect("WA_NODE_TOKEN must be set"),
        version: env!("CARGO_PKG_VERSION").to_string(),
        supported_features: vec![
            "invoke".to_string(),
            "health".to_string(),
        ],
        metadata: Default::default(),
    },
    
    reconnect: ReconnectConfig {
        base_delay_secs: 1,
        max_delay_secs: 60,
        max_retries: 0, // 无限重试
        backoff_multiplier: 2.0,
    },
    
    heartbeat_interval_secs: 30,
    heartbeat_timeout_secs: 90,
};
```

### 2. 优雅关闭

```rust
use tokio::signal;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let gateway = ReverseGateway::new(config).await?;
    gateway.add_service(MyService::new());
    
    // 启动网关
    let gateway_handle = tokio::spawn(async move {
        gateway.serve().await
    });
    
    // 等待停止信号
    signal::ctrl_c().await?;
    println!("Shutting down...");
    
    // 优雅关闭（等待现有请求完成）
    gateway.shutdown_with_timeout(Duration::from_secs(30)).await?;
    
    gateway_handle.await??;
    println!("Shutdown complete");
    
    Ok(())
}
```

### 3. 日志和追踪

```rust
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

fn init_tracing() {
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .with(tracing_subscriber::EnvFilter::from_default_env())
        .init();
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    init_tracing();
    
    // Gateway 会自动记录结构化日志
    let gateway = ReverseGateway::new(config).await?;
    gateway.serve().await?;
    
    Ok(())
}
```

**日志级别设置：**

```bash
# 设置全局日志级别
RUST_LOG=info ./my_gateway

# 细粒度控制
RUST_LOG=wa_emu_rs=debug,my_service=info ./my_gateway

# 只显示错误
RUST_LOG=error ./my_gateway
```

### 4. 健康检查端点

```rust
use tonic::{Request, Response, Status};

pub mod health {
    tonic::include_proto!("grpc.health.v1");
}

use health::{HealthCheckRequest, HealthCheckResponse, ServingStatus};
use health::health_server::{Health, HealthServer};

pub struct HealthService {
    gateway: Arc<ReverseGateway>,
}

#[tonic::async_trait]
impl Health for HealthService {
    async fn check(
        &self,
        _request: Request<HealthCheckRequest>,
    ) -> Result<Response<HealthCheckResponse>, Status> {
        let status = if self.gateway.is_connected() {
            ServingStatus::Serving
        } else {
            ServingStatus::NotServing
        };
        
        Ok(Response::new(HealthCheckResponse {
            status: status as i32,
        }))
    }
}

// 注册健康检查服务
gateway.add_service(HealthServer::new(HealthService {
    gateway: Arc::clone(&gateway),
}));
```

---

## 故障排查

### 问题 1：连接超时

**症状：**
```
ERROR Connection timeout: failed to connect to gateway.example.com:8443
```

**可能原因：**
1. 网络不可达
2. 防火墙阻止
3. 服务端未启动

**解决方案：**
```bash
# 测试网络连通性
ping gateway.example.com

# 测试端口
telnet gateway.example.com 8443

# 检查防火墙
sudo iptables -L -n | grep 8443
```

### 问题 2：TLS 握手失败

**症状：**
```
ERROR TLS handshake error: invalid certificate
```

**可能原因：**
1. CA 证书不匹配
2. 证书过期
3. 服务端域名不匹配

**解决方案：**
```bash
# 检查证书有效期
openssl x509 -in /etc/wa-emu/ca.crt -noout -dates

# 验证证书链
openssl s_client -connect gateway.example.com:8443 -CAfile /etc/wa-emu/ca.crt

# 检查服务端域名
openssl s_client -connect gateway.example.com:8443 | openssl x509 -noout -text | grep DNS
```

### 问题 3：认证失败

**症状：**
```
ERROR Handshake rejected: invalid token
```

**解决方案：**
1. 确认 Token 正确配置
2. 检查 Token 是否在服务端允许列表中
3. 确认 Token 未过期

```bash
# 验证环境变量
echo $WA_NODE_TOKEN

# 重新生成 Token
cd grpc-mesh-server
bash scripts/gen-token.sh
```

### 问题 4：频繁重连

**症状：**
```
WARN Reconnecting (attempt 10)
WARN Reconnecting (attempt 11)
```

**可能原因：**
1. 网络不稳定
2. 服务端过载
3. 心跳超时设置过短

**解决方案：**
```rust
// 调整重连参数
let config = GatewayConfig {
    reconnect: ReconnectConfig {
        base_delay_secs: 2,      // 增加初始延迟
        max_delay_secs: 120,     // 增加最大延迟
        backoff_multiplier: 2.5, // 增加退避倍数
        ..Default::default()
    },
    heartbeat_interval_secs: 60, // 增加心跳间隔
    ..Default::default()
};
```

---

## 性能优化

### 1. 并发流控制

```rust
// 限制最大并发流数量
let config = GatewayConfig {
    max_concurrent_streams: 64,
    ..Default::default()
};
```

### 2. 缓冲区大小

```rust
// 调整 Yamux 窗口大小
let config = GatewayConfig {
    yamux_window_size: 256 * 1024, // 256KB
    ..Default::default()
};
```

### 3. Keep-Alive 设置

```rust
// 启用 TCP Keep-Alive
let config = GatewayConfig {
    tcp_keepalive_secs: Some(60),
    tcp_nodelay: true,
    ..Default::default()
};
```

---

## 示例代码

完整示例请参考：
- `examples/basic_gateway.rs` - 基础用法
- `examples/multi_service.rs` - 多服务注册
- `examples/custom_middleware.rs` - 自定义中间件
- `examples/metrics_export.rs` - 指标导出

---

## API 参考

完整 API 文档请运行：

```bash
cargo doc --open
```

---

## 常见问题（FAQ）

**Q: 可以在一个进程中运行多个 Gateway 实例吗？**

A: 可以，只要它们使用不同的 Node ID：

```rust
let gateway1 = ReverseGateway::builder()
    .node_id("node-1")
    .config(config1)
    .build().await?;

let gateway2 = ReverseGateway::builder()
    .node_id("node-2")
    .config(config2)
    .build().await?;

tokio::try_join!(gateway1.serve(), gateway2.serve())?;
```

**Q: 如何限制 gRPC 消息大小？**

A: 在注册服务时设置：

```rust
let service = GreeterServer::new(greeter)
    .max_decoding_message_size(4 * 1024 * 1024)  // 4MB
    .max_encoding_message_size(4 * 1024 * 1024);

gateway.add_service(service);
```

**Q: 支持 WebSocket 吗？**

A: 当前版本不支持，但可以通过 gRPC 流实现类似功能：

```rust
#[tonic::async_trait]
impl MyService for MyServiceImpl {
    type StreamMethod = ReceiverStream<Result<Response, Status>>;
    
    async fn stream_method(
        &self,
        request: Request<StreamRequest>,
    ) -> Result<Response<Self::StreamMethod>, Status> {
        let (tx, rx) = mpsc::channel(128);
        // 实现双向流
        Ok(Response::new(ReceiverStream::new(rx)))
    }
}
```

**Q: 如何实现负载均衡？**

A: 负载均衡由服务端（grpc-mesh-server）实现，客户端只需注册多个相同方法即可。

---

## 更新日志

### v0.1.0 (2024-01-15)
- ✅ 初始版本发布
- ✅ TLS + Yamux 传输层
- ✅ 控制流（握手、心跳）
- ✅ gRPC 服务托管
- ✅ 自动重连机制
- ✅ Token 认证

---

## 许可证

[待定]

---

## 技术支持

- 📧 Email: support@example.com
- 💬 Issues: https://github.com/wa-emu/grpc-mesh-node/issues
- 📖 文档: https://wa-emu.example.com/docs
