# Sparkle 代理扩展维护

本目录保存固定 Sparkle 2.10.0 的小范围源码补丁。来源、SDK／源码归档、许可与补丁 SHA-256 由 [sparkle-dependency.json](../../scripts/sparkle-dependency.json) 锁定。更新上游版本时须重新审查源码集合、编译特性和安装兼容性，不能仅替换版本号或摘要。

`xdzip_setDownloadProxyURL:error:` 是下游扩展，不属于官方 Sparkle API。它只接收无凭据 HTTP CONNECT／SOCKS5 地址，在主线程、updater 启动前调用；配置复制到该实例，并贯通临时 feed 和持久归档下载。启动后不允许修改，非系统代理不支持 Downloader XPC，错误不降级为系统代理。原生代理显式禁止失败后直连。`nil` 保留系统配置，不表示直连。

补丁不改变签名、候选、解压、安装、授权、退出或重启协议。应用中的直连模式仍走手动下载。代理模式和地址只由应用设置产生，不能由发布清单或归档选择。当前产品未启用 Downloader XPC，不向上游服务传入新的未支持协议。

## 构建与来源

[prepare-sparkle.py](../../scripts/prepare-sparkle.py) 先核对原始归档摘要，再在临时目录应用补丁。构建器读取锁定 Xcode 工程的框架与 Ed25519 源码阶段，用原生 Clang 编译全部 77 个单元。保留上游构建特性，主库仅产出产品支持的 arm64，最低 macOS 14。无需运行下载源码中的构建脚本或安装完整 Xcode。

框架资源、原生安装助手和签名工具保留固定官方 SDK 内容。旧主库 dSYM 不再保留，修改后的主库和框架完整性封装使用必要的 ad-hoc 签名；部署时仍按安装助手、框架与应用的既有顺序分别签名，保留助手专属 entitlements。此构建不冒充 Developer ID 或公证，也不声称重建了 SDK 内所有二进制。

构建回执包含 variant、源码／SDK／补丁／构建器摘要、主库及框架逐文件摘要。准备、通知、部署归属和 SBOM 使用派生身份 `2.10.0-xdzip-proxy.1`，不将其等同于官方 SDK 字节。源码交付脚本只在用户另行明确要求时运行，本准备入口不会调用它。

## 验证

`check-macos-proxy.py` 使用实际派生 Sparkle 下载器，验证 HTTP CONNECT、SOCKS5、启动后配置不可变、不可用代理、无凭据边界及篡改 feed。受控域名仅由测试代理映射到自有回环服务，普通成功下载必须同时提供代理命中证据。该检查在归档解压和安装之前退出，不代替产品升级验收。

完整产品更新夹具复用同一原生提供器，分别执行 HTTP／SOCKS5 的实际旧版到新版安装、重启与配置快照，并保留原有取消、失败、恢复、设置与关联检查。执行状态和平台限制记录在 issue；生成日志、包和回执只保留在 build 或 CI。

English: This downstream API freezes a credential-free HTTP CONNECT/SOCKS5 proxy before each updater starts and applies it to feed and archive sessions without direct failover. It rejects unsupported XPC configurations and changes no signing or installation protocol. The arm64 framework library and Ed25519 units are rebuilt from pinned source and patch; resources, helpers and tools retain their pinned SDK origin. Derived identities and file receipts distinguish these origins. Transport probes stop before extraction and do not replace full native product update acceptance.
