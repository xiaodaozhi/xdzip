# 实际组件归属与许可材料

本流程补充文件级 SBOM，将实际 Qt 库／插件与 SDK 的 SPDX 声明关联。产物是可复核证据，不是自动许可选择、合规批准或独立安全审查。正式发行完整性继续为 incomplete。

## 输入与来源

Qt 6.8 起提供构建对应的 [SPDX 清单](https://doc.qt.io/qt-6.8/licenses-used-in-qt.html)。工具只读取本次固定 Qt 6.8.2 SDK 的 sbom/ 中 qtbase、qtdeclarative、qtsvg 和 qttranslations 等运行时声明，排除 qttools 构建／测试工具。验证 SPDX 版本、模块名和 Qt 版本，保存原始文档 SHA-256、namespace 和稳定包引用。输出只保留必要字段，不复制包含开发机路径的上游构建注释。

`scripts/qt-license-sources.json` 固定官方 Qt 源码及 SPDX license-list-data 提交、许可文本 URL 和 SHA-256。下载内容放入 build/ 缓存；任何缓存或下载的哈希不符都会失败。不会跟随 SPDX 中任意地址下载材料。除 LICENSES/ 的标准文本，还包含 Qt GUI 平滑缩放算法与 SVG 的原始通知，以及系统依赖声明涉及的 LGPL-2.1 文本；上游 LicenseRef 原文和版权声明保留在声明 JSON 中。

## 文件与组件关联

1. 对核验根内文件重新计算哈希，拒绝在 deployment-audit.json 之后被修改的文件。
2. macOS SDK 模式使用实际解析的 Qt SDK 相对路径和字节身份，不据此声称独立部署。
3. Linux 部署阶段记录 SDK 原文件路径／SHA-256 与 patchelf 后工件路径／SHA-256。后续双向校验记录，拒绝过期或篡改的复制证据。
4. Windows 优先按 SDK 文件字节身份关联。文件名唯一但字节不同的候选仅标记 filename-only-unverified-rewrite，并登记缺口，不能当作来源验证通过。
5. 通过 SPDX 的 CONTAINS 找到文件所有者，再递归解析 DEPENDS_ON，包括跨 Qt 仓库引用和内嵌第三方。缺失包引用保留为缺口。不从仓库总包的 CONTAINS 将全部工具错误纳入运行时。
6. 非原生 QML／翻译若没有逐文件声明，只关联到对应源码仓库范围，不推断该文件的许可证。没有 SPDX 文件记录的 SDK 原生库、Qt 范围外原生文件、NOASSERTION 许可及缺失文本均单独登记。

SDK 中实际文件可能与上游构建 SPDX 中的 SHA-1 不同，例如后续安装处理或签名；工具保留预期值、对比结果并登记差异，不自行推断原因。SDK 字节关联、部署重写记录和上游原始哈希是三种不同证据。

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

Linux 传入部署重写记录；其他平台省略 --deployment。报告写在核验根外，避免自身清单递归。CI 在部署／最终哈希之前准备许可材料，Windows 和 Linux 将 notices 放入实际运行目录，Linux tar.gz 同时交付这些材料；macOS 只做 SDK 读取与本地／CI 记录，仍不部署 Qt、签名、压缩或上传应用。

runtime-attribution.json 包含文件、组件、关系、上游声明、版权、文本来源与缺口。artifact-files.cdx.json 增加相同组件与已知所有者关系，并绑定该报告和同一实际审计的哈希／构建身份。官方 CycloneDX schema 与引用检查继续执行，compositions 仍为 incomplete。

默认检查允许明确登记的开发审查缺口，禁止把它们静默标记通过；身份、路径、字节、版本、复制记录或许可文本哈希错误直接失败。--require-complete 是发行阻断检查，当前会因未完成归属或发行材料而失败。测试覆盖文件被改写、过期复制记录、路径逃逸、版本错误、跨文档缺失、部分 SBOM 及 Rust 工具排除。

## 尚未解决的发行门槛

已增加[源码交付收集器](source-delivery.md)，保存实际项目源码、Rust vendor 和四份固定 Qt 模块归档、重建及动态替换说明。它不能证明 SDK 与上游逐项构建等价。精确静态链接归属、操作系统／再分发运行库、所有内嵌资源逐项许可、开源许可分支选择及履约仍需完成。[本地 Mac 核验](macos-deployment.md)只证明元数据修改的兼容 QtCore 可替换，不冒充从源码重建 Qt。源清单与 SPDX 字段不能替代这些义务。真实安装、签名身份和独立安全审查按 [WP-12 准入](../qa/private-test-readiness.md)单独登记。
