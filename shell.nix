let
  nixpkgs = fetchTarball "https://github.com/NixOS/nixpkgs/tarball/nixos-24.05";
  pkgs = import nixpkgs { config = {}; overlays = []; };
in

  pkgs.mkShell {
    packages = with pkgs; [
      sqlite
      texlive.combined.scheme-small
    ];

    # pkgs.mkShellNoCC {
    #   packages = with pkgs; [
    #     sqlite
    #   ];

    DATABASE_URL="sqlite://~/.config/rusty-cv-creator/applications.db";
}
