use anyhow::Result;
use clap::Parser;

use camptask::app::App;
use camptask::config::{Cli, RawEnv, RuntimeConfig};
use camptask::run;

fn main() -> Result<()> {
    let cli = Cli::parse();
    let config = RuntimeConfig::from_cli_and_env(cli, RawEnv::from_process())?;
    let app = App::new(config);
    let _ = run(&app)?;
    Ok(())
}
