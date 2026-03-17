<div align="center">
  <img src="docs/logo.png" style="width:150px" alt="Aiway Logo">

![Release](https://github.com/xgpxg/aiway/actions/workflows/publish.yml/badge.svg)
![GitHub release](https://img.shields.io/github/v/release/xgpxg/aiway?label=Version)
![License](https://img.shields.io/github/license/xgpxg/aiway)

[中文](README.md) | [English](README_en.md)
</div>


**aiway** is a high-performance API + AI gateway developed in Rust, dedicated to providing a stable, efficient, and scalable request forwarding and management solution.

Leveraging Rust's memory safety features and zero-cost abstractions, aiway delivers excellent security and stability while maintaining high performance.

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

## Core Features

- **Dynamic Routing** - Flexible configuration of request routing rules
- **Service Management** - Unified management of backend service instances
- **Plugin System** - Extensible plugin architecture
- **Security Protection** - Built-in firewall and security verification mechanisms
- **API Key Management** - Unified API key management
- **Log Monitoring** - Complete log storage and real-time monitoring
- **Visual Dashboard** - Intuitive management console
- **AI Model Proxy** - Intelligent AI model proxy and request forwarding
- **MCP Integration** - Model Context Protocol support

## Plugin Ecosystem

### Official Plugins

We provide a series of commonly used plugins. Visit [aiway-plugins](https://github.com/xgpxg/aiway-plugins) for more information.

### Custom Plugins

If you need to develop custom plugins, please refer to [Plugin Development Documentation](https://aiway.coderbox.cn/doc.html?path=docs/plugins/introduction.md).

## Interface Preview

![Dashboard](docs/images/screenshot.png)

## Documentation

For detailed documentation, please visit: [https://aiway.coderbox.cn/doc.html](https://aiway.coderbox.cn/doc.html)

## Performance

| Requests        | Concurrency | Success Rate | Throughput (req/s) | Avg Latency (ms) | P50 (ms) | P90 (ms) | P95 (ms) | P99 (ms) | P99.9 (ms) | Fastest (ms) | Slowest (ms) | Total Time (ms) |
|-----------------|-------------|--------------|-------------------|------------------|----------|----------|----------|----------|------------|--------------|--------------|-----------------|
| **10,000**      | 300         | 100%         | 42,381.22         | 6.89             | 5.00     | 11.95    | 19.27    | 41.22    | 47.52      | 0.47         | 55.69        | 235.95          |
| **100,000**     | 300         | 100%         | 61,150.17         | 4.89             | 4.65     | 7.23     | 8.22     | 10.72    | 23.58      | 0.25         | 31.33        | 1,635.32        |
| **1,000,000**   | 300         | 100%         | 60,574.36         | 4.95             | 4.74     | 7.36     | 8.33     | 10.53    | 13.90      | 0.19         | 28.78        | 16,508.63       |

> Test Environment:
> - Hardware: Ubuntu 24.04, Intel i7-12700K, 16 GB RAM
> - Routes: 10
> - Plugins: 2
> - Test service and gateway on the same machine


## Contributing

Issues and Pull Requests are welcome to help improve aiway!




