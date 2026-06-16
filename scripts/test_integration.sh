#!/bin/bash
set -e

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"

# 1. Build
echo "Building server..."
cargo build --bin server

# 2. Find free port
PORT=$(python3 -c "import socket; s=socket.socket(); s.bind(('127.0.0.1',0)); print(s.getsockname()[1]); s.close()")
echo "Using port $PORT"

# 3. Clean logs from previous runs
rm -f "$PROJECT_DIR/logs/server.log"

# 4. Launch server in background (logs go to logs/server.log via flexi_logger)
cd "$PROJECT_DIR"
cargo run --bin server -- --port "$PORT" >/dev/null 2>&1 &
SERVER_PID=$!

# Cleanup: always kill server on exit
cleanup() {
    echo "Stopping server (PID $SERVER_PID)..."
    kill $SERVER_PID 2>/dev/null || true
    wait $SERVER_PID 2>/dev/null || true
}
trap cleanup EXIT

# 5. Wait for server to be ready (10s max)
echo "Waiting for server to be ready..."
for i in $(seq 1 50); do
    python3 -c "import socket; s=socket.socket(); s.connect(('127.0.0.1',$PORT)); s.close()" 2>/dev/null && break
    sleep 0.2
done

# 6. Run pytest
echo "Running integration tests..."
PORT=$PORT pytest "$PROJECT_DIR/tests/" -v

echo "All tests passed!"
