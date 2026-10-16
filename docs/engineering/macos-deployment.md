# 本地 macOS 开发 DMG

用户于 2026-10-16 要求接入 DMG，本地打包作为原有不自动部署约定的明确例外。CI 继续只验证 Apple Silicon 原生构建／启动，不自动打包、签名或上传 Mac 应用。本地脚本不安装应用、不改默认关联、不上传文件，不读取发行凭据。

## 制作与核验

先从干净提交构建 Release，再执行：

```sh
python3 scripts/build-macos-dmg.py --app build/ui-release/apps/desktop/xdzip.app \
  --qt-prefix .cache/Qt/6.8.2/macos --output build/review/xdzip-macos-arm64.dmg
python3 scripts/verify-macos-dmg.py --dmg build/review/xdzip-macos-arm64.dmg \
  --output build/review/macos-dmg-verification
```

输出路径必须位于 build/ 且尚不存在。macdeployqt 扫描实际 QML，部署 Qt frameworks、插件和 QML；补充 Qt 系统控件翻译、项目通知和固定许可材料。保留 framework 内部符号链接，拒绝逃逸应用目录的链接；移除开发机绝对 RPATH，递归检查全部原生导入只解析到应用或系统。构建身份不能为 dirty 或 unknown。

镜像提供 xdzip.app、Applications 链接和安装说明，使用 UDZO 压缩并执行 hdiutil verify。拖入 Applications 前应正常退出旧版，未提供自动更新器或安装中的任务迁移。设置、最近记录位于用户目录，移除应用不自动删除这些数据。

ARM64 的已修改 Mach-O 需要有效代码目录，因此复制／重写后使用 ad-hoc 本地签名，并验证嵌套代码。**这不属于 Developer ID 签名，也未公证，不表示 Gatekeeper 信任或可公开发行。** 不关闭 SIP，不删除用户全局安全策略。

核验脚本只读挂载镜像，核对 Applications 链接，复制到含中文与空格的隔离目录，移除 SDK／QML／插件搜索环境，检查递归原生导入，再以 Cocoa／Metal 启动关闭。随后仅在核验副本修改 QtCore 的 Mach-O 库身份并重新 ad-hoc 签名，验证兼容修改库可加载。该测试证明部署允许修改镜像替换，不冒充从对应源码重建 Qt、任意 ABI 兼容或法律履约批准。

本机仍安装 SDK，导入核验和清除环境不能代替另一台干净 Mac。Finder 拖入、系统打开方式、默认选择、Gatekeeper、真实重启和签名公证继续按[人工清单](../qa/private-test-manual.md)验收。
