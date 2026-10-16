# 对应源码与重建材料

scripts/prepare-source-delivery.py 从干净提交收集项目源码、Cargo.lock 对应的完整 Rust vendor 输入、固定 Qt 6.8.2 模块源码归档和重建／替换说明。所有产物位于 build/，不把下载地址称为已交付源码，不自动批准开源许可分支。

```sh
python3 scripts/prepare-source-delivery.py --output build/review/source-delivery
```

四份 Qt 模块归档为 qtbase、qtdeclarative、qtsvg、qttranslations，来自 [Qt 官方 6.8.2 归档](https://download.qt.io/archive/qt/6.8/6.8.2/submodules/)。官方 .sha256 值固定于 scripts/corresponding-sources.json；缓存与下载均重新校验，下载先写 partial，再验证后提交。只检查归档目录，不自动解包第三方路径。Cargo vendor 使用 --locked，保留 registry 校验与构建依赖，不执行第三方构建脚本。

输出包含实际固定提交的 xdzip-source.tar、rust-vendor.tar.gz、离线 Cargo 配置、Qt 原始归档、REBUILD.md 和逐文件 SHA-256／大小清单。清单明确为 collected-not-compliance-verified，不能替代 SDK 构建 features／补丁、平台运行库、内嵌资源和许可履约的核对。项目源码重新配置的身份可能为 unknown，应由外部交付清单绑定原提交，不能伪造干净 Git 身份。

应用动态链接 Qt，提供源码重建入口和兼容库替换说明。[macOS 部署核验](macos-deployment.md)覆盖隔离副本的修改库加载，不声称已从源码重建整套 Qt。完整平台构建等价性、精确静态链接归属、许可分支和独立审查仍属于[WP-12 准入](../qa/private-test-readiness.md)。
