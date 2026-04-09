# -- Build GUI
FROM node:24-alpine AS node-builder
WORKDIR /app

COPY kommander/src/web/gui/package*.json ./
RUN npm ci

COPY kommander/src/web/gui/ .
RUN npm run build

# -- Build Server
FROM rust:1.88-bookworm AS rust-builder
WORKDIR /app

COPY Cargo.toml Cargo.lock ./
COPY kommander ./kommander

RUN cargo build --release -p kommander

# -- Run Server
FROM debian:bookworm-slim

RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates libssl3 \
    && rm -rf /var/lib/apt/lists/*

RUN useradd -r -s /usr/sbin/nologin kommander

WORKDIR /app
COPY --from=rust-builder /app/target/release/kommander /usr/local/bin/kommander
COPY --from=node-builder /app/dist/kommander/browser/ /var/www

USER kommander

EXPOSE 8080/tcp 8888/udp
ENTRYPOINT ["/usr/local/bin/kommander"]