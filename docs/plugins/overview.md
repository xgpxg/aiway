# 插件开发

aiway 插件系统基于 WASM (WebAssembly) 技术。插件编译为 WASM 模块，在沙箱环境中安全执行。

## 架构

```
┌─────────────────────────────┐
│        网关 (Host)           │
│  ┌───────────────────────┐  │
│  │   plugin-manager      │  │
│  │   (wasmtime 运行时)    │  │
│  └───────┬───────────────┘  │
│          │  WASM 调用        │
│  ┌───────▼───────────────┐  │
│  │   WASM 插件 (Sandbox)  │  │
│  │   - 无网络访问          │  │
│  │   - 无文件系统访问      │  │
│  │   - 通过 Host API 通信 │  │
│  └───────────────────────┘  │
└─────────────────────────────┘
```

## 技术选型

- **运行时**: wasmtime（Rust WASM 运行时）
- **编译目标**: `wasm32-wasip1`
- **序列化**: bincode（Host 与 WASM 间通信）
- **SDK**: `aiway-plugin` crate

## 官方插件

官方插件仓库：[aiway-plugins](https://github.com/xgpxg/aiway-plugins)
