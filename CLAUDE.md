# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Overview

nano-vector is a semantic text retrieval system using vector embeddings. It consists of two components:

1. **rust_engine/** - Standalone Rust CLI tool for text indexing and search (edition 2024)
2. **nano_vector_app/** - Flutter desktop/mobile app with the same functionality via flutter_rust_bridge

Both use the BAAI/bge-small-zh-v1.5 Chinese BERT model (downloaded from Hugging Face on first run) for generating 512-dimensional embeddings.

## macOS Development Setup

### Requirements
- **Rust**: 1.75+ (rust_engine uses 2024 edition, nano_vector_app/rust uses 2021)
- **Flutter SDK**: 3.24.0+
- **Xcode**: Full installation with Command Line Tools

### Install Dependencies
```bash
brew install cmake ninja
```

### Build & Run (macOS Desktop)
```bash
cd nano_vector_app
flutter pub get
flutter_rust_bridge_codegen generate  # Only after modifying rust/src/api/*.rs
flutter run -d macos
```

### Common Issues

**Model Download**: First run downloads `bge-small-zh-v1.5` (~100MB) to `~/.cache/huggingface/`. If download fails, manually download `config.json`, `tokenizer.json`, `model.safetensors`.

**macOS Network Permission**: If you see `Operation not permitted (os error 1)`, ensure `macos/Runner/*.entitlements` contains `com.apple.security.network.client`.

**permission_handler MissingPluginException**: The `permission_handler` plugin (used for microphone access in speech-to-text) is not implemented for macOS desktop. The speech-to-text feature only works on iOS/Android. On macOS, the error is non-fatal but will appear in logs.

**macOS Linker Errors**: For `Undefined symbols` errors involving `_SC*` or C++ symbols:
```bash
cd nano_vector_app/macos && rm -rf Pods Podfile.lock && pod install && cd .. && flutter clean
```

**FRB Binding Issues**: If new Rust API functions aren't callable from Flutter, regenerate bindings. Ensure functions are `pub` with serializable parameter types.

## Build Commands

### Rust Engine (CLI)
```bash
cd rust_engine
cargo build
cargo run -- index --text "示例文本"
cargo run -- search --query "搜索词" --limit 5
cargo test
```

### Flutter App
```bash
cd nano_vector_app
flutter pub get
flutter_rust_bridge_codegen generate    # After Rust API changes
flutter run -d macos                    # macOS desktop
flutter run -d linux                    # Linux desktop
flutter test                            # Widget tests
flutter test integration_test/          # Integration tests
```

### Flutter App Rust FFI
```bash
cd nano_vector_app/rust
cargo build
cargo test
```

## Architecture

### Data Flow
```
Text Input → Tokenize (bge-small-zh) → Chunk (200 tokens max) → Generate Embeddings → Store
                                                                        ↓
                                                   SQLite (documents, chunks) + sqlite-vec (vectors)

Query → Add Chinese prefix → Generate Query Embedding → Vector Search (sqlite-vec) → Return ranked chunks
```

### Key Components

**Rust Core (`embeddings.rs`, `db.rs`):**
- `TextEmbeddingModel` - Loads BERT model via candle, handles tokenization/embedding
- `DatabaseManager` - Manages SQLite (metadata) and sqlite-vec (vectors) via `rusqlite` with `load_extension`
- Query prefix for retrieval: `"为这个句子生成表示以用于检索相关文章："`
- Embeddings are L2-normalized (CLS token pooling)

**Flutter FFI (`nano_vector_app/rust/src/api/simple.rs`):**
- `AppCore` - Thread-safe wrapper using `tokio::sync::Mutex` exposing `index_text()` and `search_text()` to Dart
- `#[flutter_rust_bridge::frb(opaque)]` for opaque Rust types

**Flutter UI (`lib/`):**
- `main.dart` - App initialization, creates global `appCore` instance
- `index_page.dart` - Text input UI with speech-to-text (iOS/Android only)
- `search_page.dart` - Natural language search UI with results display

### Database Schema

**SQLite Tables:**
- `documents(id, content, created_at)` - Full document text
- `chunks(id, doc_id, chunk_text, chunk_index)` - Chunked text pieces

**sqlite-vec Virtual Table:**
- `vector_chunks` with `id` (INTEGER PRIMARY KEY) and `vector` (float[512])

### Dual Rust Implementations

The project has two separate Rust codebases with similar logic:
- `rust_engine/src/` - Standalone CLI with `clap` for argument parsing
- `nano_vector_app/rust/src/` - FFI library for Flutter with `flutter_rust_bridge`

When modifying core logic (embedding generation, chunking, vector search), changes may need to be applied to both.

## Dependencies

Key Rust crates:
- `candle-core/nn/transformers` - ML inference (requires opt-level=3 even in debug)
- `sqlite-vec` - Vector search extension for SQLite
- `rusqlite` - SQLite with `bundled` and `load_extension` features
- `hf-hub` - Hugging Face model downloads
- `tokenizers` - Text tokenization
- `flutter_rust_bridge` - FFI generation (Flutter app only)
