### 简介
一个Rust实现的API网关，性能还算不错。

### 快速启动

```shell
cargo build --bin gateway -F standalone && \
cargo build --bin console -F standalone && \
cargo build --bin logg && \
cargo run --bin aiway
```

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