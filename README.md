<div align="center">
  <img src="docs/logo.png" style="width:150px" alt="Conreg Logo">

![Release](https://github.com/xgpxg/aiway/actions/workflows/publish.yml/badge.svg)
![GitHub release](https://img.shields.io/github/v/release/xgpxg/aiway?label=Version)
![License](https://img.shields.io/github/license/xgpxg/conreg)

[中文](README.md) | [English](README_en.md)
</div>

### 简介

一个Rust实现的API网关，性能还算不错。

支持的平台：

- Linux
- MacOS

### 快速启动

单机模式：

```shell
cargo build --bin gateway -F standalone && \
cargo build --bin console -F standalone && \
cargo build --bin logg && \
cargo run --bin aiway
```

集群模式：

```shell
cargo run --bin console -F cluster && \
cargo build --bin gateway -F cluster
```

> 集群模式下需要单独部署[Redis](https://redis.io/)和[Quickwit](https://quickwit.io/)

控制台：http://127.0.0.1:7000

网关入口：http://127.0.0.1:7001

UI：https://github.com/xgpxg/aiway-ui

## 功能

- 动态路由
- 服务管理
- 插件系统
- 安全验证（防火墙）
- 统一API Key管理
- 日志存储和监控
- 可视化
- 支持单机/集群部署

### 截图

![Dashboard](docs/images/1.png)

### 项目结构

```text
├── aiway                   # 单机网关(bin)
├── console                 # 控制台(bin)
├── gateway                 # 网关核心(bin)
├── lib                     # 子库
│   ├── alert               # 告警
│   ├── cache               # 缓存
│   ├── common              # 通用
│   ├── loadbalance         # 负载均衡
│   ├── logging             # 日志  
│   ├── plugin              # 插件
│   └── protocol            # 交互协议
└── logg                    # 日志服务(bin)
```
