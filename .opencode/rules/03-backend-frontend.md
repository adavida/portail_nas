# 03 — Backend & Frontend

> Skills: `backend-format` (trigger `*.rs`), `backend-structure` (trigger `http/`, `controllers/`, `domain/`), `frontend-format` + `frontend-structure` (trigger `frontend/src`, `*.svelte`).

## Backend workspace — 7 crates (Axum 0.8, Tokio full)

- Workspace `Cargo.toml` `members = ["backend/core", "backend/db", "backend/directory", "backend/metrics", "backend/oidc", "backend/server", "backend/units"]`.
- `portail-core` (`backend/core`): `config/` (TOML, `Config`/`ConfigError`) + `error` (AppError + `IntoResponse`) — dépendance de tous.
- `portail-db` (`backend/db`): workspace SQLite isolé — `connect.rs` (ouverture + schéma + 0600), `path.rs`, `subjects/{find,insert}` — seul crate avec `rusqlite`/SQL (règle `15-db-workspace.md`).
- `portail-units` (`backend/units`): grandeurs entières réutilisables, **1 unité = 1 fichier** (règle `16-units.md`) — stockage toujours en unité de base (octet, milliseconde), sérialisation en chaîne auto-échelonnée portant l'unité (`"16 GiB"`, `"1 s"`, `"45.5 °C"`) — `bytes.rs` (`Bytes`), `seconds.rs` (`Seconds`), `celsius.rs` (`Celsius`, milli-degrés signés), `hundredths.rs` (`Hundredths(u32)` fix-point ×100), `percent.rs` (`Percent(u8)` clamp 100), `bytes_per_second.rs` / `count_rate.rs` / `bits_per_second.rs` / `rpm.rs` (taux et capteurs, sérialisés par `impl Serialize` propre) — delta saturant, aucune dépendance métier, seul `serde`, consommé par `metrics`.
- `portail-directory` (`backend/directory`): `domain/{users,groups}`, `repository/ldap`, `controllers/{users,groups}`, `http/{users,groups}` — routers **sans middleware**, `ldap3`/`native-tls` isolés ici.
- `portail-oidc` (`backend/oidc`): `auth/` (dont middleware `require_auth`/`require_admin`), `domain/oidc`, `controllers/oidc`, `http/{oidc,auth}` — dépend directory (login authentifie via LDAP) et db (subs).
- `portail-metrics` (`backend/metrics`): 1 struct sérialisé = 1 fichier (`cpu.rs`, `memory.rs`, `net.rs`, `interface.rs`, `tcp.rs`, `disk.rs`, `disk_io.rs`, `raid.rs`, `temp.rs`, `fan.rs`, `hwmon.rs`, `system.rs` = struct `System` + `snapshot()`) ; grandeurs **entières** via le crate `portail-units` (`backend/units`: `Bytes(u64)` + helpers de conversion et sérialiseurs serde, taux et deltas en arithmétique entière saturante — zéro `uom` ni `f64` dans les structs; les tailles/durées/taux sont des chaînes auto-échelonnées portant l'unité (`ram_total: \"16 GiB\"`, `uptime: \"1 s\"`, `celsius: \"45.5 °C\"`), les comptes restent des nombres plats) ; `protocol.rs` = index de colonnes procfs/sysfs + fenêtre d'échantillonnage (catégorie 2 règle 14, jamais configurables) — dépend core (AppError + config) et units.
- `portail-server` (`backend/server`): assemblage `router()` (`lib.rs` `pub use http::router as app`), health, handlers fins (`http/system.rs` = appel `portail_metrics::snapshot`), `main.rs` (bin `portail-backend` via `[[bin]]`), `tests/` intégration — dépend core, directory, metrics et oidc (db par transitif).
- Graphe acyclique imposée: server → {oidc → {directory → core, db → core}, metrics → {core, units}}. Aucun crate métier ne câble de middleware; l'assemblage est seul dans server.

## Tests API — `backend/server/tests/`

- **1 endpoint = au moins 1 test d'intégration** — nouvelle route sans test = pas finie.
- 1 fichier = 1 ressource (`health.rs`, `users.rs`); nouvelle ressource ⇒ nouveau fichier `tests/<ressource>.rs`.
- Router via `portail_server::app()` + `tower::ServiceExt::oneshot` (pas de vrai serveur); helper partagé `send(method, uri, Option<json>) -> (StatusCode, Value)` dans `tests/common/send.rs` (règle `10-test-helpers.md`).
- LDAP test uniquement (`config.test.toml`, 3891) — jamais `dev` (3890).
- Auto-clean: `mod common;` + `tests/common/purge.rs` (`#[ctor] fn purge_test_ldap`, dev-dep `ctor`) purge `ou=people` de la base test au démarrage; LDAP down ⇒ no-op silencieux.
- Données uniques: uid timestamp (`TestUser::new`) — prefix `^[a-z0-9._-]+$` (pénètre dans l'URL). Pas de cleanup par test — la purge `ctor` suffit (sauf tests dont le DELETE est le sujet).
- Corps 204 vide ⇒ parse JSON avec fallback `unwrap_or(json!({}))`. LDAP inatteignable ⇒ skip via guard (`500`/`Err(AppError::Ldap)`), CI sans LDAP reste verte.
- **Routes test-only** (ex: `POST /api/users/:uid/authenticate`): feature flag `test-api` déclaré dans `backend/directory/Cargo.toml` + `backend/oidc/Cargo.toml` `[features]`, relayé par `backend/server/Cargo.toml` (`test-api = ["portail-directory/test-api", "portail-oidc/test-api"]`), `#[cfg(feature = "test-api")]` dans `users_router`/`protected_auth_router` — absentes du build prod. **Piège**: `#[cfg(test)]` est ignoré dans la lib quand `backend/server/tests/*.rs` la compilent comme dépendance — dans `tests/`, seul le feature flag filtre (`#[cfg(feature = "test-api")]` au-dessus de `#[tokio::test]`); `#[cfg(test)]` réservé aux unit tests collocalisés.
- Verif: `devenv shell -- cargo test --features test-api -- --nocapture` (unit + intégration).

## Frontend `portail-frontend` (Vite + Svelte 5 runes + TS)

- `package.json:6` scripts `dev`/`build`/`test` (`vitest run`)/`test:watch`/`check` (`svelte-check`). `vite.config.ts` jsdom + `setupFiles=./src/test/setup.ts` + plugin `svelteTesting()` sous `VITEST`; preprocess dans `svelte.config.js` (`vitePreprocess`).
- proxy `/api → backend`, port `5173`; `pages/Users.svelte` fetch `/api/users`, `App.svelte` glue (pas de routeur: `window.location.pathname === "/callback"`).
- Tests collocalisés `@testing-library/svelte` + `jsdom` (`<X>.test.ts` à côté de `<X>.svelte`, `App.test.ts`); toast store resetté dans `test/setup.ts`.
- Miroir frontend des unités: `frontend/src/units/<unité>.ts` (1 classe = le type `portail-units` homonyme, nom de fichier snake_case) — construites depuis la chaîne JSON auto-échelonnée, `.value` = grandeur de base normalisée (octets, bits/s, secondes, degrés, centièmes, %), `.toString()` = texte du fil; `api/system.ts` mappe le JSON wire en objets typés (`toSystem`).
- Commandes: `devenv shell -- npm --prefix frontend test -- --run` (single: `-t "renders title"`); prérequis `devenv shell -- bash -c 'cd frontend && npm install'`.

## Lien backend ↔ frontend

- Le proxy `vite.config.ts` doit matcher `main.rs` `bind 0.0.0.0:3000`. Changer l'un ⇒ changer l'autre + `justfile:5`.
