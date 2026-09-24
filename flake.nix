{
  description = "Portail — backend Rust axum + frontend React Vite, NixOS module (backend service only — nginx/OpenLDAP/Authelia provisioned by the host)";

  inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixos-26.05";

  outputs =
    { self, nixpkgs }:
    let
      system = "x86_64-linux";
      pkgs = nixpkgs.legacyPackages.${system};
    in
    rec {
      # Frontend with OIDC URLs baked at build time — overridden by the NixOS
      # module via overrideAttrs.
      mkFrontend = pkgs.callPackage ./nix/frontend.nix { };

      packages.${system} = {
        portail-backend = pkgs.callPackage ./nix/backend.nix { };
        portail-frontend = mkFrontend { };
        default = self.packages.${system}.portail-backend;
      };

      nixosModules.default = ./nixos/portail.nix;

      formatter.${system} = nixpkgs.legacyPackages.${system}.nixfmt-tree;

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
