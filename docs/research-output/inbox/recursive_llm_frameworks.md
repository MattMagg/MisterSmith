# Recursive Language Models (RLM): Agent Framework Knowledge Base
> Source: Prime Intellect Blog — "Recursive Language Models: the paradigm of 2026" (January 1, 2026)
> Author: Sebastian (Prime Intellect Research)
> Paper: https://arxiv.org/abs/2512.24601
---
## 1. Problem Statement
LLM agents now autonomously implement complex changes across large codebases,
reading/editing dozens of files, searching the web, and maintaining context over
multiple requests. This demands vast token budgets, but:
- **Per-token cost scales linearly** with context length.
- **Context rot**: model capability degrades as context grows.
- Architecture/training improvements help but are insufficient alone.
- **Scaffolding** has consistently been the biggest multiplier for effective context length.
---
## 2. Existing Approaches to Long-Context Scaffolding
### 2.1 File-System + Summarization (Current Industry Standard)
- Used by Claude Code, OpenAI Codex, and similar TUI systems.
- Technique: file-system state + periodic LLM summarization → context compression.
- Result: a succession of agents connected by a shared prompt and file state.
### 2.2 Context Folding (Emerging Paradigm)
Goal: manage the context window itself (not external files) to keep it short, while
maintaining a continual, growing rollout. Compatible with file-based scaffolding since
the model appears as a normal LLM externally.
#### Notable Context Folding Methods:
| Method | Mechanism |
|---|---|
| **Context-Folding (branching)** | Agent branches its rollout and returns; only a self-chosen summary of the branch persists in context. |
| **AgentFold** | Every action produces a result + a summary of the action and reasoning. Summaries can be hierarchical (multi-action consolidation) or per-action. |
| **Agentic Context Engineering** | Three-agent system: Generator (uses knowledge base), Reflector (extracts lessons), Curator (updates knowledge base). |
---
## 3. The Recursive Language Model (RLM)
### 3.1 Core Concept
Instead of ingesting all input directly, the RLM gives the LLM:
- A **persistent Python REPL** to inspect/transform input data.
- The ability to **call sub-LLMs** (fresh instances of itself) from within the REPL.
### 3.2 Why RLM Over Alternatives
- **Aligned with "The Bitter Lesson"**: enables direct training with RLM scaffolding for learned context folding.
- **No summarization** (avoids information loss). Instead, proactively delegates context to Python scripts and sub-LLMs.
- **End-to-end RL trainability**: teaching models to manage their own context through reinforcement learning is the target breakthrough.
### 3.3 Key Capabilities
1. Huge input data (PDFs, datasets, videos) never loaded directly into context → avoids context rot.
2. Python-based search, filter, and transformation of context → no redundant input processing.
3. Sub-LLMs (fresh instances) perform delegated work; input data is programmatically piped to them.
### 3.4 Architecture Details
#### Sub-LLM System
- **Parallelizable**: `llm_batch()` dispatches a batch of prompts in parallel.
- **Sub-LLMs get tools**: any tools added to the environment are only usable by sub-LLMs (not the main RLM). Rationale: tools produce many tokens; the main RLM stays lean.
- Main RLM delegates tool-heavy work → never sees verbose tool output directly.
#### Python Environment
- Any pip package can be installed (model is aware of what's available).
- Code executes in **isolated Sandboxes**.
- Standard library always available.
#### Answer Mechanism (Diffusion-Style)
```python
answer = {"content": "", "ready": False}
```
- `answer["content"]`: the model writes, deletes, edits over multiple turns.
- `answer["ready"]`: only when set to `True` does the rollout end.
- This allows **iterative answer refinement** across the reasoning chain — a form of diffusion.
#### Input Data Handling
- **Prompt**: placed directly into RLM's context window.
- **Extra input data**: available only programmatically (must `print()` in REPL to view).
- REPL output is truncated (default: 8192 chars per turn) → forces use of Python + sub-LLMs.
---
## 4. Experimental Design
### 4.1 Three-Way Comparison
For every environment, three scaffolds are tested:
1. **Standard LLM** with native tools.
2. **RLM** (no environment-specific guidance).
3. **RLM + Environment Tips** (strategy hints for the scaffold).
### 4.2 Constraints
- Per-REPL-call timeout: 120 seconds (default).
- 50 rollouts per configuration.
- No hyperparameter tuning — only relative LLM vs RLM performance matters.
### 4.3 Models Tested
| Model | Source | Notes |
|---|---|---|
| GPT-5-mini | OpenAI API | Best RLM user; primary ablation model |
| GLM 4.6 | OpenRouter (z-ai) | Strong DeepDive RLM gains |
| GLM 4.5 Air | OpenRouter (z-ai) | Follows general trends |
| INTELLECT-3 | OpenRouter (nebius/fp8) | Prime Intellect's own 100B+ MoE model |
---
## 5. Evaluation Environments
### 5.1 DeepDive (Deep Research / Web Tool-Use)
- **Task**: Answer complex, multi-hop questions by walking knowledge graphs.
- **Tools**: `search(query)`, `open(url)` — heavy token producers (10k+ tokens per call; up to 1.5M without truncation).
- **RLM advantage**: Sub-LLMs handle verbose web content, return concise summaries → main context stays clean.
- **Key finding**: RLM underperforms LLM without tips, but **outperforms with strategic tips** (decompose question → parallel sub-LLM research → synthesize → iterate).
#### Optimal Strategy (Environment Tips):
1. Decompose the question into focused sub-tasks.
2. Dispatch sub-tasks via `llm_batch()` in parallel — each sub-LLM has search/open tools.
3. Synthesize findings; cross-reference for consistency.
4. Iterate with follow-up sub-tasks if gaps exist.
5. Finalize into `answer["content"]`, set `answer["ready"] = True`.
### 5.2 math-python (Mathematical Problem Solving)
- **Task**: Difficult math problems with Python tool access.
- **Libraries**: numpy, scipy, sympy pre-installed.
- **Key finding**: RLM **hurts** performance. The RLM enables identical behavior to standard LLM (both have Python), but the scaffolding overhead causes inefficient thinking.
- **Hypothesis**: Math may not benefit from multi-agent decomposition, OR models need RLM-specific training.
### 5.3 Oolong (Long-Context Evaluation)
- **Task**: Classification + data extraction + aggregation over very long inputs.
- **Subsets**: synth, synth-with-labels, real (D&D session transcripts).
- **Key finding**: RLM **significantly outperforms** LLM on real (most complex) data. LLM fails entirely on long contexts (API rejects inputs exceeding context window), while RLM handles up to ~1.5M characters (~300-400k tokens).
- **RLM solved synth+labels with regex alone** (no sub-LLMs needed, perfect scores).
#### Optimal Strategy (Environment Tips):
1. Split context into chunks (paragraphs or fixed character windows with overlap).
2. Write a search prompt, append to each chunk.
3. Call `llm_batch()` with all prompts to scan in parallel.
4. Aggregate findings from responses.
### 5.4 Verbatim Copy (Exact Text Reproduction)
- **Task**: Reproduce complex text exactly.
- **Content types**: words, JSON, CSV, codes (UUIDs), mixed.
- **Key finding**: RLM **improves** performance, especially on JSON (hardest type). The iterative answer refinement mechanism is valuable — write attempt → print → compare → fix with `str.replace()`.
- **LLM limitation**: must one-shot the entire answer; RLM can iterate.
#### Optimal Strategy (Environment Tips):
1. Write initial attempt to `answer["content"]`.
2. Print it to see exactly what was written.
3. Compare carefully with original — find typos, transpositions, missing chars.
4. Fix errors with string operations (slicing, replacement).
5. Set `answer["ready"] = True` only after verification.
---
## 6. Key Results Summary
### 6.1 Where RLM Helps
| Dimension | Finding |
|---|---|
| **Long-context tasks** | RLM handles inputs far beyond LLM context limits (e.g., 1.5M chars in Oolong) |
| **Token-heavy tool-use** | Sub-LLMs absorb verbose tool outputs; main context stays compact |
| **Iterative refinement** | Answer-in-variable mechanism enables progressive improvement |
| **Main model token efficiency** | Dramatically improved for tool-heavy tasks (DeepDive) |
| **Thinking-token scaling** | Sub-LLMs enable scaling completion tokens at low main model context cost |
### 6.2 Where RLM Hurts
| Dimension | Finding |
|---|---|
| **Simple tool-use tasks (math)** | Scaffolding overhead causes inefficient reasoning |
| **Without strategic guidance** | RLM underperforms on DeepDive without tips (poor scaffold utilization) |
| **Wall-clock time** | RLM increases time in all cases (more tokens, sequential sub-LLM calls, code execution overhead) |
| **Short-context tasks** | LLM beats RLM on shortest inputs (scaffolding overhead not justified) |
### 6.3 Cross-Model Observations
- **GPT-5-mini**: Best RLM user overall.
- **GLM 4.6**: Enormous DeepDive RLM boost (~2x), but crashes when given tips that over-rely on sub-LLMs.
- **INTELLECT-3**: Benefits greatly from environment tips; similar patterns to GPT-5-mini on Oolong.
- **General**: All models show RLM potential limited by lack of RLM-specific training.
---
## 7. RLM vs. Long-Context Attention
These are **dual, complementary** approaches:
| Aspect | Long-Context Attention | Context Folding (RLM) |
|---|---|---|
| **Learns during** | Pretraining / midtraining | RL on task outcomes |
| **Decides what to forget** | From language modeling perspective | From task reward signal |
| **Mechanism** | Efficient attention architectures | Active context management via code + sub-LLMs |
**Conclusion**: Both are needed. Better attention delays context rot; context folding lets the model actively manage context beyond what attention alone can handle.
---
## 8. Design Principles for Agent Framework Developers
### 8.1 From RLM Design Decisions
1. **Separate tool execution from main reasoning context**: tools produce too many tokens; delegate to sub-agents.
2. **Make input data programmatically accessible, not context-injected**: force the agent to selectively retrieve what it needs.
3. **Provide iterative answer refinement**: let the agent draft, inspect, and correct its output across turns.
4. **Enable parallel sub-agent dispatch**: `llm_batch()` pattern for concurrent work decomposition.
5. **Isolate execution environments**: sandboxed code execution prevents state leakage.
6. **Truncate tool outputs**: cap REPL/tool output shown to the main agent (e.g., 8192 chars) to force selective information retrieval.
### 8.2 From Experimental Findings
7. **Provide environment-specific strategy tips**: untrained models dramatically underperform without guidance on how to use the scaffold.
8. **Don't force scaffolding complexity on simple tasks**: math-python shows that overhead can outweigh benefits when the base task is straightforward.
9. **Train models on the scaffold**: the largest performance gaps come from models not knowing how to use the RLM effectively, not from the scaffold itself.
10. **Chunk-and-dispatch for long contexts**: the pattern of splitting input → parallel sub-LLM classification → aggregation is consistently effective.
---
## 9. Future Directions
1. **Variable recursion depth**: sub-LLMs calling further sub-LLMs (depth > 1), or depth 0 (normal LLM + REPL + tools).
2. **Custom REPL functions**: user-defined functions available to the RLM in its REPL.
3. **Package documentation injection**: make the model aware of installed packages' details without rewriting the prompt.
4. **Multi-turn context compression**: native context compression across assistant-user turns.
5. **Multi-modal support**: images, video, and custom data types (challenge: sandbox communication).
6. **RL training on RLM scaffold**: starting with small models; expected to be the primary unlock for performance.
---
## 10. Key References & Resources
- **RLM Paper**: https://arxiv.org/abs/2512.24601 (Alex Zhang, October 2025)
- **Implementation**: `verifiers` library → `RLMEnv` (experimental), `sebastian/experiment/rlm` branch
- **Environments Hub**: DeepDive (`deepdive-rlm`), math-python (`math-env-rlm`), Oolong (`oolong-rlm`), verbatim-copy (`verbatim-copy-rlm`)
- **Training**: `prime-rl` (Prime Intellect's RL training stack)
- **DeepDive Dataset**: Available on GitHub + HuggingFace
- **Oolong Dataset**: Available on GitHub + HuggingFace