# GameSwiper

GameSwiper shows games from finished IGDB events as cards. You can like, dislike, or defer each card. Later visits show only the games that you did not rate.

## Stack

The project has two parts:

- **backend/** — A Rust API (Axum + SQLite). The API talks to [IGDB](https://api-docs.igdb.com/) with Twitch OAuth.
- **frontend/** — A Nuxt 4 (Vue 3) app. The app uses mock fixtures when `NUXT_PUBLIC_USE_MOCK` is `true` (default). When that value is `false`, the app uses the live API through the `/api` proxy.

## Setup

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

### 3. Frontend

```bash
cd frontend
cp ../.env.example .env   # optional
# NUXT_PUBLIC_USE_MOCK=true  → fixtures (default)
# NUXT_PUBLIC_USE_MOCK=false → live API via /api proxy

npm install
npm run dev
```

Open `http://localhost:3000`.

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
