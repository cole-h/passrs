{ doCheck ? false
, release ? true
}:
let
  sources = import ./nix/sources.nix;
  mozilla = import sources.nixpkgs-mozilla;
  pkgs = import sources.nixpkgs { overlays = [ mozilla ]; };
  lib = pkgs.lib;
  naersk = pkgs.callPackage sources.naersk { };
in
naersk.buildPackage {
  pname = "passrs";
  version = lib.commitIdFromGitRepo ./.git;

  src =
    let
      dir = src: builtins.readDir src;
      isGit = src: (dir src) ? ".git" && (dir src).".git" == "directory";
      getSrc = src:
        if isGit src then
          fetchGit src
        else
          builtins.filterSource
            (path: type: type != "directory" || builtins.baseNameOf path != "target")
            src;
    in
    getSrc ./.;

  nativeBuildInputs = with pkgs; [
    git # for build script to retrieve git hash and add to version info
    gpgme # `gpgme-config` required by crate gpgme
    installShellFiles
    libgpgerror # `gpg-error-config` required by crate libgpg-error-sys
  ];

  buildInputs = with pkgs; [
    gpgme
    libgit2
    libgpgerror
  ];

  inherit doCheck;
  inherit release;

  cargoTestOptions = opts: opts ++ [ "--" "--include-ignored" "-Z unstable-options" ];

  # NOTE: Completions require the gpg2 binary to be in path in order to complete
  # keys for commands like `passrs init`
  #
  # `|| true` is necessary because postInstall is run for both the
  # dependencies and the actual package.
  # https://github.com/nmattia/naersk/issues/105
  postInstall = ''
    installShellCompletion --fish completions/passrs.fish || true
  '';
}
