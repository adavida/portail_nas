# Image prod du portail-frontend : build vite (URLs compilées via ARG) -> nginx TLS.
# Build hors compose : docker build -f docker/frontend.Dockerfile -t portail-frontend:$(git rev-parse --abbrev-ref HEAD | tr / -) frontend

FROM docker.io/library/node:22-alpine AS build

WORKDIR /app

ARG VITE_APP_URL=https://portail.localhost
ARG VITE_BACKEND_URL=https://portail.localhost
ARG VITE_OIDC_ISSUER_URL=https://authelia.localhost
ARG VITE_OIDC_REDIRECT_URI=https://portail.localhost/callback
ENV VITE_APP_URL=$VITE_APP_URL \
    VITE_BACKEND_URL=$VITE_BACKEND_URL \
    VITE_OIDC_ISSUER_URL=$VITE_OIDC_ISSUER_URL \
    VITE_OIDC_REDIRECT_URI=$VITE_OIDC_REDIRECT_URI

COPY package.json package-lock.json ./
RUN npm ci

COPY . .
RUN npm run build

FROM docker.io/library/nginx:alpine

COPY nginx.conf /etc/nginx/conf.d/default.conf
COPY --from=build /app/dist /usr/share/nginx/html

EXPOSE 443
