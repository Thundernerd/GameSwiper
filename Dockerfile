# syntax=docker/dockerfile:1
#
# Hype Check in one image: the backend serves the API and the built frontend on port 8080, and
# keeps its SQLite database in /data. See README "Docker".
#
# Every stage builds for the platform it runs on. The release workflow builds each architecture on
# its own native runner, so nothing is compiled under emulation.

FROM node:26-trixie-slim AS frontend
WORKDIR /src/frontend
COPY frontend/ ./
RUN --mount=type=cache,target=/root/.npm npm ci
RUN NUXT_PUBLIC_USE_MOCK=false npm run generate \
 # useMock defaults to true (nuxt.config.ts); the image must call /api, not the fixtures.
 && grep -q 'useMock:false' .output/public/index.html

# Same Debian release as the runtime stage, so the binary links against the glibc it runs with.
FROM rust:1-slim-trixie AS backend
WORKDIR /src/backend
COPY backend/ ./
# rustup installs the toolchain named in rust-toolchain.toml on the first cargo call. The binary
# is copied out of the cached target directory, which later stages can't see.
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/src/backend/target \
    cargo build --release --locked \
 && cp target/release/hypecheck_api /usr/local/bin/hypecheck_api

FROM debian:trixie-slim
# ca-certificates for the HTTPS calls to IGDB and Twitch; curl for the health check.
RUN apt-get update \
 && apt-get install -y --no-install-recommends ca-certificates curl \
 && rm -rf /var/lib/apt/lists/*
RUN useradd --system --uid 10001 --user-group --no-create-home --home-dir /nonexistent hypecheck \
 && mkdir /data \
 && chown hypecheck:hypecheck /data

COPY --from=backend /usr/local/bin/hypecheck_api /usr/local/bin/hypecheck_api
COPY --from=frontend /src/frontend/.output/public /app/public

ENV BIND_ADDR=0.0.0.0:8080 \
    STATIC_DIR=/app/public \
    DATABASE_URL=sqlite:/data/hypecheck.db?mode=rwc

WORKDIR /app
USER hypecheck
VOLUME /data
EXPOSE 8080
# Checks the port in BIND_ADDR, so the check still works if that is overridden.
HEALTHCHECK --interval=30s --timeout=3s --start-period=10s --retries=3 \
  CMD curl -fsS "http://127.0.0.1:${BIND_ADDR##*:}/api/health" || exit 1
CMD ["hypecheck_api"]
