use crate::{
    output::{Failure, Response, Result},
    terminal,
};
use serde_json::{Value, json};
use std::{collections::VecDeque, sync::mpsc, thread, time::Duration};
use xdzip_runtime::{
    extraction::{self, ConflictPolicy, Snapshot, Status},
    service::{self, Command, Payload},
};
use zeroize::Zeroizing;

pub struct Runtime {
    pub client: service::Client,
    events: mpsc::Receiver<service::Event>,
    join: Option<thread::JoinHandle<()>>,
    tasks: VecDeque<Snapshot>,
    pub session: u64,
    pub format: String,
    pub interactive: bool,
    pub chinese: bool,
    pub secret: Option<Zeroizing<String>>,
    pub secret_from_stdin: bool,
}
impl Runtime {
    pub fn new(interactive: bool, chinese: bool, secret_from_stdin: bool) -> Result<Self> {
        let (client, mut receiver) = service::start()?;
        let (tx, events) = mpsc::sync_channel(4);
        let join = thread::spawn(move || {
            while let Some(event) = receiver.receive() {
                if tx.send(event).is_err() {
                    break;
                }
            }
        });
        Ok(Self {
            client,
            events,
            join: Some(join),
            tasks: VecDeque::new(),
            session: 0,
            format: String::new(),
            interactive,
            chinese,
            secret: None,
            secret_from_stdin,
        })
    }
    fn next(&mut self, request: Option<u64>, task: Option<u64>) -> Result<service::Event> {
        loop {
            if terminal::interrupted() {
                if let Some(id) = request {
                    let _ = self.client.cancel(id);
                }
                if let Some(id) = task {
                    let _ = self.client.cancel_task(id);
                }
            }
            match self.events.recv_timeout(Duration::from_millis(100)) {
                Ok(event) => return Ok(event),
                Err(mpsc::RecvTimeoutError::Timeout) => continue,
                Err(_) => return Err("runtime_closed".into()),
            }
        }
    }
    pub fn call(&mut self, command: Command) -> Result<Vec<Payload>> {
        let id = self
            .client
            .submit_wait(command)
            .map_err(|_| Failure::new("runtime_busy"))?;
        let mut results = vec![];
        loop {
            let event = self.next(Some(id), None)?;
            match event.payload {
                Payload::Task(s) => self.tasks.push_back(*s),
                Payload::Finished(error) if event.request == id => {
                    if error.is_empty() {
                        return Ok(results);
                    }
                    return Err(error.into());
                }
                Payload::RuntimeFault => return Err("runtime_fault".into()),
                payload if event.request == id => results.push(payload),
                _ => {}
            }
        }
    }
    pub fn open(&mut self, path: String) -> Result<()> {
        let mut attempted = false;
        loop {
            let command = if let Some(secret) = &self.secret {
                Command::OpenWithPassword {
                    path: path.clone(),
                    password: extraction::Password::new(secret.to_string())
                        .map_err(Failure::from)?,
                }
            } else {
                Command::Open(path.clone())
            };
            match self.call(command) {
                Ok(pages) => {
                    for page in pages {
                        if let Payload::Page {
                            session, format, ..
                        } = page
                        {
                            self.session = session;
                            self.format = format.label().into();
                        }
                    }
                    return Ok(());
                }
                Err(error)
                    if matches!(
                        error.code.as_str(),
                        "password_required" | "wrong_password" | "password_or_integrity"
                    ) =>
                {
                    if attempted && (self.secret_from_stdin || !self.interactive) {
                        return Err(error);
                    }
                    self.secret = Some(terminal::password(
                        self.secret_from_stdin && !attempted,
                        self.interactive,
                        self.chinese,
                    )?);
                    attempted = true;
                }
                Err(error) => return Err(error),
            }
        }
    }
    pub fn select(
        &mut self,
        members: Vec<String>,
        ids: Vec<u64>,
        directory: String,
    ) -> Result<(Vec<String>, String)> {
        let result = self.call(Command::SelectMembers {
            session: self.session,
            members,
            entry_ids: ids,
            directory,
        })?;
        for payload in result {
            if let Payload::Selected {
                selected,
                directory,
            } = payload
            {
                return Ok((selected, directory));
            }
        }
        Err("invalid_runtime_reply".into())
    }
    pub fn list(&mut self) -> Result<Response> {
        let mut rows = vec![];
        let mut text = String::new();
        let mut offset = 0;
        loop {
            let results = self.call(Command::Catalog {
                session: self.session,
                offset,
            })?;
            let mut total = 0;
            for payload in results {
                if let Payload::Catalog {
                    entries,
                    total: count,
                    ..
                } = payload
                {
                    total = count;
                    offset += entries.len() as u64;
                    for entry in entries {
                        use std::fmt::Write;
                        let _ = writeln!(
                            text,
                            "{:>6} {:>12} {}{}{}",
                            entry.id,
                            entry.size.map_or("?".into(), |n| n.to_string()),
                            if entry.encrypted { "[encrypted] " } else { "" },
                            if entry.warning { "[warning] " } else { "" },
                            entry.display_name
                        );
                        rows.push(json!({"id":entry.id, "path":entry.path, "display_name":entry.display_name, "size":entry.size, "packed_size":entry.packed_size, "method":entry.method, "directory":entry.directory, "link":entry.link, "payload_supported":entry.payload_supported, "encrypted":entry.encrypted, "warning":entry.warning}));
                    }
                }
            }
            if offset >= total {
                break;
            }
        }
        let mut response = Response::success(
            "list",
            json!({"format":self.format,"entries":rows}),
            text.trim_end().into(),
        );
        response.summary = json!({"entries":offset});
        Ok(response)
    }
    pub fn task(&mut self, command: &str, intent: Command) -> Result<Response> {
        let replies = self.call(intent)?;
        let id = replies
            .into_iter()
            .find_map(|p| {
                if let Payload::TaskAccepted(id) = p {
                    Some(id)
                } else {
                    None
                }
            })
            .ok_or("invalid_runtime_reply")?;
        let mut prompt = 0;
        let mut password_prompt = 0;
        let mut password_attempted = false;
        let mut interaction_failure = None;
        let mut last_progress = std::time::Instant::now();
        loop {
            let snapshot = if let Some(snapshot) = self.tasks.pop_front() {
                snapshot
            } else {
                match self.next(None, Some(id))?.payload {
                    Payload::Task(s) => *s,
                    Payload::RuntimeFault => return Err("runtime_fault".into()),
                    _ => continue,
                }
            };
            if snapshot.id != id {
                continue;
            }
            if terminal::interrupted() {
                let _ = self.client.cancel_task(id);
            }
            if self.interactive
                && last_progress.elapsed() >= Duration::from_millis(500)
                && !snapshot.status.terminal()
            {
                eprintln!(
                    "{}: {} bytes read, {} bytes committed",
                    snapshot.status.code(),
                    snapshot.read_bytes,
                    snapshot.committed_bytes
                );
                last_progress = std::time::Instant::now();
            }
            if snapshot.status.terminal() {
                let summary = summary(&snapshot);
                if let Some(code) = interaction_failure {
                    return Err(Failure {
                        summary,
                        ..Failure::new(code)
                    });
                }
                if snapshot.status != Status::Succeeded {
                    let code = if snapshot.status == Status::Cancelled {
                        "cancelled"
                    } else {
                        &snapshot.error
                    };
                    return Err(Failure {
                        summary,
                        ..Failure::new(code)
                    });
                }
                let mut response = Response::success(
                    command,
                    json!({"format":snapshot.format, "output":snapshot.open_path, "check_kind":if matches!(snapshot.format, "TAR" | "ISO" | "UDF") { "structure_and_readability" } else { "supported_content" }}),
                    if self.chinese {
                        format!(
                            "操作完成，已提交 {} 个文件，已跳过 {} 个文件。",
                            snapshot.committed_files, snapshot.skipped_files
                        )
                    } else {
                        format!(
                            "Completed, {} files committed, {} skipped.",
                            snapshot.committed_files, snapshot.skipped_files
                        )
                    },
                );
                if command == "test" {
                    response.text = if self.chinese {
                        if matches!(snapshot.format, "TAR" | "ISO" | "UDF") {
                            "结构与可读性检查完成，不提供逐文件内容校验和保证。".into()
                        } else {
                            format!("测试完成，已校验 {} 个文件。", snapshot.verified_files)
                        }
                    } else if matches!(snapshot.format, "TAR" | "ISO" | "UDF") {
                        "Structure/readability check completed; per-file content checksums are not guaranteed.".into()
                    } else {
                        format!(
                            "Test completed, {} files verified.",
                            snapshot.verified_files
                        )
                    };
                }
                response.summary = summary;
                if snapshot.cleanup_warning || snapshot.outcome_uncertain {
                    response.exit = 1;
                    response.status = "warning".into();
                    response
                        .warnings
                        .push("cleanup_or_outcome_uncertain".into());
                }
                return Ok(response);
            }
            if terminal::interrupted() || interaction_failure.is_some() {
                continue;
            }
            if let Some(ref waiting) = snapshot.prompt
                && waiting.id != prompt
            {
                prompt = waiting.id;
                if !self.interactive {
                    interaction_failure = Some("interaction_required");
                    let _ = self.client.cancel_task(id);
                } else {
                    eprintln!("{}", waiting.name);
                    let answer = terminal::line(
                        if self.chinese {
                            "重名：[s]跳过，[r]改名，[o]覆盖，[c]取消："
                        } else {
                            "Conflict: [s]kip, [r]ename, [o]verwrite, [c]ancel: "
                        },
                        false,
                        32,
                    );
                    let policy = answer.as_ref().ok().and_then(|s| match s.trim() {
                        "s" => Some(ConflictPolicy::Skip),
                        "r" => Some(ConflictPolicy::Rename),
                        "o" if waiting.overwrite_allowed => Some(ConflictPolicy::Overwrite),
                        _ => None,
                    });
                    if let Some(policy) = policy {
                        let _ = self.client.decide_task(id, prompt, policy, false);
                    } else {
                        let _ = self.client.cancel_task(id);
                    }
                }
            }
            if let Some(ref waiting) = snapshot.password_prompt
                && waiting.id != password_prompt
            {
                password_prompt = waiting.id;
                let password = if !password_attempted && let Some(secret) = &self.secret {
                    Ok(secret.clone())
                } else if password_attempted && self.secret_from_stdin {
                    Err(Failure::new("password_or_integrity"))
                } else {
                    terminal::password(
                        self.secret_from_stdin && !password_attempted,
                        self.interactive,
                        self.chinese,
                    )
                };
                password_attempted = true;
                match password.and_then(|mut secret| {
                    extraction::Password::new(std::mem::take(&mut *secret)).map_err(Failure::from)
                }) {
                    Ok(secret) => {
                        let _ = self.client.respond_password(id, password_prompt, secret);
                    }
                    Err(error) => {
                        interaction_failure = Some(if error.code == "password_or_integrity" {
                            "password_or_integrity"
                        } else if error.exit == 130 {
                            "cancelled"
                        } else {
                            "interaction_required"
                        });
                        let _ = self.client.cancel_task(id);
                    }
                }
            }
        }
    }
}
impl Drop for Runtime {
    fn drop(&mut self) {
        self.secret = None;
        self.client.shutdown();
        while let Ok(event) = self.events.recv() {
            if matches!(event.payload, Payload::ShutdownReady) {
                break;
            }
        }
        if let Some(join) = self.join.take() {
            let _ = join.join();
        }
    }
}
fn summary(s: &Snapshot) -> Value {
    json!({"files":s.files, "declared_bytes":s.total_known.then_some(s.declared_bytes), "decoded_bytes":s.decoded_bytes, "read_bytes":s.read_bytes, "verified_files":s.verified_files, "verified_bytes":s.verified_bytes, "committed_files":s.committed_files, "committed_bytes":s.committed_bytes, "skipped_files":s.skipped_files, "created_directories":s.created_directories, "output_parts":s.output_parts, "cleanup_warning":s.cleanup_warning, "outcome_uncertain":s.outcome_uncertain, "leftovers":s.leftovers})
}
