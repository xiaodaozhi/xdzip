# 依赖与格式后端准入

首次添加任何依赖，记录用途、替代方案、版本、来源、许可证、传递依赖、特性、原生构建行为、已知漏洞和维护情况。仅有 crate 名称不算准入完成。当前不列虚构的已审计版本。

| 候选方向 | 必须验证 |
| --- | --- |
| ZIP Rust 实现 | Store/Deflate、ZIP64、CRC、重复项、名称原始字节、取消和宿主输出 sink |
| TAR + GZIP Rust 实现 | 流式索引、PAX/GNU 元数据、链接/设备项拒绝、未知大小、损坏流 |
| cxx | 所有权、生成器一致性、跨平台静态核心链接、异常不越界 |
| Qt 模块 | 实际运行与传递依赖、可替换部署、许可证和安全更新渠道 |
| 图标集合 | 明确再分发许可、统一线宽、可访问性与资产清单 |

验证顺序：最小互操作 → 恶意/截断输入 → 取消/资源上限 → 许可/SBOM → 干净构建 → 记录决定。不能通过调用库的通用 unpack 绕过宿主路径保护。

应用型 Rust workspace 提交 Cargo.lock。新增原生库和网络下载 build script 需专门审查；禁用未使用 features。弃用或漏洞无修复时按实际可达性决定升级、替换或关闭能力，记录原因。

## 更新元数据与发行者验签

`xdzip-update` 是不依赖 Qt 的清单、渠道和签名验证核心，产品通过独立 cxx 边界调用；发布工具和安装助手使用相同规则。选择 `ed25519-dalek = 2.2.0` 的安全 `VerifyingKey::verify_strict`、弱公钥拒绝和精确原始字节验签，不自行实现签名运算，不把 TLS 或 SHA-256 当作发行者签名。默认启用上游 fast、std、zeroize，未启用 batch、hazmat、legacy_compatibility、rand_core 或 PKCS#8；发布工具仅导入由所有者配置的 32 字节 seed，不生成生产私钥。

| 直接或关键传递依赖 | 精确版本 | 来源、许可与用途 |
| --- | --- | --- |
| ed25519-dalek、curve25519-dalek | 2.2.0、4.1.3 | [dalek-cryptography](https://github.com/dalek-cryptography/curve25519-dalek)，BSD-3-Clause，Ed25519 与曲线运算 |
| base64ct | 1.8.3，alloc | [RustCrypto/formats](https://github.com/RustCrypto/formats)，Apache-2.0 或 MIT，Sparkle 兼容签名编码 |
| sha2 | 0.10.9 | [RustCrypto/hashes](https://github.com/RustCrypto/hashes)，MIT 或 Apache-2.0，安装包摘要 |
| semver | 1.0.28 | [dtolnay/semver](https://github.com/dtolnay/semver)，MIT 或 Apache-2.0，有界产品版本解析 |
| subtle | 2.6.1 | [dalek-cryptography/subtle](https://github.com/dalek-cryptography/subtle)，BSD-3-Clause，上游常量时间运算支持 |
| zeroize | 1.9.1 | [RustCrypto/utils](https://github.com/RustCrypto/utils)，Apache-2.0 或 MIT，发布进程私钥临时容器归零 |

serde 1.0.229 和 serde_json 1.0.154 复用锁定版本。其余传递依赖以 Cargo.lock 和实际特性树为准，包括 digest、block-buffer、crypto-common、generic-array、typenum、cpufeatures 与 derive 宏。curve25519-dalek 的构建脚本仅检测本地编译器能力，运行时可选择上游的 CPU 特定实现；不下载外部源码、不新增 C/C++ 密码学库或 OpenSSL 运行依赖。已有通知和 vendor 收集入口根据 Cargo metadata 纳入这些包，实际包的归属与许可仍由发行材料核验。

保留系统密码学 API 的替代方案会分裂三平台验证规则，外部 OpenSSL CLI 会扩大运行依赖及参数边界，因此采用同一 Rust 核心。已知历史 [RUSTSEC-2022-0093](https://rustsec.org/advisories/RUSTSEC-2022-0093.html) 的旧版 Keypair API 不用于当前实现；[RUSTSEC-2024-0344](https://rustsec.org/advisories/RUSTSEC-2024-0344.html) 要求的 curve25519-dalek 修复边界为 4.1.3，锁定版本满足该边界。这两项核对不是全部依赖无漏洞的证明，后续升级继续核对维护与公告。

回归同时验证精确字节变更、错误签名、未知／重复／弱公钥、空生产信任集、渠道回退、版本与 build 绑定、六包完整性和签名 feed。工具临时测试密钥只存在于测试代码和隔离工件，禁止写入产品信任资源。原生安装和跨平台结果分别记录，不以单平台核心测试替代安装验收。

## Windows 来源身份

M2 在 Windows 运行时直接使用精确版本 `winapi-util = 0.1.11` 的 `file::information(&File)` 安全封装，比较卷序列号与文件索引。长度、修改／创建时间无法可靠检测 NTFS 同名替换；标准库对应文件索引接口尚非稳定 API，自行写 Win32 unsafe 会扩大审查范围。因此复用锁文件已有的库，不新增包版本。

来源为 crates.io／[BurntSushi/winapi-util](https://github.com/BurntSushi/winapi-util)，许可选择 MIT（上游 MIT 或 Unlicense 双许可）。Windows 传递依赖为已锁定的 windows-sys 0.61.2、windows-link 0.2.1，均为 MIT 或 Apache-2.0；仅启用上游声明的 Win32 模块，禁用默认特性，无 build script、外部原生构建或网络下载。非 Windows 目标不编译这条运行依赖。上游安全封装借用有效 File 句柄，系统调用失败使旧来源失效；不跨 FFI 传递这个句柄，不将身份检查当作安全写入授权。

版本来源、特性、原生编译行为已检查；Windows GNU 目标类型检查已通过，原生 Windows 回归必须覆盖保留修改时间的同名／同大小替换。上游为小型 Win32 安全封装项目，升级由锁文件和原生回归约束。2026-10-13 的 OSV 精确版本查询未返回这三个包的已知公告，历史响应保留在原执行工件，新查询按[验证方法](../qa/README.md)另记实际响应；这不是无漏洞证明。开发包继续收集所有上游许可证。

## Rust 格式后端与共享工作进程桥接

`xdzip-codec-7z` 使用项目 API/domain、无第三方生产依赖的共享压缩算法及通用密码学原语。AES 0.9.3 启用 zeroize，getrandom 0.4.2 与 zeroize 1.9.1 复用 ZIP 版本。可清除敏感状态的 SHA-256 提取到格式无关的 `xdzip-crypto`，在已准入 sha2 0.10.9 的 compress 原语上管理自有增量状态，不在各格式复制包装。共享 crate 还依据公开数学规范实现 HMAC-SHA256、BLAKE2s/BLAKE2sp，来源及边界见其 [README](../../crates/xdzip-crypto/README.md)，不提供归档或发布权限，不升级第三方密码学依赖版本。格式属性、CBC 组合、KDF、创建及重写均为独立实现，不由这些基础库提供归档处理。来源、分析角色及许可区分见[7z 依据](sevenzip-format-sources.md)。

`xdzip-codec-optical` 的格式逻辑只使用标准库及本项目 codec API/domain，不引入 ISO/UDF 或通用归档库。规范、范围与错误边界见[独立实现契约](independent-rust-codecs.md#iso与udf只读规格)。`xdzip-codec-bridge` 复用已锁定的 cxx/cxx-build 版本和生成流程，不新增外部包版本、下载构建脚本或格式算法库。替代的进程内路由会失去既有光盘任务的独立进程治理，因此保留既有 worker 和受控流协议。

`xdzip-codec-cab` 只依赖本项目 API、domain 与无第三方生产依赖的 `xdzip-compression`。MSZIP 复用 ZIP/GZIP 的 Deflate，LZX 和 Quantum 为独立算法模块，不引入额外 Deflate 或通用归档库。7z、CAB 与光盘后端共享 `xdzip-codec-bridge` 及宿主授权流，创建仅增加受控输入与随机暂存回调，不复制通用边界。产品固定使用 Rust，官方 Cab 与其专用 LZX/Quantum 不进入构建；资源及格式边界见[自研契约](independent-rust-codecs.md#cab只读规格)，参考逻辑的分析范围与许可见[CAB 来源](cab-format-sources.md)。

`xdzip-codec-rar` 复用同一 codec API、共享压缩算法与密码原语，不引入 RAR 或其他归档格式库。PPMd H 统计模型由 7z 与 RAR 共用，格式各自定义 range、重置与终止；旧密码的特殊兼容组合保留在 RAR 模块。系统日历及 OEM/ANSI/本地编码由薄工作进程回调提供，格式字段、密码截断与校验仍由 Rust 处理，Qt 不进入格式逻辑。

生产依赖政策同时检查格式 crate 与桥接 crate 的普通/构建依赖闭包。官方 7-Zip 与 UnRAR 的容器、算法和通用设施全部退出产品 CMake 清单，产品源码准备清单为空，没有失败回退。历史交付的许可义务不因当前退出而消失。独立 pycdlib、Microsoft IMAPI、mkudffs/udfclient 夹具、官方参考 worker 和规范向量仅用于测试，来源和工具许可与产品分别管理，不进入应用运行依赖。项目自有内容的小型固定输入及生成脚本纳入测试目录，展开的像和执行证据只留在 build/。

## HTML 复核字体与工具

采用官方 Noto Sans SC 2.004-H2 的离线可变子集，替代审阅设备的系统回退字体。固定 Google Fonts 提交、原始及衍生哈希、版权和 OFL-1.1 许可见 [材料清单](../../design/assets/fonts/manifest.json)。HTML 仅加载约128KB WOFF2，含400–750字重；无外部服务、传递运行依赖或原生编译。保留系统字体方案会导致跨平台文字度量不同，因此审阅稿采用随包字体。覆盖已检查，未知生成原稿字体仍不能证明匹配。字体上游持续维护，固定版本需随维护情况复核。

设计检查工具为 Playwright（Apache-2.0）、Pillow（HPND）、FontTools（MIT）和 Brotli（MIT），版本固定于 [工具要求](../../scripts/html-design-requirements.txt)，通过独立 venv 安装；Playwright 下载 Chromium 仅作为 CI／本地检查浏览器，不进入应用或离线设计交付。FontTools 和 Brotli 只负责子集生成与检查，不进入运行代码。

## 自研 ZIP 与通用密码学基础库

ZIP 容器、流式创建、普通重写与 AES 封装采用[独立 Rust 实现](independent-rust-codecs.md)，Deflate/CRC-32 复用 `xdzip-compression`。`rawzip = 0.5.2`、`zip = 9.0.3`、`flate2 = 1.1.10` 仅为开发测试参照，不进入 ZIP 生产或构建依赖闭包，不调用通用 unpack 或授权任意文件路径。历史上使用它们的交付继续保留原有许可义务。

密码学基础原语复用原有传递依赖的精确版本，提升为直接依赖：RustCrypto `aes = 0.9.3`、`hmac = 0.13.0`、`sha1 = 0.11.0`、`pbkdf2 = 0.13.0`，以及 `getrandom = 0.4.2`、`constant_time_eq = 0.6.1`。PBKDF2 只启用 hmac，SHA1 关闭默认 alloc/oid，只启用 zeroize；AES/HMAC 同时启用 zeroize。没有新增格式、原生压缩库或运行下载。CTR、随机盐封装、派生结果分割、密码验证值和认证 EOF 由项目实现，不能借通用基础库重新引回 ZIP 解析器。

这些包及 cipher/digest 等传递依赖锁定在 Cargo.lock，来源为 crates.io 与各自上游。AES/HMAC/SHA1/PBKDF2/zeroize/getrandom 采用 MIT 或 Apache-2.0，constant_time_eq 采用 CC0-1.0 或 MIT-0 或 Apache-2.0，以实际锁定包原文为准。SHA-1/PBKDF2 参数来自 WinZip AES 格式，不用于另选的用户密码存储方案。构建脚本只做平台配置，没有网络下载或 C 解码器编译。

`zeroize` 统一精确升级为 1.9.1，以满足 AES 0.9.3 的密钥清理特性所需版本；所有宿主、原生适配、更新和探针密码容器保持同一版本，避免两套同主版本依赖。秘密 Debug 继续脱敏，派生缓冲与 expanded key 采用基础库支持的归零生命周期，不声称所有历史内存副本均可擦除。准入须包含锁文件、许可、实际特性树、故障测试及独立 7-Zip 互操作；漏洞查询原文只保留为构建工件，成功查询未返回公告不等于无漏洞保证。

## WP-11 TAR/GZIP

精确固定 [tar 0.4.46](https://docs.rs/tar/0.4.46/tar/)（crates.io、[上游源码](https://github.com/alexcrichton/tar-rs)，MIT 或 Apache-2.0），关闭默认 xattr 特性，仅用作开发测试的独立读写参照。产品 TAR 头、PAX、创建与重写使用[独立 Rust 记录层](independent-rust-codecs.md)，不调用第三方 TAR 解析或 Builder，不调用通用 unpack 或所有者／权限恢复。生产依赖闭包禁止 tar 和 filetime 回流，测试及历史交付的许可证仍保留。编码显式写 PAX UTF-8 path 及超范围数字，保证跨平台名称互操作。

GZIP 使用无第三方生产依赖的 `xdzip-compression` 独立实现，包括 RFC 1951 Deflate、RFC 1952 封装和 CRC-32。压缩输入和解码输出均受宿主取消及字节预算约束，结束时验证所有 member 的 CRC／ISIZE。flate2 1.1.10 仅用于开发测试参照，ZIP 与 GZIP 使用相同的自研 Deflate 算法。独立非 TAR 的 GZIP 仍明确拒绝。

tar 的测试传递依赖含 filetime 0.2.29（MIT 或 Apache-2.0），复用既有 cfg-if、libc；filetime 不进入该 codec 的产品依赖闭包。没有新增下载构建脚本、运行网络请求或外部进程格式后端。许可证由既有 collect-notices.py 根据锁文件收集，锁文件存在测试包不表示产品链接这些包。系统 tar 与 Python tarfile 仅用于测试，不进入应用运行依赖。

2026-10-15 对 tar 0.4.46、filetime 0.2.29、flate2 1.1.10 执行 OSV 精确版本查询，未返回公告；查询原文只保存在 build/review/wp11。此结果不等于无漏洞保证。双向互操作、坏输入、配额、取消和三平台验证必须实际运行，证据入口见 [WP-11 契约](m4-tar-gzip.md)。


## macOS 更新提供者

Mac 更新集成锁定 Sparkle 2.10.0，SDK、源码归档及完整许可正文的哈希统一见
[依赖配置](../../scripts/sparkle-dependency.json)。它只负责应用更新，不参与归档解码。
官方 SDK 原始许可包含 MIT、BSD-2-Clause 和 Zlib 授权正文，随 Mac notices 保留；
对应源码交付同时保留锁定源码与 SDK 原始归档，支持离线准备和动态框架重链接。
SDK 与实际部署包的字节或 arm64 分片对照分别记录，不宣称已重建上游二进制。

普通 Mac CI 外部链接 SDK，不部署框架。专用打包在临时副本嵌入框架，删除非沙盒应用
未启用的 XPC 服务，保留必要安装助手。裁剪、重定位后按助手、framework、应用顺序
做 ad-hoc 完整性签名，助手保留所需 Hardened Runtime 和专属 entitlements，禁止套用
通用原生库的重签命令。部署归属、SBOM、许可及源码材料随同版本记录。

## 实验安装器工具

Windows 构建引入 Inno Setup 6.7.3，仅用于编译自有运行目录的当前用户安装器，来源为 [jrsoftware/issrc 官方发行](https://github.com/jrsoftware/issrc/releases/tag/is-6_7_3)，不可变源码提交、官方二进制／中文语言资源／许可正文的 SHA-256 固定在 [installer-toolchain.json](../../scripts/installer-toolchain.json)。选择其当前用户模式和累计卸载日志；NSIS 与 WiX/MSI 是替代方案，本轮不另维护自制清理数据库或企业部署规则。

许可采用该固定源码中的 Inno Setup License，保留版权／原始来源及引擎界面，不修改编译器；正文随 Windows notices 交付。传递代码使用官方编译器／安装存根提供的 Pascal Script、压缩与平台组件，本轮不重新编译引擎，内嵌组件的来源与许可按实际交付范围核对。编译器安装在 CI build/，不进入 Qt 运行目录；安装存根进入最终安装器，其版本与安装器哈希由回执记录，运行目录 SBOM 不冒充安装器存根的完整 SBOM。

仅启用标准文件复制、当前用户注册表、自有快捷方式和互斥量保护，关闭自动关闭／重启应用，无安装时网络下载、自定义原生插件或新 Rust 构建脚本。Linux .deb 使用 Ubuntu 的 dpkg-deb 与系统维护工具，不引入额外应用编解码器。Qt 下载工具工作目录固定在 build/，避免 aqtinstall.log 污染源码身份。

2026-10-16 查询官方仓库公开 security-advisories API 返回空列表，原文保存在 build/review/installer-tool-advisories.json。这不代表无漏洞或 NVD 全库检索通过；[官方版本记录](https://jrsoftware.org/files/is6-whatsnew.htm)用于后续安全维护复核，固定 6.7.3 不代表最新版本或长期支持承诺。哈希校验不代替签名身份与完整引擎许可／安全准入。真实编译、安装、系统打开、更新／回退／卸载和未通过项按[安装契约](installers-associations.md)及实际 CI 记录登记。

## 文件名旧编码检测与解码

精确固定 [chardetng 1.0.0](https://docs.rs/chardetng/1.0.0/chardetng/) 和 [encoding_rs 0.8.42](https://docs.rs/encoding_rs/0.8.42/encoding_rs/)，来源均为 crates.io 和 hsivonen 上游。前者采用 MIT 或 Apache-2.0，后者的代码采用 MIT 或 Apache-2.0，WHATWG 映射数据额外采用 BSD-3-Clause，保留 LICENSE-WHATWG 与 COPYRIGHT。通知收集器包含 COPYRIGHT，不只收集许可全文。

关闭两者默认特性，encoding_rs 只直接启用 alloc，禁用检测器的 multithreading/rayon。锁定的新传递包为 core_detect 1.0.0、multiversion_no_op 1.0.0、scopeguard 1.2.0、simdutf8 0.1.5，均为 MIT 或 Apache-2.0；cfg-if 1.0.5、memchr 2.8.3 和构建宏 rustversion 1.0.23 复用锁定版本。core_detect 提供本地 CPU 能力检测，simdutf8 为上游 SIMD UTF-8 验证，multiversion_no_op 是编译期空操作宏；没有新 C/C++ 原生库、网络构建脚本或运行下载。encoding_rs 构建脚本只按编译器版本选择本地配置。CPU 特定 unsafe 实现留在受维护上游，宿主不新增 unsafe/FFI。

选择同一 Rust 检测器避免 Qt、系统 iconv 和 Windows 代码页分裂规则；自行维护 CJK 统计模型或调用外部解压工具都会扩大实现与部署边界。chardetng 上游标记为被动维护，版本与模型变化需重跑独立文件名样本。ZIP CP437 是固定单字节映射，依据[Unicode CP437 表](https://www.unicode.org/Public/MAPPINGS/VENDORS/MICSFT/PC/CP437.TXT)，不引入平台 API；测试原始字节另由 Python 标准库生成。检测是启发式结果，不作为路径授权或内容校验证据。

2026-10-24 对六个新增锁定包执行 OSV 精确版本查询，响应未返回公告，原始响应只保存在本地工件。这是定向核对，不是无漏洞保证。准入同时检查实际 Cargo features、来源及许可全文，回归范围与资源和失败规则见[文件名编码契约](filename-encoding.md)；三平台构建、内容互操作和实际结果在对应 issue/CI 登记。
