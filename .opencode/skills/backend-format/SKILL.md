---
name: backend-format
description: Enforce Rust backend formatting (rustfmt via treefmt) + clippy lints — use when editing backend/src, Cargo.toml, or any .rs file.
---

# backend-format

## Rule — `rustfmt`/`clippy` via `devenv`, never bare `cargo fmt`

```bash
devenv shell -- cargo fmt            # obligatoire après toute modif Rust
devenv shell -- treefmt --fail-on-change   # verif globale avant git add
devenv shell -- cargo clippy -- -D warnings   # 0 warnings allowed
devenv shell -- cargo clippy --fix --allow-dirty   # auto-fix
```

Never run bare `cargo fmt`/`rustfmt`/`clippy` outside devenv shell (rust 1.98 toolchain).

### Enforced

- `rustfmt` defaults via treefmt (no custom `rustfmt.toml`, YAGNI) — imports triés, trailing commas, 100 chars. Pas de `#[rustfmt::skip]` sans justification.
- `clippy`: pas de `unwrap()`/`expect()` hors `main.rs:6` boot ou tests; pas de `allow(clippy::*)` sans `// ponytail: ...` + justification.

### Verification

```bash
devenv shell -- cargo fmt -- --check
devenv shell -- treefmt --fail-on-change
devenv shell -- cargo clippy -- -D warnings
devenv shell -- cargo test
```
