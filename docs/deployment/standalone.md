# 单机部署

单机模式无需任何依赖，适合中小规模（QPS ≤ 10K）场景。通常情况下，单机模式即可满足大部分需求。

## 1. 下载部署包

```shell
wget https://github.com/xgpxg/aiway/releases/latest/download/aiway-linux-amd64-standalone.tar.gz
```

如果需要其他平台的部署包，请在 [发布页面](https://github.com/xgpxg/aiway/releases) 中下载。

## 2. 解压

```shell
tar -zxvf aiway-linux-amd64-standalone.tar.gz
```

## 3. 启动

```shell
./aiway
```

以下为可选的启动参数：

| 参数                    |                说明 |              默认值 |
|-----------------------|------------------:|-----------------:|
| `--console-address`   |      Console 监听地址 |      `127.0.0.1` |
| `--console-port`      |      Console 监听端口 |           `7000` |
| `--gateway-address`   |      Gateway 监听地址 |      `127.0.0.1` |
| `--gateway-port`      |      Gateway 监听端口 |           `7001` |
| `--log-server`        |           日志服务器地址 | `127.0.0.1:7280` |
| `--access-address`    |       Access 监听地址 |        `0.0.0.0` |
| `--access-http-port`  |  Access HTTP 监听端口 |           `7080` |
| `--access-https-port` | Access HTTPS 监听端口 |           `7443` |

## 4. 访问地址

- 控制台：`http://127.0.0.1:7000`
- 网关（直接访问）：`http://127.0.0.1:7001`
- 网关（通过 access 访问）：`http://127.0.0.1:7080`

控制台默认用户名/密码：`admin` / `admin`

## 5. 域名接入（可选）

如果需要将网关提供给外部（如公网）访问，则需要将你的域名解析到以上服务部署的 IP 地址。并通过以下方式启动，以便外网访问：

```shell
sudo ./aiway --access-address=0.0.0.0 --access-http-port=80 --access-https-port=443
```
