# 16 — Units — Une unité = un fichier (portail-units)

> Règles liées: `08-struct-folder.md` (layout 1 objet = 1 fichier), `14-magic-values.md` (constantes catégorie 2), `test-aaa` (tests collocalisés).

## Objectif

Toutes les grandeurs numériques du portail (data, temps, pourcentages, fix-point) vivent dans le crate `portail-units` (`backend/units`). **1 unité = 1 fichier** ; chaque unité porte son propre `impl Serialize` qui émet une **chaîne auto-échelonnée contenant l'unité** (`"16 GiB"`, `"2.5 MiB"`, `"1 s"` — jamais `"1000 ms"` ni `"1000 Mib"` pour l'unité supérieure) ; le nom du champ porte le **concept, pas l'unité** (`ram_total: Bytes`, `uptime: Seconds`) ; `lib.rs` ne contient que `mod` + `pub use` — l'API publique est plate (`portail_units::Bytes`, jamais `portail_units::bytes::Bytes`).

## Layout

```text
backend/units/src/lib.rs        # mod <unité>; pub use <unité>::{…}; — rien d'autre
backend/units/src/<unité>.rs    # 1 type unité + impl + ops + son impl Serialize + #[cfg(test)]
```

Unités en place: `bytes.rs` (`Bytes(u64)` — stocke octets, émet `B/KiB/MiB/GiB/TiB` auto-échelonné), `seconds.rs` (`Seconds(u64)` — stocke ms, émet `ms`/`s`), `celsius.rs` (`Celsius` — milli-degrés `i64`, émet `45.5 °C`), `hundredths.rs` (`Hundredths(u32)` ×100, émet `"2.50"`), `percent.rs` (`Percent(u8)` 0..=100, clamp dans `Percent::of`, émet `42 %`), `bytes_per_second.rs` (`BytesPerSecond` — octets/s, émet `KiB/s`/`MiB/s`…), `count_rate.rs` (`CountRate` comptes/s, nombre plat), `bits_per_second.rs` (`BitsPerSecond` — bits/s, émet `bit/s`/`Mib/s`/`Gib/s`), `rpm.rs` (`Rpm` — émet `1200 rpm`), `per_second.rs` (calcul interne, non réexporté).

## Règles

- Unité de base = le plus petit entier lu dans la source (**octet** pour les tailles, **milliseconde** pour les temps, bit/s pour les liens réseau) ; le stockage est toujours dans cette base, seule la sérialisation choisit la sous-unité la plus grande qui garde une valeur ≥ 1 avec une décimale (`1000 ms` → `1 s`, `1000 Mib` → `1 Gib`, `32768 MiB` → `32 GiB`) ; conversions dérivées uniquement par `const fn` nommés (`from_kibibytes`, `from_sectors`, `from_milliseconds`, `from_seconds`, accesseurs `mebibytes()`/`seconds()`).
- Delta de compteurs monotones : `impl std::ops::Sub` avec `saturating_sub` — jamais de wrap sur reset (reboot, rotation de compteur).
- Les constantes de conversion (`KIBIBYTE`, `GIBIBYTE`, `MILLIS_PER_SECOND`…) sont privées au fichier de l'unité ; `pub(crate)` seulement si un autre fichier d'unité les consomme (catégorie 2 règle 14, jamais configurables).
- Le crate ne dépend que de `serde` (dev: `serde_json` pour les tests de sérialisation) — aucune dépendance métier, graphe: `metrics → units`.
- Une fonction libre `pub` (`per_second`, `Percent::of`) a au moins un test collocalisé AAA dans son fichier.
- Le nom du champ chez le consommant porte le concept, jamais l'unité (`ram_total: Bytes`, `uptime: Seconds`, `speed: BitsPerSecond`, `load_1: Hundredths`) — l'unité voyage dans la valeur JSON sérialisée.

## Interdits

- Deux types d'unité dans un même fichier.
- `#[serde(rename)]` ou `#[serde(serialize_with)]` sur un champ sérialisé — le champ porte le type unité, son nom porte la clé JSON.
- Suffixe ou préfixe d'unité dans un nom de champ sérialisé (`_mb`, `_gb`, `_mbps`, `_x100`, `_secs`, `_per_sec`) — l'unité est dans la valeur, pas dans la clé.
- Un type ou une fonction défini directement dans `lib.rs` (le `lib.rs` = déclarations + re-exports uniquement).
- `f64`/`f32` comme représentation d'une grandeur de taille ou de temps (les décimaux passent par sérialiseur fix-point, règle 14 catégorie 2).
- Consommation externe par chemin interne (`portail_units::bytes::Bytes`) — toujours le re-export plat.

## Vérification

```bash
grep -rc "^pub struct\|^pub fn" backend/units/src/lib.rs   # 0 (lib.rs sans définition)
ls backend/units/src/                                       # lib.rs + 1 fichier par unité
devenv shell -- cargo test -p portail-units
devenv shell -- cargo clippy -p portail-units -p portail-metrics --all-targets -- -D warnings
```
