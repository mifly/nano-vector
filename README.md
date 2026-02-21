# Nano Vector - 端侧语义搜索应用

这是一个基于 Flutter 和 Rust 构建的端侧文本检索应用，支持将长文本自动分片、向量化并存储在本地。

## 核心特性
- **本地推理**: 使用 `candle` 框架和 `bge-small-zh-v1.5` 模型，无需联网即可生成 Embedding。
- **混合存储**: 
  - **SQLite**: 存储原始文本和分片元数据。
  - **sqlite-vec**: 高性能端侧向量扩展，支持语义相似度搜索。
- **跨平台**: 采用 `flutter_rust_bridge` (v2) 架构，逻辑在 Rust 层，界面在 Flutter 层。

## 项目结构
- `rust_engine/`: 纯 Rust 实现的核心逻辑验证 (含 CLI 工具)。
- `nano_vector_app/`: Flutter 移动/桌面端完整应用。
- `docs/`: 编译、调试与技术文档。

## 快速开始
详细的环境配置、编译步骤和调试建议，请参阅：
👉 [**编译与调试指南**](./docs/build_and_debug.md)

## 预览
1. **文本索引**: 输入文本 -> Rust 核心分词 -> 生成 512 维向量 -> 存入 sqlite-vec。
2. **语义检索**: 输入问题 -> 生成查询向量 -> 向量相似度检索 -> 返回相关文本分片。
