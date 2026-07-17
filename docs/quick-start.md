# 快速开始

## 预编译版本

```bash
curl -L https://github.com/xgpxg/aiway/releases/latest/download/aiway-linux-amd64-standalone.tar.gz | tar -zxvf - -C .
./aiway
```

> 预编译版本基于 glibc 2.34 构建。若系统 glibc 版本低于 2.34，请从源码构建。

## 源码构建

### 环境要求

- Rust 最新稳定版
- OpenSSL 开发库
- Node.js 20+（构建前端）

### 构建步骤

```bash
# 构建全部二进制
cargo build --release --bin gateway
cargo build --release --bin console
cargo build --release --bin logg
cargo build --release --bin access
cargo build --release --bin aiway

# 运行（一体化模式）
cargo run --release --bin aiway
```

## 访问服务

| 服务 | 地址 |
|------|------|
| 管理控制台 | http://127.0.0.1:7000 |
| 网关入口 | http://127.0.0.1:7001 |
| 日志服务 | http://127.0.0.1:7280 |
| Access 入口 | http://127.0.0.1:7080 |
| Access HTTPS | https://127.0.0.1:7443 |

默认账号: `admin` / `admin`

## 分别运行（调试模式）

```bash
# 分别启动各组件
cargo run --bin logg      # 先启动日志服务
cargo run --bin console   # 管理控制台
cargo run --bin gateway   # 网关服务
cargo run --bin access    # 接入层（可选）
```
