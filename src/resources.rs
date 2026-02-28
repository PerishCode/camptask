use std::fs;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

#[derive(Debug, Deserialize)]
struct Manifest {
    contents: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct State {
    schema_version: String,
    installed_from: String,
    version: String,
    installed_at: String,
    manifest_sha256: String,
    files: Vec<StateFile>,
}

#[derive(Debug, Serialize, Deserialize)]
struct StateFile {
    path: String,
    sha256: String,
}

const EMBEDDED_FILES: &[(&str, &str)] = &[
    ("manifest.json", include_str!("../resources/manifest.json")),
    (
        "prompts/UNIFIED.md",
        include_str!("../resources/prompts/UNIFIED.md"),
    ),
    (
        "schemas/camp_event.v0.schema.json",
        include_str!("../resources/schemas/camp_event.v0.schema.json"),
    ),
    (
        "schemas/camp_metadata.v0.schema.json",
        include_str!("../resources/schemas/camp_metadata.v0.schema.json"),
    ),
    (
        "specs/camp_runtime_v0.md",
        include_str!("../resources/specs/camp_runtime_v0.md"),
    ),
    (
        "specs/check_lite_v0.json",
        include_str!("../resources/specs/check_lite_v0.json"),
    ),
    (
        "templates/camp_handoff.md",
        include_str!("../resources/templates/camp_handoff.md"),
    ),
    (
        "templates/work_report.md",
        include_str!("../resources/templates/work_report.md"),
    ),
];

pub fn init(resource_home: &Path) -> Result<()> {
    if resource_home.exists() && has_any_entries(resource_home)? {
        bail!(
            "resource home is not empty: {}. run `camptask resources status` first",
            resource_home.display()
        );
    }

    write_embedded_files(resource_home)?;
    write_state(resource_home)?;

    println!("initialized resources at {}", resource_home.display());
    Ok(())
}

pub fn update(resource_home: &Path, dry_run: bool) -> Result<()> {
    let file_changes = count_file_changes(resource_home)?;
    if dry_run {
        println!(
            "resources update dry-run: {} file changes at {}",
            file_changes,
            resource_home.display()
        );
        return Ok(());
    }

    let parent = resource_home
        .parent()
        .context("resource home has no parent directory")?;
    fs::create_dir_all(parent)
        .with_context(|| format!("failed to create parent directory: {}", parent.display()))?;

    let suffix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .context("system clock is before unix epoch")?
        .as_nanos();
    let staging = parent.join(format!(".resources.staging.{suffix}"));

    write_embedded_files(&staging)?;
    write_state(&staging)?;

    let backup = parent.join(format!(".resources.backup.{suffix}"));
    if resource_home.exists() {
        fs::rename(resource_home, &backup).with_context(|| {
            format!(
                "failed to rotate existing resources directory: {}",
                resource_home.display()
            )
        })?;
    }

    if let Err(error) = fs::rename(&staging, resource_home) {
        if backup.exists() {
            let _ = fs::rename(&backup, resource_home);
        }
        bail!(
            "failed to install updated resources at {}: {}",
            resource_home.display(),
            error
        );
    }

    if backup.exists() {
        let _ = fs::remove_dir_all(&backup);
    }

    println!(
        "updated resources at {} ({} file changes)",
        resource_home.display(),
        file_changes
    );
    Ok(())
}

pub fn status(resource_home: &Path) -> Result<()> {
    let manifest_path = resource_home.join("manifest.json");
    let state_path = resource_home.join("state.json");
    if manifest_path.is_file() {
        let content = fs::read_to_string(&manifest_path)
            .with_context(|| format!("failed to read manifest: {}", manifest_path.display()))?;
        let manifest: Manifest = serde_json::from_str(&content)
            .with_context(|| format!("invalid manifest JSON: {}", manifest_path.display()))?;

        let state_message = if state_path.is_file() {
            let state_raw = fs::read_to_string(&state_path)
                .with_context(|| format!("failed to read state: {}", state_path.display()))?;
            let state: State = serde_json::from_str(&state_raw)
                .with_context(|| format!("invalid state JSON: {}", state_path.display()))?;
            format!(
                "source={}, version={}, installed_at={}",
                state.installed_from, state.version, state.installed_at
            )
        } else {
            "state=missing".to_string()
        };

        println!(
            "resources installed at {} ({} files, {})",
            resource_home.display(),
            manifest.contents.len(),
            state_message
        );
    } else {
        println!("resources not initialized at {}", resource_home.display());
    }
    Ok(())
}

pub fn doctor(resource_home: &Path) -> Result<()> {
    let manifest_path = resource_home.join("manifest.json");
    if !manifest_path.is_file() {
        bail!(
            "resources doctor: fail (missing manifest at {})",
            manifest_path.display()
        );
    }

    let content = fs::read_to_string(&manifest_path)
        .with_context(|| format!("failed to read manifest: {}", manifest_path.display()))?;
    let manifest: Manifest = serde_json::from_str(&content)
        .with_context(|| format!("invalid manifest JSON: {}", manifest_path.display()))?;

    for relative in manifest.contents {
        let target = resource_home.join(&relative);
        if !target.is_file() {
            bail!("resources doctor: fail (missing {})", target.display());
        }
    }

    let state_path = resource_home.join("state.json");
    if !state_path.is_file() {
        bail!(
            "resources doctor: fail (missing state at {})",
            state_path.display()
        );
    }

    let state_content = fs::read_to_string(&state_path)
        .with_context(|| format!("failed to read state: {}", state_path.display()))?;
    let state: State = serde_json::from_str(&state_content)
        .with_context(|| format!("invalid state JSON: {}", state_path.display()))?;
    let expected_manifest_sha = sha256_hex(content.as_bytes());
    if state.manifest_sha256 != expected_manifest_sha {
        bail!(
            "resources doctor: fail (manifest hash mismatch: expected {}, got {})",
            expected_manifest_sha,
            state.manifest_sha256
        );
    }

    println!("resources doctor: pass ({})", resource_home.display());
    Ok(())
}

fn has_any_entries(path: &Path) -> Result<bool> {
    let mut entries = fs::read_dir(path)
        .with_context(|| format!("failed to read directory entries: {}", path.display()))?;
    Ok(entries.next().transpose()?.is_some())
}

fn write_embedded_files(target_root: &Path) -> Result<()> {
    fs::create_dir_all(target_root)
        .with_context(|| format!("failed to create directory: {}", target_root.display()))?;

    for (relative, content) in EMBEDDED_FILES {
        let target = target_root.join(relative);
        if let Some(parent) = target.parent() {
            fs::create_dir_all(parent).with_context(|| {
                format!(
                    "failed to create resource parent directory: {}",
                    parent.display()
                )
            })?;
        }
        fs::write(&target, content)
            .with_context(|| format!("failed to write embedded resource: {}", target.display()))?;
    }

    Ok(())
}

fn write_state(target_root: &Path) -> Result<()> {
    let manifest_raw = embedded_file_content("manifest.json")
        .context("embedded manifest.json is missing from resources table")?;
    let manifest: Manifest =
        serde_json::from_str(manifest_raw).context("embedded manifest.json is invalid JSON")?;

    let mut files = Vec::with_capacity(manifest.contents.len());
    for relative in manifest.contents {
        let target = target_root.join(&relative);
        let bytes = fs::read(&target)
            .with_context(|| format!("failed to read resource file: {}", target.display()))?;
        files.push(StateFile {
            path: relative,
            sha256: sha256_hex(&bytes),
        });
    }

    let installed_at = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .context("system clock is before unix epoch")?
        .as_secs()
        .to_string();

    let state = State {
        schema_version: "camptask.resources.state.v1".to_string(),
        installed_from: "embedded".to_string(),
        version: format!("v{}", env!("CARGO_PKG_VERSION")),
        installed_at,
        manifest_sha256: sha256_hex(manifest_raw.as_bytes()),
        files,
    };

    let state_path = target_root.join("state.json");
    fs::write(
        &state_path,
        serde_json::to_string_pretty(&state).context("failed to serialize state JSON")?,
    )
    .with_context(|| format!("failed to write state file: {}", state_path.display()))?;

    Ok(())
}

fn embedded_file_content(path: &str) -> Option<&'static str> {
    EMBEDDED_FILES
        .iter()
        .find_map(|(file_path, content)| (*file_path == path).then_some(*content))
}

fn count_file_changes(resource_home: &Path) -> Result<usize> {
    let mut changes = 0usize;
    for (relative, content) in EMBEDDED_FILES {
        let target = resource_home.join(relative);
        if !target.is_file() {
            changes += 1;
            continue;
        }
        let existing = fs::read_to_string(&target)
            .with_context(|| format!("failed to read existing resource: {}", target.display()))?;
        if existing != *content {
            changes += 1;
        }
    }
    Ok(changes)
}

fn sha256_hex(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    format!("{:x}", hasher.finalize())
}
