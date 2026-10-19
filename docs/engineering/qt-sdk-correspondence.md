# Qt SDK 与源码对应核验

WP-12 的源码材料以官方归档的实际字节为准，不以相同版本号推断 SDK 构建等价。`scripts/qt_sdk_correspondence.py` 将固定源码归档、SDK 自带的源码 SPDX、构建声明、已安装配置与已知消费端补丁关联，输出到 build/；不修改 SDK，不批准许可分支。

```sh
python3 scripts/qt_sdk_correspondence.py \
  --qt-prefix .cache/Qt/6.8.2/macos \
  --output build/review/qt-sdk-correspondence.json
```

默认离线读取 build/source-cache，缺失则失败；显式 --fetch 才按 [固定清单](../../scripts/corresponding-sources.json)下载并校验官方归档。三平台 CI 安装固定 SDK 后运行 --fetch，将 JSON 与其他验证记录一起保留；Mac 仍只上传记录，不部署、签名或打包应用。源码交付收集器传入 --qt-prefix 时，将该记录与补丁、固定清单一起纳入交付哈希，见[源码交付](source-delivery.md)。

## 核验层次

1. 归档 SHA-256 必须匹配固定值，归档根目录、成员路径与 .tag 提交标记必须匹配。逐文件读取，不解包上游源码。
2. 若 SDK 提供模块的 .source.spdx，逐文件比较其 SHA-1 与归档内容。记录相同数量、不同文件双方哈希、SDK 声明中缺失于归档的文件及归档未覆盖数量。SHA-1 只复核上游原声明，整个归档另以固定 SHA-256 绑定。没有声明、声明差异及未覆盖文件不被补写为验证通过。
3. 保存 SDK 模块构建 SPDX 的哈希、编译器声明与 configure 参数，同时读取安装后的 modules/*.json、qconfig*.pri 和 QT_FEATURE 配置头，保存内容及哈希。输出将上游构建者的绝对目录替换为占位符，原始证据通过文件哈希绑定。配置不是完整 CMake cache、构建环境或工具链材料；SDK 默认架构字段不代表实际库的所有架构，不能据此把 universal 库误判为 x86_64。
4. [AGL 补丁](../../scripts/qt-patches/macos-no-agl.patch)只修改 SDK 的 lib/cmake/Qt6/FindWrapOpenGL.cmake，用于本项目消费 Qt 时避免链接旧 AGL。工具从固定 QtBase 归档读取补丁原文，重放统一差异，验证补丁、原文、结果三个 SHA-256，再检查实际 SDK 文件。只接受原版或已知补丁版；Windows SDK 的 CRLF 仅转换行尾后比较，同时保留实际字节 SHA-256、LF SHA-256 与转换标记，其他未知修改直接失败。它不改变 Qt 运行库的机器码，不能当作 Qt 已重编译的证据。完整 SDK 修改历史仍待核验。

源码对应记录中的 archive_commit 来自固定归档 .tag。qt-license-sources.json 的 source_commits 仅表示许可文本的固定取回提交，两者分别保存；例如 QtDeclarative 的许可来源提交与官方 6.8.2 归档提交不同，不应互相覆盖或冒充 SDK 编译提交。许可证文本本身仍按原有 SHA-256 检查。

## 结果解释与发行边界

模块是否提供源码 SPDX、匹配／缺失／未覆盖数量、配置和补丁状态均以本次生成的 JSON 为准，不把某个 SDK 的观测推广到其他 SDK。没有逐文件声明时不能推断对应通过，必须保留具体缺口。

报告状态固定为 evidence-collected-not-build-equivalence。[实际 Qt 源码构建和加载／替换](wp12-automation.md#qt-源码与兼容替换)已有独立流程，但不能由此推断原 SDK 的字节级重现。完整工具链与系统 SDK 材料、未覆盖／生成文件、运行库来源及许可履约尚未全部核清时，不宣称全部对应源码义务已履行，不宣称字节级可重复构建。既有[运行时归属](runtime-attribution.md)的 incomplete、许可选择、平台运行库及独立审查门槛继续保留。
