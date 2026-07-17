# 快速开始

## 预编译版本

```bash
curl -L https://github.com/xgpxg/aiway/releases/latest/download/aiway-linux-amd64-standalone.tar.gz | tar -zxvf - -C .
./aiway
```

## 源码构建

### 环境要求

- Rust 最新稳定版
- OpenSSL 开发库
- Node.js 20+（构建前端）

### 构建步骤

```bash
# 构建全部二进制包
cargo build -r --bin gateway
cargo build -r --bin console
cargo build -r --bin logg
cargo build -r --bin access
cargo build -r --bin aiway

# 运行（单机模式）
cargo run --r --bin aiway
```

## 访问地址

| 服务    | 地址                    |
|-------|-----------------------|
| 管理控制台 | http://127.0.0.1:7000 |
| 网关入口  | http://127.0.0.1:7001 |

默认账号: `admin` / `admin`

## 分别运行（调试模式）

```bash
# 分别启动各组件
cargo run --bin logg      # 先启动日志服务
cargo run --bin console   # 管理控制台
cargo run --bin gateway   # 网关服务
cargo run --bin access    # 接入层（可选）
```
