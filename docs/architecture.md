# 架构概览

## 系统架构

aiway 采用微服务架构，由多个独立二进制组件构成：

```
                     ┌─────────────┐
                     │   Access    │  L4 透传 + TLS 终止
                     │  :7080/7443 │
                     └──────┬──────┘
                            │ TCP 转发
                     ┌──────▼──────┐
                     │   Gateway   │  核心代理 (Pingora)
                     │   :7001     │  路由/负载均衡/鉴权/插件
                     └──────┬──────┘
                            │ HTTP 调用
              ┌─────────────┼─────────────┐
              │             │             │
       ┌──────▼──────┐ ┌───▼────┐ ┌─────▼──────┐
       │   Console   │ │  Logg  │ │  Backend   │
       │   :7000     │ │ :7280  │ │   Services │
       │  管理控制台  │ │ 日志服务│ │            │
       └─────────────┘ └────────┘ └────────────┘
```

## 组件职责

| 组件 | 二进制 | 说明 |
|------|--------|------|
| **aiway** | `aiway` | 一体化启动器，内嵌其他二进制 |
| **gateway** | `gateway` | 核心网关，基于 Pingora 框架，处理请求转发 |
| **console** | `console` | 管理控制台，基于 Rocket.rs，提供 Web UI 和 API |
| **logg** | `logg` | 日志服务，基于 Tantivy 全文检索引擎 |
| **access** | `access` | 接入层，L4 TCP 透传 + TLS 终止 (OpenSSL) |
| **test-server** | `test-server` | 测试服务器，用于开发和压测 |

## 库模块

| 库 | 路径 | 说明 |
|----|------|------|
| `common` | `lib/common` | 工具模块：目录管理、ID 生成 |
| `logging` | `lib/logging` | 日志框架：支持控制台/文件/远程推送 |
| `aiway-protocol` | `lib/aiway-protocol` | 协议定义：数据结构、上下文、请求/响应类型 |
| `busi` | `lib/busi` | 业务模型：请求/响应封装 |
| `cache` | `lib/cache` | 缓存：支持 Moka(本地)/Sled/Redis |
| `loadbalance` | `lib/loadbalance` | 负载均衡策略 |
| `alert` | `lib/alert` | 告警通知 |
| `aiway-plugin` | `lib/aiway-plugin` | 插件 SDK：定义 Plugin trait 和 WASM 接口 |
| `plugin-manager` | `lib/plugin-manager` | 插件管理器：加载/管理 WASM 插件 |
| `sdk` | `lib/sdk` | 客户端 SDK |

## 请求处理流程

网关的请求处理遵循 Pingora ProxyHttp 生命周期：

```
请求到达 → early_request_filter(预处理/防火墙)
                ↓
         request_filter(路由匹配/鉴权/插件/负载均衡)
                ↓
           upstream_peer(选择后端)
                ↓
         upstream_request_filter(修改请求头)
                ↓
          request_body_filter(修改请求体)
                ↓
           发送请求到后端 → 接收响应
                ↓
         upstream_response_filter(处理响应)
                ↓
       upstream_response_body_filter(修改响应体)
                ↓
              logging(日志记录/清理)
```
