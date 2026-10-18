# Image backend du portail (multi-stage : build Rust -> runtime debian slim).
# Variables obligatoires a fournir au run (cf. backend/src/env/mod.rs) :
#   OIDC_ISSUER_URL, OIDC_CLIENT_ID, OIDC_SECRETS_DIR, OIDC_REDIRECT_URI,
#   APP_URL, LDAP_URL, LDAP_BASE_DN
# OIDC_SECRETS_DIR est un repertoire ecribable par le user du process : le
# portail y genere la cle de signature et clients.json au premier start.
# LDAP_ADMIN_PW est un CHEMIN de fichier dont le contenu est le secret
# (monter les secrets, jamais d'ENV en clair).
# Optionnel : LDAP_PEOPLE_OU (people), LDAP_GROUPS_OU (groups),
# LDAP_ADMIN_PW (admin), LDAP_TLS_CA (pem CA pour LDAPS), BIND_ADDR (0.0.0.0:3000).

FROM docker.io/library/rust:1.99-slim-bookworm AS build

RUN apt-get update \
    && apt-get install -y --no-install-recommends pkg-config libssl-dev \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Etape cache deps : les manifests seuls compilent les dependances, re-invalidees
# seulement quand Cargo.toml/lock changent (le src est copie apres).
COPY Cargo.toml Cargo.lock ./
COPY backend/Cargo.toml backend/Cargo.toml
RUN mkdir -p backend/src && printf 'fn main() {}\n' > backend/src/main.rs \
    && cargo build --release -p portail-backend

COPY backend/src backend/src
RUN touch backend/src/main.rs && cargo build --release -p portail-backend

FROM docker.io/library/debian:bookworm-slim

RUN apt-get update \
    && apt-get install -y --no-install-recommends libssl3 ca-certificates tini \
    && rm -rf /var/lib/apt/lists/* \
    && useradd --system --uid 10001 --shell /usr/sbin/nologin portail \
    && mkdir -p /oidc-secrets \
    && chown 10001:10001 /oidc-secrets

COPY --from=build /app/target/release/portail-backend /usr/local/bin/portail-backend

USER 10001:10001
EXPOSE 3000
ENV BIND_ADDR=0.0.0.0:3000
ENTRYPOINT ["/usr/bin/tini", "--", "/usr/local/bin/portail-backend"]
