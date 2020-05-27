let
  sources = import ./nix/sources.nix;
  mozilla = import sources.nixpkgs-mozilla;
  pkgs = import sources.nixpkgs { overlays = [ mozilla ]; };
  lib = pkgs.lib;
  deps = import ./default.nix { };
in
pkgs.mkShell {
  inherit (deps) buildInputs nativeBuildInputs;
}
