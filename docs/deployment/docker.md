# Docker 部署

项目提供了 [`run-docker.sh`](https://github.com/xgpxg/aiway/blob/master/run-docker.sh) 脚本简化 Docker 部署。

## 使用脚本

```bash
# 构建并启动
./run-docker.sh

# 停止容器
./run-docker.sh --stop

# 重启容器
./run-docker.sh --restart

# 查看日志
./run-docker.sh --logs
```

## 手动部署

```bash
# 构建项目
cargo build --release --bin aiway

# 运行容器
docker run -d \
  --name aiway \
  --restart unless-stopped \
  -p 7000:7000 \
  -p 7001:7001 \
  -p 7281:7281 \
  -v ./target/release/aiway:/opt/aiway/aiway:ro \
  -v ./docker-data:/root/.aiway \
  openeuler/openeuler:24.03-lts \
  /opt/aiway/aiway --address 0.0.0.0 --port 7000 --gateway-port 7001
```

> 默认使用 openEuler 24.03 LTS 镜像，也可替换为其他 Linux 发行版镜像。
