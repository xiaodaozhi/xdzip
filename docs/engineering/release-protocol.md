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

未来更新客户端分页获取 GitHub Releases 列表，跳过草稿，校验清单版本，按语义版本、用户允许渠道、平台、架构、包类型选择。不能按发布时间排序判断升级，也不能猜测文件名。预发布不使用排除预发布的 `/releases/latest`。下载后校验大小与 SHA-256，任何缺失或不一致拒绝使用。本次仅准备发布协议与选择测试，不实现自动更新或自替换。

English: Every immutable release includes schema-v1 release.json and SHA256SUMS. All six native package targets must be present and hash-verified before a draft becomes public. Metadata binds the complete semantic version, channel, monotonic build, tag and full commit to platform/architecture/package requirements and pinned URLs. Future clients paginate Releases, filter allowed channels, compare semantic versions and verify downloaded bytes; they must not use publication dates, filename guesses or /releases/latest for prereleases. No updater is implemented by this change. macOS uses ad-hoc integrity signing only. Incomplete licensing checks and unperformed manual acceptance remain explicit.

Validation-only dispatches accept a candidate commit SHA and build, deploy, sign, verify and assemble the complete attachment set without creating a draft or public Release. Normal publication requires the exact version tag.

Each Windows deployment verification uses a fresh test copy and preserves prior evidence. Repeated verification never reuses or deletes an existing harness, and test DLLs stay outside the product runtime.

Release notes come from a reviewed, version-specific Markdown file with matching version/build metadata. Offline notes preflight runs before native builds. Draft creation, retries and publication use the same reviewed body; a changed draft stops publication before any asset mutation. Manual acceptance remains scoped to the recorded platform and commit, in both languages. Missing notes for a new version never inherit old acceptance. Only actually published versions are archived in the changelog.
