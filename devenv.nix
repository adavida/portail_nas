{
  pkgs,
  lib,
  config,
  inputs,
  ...
}:

let
  codiumWithExt = pkgs.vscode-with-extensions.override {
    vscode = pkgs.vscodium;
    vscodeExtensions =
      with pkgs.vscode-extensions;
      [
        bradlc.vscode-tailwindcss
        dbaeumer.vscode-eslint
        esbenp.prettier-vscode
        fill-labs.dependi
        jnoortheen.nix-ide
        mhutchie.git-graph
        mkhl.direnv
        rust-lang.rust-analyzer
        tamasfe.even-better-toml
        vadimcn.vscode-lldb
      ]
      ++ pkgs.vscode-utils.extensionsFromVscodeMarketplace [
        {
          name = "amvim";
          publisher = "auiworks";
          version = "1.37.0";
          sha256 = "0jh43cajpkpgnnkn4ficsphsrbjnykiz5fml88w2s202101pf1l1";
        }
      ];
  };
  mkAuthelia = ''
    mkdir -p "$DEVENV_STATE/authelia-dev" "$DEVENV_STATE/authelia"
    if [ ! -f "$DEVENV_STATE/authelia-dev/authelia.key" ]; then
      ${pkgs.openssl}/bin/openssl req -newkey rsa:4096 -nodes -x509 \
        -keyout "$DEVENV_STATE/authelia-dev/authelia.key" \
        -out "$DEVENV_STATE/authelia-dev/authelia.crt" \
        -days 3550 -sha256 \
        -subj "/CN=127.0.0.1" \
        -addext "subjectAltName=DNS:localhost,IP:127.0.0.1"
    fi
    if [ ! -f "$DEVENV_STATE/authelia-dev/hmac_secret" ]; then
      ${pkgs.openssl}/bin/openssl rand -hex 32 > "$DEVENV_STATE/authelia-dev/hmac_secret"
    fi
    if [ ! -f "$DEVENV_STATE/authelia-dev/jwks.pem" ]; then
      ${pkgs.openssl}/bin/openssl genrsa 2048 > "$DEVENV_STATE/authelia-dev/jwks.pem"
    fi
    if [ ! -f "$DEVENV_STATE/authelia-dev/users.yml" ]; then
      cat > "$DEVENV_STATE/authelia-dev/users.yml" <<'USERS'
    users:
      admin:
        displayname: Admin
        email: admin@example.com
        groups:
          - admin
        password: "$argon2id$v=19$m=65536,t=3,p=4$I/IZNPA3vKfOQLuA2W/qpQ$nuM7fyyQH7YP8ZKzEbEr5/R5A6a6Ofl3m/mWtp14E6Y"
      user:
        displayname: User
        email: user@example.com
        groups: []
        password: "$argon2id$v=19$m=65536,t=3,p=4$1tkN6rrJ0pDXaH4Hhod83g$Dfu+PQQHaun9+IGT4n/mTp4jeqQVZGgyyD/RL4l5KoM"
    USERS
    fi
    HMAC=$(cat "$DEVENV_STATE/authelia-dev/hmac_secret")
    JWKS=$(sed 's/^/              /' "$DEVENV_STATE/authelia-dev/jwks.pem")
    cat > "$DEVENV_STATE/authelia-dev/config.yml" <<EOF
    server:
      address: tcp://127.0.0.1:9091
      tls:
        certificate: $DEVENV_STATE/authelia-dev/authelia.crt
        key: $DEVENV_STATE/authelia-dev/authelia.key
    authentication_backend:
      file:
        path: $DEVENV_STATE/authelia-dev/users.yml
    access_control:
      default_policy: one_factor
    identity_validation:
      reset_password:
        jwt_secret: portail-dev-reset-password-jwt-secret
    identity_providers:
      oidc:
        hmac_secret: $HMAC
        jwks:
          - key: |
    $JWKS
        clients:
          - client_id: portail-dev
            client_name: Portail Dev
            client_secret: "\$pbkdf2-sha512\$310000\$YudH3UkHfJ.RW5a8z2zTqw\$.cKmbS5jKBVNHGZo3g1B9AHBPfzKufixtQ4MFP57FN7n07FU5srD35VtG6u0lJEjg9XAoXiyJuyclai33XDjOw"
            pkce_challenge_method: S256
            public: false
            authorization_policy: one_factor
            consent_mode: implicit #need ?
            redirect_uris:
              - $OIDC_REDIRECT_URI
              - http://127.0.0.1:5173/callback
              - http://localhost:3000/callback
            scopes:
              - openid
              - groups
              - email
              - profile
              - offline_access
            grant_types:
              - authorization_code
              - refresh_token
            response_types:
              - code
            token_endpoint_auth_method: client_secret_basic
    session:
      name: portail_session
      secret: portail-dev-session-secret
      cookies:
        - domain: 127.0.0.1
          authelia_url: $OIDC_ISSUER_URL
    storage:
      encryption_key: portail-dev-encryption-key
      local:
        path: $DEVENV_STATE/authelia/db.sqlite3
    notifier:
      filesystem:
        filename: $DEVENV_STATE/authelia/notifications.txt
    EOF
    exec ${pkgs.authelia}/bin/authelia --config "$DEVENV_STATE/authelia-dev/config.yml"
  '';
  mkSlapd =
    {
      port,
      suffix,
      dir,
    }:
    ''
      mkdir -p "$DEVENV_STATE/${dir}/data" "$DEVENV_STATE/${dir}/run"
      if [ ! -f "$DEVENV_STATE/${dir}/slapd.conf" ]; then
        cat > "$DEVENV_STATE/${dir}/slapd.conf" <<EOF
      include ${pkgs.openldap}/etc/schema/core.schema
      include ${pkgs.openldap}/etc/schema/cosine.schema
      include ${pkgs.openldap}/etc/schema/inetorgperson.schema
      modulepath ${pkgs.openldap}/lib/modules
      moduleload back_mdb.la
      pidfile $DEVENV_STATE/${dir}/run/slapd.pid
      argsfile $DEVENV_STATE/${dir}/run/slapd.args
      database mdb
      maxsize 1073741824
      suffix "${suffix}"
      rootdn "cn=admin,${suffix}"
      rootpw admin
      directory $DEVENV_STATE/${dir}/data
      index objectClass eq
      EOF
      fi
      if [ ! -f "$DEVENV_STATE/${dir}/data/data.mdb" ]; then
        dc=$(echo "${suffix}" | cut -d',' -f1 | cut -d'=' -f2)
        cat > /tmp/seed-${dir}.ldif <<SEED
      dn: ${suffix}
      objectClass: dcObject
      objectClass: organization
      dc: $dc
      o: $dc example

      dn: ou=people,${suffix}
      objectClass: organizationalUnit
      ou: people

      dn: ou=groups,${suffix}
      objectClass: organizationalUnit
      ou: groups
      SEED
        ${pkgs.openldap}/bin/slapadd -f "$DEVENV_STATE/${dir}/slapd.conf" -l /tmp/seed-${dir}.ldif
      fi
      exec ${pkgs.openldap}/libexec/slapd -h "ldap://127.0.0.1:${toString port}/" -f "$DEVENV_STATE/${dir}/slapd.conf" -d 0
    '';
  # Secret vars hold the path of a file whose content is the secret value.
  # OIDC client secret must match Authelia dev client_secret (pbkdf2 hash of
  # `portail-dev-secret`). LDAP_ADMIN_PW stays unset in dev (default `admin`).
  mkDevSecrets = ''
    mkdir -p "$DEVENV_STATE/secrets"
    printf 'portail-dev-secret' > "$DEVENV_STATE/secrets/oidc-client-secret"
  '';
in
{
  env = {
    APP_URL = "http://localhost:5173";
    BIND_ADDR = "0.0.0.0:3000";
    AUTHELIA_CONFIG = "${config.env.DEVENV_STATE}/authelia-dev/config.yml";
    LDAP_BASE_DN = "dc=dev,dc=example,dc=com";
    LDAP_TEST_BASE_DN = "dc=test,dc=example,dc=com";
    LDAP_TEST_URL = "ldap://127.0.0.1:3891";
    LDAP_URL = "ldap://127.0.0.1:3890";
    OIDC_CLIENT_ID = "portail-dev";
    OIDC_CLIENT_SECRET = "${config.env.DEVENV_STATE}/secrets/oidc-client-secret";
    OIDC_ISSUER_URL = "https://127.0.0.1:9091";
    OIDC_REDIRECT_URI = "http://localhost:5173/callback";
    VITE_APP_URL = "http://localhost:5173";
    VITE_BACKEND_URL = "http://localhost:3000";
    VITE_OIDC_ISSUER_URL = "https://127.0.0.1:9091";
    VITE_OIDC_REDIRECT_URI = "http://localhost:5173/callback";
  };

  enterShell = mkDevSecrets + ''
    echo "portail LDAP — rust $(rustc --version) | node $(node --version)"
    echo "LDAP dev:  $LDAP_URL/$LDAP_BASE_DN"
    echo "LDAP test: $LDAP_TEST_URL/$LDAP_TEST_BASE_DN"
    echo "Authelia:  https://127.0.0.1:9091 (file: admin/admin, user/user — group admin)"
    echo "OIDC:      $OIDC_ISSUER_URL/.well-known/openid-configuration client=$OIDC_CLIENT_ID"
  '';

  enterTest = ''
    echo "== backend =="; cargo test --features test-api -- --nocapture 2>&1 | tail -n 20
    echo "== frontend =="; npm --prefix frontend test -- --run 2>&1 | tail -n 20
  '';

  languages = {
    javascript = {
      enable = true;
      package = pkgs.nodejs_22;
    };
    rust.enable = true;
  };

  packages = with pkgs; [
    authelia
    cargo-watch
    codiumWithExt
    git
    just
    openldap
    rtk
    xdg-utils
  ];

  processes = {
    authelia.exec = mkAuthelia;
    backend.exec = mkDevSecrets + "\ncargo watch -w backend -x 'run -p portail-backend'";
    "backend-test".exec =
      mkDevSecrets + "\ncargo watch -w backend -x 'test --features test-api --quiet'";
    frontend.exec = "npm --prefix frontend run dev";
    "frontend-test".exec = "npm --prefix frontend run test:watch";
    openldap.exec = mkSlapd {
      dir = "openldap-dev";
      port = 3890;
      suffix = "dc=dev,dc=example,dc=com";
    };
    "openldap-test".exec = mkSlapd {
      dir = "openldap-test";
      port = 3891;
      suffix = "dc=test,dc=example,dc=com";
    };
    vscode.exec = "${codiumWithExt}/bin/codium . 2>/dev/null || code . 2>/dev/null || echo 'vscode/codium non installé — ouvrez manuellement code .'; sleep infinity";
  };

  scripts.openfrontend.exec = "xdg-open $APP_URL/ 2>/dev/null || echo \"Ouvrez manuellement $APP_URL/\"";

  treefmt = {
    enable = true;
    config = {
      projectRootFile = "devenv.nix";
      programs = {
        nixfmt.enable = true;
        rustfmt.enable = true;
        prettier.enable = true;
      };
    };
  };
}
