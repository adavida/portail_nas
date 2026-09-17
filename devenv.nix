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
    vscodeExtensions = with pkgs.vscode-extensions; [
      rust-lang.rust-analyzer
      tamasfe.even-better-toml
      vadimcn.vscode-lldb
      esbenp.prettier-vscode
      dbaeumer.vscode-eslint
      jnoortheen.nix-ide
      mkhl.direnv
      bradlc.vscode-tailwindcss
    ];
  };
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
in
{
  env = {
    LDAP_BASE_DN = "dc=dev,dc=example,dc=com";
    LDAP_TEST_BASE_DN = "dc=test,dc=example,dc=com";
    LDAP_TEST_URL = "ldap://127.0.0.1:3891";
    LDAP_URL = "ldap://127.0.0.1:3890";
  };

  enterShell = ''
    echo "portail LDAP — rust $(rustc --version) | node $(node --version)"
    echo "LDAP dev:  $LDAP_URL/$LDAP_BASE_DN"
    echo "LDAP test: $LDAP_TEST_URL/$LDAP_TEST_BASE_DN"
  '';

  enterTest = ''
    echo "== backend =="; cargo test -- --nocapture 2>&1 | tail -n 20
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
    codiumWithExt
    git
    just
    openldap
    xdg-utils
  ];

  processes = {
    backend.exec = "cargo watch -w backend -x 'run -p portail-backend'";
    "backend-test".exec = "cargo watch -w backend -x test";
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

  scripts.openfrontend.exec = "xdg-open http://localhost:5173/ 2>/dev/null || echo 'Ouvrez manuellement http://localhost:5173/'";

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
