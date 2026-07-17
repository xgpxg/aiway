# WASM 插件

插件编译为 WASM 模块后由网关加载执行。

## 编译

```bash
# 添加到 Cargo.toml
[lib]
crate-type = ["cdylib"]

# 构建 WASM
cargo build --target wasm32-wasip1 --release
```

## 编译环境

- 使用 `wasm32-wasip1` 目标
- 限制：无网络 I/O、无文件系统访问
- 通过 Host API 与网关通信

## 上传

在控制台插件管理页面上传 `.wasm` 文件，配置插件参数后自动同步到网关。

## 安全

- WASM 在沙箱中执行
- 无直接主进程内存访问
- 通过 bincode 实现结构化数据交换
- 插件异常不影响网关稳定性

## 限制

- 不支持标准 async I/O（WASM 内使用 `block_on` 处理异步）
- 插件 `Future` 必须立即返回，不可 `Pending`
- 如需复杂网络操作，使用宿主提供的 HTTP 请求接口
