# 对应源码与重建材料

[English](../../engineering/source-delivery.md) | 简体中文 | [繁體中文](../../zh-TW/engineering/source-delivery.md) | [日本語](../../ja-JP/engineering/source-delivery.md) | [Español](../../es-ES/engineering/source-delivery.md)

**发行边界，2026-11-05：**按所有者决定，此收集器仅供本地审查和重建验证，不再把对应源码压缩包作为 GitHub Release 或 Actions 工件提供。公开历史 Release 和仍可下载的发行 Actions 工件中的对应源码包已撤下，后续发布流程禁止生成或上传。该决定不改变组件许可，不表示对应源码、重链接或其他许可义务已满足；GitHub 公共仓库自动生成的标签源码快照不属于这些 Release 附件。

常规发行及其验收不得自行运行此收集器，也不得在本地生成对应源码包；只有用户针对源码收集另行明确要求时才可运行，上传限制仍然适用。许可复核针对实际组件和具体义务进行，不将机器报告中的通用待复核标记直接认定为实际缺陷。


scripts/prepare-source-delivery.py 从干净提交收集项目源码、Cargo.lock 对应的完整 Rust vendor 输入、固定 Qt 6.8.2 模块源码归档和重建／替换说明。所有产物位于 build/，不把下载地址称为已交付源码，不自动批准开源许可分支。

```sh
python3 scripts/prepare-source-delivery.py --output build/review/source-delivery \
  --qt-prefix build/toolchain/Qt/6.8.2/macos
python3 scripts/verify-source-delivery.py --delivery build/review/source-delivery \
  --qt-prefix build/toolchain/Qt/6.8.2/macos --output build/review/source-rebuild
```

Qt 运行时模块归档为 qtbase、qtdeclarative、qtsvg、qttranslations，另交付构建所需的 qtshadertools、qttools 及固定 Rust 标准库源码组件，来自 [Qt 官方 6.8.2 归档](https://download.qt.io/archive/qt/6.8/6.8.2/submodules/)。官方 .sha256 值固定于 scripts/corresponding-sources.json；缓存与下载均重新校验，下载先写 partial，再验证后提交。只检查归档目录，不自动解包第三方路径。Cargo vendor 使用 --locked，保留 registry 校验与构建依赖，不执行第三方构建脚本。

源码交付另含 Qt Network 内嵌 Public Suffix List 的固定可编辑原始列表和 MPL-2.0 原文。收集时先核对列表、Qt 归档和生成器身份，再分别生成 C++ 数据及二进制测试数据，并与 Qt 归档中的文件逐字节比较；任一不符均停止交付。此对应关系不代表整个 SDK 构建等价或自动批准发行许可。

源码交付还保存 Qt Core MIME 数据所用的固定 Apache Tika 原始文件、上游 NOTICE 和 OGL-UK-3.0 原文。收集器用固定 Qt 归档内的转换脚本重建 Qt 的 MIME 数据，并核对归档声明和生成结果；仅允许 Windows 本地换行差异。完整 Tika NOTICE 覆盖整个上游发行，不表示其其他依赖包含于 xdzip。

源码交付另含 Qt 6.8.2 区域数据生成所用的 CLDR 46 `common` 源码和上游许可，不包含 CLDR 源码树的其余部分。收集时核对 Qt 源码归档中两个生成文件的 CLDR 版本，以及固定上游归档摘要和源文件。Qt 的第二步生成器还需要 ISO 639-3 表，该表不在交付中；试验用候选的来源、版本和摘要记录在 `qt-cldr-sources.json`，其[上游下载页面和条款](https://iso639-3.sil.org/code_tables/download_tables)须单独核对。候选可重现生成数据不证明它是原上游构建的精确输入，也不证明原始 SDK 的完整构建身份。

不提供原始 ISO 表不意味着其他 CLDR 数据没有相关标识符。CLDR 46 `common/validity/language.xml` 保留压缩表示的语言代码，`common/supplemental/supplementalMetadata.xml` 还含别名。本机对交付子集的[对照](https://github.com/xiaodaozhi/xdzip/issues/138#issuecomment-6588994510)从这两份文件构造的三字母候选集合覆盖 CLDR 46 候选 ISO 表 7,920 个 `Id` 中的 7,919 个，并另含 338 个值；CLDR 43.1 子集按相同方法与较新的同一表跨版本对照覆盖 7,896 个，并另含 338 个值。候选集合不是原表，不含原表的完整字段；标识符重合不决定来源、授权或适用条款。不得以排除五个 ISO 命名文件推断交付中不存在其他相关数据。

CLDR 43.1 子集的工具数据还保留文件头日期为 2023-02-14 的 `language-subtag-registry`。该文件中 8,049 个三字母语言子标签有 7,713 个与上述较新候选 ISO 表重合；不同版本的差额不据此归因。Unicode 的更新说明将此文件列为从 IANA 取得的语言子标签注册表；IANA 的 CC0 声明只覆盖其拥有或有权转授的权利，并保留第三方可能的权利。文件内容对照及限制见 [#140](https://github.com/xiaodaozhi/xdzip/issues/140#issuecomment-6589292716)。

CLDR 43.1 子集还包含网页工具使用的两份 Noto 字体、四份 Glyphicons 文件、Bootstrap 与 Autosize 两份脚本，以及表情图片。源码交付的 `cldr43-webapp-notices/` 单独提供上述字体、脚本和 908 份与两个固定 Noto Emoji 版本逐字节一致的图片的来源与许可，收集时逐份核对固定摘要。另外 963 张原图的来源和分发许可尚未独立确认，收集器核对每张原图摘要后，以 698 张固定版本 Noto 图片和 265 张固定版本 Twemoji 图片替换；交付清单逐项记录原图与替代图的摘要、路径和来源。这不意味着原图侵权。这些文件不属于 xdzip 界面，三份保留 IBM 版权标记的网页工具源码也在该通知中逐项记录，依据该版根目录说明按 Unicode-DFS-2016 处理并保留原版权及许可。网页根目录另有 27 个图片与固定 CLDR 2026 提交逐字节一致，按该提交的 Unicode-3.0 许可保留完整许可和逐项摘要；此判断不扩展到商标或未列出的素材及外部依赖。

CLDR 43.1 子集 `docs/rfc` 中两份完整文本 `draft-davis-t-langtag-ext.txt` 和 `draft-davis-u-langtag-ext-00.txt` 分别与 IETF 保存的 [2011 年 T 扩展草案](https://www.ietf.org/archive/id/draft-davis-t-langtag-ext-07.txt)及 [2010 年 U 扩展草案](https://www.ietf.org/archive/id/draft-davis-u-langtag-ext-00.txt)逐字节一致，源码收集时核对其摘要并保留原版权及声明。两份文本指向发表时生效的 IETF 条款；[TLP 4.0](https://trustee.ietf.org/wp-content/uploads/IETF-TLP-4.pdf)第 3.c 节允许在其可授予权利的范围内于标准流程外完整、未经修改地分发文件，并须保留要求的声明。该判断仅覆盖这两份原文，不自动覆盖其他 HTML、XML、素材或第三方权利。

对 Qt 源码中已有的语言，还可从原 `qlocale_data_p.h` 提取一个供 Qt 生成器读取的有限输入。例如，将 QtBase 归档解包到当前目录后运行 `python3 scripts/derive-qt-locale-code-input.py --qtbase qtbase-everywhere-src-6.8.2 --output build/qt-locale-codes.tsv`。此输入仅覆盖原文件中已有的查找项，不是完整 ISO 639-3 表，也不是上游历史输入；新增语言时可能仍需另行取得代码并核对条款。用固定 Qt 6.8.2 源码、CLDR 46 数据及此派生输入重生成八个源码文件的验证，仅允许生成日期和换行差异。该方法不自动解决 ISO 表的授权或完整对应源码判断。

要在有限输入中增加具体语言，可解出交付的 CLDR 43.1 子集，使用其中 `tools/cldr-code/src/main/resources/org/unicode/cldr/util/data/language-subtag-registry` 校验语言子标签，并明确填写四个代码字段。以下 Choctaw 示例的 ISO 639-2 字段可对照[美国国会图书馆列表](https://www.loc.gov/standards/iso639-2/php/code_list.php)，ISO 639-3 字段可对照[登记机构的数据](https://iso639-3.sil.org/code_tables/download_tables)；末尾空字段表示没有双字母代码：

```sh
python3 scripts/derive-qt-locale-code-input.py --qtbase path/to/qtbase \
  --iana path/to/cldr43/tools/cldr-code/src/main/resources/org/unicode/cldr/util/data/language-subtag-registry \
  --add-language 'cho,cho,cho,' --output build/qt-locale-codes-with-cho.tsv
```

`--add-language` 可重复，四个字段依次为三字母代码、ISO 639-2/B、ISO 639-2/T、双字母代码；没有的字段留空，两个 ISO 639-2 字段须同时填写或同时留空。工具校验三字母或双字母代码至少有一项存在于 IANA 语言子标签、字段格式及 Qt 查找键不冲突，**不**从 IANA 自动推断或证明各字段属于同一语言或 ISO 字段的对应关系；填写者须自行核对具体映射和来源。新增 Qt 语言还须按照 Qt 上游流程修改 `enumdata.py`、提供该语言的 CLDR 区域数据并重新生成、编译，单独增加这一行不会自动新增语言。派生表不包含完整 ISO 代码表，也不证明历史输入或完整对应源码义务。

在隔离测试中，向 Qt 工具枚举和该派生表手工增加一个私用测试代码，可生成新枚举与代码表；Linux QtCore 重建后的正反向代码查询通过，原始 alpha.8 的隔离副本也完成离屏浏览、取消和解压。该实验没有新增真实语言或 CLDR 区域格式数据，不能替代真实语言输入及其条款核对，也不能证明全部对应源码义务。步骤和首次探针失败的记录见 [#138](https://github.com/xiaodaozhi/xdzip/issues/138#issuecomment-6611169790)。

另一种方式是自行取得 `iso-codes` 的 ISO 639-2 与 639-3 JSON 数据，再用转换器生成 Qt 可读取的输入。例如，在安装了该数据包的 Linux 环境中执行：

```sh
python3 scripts/convert-iso-codes-for-qt.py \
  --iso-639-2 /usr/share/iso-codes/json/iso_639-2.json \
  --iso-639-3 /usr/share/iso-codes/json/iso_639-3.json \
  --output build/qt-iso-codes.tsv
```

转换器只提取语言代码及其映射，不保留名称；若输入是完整代码集，输出仍包含完整代码集。它拒绝无效代码、重复代码和 Qt 查找键冲突，并跳过 ISO 639-2 的本地使用保留范围。工具输出输入及结果的 SHA-256。Qt 6.8.2 与 Ubuntu `iso-codes` 4.16.0-1 的重建对照见 [#138](https://github.com/xiaodaozhi/xdzip/issues/138)；其他版本须自行检查生成结果。`iso-codes` 数据不随本交付提供，其声明许可、来源及生成表的分发条件需由获取者核对；可用替代表不证明它是 Qt 历史输入，也不自动完成对应源码或许可判断。

为复现该对照，可从 [Ubuntu 官方归档的 iso-codes 4.16.0-1 包](https://archive.ubuntu.com/ubuntu/pool/main/i/iso-codes/iso-codes_4.16.0-1_all.deb)自行取得数据。以下示例需要 Linux 的 `curl`、`dpkg-deb` 和 Python 3；它先校验包摘要，再在本地解包和转换，不修改系统安装：

```sh
mkdir -p build && mkdir build/qt-iso-peer &&
curl --fail --location --silent --show-error --connect-timeout 15 --max-time 120 https://archive.ubuntu.com/ubuntu/pool/main/i/iso-codes/iso-codes_4.16.0-1_all.deb -o build/qt-iso-peer/iso-codes.deb &&
python3 -c "import hashlib,pathlib,sys; p=pathlib.Path('build/qt-iso-peer/iso-codes.deb'); h=hashlib.sha256(p.read_bytes()).hexdigest(); sys.exit(0 if h=='cafca0a3a1f07b44fbba53ab7a1f1d9755e6b80cc041173afe2b52e53bed97a7' else 'Package SHA-256 mismatch')" &&
dpkg-deb -x build/qt-iso-peer/iso-codes.deb build/qt-iso-peer/data &&
python3 scripts/convert-iso-codes-for-qt.py \
  --iso-639-2 build/qt-iso-peer/data/usr/share/iso-codes/json/iso_639-2.json \
  --iso-639-3 build/qt-iso-peer/data/usr/share/iso-codes/json/iso_639-3.json \
  --output build/qt-iso-peer/qt-locale-codes.tsv
```

已核验两份输入的 SHA-256 分别为 `fa83810fdb59f9d84b4d58486d5e5e48e807d82a98d6a39ef0ba4fc57c2a9327` 和 `9636ce5266053867627140ce5ada1f9aa897ca07a7501302c1b14b8d1147cdda`，转换输出为 `85457a36e013e053b70dea19d500ebec2ae6bedf5dad9af0035b973c8db6a60a`。Ubuntu 包内 `usr/share/doc/iso-codes/copyright` 声明 LGPL-2.1-or-later；这项包级声明不单独证明 SIL 代码表的历史授权或完整代码集再分发权。项目不将该包或完整转换结果放入源码交付，使用者应分别核对来源、适用条款及后续分发范围。

也可自行取得 [IANA 语言子标签注册表](https://www.iana.org/assignments/language-subtag-registry/language-subtag-registry)和[美国国会图书馆 ISO 639-2 列表](https://loc.gov/standards/iso639-2/ISO-639-2_utf-8.txt)，使用以下命令生成另一份输入。工具只提取代码，不保留名称；它报告未匹配的代码、输入与输出摘要，并拒绝无效或有歧义的记录。下面的 Linux／macOS shell 示例使用新目录，校验本次核验的两个原始文件后才转换：

```sh
(
  set -eu
  mkdir -p build
  mkdir build/qt-iana-loc-inputs
  curl --fail --location --output build/qt-iana-loc-inputs/iana.txt \
    https://www.iana.org/assignments/language-subtag-registry/language-subtag-registry
  curl --fail --location --output build/qt-iana-loc-inputs/loc.txt \
    https://loc.gov/standards/iso639-2/ISO-639-2_utf-8.txt
  python3 -c 'import hashlib,pathlib,sys; p=sys.argv[1:]; sys.exit(0 if all(hashlib.sha256(pathlib.Path(p[i]).read_bytes()).hexdigest()==p[i+1] for i in (0,2)) else "Input SHA-256 mismatch")' \
    build/qt-iana-loc-inputs/iana.txt 755fad43283be7b41ebe3c89ad054b6eaf928f404f9c0edb74799e0eab74beb1 \
    build/qt-iana-loc-inputs/loc.txt 42b71885e4dc885559fda5ad059fd81838cf4782cedb5fd08cc3431f7d067371
  python3 scripts/convert-iana-loc-for-qt.py \
    --iana build/qt-iana-loc-inputs/iana.txt \
    --loc build/qt-iana-loc-inputs/loc.txt \
    --output build/qt-iana-loc-inputs/qt-codes.tsv
)
```

对照所用 IANA 文件日期为 2026-09-17，原始文件 SHA-256 为 `755fad43283be7b41ebe3c89ad054b6eaf928f404f9c0edb74799e0eab74beb1`。2026-11-03 从 LoC 直接取得的 UTF-8 原始文件为 16,010 字节、SHA-256 `42b71885e4dc885559fda5ad059fd81838cf4782cedb5fd08cc3431f7d067371`；其 487 条记录的前三个代码字段与此前网页重建记录逐项一致。转换器的 LF 输出摘要为 `6a75bebfa694258a1a4125fc30512d8ae7994080701377b41306db665ad371c6`。该组合生成 8268 行，现有 Qt 语言的 346 个不同查找键全部一致；使用固定 Qt 与 CLDR 重生八个文件，仅规范化生成日期和换行后全部匹配，修改一项输入的负对照匹配七个。对照中 `bh`、`in`、`iw`、`ji`、`jw`、`mo`、`sh` 七个 IANA 双字母子标签没有 LoC 对应记录，工具会报告它们；其他版本和新增语言须单独核对。上游地址可能更新，摘要不符时须重新核验，不应跳过检查。详情见 [#138](https://github.com/xiaodaozhi/xdzip/issues/138#issuecomment-6588040592)。

[IANA 与 IETF 的声明](https://www.iana.org/help/licensing-terms)将其拥有或有权转授的协议注册表相关权利按 CC0 处理，并保留第三方可能拥有的权利；该声明不自动证明 SIL 原始表的全部权利。项目不在此交付本次对照使用的 2026-09-17 IANA 快照、LoC 原始输入或完整转换结果；前述 CLDR 43.1 子集另含较早的注册表。接收者须分别核对来源、适用条款与后续分发条件。该替代输入不证明 Qt 上游历史输入，也不单独证明对应源码义务已经完成。

这里的独立对照指分别取得文件并运行转换，不表示标准来源独立。[RFC 5646 第 2.2.1 节](https://www.rfc-editor.org/rfc/rfc5646#section-2.2.1)将 ISO 639-2、639-3 和 639-5 列为 IANA 三字母语言子标签的标准来源，因此数值重合不能证明数据独立创作或完整转授权。


输出包含实际固定提交的 xdzip-source.tar、rust-vendor.tar.gz、离线 Cargo 配置、Qt 原始归档、REBUILD.md 和逐文件 SHA-256／大小清单。清单明确为 collected-not-compliance-verified，不能替代 SDK 构建 features／补丁、平台运行库、内嵌资源和许可履约的核对。项目源码重新配置的身份可能为 unknown，应由外部交付清单绑定原提交，不能伪造干净 Git 身份。

`rar-wire-mappings/` 单独保留有限 RAR 线映射的固定参考来源、说明和 UnRAR 7.30 beta1 精确原始许可，全部进入交付文件哈希清单。下载归档名为 `unrarsrc-7.3.1.tar.gz`，不据文件名另推上游发行版本。交付前还核对相关 Rust 源码注释中完整保留的上游许可第 2 段，作为保守通知措施；缺失或变更时停止。RAR crate 的机器可读声明采用保守的 `Apache-2.0 AND UnRAR`；`UnRAR` 是 SPDX 3.28 起可识别的非 OSI 批准标识，旧版工具可能不认识。它不携带官方引擎源码，也不将事实来源重新许可为 Apache-2.0，条款适用范围和实际再分发义务继续按候选复核。

另附固定的 ICU 73.2 完整许可、ICU4C 官方发布源码归档、包含可编辑数据的固定 ICU 源码树、CLDR 43.1 源码子集，以及生成工具使用的 ICU 73.1 源码树，服务于 Linux ICU，并纳入交付总清单。CLDR 子集排除五个 ISO 639-3 文件、`docs/ldml` 下的 UTS #35 标准正文及配套文件、两张文档截图，并替换 963 张网页工具图片；原归档摘要、逐文件排除和替换清单保留在交付记录中。[Unicode 发布的 43.1 版 UTS #35](https://www.unicode.org/reports/tr35/tr35-69/tr35.html)可单独查阅，其使用条件由权利人说明。是否存在可转授的额外授权仍待核实。收集器也拒绝引导源码归档中名称包含 `iso-639-3` 的文件，名称检查不能排除其他形式的数据。收集时校验归档摘要、版本、归档内许可及指定源文件，不解包第三方路径。[ICU 数据重建说明](icu-cldr-rebuild.md)记录工具构建、数据生成和修改范围；上游版本相符不证明供应商二进制由这些材料无补丁构建。当前 Windows 开发包不包含软件 OpenGL 库，不再为该库收集 Mesa／LLVM 材料；历史交付保留原有材料和义务，见[原生运行库来源与通知](native-runtime-materials.md)。

应用动态链接 Qt，提供源码重建入口和兼容库替换说明。[macOS 部署核验](macos-deployment.md)覆盖隔离副本的修改库加载，不声称已从源码重建整套 Qt。实际 Qt 源码构建与产品加载另有独立流程，见[自动验证契约](../../../developers/zh-CN/engineering/candidate-verification.md)。SDK 字节级构建等价性、精确静态链接归属和许可分支仍属于[发行许可复核](https://github.com/xiaodaozhi/xdzip/issues/4)，不重新列为已完成 WP-12 的开发待办。

verify-source-delivery.py 先重新核对交付文件哈希，将自有项目和 vendor 安全解到新的 build/ 目录，使用空 Cargo 缓存、CARGO_NET_OFFLINE=true 和现有固定 Qt SDK 构建真实应用，再执行启动／关闭冒烟。它验证项目与 Rust 交付输入可离线使用，不声称 Qt 已从模块源码重建，不声称字节级可重复构建。生成源码目录的 Git 身份为 unknown，由外部固定提交及交付清单绑定。

源码交付在输出目录内以相对 `vendor` 参数运行 Cargo，并显式指定项目清单；只接受当前受支持的 crates.io 源映射与相对 `vendor` 配置。输出包含绝对目录、路径逃逸或意外源映射时直接失败，避免将构建机路径写入可迁移交付。

对于锁定 vendor 源码中没有常见文件名的独立许可或通知文件的包，交付在 `rust-vendor-notices/` 另存按包索引的固定上游原文。收集器核对包的 Cargo 清单哈希、上游 Git 提交与路径、许可原文 SHA-256 和 Git blob；未知缺口或身份不符时停止。补充目录不改变 Cargo vendor 原文件及校验，许可表达式保留上游原样，不由此批准具体许可分支或既有发行的合规状态。

新增[Qt SDK 对应核验](qt-sdk-correspondence.md)：交付包总是包含固定对应清单和可重放的 AGL 消费端补丁，传入 --qt-prefix 后额外包含该 SDK 的逐文件源码声明核对、构建配置和明确缺口。交付清单递归哈希补丁子目录，重建验证同样复核这些字节。未传 SDK 时不伪造配置核验记录。

交付还保留固定的 Qt Declarative Git 提交对象输入，供离线核对发布提交与归档 `.tag` 树对象的关系；交付清单会哈希该输入。树对象对应不代表源码归档覆盖整个树，或 SDK 与该树构建等价。

实际 Qt 源码构建和加载／兼容替换流程见[自动化收尾契约](../../../developers/zh-CN/engineering/candidate-verification.md)，不再把应用使用现有 SDK 的离线重建误称为 Qt 本身的源码重建。
