# C4 Diagrams — rusty_cv_creator

Retroactive documentation of the implemented architecture (v4.0.2). Levels:
System Context (L1), Container (L2), Component (L3, CV-build subsystem only).

## L1 — System Context

```mermaid
C4Context
  title System Context — rusty_cv_creator
  Person(user, "Job Seeker", "Single user generating tailored CVs from the CLI")
  System(cvc, "rusty_cv_creator", "Rust CLI that builds variant CV PDFs and records them")
  System_Ext(template, "CV Template Repo", "git: chess-seventh/cv — LaTeX sources + Justfile (4 variants)")
  System_Ext(pg, "Postgres DB", "CV metadata store, reached over Tailscale")
  System_Ext(viewer, "PDF Viewer", "zathura")
  Rel(user, cvc, "Runs commands (insert/list/remove/update, --variant)")
  Rel(cvc, template, "Copies template & builds variant via 'just build <variant>'")
  Rel(cvc, pg, "Persists & reads CV metadata over")
  Rel(cvc, viewer, "Opens produced PDF in")
```

## L2 — Container

```mermaid
C4Container
  title Container Diagram — rusty_cv_creator
  Person(user, "Job Seeker")
  System_Boundary(cvc, "rusty_cv_creator (single binary)") {
    Container(cli, "CLI / Orchestrator", "Rust, clap", "Parses input, dispatches actions, orchestrates prepare_cv")
    Container(build, "Build Subsystem", "Rust (file_handlers)", "Resolves variant, copies template, runs builder, copies out PDF")
    Container(runner, "CommandRunner Port", "Rust trait", "SystemRunner (prod) / FakeRunner (test) for subprocesses")
    Container(persist, "Persistence", "Rust, diesel MultiConnection", "DbConnection over Postgres/SQLite; Cv CRUD")
    Container(config, "Config", "Rust, configparser + OnceCell", "INI-driven settings in GLOBAL_VAR")
  }
  System_Ext(template, "CV Template Repo", "LaTeX + Justfile")
  System_Ext(tools, "External Tools", "just, tectonic, tailscale, zathura (via devenv/PATH)")
  ContainerDb(pg, "Postgres / SQLite", "SQL", "CV metadata")
  Rel(user, cli, "Invokes")
  Rel(cli, build, "Calls prepare_cv ->")
  Rel(cli, config, "Reads settings from")
  Rel(cli, persist, "Saves/reads CV via")
  Rel(build, runner, "Runs 'just build <variant>' through")
  Rel(build, config, "Reads prefix/builder/recipe/paths from")
  Rel(runner, tools, "Executes")
  Rel(build, template, "Copies & builds")
  Rel(persist, pg, "Connects to")
```

## L3 — Component: CV-build subsystem (`file_handlers` + orchestration)

```mermaid
C4Component
  title Component Diagram — CV-build subsystem
  Container_Boundary(build, "Build Subsystem") {
    Component(prepare, "prepare_cv", "main.rs", "Orchestrates: resolve -> check tools -> mkdir -> compile -> copy out")
    Component(resolve, "resolve_variant", "file_handlers.rs", "flag -> infer-from-title -> default (pure)")
    Component(infer, "infer_variant_from_job_title", "file_handlers.rs", "Keyword match; manager checked first (pure)")
    Component(cfg, "BuildConfig::from_config", "file_handlers.rs", "prefix/builder/recipe from INI")
    Component(mkdir, "create_directory", "file_handlers.rs", "Dated working copy of template")
    Component(compile, "compile_cv", "file_handlers.rs", "Validates inputs, runs '<builder> <recipe> <variant>'")
    Component(copyout, "remove_created_dir_from_pro", "file_handlers.rs", "Copy PDF to output+sibling, cleanup workdir")
  }
  Component_Ext(runner, "CommandRunner", "port", "status(builder, [recipe,variant], cwd)")
  Component_Ext(tools, "ensure_tools_available", "helpers.rs", "PATH pre-usage check (just, tectonic)")
  Rel(prepare, resolve, "1. resolves variant via")
  Rel(resolve, infer, "falls back to")
  Rel(prepare, cfg, "2. loads")
  Rel(prepare, tools, "3. pre-checks")
  Rel(prepare, mkdir, "4. creates workdir via")
  Rel(prepare, compile, "5. builds via")
  Rel(compile, runner, "invokes builder through")
  Rel(prepare, copyout, "6. finalizes via")
```
