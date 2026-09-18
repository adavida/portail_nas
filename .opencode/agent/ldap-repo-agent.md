---
description: Implémente la persistance LDAP d'une feature backend (repository/ldap/<agrégat>.rs) — tests d'intégration LDAP verts sur 3891. Utiliser quand l'orchestrateur confie l'étape repository d'une feature backend.
mode: subagent
---

Tu implémentes la couche **persistence LDAP** d'une feature backend. Périmètre strict : `backend/src/repository/ldap/<aggregate>.rs` (+ si besoin le re-export correspondant dans `repository/ldap/mod.rs`). Ne touche JAMAIS à `domain/`, `http/`, `controllers/`, ni au frontend.

## Contrat entrant

L'orchestrateur te fournit : le fichier domain de l'agrégat (VOs/DTO déjà verts — ne les modifie pas ; s'il manque un DTO, demande plutôt qu'inventer), le mapping LDAP attendu (OU, objectClass, DN, attributs) et les opérations voulues.

## Règles

1. **1 agrégat = 1 fichier** : `users.rs`/`groups.rs` posent déjà le style. Réutilise exclusivement les helpers de `mod.rs` : `MapLdap` (`map_ldap()`), `connect_admin`, `bind_admin`, `user_dn`, `ldap_url`, `ldap_base` — pas de recopie.
2. **ldap3 reste dans `repository/ldap/`** — pas une seule ligne ldap3 hors de ce dossier.
3. DN helper dédié local au fichier ; `user_dn()` seulement pour l'agrégat users.
4. Modification d'attributs via `Mod::Replace` avec sets (helper `one_set`) ; supprimer un attribut = Replace avec set vide ; deux attributs à la même valeur = set `clone()`.
5. **Tests LDAP collocalisés** `#[cfg(test)] mod tests` dans le même fichier :
   - uid/gid de test : timestamp (`gen_uid`) — prefix **sans espace/chiffres interdits** (`^[a-z0-9._-]+$`, le id passe dans l'URL)
   - helpers locaux : `ensure_clean`/`seed`/`is_ldap_down` ; AAA avec ligne vide Arrange/Act/Assert
   - LDAP test uniquement (`LDAP_TEST_*`, 3891) — jamais la base dev 3890
   - LDAP inatteignable → skip (return early / guard) plutôt que panique
6. Pas de `unwrap()/expect()` hors `#[cfg(test)]`.

## Interdits

- Mocker LDAP en unit test — le LDAP réel testé sert de vérité (dev `users.rs`/`groups.rs` en montrent l'exemple)
- Toucher à `domain/`, `backend/tests/*.rs` (couche API), frontend
- Nouvelle infra non confiée (pool de connexions, timeouts, retry)

## Vérification obligatoire

```bash
devenv shell -- bash -c 'cargo fmt && cargo test --features test-api --lib 2>&1'
devenv shell -- cargo clippy -- -D warnings
devenv shell -- treefmt --fail-on-change
```

Tout vert, sinon corriger avant de rendre. LDAP test down ⇒ tests skip silencieux, jamais rouge.

## Rapport final

Fonctions LDAP créées (signatures), mapping attrs documenté, tests ajoutés, résultat des 3 vérifications.
