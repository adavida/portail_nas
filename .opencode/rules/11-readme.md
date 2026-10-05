# 11 — README — Structure imposée

> Cible: `README.md` racine. Les deux langues sont des miroirs exacts.

## Objectif

Le README est court et strict: français d'abord, anglais ensuite (`**Français** | [English](#english)`), toujours les mêmes 3 sections dans cet ordre, dans chaque langue:

1. **Une ligne** — le but, non technique: gérer les utilisateurs du NAS NixOS et afficher une page regroupant ses applications. Jamais plus d'une phrase d'intro, pas de stack dans cette ligne.
2. **Déploiement NixOS (flake)** — snippet `flake.nix`/config hôte minimal utilisant `nixosModules.default` (`services.portail`), tableau des options, note secrets/Authelia en 1-2 phrases, checks `nix build` + `nix flake check`. Pas de provisioning Authelia/OpenLDAP complet ni de génération de secrets: la source de vérité reste `nixos/portail.nix` et `devenv.nix`.
3. **Développement** — un seul bloc de commandes: `devenv up`/`devenv down` + ports, scripts devenv (`openfrontend`, `resetUsers`), `just test`, `treefmt` (sans préfixe `devenv shell --`, l'environnement dev est déjà actif).

## Règles

- Section 3 = commandes exactes, synchronisées avec `justfile`, `devenv.nix` (`processes`) et la règle `03-backend-frontend.md`. Changer un port/script ⇒ changer le README dans les deux langues.
- Section 2 = options strictement alignées sur `nixos/portail.nix` (`options.portail`). Ajouter/retirer une option du module ⇒ mettre à jour le tableau FR + EN.
- Titres FR/EN: `Développement` ↔ `Development`, `Déploiement NixOS (flake)` ↔ `NixOS deployment (flake)`.
- Contenu hors de ces 3 sections (layout, architecture, tutoriels) ⇒ pas dans le README, mais dans `.opencode/rules/` ou AGENTS.md.

## Interdits

- Bloc Authelia/OpenLDAP/secrets `openssl` complet dupliqué FR+EN.
- Divergence FR ↔ EN (une section, option ou commande présente dans une seule langue).
- Plus d'une ligne d'introduction.

## Vérification

```bash
rg -n "^## (Déploiement|Développement|English)" README.md          # 3 sections FR puis miroir EN
rg -c "devenv up|just test|treefmt" README.md                      # paires FR/EN (≥2 chacune)
```
