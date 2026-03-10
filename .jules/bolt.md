## 2023-11-23 - Avoiding clones in Rust iterators
**Learning:** In Rust, iterating over references `for item in &collection` requires cloning the fields if they are needed by value. Consuming the collection by taking ownership `for item in collection` avoids `clone()` calls completely and improves memory and speed.
**Action:** Always check if a collection is needed later in the function. If not, consume it instead of borrowing it to avoid unnecessary `.clone()` calls.

## 2025-02-17 - Serialize Request Body Outside Loop
**Learning:** Avoid `serde_json::to_vec` and JSON serialization on each retry loop iteration, since the request payload is static. Even though `reqwest` exposes `.json()`, passing `.body()` directly with a cloned `Vec<u8>` is considerably faster and avoids excessive object creation in high-latency network retries.
**Action:** Identify serialization inside retry loops and move it outside to save CPU cycles and reduce overall latency.
