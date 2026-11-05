# Qt 6.8.2 runtime notices

xdzip dynamically links Qt 6.8.2 Core, Gui, Network, Qml, Quick, QuickControls2 and Widgets, together with the QML modules and platform plugins needed by the application. Qt copyright belongs to The Qt Company Ltd. and other contributors.

Qt is available under commercial terms or applicable LGPL/GPL terms. For the runtime modules listed above, this distribution uses the LGPL-3.0 option. The LGPL-3.0 and its referenced GPL-3.0 license texts accompany this notice. Qt's bundled third-party components may have different licenses; their source declarations and license texts are provided in `Qt-runtime/` within the same notices directory. Qt build and test tools are not application runtime features.

This software is based in part on the work of the FreeType Team.

This software is based in part on the work of the Independent JPEG Group.

This software includes code derived from Imlib2.

Qt's original source archive: https://download.qt.io/archive/qt/6.8/6.8.2/single/qt-everywhere-src-6.8.2.tar.xz

Version-specific corresponding-source materials, including the application source, fixed Qt module archives, applicable patches and rebuild instructions, are provided through the project release channel: https://github.com/xiaodaozhi/xdzip/releases

Qt licensing: https://doc.qt.io/qt-6.8/licensing.html

Qt third-party licenses: https://doc.qt.io/qt-6.8/licenses-used-in-qt.html

For the 54 ordinary Qt 6.8.2 translation source files used by the published alpha.8 catalogs, xdzip selects BSD-3-Clause based on Qt's published translation policy and the identical files in its 6.8 maintenance tree after that policy update. The unchanged upstream license text is in `../Qt-runtime/license-texts/BSD-3-Clause.txt`; its year and owner placeholders are retained as supplied. This choice does not assign absent per-file copyright notices or change the upstream SPDX `NOASSERTION` record. Review any different translation inputs separately. Evidence and limits: https://github.com/xiaodaozhi/xdzip/issues/53#issuecomment-6572666231

Qt's locale-data generator refers to the ISO 639-3 code tables maintained by SIL: https://iso639-3.sil.org/. Qt's generated locale data includes language-code entries. Consult the code-table terms separately: https://iso639-3.sil.org/code_tables/download_tables. This attribution does not establish permission to redistribute the full tables.

The application uses shared Qt libraries. The corresponding-source materials describe how to rebuild the application and use an interface-compatible modified Qt. A modified macOS bundle may require a new local ad-hoc integrity signature; xdzip does not require the user's possession of the project's private signing key or impose a restriction on debugging modifications to the Qt libraries.

These notices identify the applicable licensing and source materials; they do not by themselves establish that every third-party source or redistribution obligation has been satisfied.

# Qt 6.8.2 运行时许可说明

xdzip 动态链接 Qt 6.8.2 的 Core、Gui、Network、Qml、Quick、QuickControls2、Widgets，以及应用所需的 QML 模块和平台插件。Qt 版权属于 The Qt Company Ltd. 及其他贡献者。上述运行时模块采用 LGPL-3.0 分支，同目录附带 LGPL-3.0 和其引用的 GPL-3.0 许可原文。Qt 内嵌的第三方组件可能使用其他许可，同一 notices 目录下的 `Qt-runtime/` 提供对应声明和许可文本。

本软件部分基于 FreeType 团队的工作。

本软件部分基于 Independent JPEG Group 的工作。

本软件包含源自 Imlib2 的代码。

上方列出 Qt 原始源码、项目发行页和许可入口。对应版本的源码材料包括应用源码、固定 Qt 模块归档、适用补丁和重建说明。用户可以构建并使用接口兼容的修改版 Qt；修改 macOS 应用副本后可能需要重新进行本地 ad-hoc 完整性签名，xdzip 不要求用户持有项目的私有签名密钥，也不限制为调试 Qt 修改而进行的逆向工程。本说明不能单独证明所有第三方源码及再分发义务已经完成。

Qt 区域数据生成器引用由 SIL 维护的 ISO 639-3 代码表，Qt 生成的区域数据包含语言代码条目，来源为 https://iso639-3.sil.org/。代码表条款须另见 https://iso639-3.sil.org/code_tables/download_tables；本署名不表示已获完整表格再分发许可。

对已公开 alpha.8 翻译目录所用的 54 份普通 Qt 6.8.2 翻译源码，xdzip 根据 Qt 公开的翻译许可说明以及政策更新后 6.8 维护树中逐字节相同的文件，选择 BSD-3-Clause 分支。未修改的上游许可原文位于 `../Qt-runtime/license-texts/BSD-3-Clause.txt`；其中年份和权利人占位符按上游原样保留。本选择不补造缺失的逐文件版权声明，也不修改上游 SPDX 的 `NOASSERTION` 记录。其他翻译输入须另行复核；证据及限制见 https://github.com/xiaodaozhi/xdzip/issues/53#issuecomment-6572666231。
