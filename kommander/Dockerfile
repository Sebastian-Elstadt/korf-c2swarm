# -- Build GUI
FROM node:24-alpine AS node-builder
WORKDIR /app

COPY kommander/web/gui/package*.json ./
RUN npm ci

COPY kommander/web/gui/ ./

RUN npm run build

# -- Build Server
FROM rust:1.88-bookworm AS rust-builder
WORKDIR /app

COPY Cargo.toml Cargo.lock ./
COPY ed25519/ ./ed25519/
COPY kommander/domain/ ./kommander/domain/
COPY kommander/data/ ./kommander/data/
COPY kommander/app/ ./kommander/app/
COPY kommander/web/ ./kommander/web/
COPY kommander/nodecom/ ./kommander/nodecom/
COPY nodus/ ./nodus/

RUN cargo build --release -p app

# -- Run Server
FROM debian:bookworm-slim

RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates libssl3 \
    && rm -rf /var/lib/apt/lists/*

RUN useradd -r -s /usr/sbin/nologin kommander

WORKDIR /app
COPY --from=rust-builder /app/target/release/app /usr/local/bin/kommander
COPY --from=node-builder /app/dist/kommander/browser/ /var/www

USER kommander

EXPOSE 8080/tcp 8888/udp
ENTRYPOINT ["/usr/local/bin/kommander"]
