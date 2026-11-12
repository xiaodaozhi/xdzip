# 图片预览输入

本目录为项目自行生成、按项目许可证再分发的 HEIC 回归输入，不含照片、用户元数据或用户文件。生成源码为 [generate.cpp](generate.cpp)，使用 Qt 6.8.2 的系统 ImageIO 写入器，在 macOS 27.2.1、Apple Silicon 上以质量 90 写入纯色 `#3492bd`。编码结果可能随系统版本改变；固定输入的身份以下表为准。

| ID／文件 | 字节 | 源尺寸 | SHA-256 | 预期 |
| --- | --- | --- | --- | --- |
| [owned-wide.heic](owned-wide.heic) | 670 | 320 × 180 | `6c7242586c96d5d718c18cd5d60842f10f933d140151954b91c5863d0a879258` | macOS 生成 512 × 288 缩略图，`.heic` 与 `.heif` 两种扩展均通过真实归档到属性查看器路径 |
| [owned-phone.heic](owned-phone.heic) | 12895 | 4032 × 3024 | `beae822663ad32a177f77c8b1fc2a89eb441667e49eece3f5bafc663c7d95fe2` | macOS 生成 512 × 384 缩略图，覆盖常见手机照片尺寸 |

关联验收为 [AC-13／CONTENT-PREVIEW](../../../docs/qa/acceptance.md)。解码回归使用固定输入，不要求 CI 有 HEVC 编码硬件；非 macOS 拒绝 HEIC／HEIF。输入仅用于预览路径与尺寸验证，不代表独立编解码互操作或照片色彩保真验收。

如需重建，在仓库根目录执行，先生成到本地工件目录，再核对输出及摘要后有意更新固定输入：

```sh
xcrun clang++ -std=c++17 -arch arm64 -mmacosx-version-min=14.0 \
  tests/fixtures/preview/generate.cpp \
  -isystem build/toolchain/Qt/6.8.2/macos/lib/QtCore.framework/Headers \
  -isystem build/toolchain/Qt/6.8.2/macos/lib/QtGui.framework/Headers \
  -F build/toolchain/Qt/6.8.2/macos/lib -framework QtCore -framework QtGui \
  -Wl,-rpath,"$PWD/build/toolchain/Qt/6.8.2/macos/lib" \
  -o build/review/generate-preview-heic
QT_QPA_PLATFORM=offscreen \
  QT_PLUGIN_PATH="$PWD/build/toolchain/Qt/6.8.2/macos/plugins" \
  build/review/generate-preview-heic build/review/preview-fixtures
shasum -a 256 build/review/preview-fixtures/*.heic
```
