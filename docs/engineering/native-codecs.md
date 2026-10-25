# 隔离格式工作进程契约

7z、CAB 使用官方 7-Zip 源码，RAR 使用 RARLAB 官方 UnRAR 源码。固定来源、版本和 SHA-256 位于 [源码清单](../../scripts/native-codec-sources.json)。ISO/UDF 使用[独立 Rust 后端](independent-rust-codecs.md#iso与udf只读规格)，通过薄 CXX 边界复用同一隔离工作进程。ZIP、TAR、TAR.GZ 使用进程内 Rust 后端。产品不从用户 PATH 查找替代工具，也不运行外部命令行解包。

## 构建与来源

统一准备入口运行 `scripts/prepare-native-codecs.py`，只将已核验源码解包到 build 或 .cache 的专用目录。下载及解包大小、成员数量和成员路径均有上限，拒绝链接、设备、绝对路径、父目录逃逸及大小写重复成员。离线构建使用已固定归档。

7-Zip 构建清单只包含 7z、Cab 两个容器处理器、其压缩算法和通用设施，不编译 ISO、UDF、ZIP、TAR、GZIP、RAR 等容器处理器，不启用外部格式或编解码器自动加载。源文件去重，处理器注册集合须与白名单完全一致。RAR 的格式识别和解码只由 UnRAR 执行。Deflate、AES、CRC 等算法设施可能与现有 Rust 依赖存在实现重叠，不因此增加同一容器的第二条产品路由。

源码修改以可审阅的桥接文件和精确匹配的构建变换维护，不把修改后的源码冒充上游原始归档。构建必须核验所需声明，声明不匹配时停止。来源对应材料随实际部署版本交付，不能只交付下载链接。

## 权力与生命周期

Rust 核心持有来源授权、分卷身份、目标授权、暂存和提交能力。官方后端和 Rust 光盘后端在独立工作进程运行，通过有界协议读取授权流及输出当前批准条目的数据，不获得用户目标路径或任意文件访问接口。UnRAR 原生文件访问必须经宿主适配，不能用临时解包再复制替代既有安全提交流程。

工作进程不接收命令行密码，不输出归档提供的任意诊断文本，也不继承应用凭据和代理环境。错误映射为稳定错误码，打开加密目录的密码只保留在当前会话及其关联任务的归零容器中，不跨启动持久化；关闭会话和任务终态分别释放各自容器。取消停止读取与输出，超时后终止工作进程并等待回收；进程失败不得产生成功终态。进程隔离不等同于操作系统沙箱，不宣称能阻止已被攻陷的进程访问所有同用户资源。

7z 创建通过 EncodeOutput::write_at 将有界偏移与数据块写入宿主私有暂存，原生进程只保存流位置，不在内存中累积整个输出归档。宿主分别累计实际写入量和归档长度高水位，头部回写不增加分卷内容长度；偏移、块大小、累计输出量和取消均在主机端重新检查。未提供随机写能力的输出明确拒绝。

任务延续宿主条目、路径、元数据和实际字节预算，默认 native 预算为 1 GiB。Windows 使用 Job Object 提交内存限制，Linux 使用绝对虚拟地址空间限制，macOS 使用启动时虚拟地址空间基线加预算，排除 dyld 预留；Unix 地址空间限制不等同于 RSS 计量。字典及固实工作区在分配前准入，无法在预算内完成的操作明确失败，不静默降级为无界磁盘暂存。重试和固实跳过均累计实际工作量。

## 能力边界

7z 的浏览、解压、完整性测试和创建分别声明，创建使用 LZMA2，默认正常级别、非固实、不加密、不分卷。固实、内容加密、文件名加密和编号分卷作为独立能力验证。文件名加密必须设置密码。文件名加密包在元数据打开阶段请求密码，未完成扫描不能发布部分索引。

未加密、非分卷且所有条目可安全保留的 7z 包可复用添加、替换和包内移动。固实内容通过受限重建处理。加密、分卷、链接和不支持特殊记录继续拒绝修改，来源重新检查及完整验证通过后才允许同卷原子替换。独立删除和重命名入口不属于本次扩展。

RAR 只读，区分旧格式、RAR5 以及 RAR 7 新算法，密码、头部加密、固实和分卷分别验证。RAR 7 格式版本 1 已验证 Store 内容与超预算字典拒绝，LZ 压缩流仍缺少独立实样互操作证据；官方 RAR 7.23 生成的固定压缩样本明确使用 RAR5 兼容算法 0，不将其标注为算法 1 的验证。不提供创建、修改、修复或密码恢复。UnRAR 是附带用途限制的源码可用软件，不标记为 Apache-2.0，也不因 Rust 封装改变原始条款。

CAB 支持 Store、MSZip、Quantum 和 LZX，单卷与同目录分卷均只读。分卷先由 Rust 根据头部关系验证和冻结授权集合，再向工作进程传递原始卷名及各卷边界。缺卷和不匹配卷不能发布部分索引，详见[分卷契约](archive-opening.md)。CAB 原始名字和 UTF-8 声明保留，旧编码采用[统一文件名解析](filename-encoding.md)，不调用宿主代码页猜测输出路径。

ISO9660、Joliet、Rock Ridge 和 UDF 的解析由 `xdzip-codec-optical` 执行，不链接官方 Iso/Udf 处理器。混合镜像优先 UDF 命名空间，已识别的 UDF 损坏不静默回退 ISO。按内容识别，不依赖扩展名授权。ISO/UDF 仅浏览、解压、临时打开、导出和测试，不挂载磁盘、不执行启动镜像、不提供创建或更新。启动镜像记录仅作为可读取文件，链接及特殊对象保持不可输出。镜像测试检查结构、范围及实际内容可读取性，不声称普通文件具有内容校验和保证。分区映射和不支持扩展的明确边界见[自研规格](independent-rust-codecs.md#iso与udf只读规格)。

English: The pinned official 7-Zip build registers exactly 7z and Cab. UnRAR handles RAR. ISO9660, Joliet, Rock Ridge and UDF use the independent Rust optical backend through a thin CXX boundary inside the existing isolated worker; official Iso/Udf handlers are excluded from the build. ZIP/TAR/GZIP remain in-process Rust implementations. CAB supports Store, MSZip, Quantum and LZX and preserves raw names; only Rust-authorized same-directory volumes are readable. Hybrid images select UDF; recognized UDF corruption does not trigger an ISO fallback. CAB and disc images cannot be created or modified. Image checks establish structure and readability, not per-file checksum authenticity. No image mounting or boot execution is provided. Optical extension boundaries are specified in the independent-codec contract.

## 验证与交付

实际支持声明须由官方工具互操作、已知输出字节、损坏和资源限制样本支撑。仅识别魔数、调用返回成功或使用同一后端自读自写不足以独立验证互操作。三平台核心和 Qt 检查、部署验证、原生视觉及桌面拖放分别记录，阶段证据保留在 issue 和本地或 CI 工件。

互操作检查使用固定 26.05 官方参考工具，独立于产品工作进程。Windows 从同一官方发布下载并核对 7zr.exe 摘要，再从已核验的 7z2605-extra.7z 仅提取 x64/7za.exe，并复核该成员的独立 SHA-256。此流程不依赖系统 tar 的 LZMA 支持，不运行安装器，也不从 PATH 借用未固定工具。Linux 与 macOS 从已核验 tar.xz 仅读取指定 7zz 成员。下载、缓存和提取结果任一摘要不符均停止；这些参考文件只保留于 build/ 测试目录，不进入产品部署。

开发包维持实验性。组件许可、对应源码和重链接要求按实际构建单独复核，本功能完成不自动批准正式发行。
