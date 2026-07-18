# TLS 终止

Access 层支持 TLS 终止，基于 OpenSSL 实现。

## 架构

```
客户端(HTTPS) → Access(:7443) TLS 终止 → 网关(:7001 明文)
```

## SNI 证书选择

根据客户端 TLS 握手时发送的 SNI（Server Name Indication）域名动态选择证书。

### 匹配优先级

1. **精确匹配**：`www.example.com`
2. **通配符匹配**：`*.example.com`

未匹配到证书时，TLS 握手失败。

## 证书管理

证书通过控制台上传和管理：

- 支持 PEM 格式
- 支持证书链（叶子证书 + 中间 CA）
- 上传后自动同步到所有 Access 节点
- 定时检查证书变更

## ALPN

TLS 握手时仅协商 HTTP/1.1，不启用 h2。
