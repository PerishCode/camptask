use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};
use clap::{Args, Parser, Subcommand};

use crate::self_update::SelfUpdateOptions;

#[derive(Debug, Parser)]
#[command(
    name = "camptask",
    version,
    about = "Agent-agnostic runtime guard CLI skeleton"
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<CliCommand>,
}

#[derive(Debug, Subcommand)]
pub enum CliCommand {
    Resources(ResourcesArgs),
    Camp(CampArgs),
    Hello,
    SelfUpdate(SelfUpdateArgs),
}

#[derive(Debug, Args)]
pub struct ResourcesArgs {
    #[command(subcommand)]
    pub command: ResourcesSubcommand,
}

#[derive(Debug, Subcommand)]
pub enum ResourcesSubcommand {
    Init,
    Update(ResourcesUpdateArgs),
    Status,
    Doctor,
}

#[derive(Debug, Args)]
pub struct ResourcesUpdateArgs {
    #[arg(long = "dry-run")]
    pub dry_run: bool,
}

#[derive(Debug, Args)]
pub struct CampArgs {
    #[command(subcommand)]
    pub command: CampSubcommand,
}

#[derive(Debug, Subcommand)]
pub enum CampSubcommand {
    Init,
    CheckLite,
    Archive,
}

#[derive(Debug, Args)]
pub struct SelfUpdateArgs {
    #[arg(long = "check")]
    pub check: bool,

    #[arg(long = "version")]
    pub version: Option<String>,

    #[arg(long = "yes", short = 'y')]
    pub yes: bool,
}

#[derive(Debug, Clone)]
pub enum Command {
    Resources(ResourcesCommand),
    Camp(CampCommand),
    Hello,
    SelfUpdate(SelfUpdateOptions),
}

#[derive(Debug, Clone, Copy)]
pub enum ResourcesCommand {
    Init,
    Update { dry_run: bool },
    Status,
    Doctor,
}

#[derive(Debug, Clone, Copy)]
pub enum CampCommand {
    Init,
    CheckLite,
    Archive,
}

#[derive(Debug, Clone)]
pub struct RuntimeConfig {
    pub command: Command,
    pub camptask_home: PathBuf,
    pub resource_home: PathBuf,
}

#[derive(Debug, Clone)]
pub struct RawEnv {
    pub home: Option<PathBuf>,
    pub camptask_home: Option<PathBuf>,
    pub camptask_resource_home: Option<PathBuf>,
}

impl RawEnv {
    pub fn from_process() -> Self {
        Self {
            home: std::env::var_os("HOME").map(PathBuf::from),
            camptask_home: std::env::var_os("CAMPTASK_HOME").map(PathBuf::from),
            camptask_resource_home: std::env::var_os("CAMPTASK_RESOURCE_HOME").map(PathBuf::from),
        }
    }
}

impl RuntimeConfig {
    pub fn from_cli_and_env(cli: Cli, env: RawEnv) -> Result<Self> {
        let command = match cli.command.unwrap_or(CliCommand::Hello) {
            CliCommand::Resources(args) => Command::Resources(match args.command {
                ResourcesSubcommand::Init => ResourcesCommand::Init,
                ResourcesSubcommand::Update(args) => ResourcesCommand::Update {
                    dry_run: args.dry_run,
                },
                ResourcesSubcommand::Status => ResourcesCommand::Status,
                ResourcesSubcommand::Doctor => ResourcesCommand::Doctor,
            }),
            CliCommand::Camp(args) => Command::Camp(match args.command {
                CampSubcommand::Init => CampCommand::Init,
                CampSubcommand::CheckLite => CampCommand::CheckLite,
                CampSubcommand::Archive => CampCommand::Archive,
            }),
            CliCommand::Hello => Command::Hello,
            CliCommand::SelfUpdate(args) => Command::SelfUpdate(SelfUpdateOptions {
                check_only: args.check,
                version: args.version,
                yes: args.yes,
            }),
        };

        let camptask_home = resolve_camptask_home(&env)?;
        let resource_home = resolve_resource_home(&env, &camptask_home)?;

        Ok(Self {
            command,
            camptask_home,
            resource_home,
        })
    }
}

fn resolve_camptask_home(env: &RawEnv) -> Result<PathBuf> {
    if let Some(path) = env.camptask_home.clone() {
        return normalize_path(path, env.home.as_ref());
    }

    let home = env
        .home
        .as_ref()
        .context("HOME is not set; unable to resolve default camptask home")?;
    Ok(home.join(".camptask"))
}

fn resolve_resource_home(env: &RawEnv, camptask_home: &Path) -> Result<PathBuf> {
    if let Some(path) = env.camptask_resource_home.clone() {
        return normalize_path(path, env.home.as_ref());
    }
    Ok(camptask_home.join("resources"))
}

fn normalize_path(path: PathBuf, home: Option<&PathBuf>) -> Result<PathBuf> {
    let expanded = expand_tilde(path, home)?;
    if expanded.is_absolute() {
        return Ok(expanded);
    }

    let cwd = std::env::current_dir().context("failed to resolve current working directory")?;
    Ok(cwd.join(expanded))
}

fn expand_tilde(path: PathBuf, home: Option<&PathBuf>) -> Result<PathBuf> {
    let raw = path
        .to_str()
        .context("path contains non-UTF-8 characters, which is not supported")?;

    if raw == "~" {
        let home = home.context("HOME is not set; unable to expand '~'")?;
        return Ok(home.clone());
    }

    if let Some(stripped) = raw.strip_prefix("~/") {
        let home = home.context("HOME is not set; unable to expand '~/'")?;
        return Ok(home.join(stripped));
    }

    if raw.starts_with('~') {
        bail!("unsupported tilde path format: {raw}");
    }

    Ok(path)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base_cli() -> Cli {
        Cli {
            command: Some(CliCommand::Resources(ResourcesArgs {
                command: ResourcesSubcommand::Status,
            })),
        }
    }

    #[test]
    fn resource_home_prefers_camptask_resource_home() {
        let config = RuntimeConfig::from_cli_and_env(
            base_cli(),
            RawEnv {
                home: Some(PathBuf::from("/Users/tester")),
                camptask_home: Some(PathBuf::from("/tmp/camptask-home")),
                camptask_resource_home: Some(PathBuf::from("/opt/camptask-res")),
            },
        )
        .expect("config should build");

        assert_eq!(config.camptask_home, PathBuf::from("/tmp/camptask-home"));
        assert_eq!(config.resource_home, PathBuf::from("/opt/camptask-res"));
    }

    #[test]
    fn resource_home_falls_back_to_camptask_home() {
        let config = RuntimeConfig::from_cli_and_env(
            base_cli(),
            RawEnv {
                home: Some(PathBuf::from("/Users/tester")),
                camptask_home: Some(PathBuf::from("/tmp/camp-home")),
                camptask_resource_home: None,
            },
        )
        .expect("config should build");

        assert_eq!(config.camptask_home, PathBuf::from("/tmp/camp-home"));
        assert_eq!(
            config.resource_home,
            PathBuf::from("/tmp/camp-home/resources")
        );
    }

    #[test]
    fn camptask_home_defaults_from_home() {
        let config = RuntimeConfig::from_cli_and_env(
            base_cli(),
            RawEnv {
                home: Some(PathBuf::from("/Users/tester")),
                camptask_home: None,
                camptask_resource_home: None,
            },
        )
        .expect("config should build");

        assert_eq!(
            config.camptask_home,
            PathBuf::from("/Users/tester/.camptask")
        );
        assert_eq!(
            config.resource_home,
            PathBuf::from("/Users/tester/.camptask/resources")
        );
    }

    #[test]
    fn defaults_require_home_when_no_overrides() {
        let err = RuntimeConfig::from_cli_and_env(
            base_cli(),
            RawEnv {
                home: None,
                camptask_home: None,
                camptask_resource_home: None,
            },
        )
        .expect_err("config should fail without HOME and overrides");

        assert!(err.to_string().contains("HOME"));
    }
}
