{
  description = "Portail — backend Rust axum + frontend React Vite, module NixOS (openldap, authelia, nginx)";

  # Pinned to the nixpkgs from devenv.lock (nixpkgs-unstable @ c7def04).
  inputs.nixpkgs.url = "github:NixOS/nixpkgs/c7def046b9a883d46974757852106483d741586f";

  outputs =
    { self, nixpkgs }:
    let
      system = "x86_64-linux";
      pkgs = nixpkgs.legacyPackages.${system};
    in
    {
      # Frontend with OIDC URLs baked at build time (import.meta.env) — overridden
      # by the NixOS module via overrideAttrs.
      lib.mkFrontend =
        {
          appUrl ? "http://localhost:5173",
          oidcIssuerUrl ? "https://127.0.0.1:9091",
          oidcRedirectUri ? "http://localhost:5173/callback",
        }:
        pkgs.buildNpmPackage {
          pname = "portail-frontend";
          version = "0.1.0";
          src = ./frontend;
          npmDepsHash = "sha256-m7ZXnXFBVNeVI4jlhAjr5hRznOoHdMMTT5xugrkDZVI=";
          VITE_APP_URL = appUrl;
          VITE_BACKEND_URL = appUrl;
          VITE_OIDC_ISSUER_URL = oidcIssuerUrl;
          VITE_OIDC_REDIRECT_URI = oidcRedirectUri;
        };

      packages.${system} = {
        portail-backend = pkgs.rustPlatform.buildRustPackage {
          pname = "portail-backend";
          version = "0.1.0";
          src = ./.;
          cargoLock.lockFile = ./Cargo.lock;
          # Integration tests = test LDAP (3891) — outside the nix sandbox.
          doCheck = false;
        };

        portail-frontend = self.lib.mkFrontend { };

        default = self.packages.${system}.portail-backend;
      };

      nixosModules.default = ./nixos/portail.nix;

      nixosConfigurations.test = nixpkgs.lib.nixosSystem {
        inherit system;
        modules = [
          self.nixosModules.default
          ./nixos/test-vm.nix
          {
            services.portail = {
              package = self.packages.${system}.portail-backend;
              frontendPackage = self.packages.${system}.portail-frontend;
            };
          }
        ];
      };

      devShells.${system}.default = pkgs.mkShell {
        packages = with pkgs; [
          cargo
          clippy
          rustc
          rustfmt
        ];
      };
    };
}
