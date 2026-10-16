# 产品版本与构建身份

[VERSION.json](../../VERSION.json) 是产品版本、渠道与构建号的唯一维护入口。首个接入基线为 `0.1.0-dev.1`，是开发构建，不表示 WP-12 已达到私测发行门槛。

本地 DMG 与源码交付批次递增为 `0.1.0-dev.2`，完整产品兼容检查使用固定历史 dev.1 和当前 dev.2；开发版本递增不改变设置 schema 1，也不表示正式发行准入完成。

外部文件启动欢迎页闪烁修复递增为 `0.1.0-dev.3`，沿用设置 schema 1，安装兼容检查继续与固定历史 dev.1 比较。

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
