pub mod app;

use crate::app::App;
use serde_json::Value;
use std::env;
use std::fs;
use std::io::{Cursor, Write};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};
use tempfile::{NamedTempFile, tempdir};
use zip::ZipArchive;

const RELEASE_BASE_URL: &str = "https://github.com/PerishCode/camptask/releases/download";

pub fn run(app: &App) -> Result<(), String> {
    let args: Vec<String> = env::args().skip(1).collect();
    if args.is_empty() {
        println!("{} ready", app.name());
        return Ok(());
    }

    match args[0].as_str() {
        "init" => run_init(args.get(1..).unwrap_or(&[])),
        "agent" => run_agent(args.get(1..).unwrap_or(&[])),
        "work" => run_work(args.get(1..).unwrap_or(&[])),
        _ => Err(
            "unknown command, supported: init, agent init, work init|update|finish|status"
                .to_string(),
        ),
    }
}

fn run_init(args: &[String]) -> Result<(), String> {
    let mut target = default_resources_dir()?;
    let mut overwrite = true;
    let mut url = default_resources_url();

    let mut index = 0;
    while index < args.len() {
        match args[index].as_str() {
            "--target" => {
                let value = args
                    .get(index + 1)
                    .ok_or_else(|| "--target requires a path value".to_string())?;
                target = PathBuf::from(value);
                index += 2;
            }
            "--no-overwrite" => {
                overwrite = false;
                index += 1;
            }
            "--url" => {
                let value = args
                    .get(index + 1)
                    .ok_or_else(|| "--url requires a value".to_string())?;
                url = value.clone();
                index += 2;
            }
            "-h" | "--help" => {
                print_init_help();
                return Ok(());
            }
            flag => {
                return Err(format!("unknown init option: {flag}"));
            }
        }
    }

    if target.exists() && !overwrite {
        return Err(format!(
            "target already exists: {} (use overwrite default or remove --no-overwrite)",
            target.display()
        ));
    }

    let bytes = download_resources_zip(&url)?;
    extract_resources_zip(&bytes, &target, overwrite)?;
    println!("initialized resources at {}", target.display());
    Ok(())
}

fn run_agent(args: &[String]) -> Result<(), String> {
    if args.is_empty() {
        return Err("agent subcommand required, supported: init".to_string());
    }

    match args[0].as_str() {
        "init" => run_agent_init(args.get(1..).unwrap_or(&[])),
        _ => Err("unknown agent subcommand, supported: init".to_string()),
    }
}

fn run_work(args: &[String]) -> Result<(), String> {
    if args.is_empty() {
        return Err(
            "work subcommand required, supported: init, update, finish, status".to_string(),
        );
    }

    match args[0].as_str() {
        "init" => run_work_init(args.get(1..).unwrap_or(&[])),
        "update" => run_work_update(args.get(1..).unwrap_or(&[])),
        "finish" => run_work_finish(args.get(1..).unwrap_or(&[])),
        "status" => run_work_status(args.get(1..).unwrap_or(&[])),
        _ => Err("unknown work subcommand, supported: init, update, finish, status".to_string()),
    }
}

fn run_work_init(args: &[String]) -> Result<(), String> {
    let mut branch: Option<String> = None;
    let mut path: Option<PathBuf> = None;

    let mut index = 0;
    while index < args.len() {
        match args[index].as_str() {
            "--branch" => {
                let value = args
                    .get(index + 1)
                    .ok_or_else(|| "--branch requires a value".to_string())?;
                if value.trim().is_empty() {
                    return Err("--branch requires a non-empty value".to_string());
                }
                branch = Some(value.to_string());
                index += 2;
            }
            "--path" => {
                let value = args
                    .get(index + 1)
                    .ok_or_else(|| "--path requires a value".to_string())?;
                if value.trim().is_empty() {
                    return Err("--path requires a non-empty value".to_string());
                }
                path = Some(PathBuf::from(value));
                index += 2;
            }
            "-h" | "--help" => {
                print_work_init_help();
                return Ok(());
            }
            flag => {
                return Err(format!("unknown work init option: {flag}"));
            }
        }
    }

    let branch = branch.ok_or_else(|| "--branch is required".to_string())?;
    let branch_for_output = branch.clone();
    let path = path.ok_or_else(|| "--path is required".to_string())?;

    ensure_git_available()?;
    ensure_inside_git_repo()?;

    if path.exists() {
        return Err(format!(
            "--path must not already exist for isolated worktree: {}",
            path.display()
        ));
    }

    let status = Command::new("git")
        .arg("worktree")
        .arg("add")
        .arg("--checkout")
        .arg("-b")
        .arg(&branch)
        .arg(&path)
        .status()
        .map_err(|e| format!("failed to run git worktree add: {e}"))?;

    if !status.success() {
        return Err("git worktree add failed".to_string());
    }

    let work_meta_dir = path.join(".work");
    fs::create_dir_all(&work_meta_dir)
        .map_err(|e| format!("cannot create {}: {e}", work_meta_dir.display()))?;

    let mut meta = serde_json::Map::new();
    meta.insert(
        "schema_version".to_string(),
        Value::String("work_meta.v1".to_string()),
    );
    meta.insert(
        "work_state".to_string(),
        Value::String("initialized".to_string()),
    );
    meta.insert("branch".to_string(), Value::String(branch));
    meta.insert(
        "worktree_path".to_string(),
        Value::String(path.display().to_string()),
    );
    meta.insert("updated_at".to_string(), Value::String(now_epoch_seconds()));

    let meta_path = work_meta_dir.join("WORK_META.json");
    write_json_atomic(&meta_path, &Value::Object(meta))?;

    println!(
        "{{\"status\":\"ok\",\"work_state\":\"initialized\",\"branch\":\"{}\",\"path\":\"{}\"}}",
        escape_json_string(&branch_for_output),
        escape_json_string(&path.display().to_string())
    );
    Ok(())
}

fn run_work_update(args: &[String]) -> Result<(), String> {
    let mut note: Option<String> = None;
    let mut index = 0;
    while index < args.len() {
        match args[index].as_str() {
            "--note" => {
                let value = args
                    .get(index + 1)
                    .ok_or_else(|| "--note requires a value".to_string())?;
                note = Some(value.to_string());
                index += 2;
            }
            "-h" | "--help" => {
                print_work_update_help();
                return Ok(());
            }
            flag => return Err(format!("unknown work update option: {flag}")),
        }
    }

    let cwd = env::current_dir().map_err(|e| format!("cannot resolve cwd: {e}"))?;
    let meta_path = cwd.join(".work").join("WORK_META.json");
    let mut meta = read_required_json_object(&meta_path)?;
    let obj = meta
        .as_object_mut()
        .ok_or_else(|| format!("{} must be a JSON object", meta_path.display()))?;
    obj.insert(
        "work_state".to_string(),
        Value::String("updated".to_string()),
    );
    obj.insert("updated_at".to_string(), Value::String(now_epoch_seconds()));
    if let Some(n) = note {
        obj.insert("note".to_string(), Value::String(n));
    }
    write_json_atomic(&meta_path, &meta)?;
    println!("{{\"status\":\"ok\",\"work_state\":\"updated\"}}");
    Ok(())
}

fn run_work_finish(args: &[String]) -> Result<(), String> {
    if args.len() == 1 && (args[0] == "-h" || args[0] == "--help") {
        print_work_finish_help();
        return Ok(());
    }
    if !args.is_empty() {
        return Err("work finish does not accept arguments".to_string());
    }

    let cwd = env::current_dir().map_err(|e| format!("cannot resolve cwd: {e}"))?;
    let meta_path = cwd.join(".work").join("WORK_META.json");
    let mut meta = read_required_json_object(&meta_path)?;
    let obj = meta
        .as_object_mut()
        .ok_or_else(|| format!("{} must be a JSON object", meta_path.display()))?;
    obj.insert(
        "work_state".to_string(),
        Value::String("finished".to_string()),
    );
    obj.insert("updated_at".to_string(), Value::String(now_epoch_seconds()));
    write_json_atomic(&meta_path, &meta)?;
    println!("{{\"status\":\"ok\",\"work_state\":\"finished\"}}");
    Ok(())
}

fn run_work_status(args: &[String]) -> Result<(), String> {
    if args.len() == 1 && (args[0] == "-h" || args[0] == "--help") {
        print_work_status_help();
        return Ok(());
    }
    if !args.is_empty() {
        return Err("work status does not accept arguments".to_string());
    }

    let cwd = env::current_dir().map_err(|e| format!("cannot resolve cwd: {e}"))?;
    let meta_path = cwd.join(".work").join("WORK_META.json");
    let meta = read_required_json_object(&meta_path)?;
    let obj = meta
        .as_object()
        .ok_or_else(|| format!("{} must be a JSON object", meta_path.display()))?;

    let state = obj
        .get("work_state")
        .and_then(Value::as_str)
        .ok_or_else(|| format!("{} missing work_state", meta_path.display()))?;

    println!(
        "{{\"status\":\"ok\",\"work_state\":\"{}\"}}",
        escape_json_string(state)
    );
    Ok(())
}

fn run_agent_init(args: &[String]) -> Result<(), String> {
    if args.len() == 1 && (args[0] == "-h" || args[0] == "--help") {
        print_agent_init_help();
        return Ok(());
    }
    if !args.is_empty() {
        return Err("agent init does not accept arguments".to_string());
    }

    let camptask_home = camptask_home()?;
    let prompts_dir = camptask_home.join("resources").join("prompts");
    let leader_prompt = prompts_dir.join("LEADER.md");
    let worker_prompt = prompts_dir.join("WORKER.md");

    if !leader_prompt.exists() || !worker_prompt.exists() {
        return Err(format!(
            "missing prompt resources in {} (run `camptask init` first)",
            prompts_dir.display()
        ));
    }

    let opencode_home = opencode_home()?;
    let config_path = opencode_home.join("opencode.json");
    fs::create_dir_all(&opencode_home).map_err(|e| {
        format!(
            "cannot create opencode home {}: {e}",
            opencode_home.display()
        )
    })?;

    let mut config = read_or_default_config(&config_path)?;
    let root = config
        .as_object_mut()
        .ok_or_else(|| format!("{} must be a JSON object", config_path.display()))?;

    if !root.contains_key("$schema") {
        root.insert(
            "$schema".to_string(),
            Value::String("https://opencode.ai/config.json".to_string()),
        );
    }

    let agent_value = root
        .entry("agent".to_string())
        .or_insert_with(|| Value::Object(serde_json::Map::new()));
    let agents = agent_value
        .as_object_mut()
        .ok_or_else(|| "config field `agent` must be an object".to_string())?;

    agents.insert(
        "camptask.leader".to_string(),
        json_agent_entry(
            "camptask leader agent",
            &format!("{{file:{}}}", leader_prompt.display()),
        ),
    );
    agents.insert(
        "camptask.worker".to_string(),
        json_agent_entry(
            "camptask worker agent",
            &format!("{{file:{}}}", worker_prompt.display()),
        ),
    );

    write_json_atomic(&config_path, &config)?;

    println!("initialized opencode agents in {}", config_path.display());
    Ok(())
}

fn json_agent_entry(description: &str, prompt_ref: &str) -> Value {
    let mut obj = serde_json::Map::new();
    obj.insert("mode".to_string(), Value::String("primary".to_string()));
    obj.insert(
        "description".to_string(),
        Value::String(description.to_string()),
    );
    obj.insert("prompt".to_string(), Value::String(prompt_ref.to_string()));
    Value::Object(obj)
}

fn read_or_default_config(path: &Path) -> Result<Value, String> {
    if !path.exists() {
        return Ok(Value::Object(serde_json::Map::new()));
    }
    let text = fs::read_to_string(path)
        .map_err(|e| format!("cannot read config {}: {e}", path.display()))?;
    serde_json::from_str(&text).map_err(|e| format!("cannot parse config {}: {e}", path.display()))
}

fn read_required_json_object(path: &Path) -> Result<Value, String> {
    if !path.exists() {
        return Err(format!(
            "required work metadata missing: {} (run `camptask work init` first)",
            path.display()
        ));
    }
    read_or_default_config(path)
}

fn write_json_atomic(path: &Path, value: &Value) -> Result<(), String> {
    let parent = path
        .parent()
        .ok_or_else(|| format!("invalid config path {}", path.display()))?;
    fs::create_dir_all(parent)
        .map_err(|e| format!("cannot create directory {}: {e}", parent.display()))?;

    let data = serde_json::to_vec_pretty(value)
        .map_err(|e| format!("cannot serialize config {}: {e}", path.display()))?;

    let mut tmp = NamedTempFile::new_in(parent)
        .map_err(|e| format!("cannot create temp config in {}: {e}", parent.display()))?;
    tmp.write_all(&data)
        .map_err(|e| format!("cannot write temp config: {e}"))?;
    tmp.write_all(b"\n")
        .map_err(|e| format!("cannot finalize temp config: {e}"))?;
    tmp.flush()
        .map_err(|e| format!("cannot flush temp config: {e}"))?;
    tmp.persist(path)
        .map_err(|e| format!("cannot persist config {}: {}", path.display(), e.error))?;

    Ok(())
}

fn print_init_help() {
    println!("camptask init [--target <path>] [--no-overwrite] [--url <resources-zip-url>]");
}

fn print_agent_init_help() {
    println!("camptask agent init");
}

fn print_work_init_help() {
    println!("camptask work init --branch <branch> --path <worktree-path>");
}

fn print_work_update_help() {
    println!("camptask work update [--note <text>]");
}

fn print_work_finish_help() {
    println!("camptask work finish");
}

fn print_work_status_help() {
    println!("camptask work status");
}

fn ensure_git_available() -> Result<(), String> {
    let output = Command::new("git")
        .arg("--version")
        .output()
        .map_err(|e| format!("failed to execute git: {e}"))?;
    if output.status.success() {
        Ok(())
    } else {
        Err("git is required".to_string())
    }
}

fn ensure_inside_git_repo() -> Result<(), String> {
    let output = Command::new("git")
        .arg("rev-parse")
        .arg("--is-inside-work-tree")
        .output()
        .map_err(|e| format!("failed to execute git rev-parse: {e}"))?;
    if !output.status.success() {
        return Err("current directory is not inside a git repository".to_string());
    }
    let stdout = String::from_utf8_lossy(&output.stdout);
    if stdout.trim() == "true" {
        Ok(())
    } else {
        Err("current directory is not inside a git repository".to_string())
    }
}

fn now_epoch_seconds() -> String {
    match SystemTime::now().duration_since(UNIX_EPOCH) {
        Ok(d) => d.as_secs().to_string(),
        Err(_) => "0".to_string(),
    }
}

fn escape_json_string(raw: &str) -> String {
    raw.replace('\\', "\\\\").replace('"', "\\\"")
}

fn default_resources_url() -> String {
    let version = env!("CARGO_PKG_VERSION");
    format!("{RELEASE_BASE_URL}/v{version}/resources.zip")
}

fn home_dir() -> Result<PathBuf, String> {
    let home = env::var("HOME")
        .or_else(|_| env::var("USERPROFILE"))
        .map_err(|_| "cannot resolve home directory from HOME/USERPROFILE".to_string())?;
    Ok(PathBuf::from(home))
}

fn camptask_home() -> Result<PathBuf, String> {
    if let Ok(home) = env::var("CAMPTASK_HOME") {
        if !home.trim().is_empty() {
            return Ok(PathBuf::from(home));
        }
    }
    Ok(home_dir()?.join(".camptask"))
}

fn opencode_home() -> Result<PathBuf, String> {
    if let Ok(home) = env::var("CAMPTASK_AGENT_OPENCODE_HOME") {
        if !home.trim().is_empty() {
            return Ok(PathBuf::from(home));
        }
    }
    Ok(home_dir()?.join(".config").join("opencode"))
}

fn default_resources_dir() -> Result<PathBuf, String> {
    Ok(camptask_home()?.join("resources"))
}

fn download_resources_zip(url: &str) -> Result<Vec<u8>, String> {
    let output = Command::new("curl")
        .arg("-fsSL")
        .arg(url)
        .output()
        .map_err(|e| format!("failed to execute curl: {e}"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("download failed: {}", stderr.trim()));
    }

    Ok(output.stdout)
}

fn extract_resources_zip(bytes: &[u8], target: &Path, overwrite: bool) -> Result<(), String> {
    let temp = tempdir().map_err(|e| format!("cannot create temp dir: {e}"))?;
    let staging = temp.path().join("resources");
    fs::create_dir_all(&staging).map_err(|e| format!("cannot create staging dir: {e}"))?;

    let cursor = Cursor::new(bytes);
    let mut archive = ZipArchive::new(cursor).map_err(|e| format!("invalid zip archive: {e}"))?;

    for index in 0..archive.len() {
        let mut entry = archive
            .by_index(index)
            .map_err(|e| format!("cannot read zip entry: {e}"))?;
        let enclosed = entry
            .enclosed_name()
            .ok_or_else(|| "zip contains invalid path".to_string())?
            .to_path_buf();
        let out_path = staging.join(enclosed);

        if entry.is_dir() {
            fs::create_dir_all(&out_path)
                .map_err(|e| format!("cannot create directory {}: {e}", out_path.display()))?;
            continue;
        }

        if let Some(parent) = out_path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| format!("cannot create directory {}: {e}", parent.display()))?;
        }

        let mut out_file = fs::File::create(&out_path)
            .map_err(|e| format!("cannot create file {}: {e}", out_path.display()))?;
        std::io::copy(&mut entry, &mut out_file)
            .map_err(|e| format!("cannot write file {}: {e}", out_path.display()))?;
        out_file
            .flush()
            .map_err(|e| format!("cannot flush file {}: {e}", out_path.display()))?;
    }

    if target.exists() {
        if !overwrite {
            return Err(format!("target exists: {}", target.display()));
        }
        fs::remove_dir_all(target)
            .map_err(|e| format!("cannot remove target {}: {e}", target.display()))?;
    }

    if let Some(parent) = target.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| format!("cannot create parent {}: {e}", parent.display()))?;
    }

    fs::rename(staging, target)
        .map_err(|e| format!("cannot move resources into {}: {e}", target.display()))?;

    Ok(())
}
