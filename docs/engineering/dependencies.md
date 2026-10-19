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

## Windows 来源身份

M2 在 Windows 运行时直接使用精确版本 `winapi-util = 0.1.11` 的 `file::information(&File)` 安全封装，比较卷序列号与文件索引。长度、修改／创建时间无法可靠检测 NTFS 同名替换；标准库对应文件索引接口尚非稳定 API，自行写 Win32 unsafe 会扩大审查范围。因此复用锁文件已有的库，不新增包版本。

来源为 crates.io／[BurntSushi/winapi-util](https://github.com/BurntSushi/winapi-util)，许可选择 MIT（上游 MIT 或 Unlicense 双许可）。Windows 传递依赖为已锁定的 windows-sys 0.61.2、windows-link 0.2.1，均为 MIT 或 Apache-2.0；仅启用上游声明的 Win32 模块，禁用默认特性，无 build script、外部原生构建或网络下载。非 Windows 目标不编译这条运行依赖。上游安全封装借用有效 File 句柄，系统调用失败使旧来源失效；不跨 FFI 传递这个句柄，不将身份检查当作安全写入授权。

版本来源、特性、原生编译行为已检查；Windows GNU 目标类型检查已通过，原生 Windows 回归必须覆盖保留修改时间的同名／同大小替换。上游为小型 Win32 安全封装项目，升级由锁文件和原生回归约束，未做独立维护或安全审计。2026-10-13 的 OSV 精确版本查询未返回这三个包的已知公告，历史响应保留在原执行工件，新查询按[验证方法](../qa/README.md)另记实际响应；这不是无漏洞证明。开发包继续收集所有上游许可证。

## HTML 复核字体与工具

采用官方 Noto Sans SC 2.004-H2 的离线可变子集，替代审阅设备的系统回退字体。固定 Google Fonts 提交、原始及衍生哈希、版权和 OFL-1.1 许可见 [材料清单](../../design/assets/fonts/manifest.json)。HTML 仅加载约128KB WOFF2，含400–750字重；无外部服务、传递运行依赖或原生编译。保留系统字体方案会导致跨平台文字度量不同，因此审阅稿采用随包字体。覆盖已检查，未知生成原稿字体仍不能证明匹配；未实施独立漏洞审计。字体上游持续维护，固定版本需随维护情况复核。

设计检查工具为 Playwright（Apache-2.0）、Pillow（HPND）、FontTools（MIT）和 Brotli（MIT），版本固定于 [工具要求](../../scripts/html-design-requirements.txt)，通过独立 venv 安装；Playwright 下载 Chromium 仅作为 CI／本地检查浏览器，不进入应用或离线设计交付。FontTools 和 Brotli 只负责子集生成与检查，不进入运行代码。

## ZIP AES 创建

精确固定 `zip = 9.0.3`（crates.io、https://github.com/zip-rs/zip2，MIT），仅启用 aes-crypto 和 deflate-flate2；flate2 延续现有 Rust backend，关闭 bzip2、zstd、lzma、ppmd、time、原生 zlib 等默认能力。只有 AES 创建及写后认证校验使用 zip-rs，普通 ZIP 元数据、解压和明文创建继续使用已准入的 rawzip；没有通用 unpack 或文件路径授权。上游提供纯 Write 流包装器，保持现有扩展边界。

AES、cipher、hmac、pbkdf2、sha1、zeroize 等 RustCrypto 依赖、随机源和常量时间比较锁定在 Cargo.lock；这些包采用 MIT 或 Apache-2.0 双许可。SHA-1/PBKDF2 参数来自 WinZip AES 格式，并非另选的用户密码存储方案。禁用原生压缩库与网络下载构建行为，构建脚本只做平台配置；不增加应用进程、运行下载或 subprocess 后端。直接依赖 zeroize 1.8.2 为宿主密码容器归零，Debug 固定脱敏。

rawzip 暂无对应 AES 编码能力；自行实现 AES/PBKDF2 或把真实密码传到外部 CLI 都会扩大安全边界，因此选用受维护的库。准入包含精确特性树、锁文件、许可证、来源、故障测试和独立 7-Zip 互操作。2026-10-14 对 zip、aes、cipher、hmac、pbkdf2、sha1、zeroize、constant_time_eq 和 getrandom 的锁定版本执行 OSV 精确版本查询，未返回公告；原始响应仅保留为本地工件。这不是无漏洞保证，未实施独立密码学安全审计；新增版本须重新检查可达特性和平台构建。

## WP-11 TAR/GZIP

精确固定 [tar 0.4.46](https://docs.rs/tar/0.4.46/tar/)（crates.io、[上游源码](https://github.com/alexcrichton/tar-rs)，MIT 或 Apache-2.0），关闭默认 xattr 特性。仅使用 Header、PaxExtensions 和对宿主 Write 的 Builder::append_data，不调用 Archive::unpack、Entry::unpack、文件路径读取或所有者／权限恢复。读取采用显式有界头与扩展头处理，避免不受宿主约束的扩展分配；编码通过显式 PAX UTF-8 path 及大尺寸字段保证跨平台名称互操作，不依赖 Windows 系统 tar 对 GNU 原始名称的本地编码猜测。

GZIP 复用已锁定的 flate2 1.1.10、Rust backend、miniz_oxide，无新增原生压缩库。MultiGzDecoder 只作为一层过滤器组合到 TAR，压缩输入和解码输出均受取消及字节预算约束，结束时验证所有 member 的 CRC／ISIZE。独立非 TAR 的 GZIP 仍明确拒绝。

新增传递依赖 filetime 0.2.29（MIT 或 Apache-2.0），复用既有 cfg-if、libc；filetime 的系统时间恢复接口不由产品调用。没有新增下载构建脚本、运行网络请求或外部进程格式后端。许可证由既有 collect-notices.py 根据锁文件收集。系统 tar 与 Python tarfile 仅用于测试，不进入应用运行依赖。

2026-10-15 对 tar 0.4.46、filetime 0.2.29、flate2 1.1.10 执行 OSV 精确版本查询，未返回公告；查询原文只保存在 build/review/wp11。此结果不等于无漏洞保证或独立安全审查。双向互操作、坏输入、配额、取消和三平台验证必须实际运行，证据入口见 [WP-11 契约](m4-tar-gzip.md)。


## 实验安装器工具

Windows 构建引入 Inno Setup 6.7.3，仅用于编译自有运行目录的当前用户安装器，来源为 [jrsoftware/issrc 官方发行](https://github.com/jrsoftware/issrc/releases/tag/is-6_7_3)，不可变源码提交、官方二进制／中文语言资源／许可正文的 SHA-256 固定在 [installer-toolchain.json](../../scripts/installer-toolchain.json)。选择其当前用户模式和累计卸载日志；NSIS 与 WiX/MSI 是替代方案，本轮不另维护自制清理数据库或企业部署规则。

许可采用该固定源码中的 Inno Setup License，保留版权／原始来源及引擎界面，不修改编译器；正文随 Windows notices 交付。传递代码使用官方编译器／安装存根提供的 Pascal Script、压缩与平台组件，本轮不重新编译引擎或宣称其全部内嵌组件已独立审查。编译器安装在 CI build/，不进入 Qt 运行目录；安装存根进入最终安装器，其版本与安装器哈希由回执记录，运行目录 SBOM 不冒充安装器存根的完整 SBOM。

仅启用标准文件复制、当前用户注册表、自有快捷方式和互斥量保护，关闭自动关闭／重启应用，无安装时网络下载、自定义原生插件或新 Rust 构建脚本。Linux .deb 使用 Ubuntu 的 dpkg-deb 与系统维护工具，不引入额外应用编解码器。Qt 下载工具工作目录固定在 build/，避免 aqtinstall.log 污染源码身份。

2026-10-16 查询官方仓库公开 security-advisories API 返回空列表，原文保存在 build/review/installer-tool-advisories.json。这不代表无漏洞、NVD 全库检索通过或独立审查通过；[官方版本记录](https://jrsoftware.org/files/is6-whatsnew.htm)用于后续安全维护复核，固定 6.7.3 不代表最新版本或长期支持承诺。哈希校验不代替签名身份与完整引擎许可／安全准入。真实编译、安装、系统打开、更新／回退／卸载和未通过项按[安装契约](installers-associations.md)及实际 CI 记录登记。
