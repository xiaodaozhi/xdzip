# 分卷读取与临时打开

## 分卷 ZIP

读取层支持编号连续字节流 `name.zip.001`、`.002` 等，以及 PKZIP 的
`name.z01`、`.z02`、最终 `name.zip`。任意一卷均可作为打开入口。
卷必须在同一目录，名称连续，最多 1024 卷；拒绝缺卷、链接及特殊文件。
文件后缀只用于发现相邻卷组，真实容器仍由来源字节探测并由对应后端完整扫描。

Rust 主机持有各卷的身份、长度和修改时间。逻辑输入按需跨卷读取，不合并
整个归档。PKZIP 的磁盘相对偏移转换为有界的单卷中央目录视图；原有编解码器
继续检查目录、本地头、CRC、密码认证和解压配额。每次使用及提交前重新核验
所有源卷，任何变化均停止任务。所有源卷身份均受输出覆盖保护。

验收覆盖编号卷、PKZIP、ZIP64、跨卷头与负载、缺卷、非连续卷、源替换、
非法磁盘编号、中文名称，以及普通单卷回归。

## 7z 与 RAR 分卷

7z 编号卷使用 name.7z.001、.002 等连续字节流。RAR 使用 name.partN.rar 或 name.rar、name.r00 等旧式卷名，保留 RAR 每个物理卷的边界，不将其当成单一 RAR 字节流交给 UnRAR。两者均可从任意现存卷开始定位同目录卷组；上限 1024 卷，目录发现上限十万项，缺卷、名称编号冲突、链接和重解析点均明确拒绝。

Rust 保存每卷句柄身份、长度和时间，并通过 ReadSource::volume_lengths() 向原生适配器声明授权边界。原生进程只能请求宿主读取已授权范围；换卷不接纳任意归档提供的主机路径。所有卷参与来源复核和输出覆盖保护，单卷或后续卷发生变化均停止任务。

7z 和 RAR 文件名加密可能在打开阶段要求密码。普通 Open 返回 password_required 时不发布索引，Qt 复用密码窗口，以 OpenWithPassword 重新提交原选中文件与受限秘密；错误验证值返回 wrong_password。成功后密码只由会话及其已准入任务持有，关闭会话释放会话副本，不写设置和恢复记录。

## 双击普通文件

目录双击仍进入目录。普通文件双击提交仅包含该物理条目的后台解压任务，
输出到用户私有缓存中的随机目录。链接和特殊对象不能通过此入口打开。
沿用密码、CRC、认证、取消和应用级重任务串行准入；无需用户选择解压目标。

成功完成输出后，通过本地文件 URL 调用系统关联，不经过命令解释器。
只有系统接受打开请求时才自动隐藏对应任务进度窗口。解压失败、取消或系统
拒绝打开时不自动隐藏，并保留可读错误。系统接受请求不等于外部程序已读完
文件，临时文件不能在发出请求时删除。

临时目录使用不可猜测的 128 位随机名称，载荷与身份标记分开。输出载荷目录采用仅当前用户及可信系统主体可访问的权限。Windows 仅对本次独占创建的空目录，通过持有句柄设置受保护 DACL，写出明文前重新检查；不修改用户所选目录的共享权限。POSIX 使用 0700，并检查 macOS 继承 ACL。权限无法保证时拒绝写出，见 [Win32 DACL 接口](https://learn.microsoft.com/en-us/windows/win32/api/aclapi/nf-aclapi-setsecurityinfo)。

持有锁期间不清理，完成后至少保留七天，在后续临时打开时清理过期且未锁定的目录。缓存最多保留 256 项，达到上限明确拒绝；每次扫描上限 4096 项。清理校验私有权限、标记与名称，不遍历链接。临时副本修改不写回归档。

临时文件的清理限定于应用创建的私有缓存，不扫描或删除用户解压目录。
验收覆盖成功自动隐藏、失败保留、取消、密码重试、重复文件名、Unicode 和
特殊字符、关闭页面后任务路由、临时目录安全和退出清理策略。

English: 7z numbered parts form one authorized byte stream. RAR partN and legacy volume names preserve physical boundaries supplied through ReadSource::volume_lengths; UnRAR cannot select arbitrary host paths. Every part is version checked and protected from output overwrite. Encrypted directory opens publish no partial metadata and retain secrets only in their session and admitted tasks.
