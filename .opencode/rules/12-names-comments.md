# 12 — Names & Comments — Expressive English Names, Zero Comments

> Règles liées: `07-errors.md` (messages runtime en anglais), `test-aaa` (noms de tests), `08-struct-folder.md` (layout fichiers).

## Objectif

Un nom de variable/fonction/paramètre/struct/VO s'exprime en **anglais, complet et descriptif** — `redirect_uri`, `generate_signing_key`, `ensure_user_group`, jamais `x`, `tmp`, `do_it`, `ur`. Le nom et le type doivent tout dire: les **commentaires sont interdits**, seule exception le marqueur `// shortcut:`.

## Noms

- Anglais partout: identifiants, paramètres, noms de tests (`concurrent_creates_both_join_the_default_group`), messages d'assertion.
- Précis sur le domaine: `oidc_key_bits` pas `bits`; `generate_signing_key` pas `gen_key`; booléens en `is_/has_/allows_`.
- Un nom trop court pour être clair est un mauvais nom — pas une raison pour écrire un commentaire.
- Abréviations acceptées seulement si déjà universelles dans le code: `uid`, `gid`, `dn`, `ou`, `vo`, `pkce`, `jwks`, `oidc`, `pem`.

## Commentaires — politique zéro

- Interdits partout dans `backend/*/src`, `backend/server/tests/`, `frontend/src`: `//`, `///`, `//!` — y compris doc publique, WHY, messages de sécurité, marqueurs de step de test. La sémantique se lit dans le nom, le type, et les tests.
- **Seule exception**: le marqueur de raccourci architecture — `// shortcut: <la limite>, <quand upgrader>` — une ligne, anglais. C'est l'inventaire de dette technique (audit `ponytail-debt`); un raccourci sans marqueur est un bug caché.
- Non concernés (directives compiler, pas des commentaires): `// @ts-expect-error`, `/// <reference … />`, `<!-- svelte-ignore … -->`, `#[allow(...)]`.

## Interdits

- Tout commentaire hors `// shortcut:` (répète le code, reformule une signature, explique un WHY non codable).
- Un raccourci avec limite connue sans marqueur `// shortcut:`.
- Commentaire à la place du nom: `// the redirect target` sur `let x` → renommer `redirect_target`.
- `// TODO` sans propriétaire ni contexte → ticket.

## Vérification

```bash
# zéro commentaire hors shortcut: et directives compiler
rg -n "^\s*(//|///)" backend/*/src backend/server/tests frontend/src --glob '*.rs' --glob '*.ts' --glob '*.svelte' \
  | grep -v "shortcut:\|@ts-\|<reference\|svelte-ignore"   # doit être vide
rg -n "//.*$" backend/*/src frontend/src --glob '*.{rs,ts,svelte}' | grep -v "http://\|shortcut:\|@ts-\|<reference" # inline: vide
# pas de nom court hors boucle/fermetures
rg -n "let (x|tmp|val|res|obj|foo|bar)\b" backend/*/src frontend/src
devenv shell -- cargo clippy --all-targets -- -D warnings
```
