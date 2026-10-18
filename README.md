# Portail

**Français** | [English](#english)

Portail perso pour gérer les utilisateurs de mon NAS NixOS et afficher une page regroupant ses applications.

## Déploiement NixOS (flake)

Le flake expose `packages.portail-backend`, `packages.portail-frontend` et `nixosModules.default` (module `services.portail` : service systemd du backend + vhost nginx du frontend, proxy `/api`, `forceSSL` par défaut). OpenLDAP est provisionné par la config hôte. Le backend est **son propre fournisseur OIDC** (`/.well-known/openid-configuration`, `/jwks.json`, `/api/oidc/*`, `/logout` servis sur `appUrl`) : il génère au premier démarrage la clé de signature et `clients.json` dans `oidc.secretsDir` — ajouter un client = éditer le fichier, sans redémarrage.

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
    vhost.hostName = "portail.example.com";
    vhost.sslCertificate = "/etc/portail/ssl/fullchain.pem";
    vhost.sslCertificateKey = "/etc/portail/ssl/key.pem";
    ldap.baseDn = "dc=example,dc=com";
    ldap.adminPasswordFile = "/etc/portail/ldap-admin-pw";
    lang = "fr";
  };
}
```

| Option                       | Défaut                  | Rôle                                                                                       |
| ---------------------------- | ----------------------- | ------------------------------------------------------------------------------------------ |
| `appUrl`                     | (requis)                | URL publique du portail = issuer OIDC (APP_URL + redirect `/callback`)                     |
| `package`                    | (requis)                | binaire backend (ex: `inputs.portail.packages...`)                                         |
| `bindAddress`                | `127.0.0.1:3000`        | BIND_ADDR du backend                                                                       |
| `oidcClientId`               | `portail`               | client_id du portail (généré dans `clients.json` au 1er start)                             |
| `oidc.secretsDir`            | `/var/lib/portail/oidc` | répertoire des secrets OIDC générés (clé RSA + clients.json)                               |
| `lang`                       | `en`                    | langue par défaut de l'interface (`fr`/`en`)                                               |
| `ldap.url`                   | `ldap://127.0.0.1`      | URL du serveur LDAP (`ldap://` ou `ldaps://`)                                              |
| `ldap.baseDn`                | `dc=example,dc=com`     | suffixe LDAP                                                                               |
| `ldap.adminPasswordFile`     | (requis)                | mot de passe `cn=admin,<baseDn>`                                                           |
| `vhost.enable`               | `true`                  | vhost nginx (frontend en `root` + proxy `/api`, `/jwks.json`, `/.well-known/*`, `/logout`) |
| `vhost.hostName`             | (si vhost)              | server_name nginx + CN du certificat                                                       |
| `vhost.forceSSL`             | `true`                  | HTTPS + redirection 80→443                                                                 |
| `vhost.sslCertificate{,Key}` | (si forceSSL)           | certificat/clé TLS fournis par l'hôte (agenix/sops)                                        |
| `extraBackendEnvironment`    | `{}`                    | env backend additionnelle (non secrète)                                                    |
| `apps`                       | `[]`                    | applications listées sur la page d'accueil                                                 |

### Applications affichées

`services.portail.apps` alimente la page d'accueil : chaque entrée = un lien (nom + URL, description et icône optionnelles), trié par nom. Le module le rend dans un `/apps.json` statique servi par le vhost nginx — le backend n'intervient pas. En dev, la liste vient de `frontend/public/apps.json` (servie par vite, modifiable à chaud).

```nix
services.portail.apps = [
  { name = "Vaultwarden"; url = "https://vault.portail.example.com"; description = "Mots de passe"; }
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
devenv up                    # lance tout : backend 3000 (OIDC inclus), frontend 5173, LDAP 3890/3891
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

The flake exposes `packages.portail-backend`, `packages.portail-frontend` and `nixosModules.default` (the `services.portail` module: backend systemd service + frontend nginx vhost, `/api` proxy, `forceSSL` by default). OpenLDAP is provisioned by the host config. The backend is **its own OIDC provider** (`/.well-known/openid-configuration`, `/jwks.json`, `/api/oidc/*`, `/logout` served on `appUrl`): on first start it generates the signing key and `clients.json` in `oidc.secretsDir` — adding a client = edit the file, no restart.

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
    vhost.hostName = "portail.example.com";
    vhost.sslCertificate = "/etc/portail/ssl/fullchain.pem";
    vhost.sslCertificateKey = "/etc/portail/ssl/key.pem";
    ldap.baseDn = "dc=example,dc=com";
    ldap.adminPasswordFile = "/etc/portail/ldap-admin-pw";
    lang = "fr";
  };
}
```

| Option                       | Default                 | Purpose                                                                                    |
| ---------------------------- | ----------------------- | ------------------------------------------------------------------------------------------ |
| `appUrl`                     | (required)              | public portal URL = OIDC issuer (APP_URL + `/callback` redirect)                           |
| `package`                    | (required)              | backend binary (e.g. `inputs.portail.packages...`)                                         |
| `bindAddress`                | `127.0.0.1:3000`        | backend BIND_ADDR                                                                          |
| `oidcClientId`               | `portail`               | portal client_id (generated into `clients.json` on first start)                            |
| `oidc.secretsDir`            | `/var/lib/portail/oidc` | OIDC secrets directory (RSA key + clients.json)                                            |
| `lang`                       | `en`                    | default UI language (`fr`/`en`)                                                            |
| `ldap.url`                   | `ldap://127.0.0.1`      | LDAP server URL (`ldap://` or `ldaps://`)                                                  |
| `ldap.baseDn`                | `dc=example,dc=com`     | LDAP suffix                                                                                |
| `ldap.adminPasswordFile`     | (required)              | password of `cn=admin,<baseDn>`                                                            |
| `vhost.enable`               | `true`                  | nginx vhost (frontend as `root` + `/api`, `/jwks.json`, `/.well-known/*`, `/logout` proxy) |
| `vhost.hostName`             | (with vhost)            | nginx server_name + certificate CN                                                         |
| `vhost.forceSSL`             | `true`                  | HTTPS + 80→443 redirect                                                                    |
| `vhost.sslCertificate{,Key}` | (with forceSSL)         | TLS certificate/key provided by the host (agenix/sops)                                     |
| `extraBackendEnvironment`    | `{}`                    | additional backend env (non-secret)                                                        |
| `apps`                       | `[]`                    | applications listed on the home page                                                       |

#### Listed applications

`services.portail.apps` feeds the home page: each entry = a link (name + URL, optional description and icon), sorted by name. The module renders it into a static `/apps.json` served by the nginx vhost — the backend is not involved. In dev, the list comes from `frontend/public/apps.json` (served by vite, hot-editable).

```nix
services.portail.apps = [
  { name = "Vaultwarden"; url = "https://vault.portail.example.com"; description = "Mots de passe"; }
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
devenv up                    # starts everything: backend 3000 (incl. OIDC), frontend 5173, LDAP 3890/3891
openfrontend                 # opens the portal http://127.0.0.1:5173
resetUsers                   # reseed dev LDAP accounts (admin/admin, user/user)
devenv down                  # stop
just test                    # backend (cargo test --features test-api) + frontend (vitest)
treefmt                      # formatting (nixfmt + rustfmt + prettier)
```
