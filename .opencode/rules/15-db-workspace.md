# 15 — DB Workspace — `portail-db` est le seul SQL du dépôt

> Règles liées: `08-struct-folder.md` (layout 1 fonction = 1 fichier), `06-env.md` (`db_path`), `03-backend-frontend.md` (graphe des crates).

## Objectif

La persistance SQLite vit dans **un crate workspace dédié**, `backend/db` (`portail-db`): c'est le seul endroit du dépôt qui dépend de `rusqlite`, connaît du SQL et le fichier `db_path`. Les crates métiers (`oidc`, futurs autres) appellent des **fonctions** de `portail_db` et ne voient jamais une chaîne SQL hors de leurs fixtures de test.

## Layout (`backend/db/src/`)

```text
backend/db/migrations/000N_<objet>.sql  # DDL versionné, un fichier par évolution de schéma
backend/db/src/lib.rs              # pub mod connect/path/subjects + pub use connect::connect + #[cfg(test)] test_helpers
backend/db/src/connect.rs          # pub fn connect(config) -> Connection + init_schema (moteur de migrations: table `migrations`, 0600, create_dir_all)
backend/db/src/path.rs             # pub fn portal_db_path(config) -> PathBuf (lit config.db_path)
backend/db/src/subjects/mod.rs     # pub mod find/insert + pub use
backend/db/src/subjects/<fn>.rs    # 1 pub fn (<fn>, conn: &Connection, …) -> Result<…, String> + sa requête + #[cfg(test)] en mémoire
```

- Une nouvelle table = un nouveau module d'objet (`backend/db/src/<objet>/`) pour les requêtes, et son DDL dans un **nouveau fichier** `backend/db/migrations/000N_<objet>.sql` référencé dans la liste `MIGRATIONS` (`include_str!`). `init_schema` est un **moteur** (crée la table `migrations`, rejoue en transaction chaque fichier absent de cette table) — il ne contient plus jamais de SQL métier.
- Une migration appliquée n'est **jamais éditée** : on ajoute `000N+1` (ALTER…). Les db existantes montent au dernier schéma au prochain `connect`.
- `rusqlite` n'est dépendance QUE de `backend/db/Cargo.toml`.
- Les fonctions prennent `&Connection` en paramètre et renvoient des types du domaine (`Option<String>`, `()`, structs) — jamais `rusqlite::Error` dehors (converti en `String`).
- Un crate métier: `let conn = portail_db::connect(config)?;` puis les fonctions `portail_db::<objet>::<fn>(&conn, …)`. Il ne nomme jamais le type `Connection` (juste le passe), n'ouvre/autorise/dimensionne aucun fichier.
- Tests: les requêtes se testent sur **db en mémoire** (`Connection::open_in_memory()` + `init_schema`) — jamais de fichier ni de LDAP. Ce choix survit à un futur passage PostgreSQL tant que le SQL de `backend/db` reste du **sous-ensemble commun** aux moteurs (`CREATE TABLE IF NOT EXISTS`, `TEXT`, `SELECT ?`, `INSERT`). SQL **spécifique** à un engine (`RETURNING`, `ON CONFLICT DO UPDATE`, `JSONB`…) ⇒ interdiction de ne tester que sur sqlite in-memory: ajouter alors un process `postgres-test` dans devenv (port isolé, pattern `openldap-test:3891` — règle 02) + tests d'intégration dédiés. On ne code pas cette infra avant que ce SQL existe (YAGNI).
- Placeholders: `?`/`?n` (sqlite seul) aujourd'hui; si deux engines coexistent un jour, la conversion des placeholders reste confinée dans `backend/db`.
- `Config::global()` en test ⇒ helper `set_test_config` propre au crate (règle `10-test-helpers.md`, duplication assumée).
- Fichier db créé à la volée (0600, parent `create_dir_all`) — jamais de db commitée dans le dépôt.

## Interdits

- `use rusqlite` ou chaîne SQL (`SELECT`/`INSERT`/`UPDATE`/`DELETE FROM`/`CREATE TABLE`) dans `backend/{core,directory,oidc,server}/src/**` hors `#[cfg(test)]` — le SQL vit seulement dans `backend/db/**` (`src` pour les requêtes, `migrations/` pour le DDL).
- Éditer un fichier `migrations/000N_*.sql` déjà appliqué : uniquement ajouter le suivant.
- Ajouter `rusqlite` à un autre `Cargo.toml` de crate.
- `Connection::open`/`open_in_memory` dans un crate métier (ouvertures = `connect.rs` en prod, tests in-memory ailleurs dans db).
- Exposer un chemin de db ou `Connection` comme type dans une signature publique de crate.
- Écrire dans la db en dehors de `portail-db` (le script `scripts/migrate-authelia-subs.sh` est l'exception opérationnelle, hors dépôt Rust).
- Tester du SQL spécifique à un engine (PG-only) contre du sqlite in-memory seul.

## Vérification

```bash
rg -n "rusqlite" backend/*/Cargo.toml            # seul backend/db/Cargo.toml
rg -n "rusqlite|Connection::open|SELECT |INSERT |UPDATE |DELETE FROM|CREATE TABLE" backend/*/src --glob '*.rs' -g '!backend/db/**' | grep -v "cfg(test)"   # vide
devenv shell -- cargo clippy --workspace --all-targets --features test-api -- -D warnings
devenv shell -- cargo test --features test-api
```
