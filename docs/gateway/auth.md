# 鉴权

网关支持基于 API Key 的 Bearer Token 认证。

## 认证流程

1. 客户端在请求头中添加 `Authorization: Bearer <api_key>`
2. 网关验证 API Key 格式和加密签名
3. 验证 API Key 是否存在于缓存中
4. 验证通过则放行，否则返回 `401 Unauthorized`

## API Key 管理

- 在控制台的密钥管理模块创建和管理
- 密钥使用 ChaCha20Poly1305 加密存储
- 变更后自动同步到网关缓存
- 支持禁用/删除密钥

## 白名单

路由可配置鉴权白名单，列表中的路径跳过鉴权，适用于公开 API。
