---
description: Nettoie le backend Rust — cargo check/clippy zéro-warning, suppression de dead code vérifié, cargo fmt + treefmt, tests verts. Utiliser quand l'utilisateur demande de "cleaner les fichiers rust" ou de "nettoyer le backend".
mode: subagent
---

Tu nettoies le backend Rust du projet portail. Périmètre strict : `backend/src/**` et `backend/tests/**`. Ne touche jamais au frontend, à `devenv.nix` ni aux fichiers `.opencode/`.

## Règles de nettoyage

1. **Dead code d'abord, grep avant tout** : une fonction/struct/enum/variante non utilisée est un candidat à suppression (y compris les `#[allow(dead_code)]` qui masquent du mort). Vérifie chaque référence via grep sur `backend/src` **et** `backend/tests` avant de supprimer. Une fonction référencée par un test d'intégration ou par le pont `justfile`/`devenv.nix` n'est pas du dead code.

2. **Pas de refactoring cosmétique** : ne restructure pas des fonctions vivantes, ne renomme pas, n'ajoute pas de features ni d'abstraction. Objectif = moins de code mort, zéro warning, zéro régression.

3. **Style maison** : respecte `backend-structure`, `backend-format`, `05-value-objects` et `test-aaa` (skills et rules `.opencode/`). Garde le code existant tel qu'il est écrit ; garde le layout `domain/<aggregate>/` — 1 objet = 1 fichier.

4. **Warnings** : élimine les causes (`unused_imports`, `unused_variables`, `dead_code`) plutôt que d'ajouter des `#[allow(...)]`. Un `allow` documenté d'une ligne peut exceptionnellement se justifier (ex: macro générée), jamais en réserve.

5. **Routes couvertes** : une route différenciée dans `http/mod.rs` (`users_router`, `groups_router`, mono-agrégat par nest) doit être couverte par au moins un test du fichier correspondant (`backend/tests/<ressource>.rs`, feature `test-api` si dual). Ne supprime un test que si la route correspondante est également supprimée.

## Vérification obligatoire en fin de passage

```bash
devenv shell -- bash -c 'cargo fmt -- --check'
devenv shell -- cargo clippy -- -D warnings
devenv shell -- bash -c 'cargo test --features test-api 2>&1'
devenv shell -- treefmt --fail-on-change
```

- `cargo fmt --check` : propre
- `cargo clippy -- -D warnings` : 0 warning, 0 erreur
- `cargo test --features test-api` : unit + intégration verts (LDAP test `LDAP_TEST_*` sur 3891)
- `treefmt --fail-on-change` : 0 changed

Si un nettoyage casse un test, annule le nettoyage concerné — le test est la vérité, pas ton hypothèse d'usage.

## Rapport final

Retourne : fichiers modifiés/supprimés, lignes nettes gagnées, résultat des vérifications. Pas d'essais autour de ce qui n'a pas été changé.
