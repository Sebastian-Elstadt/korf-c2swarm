# KORF [ Under Development - Currently At MVP ]

**Kommander** (control plane) and **nodus** (lightweight node agent)—a small Rust project exploring command-and-control style networking, cryptography, and persistence. This repo is a **learning exercise** and portfolio piece: it shows how I work in **Rust** (async I/O, crates, Postgres, Axum) and how I use **AI tooling** for faster iteration on straightforward surfaces—mainly the **Angular GUI**—while keeping core protocol and server logic hand-owned and reviewed.

The system is **not production-ready**; behaviour and APIs will change. Further work is tracked in `todos.txt` when that file is present in the tree.

## What’s here

| Part | Role |
|------|------|
| **kommander** | Server: HTTP API + static web UI, UDP listener for node traffic, Postgres for nodes and logs. |
| **nodus** | Agent: identifies the host, registers with kommander, sends signed heartbeats. |
| **ed25519** (`korf-ed25519`) | Shared Ed25519 helpers (`ed25519-dalek`) so kommander and nodus agree on crypto without duplicating versions. |

**Transport:** Nodus and kommander currently talk over **UDP** only as a **simple proof of concept**—not because UDP is inherently “right” for this. The on-the-wire layer is meant to stay swappable: other transports or protocols can be added or substituted later; that work is not done here yet.

## Stack (high level)

- **Rust** workspace (see root `Cargo.toml`): `tokio`, `axum`, `sqlx`, UDP sockets, etc.
- **Postgres** for node registry and event logs.
- **Angular** SPA under `kommander/web/gui`, served by the same binary as the API.
- **Docker** build path documented in `kommander/Dockerfile` (build from repo root).

## Repo layout

```
ed25519/          # shared korf-ed25519 crate
kommander/        # server + GUI + nodecom — see kommander/README.md
nodus/            # agent — see nodus/README.md
```

## Quick start (developers)

1. **Database:** Postgres with `DATABASE_URL` set (migrations live under `kommander/data/migrations/`).
2. **Kommander:** From the repo root, `cargo run -p app` (bind addresses via env as implemented in `kommander/app`).
3. **Nodus:** `cargo run -p nodus` with config pointing at your kommander UDP endpoint.
4. **GUI dev:** `cd kommander/web/gui && npm install && npm start` (API proxied or same origin as configured).

For image builds and exact copy paths, see `kommander/README.md`.

## Documentation map

- [`kommander/README.md`](kommander/README.md) — control plane: crates, API surface, Docker.
- [`nodus/README.md`](nodus/README.md) — agent: identity, payloads, security model at a glance.

---

*Portfolio note: scope and quality reflect deliberate practice, not a shipped product.*
