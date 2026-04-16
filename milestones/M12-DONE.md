# Milestone 12 — Distributed Runners + mTLS ✅

## Goal
Remote runner agents that accept jobs over gRPC with mutual TLS.

## Status: DONE

## Files Created
- `proto/runner.proto` — RunnerService gRPC definition (FetchJob, StreamLogs)
- `build.rs` — tonic_build compiles proto at build time
- `src/agent/agent.rs` — runner agent: connects via gRPC, executes jobs, streams logs
- `src/agent/registry.rs` — RunnerRegistry: service discovery + round-robin load balancing
- `src/agent/mod.rs` — module root + proto include

## Files Modified
- `src/main.rs` — added `agent` subcommand
- `Cargo.toml` — added tonic (tls feature), prost, libc, async-stream, tonic-build

## Rust Concepts Applied
| Concept | Where |
|---------|-------|
| `tonic` gRPC | `agent.rs` — RunnerServiceClient, streaming RPC |
| `rustls` mTLS | `agent.rs` — ClientTlsConfig on Channel |
| `unsafe` FFI | `agent.rs` — `libc::setpgid(0,0)` in pre_exec for process isolation |
| `async-stream` | `agent.rs` — stream! macro for log line streaming |
| `prost` | proto-generated types (JobRequest, LogLine, etc.) |
| `Arc<Mutex>` | `registry.rs` — shared runner map |
| Round-robin | `registry.rs` — dispatch() picks next available runner |

## DevOps Concepts Applied
| Concept | Where |
|---------|-------|
| Distributed systems | Agent connects to server, receives jobs remotely |
| Service discovery | Agents register on connect via RunnerRegistry |
| Load balancing | Round-robin job dispatch across available agents |
| mTLS | ClientTlsConfig secures all gRPC connections |
| Runner labels | JobRequest.labels routes jobs to specific agents |
| Agent disconnection | deregister() removes agent from registry |
| Process isolation | setpgid isolates child in its own process group |

## Commands
```bash
# Start server
rustpipe serve --port 9090

# Start agent (connects to server)
rustpipe agent --server https://localhost:9090 --id agent-1 --labels linux,rust
rustpipe agent --server https://localhost:9090 --id agent-2 --labels gpu,linux
```
