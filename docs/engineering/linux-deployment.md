# Linux 独立运行目录与验证

当前开发部署基线为 Ubuntu 24.04、x86_64、X11。目标是脱离开发 Qt SDK 运行当前产品，不据此承诺其他发行版、旧 glibc、Wayland 或完整安装器支持。macOS CI 保持原生编译、测试与启动验证，本地开发 DMG 按[Mac 部署指南](macos-deployment.md)处理，不受本流程影响。

## 布局和生成

`scripts/deploy_linux.py` 只接受可信自有 ELF 和明确的 Qt 6.8.2 SDK，在新的 build/ 或 dist/ 目录生成：

- `xdzip` 为启动入口，使用当前目录的库、插件和 QML，不继承开发 SDK 搜索路径。
- `bin/xdzip` 为产品，旁边 `qt.conf` 将 Qt 路径改为相对布局。
- `lib/` 为实际导入的 Qt／SDK 共享库，`qml/` 为扫描得到的 Qt QML 模块根目录，保留控件样式及实现子模块。
- `plugins/` 包含 xcb、离屏验证、图片、图标、平台输入与 xcb 图形集成插件；不把 EGLFS、帧缓冲或 Wayland 平台插件作为已验证交付。
- `translations/` 保留 Qt 标准控件翻译，`share/notices/` 交付当前通知和源码依赖材料。
- `share/deployment.json` 记录 QML、系统运行依赖、复制的 SDK 库和边界，README.md 提供使用方法。

扫描器使用固定 Qt 版本的 [qmlimportscanner](https://doc.qt.io/qt-6.8/qtqml-tooling-qmlimportscanner.html)。Xdzip.Settings／Xdzip.Native 是产品内建注册模块，不复制开发静态库。缺少普通依赖、QML 路径逃逸 SDK、同名库冲突或未知外部库均失败。原生导入递归收集只复制 SDK 库，不捎带宿主 glibc 或图形驱动；系统依赖按实际 Debian 包归属列出。

每个 ELF 使用 patchelf 设置相对 `$ORIGIN` RPATH，插件按自身目录到 lib/ 的相对距离计算。Qt 的路径重定向按 [qt.conf 官方规则](https://doc.qt.io/qt-6.8/qt-conf.html)执行，允许整个目录迁移。库保持动态链接和可替换，不限制用户更换兼容 Qt。通用 Linux 部署边界见 [Qt Linux 部署说明](https://doc.qt.io/qt-6.8/linux-deployment.html)。

```sh
python3 scripts/deploy_linux.py --app build/desktop/apps/desktop/xdzip \
  --qt-prefix /path/to/Qt/6.8.2/gcc_64 --notices build/review/notices \
  --output build/review/linux
python3 scripts/verify_linux_runtime.py --root build/review/linux \
  --fixture build/desktop/apps/desktop/m2-flat.zip --output build/review/linux-runtime.json
```

脚本只在原生 Linux x86_64 部署，不覆盖既有目录。它依赖构建工具 qmlimportscanner、ldd、patchelf 和 dpkg-query；这些工具不进入产品。运行容器验证另外需要 Docker，其构建只针对固定 Ubuntu 镜像和所记录的 OS 包。

## 脱离 SDK 的自动化证据

容器基于固定 digest 的 Ubuntu 24.04，安装实际 OS 运行依赖和 Xvfb／字体／locale 测试支持，不安装 Qt、Rust 或编译器，并再次检查其不存在。容器不挂载开发 SDK 或整个源码树，只挂载只读产品目录、单独的自有十万条目 ZIP 和验证脚本。运行时关闭网络、禁止提权，仅临时配置目录可写。

应用被挂载到带空格的新路径，在清除开发搜索变量后，经启动入口检查：

1. 实际 `--version` 可运行。
2. X11 下真实 QML 窗口启动并安全关闭。
3. 原生产品探针完成首帧、十万条目读取、活动请求取消与退出，计数和取消结果须符合契约。
4. 宿主清除开发变量后，以 deployed 模式核验全部打包 ELF；不允许回退 SDK。实际文件 SBOM 保持同一版本／提交身份并通过官方 schema。

linux-runtime.json 记录实际事件与可执行文件 SHA-256；Xvfb／Qt Quick 软件渲染仅用于运行闭环，不能证明物理 GPU、中文候选窗口、真实读屏或完整桌面安装通过。

## 下载交付与剩余门槛

CI 生成 xdzip-linux-x86_64.tar.gz 并记录 SHA-256 和可执行文件哈希。tar.gz 保留执行权限，避免 Actions ZIP 解压丢失入口执行位；归档路径、UID/GID 和时间规范化，不写入私人目录。下载后解压并运行目录内 `./xdzip`，不能只搬走 bin/xdzip。原始 linux/ 不重复上传。

该运行目录仍是实验开发交付，其 tar.gz 不自动安装或改默认，另提供实验 .deb 安装与打开候选，仍未签名。平台范围以[支持政策](support-policy.md)为准，其他发行版与桌面、Wayland 等扩展另建 issue。已有桌面验收保留于 [WP-12 #5](https://github.com/xiaodaozhi/xdzip/issues/5)，后续变更按受影响项复验真实输入法／读屏及干净安装升级卸载，不因重建目录重复要求既有验收。完整组件许可归属与独立安全审查由[发行审查 #4](https://github.com/xiaodaozhi/xdzip/issues/4)跟踪，目录运行通过不自动完成发行门槛。

Ubuntu .deb 的安装／升级／回退／卸载、运行锁与桌面 MIME 自动验证见[安装与关联](installers-associations.md)，运行容器的通过不替代真实桌面专项与独立发行审查。
