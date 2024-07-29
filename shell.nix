let
  nixpkgs = fetchTarball "https://github.com/NixOS/nixpkgs/tarball/nixos-24.05";
  pkgs = import nixpkgs { config = {}; overlays = []; };
in

  pkgs.mkShell {
    packages = with pkgs; [
      sqlite
      texlive.combined.scheme-small
      diesel-cli
      postgresql
    ];

    # pkgs.mkShellNoCC {
    #   packages = with pkgs; [
    #     sqlite
    #   ];

    # DATABASE_URL="sqlite://~/.config/rusty-cv-creator/applications.db";
    # DATABASE_URL="/home/seventh/.config/rusty-cv-creator/applications.db";
    DATABASE_URL="postgres://rusty_cv:rusty-cv-01@nixos-01.caracara-palermo.ts.net/db_rusty_cv";


}
