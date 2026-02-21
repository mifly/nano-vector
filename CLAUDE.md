# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Overview

nano-vector is a semantic text retrieval system using vector embeddings. It consists of two components:

1. **rust_engine/** - Standalone CLI tool for text indexing and search
2. **nano_vector_app/** - Flutter mobile app with the same functionality via flutter_rust_bridge

Both use the BAAI/bge-small-zh-v1.5 Chinese BERT model (downloaded from Hugging Face on first run) for generating embeddings.

## macOS 开发环境配置

### 环境要求
- **Rust**: 1.75+ (项目使用 2024 edition)
- **Flutter SDK**: 3.24.0+
- **Xcode**: 需要完整安装 (包括 Command Line Tools)
- **protoc**: LanceDB 编译依赖

### 安装依赖
```bash
# 安装 Homebrew (如未安装)
/bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"

# 安装编译工具
brew install cmake ninja protobuf

# 验证 protoc 安装
protoc --version  # 建议 25+
```

### 编译运行 (macOS 桌面)
```bash
cd nano_vector_app

# 安装 Flutter 依赖
flutter pub get

# 生成 FFI 绑定 (修改 rust/src/api/*.rs 后需要)
flutter_rust_bridge_codegen generate

# 编译运行 macOS 桌面应用
flutter run -d macos
```

### 调试说明

**模型下载**: 首次运行会从 HuggingFace 下载 `bge-small-zh-v1.5` 模型 (~100MB)，存储在 `~/.cache/huggingface/`。如下载失败可配置代理或手动下载 `config.json`, `tokenizer.json`, `model.safetensors`。

**macOS 网络权限**: 如遇 `Operation not permitted (os error 1)` 错误，需确保 `macos/Runner/*.entitlements` 包含 `com.apple.security.network.client` 权限。

**数据目录**: 开发时数据保存在 `nano_vector_app/data/` (SQLite + LanceDB)。

**FRB 绑定问题**: 新增 Rust API 后 Flutter 侧无法调用时，重新执行 `flutter_rust_bridge_codegen generate`，确保函数为 `pub` 且参数类型可序列化。

**macOS 链接错误**: 如遇 `Undefined symbols` 错误涉及 `_SC*` 或 C++ 符号，需清理重建：
```bash
cd macos && rm -rf Pods Podfile.lock && pod install && cd .. && flutter clean
```

## Build Commands

### Rust Engine (CLI)
```bash
cd rust_engine
cargo build                             # Build
cargo run -- index --text "示例文本"     # Index text
cargo run -- search --query "搜索词" --limit 5  # Search
cargo test                              # Run tests
```

### Flutter App
```bash
cd nano_vector_app
flutter pub get                           # Install dependencies
flutter_rust_bridge_codegen generate      # Regenerate FFI bindings after Rust API changes
flutter run -d macos                      # Run macOS desktop app
flutter run -d linux                      # Run Linux desktop app
flutter test                              # Run widget tests
flutter test integration_test/            # Run integration tests
```

### Rust FFI (for Flutter app)
```bash
cd nano_vector_app/rust
cargo build              # Build native library
cargo test               # Run tests
```

## Architecture

### Data Flow
```
Text Input → Tokenize (bge-small-zh) → Chunk (200 tokens max) → Generate Embeddings → Store
                                                                        ↓
                                                   SQLite (documents, chunks) + LanceDB (vectors)

Query → Add Chinese prefix → Generate Query Embedding → Vector Search (LanceDB) → Return ranked chunks
```

### Key Components

**Rust Core (`embeddings.rs`, `db.rs`):**
- `TextEmbeddingModel` - Loads BERT model via candle, handles tokenization/embedding generation
- `DatabaseManager` - Manages SQLite (metadata) and LanceDB (vectors) connections
- Query prefix for retrieval: `"为这个句子生成表示以用于检索相关文章："`

**Flutter FFI (`nano_vector_app/rust/src/api/simple.rs`):**
- `AppCore` - Thread-safe wrapper exposing `index_text()` and `search_text()` to Dart
- Uses `#[flutter_rust_bridge::frb(opaque)]` for opaque Rust types

**Flutter UI (`lib/`):**
- `main.dart` - App initialization, creates global `appCore` instance
- `index_page.dart` - Text input and indexing UI
- `search_page.dart` - Natural language search UI with results display

### Database Schema

**SQLite:**
- `documents(id, content, created_at)` - Full document text
- `chunks(id, doc_id, chunk_text, chunk_index)` - Chunked text pieces

**LanceDB:**
- `vector_chunks` table with `id`, `text`, `vector` (512-dim float32 array)

## Dependencies

Key Rust crates:
- `candle-core/nn/transformers` - ML inference
- `lancedb` - Vector database
- `rusqlite` - SQLite
- `hf-hub` - Hugging Face model downloads
- `tokenizers` - Text tokenization
- `flutter_rust_bridge` - FFI generation (Flutter app only)
