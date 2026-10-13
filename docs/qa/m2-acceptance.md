# M2 完整 ZIP 只读浏览验收

状态：M2 开发与自动化验收完成；Apple Silicon 桌面人工体验待一次性验收。日期：2026-10-13。

验证的实现提交为 `a40dc72259d5f87f3c20b4fd9c35ba8a068d45c5`。后续证据文档提交不改变应用实现；开发包的 build.json 与此提交一致。

## 已实现

单卷 ZIP／ZIP64 最多 100,000 条真实物理元数据；重复记录与精确 64 位声明大小保留，普通 ZIP 恰好 65,535 条也可读取。根目录和普通子目录仅显示直属文件／文件夹，搜索递归当前范围。类型排序依据目录标记，两个方向都保持目录在前。

直属目录索引只保存身份，完整查询只过滤和排序一次。产品页面携带会话、查询 ID、GUI 代际、快照修订及偏移；批次最多 128 行、名称 64 KiB。Qt 合并最新搜索／目录／排序意图并取消旧工作，内部批次不构成界面分页。普通查询取消保留最近成功的列表；源变化使旧会话失效，查询及预检都会清空旧列表、目录树、选择和详情，错误报告继续可读。

来源指纹检查打开的句柄和当前路径，Unix 包含设备／inode，Windows 包含卷号／文件索引，能够检测保留修改时间的同名替换。这是尽力检测，不是文件系统原子快照或写入授权。Windows 安全封装的准入见 [依赖说明](../engineering/dependencies.md)。

选择覆盖完整当前查询，汇总按选择修订缓存，用 Rust u128 精确求和；包含隐式目录或超过 128 个物理记录的选择仍不能发起元数据预检。列表／网格按可见区域实例化委托。

## 原生三平台验证与下载

[Foundation probes](https://github.com/xiaodaozhi/xdzip/actions/runs/38406277005)、[Qt components](https://github.com/xiaodaozhi/xdzip/actions/runs/38406277351)、[Documentation](https://github.com/xiaodaozhi/xdzip/actions/runs/38406278000) 全部成功。完整身份、任务及工件哈希见 [CI 记录](m2/native-ci.json)。

| 平台 | 核心／桥接 | Qt 全流程 | 开发工件 |
| --- | --- | --- | --- |
| macOS 14.8 ARM64 | 通过 | 通过 | [Apple Silicon](https://github.com/xiaodaozhi/xdzip/actions/runs/38406277351/artifacts/11819103766) |
| Windows Server 2022 X64 | 通过 | 通过 | [Windows](https://github.com/xiaodaozhi/xdzip/actions/runs/38406277351/artifacts/11817214725) |
| Ubuntu 24.04 X64 | 通过 | 通过 | [Linux](https://github.com/xiaodaozhi/xdzip/actions/runs/38406277351/artifacts/11816799807) |

Qt 全流程包含 6 组 CTest、生命周期套件连续 10 次、实际十万条 ZIP64 查询、75 项布局契约和 DPR 1／1.25／2 各 729 项独立布局检查。部署后的 macOS Cocoa／Windows 启动关闭检查通过。HTML 未改动，本次检查生成输出与共享资产，没有重复整套浏览器视觉测试。

macOS 工件内包含 `xdzip-macos-arm64.zip`，约 49.5 MiB，解压后为 xdzip.app；同时提供源代码、许可、截图、日志和 build.json。已下载核对工件 SHA-256、build.json、ARM64 Mach-O 及 macOS 14.0 部署目标，见 [包核验](m2/bundle-receipt.json)。采用 ad-hoc 签名，CI 中 codesign 严格校验通过，尚未公证。Actions 工件于 2026-10-27 到期；过期后可从实现提交重建。

## 测量与本地证据

自建数据集是十万条零字节文件组成的有效 ZIP64，另有分层与固定种子乱序样本；用于真实元数据扫描与查询，不替代内容解码或压缩／解压基准。生成方式见 [脚本](../../scripts/generate-m2-fixture.py)，核心探针见 [使用说明](../../tools/m2-probe/README.md)。大样本只生成在 build 或临时目录，不进入源码仓库。

Linux 本地 release 核心：读取与索引约 75ms、名称排序约 27ms、后续 781 批次约 2.8ms、检索约 8ms，峰值 RSS 约 19MiB，见 [核心 JSON](m2/local-core-performance.json)。macOS ARM64 CI 核心：读取约 241ms、排序约 24ms，见 [JSON](m2/native-mac-core-performance.json)。

Qt 6.8.2 release、offscreen / software 的完整场景测量见 [JSON](m2/native-qt-performance.json)：

| CI 平台 | 十万条完整加载 | 全结果排序并展示 | 场景峰值 RSS | 可见单元格委托 |
| --- | --- | --- | --- | --- |
| Linux X64 | 579ms | 417ms | 231.6 MiB | 55 |
| Windows X64 | 707ms | 610ms | 未测量 | 55 |
| macOS ARM64 | 2060ms | 800ms | 229.6 MiB | 55 |

这些是共享 runner 的单次记录，不作为稳定参考机性能承诺。峰值包括加载、全选、排序、后续搜索与分层查询，不能称作空闲内存。55 个委托是约 11 行 × 5 列的可见单元格。

本地 debug Qt 场景加载约 957ms、排序约 652ms、峰值约 213MiB，见 [JSON](m2/local-qt-performance.json) 和 [日志](m2/local-qt-performance.log)。实际 Qt 截图见 [HTML 证据页](m2/index.html)。截图来自 Linux 离屏软件渲染，不能替代 Cocoa 桌面或整套原稿的人工验收。

Rust 工作区、Clippy、Windows 目标类型／Clippy 检查及本地 [5 组 CTest](m2/local-tests.log)、[连续 10 次回归](m2/local-repeat.log) 通过。新增回归覆盖十万条 ZIP64、超过 2^53 的大小、ZIP64 扩展磁盘、损坏尾记录／locator、显示名预算、跨批次共享身份、四维过期键、最新意图、源修改／保留时间戳的替换，以及预检失效时清空 Qt 状态。独立布局检查见 [JSON](m2/local-fidelity.json)，两份大归档经 Python 标准库独立读取及 CRC 校验见 [记录](m2/local-interop.json)。内存和传输约束见 [M2 契约](../engineering/m2-readonly.md)。

Qt mailbox 压力回归验证队列峰值 256、字节不超过 8 MiB，并验证满队列关闭与终态交付；Rust 事件背压和控制通道另有回归。此结果不等于十万条场景始终占满队列。

## 桌面人工验收及范围

打开普通 ZIP 和生成的十万条样本；连续搜索／切换目录；五列表头排序；跨可见区选择及全选；确认根目录直属项、搜索完整路径、类型目录优先；修改来源后查询及预检均要求重新打开；退出与 Cmd+Q 等待异步关闭。一次性核对最新 macOS 上的红绿灯、浅深色、200% 字号、系统 DPI、中文输入法候选／提交及辅助功能。原生 CI 并未代替这些桌面人工操作。

内容预览、CRC／本地头验证、真实解压、创建、分卷、猜测编码、第三方插件和发行签名尚未实现。目录导航另有 32,768 节点／64 层／4 MiB 限额，超限整体降级。下一实现阶段为 M3 安全文件原语和真实解压闭环。

## CI 查询与修复记录

此前“main 未触发 CI”的说明有误：连接器按提交查询的工具只返回 PR 事件。完整运行列表确认 main push 正常触发，当前按 head_sha 核验。Windows 替换来源检测已补齐句柄身份；Qt 测量采用显式文件日志；macOS 从网格切回列表后的表头测试等待新帧、验证命中范围并移动指针后点击，全部原有排序断言保留。完整套件外层时限为 120 秒，各断言局部期限保留；最终实现提交通过三平台重复回归。
