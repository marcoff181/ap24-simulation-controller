pub use app::App;

pub mod app;



fn run() -> Result<(), std::io::Error> {
    let terminal = ratatui::init();
    let result = App::new().run(terminal);
    ratatui::restore();
    result // Assuming `result` is of type Result<(), std::io::Error>
}
