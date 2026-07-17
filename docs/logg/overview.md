# 日志服务

日志服务（Logg）基于 [Tantivy](https://github.com/quickwit-oss/tantivy) 全文检索引擎构建。

## 功能

- **通用日志**：接收应用日志并建立全文索引（索引名: aiway-logs）
- **请求日志**：接收网关请求日志并建立索引（索引名: request-logs）
- **REST API**：兼容 Quickwit 的查询接口

## 启动

```bash
./logg
```

## API

| 方法   | 路径                         | 说明   |
|------|----------------------------|------|
| POST | `/api/v1/{index}/ingest`   | 写入日志 |
| GET  | `/api/v1/{index}/search`   | 搜索日志 |
