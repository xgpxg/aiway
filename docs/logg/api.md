# Logg API

日志服务提供 RESTful API，兼容 Quickwit 接口。

## 写入日志

```
POST /api/v1/{index}/ingest
```

请求体为 JSON 数组，包含多条日志记录。

## 搜索日志

```
GET /api/v1/{index}/search?query={query}
```

参数：

| 参数 | 说明 |
|------|------|
| query | 查询语句 |
| start_timestamp | 起始时间戳 |
| end_timestamp | 结束时间戳 |
| max_hits | 返回最大条数 |
| sort_field | 排序字段 |
| sort_order | 排序方向 (asc/desc) |

## 刷新索引

```
GET /api/v1/{index}/_refresh
```

强制提交内存中的文档到索引段，使其可被搜索。
