# 部署依赖核验与 SBOM

WP-12 使用两类清单，均为开发审阅材料，完整性明确标记 `incomplete`，不自动提升为可发行结论。格式使用 [CycloneDX 1.6 官方 schema](https://github.com/CycloneDX/specification/blob/55343ba19dee1785acf1ce9191540d5fd7b590db/schema/bom-1.6.schema.json)。schema 的上游提交与三个文件 SHA-256 固定在 [配置](../../scripts/sbom-schemas.json)，验证时只下载并解析这些资源，缓存到 build/，不会从 SBOM 中的任意 URL 下载 schema。

| 工件 | 数据来源与用途 | 不能证明的事项 |
| --- | --- | --- |
| source-build.cdx.json | Cargo 宿主解析的工作区依赖图，含开发／构建工具；声明的 Qt 模块；字体和品牌源文件哈希 | 某个 Rust 包一定静态链接进产品、所有 Qt 插件／内嵌第三方均已识别 |
| deployment-audit.json | 指定实际工件根的文件 SHA-256、原生二进制导入、SDK／系统／未解析依赖以及实际 CLI 构建身份 | 所有按名称动态加载的 QML／插件、干净安装、系统 ABI、签名或许可证合规 |
| artifact-files.cdx.json | 实际工件及观察到的外部 SDK 文件，按原生导入建立已知边；绑定源码清单和实际可执行文件版本／提交／dirty | 完整部署组件身份、静态链接归属、Qt 内嵌库许可、正式产品 SBOM 已完成 |
| manual-desktop.json | 平台和提交已填入、八项均为待验证的人工记录表 | 已执行任何人工操作或通过准入 |

源码不把 Cargo 的路径型 opaque ID 写入记录，工作区包不伪造 crates.io purl。工件路径相对于指定根或 Qt SDK 记录，其他外部依赖以内容哈希和文件名标识，不输出私人 SDK 绝对路径。字体使用 OFL-1.1，原创品牌源沿用素材库的 Apache-2.0；声明 Qt 模块不据此推断所有实际部署组件的许可证。

## 自动核验

`deployment_audit.py` 只检查明确指定的自有可信构建，不接收用户压缩包。macOS 使用 otool 解析导入和 RPATH；Windows 使用 MSVC dumpbin 解析普通及延迟 DLL 导入；Linux 使用 ldd，禁止用于来源不明的可执行文件。文件符号链接若逃逸指定工件根，核验失败；报告必须写到根外，避免把输出包含进自身清单。

- `--mode sdk` 允许解析到外部 Qt SDK，并明确显示不独立；未解析导入仍失败。递归范围为工件及声明的 Qt SDK，其他外部库仅记录哈希与未递归计数，不视为闭包已核验。
- `--mode deployed` 不允许非系统外部依赖或未解析导入。Windows 同时继续执行清除 Qt 开发搜索路径后的原生启动冒烟。
- 未解析导入、源码与实际二进制版本／提交／dirty 不符、官方 schema 哈希或格式不符、重复／悬空组件引用均阻断相关检查。
- 原生导入闭合不能替代按名称加载插件的覆盖和干净机器检查。Windows 已部署目录的导入通过也不等于安装器验收。

三平台 CI 保存根目录下的两个 cdx.json、deployment-audit.json 和待填写人工表。macOS 只读取本地编译 .app 与 SDK，仍不运行 macdeployqt、签名、压缩或上传应用包。Linux 通过[独立运行目录](linux-deployment.md)提供 Qt／QML／插件，使用 deployed 核验及无 SDK 容器验证；下载 tar.gz 保留执行位。当前只声明 Ubuntu 24.04、x86_64、X11 基线，不称为通用 Linux 安装器。

本地 macOS 示例，先从一致源码重新构建，然后运行：

```sh
python3 scripts/collect-notices.py build/review/wp12/notices
python3 scripts/deployment_audit.py \
  --root build/ui/apps/desktop/xdzip.app \
  --app build/ui/apps/desktop/xdzip.app/Contents/MacOS/xdzip \
  --qt-prefix .cache/Qt/6.8.2/macos --mode sdk \
  --output build/review/wp12/deployment-audit.json
python3 scripts/sbom.py \
  --audit build/review/wp12/deployment-audit.json \
  --source build/review/wp12/notices/source-build.cdx.json \
  --output build/review/wp12/artifact-files.cdx.json
```

在隔离验证环境安装 jsonschema==4.23.0 后，使用该环境的 Python 运行 `scripts/validate-sbom.py`，传入上述两个 cdx.json。参考的注册表用法见 [jsonschema 4.23 文档](https://python-jsonschema.readthedocs.io/en/v4.23.0/referencing/)。验证环境的工具不部署到产品中。

## 下一层发行门槛

已接入[实际组件归属](runtime-attribution.md)：使用 SDK SPDX 关联实际文件、Qt 模块与内嵌第三方，交付固定哈希的官方许可文本，记录 SDK 与上游哈希差异及未确认归属。Rust 普通运行时候选与构建／开发／过程宏工具分开。新增组件关系绑定实际审计并合入工件 SBOM，仍明确标记不完整。

在批准的平台部署环境获得真实安装目录后，重新核验并逐一将 DLL／framework／插件与上游组件、版本、许可证和对应源码关联，补全 Qt 内嵌第三方、静态 Rust 链接归属、重链接材料和工具链来源。只有同时具备人工干净安装证据、发布身份及所需安全审查，才提交 WP-12 准入评审。当前脚本不会自动签发该结论。
