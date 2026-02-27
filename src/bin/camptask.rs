use camptask::app::App;

fn main() {
    let app = App::new("camptask");
    if let Err(error) = camptask::run(&app) {
        eprintln!("camptask: {error}");
        std::process::exit(1);
    }
}
