---
name: devenv-format
description: Enforce devenv.nix formatting via treefmt/nixfmt — alphabetical top-level attrs, grouped env/languages, dotted singletons, no comments. Use when editing devenv.nix, devenv.yaml, or adding packages/processes.
---

# devenv-format

## Rule — `devenv.nix` must be formatted by `treefmt`, never by hand

Executable source of truth: `devenv.nix:154` + `devenv.yaml:5`.

### How to format

```bash
devenv shell -- treefmt          # formats 15 files, 3 changed typical
# also runs automatically on enterShell via tasks."devenv:treefmt:run"
```

Never run `nixfmt`, `rustfmt`, `prettier`, `alejandra` directly.

### What `nixfmt` enforces (will fail CI if violated)

1. **Top-level alphabetical** inside `{ ... }:`  
   `env` < `enterShell` < `enterTest` < `languages` < `packages` < `processes` < `scripts` < `tasks` < `treefmt`
2. **Grouped vs dotted**
   - `>1` prop → grouped: `env = { LDAP_BASE_DN; LDAP_URL; }`, `languages.javascript = { enable; package; }`, `tasks = { "x".exec; }`, `processes = { backend.exec; ... }`
   - `1` prop → dotted singleton: `scripts.hello.exec`, `languages.rust.enable`, `processes.openldap.exec`  
     Do not write `scripts = { hello = { exec; } }` or `env.LDAP_URL = ...` (grouped form required).
3. **Inside `processes` alphabetical**: `backend` < `backend-test` < `frontend` < `frontend-test` < `openldap` < `openldap-test` < `vscode`
4. **Inside `packages` alphabetical**: `cargo-watch` < `codiumWithExt` < `git` < `just` < `openldap`
5. **No comments** in `devenv.nix` (enforced by `nixfmt` stripping).
6. **Config**: `treefmt.config.projectRootFile = "devenv.nix"`; `programs.nixfmt/rustfmt/prettier.enable = true` via `treefmt-nix` input (`devenv.yaml:5` follows `nixpkgs`).

### When adding a package/process

- Insert at the alphabetically correct position, keep grouping rule, then run `treefmt` immediately.
- After editing `devenv.yaml` (adding inputs), run `devenv shell -- treefmt` to reformat `devenv.lock`.

### Verification

```bash
devenv shell -- treefmt --fail-on-change  # CI check — exits 1 if not formatted
```
