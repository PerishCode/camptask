use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{Context, Result, bail};
use reqwest::StatusCode;
use reqwest::blocking::Client;
use semver::Version;
use serde::Deserialize;

const REPO_OWNER: &str = "PerishCode";
const REPO_NAME: &str = "camptask";
const REPO_URL: &str = "https://github.com/PerishCode/camptask.git";

#[derive(Debug, Clone)]
pub struct SelfUpdateOptions {
    pub check_only: bool,
    pub version: Option<String>,
    pub yes: bool,
}

#[derive(Debug, Deserialize)]
struct Release {
    tag_name: String,
}

pub fn run(options: SelfUpdateOptions) -> Result<()> {
    let client = http_client()?;
    let current = current_version()?;
    let target_tag = if let Some(version) = options.version.as_deref() {
        normalize_tag(version)
    } else {
        fetch_latest_tag(&client)?
    };
    let target = parse_semver_tag(&target_tag)?;

    if target <= current {
        println!(
            "camptask is up to date (current: v{}, target: {}).",
            current, target_tag
        );
        return Ok(());
    }

    if options.check_only {
        println!("Update available: v{} -> {}", current, target_tag);
        return Ok(());
    }

    if !options.yes {
        prompt_for_confirmation(&current, &target_tag)?;
    }

    let install_root = resolve_install_root_from_current_exe()?;
    install_tag_to_root(&target_tag, &install_root)?;
    println!(
        "Updated camptask to {} at {}/bin/camptask",
        target_tag,
        install_root.display()
    );
    Ok(())
}

fn http_client() -> Result<Client> {
    Client::builder()
        .user_agent(format!("{REPO_NAME}-self-update"))
        .build()
        .context("failed to build HTTP client")
}

fn fetch_latest_tag(client: &Client) -> Result<String> {
    let url = format!("https://api.github.com/repos/{REPO_OWNER}/{REPO_NAME}/releases/latest");
    let response = client
        .get(url)
        .send()
        .context("failed to fetch latest release")?;
    if response.status() == StatusCode::NOT_FOUND {
        bail!("no published release found yet")
    }
    let release = response
        .error_for_status()
        .context("latest release request failed")?
        .json::<Release>()
        .context("failed to parse latest release response")?;
    Ok(release.tag_name)
}

fn current_version() -> Result<Version> {
    Version::parse(env!("CARGO_PKG_VERSION")).context("invalid current version")
}

fn normalize_tag(input: &str) -> String {
    if input.starts_with('v') {
        return input.to_string();
    }
    format!("v{input}")
}

fn parse_semver_tag(tag: &str) -> Result<Version> {
    let normalized = tag.strip_prefix('v').unwrap_or(tag);
    Version::parse(normalized).with_context(|| format!("invalid release tag version: {tag}"))
}

fn prompt_for_confirmation(current: &Version, next_tag: &str) -> Result<()> {
    print!(
        "Upgrade camptask from v{} to {}? [y/N]: ",
        current, next_tag
    );
    io::stdout().flush().ok();
    let mut answer = String::new();
    io::stdin()
        .read_line(&mut answer)
        .context("failed to read confirmation input")?;
    let answer = answer.trim().to_lowercase();
    if answer != "y" && answer != "yes" {
        bail!("update cancelled by user");
    }
    Ok(())
}

fn resolve_install_root_from_current_exe() -> Result<PathBuf> {
    let exe = std::env::current_exe().context("failed to resolve current executable")?;
    let canonical = exe
        .canonicalize()
        .with_context(|| format!("failed to resolve executable realpath: {}", exe.display()))?;
    resolve_install_root_from_exe_path(&canonical).with_context(|| {
        format!(
            "unsupported install layout at {} (expected <prefix>/bin/camptask)",
            canonical.display()
        )
    })
}

fn resolve_install_root_from_exe_path(path: &Path) -> Result<PathBuf> {
    let file_name = path
        .file_name()
        .and_then(|value| value.to_str())
        .context("executable path has no file name")?;
    if file_name != "camptask" {
        bail!("current executable is not camptask");
    }

    let bin_dir = path
        .parent()
        .context("failed to resolve executable parent")?;
    let bin_name = bin_dir
        .file_name()
        .and_then(|value| value.to_str())
        .context("failed to resolve bin directory name")?;
    if bin_name != "bin" {
        bail!("executable is not under a bin directory");
    }

    bin_dir
        .parent()
        .map(PathBuf::from)
        .context("failed to resolve install root")
}

fn install_tag_to_root(tag: &str, install_root: &Path) -> Result<()> {
    let status = Command::new("cargo")
        .args([
            "install", "--git", REPO_URL, "--tag", tag, "--bin", "camptask", "--locked", "--force",
            "--root",
        ])
        .arg(install_root)
        .status()
        .context("failed to run cargo install")?;

    if !status.success() {
        bail!("cargo install failed with status {status}");
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_tag_adds_prefix() {
        assert_eq!(normalize_tag("0.1.0"), "v0.1.0");
        assert_eq!(normalize_tag("v0.1.0"), "v0.1.0");
    }

    #[test]
    fn parse_semver_tag_accepts_v_prefix() {
        let version = parse_semver_tag("v1.2.3").expect("version should parse");
        assert_eq!(version, Version::new(1, 2, 3));
    }

    #[test]
    fn resolve_install_root_from_valid_exe_path() {
        let root = resolve_install_root_from_exe_path(Path::new("/tmp/prefix/bin/camptask"))
            .expect("install root should resolve");
        assert_eq!(root, PathBuf::from("/tmp/prefix"));
    }

    #[test]
    fn resolve_install_root_rejects_non_bin_layout() {
        let err = resolve_install_root_from_exe_path(Path::new("/tmp/prefix/camptask"))
            .expect_err("non-bin layout should fail");
        assert!(err.to_string().contains("bin"));
    }
}
