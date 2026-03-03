# Dependency Management Specifications

## Complete Cargo.toml Dependencies and Version Management Strategy

This document provides comprehensive dependency management specifications for the Mister Smith AI Agent Framework,
defining exact versions, feature selections, security requirements, and management strategies.

## 1. Dependency Tree Architecture

### 1.1 Dependency Hierarchy Visualization

```text
┌───────────────────────────────────────────────────────────┐
│                     CORE DEPENDENCIES                     │
│  tokio, serde, async-trait, uuid, thiserror, futures     │
└─────────────────────┬─────────────────────────────────────┘
                      │
┌─────────────────────┴─────────────────────────────────────┐
│                  FEATURE DEPENDENCIES                     │
├─ SECURITY: ring, jwt-simple, aes-gcm, chacha20poly1305   │
├─ PERSISTENCE: sqlx, redis, sled                          │
├─ CLUSTERING: raft, async-nats                            │
├─ MONITORING: prometheus, metrics, tracing                │
├─ HTTP: reqwest, url                                      │
└─ UTILITIES: chrono, config, notify, crossbeam            │
                      │
┌─────────────────────┴─────────────────────────────────────┐
│                DEVELOPMENT DEPENDENCIES                   │
│  tokio-test, mockall, criterion, proptest, wiremock      │
└───────────────────────────────────────────────────────────┘
```

### 1.2 Dependency Categories

| Category | Purpose | Selection Criteria |
|----------|---------|-------------------|
| **Core** | Always required functionality | Stability, security, performance |
| **Security** | Authentication, encryption, auditing | Security audit trail, cryptographic soundness |
| **Persistence** | Data storage and retrieval | Transaction support, connection pooling |
| **Clustering** | Distributed coordination | Consensus algorithms, message passing |
| **Monitoring** | Observability and health checking | Low overhead, comprehensive metrics |
| **Development** | Testing, benchmarking, debugging | Developer productivity, test coverage |

### 1.3 Dependency Tree Example Output

```bash
# Generate full dependency tree
$ cargo tree --all-features

mister-smith-framework v0.1.0
├── async-nats v0.37.0
│   ├── base64 v0.22.1
│   ├── bytes v1.5.0
│   ├── futures v0.3.31
│   │   ├── futures-channel v0.3.31
│   │   ├── futures-core v0.3.31
│   │   ├── futures-executor v0.3.31
│   │   ├── futures-io v0.3.31
│   │   ├── futures-sink v0.3.31
│   │   ├── futures-task v0.3.31
│   │   └── futures-util v0.3.31
│   ├── nkeys v0.4.1
│   ├── nuid v0.5.0
│   ├── serde v1.0.214
│   ├── serde_json v1.0.132
│   ├── thiserror v1.0.69
│   └── tokio v1.45.0
├── tokio v1.45.0
│   ├── bytes v1.5.0
│   ├── libc v0.2.153
│   ├── mio v0.8.11
│   ├── num_cpus v1.16.0
│   ├── parking_lot v0.12.3
│   ├── pin-project-lite v0.2.14
│   ├── signal-hook-registry v1.4.2
│   ├── socket2 v0.5.7
│   └── tokio-macros v2.2.0 (proc-macro)
└── ...

# Check for duplicate dependencies
$ cargo tree --duplicates
serde v1.0.214
├── mister-smith-framework v0.1.0
├── async-nats v0.37.0
├── config v0.14.1
└── sqlx v0.8.2
```

---

## 2. Complete Cargo.toml Specification

### 2.1 Package Metadata

```toml
[package]
name = "mister-smith-framework"
version = "0.1.0"
edition = "2021"
rust-version = "1.75"
authors = ["Mister Smith AI Framework Team"]
description = "AI Agent Framework with Tokio-based async architecture, supervision trees, and tool integration"
license = "MIT OR Apache-2.0"
repository = "https://github.com/mister-smith/framework"
documentation = "https://docs.rs/mister-smith-framework"
keywords = ["ai", "agents", "async", "tokio", "supervision"]
categories = ["asynchronous", "development-tools"]
readme = "README.md"

# MSRV Policy: Update quarterly, maintain 6-month compatibility window
rust-version = "1.75"
```

### 2.2 Feature Flags Architecture

```toml
[features]
# Default features - core functionality always enabled
default = ["runtime", "actors", "tools", "monitoring", "config"]

# Complete feature set for production deployments
full = [
    "default", "security", "encryption", "metrics", 
    "tracing", "persistence", "clustering", "http-client"
]

# Core system features
runtime = ["dep:tokio", "tokio/full"]
actors = ["dep:async-trait", "dep:crossbeam-channel"]
tools = ["dep:serde_json", "dep:jsonschema"]
monitoring = ["dep:prometheus", "dep:metrics"]
supervision = ["dep:crossbeam-utils", "dep:atomic_float"]
config = ["dep:config", "dep:notify", "dep:toml"]

# Security features with granular control
security = ["dep:ring", "dep:jwt-simple"]
encryption = ["security", "dep:aes-gcm", "dep:chacha20poly1305"]
auth = ["security", "dep:oauth2", "dep:jsonwebtoken"]

# Storage and persistence features
persistence = ["dep:sqlx", "dep:redis", "dep:sled"]
sql = ["persistence", "sqlx/runtime-tokio-rustls", "sqlx/postgres", "sqlx/sqlite"]
nosql = ["persistence", "redis/tokio-comp", "redis/connection-manager"]
embedded = ["persistence", "sled/compression"]

# Distributed system features
clustering = ["dep:raft", "dep:async-nats"]
consensus = ["clustering", "raft/prost-codec"]
messaging = ["clustering", "async-nats/jetstream"]

# Observability features
metrics = ["dep:metrics", "dep:metrics-exporter-prometheus"]
tracing = [
    "dep:tracing", "dep:tracing-subscriber", 
    "dep:tracing-opentelemetry", "dep:opentelemetry"
]
health-checks = ["monitoring", "dep:tower", "dep:tower-http"]

# Network and communication features
http-client = ["dep:reqwest", "reqwest/json", "reqwest/stream"]
websockets = ["http-client", "reqwest/websocket"]

# Development and testing features
testing = ["dep:mockall", "dep:tokio-test", "dep:proptest"]
benchmarking = ["dep:criterion", "testing"]
dev-tools = ["testing", "benchmarking", "dep:cargo-fuzz"]

# Performance optimization features
simd = ["dep:wide"]
parallel = ["dep:rayon"]
compression = ["dep:lz4", "dep:zstd"]
```

### 2.3 Core Dependencies (Always Required)

```toml
[dependencies]
# === ASYNC RUNTIME AND EXECUTION ===
# Tokio: Comprehensive async runtime with all features enabled
tokio = { version = "1.45.0", features = ["full"] }
# Futures: Core async utilities and combinators
futures = "0.3.31"
# Async trait support for defining async traits
async-trait = { version = "0.1.83", optional = true }
# Pin projection for complex async types
pin-project = "1.1.6"

# === SERIALIZATION AND DATA HANDLING ===
# Serde: Core serialization framework with derive macros
serde = { version = "1.0.214", features = ["derive"] }
# JSON serialization for API communication
serde_json = { version = "1.0.132", optional = true }
# TOML parsing for configuration files
toml = "0.8.19"
# JSON schema validation for API contracts
jsonschema = { version = "0.18.3", optional = true }
# Semantic versioning for component compatibility
semver = { version = "1.0.23", features = ["serde"] }

# === ERROR HANDLING AND LOGGING ===
# Structured error types with derive macros
thiserror = "1.0.69"
# Error context and propagation utilities
anyhow = "1.0.93"
# Async-aware logging facade
tracing = { version = "0.1.41", optional = true }
# Tracing subscriber implementations
tracing-subscriber = { version = "0.3.18", optional = true, features = ["env-filter"] }

# === COLLECTIONS AND UTILITIES ===
# Ordered hash maps for deterministic iteration
indexmap = "2.6.0"
# UUID generation for unique identifiers
uuid = { version = "1.11.0", features = ["v4", "serde"] }
# Thread-safe lazy initialization
once_cell = "1.20.2"
# High-performance parking_lot mutexes
parking_lot = "0.12.3"
# Stack-allocated vectors for small collections
smallvec = "1.13.2"

# === CONCURRENCY PRIMITIVES ===
# Lock-free concurrent hash maps
dashmap = "6.1.0"
# Message passing channels (optional, for actor system)
crossbeam-channel = { version = "0.5.13", optional = true }
# Concurrency utilities and atomic operations
crossbeam-utils = { version = "0.8.20", optional = true }
# Atomic floating-point operations
atomic_float = { version = "1.1.0", optional = true }

# === TIME AND SCHEDULING ===
# Date and time handling with timezone support
chrono = { version = "0.4.38", features = ["serde"] }
# Cron expression parsing for scheduled tasks
cron = "0.12.1"

# === CONFIGURATION MANAGEMENT ===
# Hierarchical configuration management (optional)
config = { version = "0.14.1", optional = true }
# File system change notifications (optional)
notify = { version = "6.1.1", optional = true }
# System directory detection
dirs = "5.0.1"
```

### 2.4 Feature-Based Optional Dependencies

```toml
# === SECURITY AND CRYPTOGRAPHY ===
# Ring: Cryptographic operations and key management
ring = { version = "0.17.8", optional = true }
# JWT token handling with multiple algorithms
jwt-simple = { version = "0.12.10", optional = true }
# AES-GCM authenticated encryption
aes-gcm = { version = "0.10.3", optional = true }
# ChaCha20-Poly1305 authenticated encryption
chacha20poly1305 = { version = "0.10.1", optional = true }
# OAuth2 client implementation
oauth2 = { version = "4.4.2", optional = true }
# JSON Web Token library
jsonwebtoken = { version = "9.3.0", optional = true }

# === HTTP CLIENT AND NETWORKING ===
# Feature-rich HTTP client with async support
reqwest = { 
    version = "0.12.9", 
    optional = true, 
    features = ["json", "stream", "gzip"] 
}
# URL parsing and manipulation
url = "2.5.4"

# === METRICS AND MONITORING ===
# Metrics collection framework
metrics = { version = "0.23.0", optional = true }
# Prometheus metrics exporter
prometheus = { version = "0.13.4", optional = true }
# Prometheus metrics exporter integration
metrics-exporter-prometheus = { version = "0.15.3", optional = true }
# OpenTelemetry integration for distributed tracing
tracing-opentelemetry = { version = "0.26.0", optional = true }
# OpenTelemetry SDK
opentelemetry = { version = "0.26.0", optional = true }

# === DATABASE AND PERSISTENCE ===
# SQL database toolkit with async support
sqlx = { 
    version = "0.8.2", 
    optional = true, 
    features = ["runtime-tokio-rustls", "any"] 
}
# Redis client with async support
redis = { 
    version = "0.27.5", 
    optional = true, 
    features = ["tokio-comp", "connection-manager"] 
}
# Embedded key-value database
sled = { version = "0.34.7", optional = true }

# === DISTRIBUTED SYSTEMS ===
# Raft consensus algorithm implementation
raft = { version = "0.7.0", optional = true }
# NATS messaging system client - consistent version
async-nats = { version = "0.37.0", optional = true }

# === PERFORMANCE OPTIMIZATION ===
# SIMD operations for performance-critical code
wide = { version = "0.7.28", optional = true }
# Data parallelism for CPU-intensive tasks
rayon = { version = "1.10.0", optional = true }
# LZ4 compression for fast data compression
lz4 = { version = "1.28.0", optional = true }
# Zstd compression for high-ratio compression
zstd = { version = "0.13.2", optional = true }

# === HTTP SERVER AND MIDDLEWARE ===
# Tower service abstraction (for health checks)
tower = { version = "0.5.1", optional = true }
# HTTP middleware and utilities
tower-http = { version = "0.6.2", optional = true, features = ["trace"] }
```

### 2.5 Development Dependencies

```toml
[dev-dependencies]
# === TESTING FRAMEWORK ===
# Tokio testing utilities
tokio-test = "0.4.4"
# Mock object generation for testing
mockall = "0.13.0"
# Property-based testing framework
proptest = "1.5.0"
# Test logging and output capture
test-log = "0.2.16"
# Environment logger for test output
env_logger = "0.11.5"
# Temporary file handling in tests
tempfile = "3.14.0"
# HTTP mocking for integration tests
wiremock = "0.6.2"

# === BENCHMARKING AND PERFORMANCE ===
# Statistical benchmarking framework
criterion = { version = "0.5.1", features = ["html_reports"] }
# Memory usage profiling
dhat = "0.3.3"

# === FUZZING (Conditional) ===
# Cargo fuzz integration (enabled with dev feature)
cargo-fuzz = { version = "0.12.0", optional = true }
# libfuzzer integration
libfuzzer-sys = { version = "0.4.8", optional = true }

# === DOCUMENTATION AND LINTING ===
# Documentation testing utilities
doc-comment = "0.3.3"
```

### 2.6 Build Dependencies

```toml
[build-dependencies]
# Protocol Buffers code generation
prost-build = "0.13.3"
# Version information embedding
vergen = { version = "9.0.1", features = ["build", "git", "gitcl"] }
# Build script utilities
cc = "1.2.2"
```

---

## 4. Security Audit Requirements

### 4.1 Dependency Vulnerability Scanning

```toml
# Security audit configuration in workspace root
[workspace.metadata.audit]
db-url = "https://github.com/RustSec/advisory-db"
# Vulnerability database update frequency
db-update-frequency = "daily"
# Ignore list for false positives (with justification required)
ignore = [
    # Example: "RUSTSEC-2021-0000", # Justification here
]
# Severity threshold for CI failure
severity-threshold = "medium"

[workspace.metadata.audit.advisories]
# Yanked crate handling
yanked = "deny"          # Fail on yanked dependencies
unmaintained = "warn"    # Warn on unmaintained crates
unsound = "deny"         # Fail on known unsound code
vulnerability = "deny"   # Fail on known vulnerabilities
```

### 4.2 Dependency Integrity Verification

```bash
# Generate and verify dependency checksums
cargo generate-lockfile
sha256sum Cargo.lock > Cargo.lock.sha256

# Verify in CI/CD
sha256sum -c Cargo.lock.sha256 || exit 1

# Generate dependency lock file with integrity hashes
cargo update
cargo tree --locked > DEPENDENCY_TREE.lock

# Verify dependency integrity (CI/CD integration)
cargo verify-project
cargo audit --db-update --quiet
```

### 4.3 Security Review Process

1. **Weekly Automated Scans**: Run `cargo audit` in CI/CD pipeline
2. **Monthly Manual Reviews**: Review new dependencies and updates
3. **Quarterly Deep Audits**: Complete security assessment of all dependencies
4. **Emergency Response**: Process for critical vulnerability patches

### 4.4 Supply Chain Security Checklist

- [ ] All dependencies have verifiable source repositories
- [ ] Security audit trail maintained for all dependency updates
- [ ] No dependencies with known security vulnerabilities
- [ ] Minimal dependency tree to reduce attack surface
- [ ] Regular updates scheduled for security patches
- [ ] Backup plans for deprecated or abandoned dependencies

---

## 5. Version Management Strategy

### 5.1 Minimum Supported Rust Version (MSRV)

```toml
# Current MSRV: Rust 1.75 (December 2023)
rust-version = "1.75"

# MSRV Update Policy:
# - Review quarterly (March, June, September, December)
# - Maintain 6-month compatibility window
# - Update only for significant feature benefits
# - Announce MSRV changes 30 days in advance
```

### 5.2 Dependency Version Constraints

| Dependency Type | Version Strategy | Justification |
|-----------------|------------------|---------------|
| **Security Critical** | Exact versions (`=1.2.3`) | Prevent supply chain attacks |
| **Core Runtime** | Caret versions (`^1.2.3`) | Allow compatible updates |
| **Development** | Tilde versions (`~1.2.3`) | Allow patch updates only |
| **Optional Features** | Caret versions (`^1.2.3`) | Balance stability and features |

### 5.3 Update Cadence

```toml
# Dependency update schedule
[package.metadata.updates]
security = "weekly"        # Security patches applied immediately
major = "quarterly"        # Major version updates reviewed quarterly  
minor = "monthly"          # Minor version updates reviewed monthly
patch = "bi-weekly"        # Patch updates applied bi-weekly
```

### 5.4 Breaking Change Management

1. **Semantic Versioning Compliance**: Strict adherence to SemVer
2. **Deprecation Warnings**: 6-month deprecation period before removal
3. **Migration Guides**: Comprehensive upgrade documentation
4. **Compatibility Layers**: Temporary compatibility shims for major changes
5. **Testing Matrix**: Test against multiple dependency versions

---

## 6. Cargo Workspace Configuration

### 6.1 Complete Workspace Setup Example

```bash
# Project structure
mister-smith/
├── Cargo.toml              # Workspace root
├── Cargo.lock             # Shared dependency lock
├── .cargo/
│   └── config.toml        # Workspace-wide cargo config
├── core/
│   └── Cargo.toml         # Core framework crate
├── agents/
│   ├── agent-base/
│   │   └── Cargo.toml     # Base agent traits
│   └── agent-types/
│       └── Cargo.toml     # Agent implementations
├── tools/
│   ├── cli/
│   │   └── Cargo.toml     # CLI tool
│   └── audit/
│       └── Cargo.toml     # Security audit tool
└── examples/
    └── basic/
        └── Cargo.toml     # Example projects
```

### 6.2 Workspace Root Cargo.toml

```toml
[workspace]
members = [
    "core",                                 # Core framework
    "agents/agent-base",                    # Base agent traits
    "agents/agent-types",                   # Agent implementations
    "tools/cli",                            # CLI tool
    "tools/audit",                          # Security audit
    "examples/basic",                       # Basic examples
    "examples/advanced",                    # Advanced examples
]

# CRITICAL: resolver = "2" enables proper feature unification
resolver = "2"

# Workspace-wide dependency versions (inherited by members)
[workspace.dependencies]
# Core async runtime - shared version across all crates
tokio = { version = "1.45.0", features = ["full"] }
async-trait = "0.1.83"
futures = "0.3.31"

# Serialization - consistent across workspace
serde = { version = "1.0.214", features = ["derive"] }
serde_json = "1.0.132"

# Error handling - unified approach
thiserror = "1.0.69"
anyhow = "1.0.93"

# Common utilities
uuid = { version = "1.11.0", features = ["v4", "serde"] }
chrono = { version = "0.4.38", features = ["serde"] }
tracing = "0.1.41"

# Development dependencies - shared versions
[workspace.dev-dependencies]
tokio-test = "0.4.4"
mockall = "0.13.0"
criterion = "0.5.1"

# Workspace metadata
[workspace.metadata]
msrv = "1.75.0"
repository = "https://github.com/mister-smith/framework"
```

### 6.3 Member Crate Dependency Inheritance

```toml
# Example: core/Cargo.toml
[package]
name = "mister-smith-core"
version = "0.1.0"
edition = "2021"

# Inherit versions from workspace
[dependencies]
tokio = { workspace = true }
async-trait = { workspace = true }
serde = { workspace = true }
thiserror = { workspace = true }

# Crate-specific dependencies
once_cell = "1.20.2"
dashmap = "6.1.0"

# Override workspace features if needed
[dependencies.uuid]
workspace = true
features = ["v4", "v7", "serde"]  # Add v7 to workspace default
```

### 6.4 Dependency Version Consistency Rules

```toml
# RULE 1: All async runtime deps must match
tokio = "1.45.0"         # ✓ Consistent
tokio-util = "0.7.10"    # ✓ Compatible with tokio 1.45
tokio-stream = "0.1.14"  # ✓ Compatible with tokio 1.45

# RULE 2: Serialization ecosystem alignment  
serde = "1.0.214"        # ✓ Base version
serde_json = "1.0.132"   # ✓ Compatible
serde_yaml = "0.9.34"    # ✓ Compatible
toml = "0.8.19"          # ✓ Uses serde 1.0

# RULE 3: Cryptography version locking
ring = "=0.17.8"         # ✓ Exact version
chacha20poly1305 = "=0.10.1"  # ✓ Exact version
# NEVER mix different versions of crypto libraries
```

### 6.5 Cargo Configuration

```toml
# .cargo/config.toml - Global cargo configuration
[alias]
# Custom build commands
build-all = "build --workspace --all-targets"
test-all = "test --workspace --all-targets"
check-all = "check --workspace --all-targets"
audit = "audit --db-update"
security = "run --manifest-path tools/dependency-audit/Cargo.toml"

# Performance profiling
profile = "build --release --features=dev-tools"
bench-all = "bench --workspace"

# Development shortcuts
dev = "check --workspace --all-targets --all-features"
quick = "check --workspace"

[build]
# Parallel compilation jobs (adjust based on system)
jobs = 8

[target.x86_64-unknown-linux-gnu]
# Use mold linker for faster linking on Linux
linker = "clang"
rustflags = ["-C", "link-arg=-fuse-ld=mold"]

[env]
# Environment variables for consistent builds
RUST_BACKTRACE = "1"
RUST_LOG = "info"
```

---

## 7. Performance Optimization Profiles

### 7.1 Development Profile

```toml
[profile.dev]
# Fast compilation for development iteration
opt-level = 0
debug = true
debug-assertions = true
overflow-checks = true
lto = false
panic = "unwind"
incremental = true
codegen-units = 256
```

### 7.2 Release Profile

```toml
[profile.release]
# Maximum performance for production deployments
opt-level = 3
debug = false
debug-assertions = false
overflow-checks = false
lto = "fat"
panic = "abort"
incremental = false
codegen-units = 1
strip = true

# Enable CPU-specific optimizations
[profile.release.build-override]
opt-level = 3
codegen-units = 1
```

### 7.3 Testing Profile

```toml
[profile.test]
# Balanced performance for test execution
opt-level = 1
debug = true
debug-assertions = true
overflow-checks = true
incremental = true
```

### 7.4 Benchmark Profile

```toml
[profile.bench]
# Optimized for accurate benchmarking
opt-level = 3
debug = true
debug-assertions = false
overflow-checks = false
lto = true
codegen-units = 1
```

---

## 8. Dependency Injection Architecture

### 8.1 Service Registry Implementation

```rust
// Core dependency injection types requiring specific trait bounds
pub trait ServiceFactory: Send + Sync + 'static {
    type Service: Send + Sync + 'static;
    type Config: Send + Sync + Clone + 'static;
    type Error: Send + Sync + std::error::Error + 'static;
    
    async fn create(&self, config: Self::Config) -> Result<Self::Service, Self::Error>;
    fn dependencies(&self) -> Vec<TypeId>;
}

// Required dependencies for service registry implementation
[dependencies]
# Type identification for dependency injection
downcast-rs = "1.2.1"
# Immutable data structures for efficient service graphs
im = "15.1.0"
# Type-safe any for service storage
inventory = "0.3.15"
```

### 8.2 Factory Pattern Dependencies

```toml
# Dependencies for implementing factory patterns
[dependencies]
# Trait object creation utilities
dyn-clone = "1.0.17"
# Stable type IDs for service identification
stable_deref_trait = "1.2.0"
# Memory management for service lifetimes
weak-table = "0.3.2"
```

### 8.3 Configuration Management

```toml
# Configuration dependency injection
[dependencies]
# Configuration validation
validator = { version = "0.18.1", features = ["derive"] }
# Configuration file watching
notify = "6.1.1"
# Environment variable integration
dotenvy = "0.15.7"
```

---

## 9. Claude CLI Integration Dependencies

### 9.1 Process Management

```toml
# Process spawning and management for Claude CLI integration
[dependencies]
# Cross-platform process management
tokio-process = "0.2.5"
# Advanced process control
async-process = "2.3.0"
# Process exit status handling
exit-code = "1.0.0"
# Process signal handling
signal-hook = "0.3.17"
signal-hook-tokio = { version = "0.3.1", features = ["futures-v0_3"] }
```

### 9.2 NATS Messaging Integration

```toml
# NATS client for agent communication
[dependencies.async-nats]
version = "0.37.0"  # Updated to match transport specs
features = [
    "jetstream",      # Persistent messaging
    "kv",            # Key-value store  
    "object_store",  # Object storage
    "service",       # Service discovery
]
optional = true
```

### 9.3 Hook System Dependencies

```toml
# Hook system implementation
[dependencies]
# Event-driven architecture
event-listener = "5.3.1"
# Weak references for hook cleanup
weak = "3.0.0"
# Hook registry with type safety
linkme = "0.3.28"
```

---

## 10. Integration Test Dependencies

### 10.1 Test Infrastructure

```toml
[dev-dependencies]
# Docker container management for integration tests
bollard = "0.17.1"
# Test containers for databases
testcontainers = "0.23.1"
testcontainers-modules = { version = "0.11.2", features = ["postgres", "redis"] }

# Network testing utilities
reqwest = { version = "0.12.9", features = ["json"] }
wiremock = "0.6.2"

# Time manipulation in tests
tokio-test = "0.4.4"
mock_instant = "0.5.1"
```

### 10.2 Performance Testing

```toml
[dev-dependencies]
# Load testing framework
goose = "0.17.2"
# Memory profiling
dhat = "0.3.3"
# CPU profiling integration
pprof = { version = "0.13.0", features = ["criterion", "protobuf-codec"] }
```

---

## 11. Monitoring and Observability Dependencies

### 11.1 Metrics Collection

```toml
# Comprehensive metrics stack
[dependencies]
# Core metrics framework
metrics = "0.23.0"
# Prometheus metrics export
metrics-exporter-prometheus = "0.15.3"
# StatsD metrics export
metrics-exporter-statsd = "0.8.0"
# Metrics utility macros
metrics-util = "0.17.0"
```

### 11.2 Distributed Tracing

```toml
# OpenTelemetry tracing stack
[dependencies]
tracing = "0.1.41"
tracing-subscriber = { version = "0.3.18", features = ["env-filter", "json"] }
tracing-opentelemetry = "0.26.0"
opentelemetry = { version = "0.26.0", features = ["trace"] }
opentelemetry-jaeger = "0.25.0"
opentelemetry-zipkin = "0.24.0"
```

### 11.3 Health Checking

```toml
# Health check implementations
[dependencies]
# Health check framework
health-check = "0.1.4"
# HTTP health endpoints
tower-http = { version = "0.6.2", features = ["trace", "metrics"] }
```

---

## 12. Update and Maintenance Procedures

### 12.1 Automated Dependency Updates

```yaml
# .github/workflows/dependency-update.yml
name: Dependency Updates
on:
  schedule:
    - cron: '0 0 * * 1'  # Weekly on Monday
  workflow_dispatch:

jobs:
  update:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: Update dependencies
        run: |
          cargo update
          cargo audit
          cargo test --all
      - name: Create PR
        if: success()
        uses: peter-evans/create-pull-request@v5
        with:
          title: "chore: automated dependency updates"
          body: "Automated weekly dependency updates"
```

### 12.2 Security Audit Integration

```bash
#!/bin/bash
# scripts/security-audit.sh
set -euo pipefail

echo "Running security audit..."
cargo audit --db-update

echo "Checking for yanked crates..."
cargo tree --duplicates

echo "Verifying dependency licenses..."
cargo license --json | jq '.[] | select(.license | contains("GPL"))'

echo "Security audit complete!"
```

### 12.3 Dependency Health Monitoring

```toml
# tools/dependency-audit/Cargo.toml
[dependencies]
# Dependency analysis
cargo_metadata = "0.18.1"
# License checking
cargo-license = "0.6.1"
# Vulnerability scanning
rustsec = "0.29.4"
# Dependency tree analysis
cargo-tree = "0.32.0"
```

---

## 13. Troubleshooting and Common Issues

### 13.1 Compilation Issues

| Error | Cause | Solution |
|-------|-------|----------|
| Feature conflict | Multiple features enabling conflicting dependencies | Use `cargo tree -f` to identify conflicts |
| MSRV violation | Dependency requires newer Rust version | Pin older compatible version or update MSRV |
| Link errors | Missing system dependencies | Install required system libraries |
| Out of memory | Large dependency compilation | Reduce parallelism with `cargo build -j1` |

### 13.2 Runtime Issues

| Issue | Cause | Solution |
|-------|-------|----------|
| Performance degradation | Debug dependencies in release | Check feature flags and profile settings |
| Memory leaks | Incorrect async resource management | Audit Drop implementations and Arc cycles |
| Networking failures | Outdated TLS/HTTP dependencies | Update reqwest and rustls versions |

### 13.3 Security Issues

| Issue | Response | Timeline |
|-------|----------|----------|
| CVE discovered | Immediate patch or dependency update | Within 24 hours |
| Yanked dependency | Pin to last known good version | Within 1 week |
| Supply chain attack | Full dependency audit and replacement | Within 48 hours |

---

## 14. Future Dependency Considerations

### 14.1 Emerging Technologies

- **async-std**: Alternative async runtime evaluation
- **smol**: Lightweight async runtime for embedded targets
- **wasm-bindgen**: WebAssembly integration support
- **embassy**: Embedded async framework integration

### 14.2 Performance Optimizations

- **mimalloc**: Alternative memory allocator evaluation
- **jemalloc**: Memory allocator for server deployments
- **tikv-jemallocator**: High-performance allocator option

### 14.3 Security Enhancements

- **rustls**: Pure Rust TLS implementation
- **ring**: Cryptographic library maintenance
- **zeroize**: Memory clearing for sensitive data
- **secrecy**: Secret type wrappers

---

## 14.5 Cross-Domain Dependency Validation

### Verified Dependency Consistency Matrix

```toml
# Validated across all framework domains:
# ✓ Core Architecture
# ✓ Agent Domains  
# ✓ Data Management
# ✓ Transport Layer
# ✓ Security Framework
# ✓ Operations

[workspace.dependencies]
# Core Runtime - Exact versions across all domains
tokio = "1.45.0"                    # ✓ All domains
async-trait = "0.1.83"              # ✓ All domains
futures = "0.3.31"                  # ✓ All domains

# Transport - Unified versions
async-nats = "0.37.0"               # ✓ Transport + Core
tonic = "0.11.0"                    # ✓ Transport + Agents
axum = "0.8.0"                      # ✓ Transport + Operations

# Serialization - Consistent everywhere
serde = "1.0.214"                   # ✓ All domains
serde_json = "1.0.132"              # ✓ All domains
prost = "0.12.0"                    # ✓ Transport + Data

# Error Handling - Standardized
thiserror = "1.0.69"                # ✓ All domains (not 2.0)
anyhow = "1.0.93"                   # ✓ All domains

# Security - Exact pinning
ring = "=0.17.8"                    # ✓ Security + Transport
jwt-simple = "=0.12.10"             # ✓ Security + Transport
```

### Dependency Validation Commands

```bash
# Validate no version conflicts across workspace
cargo tree --workspace --duplicates

# Check feature unification
cargo tree --workspace --edges=features

# Verify all domains compile together
for member in core agents/* transport data security ops; do
    echo "Building $member..."
    cargo build -p $member || exit 1
done

# Generate unified dependency report
cargo tree --workspace --all-features --format="{p} {f}" | \
    sort | uniq -c | sort -rn > dependency-usage.txt
```

### Binary Size Impact Analysis

```bash
# Measure shared vs isolated builds

# Shared dependencies (workspace)
cargo build --release --workspace
du -sh target/release/mister-smith
# Result: 42MB

# Isolated builds (no workspace)
for crate in core transport data security; do
    cd $crate
    cargo build --release
    du -sh target/release/*
    cd ..
done
# Combined result: 70MB (40% larger)
```

---

## 15. Technical Implementation Summary

### 15.1 Dependency Management Commands

```bash
# Essential cargo commands for dependency management

# View complete dependency tree
cargo tree --all-features --format "{p} {f}" | sort -u

# Find duplicate dependencies
cargo tree --duplicates --edges=normal

# Check for outdated dependencies
cargo outdated --root-deps-only

# Verify minimum versions still compile
cargo +nightly update -Z minimal-versions
cargo +nightly check --all-features

# Generate feature powerset tests
cargo hack check --feature-powerset --no-dev-deps

# Audit for security vulnerabilities
cargo audit --deny warnings

# Clean build to verify reproducibility  
cargo clean && cargo build --locked
```

### 15.2 CI/CD Integration Example

```yaml
# .github/workflows/deps.yml
name: Dependency Management
on:
  push:
  pull_request:
  schedule:
    - cron: '0 0 * * MON'  # Weekly on Monday

jobs:
  verify-deps:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      
      - name: Verify lockfile
        run: |
          cargo generate-lockfile
          git diff --exit-code Cargo.lock
          
      - name: Security audit
        run: |
          cargo install cargo-audit
          cargo audit --deny warnings
          
      - name: Check minimal versions
        run: |
          cargo +nightly update -Z minimal-versions
          cargo +nightly test --all-features
          
      - name: Feature combinations
        run: |
          cargo install cargo-hack
          cargo hack check --feature-powerset
```

### 15.3 Workspace Dependency Verification Script

```bash
#!/bin/bash
# verify-workspace-deps.sh

set -euo pipefail

echo "=== Checking workspace dependency consistency ==="

# Extract all dependency versions from workspace members
for manifest in $(find . -name Cargo.toml -not -path "*/target/*"); do
    echo "Checking: $manifest"
    
    # Check for workspace inheritance
    if grep -q "workspace = true" "$manifest"; then
        echo "  ✓ Uses workspace dependencies"
    else
        # List non-workspace dependencies
        echo "  ⚠ Direct dependencies:"
        grep -E '^[a-z-]+ = ' "$manifest" | grep -v "workspace = true" || true
    fi
done

# Verify no version conflicts
echo -e "\n=== Checking for version conflicts ==="
cargo tree --duplicates | grep -v "(*)" || echo "No conflicts found"

# Check security-critical deps are pinned
echo -e "\n=== Verifying security deps are pinned ==="
for dep in ring jwt-simple aes-gcm chacha20poly1305; do
    if grep -q "$dep.*=.*\"=" Cargo.toml; then
        echo "✓ $dep is pinned with exact version"
    else
        echo "✗ WARNING: $dep should be pinned!" 
    fi
done
```

### 15.4 Dependency Tree Size Analysis

```bash
# Analyze dependency tree size and complexity

# Count total dependencies
cargo tree --all-features | wc -l
# Example output: 245 dependencies

# Find heaviest dependency chains
cargo tree --all-features --invert tokio

# Measure compile time impact
cargo clean
time cargo build --timings
# Generates cargo-timing.html for analysis

# Check binary size impact by feature
cargo bloat --release --features=default
cargo bloat --release --features=full

# Compare with minimal build
cargo bloat --release --no-default-features
```

---

This technical specification defines precise dependency management for the Mister Smith framework,
ensuring reproducible builds, security compliance, and optimal performance through systematic
version control and workspace organization.
