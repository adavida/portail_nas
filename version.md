# Portail — v0.99.0

**Français** | [English](#english)

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

- Personal portal to manage the users of a NixOS NAS and display a page gathering its applications.
- Users/groups directory on LDAP or SQLite (chosen by `config.toml`), argon2-hashed passwords, CRUD users + groups + memberships through a protected REST API.
- Backend = its own OIDC provider: discovery, RS256 JWKS (key generated on first startup), authorize + login + token with PKCE, userinfo, logout; clients declared in `config.toml`.
- Bootstrap of the `admin` account and the `admin` group at startup if missing.
- System metrics on `GET /api/system`: CPU, RAM, disk, disk I/O, RAID, temperatures, fans, network — auto-scaled units (`"16 GiB"`, `"45.5 °C"`).
- Svelte 5 frontend: portal logged in via OIDC (it is its own client), users/groups management, monitoring page, light/dark theme, fr/en UI.
- NixOS deployment via flake: backend/frontend packages + `services.portail` module (systemd service + nginx vhost, `/api` proxy, `forceSSL`).
- Provided NixOS configuration: complete host `flake.nix` example (flake imports, `services.portail.*`, OpenLDAP, secrets), validated by `nix flake check`.
- devenv development environment: `devenv up` = backend + frontend + 2 slapd (dev/test) + test watchers + oauth2-debugger.
- Integration suites run on both backends (ldap + sqlite), at least 1 test per endpoint.
