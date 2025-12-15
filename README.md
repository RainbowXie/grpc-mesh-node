# grpc-mesh-node

`grpc-mesh-node` 是运行在内网节点上的 Rust 代理，负责：

- 主动发起 `TCP → TLS → Yamux` 隧道到公网控制面（`grpc-mesh-server`）。
- 在控制流上发送 JSON Handshake（NodeID、Token、版本、特性等），完成注册与鉴权。
- 通过 `reverse_gateway` 进程将 Yamux 子流适配为 `tonic` 的 `Incoming`，并向控制面暴露 gRPC 方法（默认提供 `health.echo` / `health.ping`）。

该 README 重点介绍 `reverse_gateway` 的使用方法与配置项，帮助你快速把 Rust 节点接入到 TLS + Yamux + 反向 gRPC 的通道中。

## 🚀 独立二进制部署

**`reverse_gateway` 采用嵌入式配置设计**，所有配置和证书在编译时通过 `include_str!` 宏嵌入到二进制文件中：

✅ **编译后只有一个独立的二进制文件**  
✅ **无需外部配置文件或证书文件**  
✅ **自动生成唯一的 Node ID**（主机名 + 短UUID）  
✅ **可直接分发和运行**

详细说明请查看 [独立构建指南](docs/standalone_build.md)。

---

## 目录结构概览

```
grpc-mesh-node/
├── Cargo.toml                      # Crate 定义以及主要依赖
├── config/                         # 嵌入式配置（编译时嵌入）
│   ├── reverse_gateway_config.json # 节点配置
│   └── ca.crt                      # CA 证书
├── src/
│   ├── bin/reverse_gateway.rs      # 入口：使用 include_str! 嵌入配置
│   ├── lib.rs                      # RPC 抽象 & MethodRegistry
│   ├── tunnel/                     # TLS + Yamux connector、handshake、incoming 适配器
│   └── ...
├── docs/
│   └── standalone_build.md         # 独立构建详细指南
└── build.rs / generated/           # Protobuf 生成、资产构建
```

---

**配置文件示例** (`config/reverse_gateway_config.json`)：

```json
{
  "server": {
    "address": "127.0.0.1:8443",
    "tls": {
      "server_name": "localhost"
    }
  },
  "node": {
    "id": "auto",
    "token": "waemu_7RCx4i4T6gU3O9Gqcx4-SvHMRN1V8dJ9",
    "version": "1.0.0",
    "supported_features": ["invoke", "health", "echo"],
    "metadata": {
      "role": "reverse_gateway",
      "environment": "production"
    }
  },
  "heartbeat": {
    "interval_secs": 10
  },
  "reconnect": {
    "base_delay_secs": 1,
    "max_delay_secs": 60,
    "max_retries": 0
  },
  "logging": {
    "level": "info"
  }
}
```

**Token 配置说明（重要）**：

Token 用于节点身份验证，必须与 Go 控制面配置匹配。

| 配置方式 | 优先级 | 说明 | 适用场景 |
| --- | --- | --- | --- |
| 环境变量 `WA_NODE_TOKEN` | 高 | 无需重新编译 | **生产环境推荐** |
| 配置文件 `node.token` | 低 | 需要重新编译 | 测试环境 |

**生成 Token：**
```bash
# 在 grpc-mesh-server 目录运行
bash scripts/gen-token.sh

# 输出示例：waemu_a4Fu1Kjz5nxtxeR8udKxTYiHhdfL9rAyI3429ESv
```

**配置 Token：**

方式 1（推荐）- 环境变量：
```bash
export WA_NODE_TOKEN="waemu_a4Fu1Kjz5nxtxeR8udKxTYiHhdfL9rAyI3429ESv"
./reverse_gateway
```

方式 2 - 配置文件：
```json
{
  "node": {
    "token": "waemu_a4Fu1Kjz5nxtxeR8udKxTYiHhdfL9rAyI3429ESv"
  }
}
```
⚠️ 修改配置文件后需要重新编译：`cargo build --release --bin reverse_gateway`

详细说明请查看 [Token 策略文档](../grpc-mesh-server/docs/token_policy.md)。

**Node ID 配置说明**：

| 配置值 | 说明 |
| --- | --- |
| `"auto"` | 自动生成唯一 ID（推荐），格式：`{hostname}-{uuid_prefix}`，例如 `myserver-a1b2c3d4` |
| `""` (空字符串) | 同 `"auto"`，自动生成 |
| `"custom-id"` | 使用指定的固定 ID（仅适用于单节点部署） |

**环境变量覆盖**：

即使配置文件中设置了 `"auto"`，仍可通过环境变量动态指定 ID：

```bash
export WA_NODE_ID="my-custom-node-id"
./reverse_gateway
```

---

## 配置方式

### 方式一：嵌入式配置（推荐）

**所有配置和证书在编译时嵌入到二进制中**，无需外部文件。

**快速开始**：

```bash
# 1. 编辑配置文件
vim config/reverse_gateway_config.json

# 2. 复制 CA 证书（从 Go server 获取）
cp ../grpc-mesh-server/config/tls/ca.crt config/ca.crt

# 3. 编译
cargo build --bin reverse_gateway --release

# 4. 运行（无需任何参数或环境变量）
./target/release/reverse_gateway
```

**配置文件示例** (`config/reverse_gateway_config.json`)：

```json
{
  "server": {
    "address": "127.0.0.1:8443",
    "tls": {
      "server_name": "localhost"
    }
  },
  "node": {
    "id": "rust-node-001",
    "token": "",
    "version": "1.0.0",
    "supported_features": ["invoke", "health", "echo"],
    "metadata": {
      "role": "reverse_gateway",
      "environment": "production"
    }
  },
  "heartbeat": {
    "interval_secs": 10
  },
  "reconnect": {
    "base_delay_secs": 1,
    "max_delay_secs": 60,
    "max_retries": 0
  },
  "logging": {
    "level": "info"
  }
}
```

详细说明请查看 [独立构建指南](docs/standalone_build.md)。

### 方式二：环境变量配置（向后兼容）

| 变量名 | 默认值 | 说明 |
| --- | --- | --- |
| `WA_SERVER_ADDR` | `127.0.0.1:8443` | Go 控制面的 TLS 监听地址 |
| `WA_NODE_TOKEN` | 配置文件值 | 节点认证 Token，需与控制面 `security.allowed_tokens` 匹配；环境变量优先级高于配置文件 |
| `WA_NODE_ID` | 自动生成 | 节点逻辑 ID；未设置时自动生成（主机名 + 短UUID） |
| `WA_NODE_VERSION` | `0.0.1` | 上报给控制面的版本号 |
| `WA_NODE_FEATURES` | `yamux-reverse-grpc` | 逗号分隔的特性列表（自动去重、大小写规范化） |
| `WA_TLS_SNI` | `server_addr` host | 可选，自定义 TLS SNI |
| `WA_TLS_CA_BUNDLE` | 空 | PEM 格式 CA 文件路径；为空则使用嵌入的 CA 证书 |
| `WA_MAX_BACKOFF_SECS` | `30` | Yamux 重连的指数退避上限（秒） |

> **Node ID 自动生成**：当 `WA_NODE_ID` 未设置时，系统会自动生成格式为 `{hostname}-{uuid}` 的唯一 ID（例如：`myserver-a1b2c3d4`），确保多节点部署时不会发生 ID 冲突。

> 说明：`reverse_gateway` 会在 Handshake 中加入 `metadata`（如平台、架构）和 `supported_features`，控制面可据此做策略分发。

---

## 快速开始（嵌入式配置）

1. **准备依赖**
   - Rust 1.77+（Stable）
   - `cargo install protobuf-codegen`（如果需要重新生成 `.proto`）

2. **配置和编译**

```bash
# 从 Go server 获取 CA 证书
cp ../grpc-mesh-server/config/tls/ca.crt config/ca.crt

# 生成 Token（在 grpc-mesh-server 目录）
cd ../grpc-mesh-server
bash scripts/gen-token.sh
# 复制生成的 token

# 编辑配置（修改 node.token、server.address 等）
cd ../grpc-mesh-node
vim config/reverse_gateway_config.json

# 编译（配置和证书会被嵌入）
cargo build --bin reverse_gateway --release
```

3. **运行**

```bash
# 方式 1：直接运行（使用嵌入的配置）
./target/release/reverse_gateway

# 方式 2：通过环境变量指定 Token（推荐）
WA_NODE_TOKEN="waemu_your_token_here" ./target/release/reverse_gateway

# 示例输出：
# ╔════════════════════════════════════════════════════════════╗
# ║  wa-emu Reverse Gateway (Embedded Config)                  ║
# ╚════════════════════════════════════════════════════════════╝
#
# Configuration (embedded):
#   Server:   127.0.0.1:8443
#   Node ID:  myserver-a1b2c3d4 (auto-generated)
#   Version:  1.0.0
#   Token:    <none>
#   Features: ["invoke", "health", "echo"]
#
# [INFO] Auto-generated node_id: myserver-a1b2c3d4
# [INFO] Tunnel established
# [INFO] Starting reverse gateway with embedded config
```

4. **分发部署**

编译后的二进制文件可以直接复制到任何机器运行，**每个节点会自动生成唯一的 Node ID**：

```bash
# 复制到多个目标机器
scp target/release/reverse_gateway user@host1:/usr/local/bin/
scp target/release/reverse_gateway user@host2:/usr/local/bin/
scp target/release/reverse_gateway user@host3:/usr/local/bin/

# 在不同机器上运行（每个会自动生成不同的 ID）
# 在目标机器上运行（使用环境变量指定 Token）
ssh user@host1 "WA_NODE_TOKEN=waemu_token1 /usr/local/bin/reverse_gateway"  # 生成: host1-a1b2c3d4
ssh user@host2 "WA_NODE_TOKEN=waemu_token2 /usr/local/bin/reverse_gateway"  # 生成: host2-e5f6g7h8
ssh user@host3 "WA_NODE_TOKEN=waemu_token3 /usr/local/bin/reverse_gateway"  # 生成: host3-i9j0k1l2

# 可选：同时指定 Node ID 和 Token
ssh user@host1 "WA_NODE_ID=prod-node-01 WA_NODE_TOKEN=waemu_token1 /usr/local/bin/reverse_gateway"
```

---

## 自定义 Handler

`reverse_gateway` 默认注册位于 `src/bin/reverse_gateway.rs` 中的两个 Handler：

```rust
registry.register("health.echo", Arc::new(|payload| Ok(payload)));
registry.register("health.ping", Arc::new(|_| Ok(b"PONG".to_vec())));
```

扩展方法的建议：

1. 在二进制中引入你的业务模块，实现 `Fn(Vec<u8>) -> RpcResult<Vec<u8>>`。
2. 调用 `registry.register("your.method", Arc::new(handler))`。
3. 通过控制流（后续 Roadmap Phase 5）将方法列表同步到控制面。

---

## 故障排查

| 现象 | 可能原因 | 排查建议 |
| --- | --- | --- |
| TLS 握手失败 | 证书不匹配或 CA 未配置 | 确认 `WA_TLS_CA_BUNDLE`、SNI、Server 证书是否一致 |
| Handshake 被拒绝 | Token 无效 / NodeID 不在白名单 | 确认 Token 是否在控制面 `security.allowed_tokens` 列表中；使用 `bash scripts/gen-token.sh` 生成新 Token |
| gRPC 无法访问方法 | 会话未注册或 Handler 未注册 | 确认 `Tunnel established` 日志，以及控制面 SessionManager 中的节点状态 |
| 频繁重连 | 网络波动或 TLS 失败 | 调整 `WA_MAX_BACKOFF_SECS`，并检查网络、证书有效期 |

---

## 生产环境部署

### systemd 服务

创建 `/etc/systemd/system/wa-emu-reverse-gateway.service`：

```ini
[Unit]
Description=wa-emu Reverse Gateway
After=network-online.target

[Service]
Type=simple
ExecStart=/usr/local/bin/reverse_gateway
Restart=always
RestartSec=5

[Install]
WantedBy=multi-user.target
```

启动：

```bash
sudo systemctl enable wa-emu-reverse-gateway
sudo systemctl start wa-emu-reverse-gateway
sudo systemctl status wa-emu-reverse-gateway
```

### Docker 部署

```bash
# 构建镜像（配置会被嵌入）
docker build -t wa-emu-reverse-gateway:latest .

# 运行
docker run -d --name reverse-gateway --restart unless-stopped \
  wa-emu-reverse-gateway:latest
```

详细部署说明请查看 [独立构建指南](docs/standalone_build.md)。

## 参考

- [独立构建指南](docs/standalone_build.md)：详细的编译、配置、部署说明
- [Roadmap（TLS + Yamux 方案）](../ROADMAP.md)：项目路线图、Phase 进度与技术选型
- [`grpc-mesh-server/README.md`](../grpc-mesh-server/README.md)：控制面的 Listener、SessionManager、Reverse Gateway 细节
