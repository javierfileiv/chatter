[![Rust CI](https://github.com/javierfileiv/chatter/actions/workflows/rust-ci.yml/badge.svg)](https://github.com/javierfileiv/chatter/actions/workflows/rust-ci.yml)

# Chatter

A real-time chat application built with Rust, using WebSocket for client-server communication and a broker-based message routing architecture.

## Features

- WebSocket-based real-time messaging
- Multi-room support via a central message broker
- User authentication (stub — always accepts)
- Async I/O with Tokio

## Project Structure

```
chatter/
├── client/                     # CLI client crate (stub)
│   ├── src/main.rs             #   Currently prints "Hello, world!"
│   └── Cargo.toml              #   edition = "2021", depends on clap
├── common/                     # Shared library crate
│   ├── src/lib.rs              #   Re-exports errors and ws_messages
│   ├── src/ws_messages.rs      #   Serde structs: AuthenticateUser, SendMessage, Logout
│   ├── src/errors.rs           #   Unused error enum
│   └── Cargo.toml              #   edition = "2021"
├── server/                     # Server crate
│   ├── src/main.rs             #   Tokio entrypoint, binds 127.0.0.1:8080
│   ├── src/auth.rs             #   Auth module root
│   ├── src/auth/client.rs      #   authenticate() stub — always returns true
│   ├── src/core.rs             #   Core module root
│   ├── src/core/server.rs      #   Accept loop, spawns per-connection tasks
│   ├── src/core/broker.rs      #   Central event loop: rooms, clients, routing
│   ├── src/core/connection.rs  #   Per-connection WebSocket handler
│   └── Cargo.toml              #   edition = "2021"
├── Cargo.toml                  # Workspace manifest (3 members)
├── Cargo.lock
├── .pre-commit-config.yaml     # fmt + clippy hooks
├── tools/requirements.txt      # pre-commit (Python)
└── AGENTS.md                   # Agent-specific development notes
```

## Architecture

```
client ──WebSocket──▶ server::core::server (accept loop)
                          │
                          ▼
                    server::core::connection (per-connection task)
                          │  authenticate (stub)
                          │  read incoming frames
                          ▼
                    server::core::broker (central event loop)
                          │  manages rooms & clients
                          │  routes Connect / Broadcast / JoinRoom events
                          ▼
                    BrokerToClientMsg → back to connection → WebSocket → client
```

| Crate | Role | Key Dependencies |
|-------|------|-----------------|
| `common` | Shared message types & errors | `serde`, `serde_json`, `anyhow`, `tokio-util` |
| `server` | WebSocket server + broker | `tokio` (full), `tokio-tungstenite`, `futures-util`, `flexi_logger`, `tracing` |
| `client` | CLI client (stub) | `clap` |

## Getting Started

### Prerequisites

- Rust stable toolchain (`rustup`)
- Python 3 (for pre-commit hooks)

### Build

```bash
cargo build --workspace
```

### Run the Server

```bash
# Default: 127.0.0.1:8080
cargo run --bin server

# Custom port
cargo run --bin server -- --port 3000
cargo run --bin server -- -p 9090
```

Logs are written to `logs/server.log` (gitignored) and echoed to stderr for warnings.

### Run the Client

```bash
cargo run --bin client
```

Currently a stub — prints "Hello, world!".

## Development

### Pre-commit Hooks

Install once after cloning:

```bash
pip install -r tools/requirements.txt
pre-commit install
```

Hooks run `cargo fmt --all` and `cargo clippy --all-targets --all-features -- -D warnings` on every commit.

### Format & Lint (CI-equivalent check)

```bash
cargo fmt --all && cargo clippy --all-targets --all-features -- -D warnings
```

### Tests

```bash
cargo test --all          # Run all tests
cargo test -p server      # Server crate only (includes broker tests)
cargo test -p common      # Common crate only
```

### Known Gaps

- **Connection handler** — hardcodes username/password/room instead of deserializing the first WebSocket frame via `common::ws_messages`.
- **Auth** — `server/src/auth/client.rs` always returns `true`.
- **Disconnect** — broker logs disconnect events but does not clean up clients/rooms yet.
- **Client** — `client/src/main.rs` is a stub; the previous TCP-based implementation is commented out.
- **CI** — The pre-commit hooks provide local enforcement, but no CI workflow file exists (`.github/workflows/` is empty).
- **`server/src/core/DONT_COMMITbroker_test.rs`** — stale duplicate of broker logic; keep as-is, do not modify.

## License

MIT — see [LICENSE](LICENSE).
