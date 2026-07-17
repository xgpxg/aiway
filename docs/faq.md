# FAQ

## 如何快速启动？

```bash
./aiway
```
访问 http://127.0.0.1:7000，默认账号 admin/admin。

## 如何添加路由？

在控制台 → 路由管理 → 创建路由，填写域名、路径、目标服务后保存。

## 插件支持哪些语言？

任何可编译为 WASM 的语言。推荐使用 Rust + `aiway-plugin` SDK。

## 如何启用 HTTPS？

在控制台 → 域名管理 → 上传证书，Access 层会自动使用证书进行 TLS 终止。

## 如何配置集群？

参考 [集群部署](/deployment/cluster) 文档，需要 MySQL、Redis 和可选 Quickwit。

## 为什么要使用 L4 透传？

L4 透传性能优于 L7，且不限制应用层协议。配合 TLS 终止，可在降低后端复杂度的同时保证安全性。
