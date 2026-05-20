# Orca Agent CLI — Full Design and Implementation Build Plan

**Document version:** 1.0  
**Date:** 2026-05-18  
**Project name:** Orca Agent CLI  
**Primary goal:** Build a local-first, token-efficient AI orchestration CLI that coordinates Ollama, Claude Code, Codex, Cursor SDK/Composer, and optional premium models using project memory, task routing, context packets, skills, hooks, and rules.

## Table of Contents

1. [Executive Summary](#1-executive-summary)
2. [Core Philosophy](#2-core-philosophy)
3. [What Orca Agent Is](#3-what-orca-agent-is)
4. [What Orca Agent Is Not](#4-what-orca-agent-is-not)
5. [Model and Tool Strategy](#5-model-and-tool-strategy)
6. [System Goals](#6-system-goals)
7. [MVP Scope](#7-mvp-scope)
8. [High-Level Architecture](#8-high-level-architecture)
9. [Core Agents](#9-core-agents)
10. [Execution Flow](#10-execution-flow)
11. [Model Routing Policy](#11-model-routing-policy)
12. [Context Packet Design](#12-context-packet-design)
13. [Project Memory Design](#13-project-memory-design)
14. [Obsidian Vault Design](#14-obsidian-vault-design)
15. [Graph Memory Design](#15-graph-memory-design)
16. [Skills, Hooks, and Rules](#16-skills-hooks-and-rules)
17. [Cursor SDK Integration](#17-cursor-sdk-integration)
18. [Provider Adapter Design](#18-provider-adapter-design)
19. [CLI Command Design](#19-cli-command-design)
20. [Configuration Design](#20-configuration-design)
21. [Data Schemas](#21-data-schemas)
22. [Storage Design](#22-storage-design)
23. [Prompt Library](#23-prompt-library)
24. [Implementation Roadmap](#24-implementation-roadmap)
25. [Detailed Build Tasks](#25-detailed-build-tasks)
26. [Testing Strategy](#26-testing-strategy)
27. [Security, Safety, and Approval Gates](#27-security-safety-and-approval-gates)
28. [Token Minimization Strategy](#28-token-minimization-strategy)
29. [Failure Handling and Escalation](#29-failure-handling-and-escalation)
30. [Example End-to-End Workflow](#30-example-end-to-end-workflow)
31. [Acceptance Criteria](#31-acceptance-criteria)
32. [Future Roadmap](#32-future-roadmap)
33. [Recommended First Implementation Prompt](#33-recommended-first-implementation-prompt)

---

## 1. Executive Summary

Orca Agent CLI is a command-line orchestration tool for coordinating multiple AI coding and reasoning systems while minimizing token usage and avoiding context loss.

The tool takes a project plan document, breaks it into tasks, routes each task to the cheapest capable model, generates compact context packets, executes or prepares task prompts, stores results, updates project memory, and tracks project progress.

The system is inspired by the project-learning and tool-using ideas behind [Hermes Agent](https://github.com/NousResearch/hermes-agent), but with a different memory target:

> Hermes-like systems often learn the user. Orca Agent learns the project.

Orca Agent should maintain durable knowledge about:

- project goals
- architecture
- implementation decisions
- coding rules
- current task state
- known bugs
- relevant files
- model performance
- context handoffs
- task history

It should integrate well with:

- [Obsidian](https://obsidian.md/) for human-readable project memory
- [Cursor TypeScript SDK](https://cursor.com/blog/typescript-sdk) for programmatic repo-aware coding agents
- [graphify](https://github.com/safishamsi/graphify) or a similar graph layer for relationship memory
- [rtk](https://github.com/rtk-ai/rtk) or similar runtime/tool patterns
- Ollama Cloud/local Ollama for cheap workhorse tasks
- Claude Code/Opus for planning and architecture
- Codex/GPT for scoped implementation
- Cursor Composer 2.5 as the main repo-aware coding executor
- Cursor premium models as final escalation

The ideal usage pattern:

```text
Plan document
  ↓
Claude/Ollama task decomposition
  ↓
Orca task graph
  ↓
Ollama context compression
  ↓
Router chooses model
  ↓
Cursor Composer / Codex / Claude / Ollama executes task
  ↓
Result summarized
  ↓
Project memory updated
  ↓
Next task
```

---

## 2. Core Philosophy

### 2.1 Project state, not chat history

The system must not depend on long, fragile chat histories.

Bad pattern:

```text
Long chat → model → longer chat → different model → lost context → confusion
```

Good pattern:

```text
Project memory → task packet → model execution → result summary → memory update
```

Orca Agent treats every model call as a state transition.

### 2.2 Token usage must be controlled by design

Token saving is not an afterthought. It is a core architectural requirement.

Every task should pass through:

1. context selection
2. context compression
3. task routing
4. output format enforcement
5. post-task summarization
6. durable memory update

### 2.3 The project memory is the source of truth

The orchestrator must build and maintain a persistent project memory. The memory is not just logs. It is a structured evolving understanding of the project.

It should answer:

- What is the project trying to become?
- What has been built already?
- What are the core design decisions?
- Which files are important?
- Which tasks are complete?
- Which tasks are blocked?
- Which model should handle which type of work?
- What should future models know before touching the code?

### 2.4 Human approval remains important

The first versions should not be fully autonomous. They should be controlled and inspectable.

Prefer:

```text
Generate task packet → inspect → execute → inspect diff → approve memory update
```

rather than:

```text
Agent reads project → agent edits everything → agent commits without approval
```

### 2.5 Cursor Composer becomes the main coding worker

Because Cursor Composer 2.5 has more available usage than premium Cursor models, the routing changes:

```text
Ollama = cheap context and memory engine
Claude Opus = architect and planner
Cursor Composer 2.5 = main repo-aware coding executor
Codex GPT-5.5 = scoped implementation executor
Cursor premium models = final escalation
```

---

## 3. What Orca Agent Is

Orca Agent CLI is:

- a local-first orchestration CLI
- a project memory manager
- a task graph generator
- a context packet generator
- a model router
- a provider adapter framework
- a prompt generator
- a task execution tracker
- a token-minimizing workflow engine
- a human-in-the-loop coding assistant coordinator

It should be able to:

1. initialize a project workspace
2. ingest a plan document
3. create a task graph
4. generate context packets
5. decide which model/tool should handle each task
6. prepare model-specific prompts
7. optionally execute tasks through provider adapters
8. save outputs
9. summarize outputs
10. update Obsidian project memory
11. update graph memory
12. track progress
13. escalate failed tasks
14. generate reports

---

## 4. What Orca Agent Is Not

In the MVP, Orca Agent is not:

- a fully autonomous coding system with no approval gates
- a replacement for Cursor IDE
- a replacement for Claude Code
- a replacement for Codex
- a giant always-on agent daemon
- a general personal assistant
- a user-memory system
- a web app
- a cloud platform
- a self-modifying system

The MVP should be boring, reliable, inspectable, and file-based.

---

## 5. Model and Tool Strategy

### 5.1 Available resources

| Tool / Model | Role | Cost/usage assumption | Best use |
|---|---|---:|---|
| Ollama Cloud / local Ollama | Workhorse | Low | summarization, compression, memory updates, docs, simple code |
| Claude Code Opus 4.7 | Architect | High | planning, decomposition, architecture, risk analysis |
| Codex GPT-5.5 | Scoped executor | Medium | clear implementation plans, tests, patches |
| Cursor SDK + Composer 2.5 | Main repo-aware executor | High availability | repo-aware edits, multi-file coding, semantic search, hooks, skills |
| Cursor premium models | Specialist | Scarce | high-risk bugs, failed attempts, complex repo-wide refactors |

### 5.2 Recommended default loop

```text
1. Ollama creates or compresses context.
2. Claude plans only when task ambiguity or architecture complexity is high.
3. Cursor Composer 2.5 executes most repo-aware coding tasks.
4. Codex executes clean scoped implementation tasks.
5. Ollama summarizes result and updates project memory.
6. Cursor premium is used only after approval.
```

### 5.3 Practical routing rule

Use this mental model:

```text
Need cheap text processing?         → Ollama
Need architecture or planning?      → Claude Opus
Need exact patch from exact files?  → Codex
Need repo discovery/context?        → Cursor Composer
Need elite rescue/debugging?        → Cursor premium
```

---

## 6. System Goals

### 6.1 Primary goals

1. Minimize premium token usage.
2. Preserve context across model switches.
3. Learn the project over time.
4. Produce reusable task packets.
5. Route tasks to the cheapest capable tool.
6. Keep human-readable memory in Obsidian.
7. Support repo-aware execution through Cursor SDK.
8. Keep provider adapters modular.
9. Support manual and automated execution modes.
10. Make every decision auditable.

### 6.2 Secondary goals

1. Track model performance by task type.
2. Support project graph relationships.
3. Support skills, hooks, and rules.
4. Generate ready-to-paste prompts for external tools.
5. Support future web UI.
6. Support future CI/CD execution.
7. Support self-hosted workers later.

---

## 7. MVP Scope

The MVP should be a CLI that can operate without fully automated provider calls.

### 7.1 MVP must include

1. `orca init`
2. `orca ingest-plan plan.md`
3. `orca plan`
4. `orca tasks list`
5. `orca context TASK-ID`
6. `orca route TASK-ID`
7. `orca prompt TASK-ID --provider cursor-composer`
8. `orca result add TASK-ID result.md`
9. `orca memory update TASK-ID`
10. `orca status`
11. `orca report`

### 7.2 MVP should store

```text
.orca/
  config.yaml
  task-graph.yaml
  state.json
  runs/
  context-packets/
  prompts/
  results/
  memory/
  logs/
```

### 7.3 MVP should write an Obsidian vault

```text
orca-vault/
  00_Project_Overview.md
  01_Goals.md
  02_Architecture.md
  03_Current_State.md
  04_Task_Graph.md
  05_Decisions.md
  06_Known_Issues.md
  07_Model_Routing.md
  08_Token_Strategy.md
  tasks/
  context-packets/
  results/
  decisions/
  models/
```

### 7.4 MVP may include provider calls, but does not require them

Provider automation can be added incrementally.

MVP execution modes:

```yaml
execution_modes:
  manual:
    required: true
  ollama_api:
    optional: true
  cursor_sdk:
    optional: true
  claude_code:
    optional: false_for_mvp
  codex_cli:
    optional: false_for_mvp
```

---

## 8. High-Level Architecture

```text
┌─────────────────────────┐
│   Project Plan / PRD    │
└────────────┬────────────┘
             ↓
┌─────────────────────────┐
│   Plan Ingestion Agent  │
└────────────┬────────────┘
             ↓
┌─────────────────────────┐
│    Task Graph Agent     │
│ Claude or Ollama        │
└────────────┬────────────┘
             ↓
┌─────────────────────────┐
│  Project Memory Agent   │
│ Obsidian + file store   │
└────────────┬────────────┘
             ↓
┌─────────────────────────┐
│ Context Packet Builder  │
│ mostly Ollama           │
└────────────┬────────────┘
             ↓
┌─────────────────────────┐
│      Model Router       │
└──────┬──────┬──────┬─────┘
       ↓      ↓      ↓
   Ollama  Codex  Claude  Cursor SDK
       ↓      ↓      ↓      ↓
┌─────────────────────────┐
│    Execution Result     │
└────────────┬────────────┘
             ↓
┌─────────────────────────┐
│  Review + Summarizer    │
│ mostly Ollama/Claude    │
└────────────┬────────────┘
             ↓
┌─────────────────────────┐
│ Memory + Graph Updater  │
└────────────┬────────────┘
             ↓
┌─────────────────────────┐
│      Next Task          │
└─────────────────────────┘
```

---

## 9. Core Agents

### 9.1 Plan Ingestion Agent

Purpose: read a plan document and extract goals, constraints, phases, and candidate tasks.

Inputs:

- `plan.md`
- existing project memory
- optional repo summary

Outputs:

- normalized project summary
- extracted goals
- extracted constraints
- candidate task list

### 9.2 Task Graph Agent

Purpose: convert plan into a dependency-aware task graph.

Usually Claude for complex plans, Ollama for simple plans.

Outputs:

- `task-graph.yaml`
- task dependencies
- complexity estimates
- recommended model per task
- validation steps

### 9.3 Context Packet Agent

Purpose: create compact task-specific context packets.

Usually Ollama.

Inputs:

- project memory
- task metadata
- selected file summaries
- previous task summaries

Outputs:

- `context-packets/TASK-ID.md`

### 9.4 Router Agent

Purpose: select model/provider/runtime.

Inputs:

- task type
- task complexity
- context size
- file count
- previous failures
- budget policy
- model performance history

Outputs:

- provider recommendation
- model ID
- runtime mode
- reason
- approval requirement

### 9.5 Execution Agent

Purpose: send task to a model or generate a ready-to-paste prompt.

Execution modes:

- manual prompt generation
- Cursor SDK execution
- Ollama API execution
- future Claude/Codex adapters

### 9.6 Review Agent

Purpose: review outputs and decide if the task is complete.

Can use Ollama for cheap review or Claude for high-risk review.

Outputs:

- validation status
- detected risks
- follow-up tasks
- memory update proposal

### 9.7 Memory Update Agent

Purpose: update durable project memory after every task.

Outputs:

- Obsidian task note
- current state update
- decision update
- known issue update
- model performance note
- graph relationships

---

## 10. Execution Flow

### 10.1 New project flow

```text
orca init
orca ingest-plan docs/plan.md
orca plan --planner claude
orca tasks list
orca context TASK-001
orca route TASK-001
orca prompt TASK-001 --provider cursor-composer
```

Then either:

```text
# Manual mode
Paste prompt into Cursor/Claude/Codex/Ollama.
Save output as results/TASK-001.md.
orca result add TASK-001 results/TASK-001.md
orca memory update TASK-001
```

or:

```text
# Automated mode
orca run TASK-001 --provider cursor-composer
orca memory update TASK-001
```

### 10.2 Per-task lifecycle

```text
pending
  ↓
context_ready
  ↓
routed
  ↓
prompt_generated
  ↓
running
  ↓
result_received
  ↓
reviewed
  ↓
memory_updated
  ↓
complete
```

### 10.3 Failure lifecycle

```text
running
  ↓
failed
  ↓
failure_summarized
  ↓
repair_plan_created
  ↓
retry_routed
  ↓
running
```

After repeated failure:

```text
failed_twice
  ↓
escalation_required
  ↓
human_approval
  ↓
cursor_premium_or_claude_diagnosis
```

---

## 11. Model Routing Policy

### 11.1 Default priority

```yaml
model_priority:
  context_and_memory:
    - ollama
  repo_aware_execution:
    - cursor_composer
  scoped_execution:
    - codex
  planning:
    - claude_opus
  final_escalation:
    - cursor_premium
```

### 11.2 Routing matrix

| Task type | First choice | Second choice | Escalation |
|---|---|---|---|
| Context compression | Ollama | Cursor Composer | Claude |
| Project summary | Ollama | Claude | none |
| Architecture plan | Claude | Cursor premium | none |
| Task decomposition | Claude | Ollama | none |
| Simple docs | Ollama | Cursor Composer | none |
| Simple code | Ollama or Codex | Cursor Composer | none |
| Scoped patch | Codex | Cursor Composer | Claude review |
| Repo-aware patch | Cursor Composer | Codex with packet | Cursor premium |
| Multi-file refactor | Cursor Composer | Claude plan + Cursor Composer | Cursor premium |
| Hard debugging | Cursor Composer | Claude diagnosis | Cursor premium |
| PR automation | Cursor SDK Cloud | Cursor SDK Local | manual |
| Memory update | Ollama | deterministic template | Claude |

### 11.3 Router decision rules

```yaml
rules:
  - id: use_ollama_for_compression
    if:
      task.type in: [summary, compression, memory_update, docs]
    then:
      provider: ollama

  - id: use_claude_for_architecture
    if:
      task.type in: [architecture, decomposition, risk_analysis]
      task.complexity in: [high, critical]
    then:
      provider: claude_opus

  - id: use_cursor_composer_for_repo_aware_work
    if:
      task.requires_repo_search: true
      or:
        task.estimated_files_touched_gte: 3
    then:
      provider: cursor_sdk
      model_family: composer

  - id: use_codex_for_scoped_patches
    if:
      task.type in: [implementation, tests, refactor]
      task.context_is_exact: true
      task.estimated_files_touched_lte: 2
    then:
      provider: codex

  - id: premium_requires_approval
    if:
      provider in: [cursor_premium, claude_opus_long_context]
    then:
      require_manual_approval: true

  - id: retry_then_escalate
    if:
      task.failure_count_gte: 2
    then:
      require_escalation_review: true
```

### 11.4 Cursor budget split

```yaml
cursor_budget:
  composer:
    usage_level: abundant
    default_allowed: true
    manual_approval_required: false

  premium_models:
    usage_level: scarce
    default_allowed: false
    manual_approval_required: true
    allowed_only_if:
      - task.failed_with_composer == true
      - task.failed_with_codex == true
      - task.risk == high
      - task.complexity == critical
      - task.requires_deep_repo_reasoning == true
```

---

## 12. Context Packet Design

### 12.1 Purpose

A context packet is the smallest complete unit of context required to execute a task.

It prevents model switching from losing context and prevents premium models from reading irrelevant history.

### 12.2 Context packet principles

1. Include goal.
2. Include exact task.
3. Include only relevant project memory.
4. Include exact constraints.
5. Include important decisions.
6. Include relevant files or file summaries.
7. Include validation requirements.
8. Include required output format.
9. Exclude raw chat history.
10. Exclude unrelated tasks.

### 12.3 Context packet template

```markdown
# Context Packet: {{task_id}} — {{task_title}}

## Project
Name: {{project_name}}
Short summary: {{project_summary}}
Current phase: {{current_phase}}

## Task
ID: {{task_id}}
Title: {{task_title}}
Type: {{task_type}}
Complexity: {{complexity}}
Status: {{status}}

## Objective
{{objective}}

## Why This Task Matters
{{why_it_matters}}

## Relevant Project Memory
{{relevant_memory_summary}}

## Relevant Decisions
{{decisions}}

## Relevant Files
{{#files}}
- `{{path}}`: {{reason}}
{{/files}}

## Known Constraints
{{constraints}}

## Known Risks
{{risks}}

## Dependencies
{{dependencies}}

## Previous Related Work
{{previous_work_summary}}

## Instructions
Do:
{{do_list}}

Do not:
{{do_not_list}}

## Validation Steps
{{validation_steps}}

## Required Output Format
Return:
1. Summary of changes
2. Files changed or affected
3. Tests run or validation performed
4. Risks or unresolved issues
5. Suggested memory update
6. Follow-up tasks
```

### 12.4 Context packet size targets

```yaml
context_packet_size_targets:
  small:
    max_tokens: 1500
    use_for: [ollama, codex, cursor_composer]
  medium:
    max_tokens: 4000
    use_for: [cursor_composer, claude]
  large:
    max_tokens: 8000
    use_for: [claude_only_when_justified, cursor_premium]
```

### 12.5 Context reduction order

When context is too large, remove/compress in this order:

1. raw logs
2. old task history
3. unrelated decisions
4. verbose file summaries
5. repeated constraints
6. secondary validation notes
7. low-priority examples

Never remove:

- task objective
- constraints
- relevant decisions
- safety rules
- validation requirements

---

## 13. Project Memory Design

### 13.1 Memory categories

```text
Project identity
Project goals
Architecture
Current state
Tasks
Decisions
Known issues
Code map
Model routing history
Token strategy
Validation history
```

### 13.2 Memory files

```text
.orca/memory/
  project-overview.md
  goals.md
  architecture.md
  current-state.md
  decisions.md
  known-issues.md
  code-map.md
  model-performance.md
  token-strategy.md
  task-history.md
```

### 13.3 Memory update rules

After every task, update:

1. task history
2. current state
3. decisions if new decisions were made
4. known issues if errors were found
5. code map if files/modules changed
6. model performance if the task succeeded/failed
7. token notes if context was too large

### 13.4 Memory record example

```markdown
## TASK-014 — Implement Cursor SDK provider

Status: Complete
Model used: Cursor Composer 2.5
Date: 2026-05-18
Files changed:
- `src/providers/cursor-sdk-provider.ts`
- `src/types/provider.ts`

Summary:
Implemented the initial Cursor SDK provider adapter with local execution mode and streaming output capture.

Decisions:
- Model ID is configured through `CURSOR_COMPOSER_MODEL_ID` instead of hardcoded.
- Cloud execution is left as a later phase.

Validation:
- Typecheck passed.
- Provider interface tests passed.

Follow-up:
- Add cloud runtime support.
- Add approval gate for premium Cursor models.
```

---

## 14. Obsidian Vault Design

### 14.1 Why Obsidian

Obsidian gives the project memory a human-readable knowledge base.

Benefits:

- easy manual inspection
- links between concepts
- graph view
- markdown-native
- works well with Git
- no vendor lock-in
- can be edited by human or agent

### 14.2 Vault structure

```text
orca-vault/
  00_Project_Overview.md
  01_Goals.md
  02_Architecture.md
  03_Current_State.md
  04_Task_Graph.md
  05_Decisions.md
  06_Known_Issues.md
  07_Code_Map.md
  08_Model_Routing.md
  09_Token_Strategy.md
  10_Rules.md
  11_Skills.md
  12_Hooks.md
  tasks/
    TASK-001.md
    TASK-002.md
  context-packets/
    TASK-001-context.md
  results/
    TASK-001-result.md
  decisions/
    DEC-001-use-context-packets.md
  models/
    ollama.md
    claude-opus.md
    codex-gpt.md
    cursor-composer.md
    cursor-premium.md
```

### 14.3 Task note frontmatter

```yaml
---
task_id: TASK-001
title: Implement provider interface
status: complete
type: implementation
complexity: medium
assigned_provider: cursor_sdk
assigned_model: composer_2_5
files_touched:
  - src/providers/index.ts
  - src/types/provider.ts
depends_on:
  - TASK-000
created: 2026-05-18
updated: 2026-05-18
---
```

### 14.4 Decision note template

```markdown
# DEC-001 — Use context packets as model handoff unit

## Status
Accepted

## Context
Long chat histories are expensive and fragile when switching between models.

## Decision
Every task execution must be based on a compact context packet.

## Consequences
- Reduced token usage.
- Easier model switching.
- Requires memory update discipline.

## Related Tasks
- [[TASK-001]]
- [[TASK-002]]
```

---

## 15. Graph Memory Design

### 15.1 Purpose

Graph memory captures relationships that are hard to maintain in flat markdown.

Examples:

```text
TASK-012 touches FILE src/router.ts
TASK-012 depends_on TASK-005
DECISION-003 affects TASK-012
ERROR-002 occurred_in TASK-012
MODEL cursor_composer succeeded_on TASK-012
FILE src/router.ts implements CONCEPT model_routing
```

### 15.2 Relationship types

```yaml
nodes:
  - Project
  - Task
  - File
  - Concept
  - Decision
  - Error
  - Model
  - Provider
  - Skill
  - Hook
  - Rule

edges:
  - depends_on
  - touches
  - implements
  - affected_by
  - caused
  - fixed_by
  - succeeded_on
  - failed_on
  - requires
  - validates
  - summarizes
```

### 15.3 Graph query examples

```text
Find all tasks that touched the provider system.
Find all decisions that affect Cursor SDK integration.
Find files related to context packet generation.
Find models that failed on refactor tasks.
Find tasks blocked by missing provider adapters.
```

### 15.4 MVP graph implementation

Do not start with a complex database unless needed.

MVP options:

1. JSON edge list
2. SQLite tables
3. markdown links in Obsidian
4. graphify later

Recommended MVP:

```text
.orca/graph/nodes.json
.orca/graph/edges.json
```

Later, integrate graphify or another graph visualization layer.

---

## 16. Skills, Hooks, and Rules

### 16.1 Skills

Skills are reusable procedures.

Recommended skills:

```text
create-context-packet
compress-context
decompose-plan
route-task
execute-scoped-patch
execute-repo-aware-task
review-result
update-project-memory
summarize-failure
create-repair-plan
generate-cursor-brief
generate-codex-brief
generate-claude-plan-brief
```

### 16.2 Skill file example

```markdown
# Skill: create-context-packet

## Purpose
Create a compact, task-specific context packet for model handoff.

## Steps
1. Read task metadata.
2. Read current project memory.
3. Identify relevant decisions.
4. Identify relevant files or summaries.
5. Include only context needed for this task.
6. Enforce max token target.
7. Write packet to `.orca/context-packets/{{task_id}}.md`.

## Output
A markdown context packet following the standard template.
```

### 16.3 Hooks

Hooks run before or after important events.

Recommended hooks:

```yaml
before_context_build:
  - load_project_memory
  - check_task_dependencies
  - estimate_context_size

before_model_call:
  - verify_context_packet_exists
  - estimate_token_cost
  - enforce_budget_policy
  - require_approval_if_premium

before_cursor_sdk_call:
  - verify_clean_git_status
  - verify_repo_path
  - check_cursor_model_budget

before_codex_call:
  - ensure_exact_files_known
  - enforce_patch_output

before_claude_call:
  - compress_context_with_ollama
  - require_planning_scope

before_ollama_call:
  - prefer_small_context

after_model_call:
  - save_raw_output
  - summarize_result
  - extract_files_changed
  - extract_decisions
  - propose_memory_update

on_failure:
  - summarize_failure
  - increment_failure_count
  - create_repair_plan
  - maybe_escalate
```

### 16.4 Rules

Rules are permanent constraints.

Recommended default rules:

```yaml
rules:
  - Never send full project history unless explicitly approved.
  - Always create a context packet before model execution.
  - Always update project memory after task completion.
  - Prefer Ollama for summarization and compression.
  - Prefer Cursor Composer for repo-aware coding.
  - Prefer Codex for scoped patch execution.
  - Prefer Claude for planning and architecture.
  - Cursor premium models require manual approval.
  - Do not invent file names, APIs, or schemas.
  - Ask for clarification if required inputs are missing.
  - Prefer minimal diffs over broad rewrites.
  - Preserve existing project style.
  - Do not mark a task complete without validation notes.
  - Every task result must include follow-up tasks if applicable.
```

---

## 17. Cursor SDK Integration

### 17.1 Role of Cursor SDK

Cursor SDK should be a first-class provider adapter in Orca Agent.

It is especially useful because it provides:

- programmatic agent runs
- local repo execution
- cloud execution
- repo indexing
- semantic search
- grep
- MCP support
- `.cursor/skills/`
- `.cursor/hooks.json`
- subagents
- PR creation in cloud mode

### 17.2 Cursor runtime modes

```yaml
cursor_runtimes:
  local:
    use_when:
      - repo is cloned locally
      - task needs local files
      - fast iteration is desired
      - no PR automation needed

  cloud:
    use_when:
      - repo is on GitHub
      - isolated VM is useful
      - task may run long
      - automatic PR creation is useful

  self_hosted:
    use_when:
      - code must stay inside private network
      - you want your own workers
      - future phase
```

### 17.3 Cursor model policy

```yaml
cursor_models:
  composer_2_5:
    role: default_repo_aware_executor
    usage: abundant
    approval_required: false

  premium:
    role: final_escalation_executor
    usage: scarce
    approval_required: true
```

### 17.4 Cursor SDK provider pseudocode

```typescript
import { Agent } from "@cursor/sdk";

export async function runCursorTask(input: CursorTaskInput) {
  const agent = await Agent.create({
    apiKey: process.env.CURSOR_API_KEY!,
    model: { id: input.modelId },
    local: {
      cwd: input.repoPath,
    },
  });

  const run = await agent.send(input.prompt);

  let output = "";

  for await (const event of run.stream()) {
    output += serializeCursorEvent(event);
  }

  return {
    taskId: input.taskId,
    provider: "cursor_sdk",
    modelId: input.modelId,
    output,
  };
}
```

### 17.5 Important implementation note

Do not hardcode the Composer 2.5 model ID until verified against the Cursor SDK model list or account configuration.

Use config:

```env
CURSOR_API_KEY=your_key
CURSOR_COMPOSER_MODEL_ID=cursor-composer-model-id-here
CURSOR_PREMIUM_MODEL_ID=cursor-premium-model-id-here
```

The Cursor blog sample used `composer-2`, but the actual Composer 2.5 identifier should be confirmed from Cursor SDK docs/account availability.

### 17.6 Cursor task prompt requirements

Cursor prompts should emphasize:

- search before editing
- minimal diffs
- preserve existing style
- tests if behavior changes
- final report
- suggested memory update

---

## 18. Provider Adapter Design

### 18.1 Provider interface

```typescript
export interface ModelProvider {
  id: string;
  displayName: string;
  capabilities: ProviderCapability[];
  runTask(input: TaskExecutionInput): Promise<TaskExecutionResult>;
  estimateCost?(input: TaskExecutionInput): Promise<CostEstimate>;
  validateConfig?(): Promise<ProviderConfigStatus>;
}
```

### 18.2 Provider types

```typescript
export type ProviderId =
  | "manual"
  | "ollama"
  | "claude_code"
  | "codex"
  | "cursor_sdk";
```

### 18.3 Task execution input

```typescript
export interface TaskExecutionInput {
  taskId: string;
  projectRoot: string;
  repoPath?: string;
  contextPacketPath: string;
  prompt: string;
  modelId?: string;
  runtime?: "manual" | "local" | "cloud" | "self_hosted";
  approval?: ApprovalState;
  metadata: Record<string, unknown>;
}
```

### 18.4 Task execution result

```typescript
export interface TaskExecutionResult {
  taskId: string;
  provider: ProviderId;
  modelId?: string;
  status: "success" | "failure" | "partial" | "cancelled";
  rawOutputPath: string;
  summary?: string;
  filesChanged?: string[];
  validation?: ValidationResult;
  suggestedMemoryUpdate?: string;
  followUpTasks?: string[];
  error?: string;
}
```

### 18.5 Manual provider

Manual provider is critical for MVP.

It should:

1. generate prompt file
2. tell user where to paste it
3. wait for user to add result
4. continue memory workflow

---

## 19. CLI Command Design

### 19.1 Command overview

```text
orca init
orca ingest-plan <file>
orca plan
orca tasks list
orca tasks show <taskId>
orca context <taskId>
orca route <taskId>
orca prompt <taskId> --provider <provider>
orca run <taskId> --provider <provider>
orca result add <taskId> <file>
orca review <taskId>
orca memory update <taskId>
orca status
orca report
orca graph export
orca config show
orca config set <key> <value>
```

### 19.2 `orca init`

Creates:

```text
.orca/
orca-vault/
.orca/config.yaml
.orca/state.json
```

Options:

```text
orca init --project-name "My Project" --repo ./my-repo --vault ./orca-vault
```

### 19.3 `orca ingest-plan`

Reads a markdown plan and creates initial memory notes.

```text
orca ingest-plan plan.md
```

Outputs:

```text
.orca/input/plan.md
.orca/memory/project-overview.md
orca-vault/00_Project_Overview.md
```

### 19.4 `orca plan`

Creates task graph.

```text
orca plan --planner claude
orca plan --planner ollama
orca plan --manual
```

### 19.5 `orca context`

Generates context packet.

```text
orca context TASK-001
orca context TASK-001 --max-tokens 2500
orca context TASK-001 --refresh
```

### 19.6 `orca route`

Shows routing decision.

```text
orca route TASK-001
```

Example output:

```text
Recommended provider: cursor_sdk
Recommended model: composer_2_5
Runtime: local
Reason:
- Task requires repo search.
- Estimated files touched: 4.
- Composer usage is abundant.
Approval required: no
```

### 19.7 `orca prompt`

Generates a provider-specific prompt.

```text
orca prompt TASK-001 --provider cursor-composer
orca prompt TASK-001 --provider codex
orca prompt TASK-001 --provider claude
orca prompt TASK-001 --provider ollama
```

### 19.8 `orca run`

Executes through adapter if configured.

```text
orca run TASK-001 --provider cursor_sdk --runtime local
orca run TASK-001 --provider ollama
```

### 19.9 `orca result add`

Adds manual output.

```text
orca result add TASK-001 ./result.md
```

### 19.10 `orca memory update`

Updates memory after result.

```text
orca memory update TASK-001
```

---

## 20. Configuration Design

### 20.1 `.orca/config.yaml`

```yaml
project:
  name: Orca Agent CLI
  repo_path: .
  vault_path: orca-vault

execution:
  default_mode: manual
  require_approval_for_premium: true
  require_clean_git_for_cursor: true

models:
  ollama:
    base_url: http://localhost:11434
    default_model: qwen3.5:4b
    role: context_and_memory

  claude:
    provider: claude_code
    default_model: opus-4.7
    role: planning
    approval_required_for_large_context: true

  codex:
    provider: codex
    default_model: gpt-5.5
    role: scoped_execution

  cursor:
    provider: cursor_sdk
    composer_model_id: ${CURSOR_COMPOSER_MODEL_ID}
    premium_model_id: ${CURSOR_PREMIUM_MODEL_ID}
    default_runtime: local
    cloud_auto_create_pr: false

routing:
  default_repo_executor: cursor_composer
  default_scoped_executor: codex
  default_planner: claude
  default_summarizer: ollama
  max_failures_before_escalation: 2

context:
  default_max_tokens: 3000
  hard_max_tokens: 8000
  prefer_summaries_over_raw_files: true

memory:
  update_after_each_task: true
  write_obsidian: true
  write_graph: true
```

---

## 21. Data Schemas

### 21.1 Task graph schema

```yaml
project_goal: string
current_phase: string
phases:
  - id: string
    name: string
    objective: string
    tasks:
      - id: string
        title: string
        description: string
        type: planning | implementation | refactor | testing | docs | review | memory | research
        status: pending | context_ready | routed | running | blocked | failed | complete
        complexity: low | medium | high | critical
        risk: low | medium | high
        depends_on: string[]
        required_context:
          - string
        likely_files:
          - path: string
            reason: string
        requires_repo_search: boolean
        estimated_files_touched: number
        recommended_provider: ollama | claude_code | codex | cursor_sdk | manual
        recommended_model: string
        validation:
          - string
        success_criteria:
          - string
        notes: string
```

### 21.2 State schema

```json
{
  "projectName": "Orca Agent CLI",
  "currentPhase": "MVP",
  "tasks": {
    "TASK-001": {
      "status": "pending",
      "failureCount": 0,
      "assignedProvider": null,
      "assignedModel": null,
      "contextPacketPath": null,
      "resultPath": null,
      "updatedAt": "2026-05-18T00:00:00Z"
    }
  }
}
```

### 21.3 Model performance schema

```yaml
models:
  cursor_composer:
    successful_tasks: 12
    failed_tasks: 2
    best_for:
      - repo_aware_implementation
      - multi_file_edits
    avoid_for:
      - architecture_planning
    notes:
      - Good on provider refactors.
      - Sometimes needs stricter minimal diff instructions.

  codex:
    successful_tasks: 8
    failed_tasks: 1
    best_for:
      - scoped_patch
      - test_generation
```

---

## 22. Storage Design

### 22.1 Local folder structure

```text
.orca/
  config.yaml
  state.json
  input/
    plan.md
  task-graph.yaml
  context-packets/
    TASK-001.md
  prompts/
    TASK-001.cursor-composer.md
    TASK-001.codex.md
  results/
    TASK-001.raw.md
    TASK-001.summary.md
  memory/
    project-overview.md
    goals.md
    architecture.md
    current-state.md
    decisions.md
    known-issues.md
    code-map.md
    model-performance.md
    token-strategy.md
    task-history.md
  graph/
    nodes.json
    edges.json
  logs/
    orca.log
```

### 22.2 Why file-based first

File-based storage is:

- easy to inspect
- easy to Git commit
- easy to debug
- easy for models to read
- easy to migrate later

Do not start with a complex database unless needed.

---

## 23. Prompt Library

This section contains the core prompts Orca Agent should use or generate.

### 23.1 Global Orca system prompt

```text
You are Orca Agent, a project orchestration agent.

Your purpose is to coordinate multiple AI models and coding tools to complete project tasks while minimizing token usage and preserving project context.

You do not learn the user.
You learn the project.

Your responsibilities:
1. Understand the project goal.
2. Maintain durable project memory.
3. Break plans into dependency-aware tasks.
4. Route tasks to the cheapest capable model.
5. Generate compact context packets.
6. Prevent context loss across model handoffs.
7. Update project memory after every task.
8. Track decisions, known issues, files, and model performance.
9. Escalate only when needed.
10. Keep outputs structured and auditable.

Model usage policy:
- Use Ollama for summaries, compression, memory updates, documentation, and simple repetitive work.
- Use Cursor Composer 2.5 for most repo-aware coding tasks.
- Use Codex GPT-5.5 for scoped implementation tasks where exact context is known.
- Use Claude Opus for planning, architecture, decomposition, and risk analysis.
- Use Cursor premium models only after approval for high-risk or failed tasks.

Token policy:
- Never send full project history unless explicitly approved.
- Always create a context packet before model execution.
- Summarize long outputs before storing them in memory.
- Prefer diffs over full files.
- Prefer file summaries over raw file content unless raw code is required.

Memory policy:
After every task, update:
- task history
- current state
- decisions
- known issues
- code map
- model performance
- follow-up tasks

Safety policy:
- Do not invent file names, schemas, APIs, or project facts.
- If required information is missing, ask for clarification.
- Do not perform broad refactors unless explicitly requested.
- Do not use premium models without approval.
```

### 23.2 Plan ingestion prompt

```text
You are the Plan Ingestion Agent for Orca Agent CLI.

Your job is to read a project plan document and extract the structured information needed to build a task graph and project memory.

Input:
- Project plan markdown
- Existing project memory if available

Extract:
1. Project name
2. Project purpose
3. Primary goals
4. Non-goals
5. Constraints
6. Required integrations
7. Architecture hints
8. Major phases
9. Candidate tasks
10. Risks
11. Unknowns and assumptions

Rules:
- Do not invent requirements.
- Preserve important wording from the plan.
- If something is ambiguous, mark it as unknown.
- Keep the output structured.

Output format:

# Project Summary

# Goals

# Non-Goals

# Constraints

# Integrations

# Candidate Phases

# Candidate Tasks

# Risks

# Unknowns
```

### 23.3 Claude planner prompt

```text
You are the Claude Planning Agent for Orca Agent CLI.

Your job is to turn the project plan and project memory into a dependency-aware implementation task graph.

You are used for architecture, planning, decomposition, sequencing, and risk analysis.
You should not write full implementation code unless explicitly requested.

Available tools/models in the orchestration system:

1. Ollama / Ollama Cloud
   - Cheap workhorse.
   - Best for summaries, compression, memory updates, docs, and simple tasks.

2. Cursor SDK + Composer 2.5
   - Main repo-aware coding executor.
   - Best for coding tasks that need repository indexing, semantic search, grep, multi-file awareness, hooks, skills, MCP, or PR automation.

3. Codex GPT-5.5
   - Scoped implementation executor.
   - Best when exact files and exact instructions are known.

4. Claude Opus
   - Planning and architecture model.
   - Best for complex reasoning, decomposition, design, and risk analysis.

5. Cursor premium models
   - Scarce final escalation path.
   - Use only for high-risk failed tasks.

Your task:
Create a build plan and task graph for the provided project.

Requirements:
1. Start with a minimal CLI MVP.
2. Avoid overengineering.
3. Use file-based storage first.
4. Include task dependencies.
5. Include recommended provider/model per task.
6. Include validation steps per task.
7. Include risks and unknowns.
8. Include memory updates required per phase.
9. Include when to use Cursor Composer vs Codex vs Ollama vs Claude.
10. Include clear acceptance criteria.

Output as YAML task graph plus a short explanation.

Do not invent APIs or model IDs.
Use configurable model IDs where required.
```

### 23.4 Router agent prompt

```text
You are the Model Router for Orca Agent CLI.

Your job is to choose the cheapest capable provider/model/runtime for a task.

Inputs:
- Task metadata
- Context packet summary
- Project memory
- Model performance history
- Budget policy
- Failure count

Available routes:

Ollama:
- summaries
- compression
- memory updates
- docs
- simple repetitive tasks

Cursor Composer 2.5:
- default repo-aware coding executor
- multi-file edits
- semantic code search
- moderate debugging
- applying plans inside a repo

Codex GPT-5.5:
- scoped implementation
- exact file patches
- tests
- clear refactors

Claude Opus:
- planning
- architecture
- decomposition
- risk analysis

Cursor premium:
- final escalation
- high-risk debugging
- tasks that failed with Composer/Codex
- requires manual approval

Decision rules:
1. If task is summary/compression/memory/docs, choose Ollama.
2. If task is architecture/decomposition/high-level planning, choose Claude.
3. If task needs repo discovery or touches 3+ files, choose Cursor Composer.
4. If task has exact files and exact instructions, choose Codex.
5. If task failed twice or is critical risk, recommend escalation.
6. Premium routes require manual approval.

Output format:

provider: string
model: string
runtime: string
approval_required: boolean
confidence: low | medium | high
reasoning:
  - string
fallback:
  provider: string
  model: string
```

### 23.5 Context packet builder prompt

```text
You are the Context Packet Builder for Orca Agent CLI.

Your job is to create a compact context packet for a single task.

Inputs:
- Task metadata
- Project memory
- Relevant decisions
- Relevant file summaries
- Previous task summaries

Rules:
- Include only context needed for this task.
- Do not include full project history.
- Do not include unrelated tasks.
- Prefer concise summaries over raw content.
- Preserve constraints and decisions exactly.
- If important information is missing, add an Unknowns section.

Output must follow this structure:

# Context Packet: TASK-ID — Task Title

## Project

## Task

## Objective

## Relevant Project Memory

## Relevant Decisions

## Relevant Files

## Constraints

## Risks

## Dependencies

## Instructions

## Validation Steps

## Required Output Format
```

### 23.6 Cursor Composer execution prompt

```text
You are the Cursor Composer execution agent inside Orca Agent CLI.

You are the default repo-aware coding executor.

Your job is to complete the assigned coding task using the repository context available to you.

Rules:
1. Treat the task packet as the source of truth.
2. Search the repo before editing.
3. Make the smallest correct change.
4. Preserve existing architecture and style.
5. Do not perform broad refactors unless explicitly requested.
6. Do not invent project goals, file names, APIs, or schemas.
7. If requirements are unclear, stop and ask for clarification.
8. Add or update tests when behavior changes.
9. Avoid unrelated formatting changes.
10. Return a concise execution report.

Task Packet:
{{context_packet}}

Required final output:
1. Summary of changes
2. Files changed
3. Tests run or validation performed
4. Risks or unresolved issues
5. Suggested memory update
6. Follow-up tasks
```

### 23.7 Codex execution prompt

```text
You are the Codex execution agent inside Orca Agent CLI.

You are used for scoped implementation tasks where the exact files and instructions are known.

Rules:
1. Follow the task packet exactly.
2. Make minimal changes.
3. Prefer a unified diff or exact changed functions.
4. Do not rewrite unrelated files.
5. Do not invent missing APIs, schemas, or filenames.
6. If required information is missing, ask for clarification.
7. Include tests if behavior changes.
8. Return only the required output format.

Task Packet:
{{context_packet}}

Required output:
1. Patch or exact changes
2. Explanation summary
3. Tests to run
4. Risks
5. Suggested memory update
```

### 23.8 Ollama summarizer prompt

```text
You are the Ollama Summarizer for Orca Agent CLI.

Your job is to summarize task outputs cheaply and accurately for project memory.

Input:
- Task packet
- Raw model output
- Optional diff or validation output

Extract:
1. What changed
2. Files changed or affected
3. Decisions made
4. Errors encountered
5. Validation performed
6. Follow-up tasks
7. Suggested memory update

Rules:
- Be concise.
- Do not add new claims that are not supported by the input.
- If validation is missing, say validation is missing.
- If the task failed, summarize the failure clearly.

Output format:

# Task Result Summary

## Task

## Status

## Changes

## Files Affected

## Decisions

## Validation

## Risks

## Follow-Up Tasks

## Memory Update
```

### 23.9 Review agent prompt

```text
You are the Review Agent for Orca Agent CLI.

Your job is to review whether a task result satisfies the task packet.

Inputs:
- Task packet
- Task result summary
- Raw output if needed
- Validation output if available

Check:
1. Was the objective satisfied?
2. Were constraints followed?
3. Were unrelated changes avoided?
4. Were tests or validation included?
5. Are there unresolved risks?
6. Should the task be marked complete, partial, or failed?

Output format:

status: complete | partial | failed
confidence: low | medium | high
reasons:
  - string
missing_validation:
  - string
risks:
  - string
recommended_next_action: string
memory_update_approved: true | false
```

### 23.10 Memory update prompt

```text
You are the Project Memory Update Agent for Orca Agent CLI.

Your job is to update durable project memory from a completed or failed task.

Inputs:
- Existing project memory
- Task packet
- Task result summary
- Review result

Update the following sections as needed:
1. Current state
2. Task history
3. Decisions
4. Known issues
5. Code map
6. Model performance
7. Follow-up tasks

Rules:
- Do not duplicate existing memories.
- Keep memory concise and useful.
- Record decisions only if an actual decision was made.
- Record known issues only if there is a concrete issue.
- Do not mark failed work as complete.

Output:
A memory update proposal in markdown with target file names and replacement/append instructions.
```

### 23.11 Failure diagnosis prompt

```text
You are the Failure Diagnosis Agent for Orca Agent CLI.

A task failed or produced an unsatisfactory result.

Your job is to diagnose why and create a repair plan.

Inputs:
- Original task packet
- Raw failed output
- Error logs
- Previous attempts
- Model used

Analyze:
1. Did the model misunderstand the objective?
2. Was context missing?
3. Was the task routed to the wrong model?
4. Were file/API details missing?
5. Did validation fail?
6. Should the task be retried, split, or escalated?

Output format:

# Failure Diagnosis

## Likely Cause

## Missing Context

## Routing Issue

## Repair Plan

## Recommended Next Provider

## Revised Task Packet Notes

## Escalation Required
yes/no
```

### 23.12 Cursor premium escalation prompt

```text
You are the Cursor Premium escalation agent inside Orca Agent CLI.

This task is being escalated because cheaper/default routes failed or the task is high risk.

Your goals:
1. Understand the previous failures.
2. Avoid repeating failed approaches.
3. Use repository context carefully.
4. Make the smallest correct change.
5. Validate thoroughly.
6. Return a clear final report.

Inputs:
- Original task packet
- Failure summaries
- Repair plan
- Relevant project memory

Rules:
- Do not perform broad refactors unless required.
- Do not ignore previous failure notes.
- If the task is still ambiguous, stop and ask for clarification.
- Prefer correctness over speed.

Required output:
1. Root cause
2. Changes made
3. Files changed
4. Validation performed
5. Remaining risks
6. Memory update
7. Follow-up tasks
```

---

## 24. Implementation Roadmap

### Phase 0 — Design lock

Goal: finalize MVP boundaries.

Tasks:

- choose TypeScript runtime
- choose CLI framework
- choose config format
- choose storage format
- confirm provider execution modes
- confirm Cursor model IDs later

Recommended stack:

```text
Language: TypeScript
Runtime: Node.js
CLI: commander or clipanion
Validation: zod
YAML: yaml package
Markdown: plain file templates first
Testing: vitest
Formatting: prettier
Linting: eslint
```

### Phase 1 — Project scaffolding

Build:

- CLI entrypoint
- config loader
- path resolver
- logging
- `.orca/` initializer
- vault initializer

Commands:

```text
orca init
orca config show
orca status
```

### Phase 2 — Plan ingestion

Build:

- markdown plan loader
- deterministic extraction placeholders
- manual plan summary support
- initial project memory writer

Commands:

```text
orca ingest-plan plan.md
```

### Phase 3 — Task graph system

Build:

- task graph schema
- task graph file writer
- task list/show commands
- manual task graph editing support

Commands:

```text
orca plan --manual
orca tasks list
orca tasks show TASK-001
```

### Phase 4 — Context packet generator

Build:

- context packet template
- memory selector
- relevant decision selector
- task packet writer

Commands:

```text
orca context TASK-001
```

### Phase 5 — Router

Build:

- routing policy engine
- provider recommendation output
- approval checks
- model budget config

Commands:

```text
orca route TASK-001
```

### Phase 6 — Prompt generator

Build:

- provider-specific prompt templates
- prompt file writer
- manual execution instructions

Commands:

```text
orca prompt TASK-001 --provider cursor-composer
orca prompt TASK-001 --provider codex
orca prompt TASK-001 --provider claude
orca prompt TASK-001 --provider ollama
```

### Phase 7 — Manual result workflow

Build:

- add result command
- summarize result command
- review result command
- memory update proposal

Commands:

```text
orca result add TASK-001 result.md
orca review TASK-001
orca memory update TASK-001
```

### Phase 8 — Ollama provider

Build:

- Ollama API client
- summarization calls
- context compression calls
- safe timeout/retry

Commands:

```text
orca run TASK-001 --provider ollama
```

### Phase 9 — Cursor SDK provider

Build:

- Cursor SDK adapter
- local runtime support
- streaming output capture
- configurable model IDs
- Composer vs premium approval gate

Commands:

```text
orca run TASK-001 --provider cursor_sdk --model composer
```

### Phase 10 — Graph memory MVP

Build:

- nodes.json
- edges.json
- graph update after task
- graph export command

Commands:

```text
orca graph export
```

---

## 25. Detailed Build Tasks

### TASK-001 — Initialize TypeScript CLI project

Type: implementation  
Model: Cursor Composer or Codex  
Complexity: low  
Validation:

```text
npm run build
npm test
orca --help
```

### TASK-002 — Implement config loader

Type: implementation  
Model: Codex  
Complexity: low  
Outputs:

- load `.orca/config.yaml`
- validate with zod
- environment variable substitution

### TASK-003 — Implement `orca init`

Type: implementation  
Model: Cursor Composer  
Complexity: medium  
Outputs:

- creates `.orca/`
- creates vault
- writes default config
- writes initial state

### TASK-004 — Implement task graph schema

Type: implementation  
Model: Codex  
Complexity: medium  
Outputs:

- zod schema
- YAML parse/write
- validation errors

### TASK-005 — Implement plan ingestion

Type: implementation  
Model: Cursor Composer  
Complexity: medium  
Outputs:

- copies plan
- writes project overview
- creates initial memory files

### TASK-006 — Implement manual task graph generation

Type: implementation  
Model: Codex  
Complexity: medium  
Outputs:

- creates skeleton task graph
- validates edited graph

### TASK-007 — Implement context packet generator

Type: implementation  
Model: Cursor Composer  
Complexity: medium-high  
Outputs:

- reads task graph
- reads memory
- writes context packet

### TASK-008 — Implement routing engine

Type: implementation  
Model: Codex or Cursor Composer  
Complexity: medium  
Outputs:

- deterministic routing rules
- explanation output

### TASK-009 — Implement prompt generator

Type: implementation  
Model: Codex  
Complexity: medium  
Outputs:

- provider templates
- prompt files

### TASK-010 — Implement manual result workflow

Type: implementation  
Model: Cursor Composer  
Complexity: medium  
Outputs:

- result add
- result summary placeholder
- status update

### TASK-011 — Implement memory update workflow

Type: implementation  
Model: Cursor Composer  
Complexity: medium-high  
Outputs:

- task history append
- current state update
- Obsidian task note update

### TASK-012 — Implement Ollama adapter

Type: implementation  
Model: Codex  
Complexity: medium  
Outputs:

- OpenAI-compatible or Ollama native client
- summarizer support

### TASK-013 — Implement Cursor SDK adapter

Type: implementation  
Model: Cursor Composer  
Complexity: high  
Outputs:

- local run
- stream capture
- config validation
- approval gates

### TASK-014 — Implement graph memory MVP

Type: implementation  
Model: Codex  
Complexity: medium  
Outputs:

- nodes/edges JSON
- graph update after task

### TASK-015 — Add tests and validation suite

Type: testing  
Model: Codex  
Complexity: medium  
Outputs:

- unit tests
- fixture project
- CLI smoke tests

---

## 26. Testing Strategy

### 26.1 Unit tests

Test:

- config loading
- env substitution
- task graph validation
- routing decisions
- context packet rendering
- prompt rendering
- state transitions
- memory append logic

### 26.2 Fixture project

Create a tiny fixture project:

```text
fixtures/sample-project/
  plan.md
  src/index.ts
  package.json
```

Use it to test:

```text
orca init
orca ingest-plan plan.md
orca plan --manual
orca context TASK-001
orca route TASK-001
orca prompt TASK-001 --provider codex
orca result add TASK-001 result.md
orca memory update TASK-001
```

### 26.3 Routing tests

Examples:

```text
summary task → Ollama
architecture high complexity → Claude
repo-aware 4 files → Cursor Composer
scoped exact 1 file → Codex
failed twice → escalation required
premium model → approval required
```

### 26.4 Safety tests

Ensure:

- premium model cannot run without approval
- missing context packet blocks execution
- invalid task ID fails clearly
- unknown provider fails clearly
- config validation catches missing Cursor API key before SDK execution

---

## 27. Security, Safety, and Approval Gates

### 27.1 Required approval cases

Manual approval required for:

- Cursor premium models
- cloud runtime with auto PR creation
- destructive shell commands
- tasks touching many files above threshold
- high-risk refactors
- deleting files
- modifying secrets or environment files

### 27.2 Git safety

Before automated repo execution:

```text
check git status
warn if dirty
optionally require clean working tree
create branch per task
save diff after task
```

### 27.3 Secrets policy

Never include these in context packets:

- API keys
- tokens
- `.env` contents
- SSH keys
- private certificates
- passwords

### 27.4 Context redaction

Add a redaction step before model calls:

```text
.env values → [REDACTED_ENV]
API keys → [REDACTED_API_KEY]
private keys → [REDACTED_PRIVATE_KEY]
```

---

## 28. Token Minimization Strategy

### 28.1 Core tactics

1. Use context packets.
2. Use Ollama to summarize long memory.
3. Keep task results summarized.
4. Prefer file paths and summaries over full code.
5. Include raw code only when necessary.
6. Use Cursor Composer for repo-aware search instead of manually pasting many files.
7. Use Codex only when exact context is small.
8. Use Claude only for planning and high-value reasoning.
9. Use premium Cursor only after failure or high risk.

### 28.2 Compression pipeline

```text
Raw project state
  ↓
select relevant memory
  ↓
select relevant decisions
  ↓
select relevant file summaries
  ↓
Ollama compression
  ↓
context packet
```

### 28.3 Output control

Always ask models for structured output:

```text
Return only:
1. Summary
2. Files changed
3. Tests run
4. Risks
5. Memory update
```

Avoid:

```text
Explain everything in detail.
```

Except during planning documents like this one.

---

## 29. Failure Handling and Escalation

### 29.1 Failure types

```yaml
failure_types:
  missing_context:
    action: rebuild_context_packet

  wrong_model:
    action: reroute_task

  ambiguous_task:
    action: ask_claude_for_repair_plan

  code_error:
    action: retry_with_cursor_composer_or_codex

  repeated_failure:
    action: escalate_with_approval
```

### 29.2 Escalation ladder

```text
Ollama failed simple task
  → Codex or Cursor Composer

Codex failed scoped coding task
  → Cursor Composer

Cursor Composer failed repo-aware task
  → Claude diagnosis
  → Cursor Composer retry

Retry failed
  → Ask approval for Cursor premium
```

### 29.3 Failure summary template

```markdown
# Failure Summary: {{task_id}}

## Task
{{task_title}}

## Model Used
{{model}}

## What Failed
{{failure}}

## Evidence
{{logs_or_output}}

## Likely Cause
{{cause}}

## Missing Context
{{missing_context}}

## Recommended Next Step
{{next_step}}
```

---

## 30. Example End-to-End Workflow

### Step 1: Initialize

```bash
orca init --project-name "Orca Agent CLI" --repo . --vault orca-vault
```

### Step 2: Ingest plan

```bash
orca ingest-plan docs/orca-plan.md
```

### Step 3: Generate task graph

```bash
orca plan --planner claude
```

### Step 4: List tasks

```bash
orca tasks list
```

### Step 5: Generate context

```bash
orca context TASK-007
```

### Step 6: Route task

```bash
orca route TASK-007
```

Output:

```text
Provider: cursor_sdk
Model: composer_2_5
Runtime: local
Reason: task requires repo-aware implementation and touches multiple files.
Approval required: no
```

### Step 7: Generate prompt

```bash
orca prompt TASK-007 --provider cursor-composer
```

### Step 8: Execute manually or automatically

Manual:

```bash
# paste generated prompt into Cursor or Cursor SDK workflow
orca result add TASK-007 results/TASK-007.md
```

Automated later:

```bash
orca run TASK-007 --provider cursor_sdk --runtime local
```

### Step 9: Review and update memory

```bash
orca review TASK-007
orca memory update TASK-007
orca status
```

---

## 31. Acceptance Criteria

### 31.1 MVP acceptance criteria

The MVP is acceptable when:

1. A new project can be initialized.
2. A markdown plan can be ingested.
3. A task graph can be created or loaded.
4. Tasks can be listed and inspected.
5. A context packet can be generated for a task.
6. A routing recommendation can be produced.
7. A provider-specific prompt can be generated.
8. A manual result can be added.
9. Project memory can be updated.
10. Obsidian vault files are written.
11. Status and report commands work.
12. Routing rules protect premium models.
13. The system can run without any paid API configured.

### 31.2 Good v1 acceptance criteria

V1 is acceptable when:

1. Ollama adapter works.
2. Cursor SDK local adapter works.
3. Cursor Composer can execute a task.
4. Premium Cursor model requires approval.
5. Basic graph memory is updated.
6. Tests cover core CLI workflows.
7. The tool can complete its own small implementation task loop.

---

## 32. Future Roadmap

### 32.1 Web UI

Add a dashboard:

- task graph viewer
- model route viewer
- context packet viewer
- memory browser
- token usage estimates
- run logs
- approval buttons

### 32.2 Cloud and PR automation

Use Cursor SDK cloud runtime for:

- isolated agent execution
- auto branch creation
- auto PR creation
- CI failure fix tasks

### 32.3 Self-hosted worker support

Future option:

- run Cursor/agent workers on private machines
- keep code local
- integrate with homelab resources

### 32.4 Graph UI

Use graphify or other graph tooling to visualize:

- tasks
- files
- decisions
- models
- errors

### 32.5 CI/CD integration

Trigger Orca on:

- failed tests
- PR creation
- issue labels
- scheduled maintenance

### 32.6 Project-specific agent packs

Create reusable packs:

```text
Unity project pack
Rust backend pack
TypeScript CLI pack
Docker/self-hosting pack
```

Each pack can include:

- routing defaults
- skills
- hooks
- memory templates
- validation commands

---

## 33. Recommended First Implementation Prompt

Use this prompt with Cursor Composer 2.5 or Codex to start the implementation.

```text
You are implementing the MVP of Orca Agent CLI.

Build a TypeScript Node.js CLI tool that orchestrates AI coding tasks through project memory, task graphs, context packets, routing, prompts, and manual result workflows.

Important:
- Implement only the MVP.
- Do not overengineer.
- Use file-based storage.
- Do not implement every provider yet.
- Prioritize manual workflow first.
- Keep provider adapters modular for future Cursor SDK, Ollama, Claude, and Codex integrations.

MVP commands:
1. orca init
2. orca ingest-plan <file>
3. orca tasks list
4. orca tasks show <taskId>
5. orca context <taskId>
6. orca route <taskId>
7. orca prompt <taskId> --provider <provider>
8. orca result add <taskId> <file>
9. orca memory update <taskId>
10. orca status
11. orca report

Technical requirements:
- TypeScript
- Node.js
- Commander or similar CLI library
- Zod for schema validation
- YAML for config and task graph
- File-based storage under `.orca/`
- Obsidian-compatible vault output under `orca-vault/`
- Provider prompt templates for Ollama, Claude, Codex, Cursor Composer, and Cursor premium
- Deterministic routing engine using config rules
- No hardcoded premium model execution
- Premium model routes require approval

Implement project structure:

src/
  cli.ts
  commands/
  config/
  tasks/
  context/
  routing/
  prompts/
  memory/
  providers/
  graph/
  utils/
  schemas/

tests/
  fixtures/

Validation:
- npm run build
- npm test
- orca --help
- orca init works in a temp directory
- ingest-plan writes memory files
- context command writes a context packet
- route command recommends the correct model for sample tasks
- prompt command writes provider-specific prompts

Do not add Cursor SDK execution yet unless the MVP is complete.
Add clean provider interfaces and TODOs for later provider implementations.
```

---

## Final Recommendation

Build Orca Agent CLI in this order:

```text
1. File-based CLI foundation
2. Plan ingestion
3. Task graph
4. Context packets
5. Routing
6. Prompt generation
7. Manual results
8. Memory updates
9. Ollama adapter
10. Cursor SDK Composer adapter
11. Graph memory
12. Automated execution and PR workflows
```

The key to success is not model power alone. The key is the loop:

```text
small task → compact context → correct model → structured result → memory update
```

If Orca Agent enforces that loop, it will save tokens, preserve context, and let each model do the work it is best at.
