# 产品版本与构建身份

可选 `prerelease_number` 分离预发布序号与构建号。例如 alpha.1 对应 build 14，后续 alpha.2 仍继续递增 build。缺省保留历史 `渠道.build` 读取方式。发布清单和资产命名见[发布协议](release-protocol.md)。

[VERSION.json](../../VERSION.json) 是产品版本、渠道与构建号的唯一维护入口。当前值由构建系统读取，不在本文另行维护。版本变更见[变更日志](../../CHANGELOG.md)，各批次交付与验证见[项目 issue](https://github.com/xiaodaozhi/xdzip/issues)；版本递增本身不代表发行获批。

历史兼容检查使用固定源码基线，与当前构建共同验证安装升级、回退及设置兼容。完整产品基线由[历史产品准备脚本](../../scripts/prepare-previous-product.py)维护，设置基线由[兼容检查脚本](../../scripts/check-settings-compat.py)维护；不在本文复制随实现变化的提交或当前版本。设置 schema 与产品版本分别管理。

- `version` 使用无前导零的 `major.minor.patch`。不兼容变更、兼容功能与修复分别按版本范围管理，0.x 阶段仍须说明兼容边界。
- `channel` 支持 development、alpha、beta、rc、stable。非 stable 显示为 `version-渠道缩写.build`，development 缩写为 dev；stable 显示基础版本。
- `build` 为 1～65535 的整数。准备新的对外交付构建时递增，不按本机每次编译自动递增，也不自动创建标签或发布。数字版本各段同样不超过 65535，以兼容 Windows 资源。
- CMake 直接读取 JSON。Cargo workspace 与 HTML 的派生版本由 `python3 scripts/sync-version.py` 更新，`--check` 用于 CI；所有内部 Rust 包继承 workspace 版本。同步后更新 Cargo.lock，并检查 `cargo check --workspace --locked`。
- Windows 四段数值资源为基础版本加构建号，文字版本保留渠道；macOS 的 `CFBundleShortVersionString` 为基础版本，`CFBundleVersion` 为构建号，长版本保留渠道。格式遵循 [Apple bundle 版本说明](https://developer.apple.com/documentation/bundleresources/information-property-list/cfbundleversion)；它们使用各平台允许的数值格式，不把 Git 提交塞入系统数值字段。
- 每次桌面构建生成 Git 短提交和工作区 dirty 标志，没有 Git 时显示 unknown。没有机器路径、用户名或不稳定时间戳；头文件仅在内容变化时更新，提交之后再次构建即可得到准确的干净提交身份。

QML 模块、桥接协议和设置 schema 的版本独立于产品版本，不随产品版本自动变更。

`xdzip --version` 与“设置 → 关于 xdzip”读取同一构建信息，预览应用也一致。CLI 在创建 GUI 之前处理版本参数。“关于”窗口显示版本、构建号、提交、实际 Qt、系统和架构，开发渠道与未提交修改明确标记。复制结果为 JSON，不包含个人文件、路径、机器名或凭据。项目主页按钮由用户点击后打开浏览器。

关于的大图标使用对应平台的 512px PNG，HTML 使用同源矢量，不放大 72px 的界面小图。

关于窗口的逻辑参数来自 [about-reference.json](../visual/about-reference.json)，Qt 复用操作窗口及 footer，支持简体中文、English (US)、浅深色与 100%／125%／150% 字号。当前设计见 [A01 高保真](../../design/hifi/about.html)，HTML 版本信息由同一 JSON 生成，原生 Qt／提交未知时不伪造数值。

验证入口：`scripts/check-build-version.py` 对照实际 CLI 与原生版本资源，`scripts/test-version.py`、`scripts/check-html-about.py`、`BrowserTest::versionAboutWindow`。截图与测量仅保存在 `build/review/about/`。源码、系统元数据与实际运行版本一致是检查条件；版本号本身不证明签名、私测准入、兼容性或独立安全审查已经通过。
