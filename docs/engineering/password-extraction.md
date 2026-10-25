# 压缩包密码与加密目录打开

本契约补充[安全解压](m3-extraction.md)，定义密码、认证和任务交互。格式与文件安全逻辑仍独立于 Qt，密码窗口只提交用户意图。实际开发进度与平台验证见 [issue #11](https://github.com/xiaodaozhi/xdzip/issues/11)，本页不保存执行报告。

## 能力与格式边界

支持 ZIP／ZIP64 的 WinZip AES-256，AE-1 和 AE-2，实际压缩方法为 Store 或 Deflate。可以选择明文、加密及混合条目，空文件也须验证认证码；各文件密码可以不同。AES-128／192、传统 ZipCrypto、其他加密方式仍不支持；受支持分卷使用[统一逻辑输入](archive-opening.md)，密码和认证规则不变。文件名、目录结构和普通 ZIP 元数据可见，密码功能不为它们提供加密或真实性保证。

AES 字段遵循 [WinZip 格式规范](https://www.winzip.com/en/support/aes-encryption/)：标记方法为 99、加密位开启，唯一的 AES extra 使用 AE vendor、已知版本、256-bit strength 和受支持的实际方法。当前只接受 7 字节已定义数据，未知扩展保持拒绝。AE-1 另外核对解码内容 CRC，AE-2 的存储 CRC 必须为零。中央目录、本地头、数据描述符的声明一致性仍独立检查，不以认证码代替结构验证。

格式适配器先对照冻结快照的物理记录身份、原始名称、类型、大小、方法、加密位和密码支持标记，再为本次任务记录完整 flags 与 AES profile，验证本地头、中央目录、描述符与负载范围的一致性。最初快照不保存完整 flags／profile，来源变化仍依赖既有身份与版本的尽力检测，不构成输入内容的原子快照。局部或中央目录的 ZIP64 大小哨兵均参与描述符宽度判定。ZIP 容器与 AES 封装由项目独立实现，密码学原语使用[已准入基础库](dependencies.md)，不调用格式库的解包功能。

解密器只读取已经验证的单条密文范围，不接收输出路径。Store 或共享 Deflate 裸流解码器核对精确压缩字节消费、解码大小及 AE-1 CRC，密文完整消费后核对 HMAC 认证码，全部成功才返回成功。空文件同样完成认证。密文和解码数据使用有界缓冲，不将整文件累积在内存；不再构造供第三方 ZIP 库读取的私有归档视图。

## 7z、RAR 与加密目录

7z 使用官方 7-Zip 加密实现，RAR 使用官方 UnRAR 的历史版本和当前版本密码处理。两者沿用相同的宿主解码接收器、安全输出、实际配额及错误分类。无法确认为密码验证值错误的校验损坏按完整性失败处理，不能无限重试。新旧格式、头部加密、固实和分卷的组合单独验收，具体后端和资源边界见[官方后端契约](native-codecs.md)。

文件名或目录加密在元数据扫描前要求秘密，OpenWithPassword 是唯一携带该秘密的打开命令，密码容器在入队前有界并归零释放，事件不包含秘密。完整扫描成功前没有部分可浏览索引。成功后会话保留零化容器，已准入任务可克隆一个私有副本以便关闭页面后继续；副本不进入快照、日志或设置。任务解码器重开时使用同一密码，随后条目密码仍通过独立提示控制槽响应。

7z 创建可同时设置内容密码和文件名加密；文件名加密要求非空创建密码，仅 7z 可设置。ZIP 保持仅内容加密，TAR／TAR.GZ 拒绝加密参数。加密 7z 和所有 RAR 不接纳修改。

## 秘密与控制协议

`extraction::Password` 与创建密码类型分开。解压接受原样 UTF-8，最多 1,024 字节，允许空值、空格及控制字符，不修剪或套用创建的 1–128 字节规则；非 UTF-8 原始密码不由当前文本界面表达。输入分配容量同样有界。Rust 所有权容器在 Debug 中固定脱敏，在拒绝、替换、取消或离开作用域时归零。

密码由 `Client::respond_password(task, prompt, Password)` 通过独立控制槽传入，不进入普通命令队列。每个提示使用任务内单次有效的 ID，并区分重名与密码两种提示；旧 ID、跨类型响应、重复响应、终态或取消后的响应均拒绝。快照只携带 `PasswordPrompt { id, name, retry }` 与 `waiting_password` 状态，不含密码或密码摘要。GUI mailbox 对提示名称继续计入字节限额。

第一次需要密码时暂停当前任务，等待显式响应；任务内成功使用的密码可用于后续加密条目。验证值不匹配时清除旧缓存、给出新提示，最多每物理条目十次失败，达到上限停止任务。认证码不匹配按内容完整性失败停止，不自动变成无限密码重试。该区分不能保证所有错误密码均被两字节验证值排除，最终认证仍为必需条件。

取消和退出不依赖普通队列剩余容量，唤醒密码等待并清空未消费的秘密。内容密码只在任务生命周期使用，加密目录密码限定于打开会话及其已准入任务；均不写入设置、事件、日志、命令行、恢复记录或测试截图。Qt 输入和受控传输缓冲在提交、关闭、取消、换提示及退出时清空；Qt、系统输入法及依赖内部的历史副本不保证全进程擦除，Rust 容器归零不作此保证。

## 输出、重试与链接

普通文件每次尝试均使用新的私有暂存文件。错误密码、认证失败、流结构错误或取消会退出此次暂存的生命周期，尝试按身份删除；删除失败时最终 workspace 清理报告警告并保留恢复材料，不能承诺立即删除。完整校验、同步、来源复核和提交许可全部通过后，才按既有策略发布或替换文件，未认证暂存不会作为目标文件发布。已存在的目标在认证失败时保持不变。重试产生的实际解码／写入计数保留，继续受单文件 8 GiB、任务 20 GiB 与源读取限额约束，不能通过重试重置预算。

ZIP 任务建立解码器时有 64 MiB 的实际元数据读取预算，随后切换至共享的 20 GiB 输入读取预算，本地头、描述符、密文及所有密码重试都消费同一计数。该输入读取量不同于源文件长度或已保存输出量，超限停止，不在重试时重置。

一个任务可以在后续文件失败前已经提交早前验证通过的文件；仍不承诺整包事务回滚。重名跳过无需读取相应文件内容，不能将跳过记为已认证。取消后已提交项、清理警告和计数不确定性沿用安全解压契约。

若选中加密符号链接，全部选中链接的负载与依赖图在任务 Preparing 阶段、取得输出授权及创建派生目录前完成认证和校验，再先发布普通文件、最后发布链接。路径、4,096 字节单链接／4 MiB 合计、64 次跟随、闭合依赖图、无写入父链接及占用叶子拒绝等策略不放宽。POSIX 执行权限仍仅恢复已准入的安全位；Windows ZIP 链接和 TAR 链接保持不支持。

## 界面与验证

密码窗口及 HTML、高保真依据见[密码交互](../ux/password-extraction.md)。任务等待时自动显示应用模态窗口，也可从任务窗口重新显示。错误响应使用新 token，清空旧输入；接受响应后防止重复提交。系统关闭或取消按钮取消当前任务，Escape 继续遵循共享的先退出控件焦点、再关闭窗口规则。

验收要求覆盖原始 AES 字段／CRC／描述符／ZIP64、同名物理记录、正确和错误密码、认证破坏、混合文件、不同密码、空密码／控制字符、重试资源累计、满队列、取消／退出唤醒、来源变化、旧目标保护、链接和执行权限，以及独立 7-Zip 双向字节核对。界面检查分别覆盖实际控制器闭环、可访问动作、双语多字号、浅深色、DPR 与原生窗口。自动化不等于系统读屏人工验收。

English: Official 7-Zip and UnRAR handle 7z/RAR passwords. Encrypted directories require a bounded OpenWithPassword request before any metadata is published. The private session and its admitted tasks retain zeroizing secret containers; events, settings and logs never contain passwords. Existing task prompts and cancellation remain in use. Only 7z creation supports encrypted filenames, requiring a creation password. Encrypted 7z and all RAR archives reject edits.

旧式 RAR 加密条目缺少可可靠区分错误密码与损坏内容的验证值，因此每个加密条目解码前分别请求密码，不能把前一条密码直接套用到可能使用不同秘密的下一条。RAR5 和受支持的其他格式继续任务内复用。旧 RAR 的 CRC 或头部损坏仍作为完整性失败，不冒充已确认的错误密码。

English: Older RAR encrypted records request a password separately for each entry because their checks cannot reliably distinguish a wrong cached secret from corruption. RAR5 retains task-local reuse. Legacy CRC or header damage remains an integrity failure, never a claimed password-verifier mismatch.
