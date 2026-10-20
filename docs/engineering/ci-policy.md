# CI 与合入策略

已配置 Documentation、Foundation probes 和 Qt components 三组产品工作流和 HTML design review，分别检查文档与资源、无 Qt 核心及诊断桥接、三平台 Qt 产品。检查方法见[验证入口](../qa/README.md)，具体结果绑定提交并记录到 issue 与 Actions 工件。下表定义按变更类型执行的门槛，M3 已加入安全 I/O 和真实 ZIP 解压回归，WP-10 已加入真实 ZIP 创建互操作，正式发行还须按[发行审查流程](../qa/release-review.md)单独确认。

| 变更 | 必需检查 |
| --- | --- |
| 文档/视觉 | 本地链接、JSON、HTML 生成、图片存在；核对设计与范围 |
| Rust 核心 | fmt、Clippy、领域/性质测试、受影响集成测试 |
| Qt/C++/QML | format、选定 tidy、qmllint、构建和焦点/模型行为测试 |
| FFI/安全 I/O | 三平台原生测试、所有权/竞态/故障注入，额外审查 |
| 格式 | 双向互操作、坏输入、资源/取消、能力清单 |
| 发布 | 三平台安装验收、未正式签名包与必要 ad-hoc 完整性、SBOM、许可材料、基准和回退 |

普通 PR 默认最小权限、无签名凭据，外部贡献不运行带秘密的代码。Actions 依赖固定完整提交 SHA；缓存以工具链、依赖锁和架构分键，不从不可信 PR 恢复发布工件。fuzz 与性能长任务独立调度，性能判断使用稳定参考机而非共享 runner 偶然成绩。

分支保护必须等真实检查存在且成功运行后指定检查名称；CODEOWNERS 等实际审查者加入后建立。仓库当前可见性和保护设置不由文档生成过程默默修改。

Foundation probes 在 Linux x86_64、Windows MSVC x64、macOS arm64 执行 Cargo workspace 测试、Clippy、桥接探针；另检查 Rust/C++ 格式。核心测试不依赖 Qt。

Qt components 通过 scripts/ci-desktop.ps1 在三平台安装固定 Qt 6.8.2，构建真实浏览应用、执行 qmllint 与 6 组 CTest，将并发生命周期套件重复 10 次。macOS 使用已安装的 Qt SDK 和原生 cocoa 平台插件执行启动/关闭冒烟检查，不执行 macdeployqt、签名或应用归档；此检查证明原生开发构建可启动，不代表独立发行包验证。Windows 部署 Qt 后清除开发搜索路径，用 windows 插件验证部署结果。离屏截图不证明 GPU、输入法或辅助功能的人工验收已完成。

macOS CI 不自动生成部署包或上传 Apple Silicon 开发应用包。macOS 仅上传 xdzip-checks-macOS-ARM64 日志与 JSON 测量记录；Windows/Linux 仍上传实验开发构建、同提交源码与许可通知。记录或工件保存 14 天，均不作为正式签名发行。原生 CI 中间产物统一位于 build/desktop 和 build/review。

所有层级的 dist/ 及其全部内容都在 .gitignore 中排除，build/、target/ 也不进入 Git 源码树。Documentation 的 scripts/check-docs.py 检查 Git 索引，即使强制暂存 dist/ 文件也会失败。HTML 离线审阅工具仍可在本地生成 dist/，但生成结果只能作为本地文件或 Actions 工件，不能提交到仓库。

三平台 Qt 构建会生成十万条平面／分层 ZIP64 样本，执行实际 Qt 查询与源变化回归，并上传核心乱序元数据探针和 Qt 测量。Windows/Linux 开发工件按平台与架构区分，实际名称和上传范围以 [Qt 工作流](../../.github/workflows/qt-components.yml)为准。源码树不再保留历史截图、量测记录或开发包回执；历史整理必须遵循[内容与历史管理](repository-hygiene.md)，经所有者明确授权并备份后才可执行。

检查直接 main 推送时使用 `/actions/runs?head_sha=...`，核对 push 事件；连接器 `fetch_commit_workflow_runs` 仅筛选 PR 事件，空结果不表示没有自动运行。

M3 额外执行真实解压、冲突、取消等待、关闭源会话后继续任务和退出决定的 Qt 流程；测量 64 个 Deflate 文件／32 MiB 实际输出并独立复核全部字节。核心覆盖 CRC、流终止、描述符、源／目标变化、链接、ACL、提交竞争、清理及磁盘写失败。磁盘满为可重复的部分写入故障注入，不冒充真实填满设备。Windows 权限回归直接修改新建自有夹具的 ACL，只读、枚举与执行权限允许，写入、删除、所有权与 ACL 修改继续拒绝，覆盖有效与仅继承授权。Windows 另检查产品 PE 子系统为 GUI，诊断程序保持 console，并验证内建窗口按钮的最大化、还原、最小化与异步关闭。

WP-10 增加三平台真实创建窗口→生成 ZIP→重新打开→重名取消的 Qt 流程，DPR 1／1.25／2 检查逻辑尺寸、字号、字段和 footer。独立 Python zipfile 校验 Store、Deflate 1／6／9、Unicode、空项与 66,001 条记录的 ZIP64；记录实际时间与可获得的 RSS，缺失值为 null。高 DPR 离屏屏幕变小可能触发窗口高度上限，报告记录可用屏幕高度并核验约束后的逻辑尺寸，不将裁切误记为正常全窗口体验。

测试截图、量测 JSON、运行日志和生成报告均不得提交到 Git。设计源图、透明运行素材、设计参数与字体材料清单保留；它们不是实际运行的验收证据。Documentation 对 Git 索引执行检查，禁止通过强制暂存恢复旧工件。所有生成器输出到 build/review、其他 build 子目录或 dist，当前工件只作为 Actions artifact 上传。

ZIP 高级创建增加真实 AES-256 与编号分卷输出，核心覆盖错误密码、边界预算、来源／目标变化、整组冲突、身份校验及晚取消。Linux Qt CI 使用独立 7-Zip 解压 8 种 Store／Deflate、单文件／分卷和明文／AES 组合，复核空文件、空目录、Unicode 和全部有效载荷。Qt 控件校验覆盖默认级别、自定义空值／零值、单位、密码非法值与清空。结果明细检查使用真实任务记录，截图与量测遵守本地／CI 工件政策。

密码解压增加三平台格式与任务回归：AE-1／AE-2、严格结构及认证 EOF、错误重试、不同条目密码、空密码、满队列控制、取消／退出、来源变化、旧目标与链接保护。Qt 通过真实 AES 创建、自动打开、密码请求、重试、解压与关闭链路，核对密码输入角色及辅助功能动作。HTML 与 Qt 分别检查中英文、浅深色、多字号、小屏与 DPR，生成记录不含输入密码；原生装饰与系统读屏人工结果继续独立登记。互操作按[密码解压契约](password-extraction.md)及实际 CI 工件记录。

WP-11 复用三平台核心和 Qt 门槛，增加 TAR/TAR.GZ 真实创建、浏览及解压，以及 scripts/check-tar-interop.py 的 Python tarfile 与可用系统 tar 双向字节核对。GNU/PAX 长名称、空目录、多 member GZIP、逃逸及特殊项、CRC／截断、配额和取消均由实际测试登记。桌面人工验收和独立安全审查不随自动化结果标记通过。

WP-12 对实际工件递归核验原生导入，Windows/Linux 使用 deployed 模式，macOS 使用 sdk 模式并保留外部 SDK 限制。源码与实际文件分别生成 CycloneDX 1.6，绑定实际 CLI 身份并校验锁定官方 schema；不把文件清单称为完整运行时组件合规证明。所有平台生成八项全 pending 的人工表作为新候选模板，不覆盖 issue 中既有验收结论，具体边界见[部署核验](deployment-audit.md)。macOS 新增上传仍仅限 JSON，禁止应用包自动部署、签名或上传。

Linux 增加[独立部署与运行容器](linux-deployment.md)：固定 SDK 的 QML 扫描、Qt 库递归收集和相对 RPATH，清除开发搜索路径后核验。独立 Ubuntu 24.04 镜像只安装实际 OS 运行依赖与 Xvfb，确认没有系统 Qt／编译器；只读挂载带空格路径下的运行目录，关闭网络，检查版本、X11 启动及十万条目打开／取消／退出。该测试使用软件渲染，不冒充真实 GPU 或完整桌面安装。Linux 下载使用 tar.gz 保留执行权限，原始 linux/ 目录不再重复上传。

安装器 gate：Windows 固定官方 Inno Setup 编译器／中文资源哈希并验证当前用户安装、原生 ShellExecuteEx 打开、升级／回退、忙时阻止和自有项卸载；Ubuntu 在 SDK-free 容器用非 root 产品验证 dpkg／GIO 同类生命周期。QA 包只生成到 build/installer-tests/，不上传为产品；正式实验安装器及 JSON／日志随已有 Windows/Linux 工件保存。Mac 只补充文档类型与文件事件并原生编译／启动，不生成安装包。

本地部署与 CI 分开：用户明确要求时可制作开发 DMG，允许本地 Qt 部署与必需的 ad-hoc 完整性签名，见[Mac 部署](macos-deployment.md)。此例外不改变上述 CI 禁止生成／上传 Mac 应用的范围，不授权 Developer ID、公证或发布。三平台 CI 另编译固定历史／当前设置适配器并验证同 schema 往返和较新 schema 拒写；Linux 安装测试增加一次真实 postinst 配置失败、运行阻止与 dpkg 恢复。

Mac 包体积规则在 Documentation 的跨平台夹具测试 `scripts/test-macos-bundle.py` 中检查：覆盖嵌套 Mach-O 架构识别及 `arm64` 裁剪、Basic 样式与共享依赖保留、未知或缺失依赖拒绝，以及输入应用／SDK 不变。CI 不为这些测试调用 macdeployqt、lipo 或 codesign，不因此引入 Mac 包部署。真实开发 DMG 在本地部署后递归复验全部原生分片和样式集合，记录优化前后字节数，并完成重定位 Cocoa／Metal 与兼容 QtCore 修改副本启动核验；实际执行结果独立登记，不以夹具测试代替。

独立源码验证使用[Qt 源码专用工作流](../../.github/workflows/qt-source.yml)，由可信 main 显式分派，单独运行三平台所需模块的源码编译与真实产品测试。Mac 仍只上传有界日志和 JSON，不缓存或上传 SDK／应用；Windows/Linux 依赖缓存以 runner 镜像、源码配置和脚本指纹区分，源码恢复后重新校验。普通 Qt CI 同时校验原生指标协议、实际资源与链接证据；不在共享 runner 用单次性能数字批准发行预算。

源码库内容边界由 `scripts/repository_contents.py` 对 Git 索引核验，禁止强制暂存生成目录、文档阅读版、已退役阶段报告、QA 执行输出及超过审查阈值的大 blob。具体执行状态使用 issue，不把 CI 或验收报告重新提交到 docs。
