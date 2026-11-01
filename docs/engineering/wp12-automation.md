# 候选自动验证与证据汇总契约

自动验证与外部准入由[门槛清单](../../scripts/wp12-gates.json)分别登记。脚本检验候选绑定和证据完整性，完成自动项不代表正式发行获批。工作包进度与验收状态在 [Issues](https://github.com/xiaodaozhi/xdzip/issues)登记；历史 WP-12 完成记录见 [#5](https://github.com/xiaodaozhi/xdzip/issues/5)。

## Qt 源码与兼容替换

[固定来源清单](../../scripts/corresponding-sources.json)包含 QtBase、QtDeclarative、QtSvg、QtTranslations，以及构建所需的 ShaderTools、QtTools，另包含固定 Rust 标准库源码组件。归档和源码补丁均有 SHA-256；源树恢复前重新核验普通文件，只接受原文或记录的补丁，不把未知修改带入下一次构建。

```sh
python3 scripts/build-qt-source.py --output build/wp12 --resume \
  --apple-sdk /Library/Developer/CommandLineTools/SDKs/MacOSX26.5.sdk --parallel 6
python3 scripts/verify-qt-source-product.py --qt-build build/wp12 \
  --output build/wp12/source-product
```

源码构建安装在隔离 build/ 目录，不替换系统或既有 SDK。QtTools 保留 Linguist 和 lrelease，显式关闭可选的 Clang 解析器及 WrapLibClang 查找，避免 runner 中不完整的 LLVM 开发包配置进入构建。兼容补丁显式包含 ARM 指令头，并在 QtCore 增加只读证明导出；不改变现有公开函数行为。实际应用通过进程内已加载的 QtCore 查找该固定 C ABI 标记，缺失或不符直接失败。完整产品编译、六组 CTest、十万条浏览／搜索／排序／取消一起验证，不以字符串出现在文件中充当加载证明。

源码构建要求 Qt Network 的 SSL 功能：Linux 显式启用系统 OpenSSL 并要求其开发库，Windows 和 macOS 使用各自的系统 TLS 后端。安装后检查 Qt Network 生成配置，SSL 缺失或禁用直接失败；配置检查不代替实际网络更新验证。

English: The source build requires Qt Network SSL. Linux explicitly enables the system OpenSSL development dependency; Windows and macOS use their native TLS backends. The installed Qt Network configuration must show SSL enabled, which is not a substitute for a live update test.

当前本地 Command Line Tools 不提供上游参考 Xcode 版本，SDK 27 的 libc++ 与 Qt 6.8.2 的旧迭代器有编译不兼容；本地可显式使用已有 SDK 26.5。脚本记录实际 SDK 设置哈希、配置和编译器，不宣称该组合就是 Qt 官方支持矩阵或原预编译 SDK 的字节级复现。官方构建说明见 [macOS 源码构建](https://doc.qt.io/qt-6.8/macos-building.html)与 [Qt 模块构建](https://wiki.qt.io/Building_Qt_6_from_Git)。

[独立源码验证工作流](../../.github/workflows/qt-source.yml)仅由可信 main 手动分派，运行三平台源码构建及真实产品验证。Windows/Linux 可复用按 runner 镜像、配置和构建脚本指纹区分的依赖缓存，失败后的 Windows/Linux 局部编译状态也可缓存，缓存不表示验证通过，源树和配置仍重新验证，真实产品验证重新执行；不从外部 PR 取缓存。Mac 只上传有界日志与 JSON，不缓存／部署／签名／压缩／上传应用或 SDK。

本地额外执行 scripts/verify-qt-replacement.py：从开发包复制隔离应用，只替换源码构建的 QtCore，必要时重写开发 RPATH 并做 ad-hoc 完整性签名；核验递归导入无需 SDK，再从真实产品进程取得源码标记，完成十万条交互和 32 MiB 真实解压及独立字节检查。它区别于原有只修改 Mach-O 身份的替换试验，仍不保证任意 ABI 变化兼容。

Windows 和 Linux 的可信 main 源码工作流另在隔离副本核验固定 alpha.8 便携包：原始包与源码构建 QtCore 均按 SHA-256 固定，原库必须拒绝源码标记，替换后由原应用进程确认加载标记，完成十万条浏览与取消及 32 MiB 解压的独立字节比较。Windows 使用包内原生平台插件，Linux 使用包内离屏插件；只上传记录，不修改已发布资产或系统安装，不以自动验证代替真人验收，也不证明任意 ABI 兼容或原 SDK 可复现。

English: On trusted main, Windows and Linux additionally verify the pinned alpha.8 portable package in an isolated copy. The original QtCore must reject the source marker; after replacement the released app must load it, browse and cancel a 100,000-entry fixture, and extract 32 MiB with independently checked bytes. Only records are uploaded. This is not manual desktop acceptance, arbitrary ABI compatibility or proof of reproducible upstream SDK binaries.

本地 Apple Silicon Mac 可另对固定 alpha.8 原始 app.zip 执行 `scripts/verify-alpha8-macos-qt-replacement.py --qt-build build/wp12 --output build/alpha8-mac-qt-replacement`。脚本先核验原包和包内链接，在临时副本中替换已核验的源码 QtCore、执行必要的本地 ad-hoc 完整性签名，检查原应用代码段及依赖、进程内源码标记、浏览取消与真实解压，并清理本次临时应用注册和副本。它拒绝 CI 环境，不改变原安装或发布附件；受控原生验证不等于人工验收、Developer ID 签名或公证。

English: A local Apple Silicon Mac can run `scripts/verify-alpha8-macos-qt-replacement.py` against the pinned published app.zip and a source-built QtCore. It preflights the archive, replaces the library in a disposable copy, applies local ad-hoc integrity signatures, checks the original app sections and imports, verifies the loaded marker and interaction, then cleans up its temporary app registration and copy. It refuses CI and does not change installed or published files. Controlled native testing is not manual acceptance, Developer ID signing or notarization.

## 性能与资源证据

scripts/measure-desktop.py 使用 tools/process-metrics 的只读系统计数器。macOS 读取物理 footprint、CPU 和磁盘 I/O，CPU 的 Mach tick 按宿主 timebase 换算为秒，并与自有进程的独立 CPU 时钟交叉核验；Linux 使用 PSS、CPU ticks 和 /proc I/O；Windows 使用私有提交、CPU 与进程 I/O transfer。Windows transfer 不等于物理磁盘字节，各平台内存指标也不混作同一物理内存排名。进程开始时间区分 PID 复用，100 ms 采样仍可能漏峰、短命后代及退出前最后 I/O。

```sh
python3 scripts/run-performance-suite.py --app /path/to/xdzip \
  --metrics build/ui-release/tools/process-metrics/xdzip-process-metrics \
  --core target/release/xdzip-m3-probe --output build/review/performance
```

脚本生成自有夹具，实际执行 30 次新进程首帧、十万条打开、搜索／排序后帧通知及取消；稳定十秒后观察至少六十秒空闲，再观察三十秒浏览内存。真实解压由 GUI 产品与同 Rust 核心各执行五次，保持默认校验、同步和发布，独立核对全部字节。统计包括样本数、中位数、P95、最小／最大、标准差和 bootstrap 区间。先 GUI 后核心的顺序及非受控缓存明确记录，不把时间比直接归因为 GUI 损耗。

源码编译等大负载完成后再采集完整本地性能，记录构建、后端、设备与电源观测。现有宿主上的观测不是发布负责人已批准的稳定参考机，不清缓存、不调整全局节能、不关闭安全检查，也不自动批准性能预算。

实际内嵌资源通过独立 opt-in 事件导出，只枚举 :/xdzip，下游逐个对照 qrc 对应的源码哈希，保留字体和项目许可引用；不自动批准生成式资产权利。链接器 map、静态库成员和保留对象记录提供真实链接输入，不冒充内联／泛型实例化后的逐字节许可归属。

## 汇总、材料及剩余责任

scripts/collect-wp12-ci.py 从固定项目的精确提交收集 CI 元数据及选定 JSON／开发安装器。令牌仅发送到 GitHub API，不转发到存储重定向；大型工件按范围读取，大小有界，不运行下载的二进制。scripts/prepare-wp12-materials.py 核对 SDK 声明、导入、安装／关联和 SBOM，再把每个上游来源／许可缺口整理为具体审查问题。原始差异和 incomplete 保留，无法从一个哈希推导缺失的发布者构建／签名历史。

scripts/assemble-wp12-evidence.py 自动组装 CI、源码、Qt 替换、性能和三平台包记录，再调用严格门槛，不需要人工拼接 JSON。scripts/wp12-readiness.py 校验每项证据路径、哈希、提交、平台覆盖和类型，拒绝脏构建、混合身份、缺失源码加载标记及不足的采样。所有自动项齐全才输出 automatic_complete=true；release_ready 始终为 false，--require-release-pass 在外部门槛未完时拒绝通过。

最终应交付开发安装器／本地 DMG、源码与通知、同提交 SBOM、性能报告、逐项审查材料和一个汇总报告。自动项完成与正式发行获批分别判断。WP-12 的八项人工验收及支持政策已按所有者结论收口，见 [#5](https://github.com/xiaodaozhi/xdzip/issues/5)与[支持政策](support-policy.md)；后续不因重建证据而要求重复。来源与许可复核继续由[发行审查 #4](https://github.com/xiaodaozhi/xdzip/issues/4)跟踪。新候选或平台扩展仍须按实际变更选择桌面和性能复验，脚本不代替所有者的发行决定或许可复核结论。
