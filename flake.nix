{
  description = "Hello rust flake";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixpkgs-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = {
    self,
    flake-utils,
    nixpkgs,
    rust-overlay,
  }:
    {nixosModules.timelapse = import ./modules/timelapse/default.nix self;}
    // flake-utils.lib.eachDefaultSystem (system: let
      overlays = [(import rust-overlay)];
      pkgs = import nixpkgs {inherit system overlays;};
      rust = pkgs.rust-bin.stable.latest.default.override {
        extensions = ["rust-src"];
      };
      rustPlatform = pkgs.makeRustPlatform {
        rustc = rust;
        cargo = rust;
      };
    in {
      packages = rec {
        timelapse = rustPlatform.buildRustPackage rec {
          pname = "timelapse";
          version = "0.1.0";

          src = ./.;
          cargoLock = {
            lockFile = ./Cargo.lock;
          };

          doCheck = true;

          env = {};
        };

        default = timelapse;
      };
      apps = rec {
        ip = flake-utils.lib.mkApp {
          drv = self.packages.${system}.timelapse;
          exePath = "/bin/timelapse";
        };
        default = ip;
      };
      devShells.default = pkgs.mkShell {
        buildInputs = with pkgs; [rust];
        shellHook = ''
          export CARGO_HOME=$(pwd)/cargo
        '';
      };
    });
}
