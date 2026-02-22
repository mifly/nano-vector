# 发布编译与打包指南 - Nano Vector

本端侧文本检索应用基于 **Flutter** 和 **Rust** 构建。由于引入了 Rust 的 `candle` 推理引擎和 `flutter_rust_bridge` FFI 层，编译发布版本（Release Build）时 Rust 层的代码会被自动编译并以优化模式链接到应用中。

## 1. 编译前置准备

在执行发布编译之前，请确保已安装必要的平台工具（如 Xcode、Android SDK 等），并在 `nano_vector_app` 目录下执行：

```bash
cd nano_vector_app
# 清理缓存
flutter clean
# 获取依赖
flutter pub get

# 如果你修改过 rust/src/api/ 下的 Rust FFI 接口，请重新生成绑定代码
flutter_rust_bridge_codegen generate
```

> **提示**: Rust 层的 Release 编译默认会开启最高等级的编译器优化（LTO 和 -O3），首次编译耗时较长是正常现象。

---

## 2. 各平台编译命令

### 🍏 macOS (桌面版)
```bash
flutter build macos --release
```
*   **产物路径**: `build/macos/Build/Products/Release/nano_vector_app.app`
*   **注意**: 分发给其他用户前，请确保在 Xcode 的 `Signing & Capabilities` 中配置了正确的开发者签名，并检查 `Release.entitlements` 是否包含了网络访问和麦克风权限。

### 🤖 Android
**生成通用 APK (推荐用于分发测试):**
```bash
flutter build apk --release
```
*   **产物路径**: `build/app/outputs/flutter-apk/app-release.apk`

**生成 AAB (用于 Google Play 上架):**
```bash
flutter build appbundle --release
```

### 🍎 iOS
```bash
flutter build ipa --release
```
*   **产物路径**: `build/ios/archive/Runner.xcarchive` 及对应的 `.ipa` 文件。
*   **注意**: 必须在 macOS 环境下使用 Xcode 配置好开发者证书和 Provisioning Profile 才能成功构建。

### 🐧 Linux
```bash
flutter build linux --release
```
*   **产物路径**: `build/linux/x64/release/bundle/`

### 🪟 Windows
```bash
flutter build windows --release
```
*   **产物路径**: `build/windows/x64/runner/Release/`

---

## 3. 发布版本关键注意事项

1.  **性能优势**: Release 版本的 Rust 推理速度和向量检索性能比 Debug 版本快数倍甚至数十倍，因为启用了 Rust 编译优化。
2.  **包体积**: 由于集成了 `candle` 核心和 `tokenizers` C++ 库，包体积会略大。
3.  **权限验证**: 
    *   **语音输入**: 在 iOS/Android/macOS 上发布后，系统会在首次使用语音输入时弹出权限确认框。
    *   **模型下载**: 应用首次运行会访问 HuggingFace 下载模型，请确保网络权限配置正确（特别是 macOS 的沙盒配置）。
4.  **混淆 (Obfuscation)**:
    若需要混淆 Dart 代码，可以使用以下参数（注意：这不会混淆 Rust 符号）：
    ```bash
    flutter build apk --release --obfuscate --split-debug-info=./debug-info
    ```
