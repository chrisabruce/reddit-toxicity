# Reddit Toxicity Badge

A lightweight Rust microservice that generates dynamic toxicity badges for any subreddit. Outputs SVG, PNG, or JPEG. Embed them anywhere with a simple `<img>` tag.

![r/rust badge](http://localhost:3000/toxicity/r/rust.svg?size=420)

```html
<img src="https://yourdomain.com/toxicity/r/filmmakers.svg?size=400" alt="toxicity badge">
```

## How It Works

Fetches public data from Reddit's OAuth API and computes a **Toxicity Index (0–100)** from purely numerical signals — no AI, no NLP:

| Signal | Weight | Source |
|---|---|---|
| New post upvote ratio | 55% | Average `upvote_ratio` from `/new` (up to 100 posts with comments) — unfiltered community behavior with no survivorship bias |
| OP comment negativity | 30% | Fraction of the original poster's own comments with `score < 2` — measures community hostility toward the people posting |
| Negative comment % | 15% | Fraction of all comments with `score < 2` sampled from new posts |

The key insight: **hot posts have survivorship bias** — they're hot *because* they got upvoted. Toxicity shows up in new posts getting downvoted and OP comments being buried by the community. Posts with zero comments are ignored since they're too new to have signal.

Scores are cached (default 24 hours, configurable via `CACHE_TTL_HOURS`).

### Score Ranges

| Score | Label | Color |
|---|---|---|
| 0–20 | Very Low | Green |
| 21–35 | Low | Green |
| 36–50 | Moderate | Yellow |
| 51–65 | High | Yellow |
| 66–100 | Very High | Red |

## Reddit OAuth App (recommended, not required)

The server works without credentials using Reddit's public API, but you'll share rate limits with every other unauthenticated client on your IP. For reliable operation, set up OAuth:

1. Go to https://www.reddit.com/prefs/apps
2. Click **"create another app"**
3. Fill in:
   - **Name:** anything (e.g. `toxicity-badge`)
   - **Type:** select **script**
   - **About URL:** your deployed about page (e.g. `https://yourdomain.com/about`) or `http://localhost:3000/about` for local dev
   - **Redirect URI:** `http://localhost:3000` (not used by the app, but Reddit requires a value)
4. Click **Create app**
5. Note your **client ID** (string under the app name) and **client secret**

This gives you a dedicated rate limit of **60 requests/minute** from Reddit's OAuth API — not shared with anyone else.

## Quick Start

```bash
cp .env.example .env
# Edit .env with your Reddit credentials
cargo run -p reddit-toxicity-server --release
# Server starts on http://localhost:3000
```

Open http://localhost:3000 for the interactive about page, or go directly to a badge at http://localhost:3000/toxicity/r/rust.svg.

The server loads `.env` automatically via dotenvy. You can also pass env vars directly.

## Deployment

### Docker

The Docker image uses a `scratch` base with a statically-linked musl binary — the final image is just the binary plus CA certificates.

```bash
make build          # build the Docker image
make run            # build + start container on port 3000
make stop           # stop and remove container
make restart        # stop + run
make logs           # tail container logs
make status         # show container status
make clean          # stop + remove image
```

Pass credentials and change port:

```bash
make run REDDIT_CLIENT_ID=your_id REDDIT_CLIENT_SECRET=your_secret PORT=8080
```

**Caching:** In-memory via moka (configurable TTL, up to 500 subreddits). Cache is per-process — if the container restarts, it refetches on first request.

### Bare Metal / VPS

Build the release binary and run it directly. No container runtime needed.

```bash
cargo build -p reddit-toxicity-server --release
./target/release/reddit-toxicity
```

Environment variables:

| Variable | Default | Description |
|---|---|---|
| `REDDIT_CLIENT_ID` | *(optional)* | Reddit OAuth app client ID. Omit to use public API. |
| `REDDIT_CLIENT_SECRET` | *(optional)* | Reddit OAuth app client secret |
| `HOST` | `0.0.0.0` | Bind address |
| `PORT` | `3000` | Listen port |
| `CACHE_TTL_HOURS` | `24` | How long to cache scores per subreddit |
| `RUST_LOG` | `info` | Log level (`debug`, `info`, `warn`, `error`) |

All variables can be set in a `.env` file (loaded automatically) or passed directly. See `.env.example`.

```bash
REDDIT_CLIENT_ID=abc REDDIT_CLIENT_SECRET=xyz \
  HOST=127.0.0.1 PORT=8080 ./target/release/reddit-toxicity
```

For production, run behind a reverse proxy (nginx, Caddy) for TLS and put it in a systemd unit:

```ini
[Unit]
Description=Reddit Toxicity Badge
After=network.target

[Service]
ExecStart=/usr/local/bin/reddit-toxicity
Environment=REDDIT_CLIENT_ID=your_id
Environment=REDDIT_CLIENT_SECRET=your_secret
Environment=PORT=3000
Environment=RUST_LOG=info
Restart=always

[Install]
WantedBy=multi-user.target
```

## API

### `GET /` or `GET /about`

Interactive about page with live badge preview, methodology, and embedding instructions.

### `GET /health`

Returns `ok`. Use for health checks and readiness probes.

### `GET /toxicity/r/{subreddit}.{ext}`
### `GET /toxicity/{subreddit}.{ext}`

Returns a toxicity badge for the given subreddit.

**Supported formats:**

| Extension | Content-Type | Notes |
|---|---|---|
| `.svg` | `image/svg+xml` | Default. Scalable, smallest file size. |
| `.png` | `image/png` | Rasterized with embedded DejaVu Sans font. |
| `.jpg` / `.jpeg` | `image/jpeg` | White background, 90% quality. |
| *(none)* | `image/svg+xml` | Falls back to SVG. |

**Query parameters:**

| Param | Default | Range | Description |
|---|---|---|---|
| `size` | `420` | 200–650 | Badge width in pixels |

**Examples:**

```
/toxicity/r/rust.svg
/toxicity/r/politics.png?size=300
/toxicity/filmmakers.jpg?size=500
/toxicity/r/rust.jpeg
```

On error (subreddit not found, Reddit unreachable), a gray error badge is returned in the requested format.

## Project Structure

```
├── Cargo.toml              # Workspace root
├── .env.example            # Sample environment config
├── Dockerfile              # Multi-stage Docker build
├── Makefile                # Docker build/run/stop targets
├── core/                   # Shared library (no platform deps)
│   └── src/
│       ├── lib.rs
│       ├── oauth.rs        # OAuth constants + token response type
│       ├── scoring.rs      # Toxicity scoring + Reddit JSON parsing
│       └── svg.rs          # SVG badge rendering
└── server/
    ├── fonts/
    │   └── DejaVuSans-Bold.ttf   # Embedded font for PNG/JPEG rasterization
    └── src/
        ├── main.rs         # Entry point, config
        ├── about.rs        # About page (embedded HTML)
        ├── routes.rs       # Route handlers
        ├── rasterize.rs    # SVG → PNG/JPEG conversion via resvg
        ├── fetcher.rs      # Reddit OAuth client + fetching
        └── state.rs        # Moka in-memory cache
```
