use clap::{Arg, ArgAction, Command};

fn flag(name: &'static str, help: &'static str) -> Arg {
    Arg::new(name)
        .long(name)
        .action(ArgAction::SetTrue)
        .help(help)
}
fn value(name: &'static str, help: &'static str) -> Arg {
    Arg::new(name).long(name).help(help)
}
fn archive() -> Arg {
    Arg::new("archive")
        .required(true)
        .help("压缩包路径 / Archive path")
}
fn inputs() -> Arg {
    Arg::new("inputs")
        .required(true)
        .num_args(1..)
        .help("输入文件或目录 / Input files or directories")
}
fn conflict() -> Arg {
    value("conflict", "重名策略 / Existing-file policy")
        .value_parser(["ask", "skip", "rename", "overwrite"])
        .default_value("ask")
}
pub fn command(chinese: bool) -> Command {
    let description = if chinese {
        "xdzip 命令行压缩工具\n不带参数显示帮助。包内路径使用正斜杠。密码不会作为命令行参数接收。"
    } else {
        "xdzip command-line archive tool\nNo arguments shows help. Member paths use forward slashes. Password values are never accepted in arguments."
    };
    let command = Command::new("xdzip").about(description).disable_version_flag(true)
        .arg(flag("version", "查询版本 / Print version").short('V'))
        .arg(flag("version-json", "构建身份 JSON / Build identity JSON"))
        .arg(flag("json", "机器可读结果，禁用交互 / JSON result, no interaction").global(true))
        .arg(flag("non-interactive", "不询问输入 / Never prompt").global(true))
        .arg(value("language", "帮助语言 / Help language").value_parser(["zh-CN", "en-US"]).global(true))
        .subcommand(Command::new("version").about("查询版本与构建身份 / Version and build identity"))
        .subcommand(Command::new("list").visible_alias("l").about("列举压缩包条目 / List physical archive entries").arg(archive()).arg(flag("password-stdin", "从标准输入读取密码 / Read password from stdin"))
            .arg(Arg::new("offset").long("offset").value_parser(clap::value_parser!(u64).range(0..=1_000_000)).help("物理行偏移，从 0 开始 / Physical row offset, starting at 0"))
            .arg(Arg::new("limit").long("limit").value_parser(clap::value_parser!(u64).range(1..=4096)).help("分页条目数 1–4096 / Page size 1–4096")))
        .subcommand(Command::new("extract").visible_alias("x").about("安全解压 / Safely extract files").arg(archive())
            .arg(Arg::new("members").num_args(0..).help("精确包内路径 / Exact member paths"))
            .arg(value("directory", "目标目录 / Destination directory").short('C').default_value("."))
            .arg(value("entry-id", "物理条目 ID / Physical entry ID").action(ArgAction::Append).value_parser(clap::value_parser!(u64)))
            .arg(conflict()).arg(flag("password-stdin", "从标准输入读取密码 / Read password from stdin"))
            .arg(flag("allow-high-ratio", "允许高压缩比，保留硬限制 / Allow high ratios within hard limits")))
        .subcommand(Command::new("create").visible_alias("c").about("创建压缩包 / Create an archive").arg(archive()).arg(inputs()).arg(conflict())
            .arg(value("level", "压缩级别 / Compression level").value_parser(["store", "fast", "normal", "best"]).default_value("normal"))
            .arg(flag("encrypt", "加密文件内容 / Encrypt contents"))
            .arg(flag("encrypt-names", "7z 文件名加密 / Encrypt 7z filenames").requires("encrypt"))
            .arg(flag("solid", "7z 固实压缩 / Solid 7z archive"))
            .arg(flag("password-stdin", "从标准输入读取密码 / Read password from stdin").requires("encrypt"))
            .arg(value("volume-size", "分卷字节数，可用 KiB/MiB/GiB / Volume size")))
        .subcommand(Command::new("add").visible_alias("a").about("添加文件到普通压缩包 / Add files to an ordinary archive").arg(archive()).arg(inputs()).arg(conflict()).arg(value("into", "已有包内目录 / Existing member directory").default_value("")))
        .subcommand(Command::new("test").visible_alias("t").about("测试压缩包 / Test an archive").arg(archive()).arg(flag("password-stdin", "从标准输入读取密码 / Read password from stdin")))
        .subcommand(Command::new("gui").about("打开桌面版 / Open the desktop application").arg(Arg::new("archives").num_args(0..)))
        .subcommand(Command::new("setup").about("显式安装或移除用户级命令入口 / Register or remove the user command")
            .arg(flag("remove", "移除本工具创建的命令入口 / Remove an owned registration"))
            .arg(value("bin-dir", "用户命令目录 / User command directory"))
            .arg(value("install-dir", "独立 CLI 管理目录 / Managed CLI installation directory")))
        .subcommand(Command::new("update").about("检查、下载或更新当前安装 / Update the current installation")
            .arg(flag("check", "仅检查 / Check only").conflicts_with_all(["download", "status"]))
            .arg(flag("download", "仅下载并验证 / Download and verify").conflicts_with("status"))
            .arg(flag("status", "查询实际安装回执 / Read installation receipt"))
            .arg(flag("yes", "确认本次更新，不替代系统授权 / Confirm this update"))
            .arg(value("output", "已验证包的保存目录 / Directory for verified downloads"))
            .arg(value("proxy", "无凭据的 HTTP/SOCKS5 代理 / Credential-free proxy").conflicts_with("no-proxy"))
            .arg(flag("no-proxy", "直连 / Direct connection"))
            .arg(value("mirror", "HTTPS 镜像前缀 / HTTPS mirror prefix")))
        .after_help(if chinese { "示例：\n  xdzip list archive.zip\n  xdzip extract archive.zip docs/ -C output\n  xdzip create backup.7z docs/ --level normal\n  xdzip add backup.zip notes.txt\n  xdzip test archive.zip --json\n  xdzip update --check" } else { "Examples:\n  xdzip list archive.zip\n  xdzip extract archive.zip docs/ -C output\n  xdzip create backup.7z docs/ --level normal\n  xdzip add backup.zip notes.txt\n  xdzip test archive.zip --json\n  xdzip update --check" });
    localize(command, chinese)
}

fn localize(command: Command, chinese: bool) -> Command {
    let command = command
        .mut_args(|arg| {
            let help = arg.get_help().map(ToString::to_string);
            if let Some(help) = help.and_then(|s| {
                s.split_once(" / ").map(|(zh, en)| {
                    if chinese {
                        zh.to_owned()
                    } else {
                        en.to_owned()
                    }
                })
            }) {
                arg.help(help)
            } else {
                arg
            }
        })
        .mut_subcommands(|sub| {
            let about = sub.get_about().map(ToString::to_string);
            let sub = if let Some(about) = about.and_then(|s| {
                s.split_once(" / ").map(|(zh, en)| {
                    if chinese {
                        zh.to_owned()
                    } else {
                        en.to_owned()
                    }
                })
            }) {
                sub.about(about)
            } else {
                sub
            };
            localize(sub, chinese)
        });
    if chinese {
        command
            .help_template("{about-with-newline}\n用法：{usage}\n\n{all-args}{after-help}")
            .next_help_heading("选项")
    } else {
        command
    }
}
