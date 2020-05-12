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

  src = builtins.filterSource
    (path: type: type != "directory" || builtins.baseNameOf path != "target")
    (toString ./.);

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

  # NOTE: Completions require the gpg2 binary to be in path in order to complete
  # keys for commands like `passrs init`
  postInstall = ''
    installShellCompletion --fish ${./completions/passrs.fish}
  '';
}
