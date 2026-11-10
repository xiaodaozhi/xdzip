use crate::{
    installation::{self, Context},
    output::{Failure, Response, Result},
    terminal,
};
use serde_json::{Value, json};
use std::{
    path::PathBuf,
    process::{Command, Stdio},
    time::{Duration, Instant},
};

fn executable() -> Result<PathBuf> {
    if let Context::Desktop(gui) = installation::context()? {
        return Ok(gui);
    }
    // A separately installed CLI can use an explicitly registered desktop,
    // but never guesses a GUI executable from PATH or a downloaded response.
    let registration = installation::data_root()?.join("desktop.json");
    if registration.try_exists()? {
        let record = installation::read_json(&registration)?;
        let gui = PathBuf::from(
            record["executable"]
                .as_str()
                .ok_or("invalid_installation")?,
        );
        if gui.is_absolute() && gui.is_file() {
            return Ok(gui);
        }
    }
    Err("desktop_unavailable".into())
}
pub fn launch(paths: &[String]) -> Result<Response> {
    let gui = executable()?;
    let paths = paths
        .iter()
        .map(crate::absolute)
        .collect::<Result<Vec<_>>>()?;
    Command::new(gui)
        .arg("--")
        .args(paths)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()?;
    let mut result = Response::success("gui", json!({}), "Desktop launch handed off.".into());
    result.status = "handed_off".into();
    Ok(result)
}
pub fn installation_info() -> Result<Value> {
    let actual = Command::new(executable()?)
        .arg("--cli-installation-json")
        .output()?;
    if !actual.status.success() || actual.stdout.len() > 16384 {
        return Err("invalid_desktop_reply".into());
    }
    let info: Value = serde_json::from_slice(&actual.stdout)?;
    if info["schema_version"] != 1
        || info["identity"]["product"] != "xdzip"
        || info["identity"]["version"] != env!("XDZIP_DISPLAY_VERSION")
        || info["identity"]["build"].as_u64() != Some(env!("XDZIP_BUILD").parse().unwrap())
        || info["identity"]["revision"] != env!("XDZIP_REVISION")
        || info["identity"]["dirty"].as_bool() != Some(env!("XDZIP_DIRTY") == "true")
    {
        return Err("installation_mismatch".into());
    }
    Ok(info)
}
pub fn request(action: &str, candidate: Option<&str>) -> Result<Response> {
    let gui = executable()?;
    let root = installation::cache_root()?;
    let directory =
        crate::ensure_directory(&root.join(format!("desktop-{}", installation::random_id()?)))?;
    let request =
        json!({"schema_version":1,"action":action,"candidate_version":candidate.unwrap_or("")});
    installation::write_json(&directory.join("request.json"), &request, false)?;
    Command::new(gui)
        .arg("--cli-request")
        .arg(&directory)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()?;
    let deadline = Instant::now() + Duration::from_secs(20);
    while Instant::now() < deadline {
        if terminal::interrupted() {
            return Err("cancelled".into());
        }
        if directory.join("result.json").try_exists()? {
            let value = installation::read_json(&directory.join("result.json"))?;
            let error = value["error_code"]
                .as_str()
                .unwrap_or("invalid_desktop_reply");
            if !error.is_empty() {
                return Err(error.into());
            }
            let status = value["status"].as_str().ok_or("invalid_desktop_reply")?;
            if ![
                "handed_off",
                "installed",
                "recovery",
                "idle",
                "up_to_date",
                "checking",
                "downloading",
                "waiting_tasks",
                "installing",
                "available",
            ]
            .contains(&status)
            {
                return Err("invalid_desktop_reply".into());
            }
            let mut response =
                Response::success("update", value.clone(), format!("Desktop update: {status}"));
            response.status = status.into();
            if status == "recovery" {
                response.exit = 1;
            }
            return Ok(response);
        }
        std::thread::sleep(Duration::from_millis(50));
    }
    Err(Failure::new("desktop_handoff_timeout"))
}

pub fn status() -> Result<Response> {
    let info = installation_info()?;
    let identity = &info["identity"];
    let record = installation::data_root()?.join("desktop-update-status.json");
    if !record.try_exists()? {
        return Ok(Response::success(
            "update",
            json!({"identity":identity}),
            "No desktop update receipt recorded.".into(),
        ));
    }
    let record = installation::read_json(&record)?;
    let state = receipt_state(&info, &record);
    let mut response = Response::success(
        "update",
        json!({"identity":identity,"receipt":record}),
        format!("Desktop update: {state}"),
    );
    response.status = state.into();
    if state == "recovery" {
        response.exit = 1;
    } else if state == "manual_install" {
        response.exit = 3;
    }
    Ok(response)
}

fn receipt_state<'a>(info: &Value, record: &'a Value) -> &'a str {
    let identity = &info["identity"];
    let installed = info["installed"] == true
        && record["schema_version"] == 1
        && record["status"] == "installed"
        && record["target_version"] == identity["version"]
        && record["target_build"] == identity["build"]
        && record["target_version"]
            .as_str()
            .is_some_and(|s| !s.is_empty());
    if installed {
        "installed"
    } else if record["schema_version"] != 1 || record["status"] == "installed" {
        "recovery"
    } else {
        record["status"].as_str().unwrap_or("recovery")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn installed_receipt_requires_matching_current_installation() {
        let mut info = json!({"installed":true,"identity":{"version":"0.2.0","build":27}});
        let record = json!({"schema_version":1,"status":"installed","target_version":"0.2.0","target_build":27});
        assert_eq!(receipt_state(&info, &record), "installed");
        info["installed"] = json!(false);
        assert_eq!(receipt_state(&info, &record), "recovery");
        info["installed"] = json!(true);
        info["identity"]["build"] = json!(26);
        assert_eq!(receipt_state(&info, &record), "recovery");
        assert_eq!(
            receipt_state(&info, &json!({"schema_version":1,"status":"handed_off"})),
            "handed_off"
        );
    }
}
