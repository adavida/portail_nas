# 01 — devenv.nix — Formatage & conventions (bloquant)

> Skill de référence: `.opencode/skills/devenv-format/SKILL.md` (trigger `devenv.nix`, `devenv.yaml`).

## Top-level trié alphabétiquement

`env` < `enterShell` < `enterTest` < `languages` < `packages` < `processes` < `scripts` < `tasks` < `treefmt`

## Groupé vs pointé

- `>1` prop → groupé: `env = { LDAP_* }`, `languages.javascript = { enable; package; }`, `tasks = { "x".exec; }`, `processes = { backend.exec; ... }`
- `1` prop → pointé singleton: `scripts.hello.exec`, `languages.rust.enable`, `processes.openldap.exec`
- Ne pas écrire `scripts = { hello = { exec; } }` ni `env.LDAP_URL = ...`

## Ordres internes alpha

- `processes`: `backend` < `backend-test` < `frontend` < `frontend-test` < `openldap` < `openldap-test` < `vscode`
- `packages`: `cargo-watch` < `codiumWithExt` < `git` < `just` < `openldap`

## Règles formatage

- Pas de commentaires dans `devenv.nix`.
- Jamais `nixfmt`/`rustfmt`/`prettier` à la main → `devenv shell -- treefmt` (auto sur `enterShell` via `tasks."devenv:treefmt:run"` after `devenv:files` before `enterShell`).
- Config: `treefmt.config.projectRootFile = "devenv.nix"`, `programs.nixfmt/rustfmt/prettier.enable = true` via `treefmt-nix` input (`devenv.yaml:5` follows `nixpkgs`).
- Après ajout package/process: insérer à la position alpha correcte, respecter groupé/pointé, puis `treefmt` immédiat.
- Vérif CI: `devenv shell -- treefmt --fail-on-change` (exit 1 si non formaté).
