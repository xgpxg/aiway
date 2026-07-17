# 索引配置

日志服务使用两个索引分别存储通用日志和请求日志。

## aiway-logs（通用日志索引）

存储应用运行日志。

| 字段      | 类型        | 说明   |
|---------|-----------|------|
| time    | datetime  | 日志时间 |
| service | text(raw) | 服务名  |
| level   | text(raw) | 日志级别 |
| message | text      | 日志内容 |

## request-logs（请求日志索引）

存储网关请求日志。

| 字段              | 类型        | 说明      |
|-----------------|-----------|---------|
| request_id      | text(raw) | 请求 ID   |
| client_ip       | ip        | 客户端 IP  |
| client_country  | text      | 国家      |
| client_province | text      | 省份      |
| client_city     | text      | 城市      |
| method          | text(raw) | HTTP 方法 |
| path            | text      | 请求路径    |
| request_time    | datetime  | 请求时间    |
| response_time   | datetime  | 响应时间    |
| elapsed         | i64       | 耗时 (ms) |
| status_code     | u64       | 状态码     |
| response_size   | u64       | 响应大小    |
| user_agent      | text      | 客户端 UA  |
| referer         | text(raw) | 来源      |
| node_address    | text      | 节点地址    |

