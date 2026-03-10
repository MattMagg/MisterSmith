## 2026-03-10 - [Optimize Metrics Tag Allocation]
**Learning:** String allocations and deep copies for static identifiers (like Tokio worker IDs) inside high-frequency polling loops can create measurable CPU and allocator overhead. The `metrics` crate's macros (`gauge!`, `counter!`) require `SharedString` types, which, if created from `String` values via `worker.clone()` or `i.to_string()`, induce repeated heap allocations.
**Action:** Use a `std::sync::OnceLock` containing cached worker-label strings so hot-path metric collection can reuse borrowed values instead of allocating new owned labels on each tick.

## 2023-11-23 - Avoiding clones in Rust iterators
**Learning:** In Rust, iterating over references `for item in &collection` requires cloning the fields if they are needed by value. Consuming the collection by taking ownership `for item in collection` avoids `clone()` calls completely and improves memory and speed.
**Action:** Always check if a collection is needed later in the function. If not, consume it instead of borrowing it to avoid unnecessary `.clone()` calls.
