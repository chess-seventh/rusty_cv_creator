// SCAFFOLD: true
// ApplicationRow read model + AppState. Replace in DELIVER slice-01 and slice-02.

/// Display-ready projection of a Cv record. No `quote` field — display context only.
#[derive(Debug, Clone)]
pub struct ApplicationRow {
    pub id: i32,
    pub date: String,
    pub company: String,
    pub job_title: String,
    pub pdf_path: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Mode {
    Normal,
    Filter,
}

pub struct AppState {
    pub rows: Vec<ApplicationRow>,
    pub selected_index: usize,
    pub filter_text: String,
    pub mode: Mode,
}

impl AppState {
    pub fn new(_rows: Vec<ApplicationRow>) -> Self {
        panic!("Not yet implemented — RED scaffold");
    }

    pub fn is_empty(&self) -> bool {
        panic!("Not yet implemented — RED scaffold");
    }

    pub fn status_text(&self) -> String {
        panic!("Not yet implemented — RED scaffold");
    }

    pub fn filtered_rows(&self) -> Vec<&ApplicationRow> {
        panic!("Not yet implemented — RED scaffold");
    }

    pub fn filtered_count(&self) -> usize {
        panic!("Not yet implemented — RED scaffold");
    }

    pub fn move_down(&mut self) {
        panic!("Not yet implemented — RED scaffold");
    }

    pub fn move_up(&mut self) {
        panic!("Not yet implemented — RED scaffold");
    }

    pub fn move_to_first(&mut self) {
        panic!("Not yet implemented — RED scaffold");
    }

    pub fn move_to_last(&mut self) {
        panic!("Not yet implemented — RED scaffold");
    }

    pub fn set_filter(&mut self, _text: &str) {
        panic!("Not yet implemented — RED scaffold");
    }

    pub fn clear_filter(&mut self) {
        panic!("Not yet implemented — RED scaffold");
    }

    pub fn selected_row(&self) -> Option<&ApplicationRow> {
        panic!("Not yet implemented — RED scaffold");
    }
}
