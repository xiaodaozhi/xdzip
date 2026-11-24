# GitHub Release 协议 v1

[English](../../engineering/release-protocol.md) | 简体中文 | [繁體中文](../../zh-TW/engineering/release-protocol.md) | [日本語](../../ja-JP/engineering/release-protocol.md) | [한국어](../../ko-KR/engineering/release-protocol.md) | [Français](../../fr-FR/engineering/release-protocol.md) | [Deutsch](../../de-DE/engineering/release-protocol.md) | [Español](../../es-ES/engineering/release-protocol.md) | [Português (Brasil)](../../pt-BR/engineering/release-protocol.md) | [Русский](../../ru-RU/engineering/release-protocol.md)

本流程负责 GitHub 下载包。[Microsoft Store MSIX 工作流](../../../.github/workflows/windows-store-msix.yml)可基于同一版本源码生成商店包，商店认证和发布在微软合作伙伴中心完成。

专用 [Release 工作流](../../../.github/workflows/release.yml)通过 `workflow_dispatch` 手动运行。正式发行从 `main` 输入已有版本标签并冻结其提交。四个原生作业分别配置、构建、测试和打包一次。macOS 首次构建就使用发行所需的文件关联优先级，部署 Qt 并进行 ad-hoc 完整性签名，从同一应用生成 ZIP 和 DMG。普通 macOS Qt CI 使用 SDK 验证。Developer ID、Apple 公证和 Windows 发行者证书不在当前发行政策范围内。

Intel 构建单独生成 `macos-x86_64` 应用 ZIP、DMG 和 CLI 归档。Intel 桌面和 CLI 分别使用 `release-intel.json` 和 `cli-release-intel.json`，其签名及 `appcast-intel.xml` 均独立验证。既有 ARM64 清单和 feed 保持原格式，以兼容旧客户端。发行前，两种架构都必须通过原生包验证。

产品源码取自标签指向的提交。草稿、公开发布及公开下载复核均从触发工作流时的 `main` 提交读取同一份已审阅的发布说明。

完整标签等于 `v` 加 VERSION.json 派生的语义版本。`prerelease_number` 是可选正整数，缺省兼容历史构建号；alpha.1 与 build 14 是不同身份。Windows 数值资源和 macOS CFBundleVersion 仍使用单调递增构建号。公开同一版本的资产不可替换，修订必须使用新版本；草稿可重跑。

发布说明是持久且可审阅的版本输入，默认读取 `release-notes/<完整版本>.md`；CLI 也可通过 `--notes-file` 指定已审阅文件。文件含一个 `<!-- xdzip-release {"version":"0.1.0-alpha.6","build":19} -->` 元数据行及对应版本标题，版本与构建号必须匹配 VERSION.json。先运行 `python3 scripts/publish-release.py --check-notes`，该检查不联系 GitHub，专用工作流在构建前执行同样的预检。更新版本须新建对应说明，不自动借用旧版功能列表或验收状态。

说明按平台、实际提交和证据记录中英文验收边界，已有人工验收与未执行项分别保留；阶段进展仍由 issue 管理。初建草稿和公开使用同一份说明快照。草稿重跑若发现其正文与已审阅文件不同，会在上传／删除资产前停止，要求显式对齐输入；公开版本继续拒绝重跑替换资产。将待发布的变更记在[变更日志的未发布章节](../../../CHANGELOG.zh-CN.md#unreleased)，公开后再移入对应版本。同步更新所有语言的变更日志。

发布器在改动草稿资产前回读 GitHub 标签、草稿状态和渠道标记。stable 渠道必须保持非预发布状态并成为 GitHub Latest；alpha、beta、rc 渠道必须保持预发布状态且不能取代 Latest。公开操作后及公开附件复核结束后再次核对相应状态，全部符合才报告成功。公开后的核对失败时，版本可能已经对外可见，脚本停止并要求所有者处理，不自动撤回、修改或替换已公开版本及资产。

正式发行前完成本地预检并审阅对应版本说明。包验收在同一原生平台独立运行，核对交接工件的仓库、运行编号、提交、平台、架构和归档 SHA-256。Windows 和 Linux 各验证一次已部署运行时，并执行实际安装器生命周期检查。macOS 核验实际交付的 DMG 和应用 ZIP。按需候选运行使用相同作业，组装后重新下载 Actions 工件并核对校验和。候选模式使用只读仓库权限，无需标签或发行签名密钥。

发行契约作业先构建同一提交的隔离格式工作进程，再通过 `XDZIP_NATIVE_CODEC` 与 `XDZIP_NATIVE_CODEC_WORKER` 运行 Rust 工作区测试。三平台原生作业分别验证实际应用、部署和安装生命周期。macOS 开发版文件关联测试使用已编译应用的临时副本，并为副本设置开发版关联元数据。原生启动和打包使用带有正式关联优先级的发行应用。

Windows 每次部署核验使用独立的临时测试副本，保留此前日志；重复核验不复用或删除已有目录，产品运行目录不包含测试 DLL。

发布固定附带 `release.json` 与 `SHA256SUMS`。清单 `schema_version=1`，根字段为 product、version、channel、build、tag、commit（完整 40 位 SHA）、repository、assets。每个资产含 filename、platform、architecture、package_type、minimum_system、bytes、sha256、url，URL 固定到标签下载路径。

| 平台、架构 | 包类型 | 最低系统 |
| --- | --- | --- |
| windows、x86_64 | zip、exe | Windows 11 受维护版本 |
| macos、arm64 | app.zip、dmg | macOS 14.0 |
| linux、x86_64 | tar.gz、deb | Ubuntu 24.04、glibc 2.39、X11 |

六类包必须齐备，清单校验拒绝缺包、重复目标、不一致渠道、可变下载地址、错误字节数或哈希。发行附件包含平台通知、许可、SBOM 和部署／验证材料，不包含对应源码压缩包。Windows 使用 app-local VC++ 运行库，不随包分发 Windows SDK 图形编译库。

自 2026-11-05 起，所有者决定从 GitHub Releases 和 Actions 工件中撤下历史对应源码压缩包，并停止后续发行流程生成或上传此类压缩包。`scripts/prepare-source-delivery.py` 仍可用于本地审查；发布清单器和发布器拒绝文件名含 `corresponding-source` 的归档。该发布政策不改变任何组件的适用许可，也不表示对应源码、重链接或其他义务已经满足。发行审查必须针对实际交付继续处理这些义务；若条款要求的条件没有其他可行交付方式，应阻止受影响二进制的分发。GitHub 公共仓库自动生成的标签源码快照不属于这些 Release 附件。

常规发行不得自行在本地运行源码交付收集器；只有用户针对源码收集另行明确要求时才运行，且仍遵守上传限制。通用机器待复核标记须结合实际组件及具体义务判断，不能单凭标记认定出现新的发行缺陷。

macOS 的应用 ZIP、DMG 与通知材料必须在同一已部署、签名的临时应用仍存活时导出，随后才清理暂存 `.app` 和临时注册。不能在暂存清理后重新部署另一份应用来补 ZIP。

顺序为本地预检、冻结版本提交、创建标签、手动启动一次 Release 工作流、三平台构建与独立启动及安装生命周期核验、汇总草稿清单、全部齐备后按渠道公开 Release、再次下载并核对所有附件。stable 公开为普通 Release 并显式标记 Latest；alpha、beta、rc 公开为 Pre-release。任何平台失败时草稿保持未公开。发布前后均按 SHA256SUMS 复验。许可来源复核或人工验收未完成时必须在说明中保留限制，不以自动化测试冒充其实际结论。

更新客户端分页获取 GitHub Releases 列表，跳过草稿，校验清单版本，按语义版本、用户允许渠道、平台、架构、包类型选择。不能按发布时间排序判断升级，也不能猜测文件名。预发布不使用排除预发布的 `/releases/latest`。稳定渠道只接收稳定版本，alpha 允许 alpha、beta、rc 和稳定版，beta 不回到 alpha，rc 仅接收 rc 和稳定版；开发版本比较支持 dev。升级的语义版本与单调 build 均须递增。下载后校验大小与 SHA-256，任何缺失或不一致拒绝使用。

## 可信更新附件

新发行附带 `release.json.sig`，格式为 schema_version=1、algorithm=ed25519、key_id 和 128 字符小写十六进制 signature。签名覆盖 `release.json` 的精确原始字节，不能解析后重新序列化再验签。客户端和无 Qt 安装助手使用各自内置的 `resources/update-trust.json` 公钥集；不接受下载的清单或用户请求携带的信任根。未知公钥、重复 key_id、弱公钥和空信任集均拒绝安装。

原生 macOS 作业打包应用 ZIP 并核验交付包。Linux 的 `sign` 作业取得已核验的交付字节后，运行 `scripts/sign-macos-update.py`，由 `xdzip-update-tool sign-archive` 生成 `macos-update.json`，记录 schema_version、version、build、filename、bytes、sha256 与 Sparkle base64 `ed_signature`。汇总阶段重新核对归档及其签名，生成只含同一个不可变候选的 `appcast.xml`，清单可选字段 `macos_appcast_url` 固定指向同一标签下的该文件。feed 使用 build 作 Sparkle version、完整语义版本作 shortVersionString、macOS 14.0 作最低系统版本，仅内嵌简短版本说明。完整发行说明仍作为 GitHub Release 正文，客户端不读取外部发布说明。

归档和 feed 使用独立 Sparkle 密钥，feed 自身也必须签名。跨平台发布工具按 [Sparkle 上游签名格式](https://github.com/sparkle-project/Sparkle/blob/2.x/common_cli/Signing.swift)对完整原始 XML 签名，并在末尾追加 sparkle-signatures 注释块，包含 base64 edSignature 与原始字节 length；这不是只给 enclosure 添加签名。生成后再次独立验证，修改任意字节须重新签名。Mac 包的 SUPublicEDKey 必须与 `resources/sparkle-update-key.json` 中的公钥一致。

发布环境通过 `XDZIP_UPDATE_SIGNING_KEY_HEX`、`XDZIP_UPDATE_KEY_ID` 和 `XDZIP_SPARKLE_SIGNING_KEY_HEX` 提供已有密钥和公开 key ID。两份生产公钥资源由所有者提供，随客户端和安装助手纳入版本管理，不填测试公钥，也不在构建时生成私钥。所有者先将公开信任根纳入客户端版本，再在可信发布环境配置匹配私钥；密钥轮换先交付新公钥，再改为新密钥签名，不能使旧客户端失去验证路径。未内置这些公钥的旧客户端须先手动安装包含公钥的版本，不能通过下载的清单增加信任根；旧公开发行不补签或替换资产。

发布脚本先在移除两份私钥环境变量的子进程中构建签名工具，仅已编译的工具进程接收签名私钥；Cargo build script、编译器和依赖解析不获得发布签名权限。

签名作业在 `release-signing` 环境中为已验证的应用归档和更新清单签名。独立发布作业也需要该环境审批，公开前核对签名并重新下载草稿附件。最后由只读作业下载公开附件，与已签名工件核对。各作业均使用同一冻结提交及同一次运行生成的工件。

[生产更新验收](../../../.github/workflows/production-update-acceptance.yml)是另一个仅手动执行、固定 `main` 和 `contents: read` 的入口，使用同一 `release-signing` 环境及其所有者审批，不调用 Release 工作流，也不创建标签、草稿或发行。先在无秘密步骤编译工具，再在独立步骤派生已有私钥对应的公钥，与本次提交中的两份公钥和公开 key ID 严格比较；条目存在不能代替匹配成功。三平台私有旧版 200 与新版 201 保留生产公钥，构建不接收秘密。签名步骤仅接受本次源码提交、私有产品身份及完整哈希绑定的固定候选，随后在无秘密进程中执行原生升级、失败恢复和普通重启。没有任意输入清单、资产、远端工件或发布参数。

macOS 在无秘密的工具链准备步骤中一次安装 `rust-toolchain.toml` 声明的 `rustfmt` 和 `clippy`，再执行归档准入测试。测试密钥更新回归也覆盖该准备和准入流程；它不使用或验证生产私钥。

生产验收的安装包、原始清单、签名、appcast 和应用仅留在一次性 runner 内，不上传；有界收集器只上传明确允许的日志、状态、身份及哈希记录，并拒绝签名附件误入结果槽。生产密钥匹配、签名验证、实际安装及人工桌面验收分别记录。macOS 使用原生 Apple Silicon 和实际 `SUPublicEDKey` 核对；Windows Server 的夹具仍不代替 Windows 11 人工验收。此流程不会读取 GitHub Secret 的值到本地，也不授权发布。

`prepare-release.py --require-update-signature` 在缺少任一密钥、签名或原生归档元数据时停止；`publish-release.py` 独立复验清单、同候选 feed 与归档后才上传。签名、appcast 和状态附件均进入完整 SHA256SUMS。已有公开版本不补传或替换这些新附件。







## 独立 CLI 附件

从 v0.2.0 起，完整发行额外提供 `xdzip-cli-<version>-windows-x86_64.zip`、`xdzip-cli-<version>-macos-arm64.tar.gz`、`xdzip-cli-<version>-linux-x86_64.tar.gz`。三者由同提交的原生 CLI 构建与验证产生，保留 worker、实际运行依赖与通知，身份须为干净 Release。普通 CI 不因新增 CLI 而部署或上传 Mac 应用包。

CLI 使用 `cli-release.json` 与 `cli-release.json.sig`。清单 schema_version=1、product=xdzip、component=cli，version、channel、build、tag、commit、repository 与桌面清单完全一致。assets 只包含上述三个目标，字段与 desktop asset 相同；Mac 包型为 tar.gz，Linux 最低系统不含 display_server，其他系统、架构和大小／摘要要求不变。固定发布者仍为 xiaodaozhi/xdzip。

桌面 release.json 继续恰好六类包，不加入 component 或 CLI 条目。旧客户端忽略新增独立附件。CLI 清单签名复用内置 Ed25519 信任根，签署和独立验证使用 sign-cli／verify-cli，覆盖精确原始字节。完整汇总和发布在签名、身份、三个 CLI 包及全部校验和齐备后才能公开。已公开旧版本不补传、改写或替换附件。
