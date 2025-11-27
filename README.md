# Aiway - Rust 实现的高性能 API 网关

Aiway 是一个基于 Rust 开发的现代化、高性能 API 网关解决方案，提供了完整的微服务治理功能。

## 功能特性

- 🔧 **接口路由管理** - 灵活的路由配置和转发规则
- 🔄 **负载均衡** - 支持多种负载均衡算法
- 🔌 **插件系统** - 可扩展的插件架构，支持自定义功能
- 📊 **日志和监控** - 完整的日志收集和实时监控面板
- 🔐 **安全防护** - 内置防火墙和安全访问控制
- ⚡ **高性能** - 基于 Rust 的异步运行时，提供卓越性能
- 📦 **一体化部署** - 支持单机版和集群版部署模式

## 架构组成

Aiway 由多个核心组件构成：

- **Gateway** - 核心网关服务，负责请求路由和转发
- **Console** - 管理控制台，提供 Web UI 和管理 API
- **Logg** - 日志服务，负责日志收集、索引和查询
- **Message** - 消息服务，处理系统内部消息通信
- **Cache** - 缓存服务，提供分布式缓存支持
- **Plugins** - 插件系统，支持功能扩展

## 快速开始

### 启动服务

#### 方式一：一体化启动（推荐）

```bash
# 启动一体化服务（包含网关、控制台和日志服务）
cargo run -r --bin aiway
```

#### 方式二：独立组件启动

```bash
# 启动日志服务
cargo run -r --bin logg

# 启动控制台
cargo run -r --bin console

# 启动网关
cargo run -r --bin gateway
```

### 访问界面

- **管理控制台**: http://127.0.0.1:6000
- **API 网关**: http://127.0.0.1:5000
- **日志服务**: http://127.0.0.1:7281

## 部署模式

### 单机模式

适用于中小规模应用场景(QPS < 10000)。

```bash
# 构建单机版本
cargo build -r --bin gateway -F standalone && \
cargo build -r --bin console -F standalone && \
cargo build -r --bin logg && \
cargo build -r --bin aiway
```

### 集群模式

适用于生产环境，支持水平扩展和高可用部署。

组件依赖：

- 缓存：Redis
- 数据库：MySQL
- 日志：Quickwit（可选）

```bash
# 构建集群版本
cargo build -r --bin gateway -F cluster
cargo build -r --bin console -F cluster
```

## 技术栈

- **语言**: Rust
- **Web框架**: Rocket
- **搜索**: Tantivy（日志索引）
- **数据库**: MySQL / SQLite
- **缓存**: Redis / 内置缓存
- **序列化**: Serde
- **前端**: Vue.js（控制台）

## License

Apache License 2.0

## 截图

![Dashhoard](docs/images/1.png)