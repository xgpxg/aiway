<div align="center">
  <img src="docs/logo.png" style="width:150px" alt="Conreg Logo">

![Release](https://github.com/xgpxg/aiway/actions/workflows/publish.yml/badge.svg)
![GitHub release](https://img.shields.io/github/v/release/xgpxg/aiway?label=Version)
![License](https://img.shields.io/github/license/xgpxg/conreg)

[中文](README.md) | [English](README_en.md)
</div>

# Aiway - High Performance API Gateway

Aiway is a high-performance API gateway implemented in Rust.

## Supported Platforms

- Linux(x86_64)
- Linux(arm64)
- macOS(x86_64)

## Quick Start

### Standalone Mode

```shell
cargo build --bin gateway -F standalone && \
cargo build --bin console -F standalone && \
cargo build --bin logg && \
cargo run --bin aiway
```

### Cluster Mode

```shell
cargo run --bin console -F cluster && \
cargo build --bin gateway -F cluster
```

> Cluster mode requires separate deployment of [Redis](https://redis.io/) and [Quickwit](https://quickwit.io/)

### Access Points

- Console: http://127.0.0.1:7000
- Gateway Entry: http://127.0.0.1:7001
- UI: https://github.com/xgpxg/aiway-ui

## Features

- [x] Dynamic Routing
- [x] Service Management
- [x] Plugin System
- [x] Security Authentication (Firewall)
- [x] Unified API Key Management
- [x] Log Storage and Monitoring
- [x] Visualization Dashboard
- [x] Standalone/Cluster Deployment

## Screenshot

![Dashboard](docs/images/1.png)
