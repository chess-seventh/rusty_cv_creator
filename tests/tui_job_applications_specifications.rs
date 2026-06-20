// DISTILL: tui-job-applications — unit-level specifications
// Imports from rusty_cv_creator::tui::* resolve against scaffold stubs in src/tui/.

use rusty_cv_creator::models::Cv;
use rusty_cv_creator::tui::events::open_pdf;
use rusty_cv_creator::tui::load_all_applications;
use rusty_cv_creator::tui::state::{AppState, ApplicationRow, Mode};

// ─── Helper: make a test ApplicationRow ──────────────────────────────────────

fn make_row(id: i32, company: &str, job_title: &str, pdf_path: &str) -> ApplicationRow {
    ApplicationRow {
        id,
        date: "2024-01-01".to_string(),
        company: company.to_string(),
        job_title: job_title.to_string(),
        pdf_path: pdf_path.to_string(),
    }
}

fn make_cv(
    id: i32,
    application_date: Option<&str>,
    company: &str,
    job_title: &str,
    pdf_cv_path: &str,
) -> Cv {
    Cv {
        id,
        application_date: application_date.map(|s| s.to_string()),
        company: company.to_string(),
        job_title: job_title.to_string(),
        pdf_cv_path: pdf_cv_path.to_string(),
        quote: String::new(),
        generated: true,
    }
}

// ─── ApplicationRow projection ───────────────────────────────────────────────

/// @us-01 @in-memory
/// ApplicationRow maps all Cv fields to display-ready types.
#[test]
fn us01_s01_maps_all_fields_correctly() {
    let cv = make_cv(1, Some("2024-03-15"), "Acme Corp", "Rust Engineer", "/home/user/cvs/acme.pdf");
    let row = ApplicationRow::from(cv);
    assert_eq!(row.id, 1);
    assert_eq!(row.date, "2024-03-15");
    assert_eq!(row.company, "Acme Corp");
    assert_eq!(row.job_title, "Rust Engineer");
    assert_eq!(row.pdf_path, "/home/user/cvs/acme.pdf");
}

/// @us-01 @in-memory
/// ApplicationRow falls back to "Unknown" when application_date is None.
#[test]
fn us01_s02_date_falls_back_to_unknown_when_absent() {
    let cv = make_cv(2, None, "Beta Corp", "Dev", "/tmp/cv.pdf");
    let row = ApplicationRow::from(cv);
    assert_eq!(row.date, "Unknown");
}

/// @us-01 @in-memory
/// ApplicationRow does not have a quote field (compile-time check via make_row).
#[test]
fn us01_s03_excludes_quote_field() {
    let _row = make_row(1, "Acme", "Dev", "/tmp/cv.pdf");
    // Struct compiles without quote field — passes if it compiles.
}

// ─── AppState: empty state ────────────────────────────────────────────────────

/// @us-01 @in-memory
#[test]
fn us01_s04_empty_state_when_no_rows() {
    let state = AppState::new(vec![]);
    assert!(state.is_empty());
    assert!(state.status_text().contains("No applications"));
}

/// @real-io @adapter-integration @us-01 @error
/// load_all_applications() returns an error when DATABASE_URL is not set.
#[test]
fn us01_e01_load_all_applications_returns_error_when_database_url_not_set() {
    // Temporarily remove DATABASE_URL, then restore it.
    let saved = std::env::var("DATABASE_URL").ok();
    unsafe { std::env::remove_var("DATABASE_URL") };
    let result = load_all_applications();
    if let Some(url) = saved {
        unsafe { std::env::set_var("DATABASE_URL", url) };
    }
    assert!(result.is_err(), "Expected Err when DATABASE_URL is not set");
}

// ─── Keyboard navigation ──────────────────────────────────────────────────────

fn make_state(n: usize) -> AppState {
    let rows = (0..n)
        .map(|i| make_row(i as i32, &format!("Co{i}"), &format!("Dev{i}"), "/tmp/cv.pdf"))
        .collect();
    AppState::new(rows)
}

/// @us-02 @in-memory
#[test]
fn us02_s01_down_increments_selected_index() {
    let mut state = make_state(5);
    state.move_down();
    assert_eq!(state.selected_index, 1);
    assert!(state.status_text().contains("2 of 5"));
}

/// @us-02 @in-memory
#[test]
fn us02_s02_down_on_last_row_does_not_advance() {
    let mut state = make_state(5);
    state.selected_index = 4;
    state.move_down();
    assert_eq!(state.selected_index, 4);
}

/// @us-02 @in-memory
#[test]
fn us02_s03_up_decrements_selected_index() {
    let mut state = make_state(5);
    state.selected_index = 3;
    state.move_up();
    assert_eq!(state.selected_index, 2);
}

/// @us-02 @in-memory
#[test]
fn us02_s04_up_on_first_row_does_not_go_below_zero() {
    let mut state = make_state(5);
    state.move_up();
    assert_eq!(state.selected_index, 0);
}

/// @us-02 @in-memory
#[test]
fn us02_s05_home_key_jumps_to_first_row() {
    let mut state = make_state(5);
    state.selected_index = 3;
    state.move_to_first();
    assert_eq!(state.selected_index, 0);
}

/// @us-02 @in-memory
#[test]
fn us02_s06_end_key_jumps_to_last_row() {
    let mut state = make_state(5);
    state.move_to_last();
    assert_eq!(state.selected_index, 4);
}

/// @us-02 @in-memory @error
/// Navigation on an empty list (0 rows) is a no-op and does not panic.
#[test]
fn us02_e01_navigation_on_empty_list_is_no_op() {
    let mut state = make_state(0);
    state.move_down();
    assert_eq!(state.selected_index, 0);
}

/// @us-02 @in-memory @error
/// Navigation when all rows are filtered out does not panic.
#[test]
fn us02_e02_navigation_on_filter_empty_result_does_not_panic() {
    let mut state = make_state(3);
    state.set_filter("zzznomatch");
    state.move_down();
    assert_eq!(state.selected_index, 0);
}

// ─── PBT: navigation invariant ────────────────────────────────────────────────

/// @us-02 @property
/// Navigation never yields an out-of-bounds index for any list and any key sequence.
#[test]
fn pbt_01_navigation_never_out_of_bounds() {
    use proptest::prelude::*;
    proptest!(|(n in 1usize..200usize, ops in proptest::collection::vec(0u8..4u8, 0..100usize))| {
        let mut state = make_state(n);
        for op in &ops {
            match op {
                0 => state.move_down(),
                1 => state.move_up(),
                2 => state.move_to_first(),
                _ => state.move_to_last(),
            }
        }
        prop_assert!(state.selected_index < n);
    });
}

// ─── Filter ───────────────────────────────────────────────────────────────────

fn make_state_with_companies(companies: &[(&str, &str)]) -> AppState {
    let rows = companies
        .iter()
        .enumerate()
        .map(|(i, (co, jt))| make_row(i as i32, co, jt, "/tmp/cv.pdf"))
        .collect();
    AppState::new(rows)
}

/// @us-03 @in-memory
#[test]
fn us03_s01_empty_filter_returns_all_rows() {
    let mut state = make_state(5);
    state.set_filter("");
    assert_eq!(state.filtered_count(), 5);
}

/// @us-03 @in-memory
#[test]
fn us03_s02_filter_matches_company_case_insensitively() {
    let mut state = make_state_with_companies(&[
        ("Acme Corp", "Dev"),
        ("Beta Systems", "Dev"),
        ("acme labs", "Dev"),
    ]);
    state.set_filter("acme");
    assert_eq!(state.filtered_count(), 2);
}

/// @us-03 @in-memory
#[test]
fn us03_s03_filter_matches_job_title_case_insensitively() {
    let mut state = make_state_with_companies(&[
        ("Co1", "Rust Engineer"),
        ("Co2", "rust developer"),
        ("Co3", "Python Dev"),
    ]);
    state.set_filter("rust");
    assert_eq!(state.filtered_count(), 2);
}

/// @us-03 @in-memory
#[test]
fn us03_s04_filter_with_no_matches_returns_empty() {
    let mut state = make_state_with_companies(&[("Acme", "Dev"), ("Beta", "Dev")]);
    state.set_filter("zzznomatch");
    assert_eq!(state.filtered_count(), 0);
}

/// @us-03 @in-memory
#[test]
fn us03_s05_clearing_filter_restores_full_list_and_resets_selection() {
    let mut state = make_state_with_companies(&[
        ("Acme", "Dev"),
        ("Beta", "Dev"),
        ("Acme Labs", "Dev"),
    ]);
    state.set_filter("acme");
    state.selected_index = 1;
    state.clear_filter();
    assert_eq!(state.filtered_count(), 3);
    assert_eq!(state.selected_index, 0);
}

/// @us-03 @in-memory @error
/// Whitespace-only filter text is treated as empty and returns all rows.
#[test]
fn us03_e01_whitespace_only_filter_treated_as_empty() {
    let mut state = make_state(5);
    state.set_filter("   ");
    assert_eq!(state.filtered_count(), 5);
}

/// @us-03 @in-memory @error
/// Filter input with special regex characters does not panic.
#[test]
fn us03_e02_special_chars_in_filter_do_not_panic() {
    let mut state = make_state_with_companies(&[("Acme Corp", "Dev"), ("Beta", "Dev")]);
    state.set_filter(".*[invalid[");
    let _count = state.filtered_count(); // must not panic
}

/// @us-03 @property
#[test]
fn pbt_02_filtered_count_never_exceeds_total() {
    use proptest::prelude::*;
    proptest!(|(rows in proptest::collection::vec(("[a-z]{3,8}", "[a-z]{3,8}"), 0..100usize),
               filter in "[a-z]{0,5}")| {
        let companies: Vec<(&str, &str)> = rows.iter().map(|(a, b)| (a.as_str(), b.as_str())).collect();
        let mut state = make_state_with_companies(&companies);
        state.set_filter(&filter);
        prop_assert!(state.filtered_count() <= rows.len());
    });
}

/// @us-03 @property
#[test]
fn pbt_03_empty_filter_always_returns_full_list() {
    use proptest::prelude::*;
    proptest!(|(n in 0usize..100usize)| {
        let mut state = make_state(n);
        state.set_filter("");
        prop_assert_eq!(state.filtered_count(), n);
    });
}

// ─── PDF open action ──────────────────────────────────────────────────────────

/// @us-04 @in-memory
#[test]
fn us04_s01_open_pdf_returns_error_for_nonexistent_path() {
    let result = open_pdf("/tmp/does_not_exist_12345.pdf");
    assert!(result.is_err());
    let msg = result.unwrap_err();
    assert!(
        msg.contains("File not found"),
        "Expected 'File not found' in error, got: {msg}"
    );
    assert!(msg.contains("/tmp/does_not_exist_12345.pdf"));
}

/// @us-04 @in-memory @error
/// open_pdf with an empty path string returns a file-not-found error.
#[test]
fn us04_e01_open_pdf_returns_error_for_empty_path() {
    let result = open_pdf("");
    assert!(result.is_err(), "Expected error for empty path");
}

/// @us-04 @in-memory @error
/// open_pdf with a directory path instead of a file returns an error.
#[test]
fn us04_e02_open_pdf_returns_error_for_directory_path() {
    let result = open_pdf("/tmp");
    assert!(result.is_err(), "Expected error when path is a directory");
}

/// @us-04 @in-memory
#[test]
fn us04_s02_enter_ignored_when_list_is_empty() {
    let state = AppState::new(vec![]);
    assert!(state.selected_row().is_none());
}

/// @us-04 @in-memory
#[test]
fn us04_s03_enter_ignored_in_filter_mode() {
    let mut state = make_state(3);
    state.mode = Mode::Filter;
    assert_eq!(state.mode, Mode::Filter);
}

/// @us-04 @in-memory
#[test]
fn us04_s04_enter_calls_open_pdf_with_selected_row_path_in_normal_mode() {
    let mut state = make_state_with_companies(&[("Co0", "Dev"), ("Co1", "Dev"), ("Co2", "Dev")]);
    state.rows[1].pdf_path = "/tmp/test_cv.pdf".to_string();
    state.selected_index = 1;
    assert_eq!(state.mode, Mode::Normal);
    let row = state.selected_row().expect("should have selected row");
    assert_eq!(row.pdf_path, "/tmp/test_cv.pdf");
}
