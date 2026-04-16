# INSTALL.md — RustPipe Setup Guide

Everything you need to download, install, and run RustPipe from scratch.

---

## 1. System Requirements

| Requirement | Minimum Version | Notes |
|-------------|----------------|-------|
| OS | Linux / macOS / Windows (WSL2) | Ubuntu 22.04+ recommended |
| Rust | 1.75+ | Installed via rustup |
| Docker | 20.10+ | Only needed for `--docker` mode |
| protoc | 3.15+ | Required to compile gRPC proto files |
| libssl-dev | any | Required by reqwest/hyper-rustls |
| libsqlite3-dev | any | Required by sqlx |
| pkg-config | any | Required by build scripts |

---

## 2. Install Rust

```bash
# Install rustup (Rust toolchain manager)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Follow the on-screen prompts, then reload your shell
source "$HOME/.cargo/env"

# Verify
rustc --version   # should print rustc 1.75.0 or newer
cargo --version
```

---

## 3. Install System Dependencies

### Ubuntu / Debian

```bash
sudo apt update
sudo apt install -y \
    build-essential \
    pkg-config \
    libssl-dev \
    libsqlite3-dev \
    protobuf-compiler \
    docker.io

# Add your user to the docker group (avoids sudo for docker commands)
sudo usermod -aG docker $USER
newgrp docker
```

### macOS (Homebrew)

```bash
brew install protobuf openssl sqlite pkg-config
# Docker: install Docker Desktop from https://www.docker.com/products/docker-desktop
```

### Windows (WSL2)

```bash
# Inside WSL2 Ubuntu terminal — same as Ubuntu steps above
# Docker Desktop with WSL2 backend: https://docs.docker.com/desktop/windows/wsl/
```

---

## 4. Get the Code

### Option A — Clone the full repo

```bash
git clone https://github.com/your-username/Rust-DevOps-Intermediate.git
cd Rust-DevOps-Intermediate/building-blocks/02-rustpipe
```

### Option B — If you already have the folder

```bash
cd /path/to/02-rustpipe
```

---

## 5. Build

```bash
# Debug build (fast compile, slower binary)
cargo build

# Release build (optimized — use this for real runs)
cargo build --release
```

The first build downloads ~100 crates and compiles them. This takes **3–10 minutes** depending on your machine. Subsequent builds are incremental and fast.

The binary is at:
- Debug:   `./target/debug/rustpipe`
- Release: `./target/release/rustpipe`

### Optional: install to PATH

```bash
cargo install --path .
# Now you can run: rustpipe <command> from anywhere
```

---

## 6. Verify the Install

```bash
./target/release/rustpipe --version
./target/release/rustpipe --help
```

Expected output:
```
rustpipe 0.1.0
GitOps CI/CD pipeline engine

Usage: rustpipe <COMMAND>

Commands:
  validate  Validate a pipeline YAML or DSL file
  init      Scaffold a default .rustpipe.yml in the current directory
  run       Run a pipeline locally
  serve     Start the webhook + dashboard server (HTTPS)
  agent     Start a remote runner agent
  history   List last N pipeline runs from history
  logs      Print full logs for a past run
  cache     Manage artifact cache
```

---

## 7. Run Your First Pipeline

```bash
# Step 1: scaffold a default pipeline config
./target/release/rustpipe init

# Step 2: validate it
./target/release/rustpipe validate .rustpipe.yml

# Step 3: dry run — see what would execute without running anything
./target/release/rustpipe run --dry-run

# Step 4: actually run it (shell mode — no Docker needed)
./target/release/rustpipe run
```

---

## 8. Run with Docker Isolation

Make sure Docker is running:

```bash
docker info   # should print Docker version info, not an error
```

Then:

```bash
# Run each stage inside its own Docker container
./target/release/rustpipe run --docker

# Run with DAG-based parallel execution + Docker
./target/release/rustpipe run --docker --parallel
```

---

## 9. Run with Artifact Caching

```bash
# First run — computes hashes, writes cache markers
./target/release/rustpipe run --cache

# Second run — unchanged stages are skipped instantly
./target/release/rustpipe run --cache

# Clear the cache
./target/release/rustpipe cache clear
```

---

## 10. Start the Webhook Server + Dashboard

```bash
# Start HTTPS server on port 9090
./target/release/rustpipe serve --port 9090 --secret my-webhook-secret
```

Open in browser (accept the self-signed cert warning):
```
https://localhost:9090/dashboard
```

Test the API:
```bash
# List runs (self-signed cert — use -k to skip verification)
curl -k https://localhost:9090/api/v1/runs

# Prometheus metrics
curl -k https://localhost:9090/metrics

# Admin route (requires X-Role: admin header)
curl -k -H "X-User: alice" -H "X-Role: admin" https://localhost:9090/api/v1/admin/config

# Viewer route (any role)
curl -k -H "X-User: bob" -H "X-Role: viewer" https://localhost:9090/api/v1/runs
```

### Configure GitHub Webhook

1. Go to your GitHub repo → Settings → Webhooks → Add webhook
2. Payload URL: `https://your-server:9090/webhook/github`
3. Content type: `application/json`
4. Secret: same value as `--secret` flag
5. Events: `push`

---

## 11. Use Secrets

```bash
# Set secrets as env vars (prefix: RUSTPIPE_SECRET_)
export RUSTPIPE_SECRET_TOKEN=ghp_abc123
export RUSTPIPE_SECRET_DB_PASSWORD=supersecret

# Run — secrets are injected as env vars into container stages
# and masked in all log output
./target/release/rustpipe run --docker
```

---

## 12. View Run History

```bash
# List last 10 runs
./target/release/rustpipe history

# List last 25 runs
./target/release/rustpipe history --limit 25

# View stored logs for run #1
./target/release/rustpipe logs 1
```

---

## 13. Start a Remote Runner Agent (gRPC)

In one terminal — start the server:
```bash
./target/release/rustpipe serve --port 9090
```

In another terminal — start an agent:
```bash
./target/release/rustpipe agent \
    --server https://localhost:9090 \
    --id agent-1 \
    --labels linux,rust
```

Multiple agents can connect simultaneously. Jobs are dispatched round-robin.

---

## 14. Validate a DSL Pipeline

```bash
# Create a .rustpipe DSL file
cat > my-pipeline.rustpipe << 'EOF'
pipeline my-pipeline

stage lint
  runs-on rust:latest
  step "Run clippy"
    run cargo clippy -- -D warnings
  end
end

stage test
  runs-on rust:latest
  needs lint
  step "Run tests"
    run cargo test --all
  end
end
EOF

# Validate it
./target/release/rustpipe validate my-pipeline.rustpipe

# Run it
./target/release/rustpipe run --pipeline my-pipeline.rustpipe
```

---

## 15. Environment Variables Reference

| Variable | Description | Example |
|----------|-------------|---------|
| `RUSTPIPE_SECRET_<NAME>` | Inject a secret by name | `RUSTPIPE_SECRET_TOKEN=abc` |
| `RUSTPIPE_BRANCH` | Current branch (used for `when:` conditions) | `RUSTPIPE_BRANCH=main` |
| `RUST_LOG` | Tracing log level filter | `RUST_LOG=rustpipe=debug` |

---

## 16. Troubleshooting

### `error: linker 'cc' not found`
```bash
sudo apt install build-essential
```

### `error: failed to run custom build command for 'openssl-sys'`
```bash
sudo apt install libssl-dev pkg-config
```

### `error: failed to run custom build command for 'libsqlite3-sys'`
```bash
sudo apt install libsqlite3-dev
```

### `error: could not find protoc`
```bash
sudo apt install protobuf-compiler
# Verify: protoc --version
```

### `permission denied while connecting to Docker`
```bash
sudo usermod -aG docker $USER
newgrp docker
# Or log out and back in
```

### `TLS certificate error in browser`
The server uses a self-signed certificate generated at runtime. Click "Advanced → Proceed" in Chrome/Firefox, or use `curl -k` to skip verification.

### `cargo build` fails with `error[E0XXX]`
Make sure your Rust toolchain is up to date:
```bash
rustup update stable
```

---

## 17. Full Example Session

```bash
# 1. Install deps (Ubuntu)
sudo apt install -y build-essential pkg-config libssl-dev libsqlite3-dev protobuf-compiler

# 2. Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source "$HOME/.cargo/env"

# 3. Clone and build
git clone https://github.com/your-username/Rust-DevOps-Intermediate.git
cd Rust-DevOps-Intermediate/building-blocks/02-rustpipe
cargo build --release

# 4. Init and validate
./target/release/rustpipe init
./target/release/rustpipe validate .rustpipe.yml

# 5. Dry run
./target/release/rustpipe run --dry-run

# 6. Run pipeline
./target/release/rustpipe run

# 7. Check history
./target/release/rustpipe history

# 8. Start server (new terminal)
./target/release/rustpipe serve --port 9090

# 9. Open dashboard
xdg-open https://localhost:9090/dashboard
```
