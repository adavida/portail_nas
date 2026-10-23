# 12 — Names & Comments — Expressive English Names, No Redundant Comments

> Règles liées: `07-errors.md` (messages runtime en anglais — inchangé), `test-aaa` (noms de tests), `08-struct-folder.md` (noms de fichiers).

## Objectif

Un nom de variable/fonction/paramètre/struct/VO s'exprime en **anglais, complet et descriptif** — `redirect_uri`, `generate_signing_key`, `ensure_user_group`, jamais `x`, `tmp`, `do_it`, `ur`. Quand le nom est explicite, le commentaire qui répète le code est **superflu → supprimé**. Un commentaire ne survit que s'il dit ce que le nom et le type ne peuvent pas dire.

## Noms

- Anglais partout: identifiants, paramètres, noms de tests (`concurrent_creates_both_join_the_default_group`), messages d'assertion.
- Précis sur le domaine: `oidc_key_bits` pas `bits`; `generate_signing_key` pas `gen_key`; booléens en `is_/has_/allows_`.
- Un nom trop court pour être clair est un mauvais nom, pas une raison pour ajouter un commentaire.
- Abréviations acceptées seulement si déjà universelles dans le code: `uid`, `gid`, `dn`, `ou`, `vo`, `pkce`, `jwks`, `oidc`, `pem`.

## Commentaires autorisés

- `///` doc: contrat, invariant, sécurité, effet de bord non évident (`O_EXCL` = jamais écrasé, `Secure` si https…).
- `//` WHY: contournement, piège, plafond délibéré (`// ponytail: …`), exigence externe (SameSite, schéma LDAP, behaviour jsdom).
- Marqueurs de step dans les tests longs (`// 1. authorize -> login`) si le test enchaîne >3 phases HTTP.

## Interdits

- `//` qui répète ce que la ligne fait: `// validate env`, `// parse json`, `// cleanup`, `// optional vars: defaults`, `// 3 lines above en français`.
- `///` qui reformule la signature: `/// Creates a user` sur `create_user` sans rien ajouter.
- Commentaire à la place du nom: `// the redirect target` sur `let x` → renommer `redirect_target`, supprimer le commentaire.
- `// TODO` sans propriétaire ni contexte → ticket ou `ponytail:`.

## Vérification

```bash
# candidats "répètent le code" — examiner chaque hit manuellement
rg -n "^\s*// (Arrange|TODO|FIXME|validate|parse|create|get|set|return|init|check|cleanup|Optional|Calls?|Sets?|Gets?)\b" backend/*/src frontend/src -i
# pas de nom court hors boucle/ fermetures
rg -n "let (x|tmp|val|res|obj|foo|bar)\b" backend/*/src frontend/src
devenv shell -- cargo clippy --all-targets -- -D warnings
```
