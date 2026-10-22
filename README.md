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
    ldap = {
      adminPasswordFile = "/etc/portail/ldap-admin-pw";
      baseDn = "dc=example,dc=com";
    };
    vhost = {
      hostName = "portail.example.com";
      sslCertificate = "/etc/portail/ssl/fullchain.pem";
      sslCertificateKey = "/etc/portail/ssl/key.pem";
    };
    appUrl = "https://portail.example.com";
    enable = true;
    lang = "fr";
    package = inputs.portail.packages.x86_64-linux.portail-backend;
  };
}
```

| Option                       | Défaut                  | Rôle                                                                                        |
| ---------------------------- | ----------------------- | ------------------------------------------------------------------------------------------- |
| `appUrl`                     | (requis)                | URL publique du portail = issuer OIDC (APP_URL + redirect `/callback`)                      |
| `package`                    | (requis)                | binaire backend (ex: `inputs.portail.packages...`)                                          |
| `bindAddress`                | `127.0.0.1:3000`        | BIND_ADDR du backend                                                                        |
| `oidcClientId`               | `portail`               | client_id du portail (généré dans `clients.json` au 1er start)                              |
| `oidc.secretsDir`            | `/var/lib/portail/oidc` | répertoire des secrets OIDC générés (clé RSA + clients.json + `<clientId>_secret` des apps) |
| `lang`                       | `en`                    | langue par défaut de l'interface (`fr`/`en`)                                                |
| `ldap.url`                   | `ldap://127.0.0.1`      | URL du serveur LDAP (`ldap://` ou `ldaps://`)                                               |
| `ldap.baseDn`                | `dc=example,dc=com`     | suffixe LDAP                                                                                |
| `ldap.peopleOu`              | `people`                | OU des entrées utilisateurs (`LDAP_PEOPLE_OU`)                                              |
| `ldap.groupsOu`              | `groups`                | OU des entrées groupes (`LDAP_GROUPS_OU`)                                                   |
| `ldap.caCertificateFile`     | `null`                  | bundle CA PEM validant le endpoint `ldaps://` (`SSL_CERT_FILE`)                             |
| `ldap.adminPasswordFile`     | (requis)                | mot de passe `cn=admin,<baseDn>`                                                            |
| `vhost.enable`               | `true`                  | vhost nginx (frontend en `root` + proxy `/api`, `/jwks.json`, `/.well-known/*`, `/logout`)  |
| `vhost.hostName`             | (si vhost)              | server_name nginx + CN du certificat                                                        |
| `vhost.forceSSL`             | `true`                  | HTTPS + redirection 80→443                                                                  |
| `vhost.sslCertificate{,Key}` | (si forceSSL)           | certificat/clé TLS fournis par l'hôte (agenix/sops)                                         |
| `extraBackendEnvironment`    | `{}`                    | env backend additionnelle (non secrète)                                                     |
| `apps`                       | `[]`                    | applications listées sur la page d'accueil                                                  |

### Applications affichées

`services.portail.apps` alimente la page d'accueil : chaque entrée = un lien (nom + URL, description et icône optionnelles), trié par nom. Le module le rend dans un `/apps.json` statique servi par le vhost nginx — le backend n'intervient pas. En dev, la liste vient de `frontend/public/apps.json` (servie par vite, modifiable à chaud). Chaque entrée accepte un sous-objet `oidc` désactivé par défaut : `oidc.enable = true` enregistre l'application comme client OIDC du portail dans `clients.json` (relu à chaud, sans redémarrage), avec `clientId` (défaut : nom en minuscules), `redirectUris`, `scopes`, et soit `public = true` soit un secret **généré** au premier activation dans `oidc.secretsDir/<clientId>_secret` (0600, jamais écrasé) — surchargeable par `secretFile` (agenix/sops). Une app retirée de `apps` est purgée de `clients.json` (avec son `<clientId>_secret`), les entrées ajoutées à la main sont conservées.

```nix
services.portail.apps = [
  { name = "Vaultwarden"; url = "https://vault.portail.example.com"; description = "Mots de passe"; }
  { name = "Syncthing"; url = "https://sync.portail.example.com"; icon = "https://syncthing.net/img/logo.png"; }
  {
    name = "Nextcloud";
    url = "https://cloud.portail.example.com";
    description = "Files";
    oidc = {
      enable = true;
      clientId = "nextcloud";
      redirectUris = [ "https://cloud.portail.example.com/apps/oauth2/redirect_ui" ];
    };
  }
];
```

Avant `nixos-rebuild switch` :

```bash
nix build .#portail-backend .#portail-frontend
nix flake check
```

## Développement

```bash
devenv up                    # lance tout : backend 3000 (OIDC inclus), frontend 5173, LDAP 3890/3891, oauth debugger 3030
openfrontend                 # ouvre le portail http://127.0.0.1:5173
resetUsers                   # réinitialise les comptes LDAP dev (admin/admin, user/user)
devenv down                  # arrêt
just test                    # backend (cargo test --features test-api) + frontend (vitest)
treefmt                      # format (nixfmt + rustfmt + prettier)
```

### Tester le service OIDC (oauth debugger)

`devenv up` lance aussi `oauthdebugger` (`uvx oauth2-debugger`, `http://127.0.0.1:3030`). Au démarrage il enregistre le client **public** `oauth-debugger` (secret vide, `redirect_uri http://127.0.0.1:3030/callback`, scopes `openid profile email groups`) dans `$OIDC_SECRETS_DIR/clients.json` — relu à chaud, sans redémarrage.

Ouvrir `http://127.0.0.1:3030`, laisser les champs pré-remplis (`--issuer $OIDC_ISSUER_URL`, `--client-id oauth-debugger`), puis dérouler le flux Authorization Code + PKCE : `/api/oidc/authorization` → page de login (comptes LDAP dev `admin`/`admin` ou `user`/`user` via `resetUsers`) → callback avec `code` → `POST /api/oidc/token` (les clients publics n'envoient pas de secret, PKCE obligatoire) → `id_token`/`access_token` décodés + vérifiés contre `/jwks.json`, puis appel `/api/oidc/userinfo`.

Un client public n'a pas besoin de secret ; un client confidentiel (ex: `portail-dev`) doit le lire dans `clients.json` pour `client_secret_basic` — l'éditeur pour l'ajouter, pas de redémarrage.

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
    ldap = {
      adminPasswordFile = "/etc/portail/ldap-admin-pw";
      baseDn = "dc=example,dc=com";
    };
    vhost = {
      hostName = "portail.example.com";
      sslCertificate = "/etc/portail/ssl/fullchain.pem";
      sslCertificateKey = "/etc/portail/ssl/key.pem";
    };
    appUrl = "https://portail.example.com";
    enable = true;
    lang = "fr";
    package = inputs.portail.packages.x86_64-linux.portail-backend;
  };
}
```

| Option                       | Default                 | Purpose                                                                                    |
| ---------------------------- | ----------------------- | ------------------------------------------------------------------------------------------ |
| `appUrl`                     | (required)              | public portal URL = OIDC issuer (APP_URL + `/callback` redirect)                           |
| `package`                    | (required)              | backend binary (e.g. `inputs.portail.packages...`)                                         |
| `bindAddress`                | `127.0.0.1:3000`        | backend BIND_ADDR                                                                          |
| `oidcClientId`               | `portail`               | portal client_id (generated into `clients.json` on first start)                            |
| `oidc.secretsDir`            | `/var/lib/portail/oidc` | OIDC secrets directory (RSA key + clients.json + app `<clientId>_secret` secrets)          |
| `lang`                       | `en`                    | default UI language (`fr`/`en`)                                                            |
| `ldap.url`                   | `ldap://127.0.0.1`      | LDAP server URL (`ldap://` or `ldaps://`)                                                  |
| `ldap.baseDn`                | `dc=example,dc=com`     | LDAP suffix                                                                                |
| `ldap.peopleOu`              | `people`                | OU holding user entries (`LDAP_PEOPLE_OU`)                                                 |
| `ldap.groupsOu`              | `groups`                | OU holding group entries (`LDAP_GROUPS_OU`)                                                |
| `ldap.caCertificateFile`     | `null`                  | PEM CA bundle validating the `ldaps://` endpoint (`SSL_CERT_FILE`)                         |
| `ldap.adminPasswordFile`     | (required)              | password of `cn=admin,<baseDn>`                                                            |
| `vhost.enable`               | `true`                  | nginx vhost (frontend as `root` + `/api`, `/jwks.json`, `/.well-known/*`, `/logout` proxy) |
| `vhost.hostName`             | (with vhost)            | nginx server_name + certificate CN                                                         |
| `vhost.forceSSL`             | `true`                  | HTTPS + 80→443 redirect                                                                    |
| `vhost.sslCertificate{,Key}` | (with forceSSL)         | TLS certificate/key provided by the host (agenix/sops)                                     |
| `extraBackendEnvironment`    | `{}`                    | additional backend env (non-secret)                                                        |
| `apps`                       | `[]`                    | applications listed on the home page                                                       |

#### Listed applications

`services.portail.apps` feeds the home page: each entry = a link (name + URL, optional description and icon), sorted by name. The module renders it into a static `/apps.json` served by the nginx vhost — the backend is not involved. In dev, the list comes from `frontend/public/apps.json` (served by vite, hot-editable). Each entry accepts an `oidc` sub-object, disabled by default: `oidc.enable = true` registers the app as an OIDC client of the portal in `clients.json` (re-read live, no restart), with `clientId` (default: lowercased name), `redirectUris`, `scopes`, and either `public = true` or a **generated** secret written to `oidc.secretsDir/<clientId>_secret` on first activation (0600, never overwritten) — overridable with `secretFile` (agenix/sops). An app removed from `apps` is pruned from `clients.json` (along with its `<clientId>_secret`); manually added entries are kept.

```nix
services.portail.apps = [
  { name = "Vaultwarden"; url = "https://vault.portail.example.com"; description = "Mots de passe"; }
  { name = "Syncthing"; url = "https://sync.portail.example.com"; icon = "https://syncthing.net/img/logo.png"; }
  {
    name = "Nextcloud";
    url = "https://cloud.portail.example.com";
    description = "Files";
    oidc = {
      enable = true;
      clientId = "nextcloud";
      redirectUris = [ "https://cloud.portail.example.com/apps/oauth2/redirect_ui" ];
    };
  }
];
```

Before `nixos-rebuild switch`:

```bash
nix build .#portail-backend .#portail-frontend
nix flake check
```

### Development

```bash
devenv up                    # starts everything: backend 3000 (incl. OIDC), frontend 5173, LDAP 3890/3891, oauth debugger 3030
openfrontend                 # opens the portal http://127.0.0.1:5173
resetUsers                   # reseed dev LDAP accounts (admin/admin, user/user)
devenv down                  # stop
just test                    # backend (cargo test --features test-api) + frontend (vitest)
treefmt                      # formatting (nixfmt + rustfmt + prettier)
```

### Test the OIDC service (oauth debugger)

`devenv up` also starts `oauthdebugger` (`uvx oauth2-debugger`, `http://127.0.0.1:3030`). On startup it registers the **public** client `oauth-debugger` (empty secret, `redirect_uri http://127.0.0.1:3030/callback`, scopes `openid profile email groups`) into `$OIDC_SECRETS_DIR/clients.json` — re-read live, no restart.

Open `http://127.0.0.1:3030`, keep the prefilled fields (`--issuer $OIDC_ISSUER_URL`, `--client-id oauth-debugger`), then run the Authorization Code + PKCE flow: `/api/oidc/authorization` → login page (dev LDAP accounts `admin`/`admin` or `user`/`user` via `resetUsers`) → callback with `code` → `POST /api/oidc/token` (public clients send no secret, PKCE mandatory) → decoded `id_token`/`access_token` verified against `/jwks.json`, then call `/api/oidc/userinfo`.

A public client needs no secret; a confidential client (e.g. `portail-dev`) must read its secret from `clients.json` for `client_secret_basic` — edit the file to add one, no restart.
