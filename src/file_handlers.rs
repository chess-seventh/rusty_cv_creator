use crate::command_runner::CommandRunner;
use crate::config_parse::get_variable_from_config_file;
use crate::global_conf::AppContext;
use crate::helpers::{clean_string_from_quotes, fix_home_directory_path};
use chrono::{DateTime, Local};
use log::{error, info, warn};
use std::fs;
use std::io::Error;
use std::path::Path;

/// The CV variants supported by the template repository. Each one maps to a
/// driver file `<prefix>-<variant>.tex` and is built with `just build <variant>`.
pub const CV_VARIANTS: [&str; 4] = [
    "senior-devops",
    "senior-platform-engineer",
    "senior-sre",
    "engineering-manager",
];

fn check_dir_exists(dir: &str) -> bool {
    Path::new(dir).is_dir()
}

fn check_file_exists(dir: &str, file: &str) -> bool {
    let full_path = format!("{dir}/{file}");
    Path::new(&full_path).is_file()
}

/// Infer the CV variant from keywords in the job title.
///
/// `engineering-manager` is checked first on purpose: the manager variant's
/// title also contains "DevOps" and "Platform", so a job such as
/// "Engineering Manager - DevOps" must not be classified as `senior-devops`.
fn infer_variant_from_job_title(job_title: &str) -> Option<&'static str> {
    let jt = job_title.to_lowercase();
    if jt.contains("manager")
        || jt.contains("management")
        || jt.contains("head of")
        || jt.contains("lead")
    {
        Some("engineering-manager")
    } else if jt.contains("platform") {
        Some("senior-platform-engineer")
    } else if jt.contains("sre") || jt.contains("site reliability") || jt.contains("reliability") {
        Some("senior-sre")
    } else if jt.contains("devops") || jt.contains("dev ops") {
        Some("senior-devops")
    } else {
        None
    }
}

/// Resolve which CV variant to build.
///
/// Precedence: an explicit (and valid) `--variant` flag wins; otherwise the
/// variant is inferred from the job title; otherwise the configured default is
/// used.
pub fn resolve_variant(
    variant_flag: Option<&String>,
    job_title: &str,
    default_variant: &str,
) -> String {
    if let Some(flag) = variant_flag {
        let flag = flag.trim();
        if CV_VARIANTS.contains(&flag) {
            info!("✅ Using variant from --variant flag: {flag}");
            return flag.to_string();
        }
        warn!(
            "Unknown variant '{flag}' (expected one of {CV_VARIANTS:?}); \
             inferring from the job title instead"
        );
    }

    if let Some(inferred) = infer_variant_from_job_title(job_title) {
        info!("✅ Inferred variant '{inferred}' from job title: {job_title}");
        return inferred.to_string();
    }

    info!("✅ Falling back to default variant: {default_variant}");
    default_variant.to_string()
}

/// Build configuration read from the INI file: the driver/output filename
/// prefix and how each variant is compiled.
#[derive(Debug, Clone)]
pub struct BuildConfig {
    pub prefix: String,
    pub builder: String,
    pub recipe: String,
}

impl BuildConfig {
    pub fn from_context(ctx: &AppContext) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self {
            prefix: get_variable_from_config_file(ctx, "cv", "cv_file_prefix")?,
            builder: get_variable_from_config_file(ctx, "build", "builder")
                .unwrap_or_else(|_| "just".to_string()),
            recipe: get_variable_from_config_file(ctx, "build", "recipe")
                .unwrap_or_else(|_| "build".to_string()),
        })
    }
}

/// Build a single CV variant inside `cv_dir` using the project Justfile.
///
/// Runs `<builder> <recipe> <variant>` (default `just build <variant>`) in the
/// copied template directory, which produces `<prefix>-<variant>.pdf` next to
/// the driver file. All build artifacts (pdf, log, aux, ...) land in `cv_dir`,
/// so there is no hard-coded output directory.
///
/// Tool availability (`just`, `tectonic`) is checked by the caller before this
/// runs; here we only validate the inputs and invoke the builder via `runner`.
pub fn compile_cv(
    runner: &dyn CommandRunner,
    cv_dir: &str,
    variant: &str,
    cfg: &BuildConfig,
) -> Result<(), Box<dyn std::error::Error>> {
    info!("CV_DIR: {cv_dir}");
    info!("CV_VARIANT: {variant}");

    if check_dir_exists(cv_dir) {
        info!("✅ Directory exists");
    } else {
        error!("Directory does not exist: {cv_dir}");
        return Err(format!("Directory does not exist: {cv_dir}").into());
    }

    let driver_file = format!("{}-{variant}.tex", cfg.prefix);
    if check_file_exists(cv_dir, &driver_file) {
        info!("✅ Driver file exists: {driver_file}");
    } else {
        error!("Driver file does not exist: {cv_dir}/{driver_file}");
        return Err(format!("Driver file does not exist: {cv_dir}/{driver_file}").into());
    }

    info!(
        "✅ Building CV: {} {} {variant} (in {cv_dir})",
        cfg.builder, cfg.recipe
    );

    if runner.status(&cfg.builder, &[&cfg.recipe, variant], Some(cv_dir))? {
        info!("✅ CV compiled successfully");
        Ok(())
    } else {
        error!(
            "Error building CV with: {} {} {variant}",
            cfg.builder, cfg.recipe
        );
        Err(format!(
            "Error building CV with: {} {} {variant}",
            cfg.builder, cfg.recipe
        )
        .into())
    }
}

pub fn create_directory(
    ctx: &AppContext,
    job_title: &str,
    company_name: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    let var = get_variable_from_config_file(ctx, "destination", "cv_path")?;

    let destination_folder = fix_home_directory_path(&var);
    let now = ctx.get_today();

    match prepare_year_dir(&destination_folder, now) {
        Ok(y) => info!("✅ Year directory created successfully: {y:}"),
        Err(e) => {
            error!("Error creating year directory: {e:}");
            return Err(format!("Error creating year directory: {e:}")
                .to_string()
                .into());
        }
    }

    let (cv_template_path, full_destination_path) =
        match prepare_path_for_new_cv(ctx, job_title, company_name, &destination_folder, now) {
            Ok(s) => s,
            Err(e) => {
                error!("{e:?}");
                return Err(format!("{e:?}").to_string().into());
            }
        };

    match copy_dir::copy_dir(cv_template_path, full_destination_path.clone()) {
        Ok(_) => info!("✅ Directory created & copied successfully"),
        Err(e) => {
            error!("Error copying directory: {e:}");
            return Err(format!("Error copying directory: {e:}").to_string().into());
        }
    }
    Ok(full_destination_path)
}

pub fn remove_cv_dir(path_to_remove: &Path) -> std::io::Result<()> {
    fs::remove_dir_all(path_to_remove)?;
    Ok(())
}

fn prepare_path_for_new_cv(
    ctx: &AppContext,
    job_title: &str,
    company_name: &str,
    destination_folder: &str,
    now: &DateTime<Local>,
) -> Result<(String, String), Box<dyn std::error::Error>> {
    let var = get_variable_from_config_file(ctx, "cv", "cv_template_path")?;

    let cv_template_path: String = fix_home_directory_path(&var);

    let formatted_job_title = sanitize_for_path(job_title);
    let formatted_company_name = sanitize_for_path(company_name);

    let date_dir = now.format("%Y/%Y-%m-%d").to_string();
    let full_destination_path = fix_home_directory_path(&format!(
        "{destination_folder}/{date_dir}_{formatted_company_name}_{formatted_job_title}"
    ));

    info!("✅ Creating directory: {full_destination_path}");
    info!("✅ Copying from: {}", cv_template_path.clone());

    Ok((cv_template_path, full_destination_path))
}

fn prepare_year_dir(destination_folder: &String, now: &DateTime<Local>) -> Result<String, Error> {
    let year_full_dir = format!("{}/{}", destination_folder, now.format("%Y"));
    fs::create_dir_all(year_full_dir.clone())?;
    Ok(clean_string_from_quotes(&year_full_dir.clone()))
}

/// Make a value safe to embed in a path / file name (spaces become dashes).
fn sanitize_for_path(value: &str) -> String {
    value.replace(' ', "-")
}

/// Copy the built PDF out of the working directory into the configured
/// destinations, then remove the working directory so only the PDF remains.
///
/// Returns the path to the PDF placed in the configured `output_pdf` directory.
pub fn remove_created_dir_from_pro(
    ctx: &AppContext,
    job_title: &str,
    company_name: &str,
    created_cv_dir: &String,
    pdf_basename: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    let path_created_dir = Path::new(&created_cv_dir);
    let application_date = ctx.get_today_str_yyyy_mm_dd();
    let application_year = ctx.get_year_str();

    // The PDF produced by `just build <variant>` inside the working directory.
    let built_pdf = format!("{created_cv_dir}/{pdf_basename}");
    if !Path::new(&built_pdf).is_file() {
        error!("Built PDF not found: {built_pdf}");
        return Err(format!("Built PDF not found: {built_pdf}").into());
    }

    let job = sanitize_for_path(job_title);
    let company = sanitize_for_path(company_name);
    let final_name = format!("{application_date}-{job}-{company}.pdf");

    // 1) A copy kept next to the dated working directory (survives cleanup).
    let parent_dir = path_created_dir
        .parent()
        .and_then(Path::to_str)
        .ok_or("Could not determine parent directory of the CV working dir")?;
    let sibling_pdf = format!("{parent_dir}/{final_name}");

    // 2) The configured output location, organised per year.
    let output_dir = get_variable_from_config_file(ctx, "destination", "output_pdf")?;
    let output_year_dir = format!("{output_dir}/{application_year}");
    fs::create_dir_all(&output_year_dir)?;
    let output_pdf = format!("{output_year_dir}/{final_name}");

    copy_to_destination(&built_pdf, &output_pdf)?;
    copy_to_destination(&built_pdf, &sibling_pdf)?;

    // Cleanup: remove the whole working directory, keeping only the PDFs above.
    remove_cv_dir(path_created_dir)?;
    info!("✅ Cleaned up working directory: {created_cv_dir}");

    Ok(output_pdf)
}

/// Copy a single file, returning the bytes copied on success.
fn copy_to_destination(src: &str, dst: &str) -> std::io::Result<u64> {
    info!("✅ Copying {src} -> {dst}");
    fs::copy(src, dst).inspect_err(|e| {
        error!("Could not copy {src} -> {dst}: {e:}");
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_check_dir_exists_true_for_dir() {
        let td = TempDir::new().unwrap();
        assert!(check_dir_exists(td.path().to_str().unwrap()));
    }

    #[test]
    fn test_check_dir_exists_false_for_non_dir() {
        assert!(!check_dir_exists("/definitely/not/here"));
    }

    #[test]
    fn test_check_file_exists_true_for_file() {
        let td = TempDir::new().unwrap();
        let path = td.path().to_str().unwrap();
        let file = format!("{path:}/file.txt");
        fs::write(&file, "hi").unwrap();
        assert!(check_file_exists(path, "file.txt"));
    }

    #[test]
    fn test_check_file_exists_false_for_missing_file() {
        let td = TempDir::new().unwrap();
        assert!(!check_file_exists(
            td.path().to_str().unwrap(),
            "missing.txt"
        ));
    }

    #[test]
    fn test_sanitize_for_path_replaces_spaces() {
        assert_eq!(
            sanitize_for_path("Senior DevOps Engineer"),
            "Senior-DevOps-Engineer"
        );
        assert_eq!(sanitize_for_path("nospace"), "nospace");
    }

    #[test]
    fn test_infer_variant_manager_wins_over_devops() {
        assert_eq!(
            infer_variant_from_job_title("Engineering Manager - DevOps"),
            Some("engineering-manager")
        );
    }

    #[test]
    fn test_infer_variant_keywords() {
        assert_eq!(
            infer_variant_from_job_title("Senior Platform Engineer"),
            Some("senior-platform-engineer")
        );
        assert_eq!(
            infer_variant_from_job_title("Site Reliability Engineer"),
            Some("senior-sre")
        );
        assert_eq!(
            infer_variant_from_job_title("DevOps Specialist"),
            Some("senior-devops")
        );
        assert_eq!(infer_variant_from_job_title("Accountant"), None);
    }

    #[test]
    fn test_resolve_variant_flag_wins() {
        let flag = "senior-sre".to_string();
        assert_eq!(
            resolve_variant(Some(&flag), "Platform Engineer", "senior-devops"),
            "senior-sre"
        );
    }

    #[test]
    fn test_resolve_variant_invalid_flag_falls_back_to_inference() {
        let flag = "bogus".to_string();
        assert_eq!(
            resolve_variant(Some(&flag), "Platform Engineer", "senior-devops"),
            "senior-platform-engineer"
        );
    }

    #[test]
    fn test_resolve_variant_uses_default_when_nothing_matches() {
        assert_eq!(
            resolve_variant(None, "Accountant", "senior-devops"),
            "senior-devops"
        );
    }

    #[test]
    fn test_copy_to_destination_copies_file() {
        let td = TempDir::new().unwrap();
        let src = format!("{}/src.pdf", td.path().to_str().unwrap());
        let dst = format!("{}/dst.pdf", td.path().to_str().unwrap());
        fs::write(&src, "pdf-bytes").unwrap();
        let copied = copy_to_destination(&src, &dst).unwrap();
        assert_eq!(copied, "pdf-bytes".len() as u64);
        assert_eq!(fs::read_to_string(&dst).unwrap(), "pdf-bytes");
    }

    #[test]
    fn test_copy_to_destination_errors_for_missing_source() {
        let td = TempDir::new().unwrap();
        let dst = format!("{}/dst.pdf", td.path().to_str().unwrap());
        assert!(copy_to_destination("/definitely/not/here.pdf", &dst).is_err());
    }

    fn test_cfg() -> BuildConfig {
        BuildConfig {
            prefix: "TestCV".to_string(),
            builder: "just".to_string(),
            recipe: "build".to_string(),
        }
    }

    #[test]
    fn test_compile_cv_success_invokes_builder() {
        let td = TempDir::new().unwrap();
        let dir = td.path().to_str().unwrap();
        fs::write(td.path().join("TestCV-senior-devops.tex"), "x").unwrap();

        let runner = crate::command_runner::testing::FakeRunner::ok();
        assert!(compile_cv(&runner, dir, "senior-devops", &test_cfg()).is_ok());
        assert_eq!(runner.calls.borrow()[0], "just build senior-devops");
    }

    #[test]
    fn test_compile_cv_missing_driver_errors() {
        let td = TempDir::new().unwrap();
        let runner = crate::command_runner::testing::FakeRunner::ok();
        assert!(
            compile_cv(
                &runner,
                td.path().to_str().unwrap(),
                "senior-sre",
                &test_cfg()
            )
            .is_err()
        );
    }

    #[test]
    fn test_compile_cv_missing_dir_errors() {
        let runner = crate::command_runner::testing::FakeRunner::ok();
        assert!(compile_cv(&runner, "/definitely/not/here", "senior-sre", &test_cfg()).is_err());
    }

    #[test]
    fn test_compile_cv_builder_failure_errors() {
        let td = TempDir::new().unwrap();
        fs::write(td.path().join("TestCV-x.tex"), "x").unwrap();
        let runner = crate::command_runner::testing::FakeRunner::failing();
        assert!(compile_cv(&runner, td.path().to_str().unwrap(), "x", &test_cfg()).is_err());
    }

    #[test]
    fn test_remove_cv_dir_removes_directory() {
        let td = TempDir::new().unwrap();
        let sub = td.path().join("sub");
        fs::create_dir_all(sub.join("nested")).unwrap();
        assert!(remove_cv_dir(&sub).is_ok());
        assert!(!sub.exists());
    }

    #[test]
    fn test_create_directory_and_remove_flow() {
        let td = TempDir::new().unwrap();
        let base = td.path();
        let template = base.join("template");
        fs::create_dir_all(&template).unwrap();
        fs::write(template.join("TestCV-senior-devops.tex"), "x").unwrap();
        let dest = base.join("dest");
        let out = base.join("out");
        fs::create_dir_all(&dest).unwrap();

        let ini = format!(
            "[cv]\ncv_template_path = \"{tpl}\"\ncv_file_prefix = \"TestCV\"\n\
             [destination]\ncv_path = \"{dest}\"\noutput_pdf = \"{out}\"\n\
             [db]\nengine = \"sqlite\"\ndb_file = \"x.db\"\n",
            tpl = template.display(),
            dest = dest.display(),
            out = out.display()
        );
        let ini_path = base.join("conf.ini");
        fs::write(&ini_path, ini).unwrap();

        let ui = crate::cli_structure::UserInput {
            action: crate::cli_structure::UserAction::Insert(
                crate::cli_structure::FilterArgs::default(),
            ),
            save_to_database: false,
            view_generated_cv: false,
            dry_run: false,
            config_ini: ini_path.to_str().unwrap().to_string(),
            engine: "sqlite".to_string(),
        };
        let ctx = crate::config_parse::build_context(&ui);

        // create_directory copies the template into a dated dir under dest.
        let created = create_directory(&ctx, "Senior DevOps", "ACME").unwrap();
        assert!(
            Path::new(&created)
                .join("TestCV-senior-devops.tex")
                .is_file()
        );

        // Place a "built" PDF, then run the copy-out + cleanup.
        fs::write(format!("{created}/TestCV-senior-devops.pdf"), b"%PDF").unwrap();
        let output_pdf = remove_created_dir_from_pro(
            &ctx,
            "Senior DevOps",
            "ACME",
            &created,
            "TestCV-senior-devops.pdf",
        )
        .unwrap();

        assert!(Path::new(&output_pdf).is_file());
        assert!(!Path::new(&created).exists());
    }
}
