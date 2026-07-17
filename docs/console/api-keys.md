# 密钥管理

管理 API Key，用于网关鉴权。

## 功能

- **创建密钥**：生成新的 API Key
- **启用/禁用**：控制密钥是否可用
- **删除密钥**：移除密钥
- **加密存储**：使用 ChaCha20Poly1305 算法加密 API Key

## 使用

在控制台创建 API Key 后，将其配置到客户端，在请求头中加入 `Authorization: Bearer <api_key>`。
