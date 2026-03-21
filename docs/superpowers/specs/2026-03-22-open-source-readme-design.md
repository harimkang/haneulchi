# Open-Source README Design for Haneulchi

## Metadata
- Date: 2026-03-22
- Status: Approved design for implementation planning
- Target file: `README.md`
- Audience: balanced between first-time visitors and contributors

## 1. Purpose
Design a new root `README.md` that introduces Haneulchi as an open-source product while still giving contributors a credible path to evaluate, run, and navigate the repository.

The README should feel polished and product-aware, but it must stay accurate to the current product definition and repository state.

## 2. Source Basis
The README content should be grounded in the following repository materials, even when those files are not linked directly from the public README:

- `docs/Haneulchi.md`
- `docs/PRD/Haneulchi_PRD_v8_closure_synced.md`
- `docs/architecture/Haneulchi_Architecture_Document_v2.md`
- `docs/theme/BRAND_THEME.md`
- `.gitignore`
- current repository structure and tracked files

## 3. Goals
- Explain what Haneulchi is in clear English.
- Lead with product positioning before development mechanics.
- Preserve the product's terminal-first and review-first identity.
- Make the repository approachable for open-source contributors.
- Avoid stale or non-existent setup and smoke instructions.
- Mention only tracked, public-facing paths that are safe to expose in the root README.

## 4. Non-Goals
- Do not turn the README into an internal documentation index.
- Do not promise full autonomy, IDE replacement, or features not grounded in the current PRD and architecture docs.
- Do not reproduce deep implementation details that belong in subsystem-specific guides.
- Do not reference ignored files or directories from `.gitignore`.

## 5. Core Positioning
Haneulchi should be presented as:

> A terminal-first workspace for running, reviewing, and operating multi-project agent sessions on macOS.

Expanded framing:

- Haneulchi keeps the terminal at the center instead of replacing it.
- It adds operator visibility, workflow control, and reviewable outcomes around that terminal workflow.
- It is built for people who already live in CLI-heavy environments and want stronger session operations without losing manual control.

## 6. Messaging Pillars
The README should consistently reinforce these ideas:

1. `terminal-first`
Haneulchi is built around strong terminal workflows, not around hiding them.

2. `multi-project and multi-session operations`
The product is about handling more than one active flow without losing context.

3. `reviewable outcomes`
Outputs should come back in a form a human can inspect, accept, or hand off.

4. `human-in-the-loop control`
Automation is bounded, explicit, and visible.

5. `local control plane`
UI, CLI, and supporting runtime surfaces are described as one coordinated local system.

## 7. Prohibited Framing
The README must not use or imply the following claims:

- `full IDE replacement`
- `fully autonomous development`
- `AI that does everything for you`
- `one-click software factory`
- `replace your terminal`
- any claim that people are no longer needed in the loop

Preferred alternatives:

- `keeps the terminal at the center`
- `built for reviewable outcomes`
- `keeps human judgment in the loop`
- `adds an operator control plane on top of terminal work`

## 8. Audience and Tone
Primary audience:

- new visitors trying to understand the product
- potential contributors deciding whether the repository is approachable

Tone requirements:

- concise
- confident
- technical without sounding defensive
- product-aware, not marketing-heavy
- aligned with the brand's quiet-power tone

The README should sound open-source and credible, not theatrical.

## 9. README Information Architecture
The README should follow this order.

### 9.1 Hero
Contents:

- project name
- one-line positioning statement
- one short paragraph explaining that Haneulchi is a macOS terminal-first workspace for multi-project and multi-agent operation

This section should establish product identity before any commands appear.

### 9.2 Why It Exists
Contents:

- short explanation that terminals, tasks, reviews, worktrees, and operator visibility are often fragmented
- short explanation that Haneulchi brings those flows together without replacing the terminal

This section should stay short and readable.

### 9.3 What Makes It Different
Use a compact list of product principles instead of a long feature dump.

Recommended points:

- terminal-first by default
- reviewable outcomes instead of opaque automation
- explicit human control
- shared control plane across UI, CLI, and runtime
- first-class multi-project and multi-session operations

### 9.4 How It Works
Use a simple four-step flow:

1. Plan
2. Run
3. Review
4. Operate

This section should translate the PRD's `task -> session -> worktree -> evidence -> handoff` model into README-friendly language.

### 9.5 Quick Start
This section must be practical and restrained.

Allowed commands should only reference paths and scripts that currently exist, such as:

- `just check`
- `scripts/bootstrap/ensure-runtime-dirs.sh`
- `scripts/build-macos-core.sh`
- `scripts/smoke.sh`

Rules:

- do not include stale MVP script names that are not present in the repository
- include a short macOS-only note
- keep this section short enough for first-time scanning

### 9.6 Project Layout
Keep the repo map short and public-safe.

Recommended entries:

- `apps/macos`
- `crates`
- `scripts`
- `fixtures`
- `tests`

Each line should describe purpose, not internals.

### 9.7 Repository Guides
If the implementation includes a guide section, it must only link to tracked, public files.

Safe candidates:

- `apps/macos/README.md`
- `crates/README.md`
- `scripts/qa/README.md`

This section is optional. If it feels redundant, omit it.

### 9.8 Open Source and Contributing
State that Haneulchi is being built in the open.

Contents:

- issues and pull requests are welcome
- contributors should start by running checks
- contributors should keep product messaging aligned with the terminal-first and reviewable-outcomes model

### 9.9 Current Status
State the current state plainly.

Required ideas:

- macOS-only
- active development
- terminal, app-shell, and control-plane work is still evolving

### 9.10 Notes
Optional short closing notes may include:

- runtime data lives outside the repository
- secrets belong in Keychain, not committed files

## 10. Public Path Constraint from `.gitignore`
The root README must not mention ignored files or directories.

Ignored paths that must not be referenced in `README.md` include:

- `docs/README.md`
- `docs/PRD/`
- `docs/architecture/`
- `docs/execution/`
- `docs/tasks/`
- `reference/`
- any other `docs/*` path that remains ignored by default

Implication:

- the README may be grounded in those documents, but it must not point readers to them
- the README should prefer tracked repository paths outside ignored documentation trees

## 11. Existing README Cleanup Requirements
The new README should explicitly correct the current root README's main issues:

- it is too scaffold-oriented for an open-source landing page
- it starts with implementation detail before product meaning
- it includes stale smoke script references that do not exist in `scripts/`
- it references ignored content such as `reference/`

## 12. Implementation Guidance
When rewriting `README.md`:

- prefer short paragraphs over large bullet walls
- keep section count manageable
- avoid repeating the same phrase in every section
- ensure every command and path mentioned is real and tracked
- keep contributor guidance lightweight
- make the top half useful for visitors and the middle useful for contributors

## 13. Acceptance Criteria
The README design is successful when the implemented `README.md` satisfies all of the following:

1. It is written in English.
2. It presents Haneulchi as a terminal-first macOS workspace with reviewable outcomes and human-in-the-loop control.
3. It follows the approved `product-first, then operator proof` structure.
4. It includes practical startup commands that exist in the current repository.
5. It does not mention ignored files or directories from `.gitignore`.
6. It does not claim IDE replacement or full autonomy.
7. It includes a short open-source and contribution entry point.
8. It is more persuasive for first-time visitors than the current scaffold-oriented README.

## 14. Verification Checklist for the Implementation Step
- Check every linked path with `test -e` or equivalent.
- Check every command target exists.
- Confirm no ignored path names appear in the final `README.md`.
- Review the top section for prohibited framing.
- Review the final section for accuracy about platform and project status.
