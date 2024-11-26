mod app;
mod models;
mod ui;
mod widgets;

use app::App;
use std::env;

fn main() {
    let mut terminal = ratatui::init();

    let args: Vec<String> = env::args().collect();
    let app_result = App::new(args[1].as_str()).run(&mut terminal);

    ratatui::restore();
    app_result
}
