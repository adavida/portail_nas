# Portail

**Français** | [English](#english)

Portail perso pour gérer les utilisateurs de mon NAS NixOS et afficher une page regroupant ses applications.

## Déploiement NixOS (flake)

Le flake expose `packages.portail-backend`, `packages.portail-frontend` et `nixosModules.default` (module `services.portail` : service systemd du backend + vhost nginx du frontend, proxy `/api`, `forceSSL` par défaut). OpenLDAP et Authelia sont provisionnés par la config hôte.

```nix
{ inputs, ... }:
{
  inputs.portail.url = "github:adavida/portail_nas";
  inputs.portail.inputs.nixpkgs.follows = "nixpkgs";

  imports = [ inputs.portail.nixosModules.default ];

  services.portail = {
    enable = true;
    package = inputs.portail.packages.x86_64-linux.portail-backend;
    appUrl = "https://portail.example.com";
    issuerUrl = "https://auth.portail.example.com";
    vhost.hostName = "portail.example.com";
    vhost.sslCertificate = "/etc/portail/ssl/fullchain.pem";
    vhost.sslCertificateKey = "/etc/portail/ssl/key.pem";
    ldap.baseDn = "dc=example,dc=com";
    ldap.adminPasswordFile = "/etc/portail/ldap-admin-pw";
    oidc.clientSecretFile = "/etc/portail/oidc-client-secret";
  };
}
```

| Option                       | Défaut              | Rôle                                                      |
| ---------------------------- | ------------------- | --------------------------------------------------------- |
| `appUrl`                     | (requis)            | URL publique du portail (APP_URL backend + redirect OIDC) |
| `issuerUrl`                  | (requis)            | URL publique Authelia (OIDC_ISSUER_URL)                   |
| `package`                    | (requis)            | binaire backend (ex: `inputs.portail.packages...`)        |
| `bindAddress`                | `127.0.0.1:3000`    | BIND_ADDR du backend                                      |
| `oidcClientId`               | `portail`           | client_id OIDC                                            |
| `ldap.url`                   | `ldap://127.0.0.1`  | URL du serveur LDAP (`ldap://` ou `ldaps://`)             |
| `ldap.baseDn`                | `dc=example,dc=com` | suffixe LDAP                                              |
| `ldap.adminPasswordFile`     | (requis)            | mot de passe `cn=admin,<baseDn>` (backend + Authelia)     |
| `oidc.clientSecretFile`      | (requis)            | client_secret partagé backend/Authelia                    |
| `vhost.enable`               | `true`              | vhost nginx (frontend en `root` + proxy `/api`)           |
| `vhost.hostName`             | (si vhost)          | server_name nginx + CN du certificat                      |
| `vhost.forceSSL`             | `true`              | HTTPS + redirection 80→443                                |
| `vhost.sslCertificate{,Key}` | (si forceSSL)       | certificat/clé TLS fournis par l'hôte (agenix/sops)       |
| `extraBackendEnvironment`    | `{}`                | env backend additionnelle (non secrète)                   |
| `apps`                       | `[]`                | applications listées sur la page d'accueil                |

### Applications affichées

`services.portail.apps` alimente la page d'accueil : chaque entrée = un lien (nom + URL, description et icône optionnelles), trié par nom. Le module le rend dans un `/apps.json` statique servi par le vhost nginx — le backend n'intervient pas. En dev, la liste vient de `frontend/public/apps.json` (servie par vite, modifiable à chaud).

```nix
services.portail.apps = [
  { name = "Authelia"; url = "https://auth.portail.example.com"; description = "SSO"; }
  { name = "Syncthing"; url = "https://sync.portail.example.com"; icon = "https://syncthing.net/img/logo.png"; }
];
```

Avant `nixos-rebuild switch` :

```bash
nix build .#portail-backend .#portail-frontend
nix flake check
```

## Développement

```bash
devenv up                    # lance tout : backend 3000, frontend 5173, LDAP 3890/3891, Authelia 9091
openfrontend                 # ouvre le portail http://127.0.0.1:5173
resetUsers                   # réinitialise les comptes LDAP dev (admin/admin, user/user)
devenv down                  # arrêt
just test                    # backend (cargo test --features test-api) + frontend (vitest)
treefmt                      # format (nixfmt + rustfmt + prettier)
```

---

## English

Personal portal to manage the users of my NixOS NAS and show a page gathering its applications.

### NixOS deployment (flake)

The flake exposes `packages.portail-backend`, `packages.portail-frontend` and `nixosModules.default` (the `services.portail` module: backend systemd service + frontend nginx vhost, `/api` proxy, `forceSSL` by default). OpenLDAP and Authelia are provisioned by the host config.

```nix
{ inputs, ... }:
{
  inputs.portail.url = "github:adavida/portail_nas";
  inputs.portail.inputs.nixpkgs.follows = "nixpkgs";

  imports = [ inputs.portail.nixosModules.default ];

  services.portail = {
    enable = true;
    package = inputs.portail.packages.x86_64-linux.portail-backend;
    appUrl = "https://portail.example.com";
    issuerUrl = "https://auth.portail.example.com";
    vhost.hostName = "portail.example.com";
    vhost.sslCertificate = "/etc/portail/ssl/fullchain.pem";
    vhost.sslCertificateKey = "/etc/portail/ssl/key.pem";
    ldap.baseDn = "dc=example,dc=com";
    ldap.adminPasswordFile = "/etc/portail/ldap-admin-pw";
    oidc.clientSecretFile = "/etc/portail/oidc-client-secret";
  };
}
```

| Option                       | Default             | Purpose                                                |
| ---------------------------- | ------------------- | ------------------------------------------------------ |
| `appUrl`                     | (required)          | public portal URL (backend APP_URL + OIDC redirect)    |
| `issuerUrl`                  | (required)          | public Authelia URL (OIDC_ISSUER_URL)                  |
| `package`                    | (required)          | backend binary (e.g. `inputs.portail.packages...`)     |
| `bindAddress`                | `127.0.0.1:3000`    | backend BIND_ADDR                                      |
| `oidcClientId`               | `portail`           | OIDC client_id                                         |
| `ldap.url`                   | `ldap://127.0.0.1`  | LDAP server URL (`ldap://` or `ldaps://`)              |
| `ldap.baseDn`                | `dc=example,dc=com` | LDAP suffix                                            |
| `ldap.adminPasswordFile`     | (required)          | password of `cn=admin,<baseDn>` (backend + Authelia)   |
| `oidc.clientSecretFile`      | (required)          | client_secret shared by backend/Authelia               |
| `vhost.enable`               | `true`              | nginx vhost (frontend as `root` + `/api` proxy)        |
| `vhost.hostName`             | (with vhost)        | nginx server_name + certificate CN                     |
| `vhost.forceSSL`             | `true`              | HTTPS + 80→443 redirect                                |
| `vhost.sslCertificate{,Key}` | (with forceSSL)     | TLS certificate/key provided by the host (agenix/sops) |
| `extraBackendEnvironment`    | `{}`                | additional backend env (non-secret)                    |
| `apps`                       | `[]`                | applications listed on the home page                   |

#### Listed applications

`services.portail.apps` feeds the home page: each entry = a link (name + URL, optional description and icon), sorted by name. The module renders it into a static `/apps.json` served by the nginx vhost — the backend is not involved. In dev, the list comes from `frontend/public/apps.json` (served by vite, hot-editable).

```nix
services.portail.apps = [
  { name = "Authelia"; url = "https://auth.portail.example.com"; description = "SSO"; }
  { name = "Syncthing"; url = "https://sync.portail.example.com"; icon = "https://syncthing.net/img/logo.png"; }
];
```

Before `nixos-rebuild switch`:

```bash
nix build .#portail-backend .#portail-frontend
nix flake check
```

### Development

```bash
devenv up                    # starts everything: backend 3000, frontend 5173, LDAP 3890/3891, Authelia 9091
openfrontend                 # opens the portal http://127.0.0.1:5173
resetUsers                   # reseed dev LDAP accounts (admin/admin, user/user)
devenv down                  # stop
just test                    # backend (cargo test --features test-api) + frontend (vitest)
treefmt                      # formatting (nixfmt + rustfmt + prettier)
```
