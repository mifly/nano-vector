# 编译与调试指南 - Nano Vector

本端侧文本检索应用基于 **Flutter** 和 **Rust** 构建，利用 `candle` 框架实现本地向量推理。本文档说明了如何搭建开发环境、编译项目以及处理常见问题。

## 1. 环境准备

### 基础工具
*   **Rust**: 建议版本 1.75+ (本项目使用 2024 edition 预览版或 stable)。
*   **Flutter SDK**: 建议版本 3.24.0+。
*   **Protoc**: LanceDB 编译需要 protobuf 编译器 (`protoc`)。
*   **C 编译器**: Linux 下需要 `clang` 和 `cmake`。

### 安装依赖 (Ubuntu/Debian 示例)
```bash
# 安装编译工具
sudo apt-get update
sudo apt-get install -y clang cmake ninja-build pkg-config libgtk-3-dev liblzma-dev libstdc++-12-dev

# 安装 protoc (建议版本 25+)
curl -LO https://github.com/protocolbuffers/protobuf/releases/download/v25.1/protoc-25.1-linux-x86_64.zip
unzip protoc-25.1-linux-x86_64.zip -d ~/.local
export PATH="$HOME/.local/bin:$PATH"
```

## 2. 项目结构

*   `/rust_engine`: 纯 Rust 实现的核心逻辑与命令行工具 (CLI)。
*   `/nano_vector_app`: 完整的 Flutter + Rust 混合开发移动/桌面应用。
    *   `/nano_vector_app/rust`: Rust FFI 逻辑层。
    *   `/nano_vector_app/lib`: Flutter UI 展现层。

## 3. 编译步骤

### 3.1 纯 Rust CLI 验证
如果你只想在命令行测试模型推理：
```bash
cd rust_engine
# 索引文本
cargo run -- index --text "你的示例文本"
# 语义搜索
cargo run -- search --query "搜索词"
```

### 3.2 Flutter + Rust 应用编译 (Linux 桌面)
在 `nano_vector_app` 目录下执行：
```bash
# 生成 FFI 绑定代码 (仅当修改了 rust/src/api/*.rs 时需要)
flutter_rust_bridge_codegen generate

# 编译运行
export PROTOC=$HOME/.local/bin/protoc  # 确保编译能找到 protoc
flutter run -d linux
```

## 4. 调试与常见问题

### 4.1 模型下载失败
应用启动后会从 HuggingFace 自动下载 `bge-small-zh-v1.5` 模型文件。
*   **现象**: 界面长时间卡在 "Loading Weights..." 或控制台报错。
*   **原因**: 默认从 `huggingface.co` 下载，受限于网络环境。
*   **解决**: 
    1.  配置代理或使用镜像。
    2.  手动下载 `config.json`, `tokenizer.json`, `model.safetensors` 到本地缓存目录 (默认为 `~/.cache/huggingface`)。

### 4.2 数据库路径
在开发调试时，应用数据保存在：
*   **Linux**: `nano_vector_app/data/` (本项目测试代码配置)。
*   **生产环境**: 通常在 App 的 ApplicationDocumentsDirectory 下。

### 4.3 多线程安全 (Send/Sync)
由于 Rust 中的 `rusqlite::Connection` 默认不支持 `Sync`，但在 Flutter 的异步 Handler 中必须满足 `Send + Sync`。
*   **方案**: 本项目使用了 `tokio::sync::Mutex` 对 `DatabaseManager` 进行封装，确保在异步调用中安全访问。

### 4.4 FRB 绑定生成
如果添加了新的 Rust 函数但 Flutter 侧无法调用：
```bash
flutter_rust_bridge_codegen generate
```
确保 `rust/src/api/simple.rs` 中的函数是 `pub` 且参数类型可序列化。

## 5. 关键依赖版本
*   `flutter_rust_bridge`: `2.11.1`
*   `lancedb`: `0.26.2`
*   `candle-core`: `0.9.2`
*   `arrow-array`: `57.3.0`
