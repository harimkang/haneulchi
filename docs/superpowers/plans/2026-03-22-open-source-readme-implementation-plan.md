# Open-Source README Implementation Plan

> **For agentic workers:** REQUIRED: Use superpowers:subagent-driven-development (if subagents available) or superpowers:executing-plans to implement this plan. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Rewrite the root `README.md` into an English, open-source-facing product page that reflects Haneulchi's approved positioning while keeping commands, links, and public paths accurate to the live repository.

**Architecture:** This plan changes one public document, `README.md`, and treats the approved spec as the source of truth for structure, tone, and prohibited framing. Implementation should verify every referenced path and command against the live repository and use `.gitignore` as a hard public-surface boundary so the README never points to ignored docs, `landing/`, or `reference/`.

**Tech Stack:** Markdown, git, `rg`, `test`, `git check-ignore`, `just`, shell scripts

---

## Chunk 1: Rewrite the Public Narrative

### Task 1: Lock the live constraints before editing

**Files:**
- Modify: `README.md`
- Reference: `docs/superpowers/specs/2026-03-22-open-source-readme-design.md`
- Verify: `.gitignore`
- Verify: `justfile`
- Verify: `scripts/smoke.sh`
- Verify: `apps/macos/README.md`
- Verify: `crates/README.md`
- Verify: `scripts/qa/README.md`

- [ ] **Step 1: Confirm the ignored-path boundary**

Run:

```bash
git check-ignore -v landing reference docs/README.md apps/macos/README.md crates/README.md scripts/qa/README.md README.md
```

Expected:
- `landing`, `reference`, and `docs/README.md` report ignore rules
- `apps/macos/README.md`, `crates/README.md`, `scripts/qa/README.md`, and `README.md` do not report ignore rules

- [ ] **Step 2: Confirm the public guide files exist**

Run:

```bash
test -e apps/macos/README.md && test -e crates/README.md && test -e scripts/qa/README.md && echo GUIDE_PATHS_OK
```

Expected: `GUIDE_PATHS_OK`

- [ ] **Step 3: Confirm the startup and smoke commands exist**

Run:

```bash
sed -n '1,200p' justfile
sed -n '1,260p' scripts/smoke.sh
```

Expected:
- `just check` exists in `justfile`
- `scripts/smoke.sh` documents the supported smoke targets and aliases

- [ ] **Step 4: Snapshot the current README problems before rewriting**

Run:

```bash
sed -n '1,260p' README.md
```

Expected:
- the file starts with scaffold-oriented implementation wording
- the file mentions ignored or internal-only content that must be removed from the public README

### Task 2: Rewrite the README top half around product meaning first

**Files:**
- Modify: `README.md`
- Reference: `docs/superpowers/specs/2026-03-22-open-source-readme-design.md`

- [ ] **Step 1: Replace the opening scaffold text with the approved hero**

Write the top of `README.md` with this product-first structure:

```md
# Haneulchi

A terminal-first workspace for running, reviewing, and operating multi-project agent sessions on macOS.

Haneulchi keeps the terminal at the center, then adds the visibility needed to run multiple projects, sessions, and review loops without losing control.
```

Constraints:
- keep it in English
- do not mention ignored docs or `landing/`
- do not claim IDE replacement or full autonomy

- [ ] **Step 2: Add a short `Why Haneulchi Exists` section**

Write a compact section that explains:

- terminals, tasks, review, worktrees, and operator visibility are often fragmented
- Haneulchi brings those flows together without replacing the terminal
- if it reads naturally, use the three-shift framing:

```md
## Why Haneulchi Exists

- From tabs to sessions
- From output to evidence
- From automation to controllable flow
```

Keep this section short and explanatory, not slogan-heavy.

- [ ] **Step 3: Add `What Makes It Different` and `How It Works`**

Write sections that cover:

```md
## What Makes It Different

- Terminal-first by default
- Reviewable outcomes instead of opaque automation
- Explicit human control
- Shared control plane across UI, CLI, and runtime
- First-class multi-project and multi-session operations

## How It Works

1. Plan
2. Run
3. Review
4. Operate
```

Use README-friendly language that translates the spec's `task -> session -> worktree -> evidence -> handoff` model. If helpful, name a small number of concrete product surfaces such as `Review Queue`, `Attention Center`, or `Readiness Launcher`, but do not turn this into a full feature catalog.

- [ ] **Step 4: Review the top half before moving on**

Run:

```bash
sed -n '1,160p' README.md
```

Expected:
- product framing appears before any setup commands
- the README stays concise and technical
- no forbidden claims about autonomy or IDE replacement appear

## Chunk 2: Finish the Practical Sections and Validate the Public Surface

### Task 3: Rewrite the operational sections with live repo data

**Files:**
- Modify: `README.md`
- Reference: `justfile`
- Reference: `scripts/smoke.sh`
- Reference: `apps/macos/README.md`
- Reference: `crates/README.md`
- Reference: `scripts/qa/README.md`

- [ ] **Step 1: Replace `Quick Start` with commands that exist now**

Write a short `Quick Start` section using only live commands. Use this shape:

````md
## Quick Start

```bash
just check
scripts/bootstrap/ensure-runtime-dirs.sh
bash scripts/smoke.sh --help
```
````

Rules:
- keep the section short
- note that the project is macOS-only
- do not reintroduce deleted MVP-specific script names

- [ ] **Step 2: Rewrite `Project Layout` using only public-safe directories**

Write a repo map that only mentions:

```md
## Project Layout

- `apps/macos`
- `crates`
- `scripts`
- `fixtures`
- `tests`
```

Each bullet should explain purpose briefly. Do not mention `reference/`, `landing/`, or ignored docs.

- [ ] **Step 3: Add the closing sections**

Write short sections for:

```md
## Repository Guides
## Open Source
## Current Status
## Notes
```

Content rules:
- `Repository Guides` is optional; include it only if the three tracked guide files improve orientation
- `Open Source` should say issues and pull requests are welcome
- `Current Status` should say macOS-only, active development, and disciplined-MVP scope
- `Notes` may mention runtime data outside the repo and Keychain-only secrets

- [ ] **Step 4: Review the bottom half for drift**

Run:

```bash
sed -n '161,320p' README.md
```

Expected:
- every path in the lower half exists and is public-safe
- the closing sections stay short and accurate

### Task 4: Validate paths, commands, ignore safety, and final diff

**Files:**
- Modify: `README.md`
- Verify: `.gitignore`
- Verify: `justfile`
- Verify: `scripts/smoke.sh`

- [ ] **Step 1: Verify every README-linked guide path exists**

Run:

```bash
test -e apps/macos/README.md
test -e crates/README.md
test -e scripts/qa/README.md
```

Expected: all commands exit successfully

- [ ] **Step 2: Verify the command references still match the repo**

Run:

```bash
just --list
bash scripts/smoke.sh --help
```

Expected:
- `check` appears in the `just` task list
- `scripts/smoke.sh --help` prints the current smoke targets

- [ ] **Step 3: Verify ignored paths do not appear in the final README**

Run:

```bash
rg -n "docs/README.md|docs/PRD|docs/architecture|docs/execution|docs/tasks|landing/|reference/" README.md
```

Expected: no matches

- [ ] **Step 4: Review the final diff for structure and tone**

Run:

```bash
git diff -- README.md
```

Expected:
- the README begins with product positioning
- commands and links are real
- the copy stays within the approved scope and tone

- [ ] **Step 5: Commit the README rewrite**

Run:

```bash
git add README.md
git commit -m "docs: rewrite root readme for open-source positioning"
```

Expected: one docs-only commit containing the README rewrite
