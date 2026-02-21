# Nano Vector

## Project Overview

Nano Vector is a local, on-device semantic text retrieval application built with Flutter and Rust. It allows users to input long texts, automatically chunks and vectorizes them, and stores them locally for semantic similarity search.

The core machine learning and database operations are handled by Rust, ensuring high performance, while the user interface is built with Flutter for cross-platform support.

### Key Technologies
*   **Flutter**: UI framework for cross-platform app development (Mobile/Desktop).
*   **Rust**: Core engine for ML inference and database management.
*   **flutter_rust_bridge (FRB v2)**: Facilitates seamless communication between Dart (Flutter) and Rust.
*   **Candle (`candle-core`)**: A minimalist ML framework for Rust, used here to run the `bge-small-zh-v1.5` BERT model for generating text embeddings locally without internet access.
*   **LanceDB**: High-performance vector database used to store and query the generated embeddings.
*   **SQLite (`rusqlite`)**: Relational database used to store original document text and chunk metadata.

## Directory Structure

*   `rust_engine/`: A standalone Rust project containing the core text indexing and search logic. It includes a CLI tool for testing the engine without the Flutter UI.
*   `nano_vector_app/`: The main Flutter application.
    *   `nano_vector_app/lib/`: Dart code for the Flutter UI (`main.dart`, `index_page.dart`, `search_page.dart`).
    *   `nano_vector_app/rust/`: The Rust FFI (Foreign Function Interface) layer that exposes the core engine functionality to Dart via `flutter_rust_bridge`.
*   `docs/`: Contains additional documentation, such as build and debug guides.
*   `rust_builder/`: Contains native building configurations for the Rust FFI integration.

## Building and Running

### Prerequisites
*   **Rust**: 1.75+ (using 2024 edition).
*   **Flutter SDK**: 3.24.0+.
*   **protoc**: Protocol Buffers compiler (required by LanceDB). Recommended version 25+.
*   **C/C++ Build Tools**: Xcode Command Line Tools on macOS; `clang`, `cmake`, etc. on Linux.

### Running the Flutter App (macOS Desktop example)
```bash
cd nano_vector_app
flutter pub get

# Generate FFI bindings (Required if you modify Rust API in nano_vector_app/rust/src/api/*.rs)
flutter_rust_bridge_codegen generate

# Run the app
flutter run -d macos
```
*(For Linux, use `flutter run -d linux`)*

### Running the Rust CLI Engine (Standalone)
Useful for testing the core logic without launching the Flutter app.
```bash
cd rust_engine
cargo build

# Indexing example
cargo run -- index --text "Your sample text here"

# Searching example
cargo run -- search --query "Your search query" --limit 5
```

## Development Conventions & Notes

*   **Model Download**: On the first run, the app will download the `bge-small-zh-v1.5` model (~100MB) from Hugging Face to the local cache directory (`~/.cache/huggingface/`). Ensure a stable internet connection for the initial setup.
*   **FFI Code Generation**: Whenever changes are made to the Rust functions exposed to Dart (typically in `nano_vector_app/rust/src/api/`), you **must** run `flutter_rust_bridge_codegen generate` in the `nano_vector_app/` directory to update the bindings.
*   **Data Storage**: During development, databases (SQLite and LanceDB) are stored in `nano_vector_app/data/`.
*   **macOS Network Permissions**: If the model fails to download on macOS with an `Operation not permitted` error, ensure the app's entitlements (`macos/Runner/*.entitlements`) include the `<key>com.apple.security.network.client</key>` with a `<true/>` value.
*   **Concurrency**: The Rust SQLite connection (`rusqlite::Connection`) is wrapped in a `tokio::sync::Mutex` within the `DatabaseManager` to ensure it is `Send + Sync` safe for Flutter's asynchronous handlers.
