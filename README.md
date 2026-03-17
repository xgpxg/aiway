<div align="center">
  <img src="docs/logo.png" style="width:150px" alt="Aiway Logo">

![Release](https://github.com/xgpxg/aiway/actions/workflows/publish.yml/badge.svg)
![GitHub release](https://img.shields.io/github/v/release/xgpxg/aiway?label=Version)
![License](https://img.shields.io/github/license/xgpxg/aiway)

[中文](README.md) | [English](README_en.md)
</div>


**aiway** 是一个基于 Rust 开发的高性能 API + AI 网关，致力于提供稳定、高效、可扩展的请求转发与管理解决方案。

凭借 Rust 的内存安全特性和零成本抽象优势，aiway 在保证高性能的同时，提供了卓越的安全性和稳定性。

## 平台支持

- **Linux** (x86_64 / arm64)
- **macOS** (arm64)

## 协议支持

- HTTP/HTTPS
- SSE
- WebSocket
- MCP

📑 [文档](https://aiway.coderbox.cn/doc.html)

## 快速开始

### 方式一：使用预编译版本

```bash
# 下载并解压
curl -L https://github.com/xgpxg/aiway/releases/latest/download/aiway-linux-amd64-standalone.tar.gz | tar -zxvf - -C .

# 启动服务
./aiway
```

> **注意**：预编译版本基于 glibc 2.35 构建，如果您的系统glibc版本低于 2.35，请从源码构建。

### 方式二：从源码构建

```bash
# 构建 Gateway（单机模式）
cargo build --bin gateway -F standalone

# 构建 Console（单机模式）
cargo build --bin console -F standalone

# 构建 Logg
cargo build --bin logg

# 运行
cargo run --bin aiway
```

### 访问服务

- **管理控制台**: http://127.0.0.1:7000
- **网关入口**: http://127.0.0.1:7001
- **默认账号**: `admin` / `admin`

### 构建选项

**Gateway 特性：**

- `standalone` - 单机模式
- `cluster` - 集群模式
- `model-proxy` - 启用模型代理功能

**Console 特性：**

- `standalone` - 单机模式
- `cluster` - 集群模式

## 核心功能

- **动态路由** - 灵活配置请求路由规则
- **服务管理** - 统一管理后端服务实例
- **插件系统** - 可扩展的插件架构
- **安全防护** - 内置防火墙和安全验证机制
- **API Key 管理** - 统一的 API 密钥管理
- **日志监控** - 完整的日志存储和实时监控
- **可视化面板** - 直观的管理控制台
- **AI 模型代理** - 智能 AI 模型代理和请求转发
- **MCP 集成** - Model Context Protocol 支持

## 插件生态

### 官方插件

我们提供了一系列常用插件，访问 [aiway-plugins](https://github.com/xgpxg/aiway-plugins) 获取更多信息。

### 自定义插件

如需开发自定义插件，请参考 [插件开发文档](https://aiway.coderbox.cn/doc.html?path=docs/plugins/introduction.md)。

## 界面预览

![Dashboard](docs/images/screenshot.png)

## 文档

详细文档请访问：[https://aiway.coderbox.cn/doc.html](https://aiway.coderbox.cn/doc.html)

## 性能

| 请求数           | 并发数 | 成功率  | 吞吐量 (req/s) | 平均延迟 (ms) | P50 (ms) | P90 (ms) | P95 (ms) | P99 (ms) | P99.9 (ms) | 最快 (ms) | 最慢 (ms) | 总耗时 (ms)  |
|---------------|-----|------|-------------|-----------|----------|----------|----------|----------|------------|---------|---------|-----------|
| **10,000**    | 300 | 100% | 42,381.22   | 6.89      | 5.00     | 11.95    | 19.27    | 41.22    | 47.52      | 0.47    | 55.69   | 235.95    |
| **100,000**   | 300 | 100% | 61,150.17   | 4.89      | 4.65     | 7.23     | 8.22     | 10.72    | 23.58      | 0.25    | 31.33   | 1,635.32  |
| **1,000,000** | 300 | 100% | 60,574.36   | 4.95      | 4.74     | 7.36     | 8.33     | 10.53    | 13.90      | 0.19    | 28.78   | 16,508.63 |

> 测试环境:
> - 硬件：Ubuntu 24.04, Intel i7-12700K, 16 GB RAM
> - 路由：10个
> - 插件：2个
> - 测试服务和网关在同一机器


## 贡献

欢迎提交 Issue 和 Pull Request 来帮助改进 aiway！




