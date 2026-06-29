# Stage 1: build the server binary
# Uses the official Rust image with all tooling needed to compile
FROM rust:1-slim-bookworm AS builder

# Set working directory — all following commands run in /app
WORKDIR /app

# Copy the entire workspace into the image
COPY . .

# Compile only the server crate in release mode
#   --release  : optimised binary (smaller, faster)
#   -p server  : skip building the client (avoids Cursive/TUI dependencies)
RUN cargo build --release -p server

# Stage 2: minimal runtime image
# Fresh Debian slim — no Rust, no compilers, nothing but the OS bare essentials
FROM debian:bookworm-slim

# Copy only the compiled binary from the builder stage
# The binary is statically linked to Rust libs — no extra runtime needed
COPY --from=builder /app/target/release/server /usr/local/bin/chatter-server

# Document the port the server listens on
EXPOSE 1234

# Default command when the container starts
ENTRYPOINT ["chatter-server"]
