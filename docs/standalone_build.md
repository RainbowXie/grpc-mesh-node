# Standalone Reverse Gateway 构建指南

## 概述

grpc-mesh-node 的 `reverse_gateway` 二进制程序采用了**嵌入式配置**设计，所有配置和证书在编译时通过 `include_str!` 宏嵌入到二进制文件中。这意味着：

✅ **编译后只有一个独立的二进制文件**  
✅ **无需外部配置文件或证书文件**  
✅ **可直接分发和运行**  

---

## 目录结构

```
grpc-mesh-node/
├── config/
│   ├── reverse_gateway_config.json  # 配置文件（编译时嵌入）
│   └── ca.crt                        # CA 证书（编译时嵌入）
├── src/
│   └── bin/
│       └── reverse_gateway.rs        # 主程序（使用 include_str!）
├── Cargo.toml
└── docs/
    └── standalone_build.md           # 本文档
```

---

## 快速开始

### 1. 修改配置文件

编辑 `config/reverse_gateway_config.json`：

```json
{
  "server": {
    "address": "127.0.0.1:8443",        # Go server 地址
    "tls": {
      "ca_cert_path": "config/ca.crt",  # 占位符，实际使用嵌入的证书
      "server_name": "localhost"        # TLS SNI server name
    }
  },
  "node": {
    "id": "rust-node-001",              # 节点唯一 ID
    "token": "",                        # 认证 token（留空如果 server 配置 require_token: false）
    "version": "1.0.0",
    "supported_features": ["invoke", "health", "echo"],
    "metadata": {
      "role": "reverse_gateway",
      "environment": "production"       # 可自定义元数据
    }
  },
  "heartbeat": {
    "interval_secs": 10                 # 心跳间隔（秒）
  },
  "reconnect": {
    "base_delay_secs": 1,
    "max_delay_secs": 60,
    "max_retries": 0                    # 0 表示无限重试
  },
  "yamux": {
    "window_update_mode": "on_read",
    "enable_keepalive": true
  },
  "logging": {
    "level": "info"                     # trace, debug, info, warn, error
  }
}
```

### 2. 准备 CA 证书

**方式 A：使用 Go Server 生成的 CA（推荐）**

如果你已经运行了 Go server 的证书生成脚本：

```bash
# 在 grpc-mesh-server 目录
cd grpc-mesh-server
./scripts/gen-dev-certs.sh

# 复制 CA 证书到 Rust 项目
cp config/tls/ca.crt ../grpc-mesh-node/config/ca.crt
```

**方式 B：手动生成自签 CA**

```bash
cd grpc-mesh-node/config

# 生成 CA 私钥和证书
openssl req -x509 -newkey rsa:4096 \
  -keyout ca.key \
  -out ca.crt \
  -days 365 \
  -nodes \
  -subj "/CN=wa-emu-ca/O=wa-emu/C=US"

# 注意：ca.key 仅用于生成 server 证书，不需要分发
```

**方式 C：使用现有 CA**

直接将你的 CA 证书（PEM 格式）复制到 `config/ca.crt`。

### 3. 编译二进制

```bash
cd grpc-mesh-node

# Release 模式编译（推荐）
cargo build --bin reverse_gateway --release

# 二进制文件位置
ls -lh target/release/reverse_gateway

# 示例输出：
# -rwxr-xr-x  1 user  staff   8.2M  Jan 15 10:00 reverse_gateway
```

**编译优化选项**（可选）：

在 `Cargo.toml` 中添加：

```toml
[profile.release]
opt-level = "z"     # 优化二进制大小
lto = true          # Link-Time Optimization
codegen-units = 1   # 更好的优化
strip = true        # 去除符号表
```

重新编译后二进制大小可能减少 50% 以上。

### 4. 运行

```bash
# 直接运行
./target/release/reverse_gateway

# 示例输出：
# ╔════════════════════════════════════════════════════════════╗
# ║  wa-emu Reverse Gateway (Embedded Config)                  ║
# ╚════════════════════════════════════════════════════════════╝
#
# Configuration (embedded):
#   Server:   127.0.0.1:8443
#   Node ID:  rust-node-001
#   Version:  1.0.0
#   Token:    <none>
#   Features: ["invoke", "health", "echo"]
#
# [INFO] Starting reverse gateway with embedded config
# [INFO] Tunnel established
```

---

## 验证嵌入式配置

### 检查二进制大小

```bash
ls -lh target/release/reverse_gateway
# 应该是一个 5-10 MB 的文件
```

### 验证配置已嵌入

```bash
# 搜索二进制中的配置字符串
strings target/release/reverse_gateway | grep "rust-node-001"

# 如果看到输出，说明配置已成功嵌入
```

### 测试独立运行

```bash
# 复制到其他目录测试
cp target/release/reverse_gateway /tmp/
cd /tmp
./reverse_gateway

# 应该能正常启动，不依赖任何外部文件
```

---

## 多节点部署

### 场景：部署 3 个不同的节点

**步骤 1：为每个节点创建配置**

```bash
cd grpc-mesh-node

# 节点 1
cp config/reverse_gateway_config.json config/node1_config.json
# 编辑 node1_config.json，修改 node.id 为 "node-001"

# 节点 2
cp config/reverse_gateway_config.json config/node2_config.json
# 编辑 node2_config.json，修改 node.id 为 "node-002"

# 节点 3
cp config/reverse_gateway_config.json config/node3_config.json
# 编辑 node3_config.json，修改 node.id 为 "node-003"
```

**步骤 2：修改 `reverse_gateway.rs` 指向不同配置**

为每个节点创建不同的二进制（或使用符号链接技巧）：

方式 A - 创建多个二进制：

```bash
# 复制 reverse_gateway.rs
cp src/bin/reverse_gateway.rs src/bin/reverse_gateway_node1.rs
cp src/bin/reverse_gateway.rs src/bin/reverse_gateway_node2.rs

# 修改每个文件的 EMBEDDED_CONFIG 路径
# node1: include_str!("../../config/node1_config.json")
# node2: include_str!("../../config/node2_config.json")

# 编译
cargo build --bin reverse_gateway_node1 --release
cargo build --bin reverse_gateway_node2 --release
```

方式 B - 使用构建脚本（推荐）：

创建 `build_nodes.sh`：

```bash
#!/bin/bash

NODES=("node-001" "node-002" "node-003")

for node in "${NODES[@]}"; do
  # 修改配置文件中的 node.id
  jq ".node.id = \"$node\"" config/reverse_gateway_config.json > config/temp_config.json
  mv config/temp_config.json config/reverse_gateway_config.json
  
  # 编译
  cargo build --bin reverse_gateway --release
  
  # 重命名
  cp target/release/reverse_gateway "target/release/reverse_gateway_$node"
  
  echo "Built: reverse_gateway_$node"
done
```

---

## 生产环境部署

### 1. 配置 systemd 服务

创建 `/etc/systemd/system/wa-emu-reverse-gateway.service`：

```ini
[Unit]
Description=wa-emu Reverse Gateway
After=network-online.target
Wants=network-online.target

[Service]
Type=simple
User=wa-emu
Group=wa-emu
ExecStart=/usr/local/bin/reverse_gateway
Restart=always
RestartSec=5
StandardOutput=journal
StandardError=journal

# 安全加固
NoNewPrivileges=true
PrivateTmp=true
ProtectSystem=strict
ProtectHome=true
ReadWritePaths=/var/log/wa-emu

[Install]
WantedBy=multi-user.target
```

启动服务：

```bash
# 复制二进制
sudo cp target/release/reverse_gateway /usr/local/bin/
sudo chmod +x /usr/local/bin/reverse_gateway

# 创建用户
sudo useradd -r -s /bin/false wa-emu

# 启动服务
sudo systemctl daemon-reload
sudo systemctl enable wa-emu-reverse-gateway
sudo systemctl start wa-emu-reverse-gateway

# 查看状态
sudo systemctl status wa-emu-reverse-gateway
sudo journalctl -u wa-emu-reverse-gateway -f
```

### 2. Docker 部署

创建 `Dockerfile`：

```dockerfile
FROM rust:1.75 as builder

WORKDIR /build
COPY . .

# 编译
RUN cargo build --bin reverse_gateway --release

# 运行时镜像
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /build/target/release/reverse_gateway /usr/local/bin/

USER nobody
ENTRYPOINT ["/usr/local/bin/reverse_gateway"]
```

构建和运行：

```bash
# 构建镜像
docker build -t wa-emu-reverse-gateway:latest .

# 运行
docker run -d \
  --name reverse-gateway \
  --restart unless-stopped \
  wa-emu-reverse-gateway:latest

# 查看日志
docker logs -f reverse-gateway
```

---

## 更新配置流程

### 修改配置并重新编译

```bash
cd grpc-mesh-node

# 1. 编辑配置
vim config/reverse_gateway_config.json

# 2. 验证 JSON 格式
jq . config/reverse_gateway_config.json

# 3. 重新编译
cargo build --bin reverse_gateway --release

# 4. 测试新二进制
./target/release/reverse_gateway

# 5. 部署（如果测试通过）
sudo systemctl stop wa-emu-reverse-gateway
sudo cp target/release/reverse_gateway /usr/local/bin/
sudo systemctl start wa-emu-reverse-gateway
```

### 更新证书

```bash
# 1. 获取新的 CA 证书
cp /path/to/new/ca.crt config/ca.crt

# 2. 重新编译（证书会被嵌入）
cargo build --bin reverse_gateway --release

# 3. 部署
sudo systemctl stop wa-emu-reverse-gateway
sudo cp target/release/reverse_gateway /usr/local/bin/
sudo systemctl start wa-emu-reverse-gateway
```

---

## 故障排查

### 问题 1: 证书验证失败

**症状**：
```
[ERROR] TLS handshake failed: invalid peer certificate: UnknownIssuer
```

**解决**：
1. 确认 `config/ca.crt` 与 Go server 的证书是同一个 CA
2. 检查证书格式是否正确（PEM 格式）
3. 验证证书内容：
   ```bash
   openssl x509 -in config/ca.crt -text -noout
   ```

### 问题 2: 连接超时

**症状**：
```
[ERROR] Failed to connect to server: connection timeout
```

**解决**：
1. 检查 server.address 配置
2. 验证 Go server 是否在运行
3. 检查防火墙规则
4. 测试网络连通性：
   ```bash
   telnet 127.0.0.1 8443
   ```

### 问题 3: Token 认证失败

**症状**：
```
[ERROR] Handshake rejected by server
```

**解决**：
1. 检查 Go server 的 `security.require_token` 配置
2. 如果需要 token，在配置中添加：
   ```json
   "token": "your-actual-token-here"
   ```
3. 确保 token 在 Go server 的 `security.allowed_tokens` 列表中

### 问题 4: 二进制文件过大

**症状**：二进制文件 > 20 MB

**解决**：
1. 使用 release 模式编译
2. 启用编译优化（见上文）
3. 移除调试符号：
   ```bash
   strip target/release/reverse_gateway
   ```

---

## 高级配置

### 使用环境变量覆盖（可选）

虽然配置已嵌入，但你仍可通过环境变量覆盖某些值（需要代码支持）：

```bash
# 示例：覆盖日志级别
RUST_LOG=debug ./reverse_gateway
```

### 构建多架构二进制

```bash
# Linux x86_64
cargo build --bin reverse_gateway --release --target x86_64-unknown-linux-gnu

# Linux ARM64
cargo build --bin reverse_gateway --release --target aarch64-unknown-linux-gnu

# macOS
cargo build --bin reverse_gateway --release --target x86_64-apple-darwin
cargo build --bin reverse_gateway --release --target aarch64-apple-darwin
```

### 交叉编译（使用 cross）

```bash
# 安装 cross
cargo install cross

# ARM64 编译
cross build --bin reverse_gateway --release --target aarch64-unknown-linux-musl

# 输出静态链接二进制
ls -lh target/aarch64-unknown-linux-musl/release/reverse_gateway
```

---

## 安全建议

1. **保护源代码**：配置和证书在源码中，确保源码访问受限
2. **定期轮换 Token**：重新编译并部署新二进制
3. **使用生产 CA**：不要在生产环境使用自签名证书
4. **限制二进制权限**：`chmod 500` 或 `750`
5. **审计日志**：定期检查连接日志和认证失败记录

---

## 相关文档

- [Go Server 配置指南](../../grpc-mesh-server/docs/phase4_e2e_guide.md)
- [证书生成脚本](../../grpc-mesh-server/scripts/gen-dev-certs.sh)
- [ACL 配置](../../grpc-mesh-server/docs/acl_guide.md)
---

## 总结

通过嵌入式配置方式，`reverse_gateway` 编译后：

✅ 单一二进制文件，无外部依赖  
✅ 配置和证书安全嵌入  
✅ 方便分发和部署  
✅ 适合容器化和自动化部署  

修改配置或证书需要重新编译，这在配置变更不频繁的生产环境中是可接受的权衡。
