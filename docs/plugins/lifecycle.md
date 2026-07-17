# 插件生命周期

## 加载流程

1. **上传**：通过控制台上传 WASM 文件
2. **分发**：配置同步到网关节点
3. **实例化**：`plugin-manager` 使用 wasmtime 加载 WASM 模块
4. **执行**：在每个请求的对应生命周期阶段调用插件函数
5. **卸载**：插件配置删除后自动卸载

## 执行顺序

### 请求阶段

1. 全局插件 `on_request`
2. 路由匹配
3. 路由插件 `on_request`
4. 路由插件 `on_request_body`
5. 发送请求到后端

### 响应阶段

1. 收到后端响应
2. 路由插件 `on_response`
3. 全局插件 `on_response`
4. 路由插件 `on_response_body`
5. 全局插件 `on_response_body`
6. 所有插件的 `on_logging`

### 模型代理扩展

当请求命中模型代理时，额外执行模型供应商专属插件。
