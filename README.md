# Portail

**Français** | [English](#english)

Portail perso pour gérer les utilisateurs de mon NAS NixOS et afficher une page regroupant ses applications.

## Déploiement NixOS (flake)

Le flake expose `packages.portail-backend` (= `packages.default`), `packages.portail-frontend` et `nixosModules.default` (module `services.portail` : service systemd du backend + vhost nginx du frontend avec proxy `/api`, `forceSSL` par défaut). OpenLDAP est provisionné par la config hôte. Le backend est son propre fournisseur OIDC : il génère sa clé de signature au premier démarrage dans `oidc.secretsDir` et déclare ses clients OIDC dans le `config.toml` généré par le module.

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
    attrs.labels = { };
    bindAddress = "127.0.0.1:3000";
    dbPath = "/var/lib/portail/portail.db";
    defaultPhoneCountry = "FR";
    directory = {
      adminPasswordFile = null;
      backend = "ldap";
    };
    enable = true;
    lang = "fr";
    oidc.logSecretEvents = true;
    oidc.secretsDir = "/var/lib/portail/oidc";
    oidcClientId = "portail";
    package = inputs.portail.packages.x86_64-linux.portail-backend;
    security.loginMaxAttempts = 10;
    security.passwordMinLength = 8;
    systemSensors = [ "memory" "cpu" "network" "disk" ];
  };
}
```

| Option                        | Défaut                                | Rôle                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                           |
| ----------------------------- | ------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| `appUrl`                      | (requis)                              | URL publique du portail = issuer OIDC (APP_URL + redirect `/callback`)                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                         |
| `package`                     | (requis)                              | binaire backend (ex: `inputs.portail.packages...`)                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                             |
| `bindAddress`                 | `127.0.0.1:3000`                      | BIND_ADDR du backend                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                           |
| `dbPath`                      | `/var/lib/portail/portail.db`         | chemin de la base SQLite (`db_path`) : subs pairwise, et users/groupes/membres quand `directory.backend = "sqlite"` ; créée au premier démarrage (0600)                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                        |
| `defaultPhoneCountry`         | `"FR"`                                | pays pré-sélectionné des champs téléphone du profil (`default_phone_country`) : indicatif et format national affichés par défaut, modifiables par l'utilisateur via le sélecteur de pays                                                                                                                                                                                                                                                                                                                                                                                                                                                                                       |
| `directory.backend`           | `ldap`                                | backend annuaire (`directory.backend`) : `ldap` (serveur LDAP) ou `sqlite` (users/groupes/mots de passe argon2 dans `dbPath`)                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                  |
| `directory.adminPasswordFile` | `null`                                | fichier du mot de passe de bootstrap du compte `admin` (`directory.admin_password_file`) ; seulement utilisé si `admin` absent ; null = `admin`                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                |
| `attrs.labels`                | `{ }`                                 | catalogue des attributs personnalisés éditables sur la page Profil (`attrs.labels`) : chaque clé = un label, avec `type` (`text` avec `maxChars`, `number` avec `min`/`max`, `email`, `telephone`) et `names` (nom affiché par langue, ex. `{ fr = "Ville"; en = "City"; }`, repli sur le label) ; `{ }` = fonctionnalité désactivée ; en LDAP chaque label du catalogue est un attribut réel (`city: Strasbourg` — le schéma servi doit déclarer l'`attributetype`, ex. `docker/portail.schema`), les valeurs hors catalogue restent packées dans `portailAttr=label=value` ; en SQLite table `user_attrs` ; distribués en claims aux clients selon `apps[].oidc.exposeAttrs` |
| `systemSensors`               | `[ "memory" "cpu" "network" "disk" ]` | blocs affichés sur la page Système (`system_sensors`) ; noms connus : `memory`, `cpu`, `cores` (détail par cœur), `load`, `pressure`, `temps`, `fans`, `network`, `tcp`, `interfaces`, `disk`, `diskio`, `raid`, `uptime`                                                                                                                                                                                                                                                                                                                                                                                                                                                      |
| `oidcClientId`                | `portail`                             | client_id du portail (écrit **activé** dans `oidc.clients` du config généré)                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                   |
| `oidc.secretsDir`             | `/var/lib/portail/oidc`               | répertoire des secrets OIDC (clé RSA + fichiers `<clientId>_secret`)                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                           |
| `oidc.logSecretEvents`        | `true`                                | log INFO backend/ExecStartPre sur les fichiers de secrets OIDC (générer, garder, ACL, purger)                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                  |
| `lang`                        | `en`                                  | langue par défaut de l'interface (`fr`/`en`)                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                   |
| `ldap.url`                    | `ldap://127.0.0.1`                    | URL du serveur LDAP (`ldap://` ou `ldaps://`)                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                  |
| `ldap.baseDn`                 | `dc=example,dc=com`                   | suffixe LDAP                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                   |
| `ldap.peopleOu`               | `people`                              | OU des entrées utilisateurs (`LDAP_PEOPLE_OU`)                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                 |
| `ldap.groupsOu`               | `groups`                              | OU des entrées groupes (`LDAP_GROUPS_OU`)                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                      |
| `ldap.caCertificateFile`      | `null`                                | bundle CA PEM validant le endpoint `ldaps://` (`SSL_CERT_FILE`)                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                |
| `ldap.adminPasswordFile`      | (requis)                              | mot de passe `cn=admin,<baseDn>`                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                               |
| `security.passwordMinLength`  | `8`                                   | longueur minimale des mots de passe, imposée à la création et à la mise à jour d'un utilisateur (`security.password_min_length`)                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                               |
| `security.loginMaxAttempts`   | `10`                                  | tentatives d'échec de connexion consécutives avant verrouillage (`security.login_max_attempts`)                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                |
| `vhost.enable`                | `true`                                | vhost nginx (frontend en `root` + proxy `/api`, `/jwks.json`, `/.well-known/*`, `/logout`)                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                     |
| `vhost.hostName`              | (si vhost)                            | server_name nginx + CN du certificat                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                           |
| `vhost.forceSSL`              | `true`                                | HTTPS + redirection 80→443                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                     |
| `vhost.sslCertificate{,Key}`  | (si forceSSL)                         | certificat/clé TLS fournis par l'hôte (agenix/sops)                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                            |
| `apps`                        | `[]`                                  | applications listées sur la page d'accueil                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                     |

Secrets OIDC : la clé de signature et les secrets des clients confidentiels sont générés dans `oidc.secretsDir` au premier démarrage (0600, jamais écrasés une fois non vides) et ne figurent jamais dans le TOML. Ajouter ou activer un client = `nixos-rebuild switch` + redémarrage du service.

### Applications affichées

Chaque entrée de `apps` = un lien sur la page d'accueil (`name`, `url`, `description` et `icon` optionnels), rendu dans un `/apps.json` statique servi par nginx. Son sous-objet `oidc`, désactivé par défaut, enregistre l'app comme client OIDC du portail : `enable = true` l'écrit activée dans `oidc.clients` du config généré, avec `clientId` (défaut : nom en minuscules), `redirectUris`, `scopes`, `customClaims` (claims statiques `{ name, value, group | notGroup }` émises selon l'appartenance de l'utilisateur au groupe LDAP), `requiredGroup`/`forbiddenGroup` (lien masqué et échange de token refusé 403 selon les groupes), `public = true` pour un client public PKCE sans secret, sinon secret généré dans `oidc.secretsDir/<clientId>_secret` ou chemin absolu surchargé par `secretFile` (agenix/sops), `user`/`group` (ACL de lecture du secret), et `exposeAttrs` (attributs personnalisés du catalogue `attrs.labels` distribués en claims).

```nix
services.portail.apps = [
  { name = "Vaultwarden"; url = "https://vault.portail.example.com"; description = "Mots de passe"; }
  {
    name = "Nextcloud";
    url = "https://cloud.portail.example.com";
    oidc = {
      enable = true;
      clientId = "nextcloud";
      requiredGroup = "nextcloud-users";
      redirectUris = [ "https://cloud.portail.example.com/apps/oauth2/redirect_ui" ];
      customClaims = [
        { name = "role"; value = "user"; }
        { name = "role"; value = "admin"; group = "admins"; }
      ];
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
devenv up                      # lance tout : backend 3000 (OIDC inclus), frontend 5173, LDAP 3890/3891, oauth debugger 3030
devenv down                    # arrêt
openfrontend                   # ouvre le portail http://127.0.0.1:5173
resetAdminLdap                 # réinitialise les comptes LDAP dev (admin/admin, user/user)
resetAdminSqlite               # supprime la db SQLite dev — le backend recrée admin au redémarrage
just test                      # backend (cargo test --features test-api) + frontend (vitest)
treefmt                        # format (nixfmt + rustfmt + prettier)
```

### Lancer le binaire seul

`portail-backend --config <chemin>` — aucune variable d'env de config ; les chemins relatifs du TOML sont résolus contre le répertoire du fichier. Minimal (backend sqlite) :

```toml
app_url = "https://portail.example.com"
bind_addr = "127.0.0.1:3000"
db_path = "portail.db"

[directory]
backend = "sqlite"

[oidc]
issuer_url = "https://portail.example.com"
client_id = "portail"
redirect_uri = "https://portail.example.com/callback"
secrets_dir = "secrets"
clients = [
  { id = "portail", secret_file = "portail_secret", enabled = true, redirect_uris = [
    "https://portail.example.com/callback",
  ], scopes = [ "openid", "profile", "email", "groups" ] },
]
```

Avec `backend = "ldap"`, ajouter `[ldap] url` + `base_dn` (requis). Clés optionnelles et défauts : règle `.opencode/rules/06-env.md` ; exemple complet : `config.test.toml` et `docker/config.toml`.

---

## English

Personal portal to manage the users of my NixOS NAS and show a page gathering its applications.

## NixOS deployment (flake)

The flake exposes `packages.portail-backend` (= `packages.default`), `packages.portail-frontend` and `nixosModules.default` (the `services.portail` module: backend systemd service + frontend nginx vhost with `/api` proxy, `forceSSL` by default). OpenLDAP is provisioned by the host config. The backend is its own OIDC provider: it generates its signing key on first start in `oidc.secretsDir` and declares its OIDC clients in the `config.toml` generated by the module.

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
    attrs.labels = { };
    bindAddress = "127.0.0.1:3000";
    dbPath = "/var/lib/portail/portail.db";
    defaultPhoneCountry = "FR";
    directory = {
      adminPasswordFile = null;
      backend = "ldap";
    };
    enable = true;
    lang = "fr";
    oidc.logSecretEvents = true;
    oidc.secretsDir = "/var/lib/portail/oidc";
    oidcClientId = "portail";
    package = inputs.portail.packages.x86_64-linux.portail-backend;
    security.loginMaxAttempts = 10;
    security.passwordMinLength = 8;
    systemSensors = [ "memory" "cpu" "network" "disk" ];
  };
}
```

| Option                        | Default                               | Purpose                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                |
| ----------------------------- | ------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| `appUrl`                      | (required)                            | public portal URL = OIDC issuer (APP_URL + `/callback` redirect)                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                       |
| `package`                     | (required)                            | backend binary (e.g. `inputs.portail.packages...`)                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                     |
| `bindAddress`                 | `127.0.0.1:3000`                      | backend BIND_ADDR                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                      |
| `dbPath`                      | `/var/lib/portail/portail.db`         | SQLite database path (`db_path`): pairwise subs, plus users/groups/members when `directory.backend = "sqlite"`; created on first start (0600)                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                          |
| `defaultPhoneCountry`         | `"FR"`                                | country preselected in the profile phone fields (`default_phone_country`): default dial code and national format, user-switchable via the country picker                                                                                                                                                                                                                                                                                                                                                                                                                                                                                               |
| `directory.backend`           | `ldap`                                | directory backend (`directory.backend`): `ldap` (LDAP server) or `sqlite` (users/groups/argon2 passwords in `dbPath`)                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                  |
| `directory.adminPasswordFile` | `null`                                | file holding the `admin` bootstrap password (`directory.admin_password_file`); only used when `admin` is absent; null = `admin`                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                        |
| `attrs.labels`                | `{ }`                                 | catalog of custom attributes editable on the Profile page (`attrs.labels`): each key = a label with `type` (`text` with `maxChars`, `number` with `min`/`max`, `email`, `telephone`) and `names` (display name per language, e.g. `{ fr = "Ville"; en = "City"; }`, falls back to the label); `{ }` disables the feature; in LDAP each catalogued label is a real attribute (`city: Strasbourg` — the served schema must declare the `attributetype`, e.g. `docker/portail.schema`), values for unlisted labels stay packed in `portailAttr=label=value`; in SQLite table `user_attrs`; distributed as claims to clients per `apps[].oidc.exposeAttrs` |
| `systemSensors`               | `[ "memory" "cpu" "network" "disk" ]` | blocks shown on the System page (`system_sensors`); known names: `memory`, `cpu`, `cores` (per-core detail), `load`, `pressure`, `temps`, `fans`, `network`, `tcp`, `interfaces`, `disk`, `diskio`, `raid`, `uptime`                                                                                                                                                                                                                                                                                                                                                                                                                                   |
| `oidcClientId`                | `portail`                             | portal client_id (written **enabled** into the generated `oidc.clients`)                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                               |
| `oidc.secretsDir`             | `/var/lib/portail/oidc`               | OIDC secrets directory (RSA key + `<clientId>_secret` files)                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                           |
| `oidc.logSecretEvents`        | `true`                                | backend/ExecStartPre INFO logs on OIDC secret files (generate, keep, ACL, prune)                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                       |
| `lang`                        | `en`                                  | default UI language (`fr`/`en`)                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                        |
| `ldap.url`                    | `ldap://127.0.0.1`                    | LDAP server URL (`ldap://` or `ldaps://`)                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                              |
| `ldap.baseDn`                 | `dc=example,dc=com`                   | LDAP suffix                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                            |
| `ldap.peopleOu`               | `people`                              | OU holding user entries (`LDAP_PEOPLE_OU`)                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                             |
| `ldap.groupsOu`               | `groups`                              | OU holding group entries (`LDAP_GROUPS_OU`)                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                            |
| `ldap.caCertificateFile`      | `null`                                | PEM CA bundle validating the `ldaps://` endpoint (`SSL_CERT_FILE`)                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                     |
| `ldap.adminPasswordFile`      | (required)                            | password of `cn=admin,<baseDn>`                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                        |
| `security.passwordMinLength`  | `8`                                   | minimum password length, enforced on user creation and update (`security.password_min_length`)                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                         |
| `security.loginMaxAttempts`   | `10`                                  | consecutive failed login attempts before lockout (`security.login_max_attempts`)                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                       |
| `vhost.enable`                | `true`                                | nginx vhost (frontend as `root` + `/api`, `/jwks.json`, `/.well-known/*`, `/logout` proxy)                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                             |
| `vhost.hostName`              | (with vhost)                          | nginx server_name + certificate CN                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                     |
| `vhost.forceSSL`              | `true`                                | HTTPS + 80→443 redirect                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                |
| `vhost.sslCertificate{,Key}`  | (with forceSSL)                       | TLS certificate/key provided by the host (agenix/sops)                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                 |
| `apps`                        | `[]`                                  | applications listed on the home page                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                   |

OIDC secrets: the signing key and confidential-client secrets are generated in `oidc.secretsDir` on first start (0600, never overwritten once non-empty) and never appear in the TOML. Adding or activating a client = `nixos-rebuild switch` + service restart.

### Listed applications

Each `apps` entry = a link on the home page (`name`, `url`, optional `description` and `icon`), rendered into a static `/apps.json` served by nginx. Its `oidc` sub-object, disabled by default, registers the app as an OIDC client of the portal: `enable = true` writes it enabled into the generated `oidc.clients`, with `clientId` (default: lowercased name), `redirectUris`, `scopes`, `customClaims` (static claims `{ name, value, group | notGroup }` sent according to the user's LDAP group membership), `requiredGroup`/`forbiddenGroup` (link hidden and token exchange refused 403 based on groups), `public = true` for a PKCE public client without secret, otherwise a secret generated in `oidc.secretsDir/<clientId>_secret` or an absolute path overridden by `secretFile` (agenix/sops), `user`/`group` (read ACL on the secret), and `exposeAttrs` (custom attributes from the `attrs.labels` catalog distributed as claims).

```nix
services.portail.apps = [
  { name = "Vaultwarden"; url = "https://vault.portail.example.com"; description = "Mots de passe"; }
  {
    name = "Nextcloud";
    url = "https://cloud.portail.example.com";
    oidc = {
      enable = true;
      clientId = "nextcloud";
      requiredGroup = "nextcloud-users";
      redirectUris = [ "https://cloud.portail.example.com/apps/oauth2/redirect_ui" ];
      customClaims = [
        { name = "role"; value = "user"; }
        { name = "role"; value = "admin"; group = "admins"; }
      ];
    };
  }
];
```

Before `nixos-rebuild switch`:

```bash
nix build .#portail-backend .#portail-frontend
nix flake check
```

## Development

```bash
devenv up                      # starts everything: backend 3000 (incl. OIDC), frontend 5173, LDAP 3890/3891, oauth debugger 3030
devenv down                    # stop
openfrontend                   # opens the portal http://127.0.0.1:5173
resetAdminLdap                 # reseed dev LDAP accounts (admin/admin, user/user)
resetAdminSqlite               # remove the dev SQLite db — backend recreates admin on restart
just test                      # backend (cargo test --features test-api) + frontend (vitest)
treefmt                        # formatting (nixfmt + rustfmt + prettier)
```

### Running the binary standalone

`portail-backend --config <path>` — no configuration environment variables; relative paths in the TOML resolve against the config file's directory. Minimal (sqlite backend):

```toml
app_url = "https://portail.example.com"
bind_addr = "127.0.0.1:3000"
db_path = "portail.db"

[directory]
backend = "sqlite"

[oidc]
issuer_url = "https://portail.example.com"
client_id = "portail"
redirect_uri = "https://portail.example.com/callback"
secrets_dir = "secrets"
clients = [
  { id = "portail", secret_file = "portail_secret", enabled = true, redirect_uris = [
    "https://portail.example.com/callback",
  ], scopes = [ "openid", "profile", "email", "groups" ] },
]
```

With `backend = "ldap"`, add `[ldap] url` + `base_dn` (required). Optional keys and defaults: rule `.opencode/rules/06-env.md`; full examples: `config.test.toml` and `docker/config.toml`.
