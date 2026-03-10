## 2023-10-27 - [Optimize Database Migration Status Lookup]
**Learning:** `Vec::binary_search_by_key` is fast but requires the array to be explicitly sorted by the key, and relying on implicit SQL `ORDER BY` without enforcement in Rust is risky for binary search correctness. Converting an array to a `HashMap` prior to a lookup loop is a safer approach for turning O(N^2) lookups into O(N) when slice order cannot be perfectly guaranteed.
**Action:** When optimizing loop lookups, prefer `HashMap`/`HashSet` if the dataset size justifies allocation, rather than `binary_search` on vectors that aren't explicitly sorted in memory.

## 2023-11-23 - Avoiding clones in Rust iterators
**Learning:** In Rust, iterating over references `for item in &collection` requires cloning the fields if they are needed by value. Consuming the collection by taking ownership `for item in collection` avoids `clone()` calls completely and improves memory and speed.
**Action:** Always check if a collection is needed later in the function. If not, consume it instead of borrowing it to avoid unnecessary `.clone()` calls.
