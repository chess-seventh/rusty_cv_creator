use crate::tui::state::AppState;
use ratatui::layout::{Constraint, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::{Block, Borders, Row, Table};
use ratatui::Frame;

pub fn render(frame: &mut Frame, state: &AppState) {
    let area = frame.area();
    render_table(frame, area, state);
}

fn render_table(frame: &mut Frame, area: Rect, state: &AppState) {
    let filtered = state.filtered_rows();

    let header = Row::new(["ID", "Date", "Company", "Job Title", "PDF Path"])
        .style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )
        .height(1);

    let rows: Vec<Row> = filtered
        .iter()
        .enumerate()
        .map(|(i, row)| {
            let style = if i == state.selected_index {
                Style::default().bg(Color::Blue).fg(Color::White)
            } else {
                Style::default()
            };
            Row::new(vec![
                row.id.to_string(),
                row.date.clone(),
                row.company.clone(),
                row.job_title.clone(),
                row.pdf_path.clone(),
            ])
            .style(style)
        })
        .collect();

    let title = if state.filter_text.trim().is_empty() {
        "Job Applications".to_string()
    } else {
        format!("Job Applications [filter: {}]", state.filter_text.trim())
    };

    let widths = [
        Constraint::Length(6),
        Constraint::Length(12),
        Constraint::Percentage(25),
        Constraint::Percentage(25),
        Constraint::Fill(1),
    ];

    let table = Table::new(rows, widths)
        .header(header)
        .block(Block::default().title(title).borders(Borders::ALL));

    frame.render_widget(table, area);
}
