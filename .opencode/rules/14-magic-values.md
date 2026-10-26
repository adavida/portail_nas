# 14 — Valeurs magiques — Zéro nombre nu, tout paramétrable

> Règles liées: `06-env.md` (clé via `RawConfig`), `05-value-objects.md` (invariants VO), `12-names-comments.md` (pas de commentaire, le nom porte la sémantique).

## Objectif

Aucun nombre « magique » (non auto-explicable) dans le code de prod. Toute valeur qui peut raisonnablement varier selon le déploiement passe par le fichier de config ; le reste est nommé par une constante.

## Trois catégories

1. **Valeur de déploiement → clé de config** (règle 06): TTL, limites de sécurité (longueur minimale, plafond de tentatives), taille de clé, chemins, ports. Motif imposé:
   - champ `Config` + clé `Option<T>` `#[serde(default)]` dans `RawConfig` (suffixe `_secs` pour les durées);
   - défaut nommé `DEFAULT_<X>` dans `config/mod.rs`;
   - entry dans `log_summary()`;
   - assertion défaut + assertion valeur custom dans les tests `config/mod.rs`;
   - lecture unique du champ (`Config::global().<x>` ou champ déjà passé en paramètre), jamais de re-dérivation.
     Exemples en place: `oidc_*_ttl_secs`, `oidc_key_bits`, `password_min_length`, `login_max_attempts`.
     Une valeur crypto ne peut devenir clé que **montante** avec plancher validé par `Config::load` (rejet sous le plancher) — ex: `oidc_id_bytes` (défaut = plancher = 16, 32 autorisé, 4 refusé).
2. **Constante de protocole ou de force cryptographique → `const` nommée, jamais configurable** (une fausse valeur = faille ou interop cassée): codes résultat LDAP (`LDAP_RESULT_SUCCESS`, `LDAP_RESULT_INVALID_CREDENTIALS`), algorithmes (`RS256`), `SESSION_COOKIE`.
3. **Invariant de VO (règle 05) → bornes dans `try_new`** tant que le concept n'est pas déployable-variable: format `uid` 2..32 + regex, `name` ≤100, `email` ≤254. Une borne de VO qui devient tunable par déploiement migre en catégorie 1 (ex: `password_min_length`).

## Interdits

- Nombre nu dans du code de prod hors `0`/`1` évidents, indexations (`[0u8; N]` via const), et littéraux de tests.
- `const` magique non lue depuis la config alors que la valeur se justifie par environnement (ex: plafond de tentatives) → catégorie 1 obligatoire.
- Clé de config lue ailleurs que via `Config` (pas de parse TOML local, règle 06).
- Configurationner une constante de protocole (rend le provider interop-incompatible ou affaiblit la crypto).

## Vérification

```bash
rg -n "const [A-Z_]+ *(=|:) *[0-9]" backend/*/src | grep -v "DEFAULT_"   # les const restantes doivent être catégorie 2 (protocole)
rg -n "[^0-9a-zA-Z_](60|300|3600|10|8|16|49|2048)[^0-9a-zA-Z_]" backend/*/src --glob '!**/config/**' | grep -v "cfg(test)\|LDAP_RESULT\|RANDOM_ID\|DEFAULT_"   # vide hors tests/config
devenv shell -- cargo test
```
