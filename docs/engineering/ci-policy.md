# CI 与合入策略

已配置 Documentation、Foundation probes 和 Qt components 三组工作流，分别检查文档与资源、无 Qt 核心及诊断桥接、三平台 Qt 产品与开发验收包。具体执行证据见 [M1 验收入口](../qa/m1-acceptance.md)。下表定义按变更类型执行的门槛，未实现的安全 I/O、格式写入及正式发布检查在对应阶段接入。

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

Qt components 通过 scripts/ci-desktop.ps1 在三平台安装固定 Qt 6.8.2，构建真实浏览应用、执行 qmllint 与 6 组 CTest，将并发生命周期套件重复 10 次。macOS/Windows 部署 Qt 后清除开发搜索路径，用原生 cocoa/windows 平台插件执行启动/关闭冒烟检查。上传同提交源代码、许可通知、截图、日志和 build.json；开发包保存 14 天，不作为正式签名发行。离屏截图不证明 GPU、输入法或辅助功能的人工验收已完成。

M2 三平台 Qt 构建会生成十万条平面／分层 ZIP64 样本，执行实际 Qt 查询与源变化回归，并上传核心乱序元数据探针和 Qt 测量。工件命名为 xdzip-m2-平台-架构；它们仍是未公证的开发验收包。

检查直接 main 推送时使用 `/actions/runs?head_sha=...`，核对 push 事件；连接器 `fetch_commit_workflow_runs` 仅筛选 PR 事件，空结果不表示没有自动运行。
