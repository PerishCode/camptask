pub mod app;
pub mod config;
pub mod resources;
pub mod self_update;

use anyhow::Result;

use crate::app::AppContext;
use crate::config::{CampCommand, Command, ResourcesCommand};

pub struct RunResult;

pub fn run(app: &dyn AppContext) -> Result<RunResult> {
    match &app.config().command {
        Command::Hello => {
            println!("hello world");
        }
        Command::Resources(command) => match command {
            ResourcesCommand::Init => {
                resources::init(&app.config().resource_home)?;
            }
            ResourcesCommand::Update { dry_run } => {
                resources::update(&app.config().resource_home, *dry_run)?;
            }
            ResourcesCommand::Status => {
                resources::status(&app.config().resource_home)?;
            }
            ResourcesCommand::Doctor => {
                resources::doctor(&app.config().resource_home)?;
            }
        },
        Command::Camp(command) => match command {
            CampCommand::Init => {
                println!("camptask camp init skeleton ready");
            }
            CampCommand::CheckLite => {
                println!("camptask camp check-lite skeleton ready");
            }
            CampCommand::Archive => {
                println!("camptask camp archive skeleton ready");
            }
        },
        Command::SelfUpdate(options) => {
            self_update::run(options.clone())?;
        }
    }

    Ok(RunResult)
}
