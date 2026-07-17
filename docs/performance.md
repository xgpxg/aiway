# 性能测试

## 最新测试数据

通过 Access 接入点访问：

```
wrk https://aiway-test-1.coderbox.cn:7443/api/hello -t 12 -c 100
Running 10s test @ http://aiway-test-1.coderbox.cn:7080/api/hello
  12 threads and 100 connections
  Thread Stats   Avg      Stdev     Max   +/- Stdev
    Latency     0.99ms  810.80us  34.97ms   96.17%
    Req/Sec     8.43k     1.23k   16.35k    78.69%
  1012287 requests in 10.10s, 253.90MB read
Requests/sec: 100224.67
Transfer/sec:     25.14MB
```

## 历史测试数据

### Ubuntu 24.04, Intel i7-12700K, 12 Cores, 16 GB RAM

```
wrk http://127.0.0.1:7001/api/hello -t 12 -c 100
Running 10s test @ http://127.0.0.1:7001/api/hello
  12 threads and 100 connections
  Thread Stats   Avg      Stdev     Max   +/- Stdev
    Latency     0.89ms  780.09us  28.66ms   95.87%
    Req/Sec     9.54k     1.12k   18.36k    93.70%
  1144556 requests in 10.10s, 287.07MB read
Requests/sec: 113330.59
Transfer/sec:     28.43MB
```

### Ubuntu 22.04, 4 Cores, 8 GB RAM

```
wrk http://127.0.0.1:7001/hello -t 4 -c 100
Running 10s test @ http://127.0.0.1:7001/hello
  4 threads and 100 connections
  Thread Stats   Avg      Stdev     Max   +/- Stdev
    Latency     6.70ms    3.05ms  36.85ms   73.84%
    Req/Sec     7.61k   770.50     9.22k    64.50%
  302860 requests in 10.01s, 75.96MB read
Requests/sec:  30265.20
Transfer/sec:      7.59MB
```
