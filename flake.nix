{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    crane.url = "github:ipetkov/crane";
    crane.inputs.nixpkgs.follows = "nixpkgs";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, crane, flake-utils }:
    {
      overlays.default = final: prev: {
        inherit (self.packages.${prev.stdenv.hostPlatform.system}) fsh;
      };
      homeModules.fsh = import ./home.nix;
    } // flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs {
          system = system;
        };
        inherit (pkgs) lib;

        craneLib = crane.lib.${system};

        src = craneLib.cleanCargoSource ./.;

        buildInputs = [
          pkgs.pkg-config pkgs.openssl
        ] ++ lib.optionals pkgs.stdenv.isDarwin [
          # Additional darwin specific inputs can be set here
          pkgs.libiconv
        ];

        cargoArtifacts = craneLib.buildDepsOnly {
          inherit src buildInputs;
        };

        # Build the actual crate itself, reusing the dependency
        # artifacts from above.
        fsh = craneLib.buildPackage {
          inherit cargoArtifacts src buildInputs;
        };
      in
    {
      checks = {
        # Build the crate as part of `nix flake check` for convenience
        inherit fsh;

        # Run clippy (and deny all warnings) on the crate source,
        # again, resuing the dependency artifacts from above.
        #
        # Note that this is done as a separate derivation so that
        # we can block the CI if there are issues here, but not
        # prevent downstream consumers from building our crate by itself.
        fsh-clippy = craneLib.cargoClippy {
          inherit cargoArtifacts src buildInputs;
          cargoClippyExtraArgs = "--all-targets -- --deny warnings";
        };

        # Check formatting
        fsh-fmt = craneLib.cargoFmt {
          inherit src;
        };
      };

      packages.fsh = fsh;
      packages.default = self.packages.${system}.fsh;

      apps.default = flake-utils.lib.mkApp {
        drv = fsh;
      };

      devShells.default = pkgs.mkShell {
        inputsFrom = builtins.attrValues self.checks;

        # Extra inputs can be added here
        nativeBuildInputs = with pkgs; [
          cargo
          rustc
          rustfmt
          pkg-config
          openssl
        ];
      };
    });
}
