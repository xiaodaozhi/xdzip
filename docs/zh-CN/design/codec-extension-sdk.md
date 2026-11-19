# 格式适配器与扩展试验

[English](../../design/codec-extension-sdk.md) | 简体中文 | [繁體中文（台灣）](../../zh-TW/design/codec-extension-sdk.md)

## 三条执行路径

| 路径 | 实现与可用性 |
| --- | --- |
| 宿主进程内的内置适配器 | 桌面与 CLI 使用同一套固定格式注册表和运行时路由。ZIP、TAR、压缩流过滤器以及 DEB、ASAR 等受支持容器的 Rust 实现在宿主进程内调用。 |
| 随包提供的归档工作进程 | 宿主中的 `xdzip-codec-native` 启动 `xdzip-native-codec`。其 C++ 协议层通过 `xdzip-codec-bridge` 调用 Rust 的 7z、RAR、CAB 和光盘镜像后端。符合条件的 7z 创建与重写也走这条路径。ISO 创建使用进程内的光盘镜像写入器。 |
| 外部 SDK 试验 | `xdzip-plugin-host` 与 `xdzip-plugin-protocol` 组成独立且需要显式运行的 XDP1 试验。桌面和 CLI 产品构建不会发现、安装或运行第三方扩展。 |

前两条路径属于产品内置组件。随包工作进程使用专用协议与资源监督。外部试验有自己的包验证、版本协商和平台权限门槛。[系统架构](system-architecture.md)、[归档工作进程契约](../engineering/native-codecs.md)、[SDK 试验契约](../engineering/sdk-trial.md)

## 产品路由与操作能力

[格式注册表源文件](https://github.com/xiaodaozhi/xdzip/blob/main/specs/engineering/archive-formats.json)声明扩展名、容器和过滤器类型、支持的操作、方法及创建选项。生成器同步 Rust 注册表、桌面列表及设计产物。读取时，运行时探测并验证来源数据，再选择后端。创建时，指定的输出名称选定创建配置，运行时依配置校验选项。扩展名列表用于发现和界面提示，成功探测和扫描才能确定打开的格式。

读取、预览、解压、整包检查、创建和重写各有限制。可浏览的条目可能缺少受支持的内容解码方法。密码、分卷和特殊记录也影响准入。运行时还处理格式间的差异，例如 ISO/UDF 的结构检查，以及无法重写的配置。[运行时路由](https://github.com/xiaodaozhi/xdzip/blob/main/crates/xdzip-runtime/src/formats.rs)、[格式注册表契约](../engineering/format-registry.md)、[归档检查](../engineering/archive-checking.md)

## 内部 Rust 适配器接口

| 接口 | 职责 |
| --- | --- |
| `ReadSource` 与 `MetadataCodec` | 读取或定位宿主授权的来源，使用适用的分卷元数据，产生受限的格式记录。 |
| `PayloadCodec`、`PayloadReader` 与 `DecodedSink` | 定位内容并将符合条件的物理条目解码到宿主接收器。后端支持时分别报告实际字节、为跳过数据付出的解码工作量和辅助工作量。 |
| `CreationCodec`、`EncodeInput` 与 `EncodeOutput` | 使用冻结的输入元数据和按编号访问的数据流进行编码。写入宿主持有的接收器，写入器需要时可执行受限的随机写入。 |
| `RewriteCodec` | 依据保留的原条目、修改及已授权的新增内容，将支持的归档重写到新的暂存文件。 |

路径、来源复核、取消、密码、额度及最终提交由宿主负责。编解码器通过操作所需的接口获得能力。归档工作进程请求已获准的来源范围，并返回有界记录或输出块。宿主在处理前验证协议状态及字节额度。工作进程异常或响应无效时无法授权发布。工作进程边界用于故障与资源隔离，对任意外部代码的内核权限限制属于试验的独立要求。[元数据契约](https://github.com/xiaodaozhi/xdzip/blob/main/crates/xdzip-codec-api/src/lib.rs)、[内容解码契约](https://github.com/xiaodaozhi/xdzip/blob/main/crates/xdzip-codec-api/src/payload.rs)、[编码与重写契约](https://github.com/xiaodaozhi/xdzip/blob/main/crates/xdzip-codec-api/src/encode.rs)、[文件授权](file-security.md)

这些 Rust trait 是内部源码接口。产品适配器显式接入工作区与运行时路由。允许的操作需要分别验证有效文件、不受支持的功能、资源限额、数据损坏及写出行为。参见[独立 Rust 编解码契约](../engineering/independent-rust-codecs.md)和[开发指南](../contributing/README.md)。

## 已实现的外部 SDK 试验

试验接收显式选定的原生包清单和宿主选定的来源，支持扫描项目自有的 XDP1 样本并读取一个普通条目。包准入验证版本为 1 的清单、可执行目标与哈希、包内路径、固定权限配置和限额。宿主执行已验证的私有副本。声明的哈希用于确认包内容一致，发布者认证需要独立的信任策略。

版本 1 的试验协议使用长度前缀和有界 CBOR，并核对操作身份与顺序。宿主通过不透明令牌授权读取来源或写入一个接收器。对端接收允许的字节范围和单块写入额度。宿主计量并验证内容，检查协议完成状态和来源状态，并在对端正常退出后以无覆盖方式提交。试验不向对端开放归档密码、分卷、创建、修改、链接或宿主路径。

试验在 macOS 和 Linux 上有显式的原生权限门槛，平台要求和失败即拒绝行为以试验契约为准。Windows 拒绝执行。互操作与拒绝访问测试独立于桌面运行。实际清单格式、限额和测试入口见[试验运行与通信契约](../engineering/sdk-trial.md)。[试验宿主](https://github.com/xiaodaozhi/xdzip/blob/main/crates/xdzip-plugin-host/src/lib.rs)、[数据帧实现](https://github.com/xiaodaozhi/xdzip/blob/main/crates/xdzip-plugin-protocol/src/lib.rs)

## 尚未实现的设计选项

面向产品的扩展管理、发布者信任与升级策略、试验协议之外的兼容性以及外部写入或密码能力，仍属于设计工作。进入产品前，各支持平台与能力都需要验证权限限制、互操作性和故障行为。随包工作进程和 XDP1 试验都不会启用这项产品功能。[架构决策](architecture-decisions.md)
