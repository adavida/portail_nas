---
name: backend-format
description: Enforce Rust backend formatting (rustfmt via treefmt) + clippy lints — use when editing backend/src, Cargo.toml, or any .rs file.
---

# backend-format

## Rule — `rustfmt`/`clippy` via `devenv`, never bare `cargo fmt`

Executable source of truth: `devenv.nix:127` + `backend/Cargo.toml:2`.

### How to format

```bash
devenv shell -- cargo fmt        # obligatoire après toute modif Rust (*.rs, Cargo.toml)
devenv shell -- treefmt          # rustfmt via treefmt-nix, also on enterShell (nix+rust+prettier)
devenv shell -- cargo clippy -- -D warnings   # 0 warnings allowed
devenv shell -- cargo clippy --fix --allow-dirty  # auto-fix when possible
```

Never run `cargo fmt`/`rustfmt`/`clippy` outside `devenv shell` (rust 1.98 toolchain).

### Après toute modification Rust — obligatoire

Après édition de `backend/src/**/*.rs` ou `backend/Cargo.toml`, lancer systématiquement en fin de modif:

```bash
devenv shell -- cargo fmt
```

puis `devenv shell -- treefmt` (vérif globale) avant `git add`. `treefmt` seul ne suffit pas comme preuve `cargo fmt`.

### What `rustfmt` enforces (treefmt)

- `rustfmt` defaults via `treefmt.config.programs.rustfmt.enable` — no custom `rustfmt.toml` (YAGNI).
- No manual `#[rustfmt::skip]` without comment justificatif.
- Imports triés automatiquement, trailing commas, 100 chars.

### What `clippy` enforces (CI fails if `-D warnings`)

1. `clippy::pedantic` off, `warnings` denied — `cargo clippy -- -D warnings` doit passer à 0.
2. Pas de `unwrap()`/`expect()` hors `main.rs:6` boot ou tests — préférer `?` + `axum::Json` ou `anyhow`.
3. Pas de `allow(clippy::*)` sans `// ponytail: ...` + justification.

### Verification

```bash
devenv shell -- cargo fmt -- --check   # doit être clean après modif Rust
devenv shell -- treefmt --fail-on-change
devenv shell -- cargo clippy -- -D warnings
devenv shell -- cargo test
```
