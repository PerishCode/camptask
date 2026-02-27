pub mod app;

use crate::app::App;
use std::env;
use std::fs;
use std::io::{Cursor, Write};
use std::path::{Path, PathBuf};
use std::process::Command;
use tempfile::tempdir;
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
        _ => Err("unknown command, supported: init".to_string()),
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

fn print_init_help() {
    println!("camptask init [--target <path>] [--no-overwrite] [--url <resources-zip-url>]");
}

fn default_resources_url() -> String {
    let version = env!("CARGO_PKG_VERSION");
    format!("{RELEASE_BASE_URL}/v{version}/resources.zip")
}

fn default_resources_dir() -> Result<PathBuf, String> {
    let home = env::var("HOME")
        .or_else(|_| env::var("USERPROFILE"))
        .map_err(|_| "cannot resolve home directory from HOME/USERPROFILE".to_string())?;
    Ok(PathBuf::from(home).join(".camptask").join("resources"))
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
