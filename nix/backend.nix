{ pkgs }:

pkgs.rustPlatform.buildRustPackage {
  pname = "portail-backend";
  version = "0.1.0";
  src = ../.;
  cargoLock.lockFile = ../Cargo.lock;
  # Integration tests = test LDAP (3891) — outside the nix sandbox.
  doCheck = false;
}
