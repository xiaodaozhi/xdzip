# Linux 独立运行目录与验证

当前开发部署基线为 Ubuntu 24.04、x86_64、X11。目标是脱离开发 Qt SDK 运行当前产品，不据此承诺其他发行版、旧 glibc、Wayland 或完整安装器支持。macOS CI 保持原生编译、测试与启动验证，本地开发 DMG 按[Mac 部署指南](macos-deployment.md)处理，不受本流程影响。

## 布局和生成

`scripts/deploy_linux.py` 只接受可信自有 ELF 和明确的 Qt 6.8.2 SDK，在新的 build/ 或 dist/ 目录生成：

- `xdzip` 为启动入口，使用当前目录的库、插件和 QML，不继承开发 SDK 搜索路径。
- `bin/xdzip` 为产品，旁边 `qt.conf` 将 Qt 路径改为相对布局。
- `lib/` 为实际导入的 Qt／SDK 共享库，`qml/` 为扫描得到的 Qt QML 模块根目录，保留控件样式及实现子模块。
- `plugins/` 包含 xcb、离屏验证、图片、图标、平台输入、xcb 图形集成和必需的 `tls/libqopensslbackend.so`；缺少 OpenSSL 后端时部署失败。不把 EGLFS、帧缓冲或 Wayland 平台插件作为已验证交付。
- `translations/` 保留 Qt 标准控件翻译，`share/notices/` 交付当前通知和源码依赖材料。
- `share/deployment.json` 记录 QML、系统运行依赖、复制的 SDK 库和边界，README.md 提供使用方法。

扫描器使用固定 Qt 版本的 [qmlimportscanner](https://doc.qt.io/qt-6.8/qtqml-tooling-qmlimportscanner.html)。Xdzip.Settings／Xdzip.Native 是产品内建注册模块，不复制开发静态库。缺少普通依赖、QML 路径逃逸 SDK、同名库冲突或未知外部库均失败。原生导入递归收集只复制 SDK 库，不捎带宿主 glibc 或图形驱动；系统依赖按实际 Debian 包归属列出。

HTTPS 另显式要求 Ubuntu 的 `libssl3t64` 与 `ca-certificates`，验证 `libssl.so.3`、`libcrypto.so.3` 的系统包归属及非空系统 CA 信任库。Qt OpenSSL 后端可能在运行时加载这些库，不能仅凭 ldd 推断依赖完整。部署清单保留已安装的二进制包版本、源包与源包版本；安装器声明两项依赖，实际文件 SBOM 将其记录为必需 OS 包，不把系统库打包进应用。

每个 ELF 使用 patchelf 设置相对 `$ORIGIN` RPATH，插件按自身目录到 lib/ 的相对距离计算。Qt 的路径重定向按 [qt.conf 官方规则](https://doc.qt.io/qt-6.8/qt-conf.html)执行，允许整个目录迁移。库保持动态链接和可替换，不限制用户更换兼容 Qt。通用 Linux 部署边界见 [Qt Linux 部署说明](https://doc.qt.io/qt-6.8/linux-deployment.html)。

日常使用[统一构建入口](developer-build.md)分阶段生成和核验；先完成同配置 build／test 与打包准备，再执行：

```sh
python3 scripts/desktop.py doctor --for-stage package
python3 scripts/desktop.py package --kind all
python3 scripts/desktop.py verify --kind all
```

默认构建目录为 build/desktop，交付输出为 build/package，Qt SDK 默认 `.cache/Qt/6.8.2/gcc_64`；其他位置在各阶段传入共用参数。`package --kind runtime` 生成运行目录与审计材料；随后在同一 output-dir 执行 `package --kind installer`，生成 tar.gz 与 .deb。默认 `all` 依次完成两部分。底层 [deploy_linux.py](../../scripts/deploy_linux.py)、[package_linux.py](../../scripts/package_linux.py)和 [verify_linux_runtime.py](../../scripts/verify_linux_runtime.py)保留平台逻辑，需要定位故障时参考各自 `--help`。

部署只在原生 Linux x86_64 执行，不覆盖既有目录。打包前预检 qmlimportscanner、ldd、patchelf、dpkg-query 与安装器所需的 dpkg-deb；这些构建工具不进入产品。运行容器验证另外需要 Docker 和可访问的 daemon，其构建只针对固定 Ubuntu 镜像和记录的 OS 包。Docker 缺失不阻止普通 build／test，verify 则明确失败，不把未执行隔离检查写成通过。普通 verify 不在宿主安装 .deb 或修改文件关联，安装生命周期仍由一次性 CI 环境执行。

## 脱离 SDK 的自动化证据

容器基于固定 digest 的 Ubuntu 24.04，安装实际 OS 运行依赖和 Xvfb／字体／locale 测试支持，不安装 Qt、Rust 或编译器，并再次检查其不存在。容器不挂载开发 SDK 或整个源码树，只挂载只读产品目录、单独的自有十万条目 ZIP 和验证脚本。归档运行验证关闭网络、禁止提权，仅临时配置目录可写。

应用被挂载到带空格的新路径，在清除开发搜索变量后，经启动入口检查：

1. 实际 `--version` 可运行。
2. X11 下真实 QML 窗口启动并安全关闭。
3. 原生产品探针完成首帧、十万条目读取、活动请求取消与退出，计数和取消结果须符合契约。
4. 宿主清除开发变量后，以 deployed 模式核验全部打包 ELF；不允许回退 SDK。实际文件 SBOM 保持同一版本／提交身份并通过官方 schema。
5. 在相同无 SDK 镜像的新进程中启用网络，以隔离的 `XDZIP_SETTINGS_PATH` 执行产品 `--update-probe`，实际请求 GitHub HTTPS。必须报告 OpenSSL 后端可用并完成更新检查，单独保留 `.https.json` 与 `.https.log`；网络、证书或 TLS 失败均阻止验证通过。

linux-runtime.json 记录实际事件与可执行文件 SHA-256；Xvfb／Qt Quick 软件渲染仅用于运行闭环，不能证明物理 GPU、中文候选窗口、真实读屏或完整桌面安装通过。

## 下载交付与剩余门槛

CI 生成 xdzip-linux-x86_64.tar.gz 并记录 SHA-256 和可执行文件哈希。tar.gz 保留执行权限，避免 Actions ZIP 解压丢失入口执行位；归档路径、UID/GID 和时间规范化，不写入私人目录。下载后解压并运行目录内 `./xdzip`，不能只搬走 bin/xdzip。原始 linux/ 不重复上传。

该运行目录仍是实验开发交付，其 tar.gz 不自动安装或改默认，另提供实验 .deb 安装与打开候选，仍未签名。平台范围以[支持政策](support-policy.md)为准，其他发行版与桌面、Wayland 等扩展另建 issue。已有桌面验收保留于 [WP-12 #5](https://github.com/xiaodaozhi/xdzip/issues/5)，后续变更按受影响项复验真实输入法／读屏及干净安装升级卸载，不因重建目录重复要求既有验收。完整组件许可归属由[发行审查 #4](https://github.com/xiaodaozhi/xdzip/issues/4)跟踪，目录运行通过不自动完成发行门槛。

Ubuntu .deb 的安装／升级／回退／卸载、运行锁与桌面 MIME 自动验证见[安装与关联](installers-associations.md)，运行容器的通过不替代真实桌面专项与发行许可复核。
