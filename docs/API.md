# grpc-mesh-node API 文档

`grpc-mesh-node` 是 Rust 节点接入 gRPC-Mesh 的库。节点通过它主动外连控制平面（无需公网入站），在一条 TLS + yamux 隧道上对外提供 gRPC 服务。

## 概述

库提供三件事：

1. **隧道建立**（`tunnel::TunnelConnector`）：TLS 连接、yamux 多路复用、控制流握手与心跳。
2. **服务承载**（`YamuxIncoming`）：把 yamux 连接适配成 tonic 可用的 incoming，节点的 gRPC 服务直接跑在隧道上。
3. **通用方法分发**（`MethodRegistry` + `rpc::InvokeService`）：以"方法名字符串 + 不透明字节"的通用形式暴露函数，供控制平面经 `InvokePlaneService.Invoke` 调用。

节点可以同时挂载**类型化服务**（按 proto 定义、由 tonic 生成代码实现，如 `calculator.v1.Calculator`）与通用分发服务，两者在同一 tonic 服务器上按 gRPC 路径共存。控制平面侧对类型化服务用生成的客户端代码直调（`Gateway.Dial`），对通用分发走 `Invoke`。

## 架构概览

```
节点（本库）                                控制平面（grpc-mesh-server）
┌─────────────────────────┐
│ tonic Server            │
│  ├─ GreeterServer       │   类型化服务：标准 gRPC（HTTP/2 + protobuf）
│  └─ InvokeService       │   通用分发：Invoke(method: string, payload: bytes)
│ YamuxIncoming           │
│ yamux Connection        │   多条流复用一条 TLS 连接
│ TLS (rustls)            │
└───────────┬─────────────┘
            │ 控制流（第一条流）：握手 JSON + 心跳 JSON（长度前缀帧）
            ▼
```

控制流协议：4 字节大端长度前缀 + JSON。握手为裸 `Handshake` JSON；心跳为 `{"type":"heartbeat","heartbeat":{...}}` 包装。心跳由库内任务自动发送，间隔取 `ConnectorConfig::heartbeat_interval`；写失败或超过一个间隔未完成会标记隧道死亡（见 `connect_with_backoff_watched`）。

## 快速开始

### 1. 添加依赖

```toml
[dependencies]
grpc-mesh = { path = "../grpc-mesh-node" }
tokio = { version = "1", features = ["full"] }
tonic = "0.14"
prost = "0.14"
```

### 2. 实现类型化 gRPC 服务（可选）与通用方法

```rust
pub mod hello {
    tonic::include_proto!("hello");
}

use hello::greeter_server::{Greeter, GreeterServer};

pub struct MyGreeter;

#[tonic::async_trait]
impl Greeter for MyGreeter {
    async fn say_hello(
        &self,
        request: tonic::Request<hello::HelloRequest>,
    ) -> Result<tonic::Response<hello::HelloResponse>, tonic::Status> {
        let name = request.into_inner().name;
        Ok(tonic::Response::new(hello::HelloResponse {
            message: format!("Hello, {name}!"),
        }))
    }
}
```

不需要类型化服务时，只注册通用方法即可：

```rust
use grpc_mesh::{MethodRegistry, RpcResult};
use std::sync::Arc;

let registry = MethodRegistry::default();
registry.register(
    "echo",
    Arc::new(|payload: Vec<u8>| -> RpcResult<Vec<u8>> { Ok(payload) }),
);
```

### 3. 建立隧道并服务

```rust
use grpc_mesh::rpc::InvokeService;
use grpc_mesh::tunnel::{ConnectorConfig, Handshake, TunnelConnector};
use tokio::sync::watch;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (shutdown_tx, shutdown_rx) = watch::channel(false);

    let connector = TunnelConnector::new(
        ConnectorConfig {
            server_addr: "mesh.example.com:8443".into(),
            // PEM 原始字节；留空使用系统根证书
            ca_certs: vec![std::fs::read("ca.crt")?],
            sni: None,
            connect_timeout: std::time::Duration::from_secs(10),
            max_backoff: std::time::Duration::from_secs(30),
            heartbeat_interval: std::time::Duration::from_secs(15),
            insecure_skip_verify: false,
        },
        shutdown_rx,
    )?;

    let registry = MethodRegistry::default();
    // ... register(...) 方法 ...

    // 方法清单随握手上报，控制平面经 SessionState::Methods() 可查
    let mut methods = registry.methods();
    methods.sort();

    let handshake = Handshake::builder("my-node", env!("CARGO_PKG_VERSION"))
        .token(std::env::var("TUNNEL_TOKEN")?)
        .add_feature("grpc")
        .metadata_entry("mesh.methods", methods.join(","))
        .build()?;

    let incoming = connector.connect_with_backoff(handshake).await?;

    tonic::transport::Server::builder()
        .add_service(GreeterServer::new(MyGreeter))          // 类型化路径
        .add_service(InvokeService::new(registry).into_server()) // 通用分发路径
        .serve_with_incoming(incoming)
        .await?;

    Ok(())
}
```

需要"隧道断开后自动重连"的长驻进程，参考 `src/bin/reverse_gateway.rs`：用 `connect_with_backoff_watched` 取得死亡信号，在 select 中监视，隧道结束时丢弃 serve future 并带退避重连。

## 核心 API

### `ConnectorConfig`

| 字段 | 类型 | 说明 |
|---|---|---|
| `server_addr` | `String` | 控制平面地址 `host:port` |
| `ca_certs` | `Vec<Vec<u8>>` | **PEM 原始字节**。非空时仅用这些 CA 验证服务端证书；空则用系统根。注意传入 DER 或无法解析出证书会在构造时报 `Config` 错误 |
| `sni` | `Option<String>` | SNI 覆盖；缺省取 `server_addr` 的 host 部分 |
| `connect_timeout` | `Duration` | TCP + TLS + yamux + 握手帧的总预算 |
| `max_backoff` | `Duration` | 连接重试退避上限（250ms 起步倍增） |
| `heartbeat_interval` | `Duration` | 心跳间隔；同时是单次心跳写完成的超时 |
| `insecure_skip_verify` | `bool` | 跳过证书验证，仅限受控开发场景 |

### `TunnelConnector`

```rust
pub fn new(cfg: ConnectorConfig, shutdown_rx: watch::Receiver<bool>) -> Result<Self>
pub async fn connect_with_backoff(&mut self, handshake: Handshake) -> Result<YamuxIncoming>
pub async fn connect_with_backoff_watched(
    &mut self,
    handshake: Handshake,
    tunnel_dead: watch::Sender<bool>,
) -> Result<YamuxIncoming>
```

- 连接失败按 `is_retryable()` 分类：瞬时错误指数退避重试；`Config`/`Shutdown` 终止。
- TLS 握手被对端拒绝（证书过期、CA 不对、SNI 不匹配等）归类为**永久 `Config` 错误**，直接失败不重试。
- `connect_with_backoff_watched` 额外在心跳写停滞/失败时把 `tunnel_dead` 置位，供监督循环感知半死连接。

### `Handshake` / `HandshakeBuilder`

```rust
Handshake::builder(node_id, version)
    .token(token)                       // 必填
    .add_feature("grpc")                // 可重复
    .metadata_entry("mesh.methods", "a.B/C,a.B/D")  // 任意键值
    .build()?
```

`node_id`、`version`、`token` 非空才会通过校验。控制平面配置 `auth.node_tokens` 时，token 与 node_id 绑定校验。

### `MethodRegistry`

```rust
pub fn register<S: Into<String>>(&self, method: S, handler: MethodHandler)
pub fn register_with_request<S: Into<String>>(&self, method: S, handler: MethodHandlerWithRequest)
pub fn methods(&self) -> Vec<String>   // 已注册方法名（顺序不定）
```

`MethodHandler = Arc<dyn Fn(Vec<u8>) -> RpcResult<Vec<u8>> + Send + Sync>`。方法名建议用 gRPC 全名（如 `calculator.v1.Calculator/Add`），与类型化路径的 gRPC 路径一致，便于控制平面统一展示。

### `InvokeService`

实现 `InvokePlaneService`（`Invoke` 一元 + `InvokeStream` 双向流），把请求按方法名分发给 `MethodRegistry`。`.into_server()` 得到 tonic 可挂载的服务。

### `YamuxIncoming` / `Tunnel`

`connect_with_backoff*` 返回 `YamuxIncoming`，直接传给 `serve_with_incoming`。`YamuxIncoming::handshake()` 可取回握手元数据。不需要心跳托管时可用 `Tunnel::into_parts()` 自管控制流。

## 方法上报约定

握手 metadata 的 `mesh.methods` 键 = 逗号分隔的方法名清单。控制平面在会话上以 `SessionState.Methods()` 暴露（去空白、忽略空段；未上报为空）。当前为连接时一次性快照，方法动态增删不会更新。

## 错误处理

`tunnel::TunnelError` 主要变体：`Config`（配置/永久性拒绝，**不重试**）、`Network`、`Tls`、`Yamux`、`Handshake`、`Timeout`（以上重试）、`Shutdown`。用 `is_retryable()` / `is_terminal()` 判定。

## 故障排查

- **`Config("TLS handshake rejected: invalid peer certificate: UnknownIssuer ...")` 且进程退出**：这是预期行为——CA 与服务端证书不匹配属永久错误，不会重试。检查 `ca_certs` 是否为正确的 PEM 字节。
- **`Config("ca_certs contained PEM data but no certificates could be parsed")`**：传入了 DER 或损坏的 PEM。`ca_certs` 必须是 PEM 原始字节，不要预解析。
- **连接后静默无心跳**：确认控制平面可达且证书匹配；心跳写超时后监督方会收到 `tunnel_dead` 信号。
- **升级后编译报 E0063（missing field）**：`ConnectorConfig` 新增字段时需在初始化处补齐（`..Default::default()` 可避免）。

## 参考

- 完整监督循环示例：`src/bin/reverse_gateway.rs`
- 通用 + 类型化双路径示例：`demos/calculator-service/src/main.rs`
- 控制平面侧调用：grpc-mesh-server `pkg/reverse/gateway.go`（`Invoke` 通用分发 / `Dial` 类型化拨号）
