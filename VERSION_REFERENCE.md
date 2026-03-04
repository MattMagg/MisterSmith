# Version Reference

Generated: 2026-03-03

This document maps every crate referenced in `spec/core-architecture/dependency-specifications.md` to its current stable version, identifies version gaps, and flags breaking changes that affect the framework specifications.

## Crate Versions

### Core Runtime & Async

| Crate | Spec Version | Current Stable | Version Gap | Breaking Changes | Migration Priority |
|-------|-------------|----------------|-------------|------------------|--------------------|
| tokio | 1.45.0 | 1.49.0 | Minor (4 releases) | None — SemVer compatible | Low |
| futures | 0.3.31 | 0.3.32 | Patch | None | None |
| async-trait | 0.1.83 | 0.1.83+ | Patch at most | None | None |
| pin-project | 1.1.6 | 1.1.x | Patch at most | None | None |

### Messaging & Transport

| Crate | Spec Version | Current Stable | Version Gap | Breaking Changes | Migration Priority |
|-------|-------------|----------------|-------------|------------------|--------------------|
| **async-nats** | **0.37.0** | **0.46.0** | **9 minor versions** | **Yes — significant** | **CRITICAL** |
| tonic | 0.11.0 | 0.14.5 | 3 minor versions | Yes — prost 0.14, hyper 1.x | High |
| prost | 0.12.0 | 0.14.3 | 2 minor versions | Yes — API changes | High |
| prost-build | 0.13.3 | 0.14.3 | 1 minor version | Yes — must match prost | High |
| axum | 0.8.0 | 0.8.8 | Patch (8 releases) | None — SemVer compatible | Low |
| hyper | (implicit) | 1.8.1 | N/A | Hyper 1.x is current | Low |
| tower | 0.5.1 | 0.5.3 | Patch | None | None |
| tower-http | 0.6.2 | 0.6.8 | Patch | None | None |
| reqwest | 0.12.9 | 0.13.2 | 1 minor version | Yes — rustls default, query/form features | Medium |

### Serialization & Data

| Crate | Spec Version | Current Stable | Version Gap | Breaking Changes | Migration Priority |
|-------|-------------|----------------|-------------|------------------|--------------------|
| serde | 1.0.214 | 1.0.228 | Patch (14 releases) | None | None |
| serde_json | 1.0.132 | 1.0.149 | Patch (17 releases) | None | None |
| toml | 0.8.19 | 0.8.x | Patch at most | None | None |

### Error Handling

| Crate | Spec Version | Current Stable | Version Gap | Breaking Changes | Migration Priority |
|-------|-------------|----------------|-------------|------------------|--------------------|
| **thiserror** | **1.0.69** | **2.0.18** | **Major version** | **Yes — MSRV bump, trait changes** | **High** |
| anyhow | 1.0.93 | 1.0.102 | Patch (9 releases) | None | None |

### Observability & Metrics

| Crate | Spec Version | Current Stable | Version Gap | Breaking Changes | Migration Priority |
|-------|-------------|----------------|-------------|------------------|--------------------|
| tracing | 0.1.41 | 0.1.44 | Patch (3 releases) | None | None |
| tracing-subscriber | 0.3.18 | 0.3.22 | Patch (4 releases) | None | None |
| metrics | 0.23.0 | 0.24.3 | 1 minor version | Yes — API changes likely | Medium |
| metrics-exporter-prometheus | 0.15.3 | 0.18.1 | 3 minor versions | Yes — must match metrics | Medium |
| prometheus | 0.13.4 | 0.14.0 | 1 minor version | Minor changes | Low |
| opentelemetry | 0.26.0 | 0.31.0 | 5 minor versions | Yes — breaking per-release | High |
| tracing-opentelemetry | 0.26.0 | 0.32.1 | 6 minor versions | Yes — must match opentelemetry | High |

### Security & Cryptography

| Crate | Spec Version | Current Stable | Version Gap | Breaking Changes | Migration Priority |
|-------|-------------|----------------|-------------|------------------|--------------------|
| ring | =0.17.8 | 0.17.14 | Patch (6 releases) | None — SemVer compatible | Low |
| **jsonwebtoken** | **9.3.0** | **10.3.0** | **Major version** | **Yes — crypto backend traits** | **High** |
| rustls | (implicit) | 0.23.37 | N/A | Now requires crypto backend selection | Medium |
| tokio-rustls | (implicit) | 0.26.4 | N/A | Aligned with rustls 0.23 | Medium |
| rcgen | (not in spec) | 0.14.7 | N/A | N/A | N/A |
| jwt-simple | =0.12.10 | 0.12.x | Patch at most | None | None |
| aes-gcm | =0.10.3 | 0.10.x | Patch at most | None | None |
| chacha20poly1305 | =0.10.1 | 0.10.x | Patch at most | None | None |

### Database & Persistence

| Crate | Spec Version | Current Stable | Version Gap | Breaking Changes | Migration Priority |
|-------|-------------|----------------|-------------|------------------|--------------------|
| sqlx | 0.8.2 | 0.8.6 | Patch (4 releases) | None | None |
| **redis** | **0.27.5** | **1.0.4** | **Major version** | **Yes — significant API changes** | **High** |
| sled | 0.34.7 | 0.34.7 | None | None (unmaintained) | Low |
| deadpool | (not in spec) | 0.13.0 | N/A | N/A | N/A |

### Utilities & Collections

| Crate | Spec Version | Current Stable | Version Gap | Breaking Changes | Migration Priority |
|-------|-------------|----------------|-------------|------------------|--------------------|
| uuid | 1.11.0 | 1.x | Patch at most | None | None |
| chrono | 0.4.38 | 0.4.x | Patch at most | None | None |
| indexmap | 2.6.0 | 2.x | Patch at most | None | None |
| dashmap | 6.1.0 | 6.x | Patch at most | None | None |
| parking_lot | 0.12.3 | 0.12.x | Patch at most | None | None |
| once_cell | 1.20.2 | 1.x | Patch at most | None | None |

### Infrastructure

| Component | Spec Version | Current Stable | Version Gap | Notes |
|-----------|-------------|----------------|-------------|-------|
| nats-server (Docker) | 2.12.4 | 2.12.4 | None | `latest` tag on Docker Hub points to 2.12.4 |
| Rust edition | 2021 | 2021 (2024 available) | Edition 2024 available | No migration needed yet |
| MSRV | 1.75 | — | async-nats 0.46.0 requires rustc 1.88.0 | **MSRV must increase** |

---

## Critical Migration Notes

### 1. async-nats 0.37.0 --> 0.46.0 (CRITICAL)

**Impact**: 9 minor versions spanning 12+ months of development. This is the highest-risk migration in the framework.

**Key breaking changes by version**:

#### v0.38.0 (from 0.37)
- **Auth struct signature field changed** to `Vec<u8>` (was previously a different type)
- Added Websockets support behind `websockets` feature flag (enabled by default)
- Added `drain` feature for subscriptions and connections

#### v0.35.0 (already before spec version, but sets context)
- Migrated to **rustls v0.23** with selectable crypto backend (`ring`, `aws-lc-rs`, `fips`)
- TLS configuration completely reworked

#### v0.36.0
- **`get_raw_message` reworked** to return `StreamMessage` instead of previous type
- `futures` dependency replaced with `futures-util`

#### v0.42.0
- Added **Client traits** for extensibility (orbit integration)

#### v0.43.0
- Added **nats-server 2.12 support**: PriorityPolicy, persistence mode, message counters/schedules
- Added **backpressure to publish** (significant behavioral change)
- `futures` fully replaced by `futures-util` in dependencies

#### v0.44.0
- **Reorganized `message` types** (type path changes)
- Service API enabled by default
- Extended client traits

#### v0.46.0 (current)
- **Feature-gated modules/features** — `jetstream`, `kv`, `object-store`, `service`, `nkeys`, `nuid`, `websockets`, `ring` are now behind feature flags (all enabled in `default`)
- `rustls-pki-types` replaces previous TLS cert type handling
- **Minimum rustc version raised to 1.88.0** (spec says 1.75)

**Spec files affected**:
- `spec/transport/nats-transport.md` — connection, publish/subscribe patterns
- `spec/data-management/message-schemas.md` — message types
- `spec/core-architecture/dependency-specifications.md` — version numbers, feature flags
- All JetStream, KV, and Object Store references

**Migration actions**:
1. Update all `async-nats` version references from `0.37.0` to `0.46.0`
2. Update feature flags — spec references `jetstream`, `kv`, `object_store`, `service` which are now feature-gated
3. Update MSRV from `1.75` to at least `1.88.0`
4. Review `Auth` struct usage for `Vec<u8>` signature field
5. Update `get_raw_message` call sites to use `StreamMessage` return type
6. Account for publish backpressure in agent communication patterns
7. Update message type paths after v0.44.0 reorganization

### 2. tonic 0.11.0 --> 0.14.5 (HIGH)

**Key breaking changes**:
- **Prost updated to v0.14** (was v0.12 in spec). Prost has been extracted to its own crates, changing codegen
- **Hyper 1.x integration** — tonic 0.12+ moved to hyper 1.x (the spec's implicit hyper dependency was already 1.x-compatible via axum 0.8)
- Transport layer changes for HTTP/2

**Migration actions**:
1. Update `tonic` from `0.11.0` to `0.14.5`
2. Update `prost` from `0.12.0` to `0.14.3`
3. Update `prost-build` from `0.13.3` to `0.14.3`
4. Review generated protobuf code for API changes
5. Update gRPC transport spec files

### 3. thiserror 1.0.69 --> 2.0.18 (HIGH)

**Key breaking changes**:
- **MSRV raised** to 1.61+ (not a problem if MSRV already moving to 1.88+)
- The `#[error(transparent)]` and `#[from]` attributes have minor behavioral changes
- `Display` formatting changes for some edge cases
- thiserror 2.0 supports `no_std` environments

**Decision required**: The spec explicitly notes `thiserror = "1.0.69" # (not 2.0)` in the cross-domain validation matrix. This was a deliberate choice. However, thiserror 1.0 is still available and maintained — the spec can stay on 1.0.x if desired, or adopt 2.0 for the `no_std` support and cleaner API. async-nats 0.46.0 still uses `thiserror = "1.0"` in its own dependencies, so 1.0.x remains viable.

**Recommendation**: Keep `thiserror = "1.0.69"` (or latest 1.0.x patch) for now. Migrate to 2.0 only when all workspace dependencies have moved. No urgency.

### 4. jsonwebtoken 9.3.0 --> 10.3.0 (HIGH)

**Key breaking changes**:
- **Crypto backend is now trait-based** — must choose between `aws_lc_rs` or `rust_crypto` feature
- No default crypto backend — explicit selection required
- `CryptoProvider` trait allows custom backends
- API changes to token encoding/decoding

**Migration actions**:
1. Update to `jsonwebtoken = { version = "10", features = ["aws_lc_rs"] }` (or `"rust_crypto"`)
2. Update all `encode`/`decode` call patterns in security specs
3. Review `EncodingKey`/`DecodingKey` API changes
4. Update `spec/security/` files referencing JWT handling

### 5. redis 0.27.5 --> 1.0.4 (HIGH)

**Key breaking changes**:
- **`FromRedisValue` now takes owned value** instead of reference (enables zero-copy deserialization)
- **`ToSingleRedisArg` trait introduced** — compile-time distinction between single and collection arguments
- **Iterator types now return `Result`** — `safe_iterators` is the default and only behavior
- **`async-std` runtime removed** — only Tokio supported (already the spec's choice)
- Improved error types and messages (may affect error matching)

**Migration actions**:
1. Update `redis` from `0.27.5` to `1.0.4`
2. Update feature flags — `tokio-comp` may have changed names
3. Update all `FromRedisValue` implementations
4. Review iterator usage for `Result` wrapping
5. Update `spec/data-management/` persistence specs

### 6. reqwest 0.12.9 --> 0.13.2 (MEDIUM)

**Key breaking changes**:
- **rustls is now the default TLS backend** (was native-tls)
- `query()` and `form()` methods now require feature flags (`query`, `form`)
- TLS configuration methods renamed (old names soft-deprecated)
- `native-tls` now implies ALPN automatically

**Migration actions**:
1. Update `reqwest` from `0.12.9` to `0.13.2`
2. Add `query` and `form` features explicitly: `reqwest = { version = "0.13", features = ["json", "stream", "gzip", "query"] }`
3. Review TLS configuration in HTTP client specs

### 7. OpenTelemetry Stack: 0.26.0 --> 0.31.0 (HIGH)

**Key breaking changes**:
- OpenTelemetry has breaking changes in every minor release
- `tracing-opentelemetry` must be version-matched with `opentelemetry`
- The `opentelemetry-jaeger` crate referenced in the spec (v0.25.0) is **deprecated** — replaced by OTLP exporters
- `opentelemetry-zipkin` (v0.24.0 in spec) similarly deprecated

**Migration actions**:
1. Update `opentelemetry` from `0.26.0` to `0.31.0`
2. Update `tracing-opentelemetry` from `0.26.0` to `0.32.1`
3. Replace `opentelemetry-jaeger` with `opentelemetry-otlp` + Jaeger OTLP endpoint
4. Replace `opentelemetry-zipkin` with `opentelemetry-otlp` + Zipkin OTLP endpoint
5. Update `spec/operations/` monitoring specs

### 8. metrics 0.23.0 --> 0.24.3 (MEDIUM)

**Key breaking changes**:
- API changes between 0.23 and 0.24 (metrics facade redesign)
- `metrics-exporter-prometheus` jumped from 0.15.3 to 0.18.1 (must match metrics version)

**Migration actions**:
1. Update `metrics` from `0.23.0` to `0.24.3`
2. Update `metrics-exporter-prometheus` from `0.15.3` to `0.18.1`
3. Review metric registration and recording patterns

---

## MSRV Impact Summary

The spec's MSRV of **1.75** is incompatible with the current async-nats:

| Crate | Required MSRV |
|-------|--------------|
| async-nats 0.46.0 | **1.88.0** |
| tokio 1.49.0 | 1.70 |
| axum 0.8.8 | 1.78.0 |
| tonic 0.14.5 | 1.75.0 |
| thiserror 2.0.18 | 1.61.0 |

**Recommendation**: Update MSRV to **1.88.0** (driven by async-nats requirement). This is the binding constraint.

---

## Summary: Migration Priority Matrix

| Priority | Crates | Effort | Risk |
|----------|--------|--------|------|
| **CRITICAL** | async-nats (0.37 -> 0.46) | High | High — touches transport, messaging, JetStream across many spec files |
| **HIGH** | tonic+prost (0.11+0.12 -> 0.14+0.14), jsonwebtoken (9 -> 10), redis (0.27 -> 1.0), opentelemetry stack (0.26 -> 0.31), thiserror (decision: stay 1.x or go 2.x) | Medium each | Medium — localized to specific domains |
| **MEDIUM** | reqwest (0.12 -> 0.13), metrics stack (0.23 -> 0.24) | Low | Low — mostly feature flag and import changes |
| **LOW** | tokio (1.45 -> 1.49), axum (0.8.0 -> 0.8.8), ring (0.17.8 -> 0.17.14), serde/serde_json, tracing, tower, futures | Minimal | Minimal — patch-level updates, SemVer compatible |

### Crates with No Action Required

These crates are either at or very near their spec versions with no breaking changes:
- serde, serde_json, toml, anyhow, futures, tracing, tracing-subscriber
- axum, tower, tower-http, hyper
- uuid, chrono, indexmap, dashmap, parking_lot, once_cell, smallvec
- ring, jwt-simple, aes-gcm, chacha20poly1305
- sqlx, sled
- nats-server Docker image (2.12.4 matches local environment)

---

## Version Matrix for Implementation

Recommended versions for `[workspace.dependencies]` at implementation time:

```toml
[workspace.dependencies]
# Core Runtime
tokio = { version = "1.49.0", features = ["full"] }
futures = "0.3.32"
async-trait = "0.1.83"

# Transport
async-nats = { version = "0.46.0", features = ["jetstream", "kv", "object-store", "service", "nkeys", "ring"] }
tonic = "0.14.5"
prost = "0.14.3"
axum = "0.8.8"

# Serialization
serde = { version = "1.0.228", features = ["derive"] }
serde_json = "1.0.149"

# Error Handling
thiserror = "1.0.69"  # Or 2.0.18 — decision pending
anyhow = "1.0.102"

# Observability
tracing = "0.1.44"
tracing-subscriber = { version = "0.3.22", features = ["env-filter"] }
metrics = "0.24.3"
metrics-exporter-prometheus = "0.18.1"
opentelemetry = "0.31.0"
tracing-opentelemetry = "0.32.1"

# Security
ring = "=0.17.14"
jsonwebtoken = { version = "10.3.0", features = ["aws_lc_rs"] }
rustls = "0.23.37"

# Database
sqlx = { version = "0.8.6", features = ["runtime-tokio-rustls", "any"] }
redis = { version = "1.0.4", features = ["tokio-comp", "connection-manager"] }

# HTTP Client
reqwest = { version = "0.13.2", features = ["json", "stream", "gzip", "query"] }

# Build
prost-build = "0.14.3"

# Tower
tower = "0.5.3"
tower-http = { version = "0.6.8", features = ["trace"] }
```

```toml
[package]
rust-version = "1.88"
```
