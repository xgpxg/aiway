# 日志服务

日志服务 `logg` 用于接收并存储网关节点产生的所有日志，基于 [Tantivy](https://github.com/quickwit-oss/tantivy) 全文检索引擎构建。

## 功能

- 通用日志：接收应用日志并建立全文索引（索引名: aiway-logs）
- 请求日志：接收网关请求日志并建立索引（索引名: request-logs）
- 模型调用日志：接收模型调用日志并建立索引（索引名: model-call-logs）

## 启动

```bash
./logg
```

可选的启动参数：

```shell
./logg -h
Usage: logg [OPTIONS]

Options:
  -a, --address <ADDRESS>  Server address [default: 127.0.0.1]
  -p, --port <PORT>        Server port [default: 7280]
  -h, --help               Print help
  -V, --version            Print version

```
