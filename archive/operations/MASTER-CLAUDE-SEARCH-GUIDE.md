# Master CLAUDE.md Search Guide

## 🎯 Quick Start Search Strategy

### 1. Identify Your Need

```
Need Architecture? → /ms-framework-docs/core-architecture/
Need Analysis? → /internal-operations/
Need Security? → /ms-framework-docs/security/
Need Data Handling? → /ms-framework-docs/data-management/
Need Communication? → /ms-framework-docs/transport/
Need Operations? → /ms-framework-docs/operations/
```

### 2. Use the Right Keywords

```
Implementation → "implement", "build", "create", "develop"
Analysis → "analyze", "report", "examine", "review"
Security → "auth", "encrypt", "secure", "compliance"
Data → "persist", "store", "flow", "schema"
Communication → "protocol", "message", "transport", "API"
```

### 3. Apply Search Patterns

```
Specific Agent: Agent_\d+
Specific Phase: Phase_\d+
Final Reports: FINAL.*REPORT
Strategies: .*STRATEGY.*
JSON Data: \.json$
```

## 📊 Complete Directory Map with Keywords

### `/internal-operations/`

**Purpose**: Analysis results, strategies, deployment plans
**Keywords**: analysis, reports, consolidation, deployment, RUST-SS, phase, strategy, multi-agent
**Key Subdirs**:

- `analysis-reports/` - Agent outputs and phase reports
- `consolidation-reports/` - Strategy documents
- `deployment-plans/` - Multi-agent coordination

### `/ms-framework-docs/`

**Purpose**: Framework implementation documentation
**Keywords**: architecture, implementation, specifications, patterns
**Key Subdirs**:

- `core-architecture/` - System design
- `data-management/` - Data persistence and orchestration
- `security/` - Security framework
- `transport/` - Communication protocols
- `operations/` - Operational procedures

## 🔍 Universal Search Techniques

### Basic Search Hierarchy

1. Start with CLAUDE.md in target directory
2. Use keywords from search optimization section
3. Apply regex patterns for precise matching
4. Follow cross-references to related docs

### Advanced Search Patterns

#### By Task Type

```regex
# Implementation tasks
/implement.*|build.*|create.*|develop.*/i

# Analysis tasks
/analy.*|report.*|examine.*|review.*/i

# Security tasks
/secur.*|auth.*|encrypt.*|complian.*/i

# Data tasks
/data.*|persist.*|store.*|schema.*/i
```

#### By File Type

```regex
# Documentation
/\.md$/

# Structured data
/\.json$/

# Agent outputs
/Agent_\d+.*/

# Strategy docs
/.*STRATEGY.*/
```

#### By Component

```regex
# Architecture
/.*architecture.*/i

# Security
/.*security.*/i

# Transport
/.*transport.*/i

# Data management
/.*data.*management.*/i
```

## 💡 Search Best Practices

### 1. Start Broad, Narrow Down

- Begin with directory CLAUDE.md
- Use general keywords
- Apply specific patterns
- Follow cross-references

### 2. Use Multiple Approaches

- Keyword search
- Regex patterns
- Directory navigation
- Concept maps

### 3. Leverage Synonyms

- Try alternative terms
- Use both technical and common words
- Consider abbreviations

### 4. Follow the Trail

- Check cross-references
- Look for related concepts
- Use concept maps

## 🚀 Quick Reference Card

### Common Searches

| Need | Location | Keywords |
|------|----------|----------|
| System architecture | `/ms-framework-docs/core-architecture/` | architecture, design, pattern |
| Agent analysis | `/internal-operations/analysis-reports/` | Agent_, analysis, report |
| Security specs | `/ms-framework-docs/security/` | auth, encrypt, RBAC |
| Data persistence | `/ms-framework-docs/data-management/` | persist, store, schema |
| Communication | `/ms-framework-docs/transport/` | protocol, message, API |
| Deployment | `/internal-operations/deployment-plans/` | deploy, multi-agent |

### Regex Cheat Sheet

```regex
Agent_?\d+              # Agent outputs
Phase_\d+               # Phase reports
FINAL.*REPORT           # Final reports
.*STRATEGY.*            # Strategies
.*\.json$               # JSON files
.*auth.*                # Authentication
.*encrypt.*             # Encryption
.*protocol.*            # Protocols
```

### Concept Location Matrix

```
┌─────────────────────┬──────────────────────────────┐
│ Concept             │ Primary Location             │
├─────────────────────┼──────────────────────────────┤
│ System Architecture │ /core-architecture/          │
│ Multi-Agent         │ /deployment-plans/           │
│ Security            │ /security/                   │
│ Data Flow           │ /data-management/            │
│ Transport           │ /transport/                  │
│ Analysis Results    │ /analysis-reports/           │
│ Strategies          │ /consolidation-reports/      │
└─────────────────────┴──────────────────────────────┘
```

## 🎯 Search Optimization Summary

All CLAUDE.md files now include:

1. **Keywords & Synonyms** - Primary and alternative search terms
2. **Regex Patterns** - Quick find patterns for common searches
3. **Concept Locations** - Where to find specific concepts
4. **Quick Search Commands** - Pre-built search strategies
5. **Cross-Reference Maps** - Visual concept relationships

Use this master guide in conjunction with individual CLAUDE.md search sections for optimal navigation efficiency.

---

**Purpose**: Master search coordination across all CLAUDE.md files
**Target**: AI agents requiring efficient framework navigation
**Optimization**: Complete keyword indexing and search pattern library
