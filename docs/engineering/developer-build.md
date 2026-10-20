# 开发构建与平台验证

开发构建包含真实 ZIP、TAR、TAR.GZ 浏览、创建与解压，仍标记实验性。能力和限制见根目录 [README](../../README.md)，支持范围见[支持政策](support-policy.md)。WP-12 开发与验收完成不代表正式发行获批，剩余许可来源、独立安全审查与私密报告演练在[发行审查 #4](https://github.com/xiaodaozhi/xdzip/issues/4)跟踪。

## 从源码运行

使用根目录锁定的 Rust、Cargo.lock、Qt 与平台编译器。完整配置、构建、CTest 及离屏环境命令见[Qt 开发应用](../../apps/desktop/README.md)。Rust 核心可独立执行 `cargo test --workspace --locked`，不需要 Qt。

真实产品在所有平台统一使用 `xdzip` 名称，Windows 为 `xdzip.exe`，macOS 为 `xdzip.app` 内的 `xdzip`。组件示例入口为 `xdzip-ui-preview`，测试程序保留各自测试名称。用实际产品进行文件、任务及系统集成验收，不用示例进度替代真实操作。版本、渠道、提交与脏状态由构建身份及包内 build.json 记录。

macOS 默认以 `XDZIP_MACOS_HANDLER_RANK=None` 构建开发与测试包，避免普通 CMake/CTest 在系统“打开方式”中增加候选。仅为保留的主应用或 DMG 输入显式设置 `-DXDZIP_MACOS_HANDLER_RANK=Alternate`。参数仅接受这两个值，不更改用户默认关联；原生候选查询与历史副本维护见[Mac 部署指南](macos-deployment.md)。

## 平台交付

| 平台 | 开发工件与入口 | 验证边界 |
| --- | --- | --- |
| macOS arm64 | 原生构建生成 build/ 内 xdzip.app；按明确需求制作本地开发 DMG | CI 使用固定 SDK 编译、测试及 Cocoa 启动，只上传日志与 JSON，不部署、签名或上传应用；本地 DMG 允许必要 ad-hoc 完整性签名 |
| Windows x64 | 开发工件 windows/xdzip.exe 及相邻 DLL／插件；当前用户安装器 | 安装后入口固定为安装根目录 xdzip.exe，不新建 versions/提交号-哈希 层级；旧布局迁移规则见安装契约 |
| Ubuntu 24.04 x64 | xdzip-linux-x86_64.tar.gz 内 ./xdzip，或 .deb 安装后的 xdzip 命令 | Qt、QML 与插件随包提供；仍依赖声明的 OS 图形和字体库，不保证其他发行版或 Wayland |

部署流程见[Linux 部署](linux-deployment.md)、[macOS DMG](macos-deployment.md)及[安装与文件关联](installers-associations.md)。Windows/Linux 安装器未正式签名，注册打开候选而不修改用户默认应用。文件关联、权限或系统焦点的问题须使用真实安装环境验证。

## 文件与任务边界

打开只浏览，不自动解压。普通同名文件支持询问、跳过、改名及明确替换；内容校验和同步后才整文件发布。提交前取消或写入失败保留旧文件，早前已提交的解压项继续保留。关闭任务窗口仅隐藏详情，退出由活动任务决策和异步 shutdown 屏障处理。

ZIP 在受限 POSIX 条件下可恢复相对符号链接和可执行位，不恢复特权位、扩展属性或原始修改时间，不代表应用包安装或签名验证。来源与目标必须满足[安全解压契约](m3-extraction.md)；文件输入竞态和 macOS ACL 兼容修复的范围与验证记录见 [#6](https://github.com/xiaodaozhi/xdzip/issues/6)。崩溃暂存只提示并保留供检查，不自动恢复或删除未知内容。

## 证据与复验

按[通用验证方法](../qa/README.md)和[桌面清单](../qa/private-test-manual.md)选择受影响项。人工专项与离屏、Cocoa 或软件渲染自动检查分别记录；既有验收在 [#5](https://github.com/xiaodaozhi/xdzip/issues/5)保留，不因重新生成清单而重做。

Windows/Linux 工件交付同提交源码和许可通知，Actions 保存 14 天；过期后按身份和锁定输入重建。deployment-audit.json、源码及文件级 CycloneDX 清单用于来源核验，完整性仍可能为 incomplete，不能当作独立安全或许可批准。源码重建与 Qt 替换方法见[源码交付](source-delivery.md)及[自动验证契约](wp12-automation.md)。

所有构建、截图、量测、报告与包只写入忽略的 build/、target/ 或 dist/。不要把开发机 SDK 可运行、生成的 pending 表或部分 SBOM 写成干净环境与全部发行门槛通过。
