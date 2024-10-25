let
  # moz_overlay = import (builtins.fetchTarball "https://github.com/oxalica/rust-overlay/archive/master.tar.gz");
  nixpkgs = import <nixpkgs> { overlays = [  ]; };
  # ruststable = (nixpkgs.latest.rustChannels.stable.rust.override {
    # extensions = [ "rust-src" "rls-preview" "rust-analysis" "rustfmt-preview" ];
  # });
  pkgs = nixpkgs;  # Assign nixpkgs to pkgs for easier reference
in
  pkgs.mkShell {
    buildInputs = [
      pkgs.openssl
      pkgs.rustup
      # pkgs.rust-analyzer
      # ruststable
      pkgs.cmake
      pkgs.zlib
      pkgs.sqlite
      pkgs.texlive.combined.scheme-full
      pkgs.diesel-cli
      pkgs.postgresql
    ];

    shellHook = ''
      export OPENSSL_DIR="${pkgs.openssl.dev}"
      export OPENSSL_LIB_DIR="${pkgs.openssl.out}/lib"
      export DATABASE_URL="postgres://rusty_cv:rusty-cv-01@nixos-01.caracara-palermo.ts.net/db_rusty_cv";
    '';
  }

