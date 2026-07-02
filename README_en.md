<div align="center">
  <img src="docs/logo.png" style="width:150px" alt="Aiway Logo">

![Release](https://github.com/xgpxg/aiway/actions/workflows/publish.yml/badge.svg)
![GitHub release](https://img.shields.io/github/v/release/xgpxg/aiway?label=Version)
![License](https://img.shields.io/github/license/xgpxg/aiway)

[中文](README.md) | [English](README_en.md)
</div>


**aiway** (API and AI Gateway) is a high-performance API + AI gateway developed in Rust, dedicated to providing stable,
efficient, and scalable request forwarding and management solutions.

Leveraging Rust's memory safety features and zero-cost abstractions, aiway delivers exceptional security and stability
while maintaining high performance.

## Platform Support

- Linux (x86_64 / arm64)
- macOS (arm64)
- openEuler (x86_64)
- UOS Server (x86_64)
- KylinOS (x86_64)

## Protocol Support

- HTTP（IPv4/IPv6）
- SSE
- WebSocket
- MCP

## Documentation

For detailed documentation, please visit: [https://aiway.coderbox.cn](https://aiway.coderbox.cn)

## Quick Start

### Option 1: Using Pre-built Binaries

```bash
# Download and extract
curl -L https://github.com/xgpxg/aiway/releases/latest/download/aiway-linux-amd64-standalone.tar.gz | tar -zxvf - -C .

# Start the service
./aiway
```

> **Note**: The pre-built binaries are built against glibc 2.35. If your system's glibc version is lower than 2.35,
> please build from source.

### Option 2: Building from Source

```bash
# Build Gateway
cargo build --bin gateway

# Build Console
cargo build --bin console

# Build Logg
cargo build --bin logg

# Run
cargo run --bin aiway
```

### Accessing the Services

- Management Console: http://127.0.0.1:7000
- Gateway Entry: http://127.0.0.1:7001
- Default Credentials: `admin` / `admin`

### Startup Parameters

```shell
./aiway -h

Usage: aiway [OPTIONS]

Options:
  -a, --address <ADDRESS>            Listen address, like 127.0.0.1 [default: 127.0.0.1]
  -p, --port <PORT>                  Port [default: 7000]
      --gateway-port <GATEWAY_PORT>  Gateway port [default: 7001]
  -h, --help                         Print help
  -V, --version                      Print version
```

## Core Features

- Dynamic Routing - Flexibly configure request routing rules
- Service Management - Unified management of backend service instances
- Plugin System - Scalable plugin architecture
- Security Protection - Built-in firewall and security verification mechanisms
- API Key Management - Unified API key management
- Log Monitoring - Complete log storage and real-time monitoring
- Visual Dashboard - Intuitive management console
- AI Model Proxy - Intelligent AI model proxy and request forwarding
- MCP Integration - Model Context Protocol support

## Plugin Ecosystem

### Official Plugins

We provide a series of commonly used plugins. Visit [aiway-plugins](https://github.com/xgpxg/aiway-plugins) for more
information.

### Custom Plugins

If you need to develop custom plugins, please refer to
the [Plugin Development Documentation](https://aiway.coderbox.cn/doc.html?path=docs/plugins/introduction.md).

## Interface Preview

![Dashboard](docs/images/screenshot.png)

## Performance

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

> The following test data is from older versions (<0.2.2). The new version is 30%~40% faster.

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

## Contributing

Welcome to submit Issues and Pull Requests to help improve aiway!




