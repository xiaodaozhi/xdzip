# CI 与合入策略

已配置 Documentation、Foundation probes 和 Qt components 三组产品工作流和 HTML design review，分别检查文档与资源、无 Qt 核心及诊断桥接、三平台 Qt 产品。最新解压实现证据见 [M3 验收入口](../qa/m3-acceptance.md)。下表定义按变更类型执行的门槛，M3 已加入安全 I/O 和真实 ZIP 解压回归，WP-10 已加入真实 ZIP 创建互操作，正式发布检查仍按阶段接入。

| 变更 | 必需检查 |
| --- | --- |
| 文档/视觉 | 本地链接、JSON、HTML 生成、图片存在；核对设计与范围 |
| Rust 核心 | fmt、Clippy、领域/性质测试、受影响集成测试 |
| Qt/C++/QML | format、选定 tidy、qmllint、构建和焦点/模型行为测试 |
| FFI/安全 I/O | 三平台原生测试、所有权/竞态/故障注入，额外审查 |
| 格式 | 双向互操作、坏输入、资源/取消、能力清单 |
| 发布 | 三平台干净安装、签名、SBOM、许可材料、基准和回退 |

普通 PR 默认最小权限、无签名凭据，外部贡献不运行带秘密的代码。Actions 依赖固定完整提交 SHA；缓存以工具链、依赖锁和架构分键，不从不可信 PR 恢复发布工件。fuzz 与性能长任务独立调度，性能判断使用稳定参考机而非共享 runner 偶然成绩。

分支保护必须等真实检查存在且成功运行后指定检查名称；CODEOWNERS 等实际审查者加入后建立。仓库当前可见性和保护设置不由文档生成过程默默修改。

Foundation probes 在 Linux x86_64、Windows MSVC x64、macOS arm64 执行 Cargo workspace 测试、Clippy、桥接探针；另检查 Rust/C++ 格式。核心测试不依赖 Qt。

Qt components 通过 scripts/ci-desktop.ps1 在三平台安装固定 Qt 6.8.2，构建真实浏览应用、执行 qmllint 与 6 组 CTest，将并发生命周期套件重复 10 次。macOS 使用已安装的 Qt SDK 和原生 cocoa 平台插件执行启动/关闭冒烟检查，不执行 macdeployqt、签名或应用归档；此检查证明原生开发构建可启动，不代表独立发行包验证。Windows 部署 Qt 后清除开发搜索路径，用 windows 插件验证部署结果。离屏截图不证明 GPU、输入法或辅助功能的人工验收已完成。

自 2026-10-14 起不再自动生成或上传 Apple Silicon 开发应用包。macOS 仅上传 xdzip-checks-macOS-ARM64 日志与 JSON 测量记录；Windows/Linux 仍上传实验开发构建、同提交源码与许可通知。记录或工件保存 14 天，均不作为正式签名发行。原生 CI 中间产物统一位于 build/desktop 和 build/review。

所有层级的 dist/ 及其全部内容都在 .gitignore 中排除，build/、target/ 也不进入 Git 源码树。Documentation 的 scripts/check-docs.py 检查 Git 索引，即使强制暂存 dist/ 文件也会失败。HTML 离线审阅工具仍可在本地生成 dist/，但生成结果只能作为本地文件或 Actions 工件，不能提交到仓库。

M2 三平台 Qt 构建会生成十万条平面／分层 ZIP64 样本，执行实际 Qt 查询与源变化回归，并上传核心乱序元数据探针和 Qt 测量。Windows/Linux 开发工件命名为 xdzip-m4-平台-架构。M1/M2/M3 已登记的 macOS 开发包回执保留为历史证据，不代表后续构建继续打包。

检查直接 main 推送时使用 `/actions/runs?head_sha=...`，核对 push 事件；连接器 `fetch_commit_workflow_runs` 仅筛选 PR 事件，空结果不表示没有自动运行。

M3 额外执行真实解压、冲突、取消等待、关闭源会话后继续任务和退出决定的 Qt 流程；测量 64 个 Deflate 文件／32 MiB 实际输出并独立复核全部字节。核心覆盖 CRC、流终止、描述符、源／目标变化、链接、ACL、提交竞争、清理及磁盘写失败。磁盘满为可重复的部分写入故障注入，不冒充真实填满设备。Windows 权限回归直接修改新建自有夹具的 ACL，目录枚举、内容读取、写入、删除与 ACL 修改均保持拒绝。

WP-10 增加三平台真实创建窗口→生成 ZIP→重新打开→重名取消的 Qt 流程，DPR 1／1.25／2 检查逻辑尺寸、字号、字段和 footer。独立 Python zipfile 校验 Store、Deflate 1／6／9、Unicode、空项与 66,001 条记录的 ZIP64；记录实际时间与可获得的 RSS，缺失值为 null。高 DPR 离屏屏幕变小可能触发窗口高度上限，报告记录可用屏幕高度并核验约束后的逻辑尺寸，不将裁切误记为正常全窗口体验。
