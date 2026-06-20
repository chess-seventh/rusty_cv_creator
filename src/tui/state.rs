use crate::models::Cv;

/// Display-ready projection of a Cv record. No `quote` field — display context only.
#[derive(Debug, Clone)]
pub struct ApplicationRow {
    pub id: i32,
    pub date: String,
    pub company: String,
    pub job_title: String,
    pub pdf_path: String,
}

impl From<Cv> for ApplicationRow {
    fn from(cv: Cv) -> Self {
        ApplicationRow {
            id: cv.id,
            date: cv.application_date.unwrap_or_else(|| "Unknown".to_string()),
            company: cv.company,
            job_title: cv.job_title,
            pdf_path: cv.pdf_cv_path,
        }
    }
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
    pub fn new(rows: Vec<ApplicationRow>) -> Self {
        AppState {
            rows,
            selected_index: 0,
            filter_text: String::new(),
            mode: Mode::Normal,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.rows.is_empty()
    }

    pub fn status_text(&self) -> String {
        let total = self.filtered_count();
        if total == 0 {
            return "No applications".to_string();
        }
        format!("{} of {} applications", self.selected_index + 1, total)
    }

    pub fn filtered_rows(&self) -> Vec<&ApplicationRow> {
        let text = self.filter_text.trim();
        if text.is_empty() {
            return self.rows.iter().collect();
        }
        let lower = text.to_lowercase();
        self.rows
            .iter()
            .filter(|r| {
                r.company.to_lowercase().contains(&lower)
                    || r.job_title.to_lowercase().contains(&lower)
            })
            .collect()
    }

    pub fn filtered_count(&self) -> usize {
        self.filtered_rows().len()
    }

    pub fn move_down(&mut self) {
        let count = self.filtered_count();
        if count == 0 {
            return;
        }
        if self.selected_index + 1 < count {
            self.selected_index += 1;
        }
    }

    pub fn move_up(&mut self) {
        if self.selected_index > 0 {
            self.selected_index -= 1;
        }
    }

    pub fn move_to_first(&mut self) {
        self.selected_index = 0;
    }

    pub fn move_to_last(&mut self) {
        let count = self.filtered_count();
        self.selected_index = if count == 0 { 0 } else { count - 1 };
    }

    pub fn set_filter(&mut self, text: &str) {
        self.filter_text = text.to_string();
        let count = self.filtered_count();
        if self.selected_index >= count && count > 0 {
            self.selected_index = count - 1;
        } else if count == 0 {
            self.selected_index = 0;
        }
    }

    pub fn clear_filter(&mut self) {
        self.filter_text = String::new();
        self.selected_index = 0;
    }

    pub fn selected_row(&self) -> Option<&ApplicationRow> {
        let rows = self.filtered_rows();
        rows.get(self.selected_index).copied()
    }
}
