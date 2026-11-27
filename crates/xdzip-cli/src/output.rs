use serde::Serialize;
use serde_json::{Value, json};
use std::io::{self, Write};

pub type Result<T> = std::result::Result<T, Failure>;

#[derive(Debug)]
pub struct Failure {
    pub code: String,
    pub exit: i32,
    pub summary: Value,
}
impl Failure {
    pub fn new(code: impl Into<String>) -> Self {
        let code = code.into();
        let exit = match code.as_str() {
            "cancelled" => 130,
            "unsupported_system"
            | "interaction_required"
            | "authorization_required"
            | "manual_install"
            | "desktop_unavailable"
            | "installation_busy"
            | "invalid_installation"
            | "unsafe_installation"
            | "installation_mismatch"
            | "user_directory_unavailable"
            | "command_conflict" => 3,
            _ => 1,
        };
        Self {
            code,
            exit,
            summary: json!({}),
        }
    }
    pub fn usage(code: &str) -> Self {
        Self {
            exit: 2,
            ..Self::new(code)
        }
    }
}
impl From<std::io::Error> for Failure {
    fn from(_: std::io::Error) -> Self {
        Self::new("io_error")
    }
}
impl From<&str> for Failure {
    fn from(code: &str) -> Self {
        Self::new(code)
    }
}
impl From<serde_json::Error> for Failure {
    fn from(_: serde_json::Error) -> Self {
        Self::new("invalid_json")
    }
}

#[derive(Serialize)]
pub struct Response {
    pub schema_version: u32,
    pub command: String,
    pub status: String,
    pub error_code: String,
    pub data: Value,
    pub summary: Value,
    pub warnings: Vec<String>,
    #[serde(skip)]
    pub text: String,
    #[serde(skip)]
    pub exit: i32,
}
impl Response {
    pub fn success(command: &str, data: Value, text: String) -> Self {
        Self {
            schema_version: 1,
            command: command.into(),
            status: "succeeded".into(),
            error_code: String::new(),
            data,
            summary: json!({}),
            warnings: vec![],
            text,
            exit: 0,
        }
    }
    pub fn failure(command: &str, error: Failure) -> Self {
        let status = if error.exit == 130 {
            "cancelled"
        } else if error
            .summary
            .get("committed_files")
            .and_then(Value::as_u64)
            .unwrap_or(0)
            > 0
        {
            "partial"
        } else {
            "failed"
        };
        Self {
            status: status.into(),
            error_code: error.code,
            summary: error.summary,
            exit: error.exit,
            ..Self::success(command, Value::Null, String::new())
        }
    }
    pub fn emit(&self, json_mode: bool, chinese: bool) -> i32 {
        let result = if json_mode {
            serde_json::to_writer(io::stdout().lock(), self)
                .map_err(io::Error::other)
                .and_then(|_| writeln!(io::stdout().lock()))
        } else if self.exit != 0 {
            let mut stderr = io::stderr().lock();
            let printed = if self.error_code.is_empty() {
                writeln!(stderr, "{}", self.text)
            } else {
                writeln!(
                    stderr,
                    "xdzip: {} ({})",
                    explanation(&self.error_code, chinese),
                    self.error_code
                )
            };
            printed.and_then(|_| {
                if self.text.is_empty()
                    && let Some(committed) = self.summary["committed_files"].as_u64()
                {
                    let skipped = self.summary["skipped_files"].as_u64().unwrap_or(0);
                    if chinese {
                        writeln!(
                            stderr,
                            "已提交 {committed} 个文件，已跳过 {skipped} 个文件。"
                        )
                    } else {
                        writeln!(stderr, "{committed} files committed, {skipped} skipped.")
                    }
                } else {
                    Ok(())
                }
            })
        } else {
            writeln!(io::stdout().lock(), "{}", self.text)
        };
        if result.is_err() { 1 } else { self.exit }
    }
}

fn explanation(code: &str, chinese: bool) -> &'static str {
    let (zh, en) = match code {
        "interaction_required" => (
            "需要密码或重名决定，请提供相应选项或在交互终端运行",
            "A password or conflict decision is required; provide options or use an interactive terminal",
        ),
        "manual_install" => (
            "当前安装不能自动替换，请使用检查、下载或管理式安装",
            "This installation requires manual replacement; use check, download or a managed installation",
        ),
        "ambiguous_member" => (
            "存在同名物理记录，请使用 --entry-id 选择",
            "Duplicate members require --entry-id",
        ),
        "member_not_found" | "directory_not_found" => {
            ("包内路径不存在", "The archive member path does not exist")
        }
        "unsupported_feature" => (
            "该格式或归档特性不支持本次操作",
            "The archive format or feature does not support this operation",
        ),
        "entry_name_unrepresentable" => (
            "该归档格式无法原样保存一个或多个输入名称",
            "The archive format cannot store one or more input names exactly",
        ),
        "input_collision" => (
            "多个输入名称在归档内冲突，请先调整名称",
            "Input names conflict in the archive. Rename one before continuing",
        ),
        "name_collision" | "entry_name_exists" => (
            "压缩包内已有相同路径的条目",
            "An entry with the same archive path already exists",
        ),
        "protected_source" => (
            "不能覆盖本次操作的输入文件",
            "An input file cannot be overwritten",
        ),
        "unsafe_target" => (
            "目标目录未通过安全授权，请检查所有权、权限和链接",
            "Destination authorization failed; check ownership, permissions and links",
        ),
        "destination_access_denied" => (
            "当前账户无法在这个位置创建解压文件，请选择可写目录",
            "This account cannot create extraction output here; choose a writable directory",
        ),
        "destination_missing" => (
            "解压目标或其重定向位置不存在，请重新选择目录",
            "The extraction destination or its redirected location does not exist; select a directory again",
        ),
        "destination_type" => (
            "这个位置不是受支持的解压目录，请重新选择",
            "This location is not a supported extraction directory; select a directory again",
        ),
        "target_changed" => (
            "解压目标已变化，请检查目标后重试",
            "The extraction destination changed; check it before retrying",
        ),
        "unsupported_filesystem" => (
            "目标文件系统不支持本次写入，请选择本机磁盘上的其他目录",
            "The destination filesystem cannot publish this output; choose another folder on a local disk",
        ),
        "sfx_stub_missing" | "sfx_stub_invalid" => (
            "自解压组件缺失或损坏，请重新安装完整程序",
            "The self-extraction component is missing or damaged; reinstall the complete application",
        ),
        "command_conflict" => (
            "命令入口已存在或已被修改，原文件保持不变",
            "The command registration exists or has changed; its files were preserved",
        ),
        "cancelled" => (
            "操作已取消，已完成的文件保留",
            "Cancelled; already committed files remain",
        ),
        "source_io" => ("无法读取输入文件", "The input file could not be read"),
        "network" => (
            "无法连接更新服务",
            "The update service could not be reached",
        ),
        "rate_limited" => (
            "更新服务限流，请稍后重试",
            "The update service is rate limited; try again later",
        ),
        _ => (
            "操作未完成，请根据错误码检查输入和当前状态",
            "The operation did not complete; check the error code, inputs and current state",
        ),
    };
    if chinese { zh } else { en }
}

pub fn identity() -> Value {
    json!({"product":"xdzip", "component":"cli", "version":env!("XDZIP_DISPLAY_VERSION"), "build":env!("XDZIP_BUILD").parse::<u32>().unwrap(), "revision":env!("XDZIP_REVISION"), "dirty":env!("XDZIP_DIRTY")=="true", "channel":env!("XDZIP_CHANNEL"), "configuration":env!("XDZIP_CONFIGURATION"), "distribution":env!("XDZIP_DISTRIBUTION"), "platform":std::env::consts::OS, "architecture":std::env::consts::ARCH})
}
pub fn version_text() -> String {
    format!(
        "xdzip {} (build {}, {}{})",
        env!("XDZIP_DISPLAY_VERSION"),
        env!("XDZIP_BUILD"),
        env!("XDZIP_REVISION"),
        if env!("XDZIP_DIRTY") == "true" {
            ", dirty"
        } else {
            ""
        }
    )
}
