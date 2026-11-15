# Portail — v0.99.2

**Français** | [English](#english)

## v0.99.2 — depuis v0.99.1

- Nouvelle page **Mon profil** : chaque utilisateur modifie lui-même son nom, son email et son mot de passe (confirmation + règles affichées), sans passer par un admin.
- **Champs personnalisés** : catalogue de labels configurable (Nix / `config.toml`) avec types `text`/`number`/`email`/`telephone` et noms affichés localisés (`names = { fr; en; }`) ; stockés en LDAP comme vrais attributs (`city: Strasbourg`, schéma à déclarer ; legacy et hors-catalogue dans `portailAttr`) ou en SQLite (`user_attrs`) ; champ vide = suppression.
- **Téléphone** : champ unique avec sélecteur de pays (drapeau + indicatif, via libphonenumber-js), format national sans `0`, stocké en E.164 sans espaces ; pays par défaut configurable (`default_phone_country`, option NixOS `defaultPhoneCountry`).
- **Distribution OIDC des attributs** : option client `expose_attrs` — claim par label, renommage à la volée (`{ label = "city"; claim = "ville"; }`) ou bundle JSON unique avec `"*"` ; `userinfo` servi frais.
- **Nouveau design** : pages regroupées à onglets — Annuaire (Utilisateurs | Groupes), Supervision (Système | Diagnostics), profil avec avatar et groupes ; composants `Tabs` et `LabeledInput` responsives.
- **Stack Docker de test** : `docker compose up` construit les images lui-même et persiste toutes les données en volumes nommés (`ldap-data`/`ldap-config`, `backend-data` SQLite, `oidc-secrets`) ; deux modes d'annuaire documentés (`--profile ldap` ou sqlite).
- Module NixOS : nouvelles options `attrs.labels`, `defaultPhoneCountry`, `apps[].oidc.exposeAttrs` ; README et schémas LDAP d'exemple alignés ; suites d'intégration étendues (profil, attributs, docker) sur les deux backends.

## v0.99.0

- Portail perso pour gérer les utilisateurs d'un NAS NixOS et afficher une page regroupant ses applications.
- Annuaire utilisateurs/groupes en LDAP ou SQLite (choisi par `config.toml`), mots de passe hashés en argon2, CRUD users + groups + memberships via API REST protégée.
- Backend = son propre fournisseur OIDC : discovery, JWKS RS256 (clé générée au premier démarrage), authorize + login + token avec PKCE, userinfo, logout ; clients déclarés dans `config.toml`.
- Bootstrap du compte `admin` et du groupe `admin` au démarrage s'ils manquent.
- Métriques système sur `GET /api/system` : CPU, RAM, disque, disk I/O, RAID, températures, ventilateurs, réseau — unités auto-échelonnées (`"16 GiB"`, `"45.5 °C"`).
- Frontend Svelte 5 : portail connecté en OIDC (client de lui-même), gestion des utilisateurs/groupes, page de monitoring, thème clair/sombre, interface fr/en.
- Déploiement NixOS par flake : packages backend/frontend + module `services.portail` (service systemd + vhost nginx, proxy `/api`, `forceSSL`).
- Configuration NixOS fournie : exemple `flake.nix` complet sur l'hôte (imports du flake, `services.portail.*`, OpenLDAP, secrets), validé par `nix flake check`.
- Environnement de dev devenv : `devenv up` = backend + frontend + 2 slapd (dev/test) + watchers de tests + oauth2-debugger.
- Suites d'intégration exécutées sur les deux backends (ldap + sqlite), au moins 1 test par endpoint.

## English

## v0.99.2 — since v0.99.1

- New **My profile** page: every user edits their own name, email and password (confirmation + rules shown live), no admin round-trip.
- **Custom attributes**: label catalog configured in Nix / `config.toml` with `text`/`number`/`email`/`telephone` types and localized display names (`names = { fr; en; }`); stored in LDAP as real attributes (`city: Strasbourg`, schema must declare them; legacy and unlisted values packed in `portailAttr`) or in SQLite (`user_attrs`); empty field = deletion.
- **Telephone**: single field with a country picker (flag + dial code, via libphonenumber-js), national format without leading `0`, stored as E.164 without spaces; default country configurable (`default_phone_country`, NixOS option `defaultPhoneCountry`).
- **OIDC attribute distribution**: per-client `expose_attrs` — one claim per label, on-the-fly renaming (`{ label = "city"; claim = "ville"; }`) or a single JSON bundle with `"*"`; `userinfo` served fresh.
- **New design**: pages regrouped under tabs — Directory (Users | Groups), Monitoring (System | Diagnostics), profile with avatar and groups; responsive `Tabs` and `LabeledInput` components.
- **Docker test stack**: `docker compose up` builds the images itself and persists every data in named volumes (`ldap-data`/`ldap-config`, `backend-data` SQLite, `oidc-secrets`); both directory backends documented (`--profile ldap` or sqlite).
- NixOS module: new options `attrs.labels`, `defaultPhoneCountry`, `apps[].oidc.exposeAttrs`; README and example LDAP schemas aligned; integration suites extended (profile, attributes, docker) on both backends.

## v0.99.0

- Personal portal to manage the users of a NixOS NAS and display a page gathering its applications.
- Users/groups directory on LDAP or SQLite (chosen by `config.toml`), argon2-hashed passwords, CRUD users + groups + memberships through a protected REST API.
- Backend = its own OIDC provider: discovery, JWKS RS256 (key generated on first startup), authorize + login + token with PKCE, userinfo, logout; clients declared in `config.toml`.
- Bootstrap of the `admin` account and the `admin` group at startup if missing.
- System metrics on `GET /api/system`: CPU, RAM, disk, disk I/O, RAID, temperatures, fans, network — auto-scaled units (`"16 GiB"`, `"45.5 °C"`).
- Svelte 5 frontend: portal logged in via OIDC (it is its own client), users/groups management, monitoring page, light/dark theme, fr/en UI.
- NixOS deployment via flake: backend/frontend packages + `services.portail` module (systemd service + nginx vhost, `/api` proxy, `forceSSL`).
- Provided NixOS configuration: complete host `flake.nix` example (flake imports, `services.portail.*`, OpenLDAP, secrets), validated by `nix flake check`.
- devenv development environment: `devenv up` = backend + frontend + 2 slapd (dev/test) + test watchers + oauth2-debugger.
- Integration suites run on both backends (ldap + sqlite), at least 1 test per endpoint.
