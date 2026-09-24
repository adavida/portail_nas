{
  config,
  lib,
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
          File containing the `cn=admin,<baseDn>` password.
          Used by slapd (olcRootPW) and by the backend via `LDAP_ADMIN_PW` = path (read at startup). Secrets: agenix/sops-nix.
        '';
      };
    };

    oidc = {
      clientSecretFile = lib.mkOption {
        type = lib.types.path;
        description = "File with the OIDC client_secret shared backend/Authelia — passed to the backend as `OIDC_CLIENT_SECRET` = path (read at startup, trailing newline trimmed).";
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
      # Secret vars hold the path of a file whose content is the secret — the
      # backend reads and trims the file at startup. The service user must have
      # read access (e.g. agenix/sops-nix `owner`/`group` on the secret files).
      environment = {
        APP_URL = cfg.appUrl;
        BIND_ADDR = cfg.bindAddress;
        LDAP_URL = cfg.ldap.url;
        LDAP_BASE_DN = cfg.ldap.baseDn;
        LDAP_ADMIN_PW = toString cfg.ldap.adminPasswordFile;
        OIDC_CLIENT_ID = cfg.oidcClientId;
        OIDC_ISSUER_URL = cfg.issuerUrl;
        OIDC_CLIENT_SECRET = toString cfg.oidc.clientSecretFile;
        OIDC_REDIRECT_URI = "${cfg.appUrl}/callback";
      }
      // cfg.extraBackendEnvironment;
      serviceConfig = {
        ExecStart = "${cfg.package}/bin/portail-backend";
        DynamicUser = true;
        Restart = "on-failure";
        RestartSec = "60s";
      };
    };
  };
}
