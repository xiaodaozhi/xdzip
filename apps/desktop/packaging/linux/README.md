# xdzip Linux 开发运行目录

当前验证基线为 Ubuntu 24.04、x86_64、X11。此目录包含 Qt 6.8.2、QML、平台插件、标准控件翻译和许可通知，不要求安装开发 Qt SDK；系统仍须提供图形、字体、glibc 等运行库。

解压整个 tar.gz 后，在本目录运行 `./xdzip`。保留 bin、lib、qml、plugins、translations 和 share 的相对布局，可将整个目录移动到带空格的路径。不要直接从其他目录复制 bin/xdzip；入口脚本会设置本目录的运行路径。Qt 共享库保持可替换，未做静态链接或阻止用户修改库。

系统依赖名称见 share/deployment.json 的 system_packages，来源于构建基线的实际原生导入。它们是 OS 运行依赖，不是需要安装的 Qt／Rust 开发工具。其他发行版、较旧 glibc、Wayland、真实输入法和读屏均未据此宣称通过。目录没有安装器、不自动注册文件关联或改写系统设置。

share/notices 包含源码依赖、Qt 和字体通知；同一 CI 工件的 xdzip-source.tar 提供本项目同提交源码。外层 artifact-files.cdx.json、linux-runtime.json 和 linux-package.json 分别记录文件与已知依赖、隔离容器运行结果、下载 tar.gz 的 SHA-256。完整组件许可归属和独立安全审查仍是发行门槛。

CI 使用没有系统 Qt／编译器的运行容器，网络禁用，运行目录只读，执行 Xvfb 软件渲染启动及十万条自有 ZIP 的打开、取消和退出。该证据不代替物理 GPU、真实桌面、干净安装升级或签名发行验收。
