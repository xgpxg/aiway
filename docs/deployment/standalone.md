# 单机部署

单机模式使用本地缓存 + SQLite，不依赖外部中间件，适合测试和小规模部署。

## 一体化运行

直接运行 `aiway` 二进制，自动内嵌并启动所有子进程：

```bash
cargo build --release --bin aiway
# 或下载预编译版本

./aiway
# 默认监听 127.0.0.1:7000 (控制台)，:7001 (网关)
```

## 分别运行

各组件可独立启动，便于调试：

```bash
# 1. 启动日志服务
cargo run --release --bin logg

# 2. 启动控制台
cargo run --release --bin console

# 3. 启动网关
cargo run --release --bin gateway -- --console 127.0.0.1:7000

# 4. 启动接入层（可选）
cargo run --release --bin access -- --console 127.0.0.1:7000
```

## 数据目录

所有数据存储在 `~/.aiway/` 目录下：

```
~/.aiway/
├── data/
│   ├── sqlite/    # 数据库
│   ├── cache/     # 缓存
│   ├── file/      # 文件存储
│   └── temp/      # 临时文件
├── logs/          # 日志
└── resources/     # 资源文件
```
