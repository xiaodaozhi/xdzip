# xdzip M1 验收入口

范围：路线图的 **M1 产品骨架**，不是 Apple M1 芯片的产品功能版本号，也不是 M3 完整解压。保留已有 ZIP 浏览、查询、本页选择和元数据预检能力。

## 交付内容

- 正式 Rust 命令/事件运行时、独立 cxx 双句柄与 Qt 接收线程。
- 有界请求、控制通道和 Qt mailbox，空闲事件等待、取消、异步关闭与故障处理。
- 无 Qt 核心测试、原生三平台产品构建及关闭竞态测试。
- Apple Silicon 开发 app、Windows 开发运行目录、Linux 同版 Qt 验收二进制，以及同提交源代码、依赖通知、截图和测试日志。

实现契约见 [M1 运行时](../engineering/m1-runtime.md)。M1 开发与自动化验证已完成，Apple Silicon 桌面人工体验待一次性验收；未将人工检查标记为已通过。

视觉状态：后续[严格复核](visual-audit-review.md)确认布局、尺寸、字号缩放、主题和基线仍有差异，整套高保真一致性未通过。运行时/生命周期完成与视觉验收是两个独立结论。

## 通过证据

验证日期：2026-10-11。产品代码提交 `1bb24392129159fefbc9cde941c69f8b2180da4c`，后续收尾整理文档并加严 macOS 部署检查。下载构建的精确提交以包内 build.json 为准；以最新通过的 Qt components 产物进行人工验收。

- [Qt components：三平台产品、测试与打包全部通过](https://github.com/xiaodaozhi/xdzip/actions/runs/38162404763)。每平台 6 组 CTest、生命周期套件再重复 10 次；macOS arm64 与 Windows x64 部署 Qt 后使用原生平台插件启动并完成异步退出。
- [Foundation probes：三平台 Rust/C++ 及格式检查全部通过](https://github.com/xiaodaozhi/xdzip/actions/runs/38162404787)。每平台 29 项 Rust 单元测试、1 项 compile-fail 文档测试，以及独立桥接检查；Clippy 无警告。
- [Documentation：链接、资源、token、脚本与空白检查通过](https://github.com/xiaodaozhi/xdzip/actions/runs/38162404840)。文档收尾提交继续运行相同工作流。
- 本地 Linux 同样通过全部检查；HTML 重新生成后验证 17 页、950 个本地引用、2 张内联图，无外部资源依赖。

首次 macOS 重复测试暴露毫秒心跳的时序误报，已改为在 drain 批次间插入 GUI 队列探针。重复测试还暴露短暂准入竞争，产品已加入有界异步重试；通过记录包含这些修复，而非跳过失败用例。

打包日志复核发现 Qt SDK 附带的非 SQLite 数据库驱动引用外部库，macdeployqt 报错却仍返回 0。收尾构建排除无关驱动、将部署 ERROR 视为失败，并增加 ad-hoc 签名严格验证；原生启动检查继续保留。

## 下载与运行

GitHub 仓库 Actions → 最新通过的 **Qt components** → Artifacts：

- `xdzip-m1-macOS-ARM64`：解开外层 artifact，再解开 `xdzip-macos-arm64.zip`，得到 xdzip.app。Qt 随包部署，Apple Silicon 原生运行；目标下限 macOS 14，最新版 macOS 的人工桌面体验由验收确认。
- `xdzip-m1-Windows-X64`：运行 windows/xdzip.exe；保留同目录 Qt 文件。
- `xdzip-m1-Linux-X64`：二进制依赖 Qt 6.8.2，尚非便携发行包。

macOS app 只有 ad-hoc 签名，没有 Developer ID 签名或公证。系统若阻止打开，可在核对仓库、提交和来源后使用 macOS“隐私与安全性”的针对该应用的允许打开入口；不需要关闭系统整体安全机制。正式签名发行属于后续发布门槛。

每个 artifact 的 build.json 指明确切提交与架构；xdzip-source.tar 是对应源码，notices/ 是依赖通知。CI artifact 保存 14 天，过期后可重新运行工作流或按开发指南重建。

## 一次性人工验收

1. 启动 app，确认窗口、中文、浅深主题和按钮显示正常。
2. 打开一个小型普通 ZIP：先显示列表，不自动解压；搜索可用，当前查询完整展示，不显示分页按钮。
3. 选择条目，打开“解压预检”：独立原生窗口有系统标题栏，关闭它保留主窗口与选择。
4. 读取时取消，随后重新打开归档；重复打开/关闭主窗口、使用 Cmd+Q，应用应能正常退出且不留后台任务。
5. 可运行包内可执行文件加 `--smoke-test`，自动启动并执行同一关闭链路；退出码 0 为通过。

视觉稿的完整主窗口、全部操作流程、实际压缩/解压、十万条索引、签名发行与性能承诺不在 M1 范围。手动结果用于界面与实际机器体验验收，不替代已经要求执行的自动化测试。

## 自动化验收项

| 项目 | 检查 |
| --- | --- |
| 双句柄所有权 | 句柄只能取一次；放弃接收关闭准入 |
| 背压 | Rust 128 个 outstanding、8 MiB 数据预算；Qt 256 个事件及控制保留槽 |
| 控制通道 | 普通队列满时取消/重复关闭仍可接收，ID 耗尽预留关闭 ID |
| 请求终态 | 每个已接受请求恰好一次 Finished，数据先于其终态 |
| 故障 | 注入 worker panic 后 RuntimeFault、pending 失败、服务废弃 |
| 空闲 | 没有循环 Qt 通知；条件变量等待及关闭唤醒 |
| Qt 生命周期 | 满 mailbox 分批 drain，GUI 队列可穿插处理，消费回调内销毁连接无悬空访问 |
| 关闭屏障 | ShutdownReady 之前释放源并 join Rust 工作线程；接收线程结束后退出 |
| 回归 | 真实浏览、预检、中文路径、窗口组件、浅深主题和字体放大 |

三平台 CI 对并发生命周期套件重复运行 10 次，正常检查不盲目重复。截图为离屏软件渲染，不代表原生标题栏/GPU/输入法/读屏验收完成。
