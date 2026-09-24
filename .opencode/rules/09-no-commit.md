# 09 — Git — pas de `git add` ni `git commit` par les agents

## Règle

Les agents (opencode / sous-agents) **ne stagent jamais** et **ne commit jamais** :

- Interdits: `git add …`, `rtk git add …`, `git commit`, `git commit …`, `git stash`, `git push`, amend/rebase/restore/checkout perdant des changements. Sauf commande explicite de l'utilisateur dans la demande elle-même.
- Autorisés (lecture seule): `git status --short`, `git diff`, `git diff --staged`, `git log --oneline`, `git show`, `git branch`, `git rev-parse`. Pour tout le reste (`git rm --cached` etc.): uniquement sur demande explicite de l'utilisateur.
- `git restore --staged` sur un fichier justement spécifié par l'utilisateur est OK.

Le workspace n'est que modifié, jamais indexé: le staged = décision de l'utilisateur (diff à relire, secrets, découpage des commits).

## Vérification

```bash
rtk git status --short   # l'agent ne doit pas ajouter de lignes à l'index
```

Si un fichier a été staged par accident pendant la session, prévenir: `git restore --staged <fichier>` puis demander.
