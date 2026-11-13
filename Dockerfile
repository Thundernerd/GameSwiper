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

# The backend's manifests with its own version set to 0.0.0 in Cargo.toml and Cargo.lock. Every
# release bumps that version, and its dependencies needn't recompile for it: the backend stage
# copies these files, and a copy is cached on the files' content. The greps fail the build if the
# manifests change shape and the seds stop matching, rather than quietly losing the cache.
FROM debian:trixie-slim AS backend-manifests
WORKDIR /src/backend
COPY backend/Cargo.toml backend/Cargo.lock backend/rust-toolchain.toml ./
RUN sed -i '1,/^version = /s/^version = .*/version = "0.0.0"/' Cargo.toml \
 && sed -i '/^name = "hypecheck_api"$/{n;s/^version = .*/version = "0.0.0"/;}' Cargo.lock \
 && grep -qx 'version = "0.0.0"' Cargo.toml \
 && grep -A1 -x 'name = "hypecheck_api"' Cargo.lock | grep -qx 'version = "0.0.0"'

# Same Debian release as the runtime stage, so the binary links against the glibc it runs with.
FROM rust:1-slim-trixie AS backend
WORKDIR /src/backend
# Dependencies first, against a stub main.rs, so a change to src/ reuses this layer. target/ stays
# in the layer rather than a cache mount: the release workflow's layer cache exports layers, not
# mounts. rustup installs the toolchain named in rust-toolchain.toml on the first cargo call.
# --locked fails the build if the masked Cargo.toml and Cargo.lock disagree. The stub covers a
# single binary crate: a lib.rs or build.rs added later needs a stub here too.
COPY --from=backend-manifests /src/backend/ ./
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    mkdir src \
 && echo 'fn main() {}' > src/main.rs \
 && cargo build --release --locked \
 && rm -r src
COPY backend/ ./
# Touch main.rs so cargo rebuilds the crate even if the copied file is older than the stub's build.
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    touch src/main.rs \
 && cargo build --release --locked \
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
