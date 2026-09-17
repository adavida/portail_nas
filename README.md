# Portail

Stack : Rust Axum (`backend`) + React Vite (`frontend`) + OpenLDAP dev/test.

Toutes les commandes passent par `devenv shell -- <cmd>` (cargo/node fournis par devenv).

## Démarrage

```bash
devenv up            # lance tout : backend 3000, frontend 5173, LDAP 3890/3891
devenv shell -- open # ouvre http://localhost:5173/ dans le navigateur
```

Services :

- backend `http://localhost:3000/api/health` (`backend/src/lib.rs:6`)
- frontend `http://localhost:5173` (proxy `/api` → `3000`, `frontend/vite.config.ts:8`)
- LDAP dev `ldap://127.0.0.1:3890/dc=dev,dc=example,dc=com`
- LDAP test `ldap://127.0.0.1:3891/dc=test,dc=example,dc=com`

Arrêt : `devenv down`

## Commandes principales

### Dev

```bash
devenv shell -- cargo run -p portail-backend          # backend seul
devenv shell -- npm --prefix frontend run dev         # frontend seul (Vite)
just dev                                              # aide mémoire
```

### Tests

```bash
devenv shell -- cargo test                            # backend
devenv shell -- cargo test health_returns_ok -- --nocapture  # un test
devenv shell -- npm --prefix frontend test -- --run   # frontend (vitest)
just test                                             # backend + frontend
```

`devenv up` lance aussi `backend-test` et `frontend-test` en watch.

### Build & format

```bash
devenv shell -- cargo build
devenv shell -- npm --prefix frontend run build       # → frontend/dist/
devenv shell -- treefmt                               # nixfmt + rustfmt + prettier (auto au enterShell)
```

### Backend

Structure `backend/src/` : `main.rs` (bootstrap) + `lib.rs` (Router) + `domain/` (métier pur) + `controllers/` (orchestration LDAP) + `http/` (handlers axum).

```bash
curl http://localhost:3000/api/health
curl http://localhost:5173/api/health  # via proxy Vite
```

### Frontend

```bash
devenv shell -- bash -c 'cd frontend && npm install'  # première install
```

## Layout

```
devenv.nix / devenv.yaml
backend/Cargo.toml → backend/src/{main,lib}.rs, domain/, http/, controllers/
frontend/{vite.config.ts,package.json,src/}
```
