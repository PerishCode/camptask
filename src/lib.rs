pub mod app;
pub mod config;
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
                println!("camptask resources init skeleton ready");
            }
            ResourcesCommand::Update => {
                println!("camptask resources update skeleton ready");
            }
            ResourcesCommand::Status => {
                println!("camptask resources status skeleton ready");
            }
            ResourcesCommand::Doctor => {
                println!("camptask resources doctor skeleton ready");
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
