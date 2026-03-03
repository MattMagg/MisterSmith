# ms-framework-docs Folder Instructions

## FOLDER CONTEXT

You are now in the MisterSmith framework documentation folder. This folder contains the technical specifications
for the future implementation. Remember: These are SPECIFICATIONS ONLY - no implementation exists yet.

## DOCUMENTATION STRUCTURE

```text
ms-framework-docs/
├── core-architecture/      # Foundational system design
├── agent-domains/         # Specialized agent specifications
├── data-management/       # Storage and messaging patterns  
├── transport/            # Communication layer specs
├── security/             # Authentication/authorization
├── operations/           # Deployment and monitoring
├── testing/              # Test specifications
└── research/             # Investigation and planning docs
```

## WORKING WITH SPECIFICATIONS

### Before Modifying Any Document

```bash
# 1. Understand dependencies
grep -r "filename_without_extension" . | grep -v "filename.md"

# 2. Check for related specifications  
find . -name "*.md" -exec grep -l "ComponentName" {} \;

# 3. Verify terminology consistency
grep -r "specific_term" . | cut -d: -f1 | sort | uniq
```

### Document Interconnections

**High-Impact Files** (changes affect many others):

- `core-architecture/system-architecture.md` - Foundation for all specs
- `core-architecture/type-definitions.md` - Core types used everywhere
- `data-management/message-schemas.md` - Message formats across system
- `agent-domains/SPECIALIZED_AGENT_DOMAINS_ANALYSIS.md` - Agent type definitions

**Always Check These When Modifying**:

- Agent-related changes → Check all files in `agent-domains/` and `data-management/agent-*.md`
- Message changes → Update `message-schemas.md`, `core-message-schemas.md`, `workflow-message-schemas.md`
- Architecture changes → Verify impact on `integration-*.md` files
- Security changes → Cross-reference with `transport/` specifications

## DOCUMENTATION PHASE SPECIFIC RULES

### Allowed Actions in This Folder

✅ Improve specification clarity
✅ Fix inconsistencies between documents
✅ Add missing technical details
✅ Remove over-engineered complexity
✅ Validate technical feasibility
✅ Ensure proper cross-references

### Forbidden Actions

❌ Adding implementation code examples beyond pseudocode
❌ Creating actual Rust code files
❌ Adding build scripts or configuration files
❌ Claiming specifications are "implemented" or "tested"
❌ Removing complexity that's necessary for the design

## SPECIFICATION QUALITY STANDARDS

### Technical Accuracy

- Specifications must be implementable with stated technologies
- Version numbers must be consistent (e.g., Tokio 1.38)
- Dependencies must be compatible
- Patterns must follow Rust best practices

### Consistency Requirements

When mentioning:

- **Agent Types**: Use exact names from `SPECIALIZED_AGENT_DOMAINS_ANALYSIS.md`
- **Messages**: Match schemas in `message-schemas.md`
- **Errors**: Follow patterns in security and transport specs
- **Async Patterns**: Align with `async-patterns.md` and `tokio-runtime.md`

### Cross-Reference Validation

```bash
# Example: After modifying agent lifecycle
files_to_check=(
    "agent-lifecycle.md"
    "agent-operations.md" 
    "agent-orchestration.md"
    "supervision-trees.md"
    "process-management-specifications.md"
)

for file in "${files_to_check[@]}"; do
    echo "Checking $file for consistency..."
    grep -n "lifecycle\|spawn\|terminate" "$file"
done
```

## COMMON DOCUMENTATION PATTERNS

### Component Specification Pattern

```markdown
## Component Name

### Purpose
[Clear, single responsibility]

### Interface
[Public API/Message contracts]

### Dependencies
- Internal: [Other components]
- External: [Libraries with versions]

### Behavior
[State transitions, error handling]

### Integration Points
[How it connects to other components]
```

### Message Definition Pattern

```rust
// Pseudocode only - NOT implementation
struct MessageName {
    field: Type,  // Purpose
    field2: Type, // Constraints
}
```

## VALIDATION CHECKLISTS

### Before Committing Changes

- [ ] All modified terms are consistent across documents
- [ ] Cross-references still valid
- [ ] No implementation code added
- [ ] Examples remain pseudocode only
- [ ] Version numbers unchanged unless intentional
- [ ] No new TODOs or TBDs introduced

### Weekly Documentation Health Check

```bash
# Run these checks periodically
echo "=== Checking for TODOs ==="
grep -r "TODO\|TBD\|FIXME" . | wc -l

echo "=== Checking for inconsistent agent names ==="
grep -r "Agent\|agent" . | grep -v "Agent::" | head -20

echo "=== Checking for implementation claims ==="
grep -r "implemented\|working\|tested\|production" . | grep -v "to be implemented"
```

## NAVIGATION TIPS

### Quick Access to Key Specifications

```bash
# View all agent specifications
ls -la agent-domains/
ls -la data-management/agent-*.md

# Find message schemas
find . -name "*message*.md" -o -name "*schema*.md"

# Locate integration patterns
ls -la core-architecture/integration-*.md
```

### Understanding Component Relationships

1. Start with `system-architecture.md` for overview
2. Follow to `component-architecture.md` for structure
3. Check `integration-patterns.md` for connections
4. Verify in specific component files

## DOCUMENTATION MAINTENANCE

### Identifying Inconsistencies

Common inconsistency patterns:

- Different versions of the same component description
- Mismatched message field names
- Varying agent capability descriptions
- Conflicting architectural decisions

### Resolving Conflicts

When specifications conflict:

1. Check which is more recently updated
2. Determine which aligns with core architecture
3. Validate technical feasibility
4. Update all affected documents
5. Document the decision in commit message

## REMEMBER IN THIS FOLDER

You are crafting the blueprint for a complex system. Every specification should be:

- **Clear**: Unambiguous to future implementers
- **Consistent**: Aligned with all related specs
- **Complete**: No critical details missing
- **Feasible**: Actually implementable

The documentation in this folder is the CONTRACT for future implementation. Make it solid.
