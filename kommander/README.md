# Kommander (control plane)

**Kommander** is the KORF server: it accepts **node registration and heartbeats** (today over **UDP** via `nodecom`), persists **nodes and structured logs in Postgres**, and exposes a **REST API** plus a **single-page GUI** (Angular) for operators. This document is the public-facing overview for the `kommander/` tree.

**UDP is not a principled choice**—it’s a minimal way to move framed payloads while the project focuses on Rust, crypto verification, and persistence. **How nodes attach to kommander can change** (additional transports, different framing, etc.); nothing here is locked to datagram semantics long term.

This project is a **learning build**; the interesting parts here are **Rust** (concurrency, protocols, data layer), while the **GUI** is intentionally lighter lift—iterated with AI assistance so time stays on core behaviour.

## Crates (workspace members under repo root)

| Crate | Purpose |
|-------|---------|
| **app** | Binary entry: loads config, runs DB migrations, spawns web + UDP tasks. |
| **web** | Axum router: `/api/*` JSON endpoints, static file fallback for the built GUI. |
| **nodecom** | UDP service: decode payloads from nodus, verify signatures (via `korf-ed25519`), update DB, append logs. |
| **domain** | Types and ports (e.g. node models, repository traits). |
| **data** | `sqlx` repositories, migrations, Postgres implementation. |

Shared crypto lives in the repo-root **`ed25519/`** package (`korf-ed25519`), not inside `kommander/`.

## API (conceptual)

- Health and **node listing** for the dashboard.
- **Per-node logs** for the operator UI.
- Exact paths and JSON shapes follow `kommander/web/src/api/`; the GUI models align with those types.

## GUI

- **Location:** `web/gui` — Angular app (modern control flow, HttpClient to same-origin `/api`).
- **Role:** View nodes, heartbeat-style metadata, command-queue UX (mock or wired depending on branch), and log tail—suitable for demos, not a security product UI.

## Docker

- **`Dockerfile`** in this folder expects the **repository root** as build context so the workspace `Cargo.toml`, `ed25519/`, and `kommander/` crates are all available.
- Typical invocation: from repo root,  
  `docker build -f kommander/Dockerfile .`
- The image builds the **`app`** package only; the **nodus** agent is not required inside that image.

## Configuration

- **`DATABASE_URL`** is required for migrations and runtime (see `kommander/app`).
- HTTP/UDP bind addresses are taken from environment variables as wired in `app` (check source for current names).

---

*Outstanding features and fixes may be listed in the repo’s `todos.txt`.*
