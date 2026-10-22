# 原生运行库来源与通知

本契约覆盖实际包中的非 Qt 原生运行库，与[运行时归属](runtime-attribution.md)及[对应源码](source-delivery.md)共同使用。源版本、文件身份、许可条件和实际履约分别核对，不能将供应方文件哈希当作许可批准。候选结论和剩余补证登记在 [#4](https://github.com/xiaodaozhi/xdzip/issues/4)，证据只生成到 build/ 或 CI 工件。

| 组件 | 原始依据与交付材料 | 核对边界 |
| --- | --- | --- |
| Qt 6.8.2 | SDK SPDX、模块原始源码和补丁、Qt 通知、LGPLv3 及 GPLv3 原文、应用重建和兼容替换说明 | 运行时采用 LGPL 分支，工具许可与第三方许可分别保留；SDK 文件对应不等于完整上游构建等价 |
| Rust 运行时代码 | Cargo.lock、实际链接输入、运行依赖候选、完整 vendor 和原始通知、Rust 标准库源码 | 保守交付构建图的通知，不把 build/dev/proc-macro 误报为已链接运行时代码 |
| Noto Sans SC 衍生字体 | 字体 manifest、原始 OFL、改名后的字体和生成入口 | 字体不适用项目 Apache-2.0，保留修改及命名说明 |
| Mesa 11.2.2、LLVM 3.6.2 | 固定 Qt 官方预编译文件哈希、原始源码归档、Mesa 许可页、LLVM 原始许可及源文件通知 | DLL 内版本字符串与来源归档分别登记；没有供应方完整构建输入时保留限制，不臆造补丁或构建等价 |
| ICU 73.2 | 实际库版本、完整 ICU 上游 LICENSE（含第三方通知）、来源记录 | 不以通用 MIT 文本替代 ICU 自带通知，不把系统包装器误当实际打包文件 |
| Microsoft VC++ runtime | 固定官方 VC_redist 包及 DLL 哈希、原始许可、版本及分发清单 | 逐文件保持未修改；维护者的适用许可资格及分发条件须另行核对 |
| Windows SDK 图形编译库 | 显式 Windows SDK REDIST 输入、逐 DLL 哈希、SDK 版本、原始 SDK 许可及可用第三方通知 | 不从 PATH 或 System32 寻找；SDK 许可与可分发清单仍须覆盖这些文件 |
| Inno Setup 6.7.3 安装引擎 | 固定编译器包及来源提交、Inno 原始许可、RemObjects Pascal Script 原始许可和双语 Installer-CREDITS.txt | 保留安装引擎既有版权界面；Pascal Script 另有在 About 或文档中显示名称及来源地址的要求，不能只交付 Inno 总许可 |

Microsoft 的[运行库分发说明](https://learn.microsoft.com/en-us/cpp/windows/redistributing-visual-cpp-files)与 [VS 2022 可分发清单](https://learn.microsoft.com/en-us/visualstudio/releases/2022/redistribution)将分发权与适用软件许可关联。安装许可文本不是维护者已满足资格的证明。Qt 的[开源义务说明](https://www.qt.io/development/open-source-lgpl-obligations)要求保留用户取得对应源码、修改、重新链接和运行修改库的权利；不能以“动态链接”代替其他义务。

## 生成与验证

`scripts/native-runtime-sources.json` 固定 Mesa、LLVM 原始源码及 ICU 许可的版本、官方 URL 和 SHA-256。`collect-notices.py` 在 Windows 收集 Mesa 和 LLVM 原始通知，附带标明范围的源码通知参考集合；在 Linux 收集 ICU 完整许可。引用集合可能包括未使用驱动和编译目标，不自动选择许可或声称全部代码都已链接。缓存和下载均重新核对哈希，损坏缓存、缺失原文及越界成员失败，不解包任意上游路径。

`prepare-source-delivery.py` 同时附带 Mesa、LLVM 原始归档、ICU 许可及固定来源清单。材料保持原始字节，交付总清单重新计算大小和哈希。源码归档的存在不能补造供应方配置、补丁或编译历史。

平台 materials 附件必须包含归属报告 `license_materials_base` 引用的目录，逐个复核许可原文哈希后再打包；不能仅因运行目录中另有相同原文，就交付指向缺失相对目录的报告。

Windows 部署禁用 windeployqt 对 D3D 编译器和 DXC 的环境搜索，从当前显式选择的 Windows SDK `Redist/D3D/x64` 复制三个库，从固定 Qt SDK 复制已经核对的 Mesa DLL。缺少 SDK 版本、许可、所需 DLL 或 Mesa 哈希不符时失败。记录只含 SDK 相对路径，不泄露开发机目录。四个图形兼容库继续保留。

macOS 打包在最终签名应用仍存在时输出 `deployment-audit.json`、`macos-deployment.json`、`runtime-attribution.json`，文件 SBOM 使用同一审计及归属报告。Qt 原生文件按 arm64 分片、全部 section 字节、UUID 和导入版本比较，允许部署需要的 Qt 路径重定位；签名、load-command 布局及 LINKEDIT 不属于该比较结论。非原生资源按精确字节关联。未关联文件和原始 SPDX 缺口继续保留，不能将这个过程宣称为完整二进制等价。

公开包发现缺失通知后，补充材料不能改写为“原包已包含”。原版本资产及校验和保持不可变，修订后的包必须使用新版本并重新验证；补充材料及其绑定对象应明确登记。

English: Native runtime provenance, license conditions and delivered obligations are separate checks. Windows graphics libraries come only from the explicitly selected SDK REDIST directory and the hash-reviewed Qt Mesa input, with supplier notices and relative-path provenance. Mesa/LLVM source archives and original notices, plus the full ICU license, are collected with pinned hashes. Final macOS attribution binds the signed deployed app to SDK inputs using arm64 sections, UUID and imports, while explicitly excluding signatures, load-command layout and LINKEDIT from equivalence claims. Unknown origins and licensee eligibility remain open review items. Published assets are immutable; corrected packages require a new version.
