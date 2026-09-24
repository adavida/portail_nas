{
  config,
  lib,
  pkgs,
  ...
}:

let
  cfg = config.services.portail;
in
{
  options.services.portail = {
    enable = lib.mkEnableOption "Portail backend (Rust axum) — frontend (nginx vhost), OpenLDAP and Authelia must be provisioned outside this module.";

    package = lib.mkOption {
      type = lib.types.package;
      description = "Backend (flake `packages.portail-backend`).";
    };

    appUrl = lib.mkOption {
      type = lib.types.str;
      description = "Public URL of the portal (APP_URL of the backend, OIDC redirect URI = appUrl + /callback). Also used to bake VITE_* into `lib.mkFrontend` for the nginx vhost root.";
      example = "http://portail.nas.local";
    };

    issuerUrl = lib.mkOption {
      type = lib.types.str;
      description = "Public URL of Authelia (OIDC_ISSUER_URL).";
      example = "http://auth.portail.nas.local";
    };

    bindAddress = lib.mkOption {
      type = lib.types.str;
      default = "127.0.0.1:3000";
      description = "Backend listen address (BIND_ADDR). In prod, 127.0.0.1 behind nginx.";
    };

    oidcClientId = lib.mkOption {
      type = lib.types.str;
      default = "portail";
      description = "OIDC client_id declared at Authelia and used by the backend.";
    };

    ldap = {
      url = lib.mkOption {
        type = lib.types.str;
        default = "ldap://127.0.0.1";
        description = "Existing LDAP server URL (LDAP_URL) — slapd must be provisioned outside this module.";
      };

      baseDn = lib.mkOption {
        type = lib.types.str;
        default = "dc=example,dc=com";
        description = "LDAP suffix (LDAP_BASE_DN).";
      };

      adminPasswordFile = lib.mkOption {
        type = lib.types.path;
        description = ''
          File (without trailing newline) containing the `cn=admin,<baseDn>` password.
          Used by slapd (olcRootPW) and the backend (LDAP_ADMIN_PW). Secrets: agenix/sops-nix.
        '';
      };
    };

    oidc = {
      clientSecretFile = lib.mkOption {
        type = lib.types.path;
        description = "File (without trailing newline) with the OIDC client_secret shared backend/Authelia.";
      };
    };

    extraBackendEnvironment = lib.mkOption {
      type = lib.types.attrsOf lib.types.str;
      default = { };
      description = "Additional environment variables for the backend (non-secret).";
    };
  };
  config = lib.mkIf cfg.enable {
    systemd.services.portail-backend = {
      description = "Portail backend (Rust axum)";
      wantedBy = [ "multi-user.target" ];
      after = [ "network.target" ];
      environment = {
        APP_URL = cfg.appUrl;
        BIND_ADDR = cfg.bindAddress;
        LDAP_URL = cfg.ldap.url;
        LDAP_BASE_DN = cfg.ldap.baseDn;
        OIDC_CLIENT_ID = cfg.oidcClientId;
        OIDC_ISSUER_URL = cfg.issuerUrl;
        OIDC_REDIRECT_URI = "${cfg.appUrl}/callback";
      }
      // cfg.extraBackendEnvironment;
      serviceConfig = {
        ExecStart = "${cfg.package}/bin/portail-backend";
        ExecStartPre = lib.getExe (
          pkgs.writeShellApplication {
            name = "portail-backend-env";
            text = ''
              printf 'OIDC_CLIENT_SECRET=%s\nLDAP_ADMIN_PW=%s\n' \
                "$(cat ${cfg.oidc.clientSecretFile})" \
                "$(cat ${cfg.ldap.adminPasswordFile})" \
                > /run/portail-backend/env
              chmod 600 /run/portail-backend/env
            '';
          }
        );
        RuntimeDirectory = "portail-backend";
        DynamicUser = true;
        Restart = "on-failure";
        RestartSec = "2s";
      };
    };
  };
}
