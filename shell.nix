let
  sources = import ./nix/sources.nix;
  mozilla = import sources.nixpkgs-mozilla;
  pkgs = import sources.nixpkgs { overlays = [ mozilla ]; };
in
pkgs.mkShell {
  buildInputs = with pkgs; [
    gpgme
    libgpgerror
    libgit2

    git # for build script to retrieve git hash and add to version info
    latest.rustChannels.stable.rust
  ];
}
