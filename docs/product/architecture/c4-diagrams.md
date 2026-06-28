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
    Container(build, "Build Subsystem", "Rust (file_handlers)", "Resolves variant, copies template, builds, files PDF")
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

## L3 — Config injection (feature `config-injection`, proposed — ADR-0006)

> Forward-looking. L1/L2 above are unchanged; the only delta is that the `Config`
> container stops being the process-global `GLOBAL_VAR` `OnceCell` and becomes an
> immutable `AppContext` value built once in `main` and passed inward by borrow.
> Contrast: **before**, every component reached the global getter; **after**,
> `&AppContext` flows down the call chain alongside the already-injected
> `CommandRunner` (ADR-0002) and `DbConnection` (ADR-0003).

```mermaid
C4Component
  title Component Diagram — Config injection (after ADR-0006)
  Person(user, "Job Seeker")
  Component(main, "main", "main.rs", "Builds AppContext once; wires runner + connection")
  Component(ctx, "AppContext", "app_context.rs", "Immutable {config, today, user_input}; read-only accessors")
  Component(dispatch, "match_user_action", "cli_structure.rs", "Dispatch; takes &AppContext")
  Component(insert, "insert_cv / prepare_cv", "cv_insert.rs + main.rs", "Build orchestration; takes &AppContext")
  Component(files, "file_handlers", "file_handlers.rs", "BuildConfig::from_config / create_directory / copy-out; read &AppContext")
  Component(remove, "remove_cv", "user_action.rs", "Resolves (engine,url) from &AppContext")
  Component_Ext(db, "establish_connection", "database.rs", "Takes resolved (engine,url) — config-free")
  Rel(user, main, "Runs CLI")
  Rel(main, ctx, "Constructs once from parsed UserInput")
  Rel(main, dispatch, "Passes &AppContext to")
  Rel(dispatch, insert, "Calls with &AppContext")
  Rel(dispatch, remove, "Calls with &AppContext")
  Rel(insert, files, "Reads config/today via &AppContext")
  Rel(insert, ctx, "Reads accessors (read-only)")
  Rel(remove, db, "Resolves (engine,url) then opens via")
```

## Template sourcing — feature `template-source` (ADR-0008)

### L1 — System Context (delta)

```mermaid
C4Context
  title System Context — template sourcing
  Person(user, "Job Seeker", "Sets cv_template_path to a local dir or a git URL")
  System(cvc, "rusty_cv_creator", "Resolves the template before building")
  System_Ext(github, "GitHub template repo", "git over SSH/HTTPS at a chosen ref")
  System_Ext(cache, "Template cache", "~/.cache/rusty-cv-creator/templates (repo@ref)")
  Rel(user, cvc, "Runs insert with a git-URL or local cv_template_path")
  Rel(cvc, github, "Clones / fetches / checks out ref via system git")
  Rel(cvc, cache, "Caches per repo@ref; reuses offline on fetch failure")
```

### L2 — Container (delta)

```mermaid
C4Container
  title Container Diagram — template sourcing
  Person(user, "Job Seeker")
  System_Boundary(cvc, "rusty_cv_creator (single binary)") {
    Container(build, "Build Subsystem", "Rust (file_handlers)", "prepare_path_for_new_cv resolves the template before copy_dir")
    Container(tsrc, "Template Sourcing", "Rust (template_source)", "TemplateSource port + adapters + cache")
    Container(runner, "CommandRunner Port", "Rust trait", "SystemRunner runs git; stderr captured for hints")
    Container(config, "Config", "Rust, AppContext", "cv_template_path/_ref/_auth/_cache (INI)")
  }
  System_Ext(git, "system git", "clone/fetch/checkout, gated by ADR-0004 PATH check")
  System_Ext(github, "GitHub template repo", "SSH/HTTPS remote")
  ContainerDb(cache, "Template cache", "filesystem", "repo@ref working trees")
  Rel(user, build, "Invokes insert")
  Rel(build, tsrc, "Resolves template via")
  Rel(build, config, "Reads sourcing keys from")
  Rel(tsrc, runner, "Runs git through")
  Rel(runner, git, "Executes")
  Rel(git, github, "Talks to")
  Rel(tsrc, cache, "Reads & writes (bounded to cache dir)")
```

### L3 — Component: TemplateSource subsystem

```mermaid
C4Component
  title Component Diagram — TemplateSource subsystem
  Container_Boundary(tsrc, "Template Sourcing") {
    Component(detect, "detect_template_source / is_git_url", "template_source.rs", "Auto-detect LOCAL vs GITHUB (D1)")
    Component(port, "TemplateSource", "trait (port)", "resolve(runner) -> local dir | TemplateSourceError")
    Component(local, "LocalDirectory", "adapter", "Passthrough of an existing dir (pure, no write)")
    Component(gh, "GitHubRepository", "adapter", "Clone/fetch/checkout at ref; write bounded to cache dir")
    Component(cache, "TemplateCache", "collaborator", "Cache key + pure CacheAction decision + executor")
    Component(auth, "AuthMode / auth plan", "value", "auto|ssh|token -> git invocation plan (pure)")
    Component(err, "TemplateSourceError", "enum", "auth | network/offline | bad-ref | no-cache -> hint")
  }
  Component_Ext(runner, "CommandRunner", "port", "runs git, captures stderr for classification")
  Component_Ext(tools, "ensure_tools_available", "helpers.rs", "ADR-0004 git PATH gate (probe)")
  Component_Ext(env, "GITHUB_TOKEN", "environment", "Read only at execution; never in INI/argv/.git/config")
  Rel(detect, port, "Constructs a")
  Rel(local, port, "Implements")
  Rel(gh, port, "Implements")
  Rel(gh, tools, "Pre-checks git via")
  Rel(gh, cache, "Asks for CacheAction from")
  Rel(gh, auth, "Builds git args from")
  Rel(gh, runner, "Runs git through")
  Rel(auth, env, "Token mode reads")
  Rel(gh, err, "Classifies git stderr into")
```
