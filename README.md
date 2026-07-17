<div align="center">
  <img src="docs/logo.png" style="width:150px" alt="Aiway Logo">

![Release](https://github.com/xgpxg/aiway/actions/workflows/publish.yml/badge.svg)
![GitHub release](https://img.shields.io/github/v/release/xgpxg/aiway?label=Version)
![License](https://img.shields.io/github/license/xgpxg/aiway)

[中文](README.md) | [English](README_en.md)
</div>


**aiway** (API and AI Gateway) 是一个基于 Rust 开发的轻量、高性能 API + AI 网关，致力于提供稳定、高效、可扩展的请求转发与管理解决方案。

凭借 Rust 的内存安全特性和零成本抽象优势，aiway 在保证高性能的同时，提供了卓越的安全性和稳定性。

与其他网关不同，aiway无需复杂的配置和环境依赖，网关节点可动态扩容/缩容，轻松应对百万并发连接。

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

详细文档请访问：[Document](https://xgpxg.github.io/aiway)

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

### 启动参数

```shell
./aiway -h

Usage: aiway [OPTIONS]

Options:
      --console-address <CONSOLE_ADDRESS>      Console listen address [default: 127.0.0.1]
      --console-port <CONSOLE_PORT>            Console listen port [default: 7000]
      --gateway-address <GATEWAY_ADDRESS>      Gateway listen address [default: 127.0.0.1]
      --gateway-port <GATEWAY_PORT>            Gateway listen port [default: 7001]
      --log-server <LOG_SERVER>                Log server address [default: 127.0.0.1:7280]
      --access-address <ACCESS_ADDRESS>        Access listen address [default: 0.0.0.0]
      --access-http-port <ACCESS_HTTP_PORT>    Access http listen port [default: 7080]
      --access-https-port <ACCESS_HTTPS_PORT>  Access https listen port [default: 7443]
  -h, --help                                   Print help
  -V, --version                                Print version
```

## 核心功能

- [x] 动态路由
- [x] 服务管理
- [x] 插件系统
- [x] 安全防护
- [x] API Key 管理
- [x] 日志监控
- [x] 可视化面板
- [x] AI 模型代理
- [x] MCP 集成
- [x] 多域名支持

## 插件生态

### 已知插件

我们提供了一系列常用插件，访问 [aiway-plugins](https://github.com/xgpxg/aiway-plugins) 获取更多信息。

### 自定义插件

如需开发自定义插件，请参考 [插件开发文档](https://xgpxg.github.io/aiway/plugins/overview.html)。

## 界面预览

![Dashboard](docs/images/screenshot.png)

## 性能

### 当前

- 通过`access`接入点访问

```shell
wrk https://aiway-test-1.coderbox.cn:7443/api/hello -t 12 -c 100
Running 10s test @ http://aiway-test-1.coderbox.cn:7080/api/hello
  12 threads and 100 connections
  Thread Stats   Avg      Stdev     Max   +/- Stdev
    Latency     0.99ms  810.80us  34.97ms   96.17%
    Req/Sec     8.43k     1.23k   16.35k    78.69%
  1012287 requests in 10.10s, 253.90MB read
Requests/sec: 100224.67
Transfer/sec:     25.14MB
```

### 2026/05/24 前

- Ubuntu 24.04, Intel i7-12700K, 12 Cores, 16 GB RAM

```shell
wrk http://127.0.0.1:7001/api/hello -t 12 -c 100

Running 10s test @ http://127.0.0.1:7001/api/hello
  12 threads and 100 connections
  Thread Stats   Avg      Stdev     Max   +/- Stdev
    Latency     0.89ms  780.09us  28.66ms   95.87%
    Req/Sec     9.54k     1.12k   18.36k    93.70%
  1144556 requests in 10.10s, 287.07MB read
Requests/sec: 113330.59
Transfer/sec:     28.43MB
```

- Ubuntu 22.04, 4 Cores, 8 GB RAM

```shell
wrk http://127.0.0.1:7001/hello -t 4 -c 100

Running 10s test @ http://127.0.0.1:7001/hello
  4 threads and 100 connections
  Thread Stats   Avg      Stdev     Max   +/- Stdev
    Latency     6.70ms    3.05ms  36.85ms   73.84%
    Req/Sec     7.61k   770.50     9.22k    64.50%
  302860 requests in 10.01s, 75.96MB read
Requests/sec:  30265.20
Transfer/sec:      7.59MB
```

### 2026/03/26 前

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




