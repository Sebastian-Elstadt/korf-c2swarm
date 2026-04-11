# Nodus (node agent)

**Nodus** is the KORF **client agent**: it runs on a host, derives a stable **nodus id** from machine characteristics, holds an **Ed25519 signing key**, and reaches **kommander** with registration and periodic **signed heartbeats**. Today that traffic goes over **UDP** as a **straightforward proof of concept**—not because UDP was required. **Transport is expected to evolve** (or gain alternatives) as the project grows; this is just the first cut.

This folder is the nodus package only; the server lives under `../kommander/`.

Like the rest of KORF, this is a **portfolio / learning** project: the focus is **Rust** (network framing, crypto usage, async runtime), not a supported endpoint agent.

## Responsibilities

1. **Identity** — Collect host signals (arch, hostname, MAC, etc.), hash into an id, generate a long-lived `SigningKey` (`korf-ed25519` / `ed25519-dalek`).
2. **Registration payload** — Send nodus id, MAC, public key, and metadata so kommander can persist the node.
3. **Heartbeat payload** — Sign a defined byte prefix so the server can verify freshness and update `last_seen` / logs.

## Security model (intended)

- **Ed25519** signatures bind payloads to the registered public key; kommander verifies using the shared **`korf-ed25519`** crate and stored keys.
- This is **educational**: threat model, key storage, and hardening are not at production standard.

## Code map

| Area | Notes |
|------|--------|
| `src/identity.rs` | Host scrape + `SigningKey` generation. |
| `src/c2com/payloads.rs` | Binary payload layout for registration and heartbeat. |
| `src/c2com/` | Reachability / command-channel helpers as implemented. |
| `src/anti_analysis.rs` | Experimental checks; not a guarantee of safety. |

## Running

- Built as **`nodus`** from the **repo root** workspace: `cargo run -p nodus`.
- Requires runtime configuration (kommander address, etc.) as defined in `src/main.rs` and related modules—see code for current env vars or constants.

## Relation to kommander

- The **core flow** is binary payloads to kommander’s **nodecom** listener (currently UDP); that can be extended or replaced without changing the high-level idea (register, heartbeat, verify, persist).
- Database and API are **server-side**; nodus only produces conformant packets for whatever transport is in use.

---

*For the full system picture, read the root [`README.md`](../README.md) and [`kommander/README.md`](../kommander/README.md).*
