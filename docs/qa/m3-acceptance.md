# M3 安全 ZIP 解压验收

状态：2026-10-13 开发与三平台自动化验收完成。实验开发包已交付，最新 macOS 桌面体验、独立安全审查和正式签名发行单独登记，不由自动化检查替代。

验证和打包提交为 `a97d24843cbd263c84358b7e16db1ef84531fde2`。后续证据文档提交不改变应用行为；开发包 build.json 与该提交一致。

构建策略于 2026-10-14 更新：后续 macOS CI 仅编译、测试并上传日志／JSON，不再生成或上传 Apple Silicon 开发应用包。下文的包、签名与部署回执为上述提交的历史验证证据，保持原始记录。dist/ 从不作为 Git 源文件目录；当前规则见 [CI 策略](../engineering/ci-policy.md)。

## 实际能力

打开 ZIP／ZIP64 仍只浏览；显式解压支持 Store／Deflate、全部物理归档或冻结的所选文件／目录。浏览根目录仅显示直属项，选择目录解压包含后代；所选文件保留相对目录结构。支持目标检查、同名目录、普通文件的询问／跳过／自动改名／明确替换、真实字节和提交计数、关闭窗口后继续任务、状态栏重开与异步退出。

格式扩展只获得输入和宿主 sink，不获得目标路径。所有输出使用目录能力、nofollow 父链、私有阶段文件和校验后的整文件提交。路径穿越、设备名、链接、特殊对象、来源变动和不可信权限失败；坏 CRC、本地头、描述符、缺失 Deflate 终止块或尾随字节不能覆盖旧文件。单文件 8 GiB、单任务 20 GiB，显式高压缩比许可不提高硬限额。早前已提交文件继续保留，不虚构任务整体回滚。

详细语义和 Windows OWNER RIGHTS／元数据 ACL 边界见 [M3 契约](../engineering/m3-extraction.md)；精确依赖、许可与公告查询见 [依赖准入](../engineering/zip-extraction-admission.md)。这是开发者自检，未宣称独立安全审查。

## 三平台原生检查与开发包

[Foundation probes](https://github.com/xiaodaozhi/xdzip/actions/runs/38450239636)、[Qt components](https://github.com/xiaodaozhi/xdzip/actions/runs/38450239808)、[Documentation](https://github.com/xiaodaozhi/xdzip/actions/runs/38450239736) 全部成功。[CI 回执](m3/native-ci.json)登记提交、任务、工件大小、SHA-256 与到期时间。HTML 应用代码与素材的完整检查在 [HTML design review](https://github.com/xiaodaozhi/xdzip/actions/runs/38441838509)通过；后续核心修复没有改变 HTML 应用代码。

| 平台 | 核心／Qt／解压／部署 | 开发工件 |
| --- | --- | --- |
| macOS ARM64，macos-14 runner | 通过，Cocoa 启动与原生标题栏检查通过 | [Apple Silicon](https://github.com/xiaodaozhi/xdzip/actions/runs/38450239808/artifacts/11840198943) |
| Windows X64，windows-2022 runner | 通过，部署后的 windows 插件启动通过 | [Windows](https://github.com/xiaodaozhi/xdzip/actions/runs/38450239808/artifacts/11840540318) |
| Linux X64，Ubuntu 24.04 | 通过；动态链接开发二进制，需要 Qt 6.8.2 | [Linux](https://github.com/xiaodaozhi/xdzip/actions/runs/38450239808/artifacts/11839409762) |

原生检查包含完整 Cargo workspace／Clippy、桥接生命周期、6 组 CTest、Qt 套件连续 10 次、真实解压／冲突／取消／来源会话关闭／Quit 决策、十万条查询回归，及 DPR 1／1.25／2 各 729 项冻结参考布局检查。主要布局契约 75 项全部通过。Qt 模型发布与 TableView 创建分别等待真实状态，避免用固定睡眠替代渲染完成。

核心回归覆盖 Store／Deflate、32 与 64-bit 数据描述符、内容／头部／终止块损坏、源变化、目标父目录替换、目录链接、硬链接、排他提交竞争、过期覆盖版本、源保护、未知暂存对象保留、单次 prompt、取消唤醒和 commit 许可线性化。Windows 以原生 ACL 工具修改新建自有夹具，验证元数据与已受信所有者权限可用，内容读取、枚举、写入、删除及修改 DACL 保持拒绝。磁盘满采用可重复的部分写入故障注入，证明旧目标不变和已知阶段文件清理；没有宣称填满物理设备或断电测试。

Apple Silicon 工件中的 xdzip-macos-arm64.zip 约 **50.0 MiB**，解压得到 xdzip.app。已独立下载核对外层与应用 ZIP 的 SHA-256、build.json、ARM64 Mach-O、14.0 部署目标和 M3 使用说明，见 [包回执](m3/bundle-receipt.json)。CI 中 `codesign --verify --deep --strict` 与部署后 Cocoa 启动通过；采用 ad-hoc 签名，尚未 Developer ID 签名或公证。Actions 工件在 2026-10-27 到期，可按锁定工具链从该提交重建。

## 真实测量与视觉证据

[原生解压测量](m3/native-core-performance.json)使用 64 个 Deflate 文件、8 个目录、32 MiB 确定性内容。计时包含权限检查、实际解码、文件同步、发布和清理，完成后通过独立句柄逐字节复核全部文件。压缩输入哈希可能因平台 Python／zlib 版本不同而变化，确定性输出和复核标准保持一致。

| 原生 CI 平台 | 32 MiB 解压／同步／发布 | 核心探针峰值 RSS |
| --- | --- | --- |
| macOS ARM64 | 166.9 ms | 3.3 MiB |
| Windows X64 | 829.0 ms | 未测量 |
| Linux X64 | 77.9 ms | 3.3 MiB |

这些是共享 runner 的单次结果，不是稳定参考机性能承诺。核心峰值不能解释成 Qt 应用总内存。[Qt 十万条回归测量](m3/native-qt-performance.json)中，Linux／Windows／macOS 完整加载分别 632／903／1501 ms，排序分别 440／779／992 ms，均为 55 个可见单元格委托；Linux／macOS 场景峰值约 229／224 MiB，Windows 未测量。该峰值包括查询、选择和后续操作，不是空闲内存或真实解压场景峰值。

本地 release 核心见 [JSON](m3/local-core-performance.json)，本地完整 Qt 日志见 [记录](m3/local-qt-tests.log)。测试归档均来自[自有生成器](../../scripts/generate-extract-fixtures.py)，没有用户文件或第三方归档。

[Qt／HTML／原稿对照 HTML](m3/index.html)分别展示未经修饰的真实任务截图、HTML 100% 逻辑尺寸捕获和保留的高保真原稿。实际 Qt 为 Linux offscreen/software，HTML 为示例数据；系统装饰、屏幕高度限制和数据差异分别标注。[操作窗口 51 项检查](m3/operation-metrics.json)核对 614／846／612 窗口宽度、footer 字号 15／13、按钮高度、提示左边缘、DPR 和底部可达性，不冒充完整像素一致性认证。捕获出处见 [Qt 回执](m3/qt/receipt.json)、[HTML 回执](m3/html/receipt.json)和 [HTML 实测](m3/html-operation-metrics.json)。

单文件离线审阅可运行 `python3 scripts/build-m3-review.py` 生成 dist/xdzip-m3-review.html；图片保留原始字节，窄屏无横向溢出，证据链接指向仓库。可操作设计稿仍由 `scripts/build-html-design.py` 生成。

## 一次性桌面验收与范围

在最新 macOS／Apple Silicon 上打开开发应用，用自有小归档检查根目录／子目录、搜索与排序；解压所选／全部；分别测试询问、跳过、改名、替换；用生成的损坏样本确认旧文件不变；取消冲突等待；隐藏并重开任务窗；检查 Cmd+Q／窗口关闭；核对红绿灯、原生独立窗口、浅深色、Retina、200% 字号、输入法与辅助功能。此项尚未执行，不用 macos-14 CI 代替最新版桌面体验。

当前不支持创建归档、TAR/GZIP、加密／分卷、其他编码、内容预览、链接或第三方进程扩展。不还原执行权限、扩展属性或原始修改时间，因此不是程序包完整安装能力。崩溃暂存目录只保留并提示，不自动恢复或删除未知对象。发行级恢复、安全审查、签名公证、SBOM、参考机与完整支持政策仍未完成。

下一工作包为 WP-10 创建 ZIP，先冻结输入树、来源变化和整体提交契约，再接已有创建窗口与互操作；不因解压验收完成就提前声明创建能力。
