# MisterSmith Async Runtime & Concurrency Documentation Workflow

## Workflow Overview

This workflow guides library context gathering and documentation revision for the Async Runtime & Concurrency domain of MisterSmith framework, focusing on Tokio and related async patterns.

---

## 🎯 Target Libraries and Context7 References

### Primary Libraries

1. **Tokio v1.45.0** - Core async runtime
   - Context7 ID: `/tokio-rs/tokio`
   - Features: `["full"]`
   - Documentation Focus: Runtime configuration, task spawning, channels, synchronization

2. **futures v0.3.31** - Async utilities and combinators
   - Context7 ID: `/rust-lang/futures-rs`
   - Documentation Focus: Stream processing, future combinators, async traits

3. **async-trait v0.1.83** - Async trait support
   - Context7 ID: `/dtolnay/async-trait`
   - Documentation Focus: Trait definitions, lifetime handling

4. **crossbeam-channel v0.5.13** - Lock-free channels
   - Context7 ID: `/crossbeam-rs/crossbeam`
   - Documentation Focus: Channel patterns, actor communication

---

## 📁 Target Documentation Files

### Core Architecture Files

```bash
# Primary files for async runtime documentation
ms-framework-docs/core-architecture/async-patterns.md
ms-framework-docs/core-architecture/tokio-runtime.md
ms-framework-docs/core-architecture/supervision-trees.md
ms-framework-docs/core-architecture/component-architecture.md

# Agent lifecycle files  
ms-framework-docs/data-management/agent-lifecycle.md
ms-framework-docs/data-management/agent-operations.md
ms-framework-docs/data-management/agent-orchestration.md

# Process management
ms-framework-docs/operations/process-management-specifications.md
```

---

## 🔄 Workflow Steps

### Step 1: Gather Tokio Context and Best Practices

```bash
# 1.1 Retrieve Tokio documentation
mcp__context7__get-library-docs \
  --context7CompatibleLibraryID="/tokio-rs/tokio" \
  --tokens=15000 \
  --topic="runtime configuration task spawning channels"

# 1.2 Verify current Tokio usage patterns in docs
grep -r "tokio::" ms-framework-docs/ | grep -E "(spawn|select|join|channel)" | head -20
grep -r "Runtime::" ms-framework-docs/ | grep -v "Runtime::" | head -10

# 1.3 Check for outdated patterns
grep -r "tokio::run\|tokio::spawn_blocking" ms-framework-docs/ || echo "No outdated patterns found"
```

### Step 2: Update Async Pattern Documentation

```bash
# 2.1 Validate current async patterns
cd ms-framework-docs/core-architecture
cat async-patterns.md | grep -E "^##|^###" | head -20

# 2.2 Cross-reference with tokio-runtime.md
diff <(grep -E "spawn|select|join" async-patterns.md) \
     <(grep -E "spawn|select|join" tokio-runtime.md) || true

# 2.3 Check for consistency in error handling patterns
grep -r "Result<.*Error>" . | grep -E "(async|await)" | wc -l
```

### Step 3: Futures Library Integration

```bash
# 3.1 Retrieve futures documentation  
mcp__context7__get-library-docs \
  --context7CompatibleLibraryID="/rust-lang/futures-rs" \
  --tokens=10000 \
  --topic="stream processing future combinators"

# 3.2 Update stream processing patterns
find ms-framework-docs -name "*.md" -exec grep -l "Stream\|StreamExt" {} \;

# 3.3 Verify combinator usage
grep -r "futures::" ms-framework-docs/ | grep -E "(map|filter|then|and_then)" | wc -l
```

### Step 4: Actor System Channel Patterns

```bash
# 4.1 Retrieve crossbeam documentation
mcp__context7__get-library-docs \
  --context7CompatibleLibraryID="/crossbeam-rs/crossbeam" \
  --tokens=8000 \
  --topic="channels select unbounded"

# 4.2 Update agent communication patterns
cd ms-framework-docs/data-management
grep -n "channel" agent-communication.md | head -10

# 4.3 Verify channel type consistency
grep -r "crossbeam.*channel\|mpsc\|broadcast" . | cut -d: -f1 | sort | uniq
```

### Step 5: Supervision Tree Async Patterns

```bash
# 5.1 Analyze supervision tree async requirements
cd ms-framework-docs/core-architecture
grep -A5 -B5 "async.*supervision\|supervision.*async" supervision-trees.md

# 5.2 Update lifecycle state machines
cd ../data-management
grep -n "state.*transition\|lifecycle.*state" agent-lifecycle.md

# 5.3 Verify error propagation patterns
grep -r "panic.*catch\|catch_unwind" ../core-architecture/ || echo "Good: No panic catching"
```

---

## ✅ Verification Checklist

### Pre-Implementation Verification

```bash
# Run comprehensive async pattern check
echo "=== Async Pattern Verification ==="
find ms-framework-docs -name "*.md" -exec grep -l "async\|await\|tokio\|futures" {} \; | wc -l
echo "Files with async patterns: $(find ms-framework-docs -name "*.md" -exec grep -l "async\|await" {} \; | wc -l)"

# Check for version consistency
echo "=== Version Consistency ==="
grep -r "tokio.*=.*\"" ms-framework-docs/ | grep -v "1.45" || echo "All tokio versions consistent"

# Verify no implementation code
echo "=== Documentation Phase Check ==="
grep -r "fn main\|impl.*for\|mod tests" ms-framework-docs/ | grep -v "pseudocode" | wc -l
```

### Cross-Reference Validation

```bash
# Files that must be checked together
files_to_validate=(
    "async-patterns.md"
    "tokio-runtime.md"
    "agent-lifecycle.md"
    "supervision-trees.md"
    "process-management-specifications.md"
)

for file in "${files_to_validate[@]}"; do
    echo "Checking references in $file..."
    grep -n "async\|spawn\|channel\|supervision" "$(find ms-framework-docs -name "$file")" | wc -l
done
```

---

## 🔧 Formatting Standards

### Tokio-Specific Patterns to Apply

1. **Runtime Configuration**:
   ```rust
   // Pseudocode pattern for documentation
   Runtime::builder()
       .worker_threads(4)
       .enable_all()
       .build()
   ```

2. **Task Spawning Patterns**:
   ```rust
   // Document spawn vs spawn_blocking usage
   tokio::spawn(async move { /* async work */ })
   tokio::task::spawn_blocking(|| { /* blocking work */ })
   ```

3. **Channel Selection Patterns**:
   ```rust
   // tokio::select! macro patterns
   select! {
       result = future1 => { /* handle */ }
       result = future2 => { /* handle */ }
   }
   ```

---

## 🚀 Parallel Agent Coordination

### Agent Assignment

```bash
# Agent 1: Core async patterns
git checkout -b agent1-async-patterns
# Focus: async-patterns.md, tokio-runtime.md

# Agent 2: Agent lifecycle async
git checkout -b agent2-agent-lifecycle  
# Focus: agent-lifecycle.md, agent-operations.md

# Agent 3: Supervision async patterns
git checkout -b agent3-supervision-async
# Focus: supervision-trees.md, process-management-specifications.md

# Coordination check
git branch | grep "agent[0-9]" | wc -l  # Should show 3
```

### Conflict Prevention

```bash
# Before starting work
for branch in agent1-async-patterns agent2-agent-lifecycle agent3-supervision-async; do
    echo "=== $branch files ==="
    git ls-tree -r --name-only $branch | grep -E "(async|tokio|agent|supervision)" | head -5
done
```

---

## 📊 Success Metrics

1. **Pattern Consistency**: All async patterns use Tokio 1.45.0 idioms
2. **No Implementation**: Zero actual Rust code (only pseudocode)
3. **Cross-References Valid**: All file references resolve correctly
4. **Version Alignment**: All version numbers match dependency-specifications.md
5. **Formatting Clean**: Passes markdown linting

---

## 🔍 Common Issues and Resolutions

### Issue: Outdated async/await patterns

```bash
# Find old patterns
grep -r "futures01\|tokio::run\|tokio-core" ms-framework-docs/

# Update to modern patterns
# Old: futures01::Future
# New: std::future::Future
```

### Issue: Inconsistent channel types

```bash
# Audit channel usage
grep -r "channel" ms-framework-docs/ | grep -E "(mpsc|broadcast|oneshot|crossbeam)" | cut -d: -f2 | sort | uniq -c

# Standardize based on use case:
# - mpsc: Multiple producers, single consumer
# - broadcast: Multiple producers, multiple consumers  
# - oneshot: Single value, then close
# - crossbeam: Lock-free for high performance
```

---

## 🎯 Final Validation

```bash
# Run complete validation suite
echo "=== Final Async Domain Validation ==="

# 1. Pattern consistency
echo "Checking pattern consistency..."
find ms-framework-docs -name "*.md" | xargs grep -l "async\|await" | \
  xargs grep -L "tokio\|futures" || echo "All async files reference tokio/futures"

# 2. No implementation
echo "Verifying documentation phase..."
find ms-framework-docs -name "*.md" | xargs grep -E "^impl|^fn.*\{$|^mod.*\{$" | \
  grep -v "pseudocode" | wc -l

# 3. Version check
echo "Validating versions..."
grep -h "version.*=" ms-framework-docs/core-architecture/dependency-specifications.md | \
  grep -E "(tokio|futures|async-trait|crossbeam)" | sort

echo "=== Validation Complete ==="
```

---

This workflow ensures systematic documentation updates based on current Tokio best practices while maintaining MisterSmith's anti-fabrication principles and supporting parallel agent operations.
