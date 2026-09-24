# Portail

**Français** | [English](#english)

Stack : Rust Axum (`backend`) + React Vite (`frontend`) + OpenLDAP + Authelia (OIDC).

- **Dev** : piloté par devenv (`devenv.nix`) — backend 3000, frontend 5173, LDAP 3890/3891, Authelia 9091.
- **Prod / déploiement** : piloté par le flake (`flake.nix`) — packages buildables + module NixOS `services.portail` qui exécute le backend (service systemd + secrets) et le vhost nginx du frontend. OpenLDAP et Authelia sont provisionnés par la config hôte — examples ci-dessous.

Toutes les commandes de dev passent par `devenv shell -- <cmd>` (cargo/node fournis par devenv).

## Déploiement NixOS — flake

Le flake expose :

- `packages.portail-backend` — binaire Rust (buildRustPackage, `Cargo.lock` du workspace).
- `packages.portail-frontend` — build statique Vite (`buildNpmPackage`), URLs OIDC cuites à la volée.
- `lib.mkFrontend { appUrl; oidcIssuerUrl; oidcRedirectUri; }` — frontend reconstruit avec les URLs de prod (à passer au `root` du vhost).
- `nixosModules.default` — module `services.portail` :
  - `portail-backend.service` (DynamicUser, `OIDC_CLIENT_SECRET`/`LDAP_ADMIN_PW` = chemins des fichiers de secrets, lus au démarrage par le backend).
  - vhost nginx optionnel (`vhost.enable`, défaut `true`) : frontend en `root`, `/api` proxy vers le backend, `forceSSL` (défaut `true`) avec certificat fourni (`vhost.sslCertificate{,Key}`).

OpenLDAP, Authelia (et leur vhost) se configurent dans la config hôte.

### Utilisation sur une machine NixOS

Récupérer le dépôt puis le référencer comme input flake dans la config hôte :

```bash
git clone https://github.com/adavida/portail_nas.git   # ou même référence directement via github:
cd portail_nas
```

Dans la config hôte — module backend + examples de provisionnement :

```nix
{
  inputs.portail.url = "github:adavida/portail_nas";   # ou path:/chemin/vers/le/repo
  inputs.portail.inputs.nixpkgs.follows = "nixpkgs";

  imports = [ inputs.portail.nixosModules.default ];

  # --- module backend + vhost portail ---
  services.portail = {
    enable = true;
    package = inputs.portail.packages.x86_64-linux.portail-backend;
    appUrl = "http://portail.example.com";                       # baked dans le front, APP_URL backend
    issuerUrl = "http://auth.portail.example.com";               # OIDC_ISSUER_URL
    vhost.hostName = "portail.example.com";
    vhost.forceSSL = true;                                       # défaut true — HTTPS + redirection 80
    vhost.sslCertificate = "/etc/portail/ssl/fullchain.pem";        # requis si forceSSL
    vhost.sslCertificateKey = "/etc/portail/ssl/key.pem";
    ldap.baseDn = "dc=example,dc=com";
    ldap.adminPasswordFile = "/etc/portail/ldap-admin-pw";       # sans newline finale
    oidc.clientSecretFile = "/etc/portail/oidc-client-secret";   # sans newline finale
  };

  # --- vhost Authelia (provisionné hors module) ---
  services.nginx.virtualHosts."auth.portail.example.com" = {
    locations."/".proxyPass = "http://127.0.0.1:9091";
    locations."/".proxyWebsockets = true;
  };
}
```

Sur LDAP : serveur existant (`ldap.url` — `ldap://` ou `ldaps://`, certificat signé attendu) ou `services.openldap` sur la même machine (config OLC + seed `ou=people`/`ou=groups`).

Sur Authelia : `services.authelia.instances.portail` avec le backend LDAP pointé vers la même base :

```nix
services.authelia.instances.portail = {
  enable = true;
  secrets = {
    jwtSecretFile = "/etc/portail/authelia/jwt-secret";
    storageEncryptionKeyFile = "/etc/portail/authelia/storage-encryption-key";
    oidcHmacSecretFile = "/etc/portail/authelia/oidc-hmac-secret";
    oidcIssuerPrivateKeyFile = "/etc/portail/authelia/jwks-key.pem";
  };
  environmentVariables = {
    X_AUTHELIA_CONFIG_FILTERS = "template";
    AUTHELIA_AUTHENTICATION_BACKEND_LDAP_PASSWORD_FILE = "/etc/portail/ldap-admin-pw";
  };
  settings = {
    server.address = "tcp://127.0.0.1:9091";
    authentication_backend.ldap = {
      address = "ldap://127.0.0.1";
      base_dn = "dc=example,dc=com";
      user = "cn=admin,dc=example,dc=com";
      additional_users_dn = "ou=people";
      users_filter = "(&({username_attribute}={input}))";
      additional_groups_dn = "ou=groups";
      groups_filter = "(member={dn})";
    };
    access_control.default_policy = "one_factor";
    session.cookies = [ {
      domain = "auth.portail.example.com";
      authelia_url = "http://auth.portail.example.com";
    } ];
    storage.local.path = "/var/lib/authelia-portail/db.sqlite3";
    notifier.filesystem.filename = "/var/lib/authelia-portail/notifications.txt";
    identity_providers.oidc.clients = [ {
      client_id = "portail";
      # lu par le template filter via X_AUTHELIA_CONFIG_FILTERS
      client_secret = "{{ secret \"/etc/portail/oidc-client-secret\" }}";
      authorization_policy = "one_factor";
      consent_mode = "implicit";
      redirect_uris = [ "http://portail.example.com/callback" ];
      scopes = [ "openid" "groups" "email" "profile" "offline_access" ];
      grant_types = [ "authorization_code" "refresh_token" ];
      response_types = [ "code" ];
      token_endpoint_auth_method = "client_secret_basic";
    } ];
  };
};
```

Les secrets attendus (générer des valeurs aléatoires, non committées) :

```bash
sudo mkdir -p /etc/portail/authelia
openssl rand -hex 32 | tr -d '\n' | sudo tee /etc/portail/authelia/jwt-secret
openssl rand -hex 32 | tr -d '\n' | sudo tee /etc/portail/authelia/storage-encryption-key
openssl rand -hex 32 | tr -d '\n' | sudo tee /etc/portail/authelia/oidc-hmac-secret
openssl genrsa 2048 2>/dev/null | sudo tee /etc/portail/authelia/jwks-key.pem
```

Puis `nixos-rebuild switch`. Le portail est sur `http://portail.example.com`, Authelia sur `http://auth.portail.example.com` (les vhost nginx `forceSSL = false` servent en HTTP ; passez les URLs en `https://` via `appUrl`/`issuerUrl`).

### Options principales (`services.portail`)

| Option                       | Défaut              | Rôle                                                      |
| ---------------------------- | ------------------- | --------------------------------------------------------- |
| `appUrl`                     | (requis)            | URL publique du portail (APP_URL backend + redirect OIDC) |
| `issuerUrl`                  | (requis)            | URL publique Authelia (OIDC_ISSUER_URL)                   |
| `bindAddress`                | `127.0.0.1:3000`    | BIND_ADDR du backend                                      |
| `oidcClientId`               | `portail`           | client_id OIDC                                            |
| `ldap.url`                   | `ldap://127.0.0.1`  | URL du serveur LDAP existant (LDAP_URL)                   |
| `ldap.baseDn`                | `dc=example,dc=com` | suffixe LDAP                                              |
| `ldap.adminPasswordFile`     | (requis)            | mot de passe `cn=admin,<baseDn>` (backend + Authelia)     |
| `oidc.clientSecretFile`      | (requis)            | client_secret partagé backend/Authelia                    |
| `vhost.enable`               | `true`              | vhost nginx backend (frontend + proxy `/api`)             |
| `vhost.hostName`             | (si vhost)          | server_name nginx + CN du certificat                      |
| `vhost.forceSSL`             | `true`              | HTTPS + redirection 80→443                                |
| `vhost.sslCertificate{,Key}` | (si forceSSL)       | certificat/clé TLS fournis par l'hôte (agenix/sops)       |
| `extraBackendEnvironment`    | `{}`                | env backend additionnelle (non secrète)                   |

Secrets sur prod : fichiers agenix/sops-nix — rien de secret en clair dans la config Nix.

### Vérifier avant de déployer

```bash
nix build .#portail-backend .#portail-frontend     # les deux packages
nix flake check                                     # module
```

## Développement

```bash
devenv up            # lance tout : backend 3000, frontend 5173, LDAP 3890/3891, Authelia 9091
devenv down          # arrêt
```

- backend `http://localhost:3000/api/health`
- frontend `http://localhost:5173` (proxy `/api` → 3000)
- LDAP dev `ldap://127.0.0.1:3890/dc=dev,dc=example,dc=com`
- LDAP test `ldap://127.0.0.1:3891/dc=test,dc=example,dc=com`
- Authelia `https://127.0.0.1:9091` (comptes de démo `admin/admin`, `user/user`, groupe `admin`)

### Tests & format

```bash
devenv shell -- cargo test --features test-api      # backend (unit + intégration LDAP 3891)
devenv shell -- npm --prefix frontend test -- --run  # frontend (vitest)
just test                                            # les deux
devenv shell -- treefmt                              # nixfmt + rustfmt + prettier
```

## Layout

```
flake.nix / flake.lock            packages + module NixOS
devenv.nix / devenv.yaml          environnement de dev (devenv up)
nixos/portail.nix                 module services.portail
backend/Cargo.toml → backend/src/{main,lib}.rs, domain/, http/, controllers/, repository/
frontend/{vite.config.ts,package.json,src/}
```

---

## English

Stack: Rust Axum (`backend`) + React Vite (`frontend`) + OpenLDAP + Authelia (OIDC).

- **Dev**: driven by devenv (`devenv.nix`) — backend 3000, frontend 5173, LDAP 3890/3891, Authelia 9091.
- **Prod / deployment**: driven by the flake (`flake.nix`) — buildable packages + the NixOS module `services.portail` which runs the backend (systemd service + secrets) and the frontend nginx vhost. OpenLDAP and Authelia are provisioned by the host config — examples below.

All dev commands go through `devenv shell -- <cmd>` (cargo/node provided by devenv).

### NixOS deployment — flake

The flake exposes:

- `packages.portail-backend` — Rust binary (buildRustPackage, workspace `Cargo.lock`).
- `packages.portail-frontend` — static Vite build (`buildNpmPackage`), OIDC URLs baked at build time.
- `lib.mkFrontend { appUrl; oidcIssuerUrl; oidcRedirectUri; }` — frontend rebuilt with the prod URLs (pass it to the vhost `root`).
- `nixosModules.default` — the `services.portail` module:
  - `portail-backend.service` (DynamicUser, `OIDC_CLIENT_SECRET`/`LDAP_ADMIN_PW` set to secret file paths, read at startup by the backend).
  - optional nginx vhost (`vhost.enable`, default `true`): frontend as `root`, `/api` proxies to the backend, `forceSSL` (default `true`) with a host-provided certificate (`vhost.sslCertificate{,Key}`).

OpenLDAP and Authelia (and their vhost) are configured in the host config.

#### Using it on a NixOS machine

Fetch the repository and reference it as a flake input in the host config:

```bash
git clone https://github.com/adavida/portail_nas.git   # or reference it directly via github:
cd portail_nas
```

In the host config — backend module + provisioning examples:

```nix
{
  inputs.portail.url = "github:adavida/portail_nas";   # or path:/path/to/repo
  inputs.portail.inputs.nixpkgs.follows = "nixpkgs";

  imports = [ inputs.portail.nixosModules.default ];

  # --- backend module + portail vhost ---
  services.portail = {
    enable = true;
    package = inputs.portail.packages.x86_64-linux.portail-backend;
    appUrl = "http://portail.example.com";                       # baked into the frontend, backend APP_URL
    issuerUrl = "http://auth.portail.example.com";               # OIDC_ISSUER_URL
    vhost.hostName = "portail.example.com";
    vhost.forceSSL = true;                                       # default true — HTTPS + port 80 redirect
    vhost.sslCertificate = "/etc/portail/ssl/fullchain.pem";     # required with forceSSL
    vhost.sslCertificateKey = "/etc/portail/ssl/key.pem";
    ldap.baseDn = "dc=example,dc=com";
    ldap.adminPasswordFile = "/etc/portail/ldap-admin-pw";       # no trailing newline
    oidc.clientSecretFile = "/etc/portail/oidc-client-secret";   # no trailing newline
  };

  # --- Authelia vhost (provisioned outside the module) ---
  services.nginx.virtualHosts."auth.portail.example.com" = {
    locations."/".proxyPass = "http://127.0.0.1:9091";
    locations."/".proxyWebsockets = true;
  };
}
```

LDAP: an existing server (`ldap.url` — `ldap://` or `ldaps://`, signed certificate expected) or `services.openldap` on the same machine (OLC config + `ou=people`/`ou=groups` seed).

Authelia: `services.authelia.instances.portail` with the LDAP backend pointed at the same database:

```nix
services.authelia.instances.portail = {
  enable = true;
  secrets = {
    jwtSecretFile = "/etc/portail/authelia/jwt-secret";
    storageEncryptionKeyFile = "/etc/portail/authelia/storage-encryption-key";
    oidcHmacSecretFile = "/etc/portail/authelia/oidc-hmac-secret";
    oidcIssuerPrivateKeyFile = "/etc/portail/authelia/jwks-key.pem";
  };
  environmentVariables = {
    X_AUTHELIA_CONFIG_FILTERS = "template";
    AUTHELIA_AUTHENTICATION_BACKEND_LDAP_PASSWORD_FILE = "/etc/portail/ldap-admin-pw";
  };
  settings = {
    server.address = "tcp://127.0.0.1:9091";
    authentication_backend.ldap = {
      address = "ldap://127.0.0.1";
      base_dn = "dc=example,dc=com";
      user = "cn=admin,dc=example,dc=com";
      additional_users_dn = "ou=people";
      users_filter = "(&({username_attribute}={input}))";
      additional_groups_dn = "ou=groups";
      groups_filter = "(member={dn})";
    };
    access_control.default_policy = "one_factor";
    session.cookies = [ {
      domain = "auth.portail.example.com";
      authelia_url = "http://auth.portail.example.com";
    } ];
    storage.local.path = "/var/lib/authelia-portail/db.sqlite3";
    notifier.filesystem.filename = "/var/lib/authelia-portail/notifications.txt";
    identity_providers.oidc.clients = [ {
      client_id = "portail";
      # read by the template filter via X_AUTHELIA_CONFIG_FILTERS
      client_secret = "{{ secret \"/etc/portail/oidc-client-secret\" }}";
      authorization_policy = "one_factor";
      consent_mode = "implicit";
      redirect_uris = [ "http://portail.example.com/callback" ];
      scopes = [ "openid" "groups" "email" "profile" "offline_access" ];
      grant_types = [ "authorization_code" "refresh_token" ];
      response_types = [ "code" ];
      token_endpoint_auth_method = "client_secret_basic";
    } ];
  };
};
```

Expected secrets (generate random values, do not commit):

```bash
sudo mkdir -p /etc/portail/authelia
openssl rand -hex 32 | tr -d '\n' | sudo tee /etc/portail/authelia/jwt-secret
openssl rand -hex 32 | tr -d '\n' | sudo tee /etc/portail/authelia/storage-encryption-key
openssl rand -hex 32 | tr -d '\n' | sudo tee /etc/portail/authelia/oidc-hmac-secret
openssl genrsa 2048 2>/dev/null | sudo tee /etc/portail/authelia/jwks-key.pem
```

Then `nixos-rebuild switch`. The portal is at `http://portail.example.com`, Authelia at `http://auth.portail.example.com` (with `forceSSL = false` the nginx vhost serves plain HTTP; switch the URLs to `https://` via `appUrl`/`issuerUrl`).

#### Main options (`services.portail`)

| Option                       | Default             | Purpose                                                |
| ---------------------------- | ------------------- | ------------------------------------------------------ |
| `appUrl`                     | (required)          | public portal URL (backend APP_URL + OIDC redirect)    |
| `issuerUrl`                  | (required)          | public Authelia URL (OIDC_ISSUER_URL)                  |
| `bindAddress`                | `127.0.0.1:3000`    | backend BIND_ADDR                                      |
| `oidcClientId`               | `portail`           | OIDC client_id                                         |
| `ldap.url`                   | `ldap://127.0.0.1`  | existing LDAP server URL (LDAP_URL)                    |
| `ldap.baseDn`                | `dc=example,dc=com` | LDAP suffix                                            |
| `ldap.adminPasswordFile`     | (required)          | password of `cn=admin,<baseDn>` (backend + Authelia)   |
| `oidc.clientSecretFile`      | (required)          | client_secret shared by backend/Authelia               |
| `vhost.enable`               | `true`              | backend module nginx vhost (frontend + `/api` proxy)   |
| `vhost.hostName`             | (with vhost)        | nginx server_name + certificate CN                     |
| `vhost.forceSSL`             | `true`              | HTTPS + 80→443 redirect                                |
| `vhost.sslCertificate{,Key}` | (with forceSSL)     | TLS certificate/key provided by the host (agenix/sops) |
| `extraBackendEnvironment`    | `{}`                | additional backend env (non-secret)                    |

Secrets in prod: agenix/sops-nix files — no secrets in plain text inside the Nix config.

#### Check before deploying

```bash
nix build .#portail-backend .#portail-frontend     # both packages
nix flake check                                     # module
```

#### Development

```bash
devenv up            # starts everything: backend 3000, frontend 5173, LDAP 3890/3891, Authelia 9091
devenv down          # stop
```

- backend `http://localhost:3000/api/health`
- frontend `http://localhost:5173` (proxy `/api` → 3000)
- dev LDAP `ldap://127.0.0.1:3890/dc=dev,dc=example,dc=com`
- test LDAP `ldap://127.0.0.1:3891/dc=test,dc=example,dc=com`
- Authelia `https://127.0.0.1:9091` (demo accounts `admin/admin`, `user/user`, group `admin`)

#### Tests & format

```bash
devenv shell -- cargo test --features test-api      # backend (unit + LDAP integration on 3891)
devenv shell -- npm --prefix frontend test -- --run  # frontend (vitest)
just test                                            # both
devenv shell -- treefmt                              # nixfmt + rustfmt + prettier
```

#### Layout

```
flake.nix / flake.lock            packages + NixOS module
devenv.nix / devenv.yaml          dev environment (devenv up)
nixos/portail.nix                 services.portail module
backend/Cargo.toml → backend/src/{main,lib}.rs, domain/, http/, controllers/, repository/
frontend/{vite.config.ts,package.json,src/}
```
