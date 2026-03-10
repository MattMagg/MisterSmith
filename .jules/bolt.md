
## 2024-03-12 - [Audit Persister Insert Loop Optimization]
**Learning:** Cloned data inside an iterative insert loop for HashSets can often be replaced by the `.extend()` method, avoiding redundant and manual iterations. Additionally, predicting capacity bounds manually can be simplified by avoiding throwaway temporary collections (such as cloning a subset and replacing) when `.extend` and direct re-assignment achieves the correct result efficiently.
**Action:** When seeing loops inserting into `HashSet`s or `HashMap`s with `.clone()`, check if `extend` with `into_iter` can avoid duplicate cloning and optimize allocations.
