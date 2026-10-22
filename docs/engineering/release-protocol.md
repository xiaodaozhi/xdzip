# GitHub Release 协议 v1

专用 [Release 工作流](../../.github/workflows/release.yml)由版本标签或手动重跑已有标签触发。普通 macOS CI 仍只进行 SDK 验证；专用发布流程明确允许 Apple Silicon 原生 Qt 部署、必要的 ad-hoc 完整性签名、同一应用的 ZIP 和 DMG 打包与核验。未使用 Developer ID、Apple 公证或 Windows 发行者证书。

完整标签等于 `v` 加 VERSION.json 派生的语义版本。`prerelease_number` 是可选正整数，缺省兼容历史构建号；alpha.1 与 build 14 是不同身份。Windows 数值资源和 macOS CFBundleVersion 仍使用单调递增构建号。公开同一版本的资产不可替换，修订必须使用新版本；草稿可重跑。

发布说明是持久且可审阅的版本输入，默认读取 `release-notes/<完整版本>.md`；CLI 也可通过 `--notes-file` 指定已审阅文件。文件含一个 `<!-- xdzip-release {"version":"0.1.0-alpha.6","build":19} -->` 元数据行及对应版本标题，版本与构建号必须匹配 VERSION.json。先运行 `python3 scripts/publish-release.py --check-notes`，该检查不联系 GitHub，专用工作流在构建前执行同样的预检。更新版本须新建对应说明，不自动借用旧版功能列表或验收状态。

说明按平台、实际提交和证据记录中英文验收边界，已有人工验收与未执行项分别保留；阶段进展仍由 issue 管理。初建草稿和公开使用同一份说明快照。草稿重跑若发现其正文与已审阅文件不同，会在上传／删除资产前停止，要求显式对齐输入；公开版本继续拒绝重跑替换资产。`CHANGELOG.md` 只归档实际公开版本，本地开发交付不补造为公开版本史。

冻结版本标签前可手动以候选提交 SHA 和 `validate_only=true` 运行同一流水线，实际构建、部署、签名、核验并汇总完整附件，保留 release-validation 工件，不创建草稿或公开 Release。正式发布仍使用版本标签，不能以提交 SHA 冒充标签。此演练避免首次打包错误发生在不可变标签创建之后。

Windows 每次部署核验使用独立的临时测试副本，保留此前日志；重复核验不复用或删除已有目录，产品运行目录不包含测试 DLL。

发布固定附带 `release.json` 与 `SHA256SUMS`。清单 `schema_version=1`，根字段为 product、version、channel、build、tag、commit（完整 40 位 SHA）、repository、assets。每个资产含 filename、platform、architecture、package_type、minimum_system、bytes、sha256、url，URL 固定到标签下载路径。

| 平台、架构 | 包类型 | 最低系统 |
| --- | --- | --- |
| windows、x86_64 | zip、exe | Windows 11 受维护版本 |
| macos、arm64 | app.zip、dmg | macOS 14.0 |
| linux、x86_64 | tar.gz、deb | Ubuntu 24.04、glibc 2.39、X11 |

六类包必须齐备，清单校验拒绝缺包、重复目标、不一致渠道、可变下载地址、错误字节数或哈希。对应源码、Rust vendor、Qt 原始源码与补丁、重建说明、平台通知和许可、SBOM 以同版本附件提供。Windows 继续使用 app-local VC++ 运行库与图形兼容库。

macOS 的应用 ZIP、DMG 与通知材料必须在同一已部署、签名的临时应用仍存活时导出，随后才清理暂存 `.app` 和临时注册。不能在暂存清理后重新部署另一份应用来补 ZIP。

顺序为同提交验证、冻结版本提交、创建并推送不可变标签、三平台构建与独立启动及安装生命周期核验、汇总草稿清单、全部齐备后公开 Pre-release、再次下载并核对所有附件。任何平台失败时草稿保持未公开。发布前后均按 SHA256SUMS 复验。许可来源复核或人工验收未完成时必须在说明中保留限制，不以自动化测试冒充其实际结论。

更新客户端分页获取 GitHub Releases 列表，跳过草稿，校验清单版本，按语义版本、用户允许渠道、平台、架构、包类型选择。不能按发布时间排序判断升级，也不能猜测文件名。预发布不使用排除预发布的 `/releases/latest`。稳定渠道只接收稳定版本，alpha 允许 alpha、beta、rc 和稳定版，beta 不回到 alpha，rc 仅接收 rc 和稳定版；开发版本比较支持 dev。升级的语义版本与单调 build 均须递增。下载后校验大小与 SHA-256，任何缺失或不一致拒绝使用。

## 可信更新附件

新发行附带 `release.json.sig`，格式为 schema_version=1、algorithm=ed25519、key_id 和 128 字符小写十六进制 signature。签名覆盖 `release.json` 的精确原始字节，不能解析后重新序列化再验签。客户端和无 Qt 安装助手使用各自内置的 `resources/update-trust.json` 公钥集；不接受下载的清单或用户请求携带的信任根。未知公钥、重复 key_id、弱公钥和空信任集均拒绝安装。

macOS 原生阶段在最终 app.zip 定名后，通过 `xdzip-update-tool sign-archive` 输出 `macos-update.json`，记录 schema_version、version、build、filename、bytes、sha256 与 Sparkle base64 `ed_signature`。汇总阶段重新核对归档及其签名，生成只含同一个不可变候选的 `appcast.xml`，清单可选字段 `macos_appcast_url` 固定指向同一标签下的该文件。feed 使用 build 作 Sparkle version、完整语义版本作 shortVersionString、macOS 14.0 作最低系统版本，发布说明内嵌，避免读取未签名的外部说明。

归档和 feed 使用独立 Sparkle 密钥，feed 自身也必须签名。跨平台发布工具按 [Sparkle 上游签名格式](https://github.com/sparkle-project/Sparkle/blob/2.x/common_cli/Signing.swift)对完整原始 XML 签名，并在末尾追加 sparkle-signatures 注释块，包含 base64 edSignature 与原始字节 length；这不是只给 enclosure 添加签名。生成后再次独立验证，修改任意字节须重新签名。Mac 包的 SUPublicEDKey 必须与 `resources/sparkle-update-key.json` 中的公钥一致。

发布环境通过 `XDZIP_UPDATE_SIGNING_KEY_HEX`、`XDZIP_UPDATE_KEY_ID` 和 `XDZIP_SPARKLE_SIGNING_KEY_HEX` 提供已有密钥和公开 key ID。两份生产公钥资源初始保持未配置，不填测试公钥，也不在构建时生成私钥。所有者先将公开信任根纳入客户端版本，再在可信发布环境配置匹配私钥；密钥轮换先交付新公钥，再改为新密钥签名，不能使旧客户端失去验证路径。

发布脚本先在移除两份私钥环境变量的子进程中构建签名工具，仅已编译的工具进程接收签名私钥；Cargo build script、编译器和依赖解析不获得发布签名权限。

正式原生签名与汇总阶段绑定受保护的 `release-signing` GitHub environment，所有者须配置匹配的秘密和环境保护规则。Mac 私钥只注入原生构建后的独立归档签名步骤，构建阶段不接收。validate_only 使用 `release-validation` 环境，不要求生产密钥；专用更新回归使用自己的临时测试密钥。配置环境和生产信任根是实际发布前提，不由普通 CI 自动创建。

`prepare-release.py --require-update-signature` 在缺少任一密钥、签名或原生归档元数据时停止；`publish-release.py` 独立复验清单、同候选 feed 与归档后才上传。validate_only 可以生成明确标记 signing_not_configured 的候选，`update-readiness.json` 不作为授权依据，也不把未签名演练称为可安装更新。签名、appcast 和状态附件均进入完整 SHA256SUMS。已有公开版本不补传或替换这些新附件。

English: Every immutable release includes schema-v1 release.json and SHA256SUMS. All six native package targets must be present and hash-verified before a draft becomes public. Metadata binds the complete semantic version, channel, monotonic build, tag and full commit to platform/architecture/package requirements and pinned URLs. Clients paginate Releases, compare semantic versions and builds, and never regress from beta/rc to an earlier channel. They must not use publication dates, filename guesses or /releases/latest for prereleases. Installation additionally requires an Ed25519 release.json.sig over exact JSON bytes and a locally embedded trusted public key. macOS uses a separate Sparkle key to sign both the final app.zip and the entire single-candidate appcast, with embedded notes and a pinned macos_appcast_url. Production public keys start unconfigured, and test keys must never be embedded. Validation-only candidates may explicitly report signing_not_configured; publication independently rejects missing or invalid signatures and never changes existing public assets. macOS application code signing remains ad-hoc; update signatures do not impersonate Developer ID or notarization. Incomplete licensing checks and unperformed manual acceptance remain explicit.

Validation-only dispatches accept a candidate commit SHA and build, deploy, sign, verify and assemble the complete attachment set without creating a draft or public Release. Normal publication requires the exact version tag.

Each Windows deployment verification uses a fresh test copy and preserves prior evidence. Repeated verification never reuses or deletes an existing harness, and test DLLs stay outside the product runtime.

Release notes come from a reviewed, version-specific Markdown file with matching version/build metadata. Offline notes preflight runs before native builds. Draft creation, retries and publication use the same reviewed body; a changed draft stops publication before any asset mutation. Manual acceptance remains scoped to the recorded platform and commit, in both languages. Missing notes for a new version never inherit old acceptance. Only actually published versions are archived in the changelog.
