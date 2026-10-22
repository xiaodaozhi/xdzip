# 对应源码与重建材料

scripts/prepare-source-delivery.py 从干净提交收集项目源码、Cargo.lock 对应的完整 Rust vendor 输入、固定 Qt 6.8.2 模块源码归档和重建／替换说明。所有产物位于 build/，不把下载地址称为已交付源码，不自动批准开源许可分支。

```sh
python3 scripts/prepare-source-delivery.py --output build/review/source-delivery \
  --qt-prefix .cache/Qt/6.8.2/macos
python3 scripts/verify-source-delivery.py --delivery build/review/source-delivery \
  --qt-prefix .cache/Qt/6.8.2/macos --output build/review/source-rebuild
```

Qt 运行时模块归档为 qtbase、qtdeclarative、qtsvg、qttranslations，另交付构建所需的 qtshadertools、qttools 及固定 Rust 标准库源码组件，来自 [Qt 官方 6.8.2 归档](https://download.qt.io/archive/qt/6.8/6.8.2/submodules/)。官方 .sha256 值固定于 scripts/corresponding-sources.json；缓存与下载均重新校验，下载先写 partial，再验证后提交。只检查归档目录，不自动解包第三方路径。Cargo vendor 使用 --locked，保留 registry 校验与构建依赖，不执行第三方构建脚本。

输出包含实际固定提交的 xdzip-source.tar、rust-vendor.tar.gz、离线 Cargo 配置、Qt 原始归档、REBUILD.md 和逐文件 SHA-256／大小清单。清单明确为 collected-not-compliance-verified，不能替代 SDK 构建 features／补丁、平台运行库、内嵌资源和许可履约的核对。项目源码重新配置的身份可能为 unknown，应由外部交付清单绑定原提交，不能伪造干净 Git 身份。

另附固定的 Mesa 11.2.2、LLVM 3.6.2 原始源码归档及 ICU 73.2 完整许可，服务于保留的 Windows 软件渲染库和 Linux ICU；各平台原始通知的收集及供应方构建历史限制见[原生运行库来源与通知](native-runtime-materials.md)。这些材料同样纳入交付总清单，不能将新补材料记作历史包已经包含。

应用动态链接 Qt，提供源码重建入口和兼容库替换说明。[macOS 部署核验](macos-deployment.md)覆盖隔离副本的修改库加载，不声称已从源码重建整套 Qt。实际 Qt 源码构建与产品加载另有独立流程，见[自动验证契约](wp12-automation.md)。SDK 字节级构建等价性、精确静态链接归属和许可分支仍属于[发行许可复核](https://github.com/xiaodaozhi/xdzip/issues/4)，不重新列为已完成 WP-12 的开发待办。

verify-source-delivery.py 先重新核对交付文件哈希，将自有项目和 vendor 安全解到新的 build/ 目录，使用空 Cargo 缓存、CARGO_NET_OFFLINE=true 和现有固定 Qt SDK 构建真实应用，再执行启动／关闭冒烟。它验证项目与 Rust 交付输入可离线使用，不声称 Qt 已从模块源码重建，不声称字节级可重复构建。生成源码目录的 Git 身份为 unknown，由外部固定提交及交付清单绑定。

新增[Qt SDK 对应核验](qt-sdk-correspondence.md)：交付包总是包含固定对应清单和可重放的 AGL 消费端补丁，传入 --qt-prefix 后额外包含该 SDK 的逐文件源码声明核对、构建配置和明确缺口。交付清单递归哈希补丁子目录，重建验证同样复核这些字节。未传 SDK 时不伪造配置核验记录。

实际 Qt 源码构建和加载／兼容替换流程见[自动化收尾契约](wp12-automation.md)，不再把应用使用现有 SDK 的离线重建误称为 Qt 本身的源码重建。
