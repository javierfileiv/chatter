# AGENTS.md

## Workspace
Cargo workspace with 3 crates: `common` → `server` | `client`

## Code Rules
For structs enums and all those atuff, do not use final period in the line. Comment should be all in english languages, check that in the whole project.
I'm learning Rust so when you do something, explain it to me.

## Commands

```sh
# Build
cargo build --workspace

# Format (required by pre-commit & CI)
cargo fmt --all && cargo clippy --all-targets --all-features -- -D warnings

# Test all
cargo test --all

# Test one crate
cargo test -p common
cargo test -p server

# Run server (default 127.0.0.1:8080, override with --port or -p)
cargo run --bin server -- --port 3000
```

## Pre-commit hooks
Installed via `pre-commit` (Python). Setup: `pip install -r tools/requirements.txt && pre-commit install`. Hooks run `cargo fmt --all` and `cargo clippy --all-targets --all-features -- -D warnings`.

## Gotchas

- BEFORE start coding let me know what you do, I want to learn Rust not only make the app.
- **`server/src/core/DONT_COMMITbroker_test.rs`** — stale duplicate of broker code. Do not modify or un-ignore; keep it as-is.
- **Broker broadcast test is `#[ignore]`** — the broadcast path contains `todo!()` and will panic. Do not un-ignore until broadcast is implemented.
- **Auth is a stub** — `server/src/auth/client.rs:8` always returns `true`. Connection handler (`server/src/core/connection.rs`) uses hardcoded username/password/room instead of parsing the actual WebSocket message.
- **`common` crate defines message types** (`ws_messages.rs`) but the server connection handler does not yet deserialize incoming frames with them.
- **Several `todo!()` calls** in broker and connection handler — the server framework exists but room-join and broadcast message routing are incomplete.
- **No CI workflow file exists** despite README referencing `.github/workflows/rust-ci.yml`.
- **Logs directory** (`logs/`) is gitignored; server writes logs there via flexi_logger.

## Architecture

```
client/src/main.rs        → CLI stub (currently only prints "Hello, world!")
server/src/main.rs        → Tokio entrypoint, binds TcpListener, spawns connections
  server/src/core/server.rs → Accept loop with ctrl-c handling, spawns per-connection tasks
  server/src/core/broker.rs → Central event loop: manages rooms/clients via mpsc channels
  server/src/core/connection.rs → Per-connection WebSocket handler, auth + message loop
  server/src/auth/client.rs     → Stub authenticate(user, pass) always returns true
common/src/lib.rs         → Re-exports `errors` and `ws_messages`
common/src/ws_messages.rs → Serde structs: AuthenticateUser, SendMessage, Logout
common/src/errors.rs      → Unused error enum (prefixed with _)
```

Flow: `client` → WebSocket → `server` accept → `connection::handle` → authenticate → send `BrokerEvent::Connect` to `broker::run` event loop. Broker manages room membership and routes `BrokerEvent::Broadcast`/`JoinRoom` back to clients via `BrokerToClientMsg`.

The idea is, connection.rs handle when a client connects, a task to read websocket (half read websocket) is spawned in the server. This task will handle the deserialization of messages sent by the client, and send the BrokerEvent to the broker that will reply back using the internal mpsc unbounded channel. This unbounded channel is read by another spawned task, and reply back through the half write websocket.
