# 集群部署

当需要支持大流量时，建议使用集群部署。

目前，aiway仅支持对无状态的组件进行集群，如：网关组件`gateway`、接入组件`access`，这类组件可无限水平扩展，支持动态扩容。

而对于有状态的组件，如：控制台`console`、日志服务`logg`，则仍以单节点的模式提供服务。

> `console`和`logg`即使宕机，也不会影响网关。

## 1. 下载部署包

```shell
wget https://github.com/xgpxg/aiway/releases/latest/download/aiway-linux-amd64-cluster.tar.gz
```

> 如果需要其他平台的部署包，请在 [发布页面](https://github.com/xgpxg/aiway/releases) 中下载。

## 2. 解压

```shell
tar -zxvf aiway-linux-amd64-cluster.tar.gz
```

## 3. 启动

解压完成后，会有3个文件：

- console: 控制台服务，仅支持单节点
- gateway: 网关核心服务，无状态，可水平扩展
- access: 接入层组件，用于将请求转发到网关和TLS终止，无状态，可水平扩展

启动控制台：

```shell
./console --address 0.0.0.0 --port 7000
```

> 控制台建议只在内网访问，不要暴露到公网。

在多台服务器上启动网关gateway，并指定控制台地址：

```shell
# 服务器1
./gateway --address 0.0.0.0 --port 7001 --console <控制台IP:PORT>

# 服务器2
./gateway --address 0.0.0.0 --port 7001 --console <控制台IP:PORT>

# 服务器3
./gateway --address 0.0.0.0 --port 7001 --console <控制台IP:PORT>
```

启动接入组件access，并指定控制台地址：

```shell
./access --address 0.0.0.0 --port 7080 --console <控制台IP:PORT>
```

> 如果需要公网访问，建议监听端口为80或443，同时需要以root权限启动，或者设置允许access监听特权端口。

## 4. 访问地址

控制台：http://<控制台IP:PORT>

网关（通过access访问）：http://<access的IP:PORT>

控制台默认用户名/密码：admin/admin
