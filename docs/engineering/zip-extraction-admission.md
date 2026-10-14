# ZIP 解压与输出依赖准入

日期：2026-10-13。用于 M3 实验开发包；正式发行的独立安全评审尚未完成。精确版本、来源、许可证、传递依赖、启用特性、build script 与 crates.io 校验和见 [锁定材料](zip-extraction-dependencies.json)和根目录 Cargo.lock。

| 依赖 | 用途和决定 |
| --- | --- |
| flate2 1.1.10 | 仅启用 rust_backend，以 miniz_oxide 的纯 Rust 流式 Deflate 解码；关闭 C zlib 与通用解包路径。相比整体 ZIP unpack，借用宿主输入与输出 sink 能保留输出授权和完整校验边界。固定缓冲区，并显式检查 StreamEnd 及压缩字节消费量。 |
| cap-std / cap-fs-ext 4.0.3 | 持有目录能力、nofollow 子目录打开、排他暂存和相对提交操作。相比字符串拼接与 canonicalize 后创建，组件权限和身份复核在宿主执行；不调用通用 unpack。cap-primitives 等传递包一并锁定。 |
| getrandom 0.4.2 | 使用操作系统随机源生成私有工作目录 nonce；不使用时间戳、伪随机或归档名字。关闭默认特性。 |
| rustix 1.1.5 | POSIX 有效用户身份、传递文件系统原语，复用已锁版本；关闭默认特性，仅启用 std/process，加上传递所需 fs。 |
| windows-sys 0.61.2 | 查询 Windows 对象所有者和 DACL、检查 SID；隔离手写 unsafe，借用有效句柄、DWORD 对齐缓冲区和 RAII LocalFree。相比只检测只读属性，DACL 能检查私有输出前提；新增 FFI 必须在正式发行前独立评审。 |
| unicode-normalization 0.1.25 | 为任务名称别名生成 NFD＋小写 admission key；原始输出拼写保留，不把此键当作文件系统完整 casefold。相比只用 ASCII 小写能覆盖常见 macOS Unicode 等价名称。 |

已阅读这些包的 Cargo 配置、许可和涉及的 API／源码。flate2 上游持续维护、cap 系列由 Bytecode Alliance 维护、getrandom 和 rustix 为系统接口项目、Unicode normalization 为通用 Unicode 库；升级仍须锁版本并重跑原生边界回归，不以维护主体代替安全审计。新增包无运行时网络请求；含 build script 的包只探测编译器／目标配置或选择 Rust 系统调用后端，没有下载外部代码或编译 C 解码器。MIT／Apache-2.0／Zlib 等许可通知通过开发包 notices 收集完整上游原文；实际许可表达式以锁定材料为准。

[OSV 精确版本查询（本地／CI 工件）](../qa/README.md)记录实际响应；查询失败不能解释成未发现公告。该检查不是完整 cargo-audit 或独立审查。既有 cxx 公告与版本选择仍见 [M2 依赖记录](dependencies.md)。

rawzip 保持 0.5.2；M2 元数据准入不自动批准解码。M3 适配器独立验证本地头、流结束、CRC 与 32／64-bit 数据描述符。rawzip 自带的数据描述符宽度推断不覆盖某些 Python force_zip64 流式产物，适配器按本地 ZIP64 哨兵验证，不复制或修改第三方源码；独立 Python 夹具同时覆盖合法混合宽度和篡改大小。

锁定材料列出全部跨目标 Cargo 解析依赖，其中 getrandom 的 WASI／WIT／WebAssembly 工具包只适用于未支持的目标，不进入 macOS、Windows、Linux 桌面编译或运行。Cargo 全目标清单不能解读为桌面运行依赖或独立安全审计范围。

开发包 notices 按 `cargo metadata --filter-platform <rustc host>` 的实际解析节点收集当前平台的工作区和构建工具许可，并登记 target.json；不会把未编译的 WASI／UEFI 包当作桌面发行依赖。许可证缺失仍使打包失败，不能用 Cargo 许可名称代替正文。r-efi 6.0.0 的完整 MIT 授权和版权位于 AUTHORS，收集器仅接受该版本与固定 SHA-256 的原文例外。
