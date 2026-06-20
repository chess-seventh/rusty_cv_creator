pub mod app;
pub mod events;
pub mod probe;
pub mod state;
pub mod terminal_guard;
pub mod ui;

/// Render the interactive job-applications table for the supplied CV records.
///
/// Pure UI: the caller (the bin crate) owns DB access and passes the already
/// loaded `cvs`. The startup probe runs first so a missing terminal fails fast
/// before any rendering is attempted.
pub fn run(cvs: Vec<crate::models::Cv>) -> Result<(), Box<dyn std::error::Error>> {
    probe::run_startup_probe().map_err(|e| -> Box<dyn std::error::Error> { e.into() })?;

    let rows: Vec<state::ApplicationRow> =
        cvs.into_iter().map(state::ApplicationRow::from).collect();
    let app_state = state::AppState::new(rows);

    let mut app = app::App::new(app_state)?;
    app.run()
}
