{
  config,
  lib,
  pkgs,
  ...
}:
let
  cfg = config.services.portail;
  sortedApps = lib.sort (a: b: a.name < b.name) cfg.apps;
  # The public /apps.json must never leak the `oidc` sub-object (paths, scopes).
  appsFile = pkgs.writeText "portail-apps.json" (
    builtins.toJSON (
      map (a: {
        inherit (a)
          name
          url
          description
          icon
          ;
      }) sortedApps
    )
  );
  oidcClients = map (
    a:
    let
      id = if a.oidc.clientId != "" then a.oidc.clientId else lib.strings.toLower a.name;
    in
    {
      inherit id;
      public = a.oidc.public;
      redirectUris = a.oidc.redirectUris;
      scopes = a.oidc.scopes;
      secretPath =
        if a.oidc.secretFile != null then
          toString a.oidc.secretFile
        else
          "${cfg.oidc.secretsDir}/${id}_secret";
    }
  ) (lib.filter (a: a.oidc.enable) cfg.apps);
  mergeClient =
    c:
    ''
      id='${c.id}'
      redirects='${builtins.toJSON c.redirectUris}'
      scopes='${builtins.toJSON c.scopes}'
    ''
    + (
      if c.public then
        ''
          secret=""
        ''
      else
        ''
          secret_file='${c.secretPath}'
          if [ ! -f "$secret_file" ]; then umask 077; head -c 16 /dev/urandom | od -An -tx1 | tr -d ' \n' > "$secret_file"; fi
          secret="$(cat "$secret_file" | tr -d '[:space:]')"
        ''
    )
    + ''
      jq --arg id "$id" --arg secret "$secret" --argjson redirects "$redirects" --argjson scopes "$scopes" 'if any(.[]; .id == $id) then map(if .id == $id then .secret = $secret | .redirect_uris = $redirects | .scopes = $scopes | .managed = true else . end) else . + [{"id": $id, "secret": $secret, "redirect_uris": $redirects, "scopes": $scopes, "managed": true}] end' "$clients" > "$clients.tmp" && mv "$clients.tmp" "$clients" && chmod 600 "$clients"
    '';
  # Runs after the backend started (it creates clients.json on first boot);
  # clients.json is re-read on every request, so the upsert needs no restart.
  # Entries the module wrote (`.managed = true`) but that are no longer in
  # `apps` are pruned, along with their generated `<clientId>_secret` file;
  # manually added entries and the portal client are never touched.
  registerOidcClients = pkgs.writeShellScript "portail-oidc-clients" (
    ''
      export PATH="${
        lib.makeBinPath [
          pkgs.coreutils
          pkgs.jq
        ]
      }:$PATH"
      clients="${cfg.oidc.secretsDir}/clients.json"
      for i in $(seq 30); do [ -f "$clients" ] && break; sleep 1; done
      if [ ! -f "$clients" ]; then echo "clients.json not found after 30s - skipping OIDC client registration" >&2; exit 0; fi
      ids='${builtins.toJSON (map (c: c.id) oidcClients)}'
      jq -r --argjson ids "$ids" '.[] | select(.managed == true and (.id | IN($ids[]) | not)) | .id' "$clients" | while read -r gone; do rm -f "${cfg.oidc.secretsDir}/$gone"_secret; done
      jq --argjson ids "$ids" 'map(select(.managed != true or (.id | IN($ids[]))))' "$clients" > "$clients.tmp" && mv "$clients.tmp" "$clients" && chmod 600 "$clients"
    ''
    + lib.concatMapStringsSep "\n" mergeClient oidcClients
  );
in
{
  options.services.portail = {
    enable = lib.mkEnableOption "Portail backend (Rust axum, embedded OIDC provider) — frontend (nginx vhost) and OpenLDAP must be provisioned outside this module.";

    package = lib.mkOption {
      type = lib.types.package;
      description = "Backend (flake `packages.portail-backend`).";
    };

    appUrl = lib.mkOption {
      type = lib.types.str;
      description = "Public URL of the portal — also the OIDC issuer (the backend is its own provider). APP_URL, OIDC_ISSUER_URL, and redirect URI = appUrl + /callback. Used to bake VITE_* into `lib.mkFrontend` for the nginx vhost root.";
      example = "http://portail.nas.local";
    };

    lang = lib.mkOption {
      type = lib.types.str;
      default = "en";
      description = "Default UI language baked into the frontend build as `VITE_APP_LANG` — `fr` or `en`.";
    };

    bindAddress = lib.mkOption {
      type = lib.types.str;
      default = "127.0.0.1:3000";
      description = "Backend listen address (BIND_ADDR). In prod, 127.0.0.1 behind nginx.";
    };

    oidcClientId = lib.mkOption {
      type = lib.types.str;
      default = "portail";
      description = "OIDC client_id used by the backend (the portal is a client of its own provider). Added with a generated secret to `oidc.secretsDir/clients.json` on first start.";
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

      peopleOu = lib.mkOption {
        type = lib.types.str;
        default = "people";
        description = "OU holding user entries (LDAP_PEOPLE_OU).";
      };

      groupsOu = lib.mkOption {
        type = lib.types.str;
        default = "groups";
        description = "OU holding group entries (LDAP_GROUPS_OU).";
      };

      caCertificateFile = lib.mkOption {
        type = with lib.types; nullOr path;
        default = null;
        description = ''
          PEM CA bundle (e.g. `homeCA.crt`) validating the `ldaps://` endpoint.
          Passed to the backend as `SSL_CERT_FILE` — leave null when the
          serving certificate is publicly trusted.
        '';
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
      secretsDir = lib.mkOption {
        type = lib.types.str;
        default = "/var/lib/portail/oidc";
        description = ''
          Directory holding the OIDC secrets (mode 0700, owned by `portail`).
          The backend **generates** the RSA signing key and a default
          `clients.json` (portal client with a random secret) on first start,
          then reads `clients.json` on every request — adding a client is a
          file edit, no restart. Confidential app clients get their secret
          generated here as `<client_id>_secret` on first activation. Pass paths via `OIDC_SECRETS_DIR`.
        '';
      };
    };

    vhost = {
      enable = lib.mkOption {
        type = lib.types.bool;
        default = true;
        description = "Create an nginx virtualhost serving the frontend build and proxying `/api` to the backend.";
      };

      hostName = lib.mkOption {
        type = lib.types.str;
        description = "nginx server_name (and TLS certificate CN).";
        example = "portail.nas.local";
      };

      forceSSL = lib.mkOption {
        type = lib.types.bool;
        default = true;
        description = "Redirect HTTP to HTTPS and serve TLS (does not impact ACME conflicts).";
      };

      sslCertificate = lib.mkOption {
        type = lib.types.path;
        description = "TLS full-chain certificate file (required when `forceSSL = true`).";
      };

      sslCertificateKey = lib.mkOption {
        type = lib.types.path;
        description = "TLS private key file (required when `forceSSL = true`).";
      };
    };

    apps = lib.mkOption {
      type =
        with lib.types;
        listOf (submodule {
          options = {
            name = lib.mkOption {
              type = lib.types.str;
              description = "Display name of the application.";
            };
            url = lib.mkOption {
              type = lib.types.str;
              description = "Absolute application URL (http or https).";
            };
            description = lib.mkOption {
              type = lib.types.str;
              default = "";
              description = "Short description shown under the link.";
            };
            icon = lib.mkOption {
              type = with lib.types; nullOr str;
              default = null;
              description = "Icon URL, optional.";
            };
            oidc = lib.mkOption {
              type = lib.types.submodule {
                options = {
                  enable = lib.mkEnableOption "OIDC client registration for this app";
                  clientId = lib.mkOption {
                    type = lib.types.str;
                    default = "";
                    description = "client_id in `clients.json` — defaults to the lowercased app `name`.";
                  };
                  redirectUris = lib.mkOption {
                    type = with lib.types; listOf str;
                    default = [ ];
                    description = "Registered redirect_uris (exact match required by the provider).";
                  };
                  scopes = lib.mkOption {
                    type = with lib.types; listOf str;
                    default = [
                      "openid"
                      "profile"
                      "email"
                      "groups"
                    ];
                    description = "Scopes granted to this client.";
                  };
                  public = lib.mkOption {
                    type = lib.types.bool;
                    default = false;
                    description = "Public client: empty secret in `clients.json`, PKCE mandatory, no `secretFile`.";
                  };
                  secretFile = lib.mkOption {
                    type = with lib.types; nullOr path;
                    default = null;
                    description = ''
                      File holding the client secret (agenix/sops), read at activation — optional override.
                      When null and `public = false`, the portal generates a random secret at
                      `oidc.secretsDir/<client_id>_secret` (0600, never overwritten) on first activation.
                      A provided file must be readable by the `portail` user.
                    '';
                  };
                };
              };
              default = { };
              description = "Optional `oidc` sub-object (disabled by default): registers the app as an OIDC client of the portal, upserted into `clients.json` at activation (re-read live, no restart). Entries added by the module and removed from `apps` are pruned, along with their generated `<clientId>_secret` file; manually added entries are kept.";
            };
          };
        });
      default = [ ];
      description = "Applications listed on the portal home page — rendered statically into apps.json served at /apps.json by the nginx vhost (no backend involved).";
      example = [
        {
          name = "Nextcloud";
          url = "https://cloud.nas.local";
          description = "Files";
          icon = null;
        }
      ];
    };

    extraBackendEnvironment = lib.mkOption {
      type = lib.types.attrsOf lib.types.str;
      default = { };
      description = "Additional environment variables for the backend (non-secret).";
    };
  };
  config = lib.mkIf cfg.enable {
    assertions = [
      {
        assertion = lib.all (c: c.redirectUris != [ ]) oidcClients;
        message = "each services.portail.apps entry with oidc.enable = true needs non-empty oidc.redirectUris";
      }
    ];

    systemd.services.portail-backend = lib.mkMerge [
      {
        description = "Portail backend (Rust axum)";
        wantedBy = [ "multi-user.target" ];
        after = [ "network.target" ];
        # Secret vars hold the path of a file whose content is the secret — the
        # backend reads and trims the file at startup. The `portail` user must
        # have read access (e.g. agenix/sops-nix `owner`/`group` on secrets).
        environment = {
          APP_URL = cfg.appUrl;
          BIND_ADDR = cfg.bindAddress;
          LDAP_URL = cfg.ldap.url;
          LDAP_BASE_DN = cfg.ldap.baseDn;
          LDAP_PEOPLE_OU = cfg.ldap.peopleOu;
          LDAP_GROUPS_OU = cfg.ldap.groupsOu;
          LDAP_ADMIN_PW = toString cfg.ldap.adminPasswordFile;
          OIDC_CLIENT_ID = cfg.oidcClientId;
          OIDC_ISSUER_URL = cfg.appUrl;
          OIDC_SECRETS_DIR = cfg.oidc.secretsDir;
          OIDC_REDIRECT_URI = "${cfg.appUrl}/callback";
        }
        // cfg.extraBackendEnvironment
        // (lib.optionalAttrs (cfg.ldap.caCertificateFile != null) {
          LDAP_TLS_CA = toString cfg.ldap.caCertificateFile;
        });
        serviceConfig = {
          ExecStart = "${cfg.package}/bin/portail-backend";
          User = "portail";
          Group = "portail";
          StateDirectory = "portail";
          Restart = "on-failure";
          RestartSec = "60s";
          # Always runs: also when `apps` becomes empty, so managed clients
          # from the previous config get pruned from clients.json.
          ExecStartPost = registerOidcClients;
        };
      }
    ];

    users.users.portail = {
      description = "Portail backend service user";
      group = "portail";
      isSystemUser = true;
    };
    users.groups.portail = { };

    services.nginx = lib.mkIf cfg.vhost.enable {
      enable = true;
      recommendedProxySettings = true;
      virtualHosts.${cfg.vhost.hostName} = lib.mkMerge [
        {
          root = (pkgs.callPackage ../nix/frontend.nix { }) {
            appUrl = cfg.appUrl;
            oidcIssuerUrl = cfg.appUrl;
            oidcRedirectUri = "${cfg.appUrl}/callback";
            appLang = cfg.lang;
          };
          locations."/".index = "index.html";
          locations."/".tryFiles = "$uri /index.html";
          locations."= /apps.json".alias = toString appsFile;
          locations."/api".proxyPass = "http://${cfg.bindAddress}";
          locations."= /jwks.json".proxyPass = "http://${cfg.bindAddress}";
          locations."= /.well-known/openid-configuration".proxyPass = "http://${cfg.bindAddress}";
          locations."= /logout".proxyPass = "http://${cfg.bindAddress}";
        }
        (lib.mkIf cfg.vhost.forceSSL {
          forceSSL = true;
          sslCertificate = cfg.vhost.sslCertificate;
          sslCertificateKey = cfg.vhost.sslCertificateKey;
        })
      ];
    };
  };
}
