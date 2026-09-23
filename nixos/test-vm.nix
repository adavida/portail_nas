# Minimal configuration for `nixos-rebuild build-vm --flake .#test` —
# end-to-end demo/troubleshooting of the module. Secrets generated in the
# store: test only, never in prod (agenix/sops-nix).
{
  pkgs,
  lib,
  ...
}:

let
  # Test secrets generated at evaluation time (world-readable — test VM only).
  genSecret =
    name: cmd:
    pkgs.runCommand name { } ''
      ${cmd}
    '';
in
{
  boot.loader.grub.device = "/dev/vda";
  fileSystems."/" = {
    device = "/dev/disk/by-label/nixos";
    fsType = "ext4";
  };

  users.users.root.initialPassword = "root";

  virtualisation.vmVariant = {
    virtualisation.graphics = false;
    virtualisation.memorySize = 2048;
    virtualisation.forwardPorts = [
      {
        from = "host";
        host.port = 8080;
        guest.port = 80;
      }
    ];
  };

  services.portail = {
    enable = true;
    vhost = "portail.test";
    ldap.baseDn = "dc=example,dc=com";
    ldap.adminPasswordFile = genSecret "portail-test-ldap-pw" ''
      echo -n adminpw > "$out"
    '';
    oidc.clientSecretFile = genSecret "portail-test-client-secret" ''
      echo -n portail-test-client-secret > "$out"
    '';
  };

  # Authelia secret files expected by the module (mkDefault above).
  environment.etc = {
    "portail/authelia/jwt-secret".source = genSecret "portail-test-jwt" ''
      ${pkgs.openssl}/bin/openssl rand -hex 32 > "$out"
    '';
    "portail/authelia/storage-encryption-key".source = genSecret "portail-test-enc" ''
      ${pkgs.openssl}/bin/openssl rand -hex 32 > "$out"
    '';
    "portail/authelia/oidc-hmac-secret".source = genSecret "portail-test-hmac" ''
      ${pkgs.openssl}/bin/openssl rand -hex 32 > "$out"
    '';
    "portail/authelia/jwks-key.pem".source = genSecret "portail-test-jwks" ''
      ${pkgs.openssl}/bin/openssl genrsa 2048 > "$out"
    '';
  };
}
