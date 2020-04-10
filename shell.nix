let
  moz_overlay = import (
    fetchTarball {
      url = "https://github.com/mozilla/nixpkgs-mozilla/archive/e912ed483e980dfb4666ae0ed17845c4220e5e7c.tar.gz";
      sha256 = "08fvzb8w80bkkabc1iyhzd15f4sm7ra10jn32kfch5klgl0gj3j3";
    }
  );
  pkgs = import <nixpkgs> { overlays = [ moz_overlay ]; };
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
