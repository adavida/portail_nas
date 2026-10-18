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
        svelte.svelte-vscode
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
  devUsers = ''

    dn: uid=admin,ou=people,$LDAP_BASE_DN
    objectClass: inetOrgPerson
    uid: admin
    cn: Admin
    sn: Admin
    mail: admin@example.com
    userPassword: $(slappasswd -s admin)

    dn: uid=user,ou=people,$LDAP_BASE_DN
    objectClass: inetOrgPerson
    uid: user
    cn: User
    sn: User
    mail: user@example.com
    userPassword: $(slappasswd -s user)

    dn: cn=user,ou=groups,$LDAP_BASE_DN
    objectClass: groupOfNames
    cn: user
    description: Utilisateurs du portail
    member: uid=admin,ou=people,$LDAP_BASE_DN
    member: uid=user,ou=people,$LDAP_BASE_DN

    dn: cn=admin,ou=groups,$LDAP_BASE_DN
    objectClass: groupOfNames
    cn: admin
    description: Administrateurs du portail
    member: uid=admin,ou=people,$LDAP_BASE_DN
  '';
  mkSlapd =
    {
      dir,
      ldapsPort,
      port,
      suffix,
      users ? "",
    }:
    ''
      mkdir -p "$DEVENV_STATE/${dir}/data" "$DEVENV_STATE/${dir}/run"
      if [ ! -f "$DEVENV_STATE/${dir}/ldap.crt" ]; then
        ${pkgs.openssl}/bin/openssl req -newkey rsa:4096 -nodes -x509 \
          -keyout "$DEVENV_STATE/${dir}/ldap.key" \
          -out "$DEVENV_STATE/${dir}/ldap.crt" \
          -days 3550 -sha256 \
          -subj "/CN=127.0.0.1" \
          -addext "subjectAltName=DNS:localhost,IP:127.0.0.1"
      fi
      cat > "$DEVENV_STATE/${dir}/slapd.conf" <<EOF
      include ${pkgs.openldap}/etc/schema/core.schema
      include ${pkgs.openldap}/etc/schema/cosine.schema
      include ${pkgs.openldap}/etc/schema/inetorgperson.schema
      modulepath ${pkgs.openldap}/lib/modules
      moduleload back_mdb.la
      pidfile $DEVENV_STATE/${dir}/run/slapd.pid
      argsfile $DEVENV_STATE/${dir}/run/slapd.args
      TLSCACertificateFile $DEVENV_STATE/${dir}/ldap.crt
      TLSCertificateFile $DEVENV_STATE/${dir}/ldap.crt
      TLSCertificateKeyFile $DEVENV_STATE/${dir}/ldap.key
      database mdb
      maxsize 1073741824
      suffix "${suffix}"
      rootdn "cn=admin,${suffix}"
      rootpw admin
      directory $DEVENV_STATE/${dir}/data
      index objectClass eq
      EOF
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
      ${users}
      SEED
        ${pkgs.openldap}/bin/slapadd -f "$DEVENV_STATE/${dir}/slapd.conf" -l /tmp/seed-${dir}.ldif
      fi
      exec ${pkgs.openldap}/libexec/slapd -h "ldap://127.0.0.1:${toString port}/ ldaps://127.0.0.1:${toString ldapsPort}/" -f "$DEVENV_STATE/${dir}/slapd.conf" -d 0
    '';
in
{
  env = {
    APP_URL = "http://localhost:5173";
    BIND_ADDR = "0.0.0.0:3000";
    LDAP_BASE_DN = "dc=dev,dc=example,dc=com";
    LDAP_TEST_BASE_DN = "dc=test,dc=example,dc=com";
    LDAP_TEST_URL = "ldap://127.0.0.1:3891";
    LDAP_TLS_CA = "${config.env.DEVENV_STATE}/openldap-dev/ldap.crt";
    LDAP_URL = "ldaps://127.0.0.1:6360";
    OIDC_CLIENT_ID = "portail-dev";
    OIDC_ISSUER_URL = "http://127.0.0.1:3000";
    OIDC_REDIRECT_URI = "http://localhost:5173/callback";
    OIDC_SECRETS_DIR = "${config.env.DEVENV_STATE}/secrets";
    VITE_APP_LANG = "en";
    VITE_APP_URL = "http://localhost:5173";
    VITE_BACKEND_URL = "http://localhost:3000";
    VITE_OIDC_ISSUER_URL = "http://127.0.0.1:3000";
    VITE_OIDC_REDIRECT_URI = "http://localhost:5173/callback";
  };

  enterShell = ''
    echo "portail LDAP — rust $(rustc --version) | node $(node --version)"
    echo "LDAP dev:  $LDAP_URL/$LDAP_BASE_DN (LDAPS, CA: $LDAP_TLS_CA)"
    echo "LDAP test: $LDAP_TEST_URL/$LDAP_TEST_BASE_DN"
    echo "OpenLDAP users: admin|user, group admin — resetUsers to reseed)"
    echo "OIDC provider: $OIDC_ISSUER_URL (self-hosted, client=$OIDC_CLIENT_ID, secrets: $OIDC_SECRETS_DIR)"
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
    cargo-watch
    # codiumWithExt
    curl
    git
    jq
    just
    openldap
    openssl
    pkg-config
    rtk
    xdg-utils
  ];

  processes = {
    backend.exec = "cargo watch -w backend -x 'run -p portail-backend'";
    "backend-test".exec = "cargo watch -w backend -x 'test --features test-api --quiet'";
    frontend.exec = "npm --prefix frontend run dev";
    "frontend-test".exec = "npm --prefix frontend run test:watch";
    openldap.exec = mkSlapd {
      dir = "openldap-dev";
      port = 3890;
      ldapsPort = 6360;
      suffix = "dc=dev,dc=example,dc=com";
      users = devUsers;
    };
    "openldap-test".exec = mkSlapd {
      dir = "openldap-test";
      port = 3891;
      ldapsPort = 6361;
      suffix = "dc=test,dc=example,dc=com";
    };
    # vscode.exec = "${codiumWithExt}/bin/codium . 2>/dev/null -w --profile devenv_portail";
    vscode.exec = "codium . --profile devenv_portail_d -w 2>/dev/null";
  };

  scripts = {
    openfrontend.exec = "xdg-open $APP_URL/ 2>/dev/null || echo \"Ouvrez manuellement $APP_URL/\"";
    resetUsers.exec = ''
      BIND="-x -H ldap://127.0.0.1:3890 -D cn=admin,$LDAP_BASE_DN -w admin"
      DNS="$(ldapsearch $BIND -s one -b "ou=people,$LDAP_BASE_DN" "(objectClass=inetOrgPerson)" dn | awk '/^dn: /{print $2}')
      $(ldapsearch $BIND -s one -b "ou=groups,$LDAP_BASE_DN" "(objectClass=groupOfNames)" dn | awk '/^dn: /{print $2}')"
      if [ -n "$(printf '%s' "$DNS" | tr -d '[:space:]')" ]; then
        ldapdelete $BIND $DNS
      fi
      ldapadd $BIND <<LDIF
      ${devUsers}
      LDIF
      echo "users reset: admin (group admin) / user"
    '';
  };

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
