## 2025-02-17 - Serialize Request Body Outside Loop
**Learning:** Avoid `serde_json::to_vec` and JSON serialization on each retry loop iteration, since the request payload is static. Even though `reqwest` exposes `.json()`, passing `.body()` directly with a cloned `Vec<u8>` is considerably faster and avoids excessive object creation in high-latency network retries.
**Action:** Identify serialization inside retry loops and move it outside to save CPU cycles and reduce overall latency.
