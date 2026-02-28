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
    Update,
    Status,
    Doctor,
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
    Update,
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
}

impl RuntimeConfig {
    pub fn from_cli(cli: Cli) -> Self {
        let command = match cli.command.unwrap_or(CliCommand::Hello) {
            CliCommand::Resources(args) => Command::Resources(match args.command {
                ResourcesSubcommand::Init => ResourcesCommand::Init,
                ResourcesSubcommand::Update => ResourcesCommand::Update,
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
        Self { command }
    }
}
