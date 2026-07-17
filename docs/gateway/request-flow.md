# 请求处理流程

网关基于 Pingora ProxyHttp 生命周期，每个请求经过以下阶段：

## 1. early_request_filter（预处理）

- 请求计数
- 防火墙校验（IP、Referer）
- 被拦截的请求直接返回 403，不记录日志

## 2. request_filter（请求过滤）

执行顺序：

1. **全局请求插件**：执行全局配置的 WASM 插件
2. **路由匹配**：根据 host + method + path 匹配路由规则
3. **鉴权**：验证 Bearer Token
4. **路由请求插件**：执行路由级别插件
5. **负载均衡**：选择后端服务实例

## 3. upstream_peer（选择上游）

根据负载均衡结果构建后端连接地址，支持 TCP 和 Unix Socket。

## 4. upstream_request_filter（上游请求过滤）

移除 `Authorization` 头，设置正确的 Host 头后转发。

### 4.5 request_body_filter（请求体过滤）

执行插件修改请求体。

## 5. upstream_response_filter（上游响应过滤）

记录响应头信息，执行响应阶段插件。

### 5.5 upstream_response_body_filter（响应体过滤）

执行响应体插件，记录响应体大小。

## 6. logging（日志记录）

记录完整请求日志，清理连接计数，执行插件的 logging 阶段。

## 错误处理

当后端不可达或插件执行失败时，进入 `fail_to_proxy` 阶段，返回对应错误码并触发告警。
