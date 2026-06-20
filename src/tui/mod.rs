pub mod app;
pub mod db;
pub mod events;
pub mod probe;
pub mod state;
pub mod terminal_guard;
pub mod ui;

pub use db::load_all_applications;

pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    probe::run_startup_probe().map_err(|e| -> Box<dyn std::error::Error> { e.into() })?;

    let cvs = db::load_all_applications()?;
    let rows: Vec<state::ApplicationRow> =
        cvs.into_iter().map(state::ApplicationRow::from).collect();
    let app_state = state::AppState::new(rows);

    let mut app = app::App::new(app_state)?;
    app.run()
}
