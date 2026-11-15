# Image backend du portail (multi-stage : build Rust -> runtime debian slim).
# Le binaire ne lit AUCUNE variable d'env de config : il prend --config <TOML>
# (voir docker/config.toml monté en lecture seule par compose.yml).
# Données persistées (volumes compose) : /data (SQLite db_path), /oidc-secrets
# (clé de signature générée au premier start). /secrets = fichiers secrets
# montés ro (mot de passe admin LDAP).

FROM docker.io/library/rust:1.99-slim-bookworm AS build

RUN apt-get update \
    && apt-get install -y --no-install-recommends pkg-config libssl-dev libacl1-dev \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY Cargo.toml Cargo.lock ./
COPY backend backend

RUN cargo build --release -p portail-server

FROM docker.io/library/debian:bookworm-slim

RUN apt-get update \
    && apt-get install -y --no-install-recommends libssl3 libacl1 ca-certificates tini \
    && rm -rf /var/lib/apt/lists/* \
    && useradd --system --uid 10001 --shell /usr/sbin/nologin portail \
    && mkdir -p /data /oidc-secrets \
    && chown 10001:10001 /data /oidc-secrets

COPY --from=build /app/target/release/portail-backend /usr/local/bin/portail-backend

USER 10001:10001
EXPOSE 3000
ENTRYPOINT ["/usr/bin/tini", "--", "/usr/local/bin/portail-backend"]
CMD ["--config", "/etc/portail/config.toml"]
