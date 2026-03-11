<div align="center">
  <img src="docs/logo.png" style="width:150px" alt="Conreg Logo">

![Release](https://github.com/xgpxg/aiway/actions/workflows/publish.yml/badge.svg)
![GitHub release](https://img.shields.io/github/v/release/xgpxg/aiway?label=Version)
![License](https://img.shields.io/github/license/xgpxg/conreg)

[中文](README.md) | [English](README_en.md)
</div>

### 简介

一个Rust实现的API+AI网关，性能还算不错。

支持的平台：

- Linux(x86_64)
- Linux(arm64)
- MacOS(arm64)

支持的协议：

- HTTP
- SSE
- Websocket

📑 [文档](https://aiway.coderbox.cn/doc.html)

### 快速启动

运行已发布的版本：

```shell
# 下载并解压
curl -L https://github.com/xgpxg/aiway/releases/latest/download/aiway-linux-amd64-standalone.tar.gz | tar -zxvf - -C .

# 启动
./aiway
```

> 发布版基于 glibc 2.35 版本构建，如果你的系统glibc版本低于2.35，请从源码构建。

从源码构建：

```shell
cargo build --bin gateway -F standalone && \
cargo build --bin console -F standalone && \
cargo build --bin logg && \
cargo run --bin aiway
```

控制台：http://127.0.0.1:7000

网关入口：http://127.0.0.1:7001

默认用户名/密码：admin/admin


Gateway Features:

- standalone: 构建单机模式
- cluster: 构建集群模式
- model-proxy: 启用模型代理

Console Features:
- standalone: 构建单机模式
- cluster: 构建集群模式


### 功能

- ✅ 动态路由
- ✅ 服务管理
- ✅ 插件系统
- ✅ 安全验证（防火墙）
- ✅ 统一API Key管理
- ✅ 日志存储和监控
- ✅ 可视化
- ✅ AI模型代理和转发

### 插件

这里提供了一些常用插件：https://github.com/xgpxg/aiway-plugins

如果需要自定义插件，可参考这里：https://aiway.coderbox.cn/doc.html?path=docs/plugins/introduction.md

### 截图

![Dashboard](docs/images/screenshot.png)


