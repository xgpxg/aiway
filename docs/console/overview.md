# 管理控制台

控制台基于 [Rocket.rs](https://rocket.rs/) 框架构建，提供 Web UI 和 RESTful API 用于管理网关系统。

## 功能模块

| 模块 | 路径 | 说明 |
|------|------|------|
| 用户管理 | `/api/user/*` | 用户登录、账号管理 |
| 路由管理 | `/api/route/*` | 路由配置 CRUD |
| 服务管理 | `/api/service/*` | 后端服务管理 |
| 密钥管理 | `/api/key/*` | API Key 管理 |
| 插件管理 | `/api/plugin/*` | 插件 CRUD |
| 监控指标 | `/api/metrics/*` | 实时监控数据 |
| 日志查询 | `/api/log/*` | 日志搜索 |
| 域名管理 | `/api/domain/*` | 域名证书管理 |
| 防火墙 | `/api/firewall/*` | 安全规则配置 |
| 模型管理 | `/api/model/*` | AI 模型配置 |
| MCP 管理 | `/api/mcp/*` | MCP 服务器配置 |
| 消息通知 | `/api/message/*` | 通知消息 |
| 网关节点 | `/api/node/*` | 网关节点管理 |
| 系统设置 | `/api/system/*` | 系统配置 |

## 启动

```bash
./console --address 127.0.0.1 --port 7000
```

## 存储

单机模式使用 SQLite，集群模式使用 MySQL。首次启动自动执行数据库迁移。
