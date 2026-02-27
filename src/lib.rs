pub mod app;

use crate::app::App;

pub fn run(app: &App) {
    println!("{} ready", app.name());
}
