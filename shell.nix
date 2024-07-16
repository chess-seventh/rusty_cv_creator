let
  nixpkgs = fetchTarball "https://github.com/NixOS/nixpkgs/tarball/nixos-24.05";
  pkgs = import nixpkgs { config = {}; overlays = []; };
in

pkgs.mkShellNoCC {
  packages = with pkgs; [
    sqlite
    texlive.combined.scheme-small
  ];

DATABASE_URL="sqlite://$HOME/.config/rusty-cv-creator/applications.db";
}
