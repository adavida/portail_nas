# Portail

**Français** | [English](#english)

Stack : Rust Axum (`backend`) + React Vite (`frontend`) + OpenLDAP + Authelia (OIDC).

- **Dev** : piloté par devenv (`devenv.nix`) — backend 3000, frontend 5173, LDAP 3890/3891, Authelia 9091.
- **Prod / déploiement** : piloté par le flake (`flake.nix`) — packages buildables + module NixOS `services.portail` qui installe backend, frontend (nginx), OpenLDAP et Authelia.

Toutes les commandes de dev passent par `devenv shell -- <cmd>` (cargo/node fournis par devenv).

## Déploiement NixOS — flake

Le flake expose :

- `packages.portail-backend` — binaire Rust (buildRustPackage, `Cargo.lock` du workspace).
- `packages.portail-frontend` — build statique Vite (`buildNpmPackage`), adossé au vhost nginx.
- `nixosModules.default` — module `services.portail` qui configure **tout** :
  - `portail-backend.service` (DynamicUser, secrets via EnvironmentFile),
  - nginx (vhost qui sert le front, `tryFiles`→`index.html`, proxifie `/api` vers le backend),
  - `services.openldap` (base `ou=people`/`ou=groups` semée au premier boot, DB persistante),
  - `services.authelia` (backend LDAP, client OIDC `portail`, redirect URI du vhost).

### Utilisation sur une machine NixOS

Récupérer le dépôt puis le référencer comme input flake dans la config hôte :

```bash
git clone https://github.com/adavida/portail_nas.git   # ou même référence directement via github:
cd portail_nas
```

Dans la config hôte :

```nix
{
  inputs.portail.url = "github:adavida/portail_nas";   # ou path:/chemin/vers/le/repo
  inputs.portail.inputs.nixpkgs.follows = "nixpkgs";

  imports = [ inputs.portail.nixosModules.default ];

  services.portail = {
    enable = true;
    package = inputs.portail.packages.x86_64-linux.portail-backend;
    frontendPackage = inputs.portail.packages.x86_64-linux.portail-frontend;
    vhost = "portail.example.com";                     # sert le frontend + /api
    ldap.baseDn = "dc=example,dc=com";
    ldap.adminPasswordFile = "/etc/portail/ldap-admin-pw";       # sans newline finale
    oidc.clientSecretFile = "/etc/portail/oidc-client-secret";   # sans newline finale
  };
}
```

Les 4 secrets Authelia attendus (générer des valeurs aléatoires, non committées) :

```bash
sudo mkdir -p /etc/portail/authelia
openssl rand -hex 32 | tr -d '\n' | sudo tee /etc/portail/authelia/jwt-secret
openssl rand -hex 32 | tr -d '\n' | sudo tee /etc/portail/authelia/storage-encryption-key
openssl rand -hex 32 | tr -d '\n' | sudo tee /etc/portail/authelia/oidc-hmac-secret
openssl genrsa 2048 2>/dev/null | sudo tee /etc/portail/authelia/jwks-key.pem
```

Puis `nixos-rebuild switch`. Le portail est sur `http://portail.example.com`, Authelia sur `http://auth.portail.example.com`.

Pour HTTPS : configurer TLS/ACME sur les vhosts (`services.nginx.virtualHosts."portail.example.com".enableACME = true;` …) et repasser les URLs en `https://` via `appUrl`/`issuerUrl`.

### Options principales (`services.portail`)

| Option                    | Défaut               | Rôle                                               |
| ------------------------- | -------------------- | -------------------------------------------------- |
| `vhost`                   | (requis)             | hôte nginx du portail (front + `/api`)             |
| `authVhost`               | `auth.<vhost>`       | hôte nginx d'Authelia                              |
| `appUrl`                  | `http://<vhost>`     | URL publique du portail (bake dans le front)       |
| `issuerUrl`               | `http://<authVhost>` | URL publique Authelia (bake dans le front)         |
| `bindAddress`             | `127.0.0.1:3000`     | BIND_ADDR du backend                               |
| `oidcClientId`            | `portail`            | client_id OIDC                                     |
| `ldap.baseDn`             | `dc=example,dc=com`  | suffixe LDAP                                       |
| `ldap.adminPasswordFile`  | (requis)             | mot de passe `cn=admin,<baseDn>` (slapd + backend) |
| `oidc.clientSecretFile`   | (requis)             | client_secret partagé backend/Authelia             |
| `extraBackendEnvironment` | `{}`                 | env backend additionnelle (non secrète)            |

Secrets sur prod : fichiers agenix/sops-nix — rien de secret en clair dans la config Nix.

### Vérifier avant de déployer

```bash
nix build .#portail-backend .#portail-frontend     # les deux packages
nix flake check                                     # module + config de test
nix run .#nixosConfigurations.test.config.system.build.vm   # VM de démo (portail.test, forward 80→8080)
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
flake.nix / flake.lock            packages + module NixOS + VM de test
devenv.nix / devenv.yaml          environnement de dev (devenv up)
nixos/portail.nix                 module services.portail
nixos/test-vm.nix                 VM de test (build-vm .#test)
backend/Cargo.toml → backend/src/{main,lib}.rs, domain/, http/, controllers/, repository/
frontend/{vite.config.ts,package.json,src/}
```

---

## English

Stack: Rust Axum (`backend`) + React Vite (`frontend`) + OpenLDAP + Authelia (OIDC).

- **Dev**: driven by devenv (`devenv.nix`) — backend 3000, frontend 5173, LDAP 3890/3891, Authelia 9091.
- **Prod / deployment**: driven by the flake (`flake.nix`) — buildable packages + the NixOS module `services.portail` which installs backend, frontend (nginx), OpenLDAP and Authelia.

All dev commands go through `devenv shell -- <cmd>` (cargo/node provided by devenv).

### NixOS deployment — flake

The flake exposes:

- `packages.portail-backend` — Rust binary (buildRustPackage, workspace `Cargo.lock`).
- `packages.portail-frontend` — static Vite build (`buildNpmPackage`), served by the nginx vhost.
- `nixosModules.default` — the `services.portail` module, which configures **everything**:
  - `portail-backend.service` (DynamicUser, secrets via EnvironmentFile),
  - nginx (vhost serving the frontend, `tryFiles`→`index.html`, proxying `/api` to the backend),
  - `services.openldap` (`ou=people`/`ou=groups` seeded on first boot, persistent DB),
  - `services.authelia` (LDAP backend, OIDC client `portail`, vhost redirect URI).

#### Using it on a NixOS machine

Fetch the repository and reference it as a flake input in the host config:

```bash
git clone https://github.com/adavida/portail_nas.git   # or reference it directly via github:
cd portail_nas
```

In the host config:

```nix
{
  inputs.portail.url = "github:adavida/portail_nas";   # or path:/path/to/repo
  inputs.portail.inputs.nixpkgs.follows = "nixpkgs";

  imports = [ inputs.portail.nixosModules.default ];

  services.portail = {
    enable = true;
    package = inputs.portail.packages.x86_64-linux.portail-backend;
    frontendPackage = inputs.portail.packages.x86_64-linux.portail-frontend;
    vhost = "portail.example.com";                     # serves frontend + /api
    ldap.baseDn = "dc=example,dc=com";
    ldap.adminPasswordFile = "/etc/portail/ldap-admin-pw";       # no trailing newline
    oidc.clientSecretFile = "/etc/portail/oidc-client-secret";   # no trailing newline
  };
}
```

The 4 expected Authelia secrets (generate random values, do not commit):

```bash
sudo mkdir -p /etc/portail/authelia
openssl rand -hex 32 | tr -d '\n' | sudo tee /etc/portail/authelia/jwt-secret
openssl rand -hex 32 | tr -d '\n' | sudo tee /etc/portail/authelia/storage-encryption-key
openssl rand -hex 32 | tr -d '\n' | sudo tee /etc/portail/authelia/oidc-hmac-secret
openssl genrsa 2048 2>/dev/null | sudo tee /etc/portail/authelia/jwks-key.pem
```

Then `nixos-rebuild switch`. The portal is at `http://portail.example.com`, Authelia at `http://auth.portail.example.com`.

For HTTPS: configure TLS/ACME on the vhosts (`services.nginx.virtualHosts."portail.example.com".enableACME = true;` …) and switch the URLs to `https://` via `appUrl`/`issuerUrl`.

#### Main options (`services.portail`)

| Option                    | Default              | Purpose                                           |
| ------------------------- | -------------------- | ------------------------------------------------- |
| `vhost`                   | (required)           | nginx host serving the portal (frontend + `/api`) |
| `authVhost`               | `auth.<vhost>`       | nginx host serving Authelia                       |
| `appUrl`                  | `http://<vhost>`     | public portal URL (baked into the frontend)       |
| `issuerUrl`               | `http://<authVhost>` | public Authelia URL (baked into the frontend)     |
| `bindAddress`             | `127.0.0.1:3000`     | backend BIND_ADDR                                 |
| `oidcClientId`            | `portail`            | OIDC client_id                                    |
| `ldap.baseDn`             | `dc=example,dc=com`  | LDAP suffix                                       |
| `ldap.adminPasswordFile`  | (required)           | password of `cn=admin,<baseDn>` (slapd + backend) |
| `oidc.clientSecretFile`   | (required)           | client_secret shared by backend/Authelia          |
| `extraBackendEnvironment` | `{}`                 | additional backend env (non-secret)               |

Secrets in prod: agenix/sops-nix files — no secrets in plain text inside the Nix config.

#### Check before deploying

```bash
nix build .#portail-backend .#portail-frontend     # both packages
nix flake check                                     # module + test config
nix run .#nixosConfigurations.test.config.system.build.vm   # demo VM (portail.test, forward 80→8080)
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
flake.nix / flake.lock            packages + NixOS module + test VM
devenv.nix / devenv.yaml          dev environment (devenv up)
nixos/portail.nix                 services.portail module
nixos/test-vm.nix                 test VM (build-vm .#test)
backend/Cargo.toml → backend/src/{main,lib}.rs, domain/, http/, controllers/, repository/
frontend/{vite.config.ts,package.json,src/}
```
