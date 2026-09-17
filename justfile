default: dev

dev:
    #!/usr/bin/env bash
    echo "backend: cargo run | frontend: npm run dev"
    echo "  terminal 1: devenv shell -- cargo run -p portail-backend"
    echo "  terminal 2: devenv shell -- npm --prefix frontend run dev"

test:
    cargo test
    npm --prefix frontend test -- --run

backend-run:
    cargo run -p portail-backend

frontend-dev:
    npm --prefix frontend run dev
