{
  description = "A crabby rewrite of `pass`, the standard unix password manager";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/master";
    naersk = { url = "github:nmattia/naersk"; inputs.nixpkgs.follows = "nixpkgs"; };
  };

  outputs = { self, nixpkgs, naersk }:
    let
      systems = [ "x86_64-linux" "i686-linux" "aarch64-linux" ];

      forAllSystems = f: nixpkgs.lib.genAttrs systems
        (system: f {
          inherit system;
          pkgs = nixpkgsFor.${system};
        });

      # Memoize nixpkgs for different platforms for efficiency.
      nixpkgsFor = forAllSystems
        ({ system, ... }: import nixpkgs { inherit system; });

      deps = pkgs: with pkgs; {
        nativeBuildInputs = [
          pkg-config
          git # for build script to retrieve git hash and add to version info
          gpgme # `gpgme-config` required by crate gpgme
          installShellFiles
          libgpgerror # `gpg-error-config` required by crate libgpg-error-sys
        ];

        buildInputs = [
          gpgme
          libgit2
          libgpgerror
        ];
      };

      # A bit hacky, but whatever. At least `nix flake check` doesn't complain
      # about this not being a package anymore.
      passrs =
        { release ? true
        , doCheck ? false
        , pkgs
        }:
        let
          inherit (pkgs.callPackage naersk { }) buildPackage;
          version = self.shortRev or "dirty";
        in
        buildPackage {
          pname = "passrs";
          inherit version;

          src = ./.;

          PASSRS_REV = version;
          RUST_BACKTRACE = 1;

          inherit (deps pkgs) nativeBuildInputs buildInputs;

          inherit release doCheck;

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
        };
    in
    {
      defaultPackage = forAllSystems ({ system, pkgs, ... }:
        self.packages.${system}.passrs);

      packages = forAllSystems ({ system, pkgs, ... }: {
        passrs = passrs {
          inherit pkgs;
        };

        test = passrs {
          inherit pkgs;
          release = false;
          doCheck = true;
        };
      });

      devShell = forAllSystems ({ system, pkgs, ... }:
        pkgs.stdenv.mkDerivation {
          name = "passrs";

          inherit (deps pkgs) buildInputs nativeBuildInputs;
        });
    };
}
