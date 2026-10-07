#!/usr/bin/env bash
# Construit les images locales taguées du nom de la branche git (sanitisée) + pull des dépendances.
# À lancer hors compose : docker/compose.yml référence ces tags avec pull_policy: never.
set -euo pipefail

tag="${GIT_TAG:-$(git rev-parse --abbrev-ref HEAD | tr '/_' '--')}"

docker pull docker.io/authelia/authelia:4
docker pull docker.io/osixia/openldap:1.5.0
docker build -f docker/backend.Dockerfile -t "portail-backend:$tag" .
docker build -f docker/frontend.Dockerfile -t "portail-frontend:$tag" frontend

echo "images: portail-backend:$tag portail-frontend:$tag"
echo "lancer: GIT_TAG=$tag docker compose -f docker/compose.yml up -d"
