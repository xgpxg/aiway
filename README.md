### 简介
一个Rust实现的API网关，性能还算不错。

### 快速启动

```shell
cargo build --bin gateway -F standalone && \
cargo build --bin console -F standalone && \
cargo build --bin logg && \
cargo run --bin aiway
```

控制台: http://127.0.0.1:6000 (默认账号/密码：admin/admin)
网关: http://127.0.0.1:5000

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

![Dashhoard](docs/images/1.png)