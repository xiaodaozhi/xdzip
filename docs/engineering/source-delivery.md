# 对应源码与重建材料

scripts/prepare-source-delivery.py 从干净提交收集项目源码、Cargo.lock 对应的完整 Rust vendor 输入、固定 Qt 6.8.2 模块源码归档和重建／替换说明。所有产物位于 build/，不把下载地址称为已交付源码，不自动批准开源许可分支。

```sh
python3 scripts/prepare-source-delivery.py --output build/review/source-delivery \
  --qt-prefix .cache/Qt/6.8.2/macos
python3 scripts/verify-source-delivery.py --delivery build/review/source-delivery \
  --qt-prefix .cache/Qt/6.8.2/macos --output build/review/source-rebuild
```

Qt 运行时模块归档为 qtbase、qtdeclarative、qtsvg、qttranslations，另交付构建所需的 qtshadertools、qttools 及固定 Rust 标准库源码组件，来自 [Qt 官方 6.8.2 归档](https://download.qt.io/archive/qt/6.8/6.8.2/submodules/)。官方 .sha256 值固定于 scripts/corresponding-sources.json；缓存与下载均重新校验，下载先写 partial，再验证后提交。只检查归档目录，不自动解包第三方路径。Cargo vendor 使用 --locked，保留 registry 校验与构建依赖，不执行第三方构建脚本。

源码交付另含 Qt Network 内嵌 Public Suffix List 的固定可编辑原始列表和 MPL-2.0 原文。收集时先核对列表、Qt 归档和生成器身份，再分别生成 C++ 数据及二进制测试数据，并与 Qt 归档中的文件逐字节比较；任一不符均停止交付。此对应关系不代表整个 SDK 构建等价或自动批准发行许可。

English: Source delivery also includes the pinned editable Public Suffix List and its MPL-2.0 text. Collection verifies the input and Qt archive, then regenerates and compares both the C++ data and binary test data byte for byte. It stops on a mismatch and does not establish full SDK equivalence or approve release licensing.

源码交付还保存 Qt Core MIME 数据所用的固定 Apache Tika 原始文件、上游 NOTICE 和 OGL-UK-3.0 原文。收集器用固定 Qt 归档内的转换脚本重建 Qt 的 MIME 数据，并核对归档声明和生成结果；仅允许 Windows 本地换行差异。完整 Tika NOTICE 覆盖整个上游发行，不表示其其他依赖包含于 xdzip。

English: Source delivery retains the pinned Apache Tika input, upstream NOTICE and OGL-UK-3.0 text for Qt Core MIME data. It verifies the Qt archive attribution and regenerates its MIME data, allowing only native Windows line endings. The complete Tika NOTICE covers the broader upstream distribution and does not imply its other dependencies are shipped by xdzip.

源码交付另含 Qt 6.8.2 区域数据生成所用的 CLDR 46 `common` 源码和上游许可，不包含 CLDR 源码树的其余部分。收集时核对 Qt 源码归档中两个生成文件的 CLDR 版本，以及固定上游归档摘要和源文件。Qt 的第二步生成器还需要 ISO 639-3 表，该表不在交付中；试验用候选的来源、版本和摘要记录在 `qt-cldr-sources.json`，其[上游下载页面和条款](https://iso639-3.sil.org/code_tables/download_tables)须单独核对。候选可重现生成数据不证明它是原上游构建的精确输入，也不证明原始 SDK 的完整构建身份。

English: The delivery also contains the CLDR 46 `common` source and upstream license used by Qt 6.8.2 locale generation, not the rest of the CLDR source tree. Collection verifies the CLDR version in two generated Qt source files, the pinned upstream archive hash and selected inputs. Qt's second generation step additionally requires an ISO 639-3 table that is not included; `qt-cldr-sources.json` records the tested candidate's source, version and digest, and its [upstream download page and terms](https://iso639-3.sil.org/code_tables/download_tables) require separate review. Reproducing generated data with this candidate does not establish the precise historical input or the original SDK's complete build identity.

对 Qt 源码中已有的语言，还可从原 `qlocale_data_p.h` 提取一个供 Qt 生成器读取的有限输入。例如，将 QtBase 归档解包到当前目录后运行 `python3 scripts/derive-qt-locale-code-input.py --qtbase qtbase-everywhere-src-6.8.2 --output build/qt-locale-codes.tsv`。此输入仅覆盖原文件中已有的查找项，不是完整 ISO 639-3 表，也不是上游历史输入；新增语言时可能仍需另行取得代码并核对条款。用固定 Qt 6.8.2 源码、CLDR 46 数据及此派生输入重生成八个源码文件的验证，仅允许生成日期和换行差异。该方法不自动解决 ISO 表的授权或完整对应源码判断。

English: For languages already present in the Qt source, `scripts/derive-qt-locale-code-input.py` extracts a limited input from the original `qlocale_data_p.h`; the command and output location are shown above. This covers only existing lookup keys and is neither the complete ISO 639-3 table nor the historical upstream input. Adding languages may still require separately obtained codes and review of their terms. Regeneration of eight files with the pinned Qt 6.8.2 source, CLDR 46 data and this derived input was checked allowing only generation-date and line-ending differences. This does not by itself settle the ISO table's licensing or establish complete corresponding source.

另一种方式是自行取得 `iso-codes` 的 ISO 639-2 与 639-3 JSON 数据，再用转换器生成 Qt 可读取的输入。例如，在安装了该数据包的 Linux 环境中执行：

```sh
python3 scripts/convert-iso-codes-for-qt.py \
  --iso-639-2 /usr/share/iso-codes/json/iso_639-2.json \
  --iso-639-3 /usr/share/iso-codes/json/iso_639-3.json \
  --output build/qt-iso-codes.tsv
```

转换器只提取语言代码及其映射，不保留名称；若输入是完整代码集，输出仍包含完整代码集。它拒绝无效代码、重复代码和 Qt 查找键冲突，并跳过 ISO 639-2 的本地使用保留范围。工具输出输入及结果的 SHA-256。Qt 6.8.2 与 Ubuntu `iso-codes` 4.16.0-1 的重建对照见 [#138](https://github.com/xiaodaozhi/xdzip/issues/138)；其他版本须自行检查生成结果。`iso-codes` 数据不随本交付提供，其声明许可、来源及生成表的分发条件需由获取者核对；可用替代表不证明它是 Qt 历史输入，也不自动完成对应源码或许可判断。

English: As another option, obtain the `iso-codes` ISO 639-2 and 639-3 JSON data independently and run the command above. The converter extracts code mappings without names; a complete input still produces a complete code set. It rejects invalid or ambiguous inputs, skips the reserved local-use range and reports input and output SHA-256 values. See [#138](https://github.com/xiaodaozhi/xdzip/issues/138) for the Qt 6.8.2 comparison with Ubuntu `iso-codes` 4.16.0-1; check results separately for other versions. The data is not included in this delivery. Review its stated license, provenance and redistribution terms for the generated table independently; an alternative table does not establish Qt's historical input or complete source compliance.

输出包含实际固定提交的 xdzip-source.tar、rust-vendor.tar.gz、离线 Cargo 配置、Qt 原始归档、REBUILD.md 和逐文件 SHA-256／大小清单。清单明确为 collected-not-compliance-verified，不能替代 SDK 构建 features／补丁、平台运行库、内嵌资源和许可履约的核对。项目源码重新配置的身份可能为 unknown，应由外部交付清单绑定原提交，不能伪造干净 Git 身份。

`rar-wire-mappings/` 单独保留有限 RAR 线映射的固定参考来源、说明和 UnRAR 7.30 beta1 精确原始许可，全部进入交付文件哈希清单。下载归档名为 `unrarsrc-7.3.1.tar.gz`，不据文件名另推上游发行版本。交付前还核对相关 Rust 源码注释中完整保留的上游许可第 2 段，作为保守通知措施；缺失或变更时停止。它不携带官方引擎源码，也不将事实来源重新许可为 Apache-2.0，条款适用范围和实际再分发义务继续按候选复核。

English: The separate rar-wire-mappings directory preserves bounded finite-mapping provenance and the exact original UnRAR 7.30 beta1 notice in the hashed delivery. The pinned download filename is unrarsrc-7.3.1.tar.gz; its numeric filename does not establish a separate upstream release version. As a precaution, delivery also requires the complete original second license paragraph in the related Rust source comments. These materials include no official engine source and do not relicense reference-derived facts as Apache-2.0 or replace candidate-specific redistribution review.

另附固定的 ICU 73.2 完整许可、ICU4C 官方发布源码归档、包含可编辑数据的固定 ICU 源码树，以及 CLDR 43.1 源码子集，服务于 Linux ICU，并纳入交付总清单。CLDR 子集明确排除五个 ISO 639-3 文件，原归档摘要和排除清单保留在交付记录中；是否存在可转授的额外授权仍待核实。收集时校验归档摘要、版本、归档内许可及指定源文件，不解包第三方路径；上游版本相符不证明供应商二进制由这些材料无补丁构建。当前 Windows 开发包不包含软件 OpenGL 库，不再为该库收集 Mesa／LLVM 材料；历史交付保留原有材料和义务，见[原生运行库来源与通知](native-runtime-materials.md)。

English: The pinned ICU license, official ICU4C 73.2 release source archive, pinned ICU source tree containing editable data and a CLDR 43.1 source subset are included for Linux. The CLDR subset excludes five ISO 639-3 files; the delivery record retains the upstream archive hash and exclusions. Whether additional transferable permission exists remains to be established. Collection checks the archive hashes, version, embedded license and selected source files without extracting third-party paths. Matching upstream versions do not prove the supplier built its binaries from these inputs without patches. Current Windows developer packages omit the software OpenGL library and its dedicated Mesa/LLVM materials; historical deliveries retain their original materials and obligations.

应用动态链接 Qt，提供源码重建入口和兼容库替换说明。[macOS 部署核验](macos-deployment.md)覆盖隔离副本的修改库加载，不声称已从源码重建整套 Qt。实际 Qt 源码构建与产品加载另有独立流程，见[自动验证契约](wp12-automation.md)。SDK 字节级构建等价性、精确静态链接归属和许可分支仍属于[发行许可复核](https://github.com/xiaodaozhi/xdzip/issues/4)，不重新列为已完成 WP-12 的开发待办。

verify-source-delivery.py 先重新核对交付文件哈希，将自有项目和 vendor 安全解到新的 build/ 目录，使用空 Cargo 缓存、CARGO_NET_OFFLINE=true 和现有固定 Qt SDK 构建真实应用，再执行启动／关闭冒烟。它验证项目与 Rust 交付输入可离线使用，不声称 Qt 已从模块源码重建，不声称字节级可重复构建。生成源码目录的 Git 身份为 unknown，由外部固定提交及交付清单绑定。

源码交付在输出目录内以相对 `vendor` 参数运行 Cargo，并显式指定项目清单；只接受当前受支持的 crates.io 源映射与相对 `vendor` 配置。输出包含绝对目录、路径逃逸或意外源映射时直接失败，避免将构建机路径写入可迁移交付。

English: Source delivery invokes Cargo in the output directory with a relative `vendor` destination and an explicit project manifest. Unexpected source mappings or non-relocatable paths are rejected.

对于锁定 vendor 源码中没有常见文件名的独立许可或通知文件的包，交付在 `rust-vendor-notices/` 另存按包索引的固定上游原文。收集器核对包的 Cargo 清单哈希、上游 Git 提交与路径、许可原文 SHA-256 和 Git blob；未知缺口或身份不符时停止。补充目录不改变 Cargo vendor 原文件及校验，许可表达式保留上游原样，不由此批准具体许可分支或既有发行的合规状态。

English: For locked vendor source packages without conventionally named notice files, the delivery separately indexes pinned upstream notice texts by package. The collector verifies package identity, upstream revision and notice hashes, and stops on unknown gaps. It does not modify Cargo vendor files or approve a license choice or an earlier release.

新增[Qt SDK 对应核验](qt-sdk-correspondence.md)：交付包总是包含固定对应清单和可重放的 AGL 消费端补丁，传入 --qt-prefix 后额外包含该 SDK 的逐文件源码声明核对、构建配置和明确缺口。交付清单递归哈希补丁子目录，重建验证同样复核这些字节。未传 SDK 时不伪造配置核验记录。

交付还保留固定的 Qt Declarative Git 提交对象输入，供离线核对发布提交与归档 `.tag` 树对象的关系；交付清单会哈希该输入。树对象对应不代表源码归档覆盖整个树，或 SDK 与该树构建等价。

English: The delivery retains a pinned Qt Declarative Git commit object for offline verification of the release commit and the archive's tree marker. This relationship does not establish complete archive coverage or SDK build equivalence.

实际 Qt 源码构建和加载／兼容替换流程见[自动化收尾契约](wp12-automation.md)，不再把应用使用现有 SDK 的离线重建误称为 Qt 本身的源码重建。
