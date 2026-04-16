# Milestone 9 — Secret Masking + RBAC ✅

## Goal
Secure secret injection and role-based access control on the API.

## Status: DONE

## Files Created
- `rustpipe-macros/` — separate proc-macro crate (workspace member)
- `rustpipe-macros/src/lib.rs` — `#[requires_role("admin")]` attribute macro
- `src/secrets/mod.rs` — env-based secret loading, masking, hardcoded-value detection
- `src/server/rbac.rs` — AuthUser, role_allows(), auth_middleware, FromRequestParts extractor

## Files Modified
- `Cargo.toml` — converted to workspace, added rustpipe-macros path dep
- `src/server/mod.rs` — added rbac module, auth_middleware layer, admin_config route
- `src/cli/mod.rs` — secret loading + hardcoded-value warnings in run_pipeline()
- `src/pipeline/model.rs` — added `secrets` field to Pipeline

## Rust Concepts Applied
| Concept | Where |
|---------|-------|
| Procedural macro | `rustpipe-macros/src/lib.rs` — `#[requires_role]` attribute |
| `syn` + `quote` | macro crate — parse ItemFn, inject guard stmts |
| `proc-macro2` | macro crate — token stream manipulation |
| Axum middleware | `rbac.rs` — `auth_middleware` reads X-Role/X-User headers |
| `FromRequestParts` | `rbac.rs` — AuthUser extractor for handler params |
| Log filtering | `secrets/mod.rs` — `mask()` replaces secret values with *** |
| Env var injection | `secrets/mod.rs` — `env_pairs()` builds container env vars |

## DevOps Concepts Applied
| Concept | Where |
|---------|-------|
| Secrets management | Secrets loaded from RUSTPIPE_SECRET_<NAME> env vars |
| Secret masking | mask() redacts values from any log string |
| Hardcoded secret detection | check_hardcoded() warns if secret appears in step run: |
| RBAC / policy engine | viewer < operator < admin hierarchy |
| Audit logging | RBAC allow/deny logged via tracing |

## Usage
```bash
# Set secrets
export RUSTPIPE_SECRET_TOKEN=my-secret-value
rustpipe run  # warns if token appears hardcoded in any step

# RBAC (server mode)
curl -k -H "X-User: alice" -H "X-Role: admin" https://localhost:9090/api/v1/admin/config
curl -k -H "X-User: bob"   -H "X-Role: viewer" https://localhost:9090/api/v1/admin/config
# → 403 Forbidden
```
