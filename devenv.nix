{
  imports,
  pkgs,
  lib,
  config,
  inputs,
  ...
}:
{
  # Optional shared modules: present on the maintainer's machine
  # (~/devenv_shared/*), absent in CI. `builtins.filter pathExists` keeps the
  # environment evaluable in CI; the packages those modules add that CI needs
  # (just, cargo-nextest/shear/llvm-cov, treefmt + formatters) are declared
  # directly below so this devenv is self-sufficient.
  imports = builtins.filter builtins.pathExists [
    "${builtins.getEnv "HOME"}/devenv_shared/shared_pkgs.nix"
    "${builtins.getEnv "HOME"}/devenv_shared/rust_pkgs.nix"
    "${builtins.getEnv "HOME"}/devenv_shared/git_hooks.nix"
  ];

  dotenv.enable = true;

  env.GREET = "Welcome to the Rusty CV Creator";
  env.DATABASE_URL = "postgres://rusty_cv:rusty-cv-01@nixos-03.caracara-palermo.ts.net/db_rusty_cv";

  packages = with pkgs; [
    zlib
    sqlite
    tectonic
    diesel-cli
    postgresql
    # Self-sufficiency for CI (these otherwise come from ~/devenv_shared/* or the
    # maintainer's global profile): build tool + test/lint tooling + formatters.
    just
    cargo-nextest
    cargo-shear
    cargo-llvm-cov
    treefmt
    yamlfmt
    toml-sort
  ];

  languages = {
    nix.enable = true;

    rust = {
      enable = true;
      channel = "nightly";
      components = [
        "rustc"
        "cargo"
        "clippy"
        "rustfmt"
        "rust-analyzer"
        "rust-std"
        "llvm-tools-preview"
      ];
    };

    shell.enable = true;
  };

  processes = {
    cargo-watch.exec = "cargo-watch";
  };

  claude.code = {
    enable = true;
    hooks = {
      # Protect sensitive files (PreToolUse hook)
      protect-secrets = {
        enable = true;
        name = "Protect sensitive files";
        hookType = "PreToolUse";
        matcher = "^(Edit|MultiEdit|Write)$";
        command = ''
          # Read the JSON input from stdin
          json=$(cat)
          file_path=$(echo "$json" | jq -r '.file_path // empty')

          if [[ "$file_path" =~ \.(env|secret)$ ]]; then
            echo "Error: Cannot edit sensitive files"
            exit 1
          fi
        '';
      };

      # Track completion (Stop hook)
      track-completion = {
        enable = true;
        name = "Track when Claude finishes";
        hookType = "Stop";
        command = ''echo "Claude finished at $(date)" >> ./claude/claude-sessions.log'';
      };
    };
  };

  tasks = {
    "bash:source_env" = {
      exec = "source $PWD/.env";
      after = [ "devenv:enterShell" ];
    };
  };

  git-hooks.hooks = {
    rusty-commit-saver = {
      enable = true;
      name = "🦀 Rusty Commit Saver";
      stages = [ "post-commit" ];
      after = [
        "commitizen"
        "gitlint"
        "gptcommit"
      ];
      entry = "${
        inputs.rusty-commit-saver.packages.${pkgs.stdenv.hostPlatform.system}.default
      }/bin/rusty-commit-saver";
      pass_filenames = false;
      language = "system";
      always_run = true;
    };

    check-merge-conflicts = {
      name = "🔒 Check Merge Conflicts";
      enable = true;
      stages = [ "pre-commit" ];
    };

    detect-aws-credentials = {
      name = "💭 Detect AWS Credentials";
      enable = true;
      stages = [ "pre-commit" ];
    };

    detect-private-keys = {
      name = "🔑 Detect Private Keys";
      enable = true;
      stages = [ "pre-commit" ];
    };

    end-of-file-fixer = {
      name = "🔚 End of File Fixer";
      enable = true;
      stages = [ "pre-commit" ];
    };

    mixed-line-endings = {
      name = "🔀 Mixed Line Endings";
      enable = true;
      stages = [ "pre-commit" ];
    };

    trim-trailing-whitespace = {
      name = "✨ Trim Trailing Whitespace";
      enable = true;
      stages = [ "pre-commit" ];
    };

    shellcheck = {
      name = "✨ Shell Check";
      enable = true;
      stages = [ "pre-commit" ];
    };

    mdsh = {
      enable = true;
      name = "✨ MDSH";
      stages = [ "pre-commit" ];
    };

    treefmt = {
      name = "🌲 TreeFMT";
      enable = true;
      settings.formatters = [
        pkgs.nixfmt
        pkgs.deadnix
        pkgs.yamlfmt
        pkgs.rustfmt
        pkgs.toml-sort
      ];
      stages = [ "pre-commit" ];
    };

    # clippy = {
    #   name = "✂️ Clippy";
    #   enable = true;
    #   entry = "cargo clippy --all-targets -- -W clippy::pedantic -A clippy::must-use-candidate";
    #   language = "system";
    #   settings.allFeatures = true;
    #   extraPackages = [ pkgs.openssl ];
    #   stages = [ "pre-commit" ];
    #   pass_filenames = false;
    # };

    commitizen = {
      name = "✨ Commitizen";
      enable = true;
      stages = [ "post-commit" ];
    };

    gptcommit = {
      name = "🤖 GPT Commit";
      enable = true;
    };

    gitlint = {
      name = "✨ GitLint";
      enable = true;
      after = [ "gptcommit" ];
    };

    markdownlint = {
      name = "✨ MarkdownLint";
      enable = true;
      stages = [ "pre-commit" ];
      settings.configuration = {
        MD033 = false;
        MD013 = {
          line_length = 120;
          tables = false;
        };
        MD041 = false;
      };
    };
  };

  scripts = {
    install_pre_hooks = {
      description = "Install Pre Hooks, such as gptcommit";
      exec = ''
        #!/usr/bin/env bash
        set -euxo pipefail
        gptcommit install
        gptcommit config set openai.model gpt-4-turbo
        gptcommit config set output.conventional_commit true
      '';
    };

    # cclippy = {
    #   description = ''
    #     Run clippy
    #   '';
    #   exec = ''
    #     cargo clippy --all-targets -- -W clippy::pedantic -A clippy::missing_errors_doc -A clippy::must_use_candidate -A clippy::module_name_repetitions -A clippy::doc_markdown -A clippy::missing_panics_doc
    #   '';
    # };

    pre-check = {
      description = ''
        runs linters, tests, and builds to prepare commit/push (more extensively than pre-commit hook)
      '';
      exec = ''
        #!/usr/bin/env bash
        set -euo pipefail

        if [ -f .env.testing ]; then
            source .env.testing
        fi

        treefmt src/
        cargo clippy --all-targets -- -D warnings
        cargo shear --fix
        cargo llvm-cov --html nextest --no-fail-fast
      '';
    };

    # devhelp = {
    #   description = ''
    #     Show helper commands for devenv.nix
    #   '';
    #   exec = ''
    #     echo
    #     echo 💡 Helper scripts to ease development process:
    #     echo
    #     ${pkgs.gnused}/bin/sed -e 's| |••|g' -e 's|=| |' <<EOF | ${pkgs.util-linuxMinimal}/bin/column -t | ${pkgs.gnused}/bin/sed -e 's|^|• |' -e 's|••| |g'
    #     ${lib.generators.toKeyValue { } (lib.mapAttrs (name: value: value.description) config.scripts)}
    #     EOF
    #     echo
    #   '';
    # };
  };

  enterShell = ''
    echo "Sourcing .env with evaluated command substitution…"
    if [ -f ".env" ]; then
      eval "$(<.env)"
    fi

    echo
    echo 💡 Helper scripts to ease development process:
    echo
    ${pkgs.gnused}/bin/sed -e 's| |••|g' -e 's|=| |' <<EOF | ${pkgs.util-linuxMinimal}/bin/column -t | ${pkgs.gnused}/bin/sed -e 's|^|• |' -e 's|••| |g'
    ${lib.generators.toKeyValue { } (lib.mapAttrs (name: value: value.description) config.scripts)}
    EOF
    echo
  '';
}
