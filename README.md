# Hype Check

[![CI](https://github.com/Thundernerd/HypeCheck/actions/workflows/ci.yml/badge.svg?branch=main)](https://github.com/Thundernerd/HypeCheck/actions/workflows/ci.yml)

Hype Check shows games from finished IGDB events as cards. You can like, dislike, or defer each card. Later visits show only the games that you did not rate.

## Stack

The project has two parts:

- **backend/** — A Rust API (Axum + SQLite). The API talks to [IGDB](https://api-docs.igdb.com/) with Twitch OAuth.
- **frontend/** — A Nuxt 4 (Vue 3) app. The app uses mock fixtures when `NUXT_PUBLIC_USE_MOCK` is `true` (default). When that value is `false`, the app uses the live API through the `/api` proxy.

## Setup

> **Renamed from GameSwiper.** The default database file is now `backend/data/hypecheck.db`. To keep your existing local data, move the old file:
>
> ```bash
> mv backend/data/gameswiper.db backend/data/hypecheck.db
> ```
>
> If your `backend/.env` was copied from an older `.env.example`, it still sets `DATABASE_URL=sqlite:data/gameswiper.db?mode=rwc`. Change that line to `hypecheck.db` after the move, or the backend starts a new, empty database. Or skip the move and leave `DATABASE_URL` pointing at the old file.

### 1. IGDB / Twitch credentials

1. Create a Twitch account.
2. Enable 2FA on that account.
3. Register a **Confidential** application at the [Twitch Developer Console](https://dev.twitch.tv/console/apps).
4. Generate a Client Secret.
5. Copy the Client ID and the Client Secret.

### 2. Backend

```bash
cp .env.example backend/.env
# edit backend/.env — set IGDB_CLIENT_ID, IGDB_CLIENT_SECRET, SESSION_SECRET

cd backend
mkdir -p data
cargo run
```

The API listens on `http://127.0.0.1:8080`.

If you do not set IGDB credentials, the backend creates a few finished demo events. You can use these events to try the full flow on your machine.

Before you commit, run the same backend checks that CI runs:

```bash
cargo fmt --check
cargo clippy --all-targets --locked -- -D warnings
cargo test --locked
```

Each command exits non-zero on a problem. Run `cargo fmt` to fix the formatting.

### 3. Frontend

The frontend uses Node 26, as set in `frontend/.nvmrc`. If you use nvm, run `nvm use` in `frontend/`.

```bash
cd frontend
cp ../.env.example .env   # optional
# NUXT_PUBLIC_USE_MOCK=true  → fixtures (default)
# NUXT_PUBLIC_USE_MOCK=false → live API via /api proxy

npm install
npm run dev
```

Open `http://localhost:3000`.

To type-check the frontend (including `.vue` files), run:

```bash
npm run typecheck
```

`nuxt build` strips types without checking them, so run this before you commit. It exits non-zero on any type error.

To lint the frontend with ESLint (unused variables, Vue template mistakes and similar), run:

```bash
npm run lint       # report problems; exits non-zero on any error
npm run lint:fix   # fix what ESLint can fix by itself
```

Run `npm run lint` before you commit too. The rules come from [`@nuxt/eslint`](https://eslint.nuxt.com/packages/module) and can be overridden in `frontend/eslint.config.mjs`.

To run the frontend unit tests with [Vitest](https://vitest.dev/), run:

```bash
npm test
```

Run `npm test` before you commit too. It exits non-zero on any failing test. Tests sit next to the code they cover, as `*.test.ts` files (for example `app/shared/swipe.test.ts`).

`frontend/package.json` has an `overrides` block that moves `minimatch` 10 to 11 and `glob` 13 to 14. This clears two moderate `npm audit` advisories in build and lint tooling ([GHSA-g3hv-75wg-m249](https://github.com/advisories/GHSA-g3hv-75wg-m249) and [GHSA-3g3m-2fjr-6pw4](https://github.com/advisories/GHSA-3g3m-2fjr-6pw4)). ESLint and Nitro still ask for the older majors. Once they ask for `minimatch` 11 and `glob` 14 themselves, remove the block and run `npm install`.

### Serving the built frontend

The backend can also serve the built frontend. Then one process on one origin runs the whole app, with no `/api` proxy and no CORS.

1. Build the frontend against the live API:

   ```bash
   cd frontend
   npm install
   NUXT_PUBLIC_USE_MOCK=false npm run generate
   ```

   This writes static files to `frontend/.output/public`.

2. In `backend/.env`, point `STATIC_DIR` at that folder. A relative path is resolved from the directory you start the backend in, which is `backend/` if you use `cargo run` as below:

   ```bash
   STATIC_DIR=../frontend/.output/public
   ```

3. Start the backend with `cargo run` and open `http://127.0.0.1:8080`.

Paths under `/api` still go to the API. Other paths serve the file with that name, or `index.html` if there is no such file, so a reload on a page such as `/events/3/overview` still works.

## Docker

Each release also publishes a container image to the GitHub Container Registry (GHCR), for `linux/amd64` and `linux/arm64`. The image runs the whole app on port 8080 and keeps its SQLite database in the `/data` volume:

```bash
docker run -d --name hypecheck \
  -p 8080:8080 \
  -v hypecheck-data:/data \
  -e SESSION_SECRET=change-me-to-a-long-random-string \
  -e IGDB_CLIENT_ID=your-client-id \
  -e IGDB_CLIENT_SECRET=your-client-secret \
  ghcr.io/thundernerd/hypecheck:latest
```

Open `http://localhost:8080`. Leave out the two IGDB variables to try the app with the demo events. Set `SESSION_SECRET` to a fixed value. If you don't, the backend picks a new one at each start.

`latest` follows the newest stable release. To pin a version, use `X.Y.Z`, or `X.Y` to also get patch releases. Pre-releases are only published under their own version, such as `1.0.0-rc.1`.

The container runs as a non-root user and reports its health from `/api/health`. The other variables in [`.env.example`](.env.example), such as `IGDB_SYNC_INTERVAL_HOURS`, can be passed with `-e` too.

To build the image yourself, run `docker build -t hypecheck .` in the repository root.

## API contract

The TypeScript types are in [`frontend/app/shared/api.ts`](frontend/app/shared/api.ts). The mock fixtures are in [`frontend/app/shared/fixtures/`](frontend/app/shared/fixtures/).

| Method | Path | Notes |
|--------|------|--------|
| POST | `/api/auth/register` | `{ username, password }` |
| POST | `/api/auth/login` | session cookie |
| POST | `/api/auth/logout` | |
| GET | `/api/auth/me` | current user |
| GET | `/api/events` | finished events + progress |
| GET | `/api/events/:id/queue` | remaining deck |
| POST | `/api/events/:id/swipes` | `{ gameId, action }` like\|dislike\|defer |
| GET | `/api/events/:id/overview` | liked / disliked + links |

## Swipe rules

- **like / dislike** — These actions are final. The card does not return to the deck.
- **defer** — The app saves the card and moves it to the back of the queue. Later visits show deferred cards after unseen games.

## Releasing

Pushing a version tag publishes a GitHub Release. The [release workflow](.github/workflows/release.yml) runs the CI checks, builds the backend for Linux, macOS and Windows on x86_64 and arm64, and packs each build with the built frontend. Each archive runs the whole app on one port; its `RUN.md` explains how. The release also gets a `SHA256SUMS` file and generated release notes. The same workflow publishes the [Docker image](#docker) to `ghcr.io/thundernerd/hypecheck`.

1. Bump the version in both `backend/Cargo.toml` and `frontend/package.json` to the same `X.Y.Z`. Run `cargo check` in `backend/` and `npm install` in `frontend/` so both lockfiles pick up the new version.
2. Open a pull request and merge it to `main`.
3. Tag the merge commit on `main` and push the tag:

   ```bash
   git checkout main && git pull
   git tag vX.Y.Z && git push origin vX.Y.Z
   ```

The workflow fails before it builds anything if the tag doesn't match both versions. No release or image tag is published if a CI check or any build fails.

A tag with a suffix, such as `v1.0.0-rc.1`, publishes a pre-release. Its manifests can say either `1.0.0-rc.1` or `1.0.0`, so you can cut release candidates without a version bump. A pre-release image gets only its own version tag, so it never moves `latest` or `X.Y`.

A stable release moves the `latest` image tag and the "Latest" GitHub Release only when no other stable `vX.Y.Z` tag is newer. A patch for an older line, such as `v1.1.5` after `v1.2.0`, gets its own `X.Y.Z` and `X.Y` tags and leaves `latest` on `1.2.0`.

GHCR makes a new package private. After the first image is published, set the `hypecheck` package's visibility to public in its package settings so `docker pull` works without logging in.

The build targets are listed once, in the workflow's `targets` job. Each entry names its Rust target, runner, archive suffix and `arch_pattern`: an extended regex that the binary's `file` description must match, ignoring case, before the release is published. Windows entries also set `ext`, the binary's file extension.

The workflow's tag check and packing live in [`.github/scripts/`](.github/scripts/). If you change them, run the same checks that CI runs:

```bash
shellcheck .github/scripts/*.sh
bash .github/scripts/check-release-version.test.sh
bash .github/scripts/docker-tags.test.sh
```
