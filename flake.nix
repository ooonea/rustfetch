{
  description = "purefetch — a fast, zero-dependency system information tool in Rust";

  # Only nixpkgs — no flake-utils — to keep the dependency graph as small as the
  # tool's own (which is empty).
  inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";

  outputs =
    { self, nixpkgs }:
    let
      # purefetch is Linux-only (raw /proc, /sys and Linux syscalls).
      systems = [
        "x86_64-linux"
        "aarch64-linux"
      ];
      forAll = f: nixpkgs.lib.genAttrs systems (system: f nixpkgs.legacyPackages.${system});
    in
    {
      packages = forAll (
        pkgs: rec {
          purefetch = pkgs.rustPlatform.buildRustPackage {
            pname = "purefetch";
            version = (builtins.fromTOML (builtins.readFile ./Cargo.toml)).package.version;
            src = self;
            cargoLock.lockFile = ./Cargo.lock;
            meta = {
              description = "A fast, fastfetch-style system information tool written entirely in Rust with zero dependencies";
              homepage = "https://github.com/ooonea/purefetch";
              license = with pkgs.lib.licenses; [
                mit
                asl20
              ];
              mainProgram = "purefetch";
              platforms = pkgs.lib.platforms.linux;
            };
          };
          default = purefetch;
        }
      );

      apps = forAll (pkgs: {
        default = {
          type = "app";
          program = "${self.packages.${pkgs.system}.purefetch}/bin/purefetch";
        };
      });

      devShells = forAll (pkgs: {
        default = pkgs.mkShell {
          packages = [
            pkgs.cargo
            pkgs.rustc
            pkgs.rustfmt
            pkgs.clippy
          ];
        };
      });

      overlays.default = final: prev: {
        purefetch = self.packages.${prev.system}.default;
      };
    };
}
