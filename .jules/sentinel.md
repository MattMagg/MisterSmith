## 2024-03-12 - [Security Enhancement: Replace Mutex<HashMap> with DashMap for RateLimiter in mister-smith-http]
**Vulnerability:** RateLimiter in `mister-smith-http` was using `Mutex<HashMap>`, which can cause lock contention across all HTTP requests during high traffic scenarios, potentially leading to denial of service or poor performance.
**Learning:** For highly concurrent data structures like rate limiters, global locks (like `tokio::sync::Mutex` around a `HashMap`) should be avoided because every request must acquire the lock. The codebase prefers `DashMap` for concurrent in-memory tracking.
**Prevention:** Use `dashmap::DashMap` for highly concurrent, in-memory state tracking to prevent global lock contention across requests.
