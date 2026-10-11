# 领域与 Qt 组件首个增量

日期：2026-10-11。基线：PR #2 已合并至 main（9e1cd91），后续通过本地检查后直接推进 main，不强制推送共享分支。

本页保留首个组件增量的历史证据；当前产品运行时、三平台 CI 和开发包状态以 [M1 验收入口](m1-acceptance.md) 为准。下文的未完成项描述该增量当时的范围。

## 交付与契约

`xdzip-domain` 无第三方依赖、无 I/O，建立非零 64 位身份类型和分页请求不变量。ID 不按路径/行号生成；保留超过 JavaScript 精确整数范围的值，后续 Qt 适配必须使用不透明字符串。分页限制 1–256 行，验证 offset+limit 不溢出；响应必须匹配 session/query/generation/revision、offset 和行数预算。这里只检查元数据，不表示查询引擎或分页缓存已经实现。

Qt 新增组件预览，直接使用高保真交付图标及视觉 token。任务窗是独立非模态窗口，关闭与取消信号分离。尚未连接真实 Rust runtime；停止状态和 60% 进度是明确标识的示例。

## 本地验证

Linux x86_64 / Qt 6.8.2 / software / offscreen：

- Rust 9 项单元测试和 1 项 compile-fail 文档测试：身份零值/大整数、类型混用拒绝、分页窄化与溢出、各维度旧响应，以及既有 6 项桥接生命周期测试。
- Rust fmt、Clippy `-D warnings`；手写 C++ clang-format 18.1.8；QML 静态检查。
- CTest：桥接、Qt 窗口探针、资源内预览加载、Qt 组件测试。
- 提示行在浅色、深色、200% 字号下的图标中心 X、宽度、文字起点 X 完全相同，文字高度不小于实际内容高度；图标状态必须 Ready。
- 关闭详情不发送取消，重开保留停止状态；未知总量不显示百分比；200% 字号下滚动后可点击取消。
- 三张实际渲染截图检查了浅色、深色和放大文字；截图只含内容区域，不含系统标题栏。

CI 延续三平台 Rust/C++ 检查，并新增 Qt 6.8.2 Linux 离屏构建、qmllint 与组件测试。Qt 工具下载使用 aqtinstall 3.3.0，仅用于开发环境；未完成 Qt 全量发行 SBOM。远端通过状态以 Actions 为准。

## 未完成

WP-02 与 WP-04 均为部分实现；WP-03 产品运行时尚未启动。尚无真实任务终态、事件 mailbox、归档读取、安全写入或安装包。macOS M1 最新系统、原生系统装饰、工作区边缘、多屏、焦点恢复、键盘/读屏完整验收仍未完成。不能把布局检查等同于整套视觉/无障碍验收。

入口：[组件说明](../../apps/desktop/README.md)、[工作清单](../planning/backlog.md)、[领域规范](../design/03-domain-data.md)、[平台窗口契约](../ux/platform-windows.md)。

## 远端检查证据

实现与 CI 修订提交 `d39f558` 已通过：

- [三平台 Rust/C++ 与 C++ 格式检查](https://github.com/xiaodaozhi/xdzip/actions/runs/38149806666)。
- [Qt 6.8.2 Linux 干净构建与组件检查](https://github.com/xiaodaozhi/xdzip/actions/runs/38149806768)。
- [文档与 token 同步检查](https://github.com/xiaodaozhi/xdzip/actions/runs/38149806708)。

首次 Qt CI 暴露缺少 OpenGL 开发包，已补上 OpenGL/EGL/xkbcommon 开发依赖并重跑通过；软件离屏模式不会免除 Qt Gui 的构建依赖。以上检查不替代 macOS/Windows 原生 Qt 验收。
