## 2024-03-12 - [Audit Persister Insert Loop Optimization]
**Learning:** Cloned data inside an iterative insert loop for HashSets can often be replaced by the `.extend()` method, avoiding redundant and manual iterations. Additionally, predicting capacity bounds manually can be simplified by avoiding throwaway temporary collections (such as cloning a subset and replacing) when `.extend` and direct re-assignment achieves the correct result efficiently.
**Action:** When seeing loops inserting into `HashSet`s or `HashMap`s with `.clone()`, check if `extend` with `into_iter` can avoid duplicate cloning and optimize allocations.

## 2023-10-27 - [Optimize Database Migration Status Lookup]
**Learning:** `Vec::binary_search_by_key` is fast but requires the array to be explicitly sorted by the key, and relying on implicit SQL `ORDER BY` without enforcement in Rust is risky for binary search correctness. Converting an array to a `HashMap` prior to a lookup loop is a safer approach for turning O(N^2) lookups into O(N) when slice order cannot be perfectly guaranteed.
**Action:** When optimizing loop lookups, prefer `HashMap`/`HashSet` if the dataset size justifies allocation, rather than `binary_search` on vectors that aren't explicitly sorted in memory.

## 2023-11-23 - Avoiding clones in Rust iterators
**Learning:** In Rust, iterating over references `for item in &collection` requires cloning the fields if they are needed by value. Consuming the collection by taking ownership `for item in collection` avoids `clone()` calls completely and improves memory and speed.
**Action:** Always check if a collection is needed later in the function. If not, consume it instead of borrowing it to avoid unnecessary `.clone()` calls.

## 2025-02-17 - Serialize Request Body Outside Loop
**Learning:** Avoid `serde_json::to_vec` and JSON serialization on each retry loop iteration, since the request payload is static. Even though `reqwest` exposes `.json()`, passing `.body()` directly with a cloned `Vec<u8>` is considerably faster and avoids excessive object creation in high-latency network retries.
**Action:** Identify serialization inside retry loops and move it outside to save CPU cycles and reduce overall latency.
