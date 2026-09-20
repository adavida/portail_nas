---
name: devenv-format
description: Enforce devenv.nix formatting via treefmt/nixfmt — alphabetical top-level attrs, grouped env/languages, dotted singletons, no comments. Use when editing devenv.nix, devenv.yaml, or adding packages/processes.
---

# devenv-format

## Rule — `devenv.nix` formatted by `treefmt`, never by hand

```bash
devenv shell -- treefmt --fail-on-change   # aussi auto sur enterShell
```

Never run bare `nixfmt`/`rustfmt`/`prettier`/`alejandra` — toujours via devenv.

### nixfmt rules (CI failed if violated)

1. Top-level `env < enterShell < enterTest < languages < packages < processes < scripts < tasks < treefmt` — ordre alphabétique dans `{ ... }:`.
2. Grouped vs dotted — `>1` prop → grouped (`env = {LDAP_BASE_DN; LDAP_URL;}`), `1` prop → dotted singleton (`scripts.hello.exec`, `languages.rust.enable`).
3. Inside `processes` & `packages` alphabetical (`backend < backend-test < frontend < frontend-test < openldap < openldap-test < vscode`).
4. No comments in `devenv.nix`.
5. `treefmt.config.projectRootFile = "devenv.nix"`; `programs.nixfmt/rustfmt/prettier.enable = true` vue `treefmt-nix` (`devenv.yaml:5`).

### Adding package/process

- Insert position alphabetique + grouped rule + `devenv shell -- treefmt` immédiatement.
- Edit `devenv.yaml` (inputs) → `devenv shell -- treefmt` pour reformatter `devenv.lock`.

### Verification

```bash
devenv shell -- treefmt --fail-on-change   # exits 1 si non formatted
```
