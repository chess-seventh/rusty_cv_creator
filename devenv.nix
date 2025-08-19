{ pkgs, lib, config, inputs, ... }:

{
  dotenv.enable = true;

  env.GREET = "Welcome to the Rusty CV Creator";
  # env.OPENSSL_DIR="${pkgs.openssl.dev}";
  # env.OPENSSL_LIB_DIR="${pkgs.openssl.out}/lib";
  env.DATABASE_URL="postgres://rusty_cv:rusty-cv-01@nixos-03.caracara-palermo.ts.net/db_rusty_cv";

  starship = {
    enable = true; 
    config = {
      enable = false;
      path = "~/.config/starship.toml";
    };
  };

  # https://devenv.sh/packages/
  packages = with pkgs; [ 
    git 
    jq
    curl
    gnused
    # pkgs.rustup
    # pkgs.rust-analyzer
    # ruststable
    zlib
    sqlite
    texlive.combined.scheme-small
    diesel-cli
    postgresql

    # cmake
    # gcc
    # openssl
  ];

  # https://devenv.sh/languages/
  languages = {
    nix.enable = true;

    rust = {
      enable = true;
      channel = "stable";
      components = [ "rustc" "cargo" "clippy" "rustfmt" "rust-analyzer" "rust-std" ];
    };

    shell.enable = true;
  };


  # https://devenv.sh/processes/
  processes = {
    cargo-watch.exec = "cargo-watch";
  };

  # https://devenv.sh/services/
  services = { 
    postgres = {
      enable = true;

      initialDatabases = [{ name = "rusty-reggae"; }];

      initialScript = ''
        CREATE EXTENSION IF NOT EXISTS postgis;
      '';
    };

    meilisearch = {
      enable = true;
    };
  };

  # https://devenv.sh/scripts/
  # scripts.hello.exec = ''
  #   echo hello from $GREET
  # '';


  tasks = {
    "bash:channel_up" = {
      exec = "nix-channel --update && sudo nix-channel --update";
      before = [ "devenv:enterShell" ];
    };

    "bash:source_env" = {
      exec = "source $PWD/.env";
      after = [ "devenv:enterShell" ];
    };
  };

  git-hooks.hooks = {
    # lint shell scripts
    shellcheck.enable = true;

    # execute example shell from Markdown files
    mdsh.enable = true;

    # format Python code
    # black.enable = true;

    # some hooks have more than one package, like clippy:
    #
    rustfmt.enable = true;

    clippy = {
      enable = true;
      packageOverrides.cargo = pkgs.cargo;
      packageOverrides.clippy = pkgs.clippy;
      # some hooks provide settings
      settings.allFeatures = true;
      extraPackages = [ pkgs.openssl ];
    };

    commitizen.enable = true;

    gptcommit = {
      enable = true;
    };

    gitlint = {
      enable = true;
      after = [ "gptcommit" ];
    };

    markdownlint = {
      enable = true;
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

    pre-check = {
      description = ''
        runs linters, tests, and builds to prepare commit/push (more extensively than pre-commit hook)
      '';
      exec = ''
        #!/usr/bin/env bash
        set -euxo pipefail

        if [ -f .env.testing ]; then
            source .env.testing
        fi

        cargo fmt --all --check
        cargo clippy --all-targets -- -D warnings
        cargo shear
        cargo audit
        cargo nextest run
        cargo test --examples
        cargo test --doc
      '';
    };
  };

  enterShell = ''
    echo
    echo 💡 Helper scripts to ease development process:
    echo
    ${pkgs.gnused}/bin/sed -e 's| |••|g' -e 's|=| |' <<EOF | ${pkgs.util-linuxMinimal}/bin/column -t | ${pkgs.gnused}/bin/sed -e 's|^|• |' -e 's|••| |g'
    ${lib.generators.toKeyValue {} (lib.mapAttrs (name: value: value.description) config.scripts)}
    EOF
    echo
  '';

  # See full reference at https://devenv.sh/reference/options/
}
