<div align="center">
  <img src="docs/logo.png" style="width:150px" alt="Aiway Logo">

![Release](https://github.com/xgpxg/aiway/actions/workflows/publish.yml/badge.svg)
![GitHub release](https://img.shields.io/github/v/release/xgpxg/aiway?label=Version)
![License](https://img.shields.io/github/license/xgpxg/aiway)

[中文](README.md) | [English](README_en.md)
</div>


**aiway** (API and AI Gateway) 是一个基于 Rust 开发的高性能 API + AI 网关，致力于提供稳定、高效、可扩展的请求转发与管理解决方案。

凭借 Rust 的内存安全特性和零成本抽象优势，aiway 在保证高性能的同时，提供了卓越的安全性和稳定性。

## 支持的系统

- Linux (x86_64 / arm64)
- macOS (arm64)
- openEuler (x86_64)
- UOS Server (x86_64)
- KylinOS (x86_64)

## 支持的协议

- HTTP（IPv4/IPv6）
- SSE
- WebSocket
- MCP

## 文档

详细文档请访问：[https://aiway.coderbox.cn/doc.html](https://aiway.coderbox.cn/doc.html)

## 快速开始

### 方式一：使用预编译版本

```bash
# 下载并解压
curl -L https://github.com/xgpxg/aiway/releases/latest/download/aiway-linux-amd64-standalone.tar.gz | tar -zxvf - -C .

# 启动服务
./aiway
```

> **注意**：预编译版本基于 glibc 2.34 构建，如果您的系统glibc版本低于 2.34，请从源码构建。

### 方式二：从源码构建

```bash
# 构建 Gateway
cargo build --bin gateway

# 构建 Console
cargo build --bin console

# 构建 Logg
cargo build --bin logg

# 运行
cargo run --bin aiway
```

### 访问服务

- 管理控制台: http://127.0.0.1:7000
- 网关入口: http://127.0.0.1:7001
- 默认账号: `admin` / `admin`

**Console Features：**

## 核心功能

- 动态路由 - 灵活配置请求路由规则
- 服务管理 - 统一管理后端服务实例
- 插件系统 - 可扩展的插件架构
- 安全防护 - 内置防火墙和安全验证机制
- API Key 管理 - 统一的 API 密钥管理
- 日志监控 - 完整的日志存储和实时监控
- 可视化面板 - 直观的管理控制台
- AI 模型代理 - 智能 AI 模型代理和请求转发
- MCP 集成 - Model Context Protocol 支持

## 插件生态

### 官方插件

我们提供了一系列常用插件，访问 [aiway-plugins](https://github.com/xgpxg/aiway-plugins) 获取更多信息。

### 自定义插件

如需开发自定义插件，请参考 [插件开发文档](https://aiway.coderbox.cn/doc.html?path=docs/plugins/introduction.md)。

## 界面预览

![Dashboard](docs/images/screenshot.png)

## 性能

- Ubuntu 24.04, Intel i7-12700K, 12 Cores, 16 GB RAM

```shell
wrk http://127.0.0.1:7001/hello -t 12 -c 100

Running 10s test @ http://127.0.0.1:7001/hello
  12 threads and 100 connections
  Thread Stats   Avg      Stdev     Max   +/- Stdev
    Latency     1.25ms  574.88us  10.34ms   82.29%
    Req/Sec     6.54k   756.78    14.64k    86.22%
  784402 requests in 10.10s, 222.18MB read
Requests/sec:  77662.06
Transfer/sec:     22.00MB
```

- openEuler 24.03 (LTS-SP3), Intel(R) Xeon(R) 6982P-C, 4 Cores, 8 GB RAM

```shell
wrk http://127.0.0.1:7001/hello -t 4 -c 100

Running 10s test @ http://127.0.0.1:7001/hello
  4 threads and 100 connections
  Thread Stats   Avg      Stdev     Max   +/- Stdev
    Latency     3.80ms    1.22ms  13.60ms   70.35%
    Req/Sec     6.62k   380.06     7.43k    62.75%
  263639 requests in 10.01s, 74.67MB read
Requests/sec:  26332.11
Transfer/sec:      7.46MB
```

- UOS Server 20, Intel(R) Xeon(R) Platinum, 4 Cores, 8 GB RAM

```shell
wrk http://127.0.0.1:7001/hello -t 4 -c 100

Running 10s test @ http://127.0.0.1:7001/hello
  8 threads and 100 connections
  Thread Stats   Avg      Stdev     Max   +/- Stdev
    Latency     3.61ms    2.48ms  84.71ms   93.78%
    Req/Sec     3.47k   452.12    11.40k    83.88%
  276965 requests in 10.06s, 78.45MB read
Requests/sec:  27518.91
Transfer/sec:      7.79MB
```

## 贡献

欢迎提交 Issue 和 Pull Request 来帮助改进 aiway！




