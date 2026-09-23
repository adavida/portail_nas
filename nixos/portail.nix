{
  config,
  lib,
  pkgs,
  ...
}:

let
  cfg = config.services.portail;
  # Frontend rebuilt with the deployment URLs (VITE_* baked at build time
  # by flake lib.mkFrontent, adjusted here via overrideAttrs — the npm hash
  # doesn't change, only the final build is redone).
  frontend = cfg.frontendPackage.overrideAttrs (old: {
    VITE_APP_URL = cfg.appUrl;
    VITE_BACKEND_URL = cfg.appUrl;
    VITE_OIDC_ISSUER_URL = cfg.issuerUrl;
    VITE_OIDC_REDIRECT_URI = "${cfg.appUrl}/callback";
  });
in
{
  options.services.portail = {
    enable = lib.mkEnableOption "Portail (backend Rust + frontend React + LDAP + Authelia)";

    package = lib.mkOption {
      type = lib.types.package;
      description = "Backend (flake `packages.portail-backend`).";
    };

    frontendPackage = lib.mkOption {
      type = lib.types.package;
      description = "Frontend (flake `packages.portail-frontend` — VITE URLs overridden per `appUrl`/`issuerUrl`).";
    };

    vhost = lib.mkOption {
      type = lib.types.str;
      description = "Host name (nginx) serving the frontend and proxying /api.";
      example = "portail.nas.local";
    };

    authVhost = lib.mkOption {
      type = lib.types.str;
      default = "auth.${cfg.vhost}";
      defaultText = lib.literalExpression ''"auth.''${config.services.portail.vhost}"'';
      description = "Host name (nginx) serving Authelia (OIDC issuer).";
    };

    appUrl = lib.mkOption {
      type = lib.types.str;
      default = "http://${cfg.vhost}";
      defaultText = lib.literalExpression ''"http://''${config.services.portail.vhost}"'';
      description = "Public URL of the portal (baked into the frontend, APP_URL of the backend). Switch to https:// here if TLS is configured on the vhost.";
    };

    issuerUrl = lib.mkOption {
      type = lib.types.str;
      default = "http://${cfg.authVhost}";
      defaultText = lib.literalExpression ''"http://''${config.services.portail.authVhost}"'';
      description = "Public URL of Authelia (OIDC_ISSUER_URL). Switch to https:// here if TLS is configured.";
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
      baseDn = lib.mkOption {
        type = lib.types.str;
        default = "dc=example,dc=com";
        description = "LDAP suffix (LDAP_BASE_DN) — also the openldap database suffix.";
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
    services.nginx = {
      enable = lib.mkDefault true;
      virtualHosts.${cfg.vhost} = {
        root = frontend;
        locations."/".tryFiles = lib.mkDefault "$uri /index.html";
        locations."/api".proxyPass = "http://${cfg.bindAddress}";
      };
      virtualHosts.${cfg.authVhost} = {
        locations."/".proxyPass = "http://127.0.0.1:9091";
        locations."/".proxyWebsockets = true;
      };
    };

    networking.firewall.allowedTCPPorts = lib.mkDefault [
      80
      443
    ];

    services.openldap = {
      enable = lib.mkDefault true;
      urlList = lib.mkDefault [ "ldap://127.0.0.1:389/" ];
      settings.children = {
        "cn=schema".includes = [
          "${config.services.openldap.package}/etc/schema/core.ldif"
          "${config.services.openldap.package}/etc/schema/cosine.ldif"
          "${config.services.openldap.package}/etc/schema/inetorgperson.ldif"
        ];
        "olcDatabase={-1}frontend".attrs = {
          objectClass = "olcDatabaseConfig";
          olcDatabase = "{-1}frontend";
        };
        "olcDatabase={0}config".attrs = {
          objectClass = "olcDatabaseConfig";
          olcDatabase = "{0}config";
          olcAccess = [ "{0}to * by * none break" ];
        };
        "olcDatabase={1}mdb".attrs = {
          objectClass = [
            "olcDatabaseConfig"
            "olcMdbConfig"
          ];
          olcDatabase = "{1}mdb";
          olcDbDirectory = "/var/lib/openldap/portail";
          olcDbIndex = [ "objectClass eq" ];
          olcSuffix = cfg.ldap.baseDn;
          olcRootDN = "cn=admin,${cfg.ldap.baseDn}";
          # read by slapadd on first startup (sensitive: file 600)
          olcRootPW = {
            path = cfg.ldap.adminPasswordFile;
          };
          olcAccess = [ "{0}to * by * read break" ];
        };
      };
    };

    # Seed ou=people / ou=groups once (the DB persists across boots).
    systemd.services.openldap.preStart = lib.getExe (
      pkgs.writeShellApplication {
        name = "portail-ldap-seed";
        runtimeInputs = [ config.services.openldap.package ];
        text = ''
          if [ ! -f /var/lib/openldap/portail/data.mdb ]; then
            slapadd -F /etc/openldap/slapd.d -b ${cfg.ldap.baseDn} -l ${pkgs.writeText "portail-seed.ldif" ''
              dn: ${cfg.ldap.baseDn}
              objectClass: dcObject
              objectClass: organization
              dc: portail
              o: Portail

              dn: ou=people,${cfg.ldap.baseDn}
              objectClass: organizationalUnit
              ou: people

              dn: ou=groups,${cfg.ldap.baseDn}
              objectClass: organizationalUnit
              ou: groups
            ''}
          fi
        '';
      }
    );

    services.authelia.instances.portail = {
      enable = lib.mkDefault true;
      secrets = {
        jwtSecretFile = lib.mkDefault "/etc/portail/authelia/jwt-secret";
        storageEncryptionKeyFile = lib.mkDefault "/etc/portail/authelia/storage-encryption-key";
        oidcHmacSecretFile = lib.mkDefault "/etc/portail/authelia/oidc-hmac-secret";
        oidcIssuerPrivateKeyFile = lib.mkDefault "/etc/portail/authelia/jwks-key.pem";
      };
      environmentVariables = {
        X_AUTHELIA_CONFIG_FILTERS = "template";
        AUTHELIA_AUTHENTICATION_BACKEND_LDAP_PASSWORD_FILE = "${cfg.ldap.adminPasswordFile}";
      };
      settings = {
        server.address = "tcp://127.0.0.1:9091";
        log.level = "info";
        authentication_backend.ldap = {
          address = "ldap://127.0.0.1";
          base_dn = cfg.ldap.baseDn;
          user = "cn=admin,${cfg.ldap.baseDn}";
          additional_users_dn = "ou=people";
          users_filter = "(&({username_attribute}={input}))";
          additional_groups_dn = "ou=groups";
          groups_filter = "(member={dn})";
        };
        access_control.default_policy = "one_factor";
        session = {
          name = "portail_session";
          cookies = [
            {
              domain = cfg.authVhost;
              authelia_url = cfg.issuerUrl;
            }
          ];
        };
        storage.local.path = "/var/lib/authelia-portail/db.sqlite3";
        notifier.filesystem.filename = "/var/lib/authelia-portail/notifications.txt";
        identity_providers.oidc = {
          clients = [
            {
              client_id = cfg.oidcClientId;
              client_name = "Portail";
              public = false;
              authorization_policy = "one_factor";
              consent_mode = "implicit";
              redirect_uris = [ "${cfg.appUrl}/callback" ];
              scopes = [
                "openid"
                "groups"
                "email"
                "profile"
                "offline_access"
              ];
              grant_types = [
                "authorization_code"
                "refresh_token"
              ];
              response_types = [ "code" ];
              token_endpoint_auth_method = "client_secret_basic";
              # read by the template filter (X_AUTHELIA_CONFIG_FILTERS) via AUTHELIA_*_FILE
              client_secret = "{{ secret \"${cfg.oidc.clientSecretFile}\" }}";
            }
          ];
        };
      };
    };

    systemd.services.portail-backend = {
      description = "Portail backend (Rust axum)";
      wantedBy = [ "multi-user.target" ];
      wants = [
        "openldap.service"
        "authelia-portail.service"
      ];
      after = [
        "network.target"
        "openldap.service"
        "authelia-portail.service"
      ];
      environment = {
        APP_URL = cfg.appUrl;
        BIND_ADDR = cfg.bindAddress;
        LDAP_URL = "ldap://127.0.0.1:389";
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
        EnvironmentFile = "/run/portail-backend/env";
        RuntimeDirectory = "portail-backend";
        DynamicUser = true;
        Restart = "on-failure";
        RestartSec = "2s";
      };
    };
  };
}
