# M9 候选格式资料和许可门槛

本记录固定 Bandizip 清单中下一阶段候选格式。资料只保留在 `build/toolchain/references`，或链接公开规范，不作为产品依赖。

## WIM

- 参考实现：`ebiggers/wimlib`，提交 `cd5e231c348c255ae5088873b5a66ee0eb96fa07`。
- 仓库对项目提供 GPLv3+，对库部分提供有条件的 LGPL-2.1+ 选择。LGPL 选择取决于 `libntfs-3g` 等第三方依赖没有把最终组合强制到 GPL 条件。
- WIM 是镜像容器，不是普通归档。受控读取器需要处理资源表、元数据流、分块资源、Xpress/LZX/LZMS 压缩、完整性表、硬链接元数据和镜像选择，同时不能挂载或恢复安全权限。

参考实现可用于线格式和样本研究，但按当前不引入第三方归档运行时的政策不能链接进 xdzip。只有在具备净室 Rust 读取器、独立生成的完整样本以及覆盖解压放大和镜像选择的资源模型后，WIM 才能从候选进入准入。

## Compound File 与 MSI

- Microsoft [MS-CFB] 是主要公开规范：https://learn.microsoft.com/en-us/openspecs/windows_protocols/ms-cfb/53989ce4-7b05-4f8d-829b-d08d6148375b 。页面发布 PDF/DOCX 版本，并包含允许实现所述技术的知识产权说明，专利许可仍受适用 Microsoft 计划约束。
- CFB 读取器需要在解释 MSI 之前完成扇区、FAT/DIFAT、目录红黑树、mini-stream 分配、流长度边界和环检测。

独立 CFB 元数据读取器技术上可行，当前已实现为 `xdzip-codec-cfb` 只读切片。它会在发布存储和流记录前验证 CFB 头部、FAT/DIFAT 链、目录树、mini-stream 分配、流长度边界和环。只有不可变元数据快照匹配后才允许复制原始流。MSI 表、内嵌 CAB、自定义动作和变换仍不在契约中，不执行安装器、自定义动作，不解释安装包脚本，也不回写，`.msi` 后缀不能授予执行权限。codec 和 runtime 的夹具及运行证据均为自动化测试。

## NSIS 与其他 SFX

- NSIS 参考源码：`kichik/nsis`，提交 `a96303eae3115221471bb127fa916a13d8bf9ecd`。
- `COPYRIGHT` 记录大部分源码和 zlib 压缩模块使用 zlib/libpng，bzip2 模块使用 bzip2 许可，LZMA 模块使用 Common Public License 1.0。
- NSIS 文件是带脚本和内嵌载荷语义的可执行安装器。解析器必须停留在执行边界之下，限制头部和载荷跨度，并把压缩方法与安装器行为分开。

NSIS 可以在完成独立头部/解析器和完整自有样本后推进只读元数据浏览。打开、检查或解压 NSIS 文件时绝不运行 stub、脚本、自定义页面、插件或提取出的载荷。其他 SFX 仍只能在内部归档通过内容验证时作为别名处理。

## 准入门槛

1. 使用参考工具前固定源码提交和许可文本哈希。
2. 每个准入容器和压缩方法都要有完整、可再分发样本，只有头部样本不够。
3. CFB、WIM、NSIS 解析必须独立于 Qt，也不能调用安装、挂载、注册表或脚本执行 API。
4. 只有在准确物理身份、重复项、环边界、累计输入／工作／输出限额、取消、来源变化和终止失败测试通过后，才接入 runtime、CLI 和桌面入口。
5. GPL/LGPL/CPL 参考组件继续保留在 `build/toolchain`，除非另有依赖与许可决策明确改变产品政策。
