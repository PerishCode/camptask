use anyhow::Result;
use clap::Parser;

use camptask::app::App;
use camptask::config::{Cli, RuntimeConfig};
use camptask::run;

fn main() -> Result<()> {
    let cli = Cli::parse();
    let config = RuntimeConfig::from_cli(cli);
    let app = App::new(config);
    let _ = run(&app)?;
    Ok(())
}
