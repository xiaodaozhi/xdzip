# 本地 macOS 开发 DMG

本地开发 DMG 是允许的开发交付方式，CI 继续只验证 Apple Silicon 原生构建／启动，不自动打包、签名或上传 Mac 应用。本地脚本不安装应用、不改默认关联、不上传文件，不读取发行凭据。

## 制作与核验

先从干净提交构建 Release，主应用必须显式启用文档候选，再执行：

```sh
cmake -S . -B build/ui-release -G Ninja -DCMAKE_BUILD_TYPE=Release \
  -DXDZIP_BUILD_DESKTOP_PREVIEW=ON -DXDZIP_MACOS_HANDLER_RANK=Alternate \
  -DCMAKE_OSX_ARCHITECTURES=arm64 -DCMAKE_OSX_DEPLOYMENT_TARGET=14.0
cmake --build build/ui-release
python3 scripts/build-macos-dmg.py --app build/ui-release/apps/desktop/xdzip.app \
  --qt-prefix .cache/Qt/6.8.2/macos --output build/review/xdzip-macos-arm64.dmg
python3 scripts/verify-macos-dmg.py --dmg build/review/xdzip-macos-arm64.dmg \
  --output build/review/macos-dmg-verification
```

配置前按[开发构建指南](developer-build.md)设置 Qt 前缀。输出路径必须位于 build/ 且尚不存在。DMG 输入的文档类型必须全部声明 `Viewer/Alternate`，否则要求重新构建，不在原包上改写 plist 或签名。macdeployqt 扫描实际 QML，部署 Qt frameworks、插件和 QML；补充 Qt 系统控件翻译、项目通知、Rust 上游通知、字体许可、源码 SBOM 和固定 Qt 许可材料。保留 framework 内部符号链接，拒绝逃逸应用目录的链接；移除开发机绝对 RPATH，递归检查全部原生导入只解析到应用或系统。构建身份不能为 dirty 或 unknown，必须匹配当前干净提交以绑定实际通知。

Qt 可选 QML LocalStorage 引入 QtSql 后，部署工具会顺带复制数据库驱动。本项目不提供服务器数据库能力，因此移除除内置 SQLite 外的驱动，避免 PostgreSQL、ODBC、Mimer 等 SDK 外部客户端依赖进入镜像；移除列表写入回执，不把 unresolved 导入隐藏为通过。

镜像提供 xdzip.app、Applications 链接和安装说明，使用 UDZO 压缩并执行 hdiutil verify。拖入 Applications 前应正常退出旧版，未提供自动更新器或安装中的任务迁移。设置、最近记录位于用户目录，移除应用不自动删除这些数据。

镜像旁保留部署核验 JSON 和 CycloneDX 文件级 SBOM，绑定同提交源码清单、最终原生文件 SHA-256 和导入关系。SBOM 继续标记 incomplete，不将文件枚举当作精确组件归属或许可批准；清单位于镜像外，避免自包含哈希递归。

打包暂存目录中的新 `xdzip.app` 只在部署、签名、镜像制作及核验期间存在，结束时自动回收，失败时也执行清理。DMG、JSON、SBOM 与说明继续保留，输入主应用不受影响，不留下可被系统再次发现的松散暂存包。

ARM64 的已修改 Mach-O 需要有效代码目录，因此复制／重写后使用 ad-hoc 本地签名，并验证嵌套代码。**这不属于 Developer ID 签名，也未公证，不表示 Gatekeeper 信任。** 按[个人开源发行政策](release-runbook.md)，正式签名与公证不是发行硬性条件，依赖、许可、安全与实际包验证仍须满足。 不关闭 SIP，不删除用户全局安全策略。

核验脚本只读挂载镜像，核对 Applications 链接，复制到含中文与空格的隔离目录，移除 SDK／QML／插件搜索环境，检查递归原生导入，再以 Cocoa／Metal 启动关闭。随后仅在核验副本修改 QtCore 的 Mach-O 库身份并重新 ad-hoc 签名，验证兼容修改库可加载。该测试证明部署允许修改镜像替换，不冒充从对应源码重建 Qt、任意 ABI 兼容或法律履约批准。

本机仍安装 SDK，导入核验和清除环境不能代替另一台干净 Mac。Finder 拖入、系统打开方式、默认选择、Gatekeeper、真实重启和签名公证继续按[人工清单](../qa/private-test-manual.md)验收。

原生菜单编辑命令和 Qt 标准控件需要系统 locale 的 qtbase 翻译。打包除复制 `Contents/Resources/translations/qtbase_*.qm` 外，还须在 `Contents/Resources/qt.conf` 的 `[Paths]` 设置 `Translations = Resources/translations`，保留已有插件与 QML 路径，并在完整性签名前完成写入。重定位启动门槛同时检查原生编辑 responder 和应用菜单角色，不能只在 SDK 开发目录验证。

## 开发与核验副本的打开方式注册

LaunchServices 会记住曾打开过的应用包，构建树中相同 bundle id 的多个副本可能同时出现在系统或 Codex 的“打开方式”菜单。不要为此删除系统数据库、重启 Finder、删除验证包或更改用户默认关联。

新开发包默认在生成 Info.plist 时写入 `LSHandlerRank=None`，仅保留指定主应用和交付包的 `Alternate`。CMake 参数 `XDZIP_MACOS_HANDLER_RANK` 严格限定这两个值；源码离线重建、源码 Qt 产品验证和 Mac 产品 CI 也显式传入 `None`，避免以后更改默认值使测试副本重新进入候选。[Apple 对 `None` 的定义](https://developer.apple.com/library/archive/documentation/General/Reference/InfoPlistKeyReference/Articles/CoreFoundationKeys.html)保留文件拖入能力，不把该包选为此类型的打开应用。这个设置不影响应用内选择文件或显式指定应用的打开流程，不设置默认关联。

Mac 产品 CI 运行 `scripts/check-macos-registration.py`，通过原生 `NSWorkspace` 的文件候选查询检查新构建包。候选数量与按 bundle id 枚举的注册数量分别判断，不能用后者代替用户实际的“打开方式”候选。

`scripts/macos_launch_services.py` 只识别 `org.xdzip.desktop`。DMG 重定位、源码离线重建、源码 Qt 产品验证、QtCore 替换和系统文件打开验证在受限 `build/` 范围内记录运行前的注册集合，结束时（含失败）仅撤销新增的测试副本注册，保留原有主应用及其他产品。清理失败使验证命令失败，不能静默写成成功。

DMG 重定位和 QtCore 替换保留交付包的原始文档声明与校验流程，但只允许复制到尚不存在的 `build/` 内新 `.app` 路径。验证结束或失败后自动回收这次新建的包，先在作用域内冻结所需哈希，JSON、日志及替换前的主包、DMG 和源码 Qt 保留。该清理不接受已有应用或符号链接，不追溯删除历史验证包。单独退注册的旧 `Alternate` 包仍可能被系统或其他应用重新发现并注册，不能保证永久不复发；需要保留旧包时可归入可校验、可还原的备份，再按明确授权处理松散副本。

历史残留用以下工具预览，再显式应用；`--keep` 必须指定仍存在且身份正确的主应用，可以是当前开发构建或 Applications 中的安装版：

```sh
python3 scripts/cleanup-macos-open-with.py --keep build/ui-release/apps/desktop/xdzip.app
python3 scripts/cleanup-macos-open-with.py --keep build/ui-release/apps/desktop/xdzip.app --apply
```

工具只撤销本仓库 `build/` 内其他 xdzip 副本及同身份失效路径的注册，不删除包、修改签名或设置默认 ZIP 应用；不同产品、Applications 中的其他安装和构建树外路径不自动处理。解析符号链接后越界的路径不清理。完成后重新核对注册集合，只保留指定主开发包；构建树外有多个安装时单独列为未处理，不冒充全局已去重。

具体清理进度、数量与运行结论记录在对应 issue，例如[开发副本维护 #7](https://github.com/xiaodaozhi/xdzip/issues/7)，生成 JSON 和原生探针结果仅存本地／CI 工件。
