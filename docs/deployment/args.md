# 启动参数

## aiway（一体化启动）

```bash
./aiway [OPTIONS]

Options:
  -a, --address <ADDRESS>            控制台监听地址 [default: 127.0.0.1]
  -p, --port <PORT>                  控制台端口 [default: 7000]
      --gateway-port <GATEWAY_PORT>  网关端口 [default: 7001]
  -h, --help                         打印帮助
  -V, --version                      打印版本
```

## console（管理控制台）

```bash
./console [OPTIONS]

Options:
  -a, --address <ADDRESS>         监听地址 [default: 127.0.0.1]
  -p, --port <PORT>               端口 [default: 7000]
      --db-url <DB_URL>           数据库连接 URL [default: sqlite://~/.aiway/data/sqlite/main.db]
      --db-username <DB_USERNAME> 数据库用户名
      --db-password <DB_PASSWORD> 数据库密码
      --log-server <LOG_SERVER>   日志服务地址 [default: 127.0.0.1:7280]
```

## gateway（网关服务）

```bash
./gateway [OPTIONS]

Options:
  -a, --address <ADDRESS>     监听地址 [default: 127.0.0.1]
  -p, --port <PORT>           端口 [default: 7001]
  -c, --console <CONSOLE>     控制台地址 [default: 127.0.0.1:7000]
  -l, --log-server <LOG_SERVER> 日志服务地址 [default: 127.0.0.1:7280]
```

## access（接入层）

```bash
./access [OPTIONS]

Options:
  -a, --address <ADDRESS>        监听地址 [default: 0.0.0.0]
  -p, --port <PORT>              HTTP 端口 [default: 7080]
      --https-port <HTTPS_PORT>  HTTPS 端口，0 表示禁用 [default: 0]
  -c, --console <CONSOLE>        控制台地址 [default: 127.0.0.1:7000]
  -l, --log-server <LOG_SERVER>  日志服务地址 [default: 127.0.0.1:7280]
```

## logg（日志服务）

```bash
./logg [OPTIONS]

Options:
  -p, --port <PORT>  服务端口 [default: 7280]
```
