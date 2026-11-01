# 实际组件归属与许可材料

本流程补充文件级 SBOM，将实际 Qt 库／插件与 SDK 的 SPDX 声明关联。产物是可复核证据，不是自动许可选择或合规批准。正式发行完整性继续为 incomplete。

## 输入与来源

Qt 6.8 起提供构建对应的 [SPDX 清单](https://doc.qt.io/qt-6.8/licenses-used-in-qt.html)。工具只读取本次固定 Qt 6.8.2 SDK 的 sbom/ 中 qtbase、qtdeclarative、qtsvg 和 qttranslations 等运行时声明，排除 qttools 构建／测试工具。验证 SPDX 版本、模块名和 Qt 版本，保存原始文档 SHA-256、namespace 和稳定包引用。输出只保留必要字段，不复制包含开发机路径的上游构建注释。

`scripts/qt-license-sources.json` 固定官方 Qt 源码及 SPDX license-list-data 提交、许可文本 URL 和 SHA-256。下载内容放入 build/ 缓存；任何缓存或下载的哈希不符都会失败。不会跟随 SPDX 中任意地址下载材料。除 LICENSES/ 的标准文本，还包含 Qt GUI 平滑缩放算法与 SVG 的原始通知，以及系统依赖声明涉及的 LGPL-2.1 文本；上游 LicenseRef 原文和版权声明保留在声明 JSON 中。

Qt 运行时通知另随包提供 Qt Network 使用的 Public Suffix List 固定可编辑源码、MPL-2.0 原文和获取说明；收集器核对上游 blob、SHA-256 及许可字节。源码交付还验证该输入能重现 Qt 归档中的两种生成数据，见[对应源码材料](source-delivery.md)。此项不代替其他 Qt 模块的对应源码及替换核验。

English: Runtime notices include the pinned editable Public Suffix List used by Qt Network, its MPL-2.0 text and source information. Collection verifies the upstream blob, SHA-256 and license bytes. The source delivery separately checks both generated forms against the Qt archive; other Qt source and replacement obligations remain separately reviewed.

## 文件与组件关联

1. 对核验根内文件重新计算哈希，拒绝在 deployment-audit.json 之后被修改的文件。
2. macOS SDK 模式使用实际解析的 Qt SDK 相对路径和字节身份，不据此声称独立部署。
3. Linux 部署阶段记录 SDK 原文件路径／SHA-256 与 patchelf 后工件路径／SHA-256。后续双向校验记录，拒绝过期或篡改的复制证据。
4. Windows 优先按 SDK 文件字节身份关联。文件名唯一但字节不同的候选仅标记 filename-only-unverified-rewrite，并登记缺口，不能当作来源验证通过。
5. 通过 SPDX 的 CONTAINS 找到文件所有者，再递归解析 DEPENDS_ON，包括跨 Qt 仓库引用和内嵌第三方。缺失包引用保留为缺口。不从仓库总包的 CONTAINS 将全部工具错误纳入运行时。
6. 非原生 QML／翻译若没有逐文件声明，只关联到对应源码仓库范围，不推断该文件的许可证。没有 SPDX 文件记录的 SDK 原生库、Qt 范围外原生文件、NOASSERTION 许可及缺失文本均单独登记。

SDK 中实际文件可能与上游构建 SPDX 中的 SHA-1 不同，例如后续安装处理或签名；工具保留预期值、对比结果并登记差异，不自行推断原因。SDK 字节关联、部署重写记录和上游原始哈希是三种不同证据。

专用 macOS 打包流程在最终签名应用仍存活时生成部署关联，按精确资源字节或 arm64 section、UUID、导入版本对应 SDK，并将该报告加入实际文件 SBOM。比较不覆盖签名、load-command 布局及 LINKEDIT，范围与原生运行库专属通知见[原生运行库来源与通知](native-runtime-materials.md)。普通 SDK 模式不据此获得部署结论。

对于从 QML 目录迁移至 `Contents/PlugIns/quick/` 的插件，部署关联允许使用已审计的包内 QML 符号链接定位唯一 SDK 输入。生成和消费记录时均重新核对真实链接、审计目标及文件摘要，再执行相同的字节或 Mach-O 比较；来源缺失保留未验证，多个不同来源或链接变更直接拒绝。不能仅凭同名文件提升归属证据，也不消除上游 SDK 摘要差异或许可审查缺口。

Rust 继续保存完整宿主构建图，并额外从实际静态库入口 xdzip-qt-bridge 沿普通依赖边生成 runtime candidates，排除 build、dev 和 proc-macro 工具及其依赖。这是潜在运行时输入，不能证明每个包的机器码最终保留；链接器裁剪、原生 C/C++ 与编译器运行库还需进一步核验。

## 执行与交付

```sh
python3 scripts/prepare-runtime-notices.py --qt-prefix /path/to/Qt/6.8.2/gcc_64 \
  --output build/review/notices/Qt-runtime
python3 scripts/runtime_attribution.py --audit build/review/deployment-audit.json \
  --root build/review/linux --qt-prefix /path/to/Qt/6.8.2/gcc_64 \
  --deployment build/review/linux-deployment.json --output build/review/runtime-attribution.json
python3 scripts/sbom.py --audit build/review/deployment-audit.json \
  --source build/review/source-build.cdx.json --attribution build/review/runtime-attribution.json \
  --output build/review/artifact-files.cdx.json
```

Linux 传入部署重写记录；其他平台省略 --deployment。报告写在核验根外，避免自身清单递归。CI 在部署／最终哈希之前准备许可材料，Windows 和 Linux 将 notices 放入实际运行目录，Linux tar.gz 同时交付这些材料。普通 macOS CI 只做 SDK 读取与日志／JSON 记录；经明确授权的专用 Release 工作流另行部署、ad-hoc 签名、打包并核对实际发布材料，按[发布协议](release-protocol.md)保留独立的部署范围与哈希，不以 SDK 记录替代最终应用包归属。

runtime-attribution.json 包含文件、组件、关系、上游声明、版权、文本来源与缺口。artifact-files.cdx.json 增加相同组件与已知所有者关系，并绑定该报告和同一实际审计的哈希／构建身份。官方 CycloneDX schema 与引用检查继续执行，compositions 仍为 incomplete。

默认检查允许明确登记的开发审查缺口，禁止把它们静默标记通过；身份、路径、字节、版本、复制记录或许可文本哈希错误直接失败。--require-complete 是发行阻断检查，当前会因未完成归属或发行材料而失败。测试覆盖文件被改写、过期复制记录、路径逃逸、版本错误、跨文档缺失、部分 SBOM 及 Rust 工具排除。

## 尚未解决的发行门槛

已增加[源码交付收集器](source-delivery.md)，保存实际项目源码、Rust vendor、四份运行时 Qt 模块归档及两份构建模块归档、重建及动态替换说明。它不能证明 SDK 与上游逐项构建等价。每次发行还须核对精确静态链接归属、操作系统／再分发运行库、所有内嵌资源逐项许可、开源许可分支选择及履约。[本地 Mac 部署核验](macos-deployment.md)中的元数据修改替换，与[实际源码 Qt 构建及加载／替换](wp12-automation.md#qt-源码与兼容替换)是不同流程，分别保留证据；均不能推断任意 SDK 的字节级构建等价。源清单与 SPDX 字段不能替代许可义务。已登记候选的来源与许可结论见[发行审查 #4](https://github.com/xiaodaozhi/xdzip/issues/4)，后续候选单独复核，付费签名身份不作为门槛；既有桌面验收在 [#5](https://github.com/xiaodaozhi/xdzip/issues/5)保留。

[Qt SDK 对应核验](qt-sdk-correspondence.md)进一步记录固定归档提交、SDK 源码声明逐文件哈希、安装配置与已知 AGL 消费端补丁。声明覆盖范围内匹配不等于完整 SDK 构建等价，原有发行阻断保持不变。

自动化收尾补充实际链接器 map／静态库成员、运行时资源哈希、可信原生依赖版本信息及精确可逆的二进制差异分析，见[完整契约](wp12-automation.md)。只在重建出的字节哈希精确匹配上游声明时标记转换已解释，原始 SDK 差异不删除；未匹配的发布者历史不能凭猜测补写。
