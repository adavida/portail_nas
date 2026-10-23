# Portail

**Français** | [English](#english)

Portail perso pour gérer les utilisateurs de mon NAS NixOS et afficher une page regroupant ses applications.

## Déploiement NixOS (flake)

Le flake expose `packages.portail-backend` (= `packages.default`), `packages.portail-frontend` et `nixosModules.default` (module `services.portail` : service systemd du backend + vhost nginx du frontend, proxy `/api`, `forceSSL` par défaut). OpenLDAP est provisionné par la config hôte. Le backend est **son propre fournisseur OIDC** (`/.well-known/openid-configuration`, `/jwks.json`, `/api/oidc/*`, `/logout` servis sur `appUrl`) : il génère sa clé de signature au premier démarrage dans `oidc.secretsDir` et lit `clients.json` (écrit **uniquement** par le script d'activation) à chaque requête — ajouter un client = éditer le fichier, sans redémarrage.

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
      caCertificateFile = null;
      groupsOu = "groups";
      peopleOu = "people";
      url = "ldap://127.0.0.1";
    };
    vhost = {
      enable = true;
      forceSSL = true;
      hostName = "portail.example.com";
      sslCertificate = "/etc/portail/ssl/fullchain.pem";
      sslCertificateKey = "/etc/portail/ssl/key.pem";
    };
    appUrl = "https://portail.example.com";
    apps = [ ];
    bindAddress = "127.0.0.1:3000";
    enable = true;
    extraBackendEnvironment = { };
    lang = "fr";
    oidc.logSecretEvents = true;
    oidc.secretsDir = "/var/lib/portail/oidc";
    oidcClientId = "portail";
    package = inputs.portail.packages.x86_64-linux.portail-backend;
  };
}
```

| Option                       | Défaut                  | Rôle                                                                                          |
| ---------------------------- | ----------------------- | --------------------------------------------------------------------------------------------- |
| `appUrl`                     | (requis)                | URL publique du portail = issuer OIDC (APP_URL + redirect `/callback`)                        |
| `package`                    | (requis)                | binaire backend (ex: `inputs.portail.packages...`)                                            |
| `bindAddress`                | `127.0.0.1:3000`        | BIND_ADDR du backend                                                                          |
| `oidcClientId`               | `portail`               | client_id du portail (ajouté **activé** à `clients.json` à l'activation)                      |
| `oidc.secretsDir`            | `/var/lib/portail/oidc` | répertoire des secrets OIDC (clé RSA + clients.json + fichiers `<clientId>_secret`)           |
| `oidc.logSecretEvents`       | `true`                  | log INFO backend/ExecStartPre sur les fichiers de secrets OIDC (générer, garder, ACL, purger) |
| `lang`                       | `en`                    | langue par défaut de l'interface (`fr`/`en`)                                                  |
| `ldap.url`                   | `ldap://127.0.0.1`      | URL du serveur LDAP (`ldap://` ou `ldaps://`)                                                 |
| `ldap.baseDn`                | `dc=example,dc=com`     | suffixe LDAP                                                                                  |
| `ldap.peopleOu`              | `people`                | OU des entrées utilisateurs (`LDAP_PEOPLE_OU`)                                                |
| `ldap.groupsOu`              | `groups`                | OU des entrées groupes (`LDAP_GROUPS_OU`)                                                     |
| `ldap.caCertificateFile`     | `null`                  | bundle CA PEM validant le endpoint `ldaps://` (`SSL_CERT_FILE`)                               |
| `ldap.adminPasswordFile`     | (requis)                | mot de passe `cn=admin,<baseDn>`                                                              |
| `vhost.enable`               | `true`                  | vhost nginx (frontend en `root` + proxy `/api`, `/jwks.json`, `/.well-known/*`, `/logout`)    |
| `vhost.hostName`             | (si vhost)              | server_name nginx + CN du certificat                                                          |
| `vhost.forceSSL`             | `true`                  | HTTPS + redirection 80→443                                                                    |
| `vhost.sslCertificate{,Key}` | (si forceSSL)           | certificat/clé TLS fournis par l'hôte (agenix/sops)                                           |
| `extraBackendEnvironment`    | `{}`                    | env backend additionnelle (non secrète)                                                       |
| `apps`                       | `[]`                    | applications listées sur la page d'accueil                                                    |

### Le fichier `clients.json`

Les clients OIDC vivent dans `oidc.secretsDir/clients.json` (passé au backend via `OIDC_CLIENTS_FILE`). **Seul Nix l'écrit** (script `ExecStartPre`) ; le backend Rust ne fait que le **lire** à chaque requête — ajouter un client ou l'activer = éditer le fichier, sans redémarrage. Les secrets ne sont **jamais dans le JSON** : un client confidentiel référence dans `secret_file` le fichier qui contient son secret — nom relatif à côté de `clients.json`, ou chemin absolu (`secretFile` agenix/sops) ; le backend génère les fichiers manquants au premier démarrage (0600, jamais écrasés). Un client enregistré est `enabled: false` par défaut : **l'activer est manuel** (`"enabled": true` dans le JSON, valeur préservée aux activations suivantes).

| Champ           | Rôle                                                            |
| --------------- | --------------------------------------------------------------- |
| `id`            | client_id                                                       |
| `secret_file`   | fichier du secret ; `null` = client public (PKCE obligatoire)   |
| `enabled`       | défaut `false` ; passer à `true` = activation manuelle          |
| `redirect_uris` | URLs exactes autorisées au authorize                            |
| `scopes`        | scopes accordables par ce client                                |
| `managed`       | interne Nix : géré par le module (purgé si l'app quitte `apps`) |

```json
[
  {
    "id": "portail",
    "secret_file": "portail_secret",
    "enabled": true,
    "redirect_uris": ["https://portail.example.com/callback"],
    "scopes": ["openid", "profile", "email", "groups"]
  },
  {
    "id": "nextcloud",
    "secret_file": "nextcloud_secret",
    "enabled": false,
    "redirect_uris": [
      "https://cloud.portail.example.com/apps/oauth2/redirect_ui"
    ],
    "scopes": ["openid", "profile", "email", "groups"],
    "managed": true
  }
]
```

### Applications affichées

`services.portail.apps` alimente la page d'accueil : chaque entrée = un lien (nom + URL, description et icône optionnelles), trié par nom. Le module le rend dans un `/apps.json` statique servi par le vhost nginx — le backend n'intervient pas. En dev, la liste vient de `frontend/public/apps.json` (servie par vite, modifiable à chaud). Chaque entrée accepte un sous-objet `oidc` désactivé par défaut : `oidc.enable = true` enregistre l'application comme client OIDC du portail dans `clients.json` (relu à chaud, sans redémarrage), avec `clientId` (défaut : nom en minuscules), `redirectUris`, `scopes`, un `secret_file` (`null` si `public = true` ; sinon `oidc.secretsDir/<clientId>_secret`, généré par le backend au premier démarrage — 0600, jamais écrasé — ou chemin absolu surchargeable par `secretFile` agenix/sops) et `enabled: false` — **l'activer est manuel**, dans le JSON. `oidc.user`/`oidc.group` (défaut : vides) déclarent le user/groupe linux dotés d'un **ACL de lecture** sur le secret généré (ré-appliqué par le backend à chaque démarrage, `ensure()` — aucune capacité requise). Une app retirée de `apps` est purgée de `clients.json` (avec son `<clientId>_secret`), les entrées ajoutées à la main sont conservées.

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

`devenv up` lance aussi `oauthdebugger` (`uvx oauth2-debugger`, `http://127.0.0.1:3030`). Au démarrage il enregistre le client **public** `oauth-debugger` (pas de secret, `redirect_uri http://127.0.0.1:3030/callback`, scopes `openid profile email groups`) dans `$OIDC_CLIENTS_FILE` — relu à chaud, sans redémarrage — **désactivé** (`enabled: false`). L'activer pour la session de test :

```bash
jq 'map(if .id == "oauth-debugger" then .enabled = true else . end)' "$OIDC_CLIENTS_FILE" > "$OIDC_CLIENTS_FILE.tmp" && mv "$OIDC_CLIENTS_FILE.tmp" "$OIDC_CLIENTS_FILE"
```

Ouvrir `http://127.0.0.1:3030`, laisser les champs pré-remplis (`--issuer $OIDC_ISSUER_URL`, `--client-id oauth-debugger`), puis dérouler le flux Authorization Code + PKCE : `/api/oidc/authorization` → page de login (comptes LDAP dev `admin`/`admin` ou `user`/`user` via `resetUsers`) → callback avec `code` → `POST /api/oidc/token` (les clients publics n'envoient pas de secret, PKCE obligatoire) → `id_token`/`access_token` décodés + vérifiés contre `/jwks.json`, puis appel `/api/oidc/userinfo`.

Un client public n'a pas besoin de secret ; le secret d'un client confidentiel (ex: `portail-dev`) vit dans le fichier pointé par `secret_file` pour `client_secret_basic` — le JSON ne contient jamais le secret.

---

## English

Personal portal to manage the users of my NixOS NAS and show a page gathering its applications.

### NixOS deployment (flake)

The flake exposes `packages.portail-backend` (= `packages.default`), `packages.portail-frontend` and `nixosModules.default` (the `services.portail` module: backend systemd service + frontend nginx vhost, `/api` proxy, `forceSSL` by default). OpenLDAP is provisioned by the host config. The backend is **its own OIDC provider** (`/.well-known/openid-configuration`, `/jwks.json`, `/api/oidc/*`, `/logout` served on `appUrl`): it generates its signing key on first start in `oidc.secretsDir` and reads `clients.json` (written **only** by the activation script) on every request — adding a client = edit the file, no restart.

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
      caCertificateFile = null;
      groupsOu = "groups";
      peopleOu = "people";
      url = "ldap://127.0.0.1";
    };
    vhost = {
      enable = true;
      forceSSL = true;
      hostName = "portail.example.com";
      sslCertificate = "/etc/portail/ssl/fullchain.pem";
      sslCertificateKey = "/etc/portail/ssl/key.pem";
    };
    appUrl = "https://portail.example.com";
    apps = [ ];
    bindAddress = "127.0.0.1:3000";
    enable = true;
    extraBackendEnvironment = { };
    lang = "fr";
    oidc.logSecretEvents = true;
    oidc.secretsDir = "/var/lib/portail/oidc";
    oidcClientId = "portail";
    package = inputs.portail.packages.x86_64-linux.portail-backend;
  };
}
```

| Option                       | Default                 | Purpose                                                                                    |
| ---------------------------- | ----------------------- | ------------------------------------------------------------------------------------------ |
| `appUrl`                     | (required)              | public portal URL = OIDC issuer (APP_URL + `/callback` redirect)                           |
| `package`                    | (required)              | backend binary (e.g. `inputs.portail.packages...`)                                         |
| `bindAddress`                | `127.0.0.1:3000`        | backend BIND_ADDR                                                                          |
| `oidcClientId`               | `portail`               | portal client_id (upserted **enabled** into `clients.json` at activation)                  |
| `oidc.secretsDir`            | `/var/lib/portail/oidc` | OIDC secrets directory (RSA key + clients.json + `<clientId>_secret` files)                |
| `oidc.logSecretEvents`       | `true`                  | backend/ExecStartPre INFO logs on OIDC secret files (generate, keep, ACL, prune)           |
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

#### The `clients.json` file

OIDC clients live in `oidc.secretsDir/clients.json` (passed to the backend via `OIDC_CLIENTS_FILE`). **Only Nix writes it** (the `ExecStartPre` script); the Rust backend **only reads** it on every request — adding a client or activating one = edit the file, no restart. Secrets are **never in the JSON**: a confidential client references through `secret_file` the file holding its secret — name relative to `clients.json`, or absolute path (`secretFile` agenix/sops); the backend generates missing files on first start (0600, never overwritten). A registered client is `enabled: false` by default: **activating it is manual** (`"enabled": true` in the JSON, the value survives later activations).

| Field           | Purpose                                                          |
| --------------- | ---------------------------------------------------------------- |
| `id`            | client_id                                                        |
| `secret_file`   | secret file; `null` = public client (PKCE mandatory)             |
| `enabled`       | defaults to `false`; set `true` = manual activation              |
| `redirect_uris` | exact URLs allowed at authorize                                  |
| `scopes`        | scopes this client may request                                   |
| `managed`       | Nix internal: module-managed (pruned when the app leaves `apps`) |

```json
[
  {
    "id": "portail",
    "secret_file": "portail_secret",
    "enabled": true,
    "redirect_uris": ["https://portail.example.com/callback"],
    "scopes": ["openid", "profile", "email", "groups"]
  },
  {
    "id": "nextcloud",
    "secret_file": "nextcloud_secret",
    "enabled": false,
    "redirect_uris": [
      "https://cloud.portail.example.com/apps/oauth2/redirect_ui"
    ],
    "scopes": ["openid", "profile", "email", "groups"],
    "managed": true
  }
]
```

#### Listed applications

`services.portail.apps` feeds the home page: each entry = a link (name + URL, optional description and icon), sorted by name. The module renders it into a static `/apps.json` served by the nginx vhost — the backend is not involved. In dev, the list comes from `frontend/public/apps.json` (served by vite, hot-editable). Each entry accepts an `oidc` sub-object, disabled by default: `oidc.enable = true` registers the app as an OIDC client of the portal in `clients.json` (re-read live, no restart), with `clientId` (default: lowercased name), `redirectUris`, `scopes`, a `secret_file` (`null` when `public = true`; otherwise `oidc.secretsDir/<clientId>_secret`, generated by the backend on first start — 0600, never overwritten — or an absolute path overridable with `secretFile` agenix/sops) and `enabled: false` — **activating it is manual**, in the JSON. `oidc.user`/`oidc.group` (default: empty) declare the Linux user/group granted **read ACL** on the generated secret (re-applied by the backend at every startup, `ensure()` — no capabilities needed). An app removed from `apps` is pruned from `clients.json` (along with its `<clientId>_secret`); manually added entries are kept.

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

`devenv up` also starts `oauthdebugger` (`uvx oauth2-debugger`, `http://127.0.0.1:3030`). On startup it registers the **public** client `oauth-debugger` (no secret, `redirect_uri http://127.0.0.1:3030/callback`, scopes `openid profile email groups`) into `$OIDC_CLIENTS_FILE` — re-read live, no restart — **disabled** (`enabled: false`). Activate it for the test session:

```bash
jq 'map(if .id == "oauth-debugger" then .enabled = true else . end)' "$OIDC_CLIENTS_FILE" > "$OIDC_CLIENTS_FILE.tmp" && mv "$OIDC_CLIENTS_FILE.tmp" "$OIDC_CLIENTS_FILE"
```

Open `http://127.0.0.1:3030`, keep the prefilled fields (`--issuer $OIDC_ISSUER_URL`, `--client-id oauth-debugger`), then run the Authorization Code + PKCE flow: `/api/oidc/authorization` → login page (dev LDAP accounts `admin`/`admin` or `user`/`user` via `resetUsers`) → callback with `code` → `POST /api/oidc/token` (public clients send no secret, PKCE mandatory) → decoded `id_token`/`access_token` verified against `/jwks.json`, then call `/api/oidc/userinfo`.

A public client needs no secret; a confidential client's secret (e.g. `portail-dev`) lives in the file pointed to by `secret_file` for `client_secret_basic` — the JSON never holds the secret.
