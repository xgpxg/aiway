# Aiway

**Aiway** (API and AI Gateway) 是一个基于 Rust 开发的高性能 API + AI 网关。

凭借 Rust 的内存安全特性和零成本抽象，aiway 在保证高性能的同时提供卓越的安全性和稳定性。无需复杂配置和环境依赖，网关节点可动态扩容/缩容，轻松应对百万并发连接。


## 核心特性

- **动态路由** — 灵活配置请求路由规则，支持多域名、路径匹配
- **服务管理** — 统一管理后端服务实例，支持健康检查
- **插件系统** — 基于 WASM 的可扩展插件架构，支持热加载
- **安全防护** — 内置防火墙、IP 黑/白名单、Referer 校验
- **API Key 管理** — 统一的 API 密钥认证
- **日志监控** — 完整的日志存储与实时监控（基于 Tantivy）
- **可视化面板** — 直观的管理控制台
- **AI 模型代理** — 智能 AI 模型代理与请求转发
- **MCP 集成** — Model Context Protocol 支持
- **TLS 终止** — SNI 动态证书选择，支持通配符

## 支持协议

- HTTP (IPv4/IPv6)
- SSE
- WebSocket
- MCP (Model Context Protocol)

## 支持平台

- Linux (x86_64 / arm64)
- macOS (arm64)
- openEuler (x86_64)
- UOS Server (x86_64)
- KylinOS (x86_64)

## 快速体验

```bash
# 下载预编译版本
curl -L https://github.com/xgpxg/aiway/releases/latest/download/aiway-linux-amd64-standalone.tar.gz | tar -zxvf - -C .

# 启动服务
./aiway
```

访问管理控制台: [http://127.0.0.1:7000](http://127.0.0.1:7000)，默认账号 `admin` / `admin`。
