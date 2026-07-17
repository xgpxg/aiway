# 集群部署

## 架构

```
          ┌──────────┐
          │  Nginx   │  负载均衡 / 反向代理
          │  (可选)  │
          └────┬─────┘
               │
     ┌─────────┼─────────┐
     │         │         │
┌────▼──┐ ┌───▼───┐ ┌───▼───┐
│Access │ │Access │ │Access │  L4 透传 (可选)
└───┬───┘ └───┬───┘ └───┬───┘
    │         │         │
┌───▼───┐ ┌───▼───┐ ┌───▼───┐
│Gateway│ │Gateway│ │Gateway│  网关节点 (多实例)
└───┬───┘ └───┬───┘ └───┬───┘
    │         │         │
    └─────────┼─────────┘
              │
        ┌─────▼─────┐
        │  Console  │  管理控制台 (主备)
        └─────┬─────┘
              │
        ┌─────▼─────┐
        │   MySQL   │  共享数据库
        └───────────┘

    ┌──────────────┐
    │    Redis     │  共享缓存
    └──────────────┘

    ┌──────────────┐
    │   Quickwit   │  日志存储
    └──────────────┘
```

## 组件依赖

| 组件 | 依赖 |
|------|------|
| Console | MySQL + Redis |
| Gateway | Redis (缓存) + Console (配置拉取) |
| Access | Console (节点发现) |
| Logg | 本地文件系统 (Tantivy) |

## 部署步骤

### 1. 数据库

初始化 MySQL 数据库，Console 启动时自动执行迁移。

### 2. 启动 Console

```bash
./console --address 0.0.0.0 --port 7000 \
  --db-url "mysql://user:pass@host:3306/aiway" \
  --log-server "logg-host:7280"
```

### 3. 启动 Gateway

```bash
./gateway --address 0.0.0.0 --port 7001 \
  --console "console-host:7000" \
  --log-server "logg-host:7280"
```

### 4. 启动 Access（可选）

```bash
./access --address 0.0.0.0 --port 7080 \
  --https-port 7443 \
  --console "console-host:7000" \
  --log-server "logg-host:7280"
```

网关节点从 Console 拉取配置（路由、服务、插件），通过控制台界面统一管理。
