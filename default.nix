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

  root = lib.cleanSourceWith {
    src = toString ./.;
    filter = name: type:
      let baseName = baseNameOf (toString name); in
        !((type == "directory" && baseName == "target")
          || (type == "symlink" && lib.hasPrefix "result" baseName));
  };

  nativeBuildInputs = with pkgs; [
    git # for build script to retrieve git hash and add to version info
    latest.rustChannels.stable.rust
    installShellFiles
  ];

  buildInputs = with pkgs; [
    gpgme
    libgpgerror
    libgit2
  ];

  # NOTE: Completions require the gpg2 binary to be in path in order to complete
  # keys for commands like `passrs init`
  postInstall = ''
    installShellCompletion --fish ${./completions/passrs.fish}
  '';
}
