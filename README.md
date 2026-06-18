[![Rust CI](https://github.com/javierfileiv/chatter/actions/workflows/rust-ci.yml/badge.svg)](https://github.com/javierfileiv/chatter/actions/workflows/rust-ci.yml)
[![Coverage](https://javierfileiv.github.io/chatter/badge.svg)](https://javierfileiv.github.io/chatter/)

# Chatter

A real-time chat application built with Rust, using WebSocket for client-server communication and a broker-based message routing architecture.

## Features

- WebSocket-based real-time messaging
- Multi-room support via a central message broker
- User authentication (stub — always accepts)
- Disconnect notifications — remaining room members are notified when a client leaves
- Graceful and abrupt disconnect handling with automatic cleanup
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
│   ├── src/main.rs             #   Tokio entrypoint, binds 127.0.0.1:8080, CLI args (port, log-dir)
│   ├── src/auth.rs             #   Auth module root
│   ├── src/auth/client.rs      #   authenticate() stub — always returns true
│   ├── src/core.rs             #   Core module root
│   ├── src/core/server.rs      #   Accept loop, spawns per-connection tasks
│   ├── src/core/broker.rs      #   Central event loop: rooms, clients, routing
│   ├── src/core/connection.rs  #   Per-connection WebSocket handler, pure parsing functions
│   ├── src/core/connection/    #   Connection module extras
│   │   └── connection_tests.rs #   Unit tests for parse functions, ws_half_reader, and ws_half_writer
│   └── Cargo.toml              #   edition = "2021"
├── tests/                      # Python integration tests
│   ├── conftest.py             #   Pytest fixtures
│   └── test_integration.py     #   End-to-end WebSocket tests
├── scripts/                    # Helper scripts
│   ├── test_integration.sh     #   Builds server, runs pytest, cleans up
│   └── gen_coverage.sh         #   Generates coverage report (text + HTML)
├── .github/workflows/          # CI workflows
│   ├── rust-ci.yml             #   Build, test, fmt, clippy
│   ├── coverage.yml            #   Coverage report + GitHub Pages deploy
│   └── integration.yml         #   Python integration tests
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
# Default: 127.0.0.1:8080, logs in ./logs/
cargo run --bin server

# Custom port and log directory
cargo run --bin server -- --port 3000 --log-dir /tmp/my-logs
```

Logs are written to the specified directory (default: `logs/`) and echoed to stderr for warnings.

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
cargo test -p server      # Server crate only (includes broker + connection tests)
cargo test -p common      # Common crate only
```

### Integration Tests

Run the integration test script which builds the server, starts it on a free port, runs pytest, and cleans up:

```bash
./scripts/test_integration.sh
```

Requires: `pip install websockets pytest pytest-asyncio`

Tests cover: authentication, message broadcasting, room isolation, logout, and disconnect notifications.

### CI Workflows

Three GitHub Actions workflows run on push/PR to main/master:

- **rust-ci.yml** — builds, runs unit tests, checks formatting and clippy
- **coverage.yml** — generates coverage report, deploys HTML to GitHub Pages
- **integration.yml** — runs Python integration tests via `./scripts/test_integration.sh`

### Known Gaps

- **Connection handler** — hardcodes username/password/room instead of deserializing the first WebSocket frame via `common::ws_messages`.
- **Auth** — `server/src/auth/client.rs` always returns `true`.
- **Client** — `client/src/main.rs` is a stub; the previous TCP-based implementation is commented out.

## License

MIT — see [LICENSE](LICENSE).
