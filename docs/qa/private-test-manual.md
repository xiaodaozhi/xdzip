# WP-12 人工专项操作清单

本清单补齐此前未覆盖的原生桌面与发行专项。已确认通过的 macOS 操作清单继续有效，不要求重复验收。自动化已验证的 Qt 接口、合成输入及离屏截图不能替代以下真实设备记录。

## 开始前

1. 固定同一提交和构建，从“设置 → 关于 xdzip”复制版本信息，记录实际被测可执行文件 SHA-256。
2. 使用自己的可删除测试目录和可公开测试数据，不上传用户压缩包、密码或完整私人路径。
3. 记录系统版本、设备、架构、原生渲染后端／驱动、系统缩放、输入法和读屏工具。macOS 用 VoiceOver，Windows 用 Narrator，Linux 用实际桌面的 Orca 或明确记录支持限制。
4. 从 [机器可读步骤](../../scripts/private-test-matrix.json) 准备本平台记录，示例：

```sh
python3 scripts/prepare-private-test.py --output build/review/private-test/manual-macos.json
```

在对应平台运行时默认填入本机系统和架构；提前为其他平台生成表格时使用 `--platform windows` 或 `--platform linux`，不会用当前 Mac 冒充目标环境。程序拒绝覆盖已有表格。生成表格时所有项目均为 pending，不表示执行通过。

## 执行顺序

| 编号 | 需要实际操作的项目 | 重点结果 |
| --- | --- | --- |
| IME-01 | 在搜索、创建名称、解压路径输入系统拼音候选，切换中英，测试组合中 Escape/Enter 和长 Unicode 名称 | 不提前提交、不误启动任务，文本完整 |
| A11Y-01 | 开启真实读屏，仅用键盘操作欢迎页、列表、设置、操作窗口和任务 | Tab 可达，名称、禁用／选中状态及任务变化能被理解，无焦点陷阱 |
| GPU-01 | 真实 GPU 下切换浅深色、100/125/150% 字号、系统缩放和显示器，展开菜单及弹窗 | 比例正确，无黑块、闪烁、裁剪和不可达按钮 |
| OS-01 | 中文系统打开／取消各种文件选择器，从 Finder/Explorer 拖入自有压缩包 | 遵循系统语言，取消不改变目标，不错误切换窗口前后顺序 |
| SET-01 | 保存语言并稍后／立即重启，在自有小任务运行中分别取消退出、等待和取消任务后退出 | 设置保留，取消退出不重启，安全关闭后重启并应用语言 |
| INSTALL-01 | 无开发 SDK 的干净系统安装、首开、真实压缩解压、退出、升级、回退及卸载 | 依赖齐全、内容正确，取消保护旧文件，用户数据保留 |
| ASSOC-01 | 检查双击、打开方式、图标、注册和卸载关联 | 当前尚未注册系统关联，本项未完成；不能因图标存在就填写通过 |
| PERF-01 | 固定参考机 Release 构建，按规范至少 30 次启动／交互，并测稳定空闲 CPU、物理内存和真实 I/O | 条件、分布及误差齐全，符合[性能规范](../design/09-performance.md) |

macOS 本地 SDK 构建可先做 IME、读屏及 GPU 等专项，但不能据此填写干净安装通过。现有约定禁止自动部署、签名或上传 macOS 应用包，该步骤仍需发布环境安排。没有其他平台设备、安装渠道、文件关联实现或签名身份时，如实填写 blocked 及具体原因。

## 记录与核对

每项填 `status`、`actual`、`evidence` 和可选问题引用。状态为 pending、passed、failed、blocked 或 not_applicable。不得仅填“通过”：需要实际操作结果、执行日期、操作者角色、设备和构建身份，以及 build/ 或 dist/ 中的相对证据文件路径。IME／读屏／GPU 通过还须填写对应工具或后端信息。

```sh
python3 scripts/prepare-private-test.py --validate build/review/private-test/manual-macos.json
python3 scripts/prepare-private-test.py --validate build/review/private-test/manual-macos.json --require-pass
```

第一条仅校验记录结构，不把 pending 当成通过；第二条要求八项均通过。缩减范围必须填写发布负责人决策引用，并单独进行准入评审，不能靠改成 not_applicable 绕过检查。实际记录和截图不提交 Git，使用本地／CI 工件保存。

三平台表格、部署依赖与 SBOM、性能记录、所需独立安全审查和发行配置汇总到 [WP-12 准入](private-test-readiness.md)。记录完整也不自动代表签名、许可合规或独立审查已通过。
