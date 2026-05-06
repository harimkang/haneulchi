<p align="center">
  <img src="./public/assets/haneulchi-readme-banner.png" alt="Haneulchi whale carrying terminal panes through a dark developer workspace" width="100%" />
</p>

# Haneulchi

**Terminal-first command center for AI coding agents.**

Haneulchi is a local-first desktop workspace for developers who run multiple projects, terminals, agents, reviews, budgets, workflows, and evidence packs at the same time. It combines a Tauri desktop shell, a React control surface, a native SQLite state store, a Unix-domain-socket control API, and the `hc` CLI into one production-oriented agent operations console.

[한국어 README](./README.ko.md)

## Why Haneulchi

AI-assisted engineering gets messy when terminals, task boards, review evidence, workflow contracts, security approvals, and cost controls live in separate tools. Haneulchi keeps those surfaces close to the terminal, where the real work happens.

- **Operate from the terminal deck.** Run local PTYs, persistent native sessions, SSH-like session metadata, command capture, terminal transcript chunks, and guarded input from one workspace.
- **Coordinate agents and projects.** Track project tabs, session stacks, reusable skill packs, runtime pools, task boards, run queues, review queues, and external tracker syncs.
- **Preserve review evidence.** Persist command blocks, run replay metadata, generated evidence packs, review decisions, follow-up tasks, transcript summaries, and PR landing plans.
- **Control risk and cost.** Gate dangerous actions through policy approvals, redact secrets, inspect permission audit logs, monitor token usage, enforce budgets, and forecast runway.
- **Stay local-first.** Core state lives in SQLite and artifacts on disk, with native Tauri commands and a local HTTP/JSON control API over a Unix domain socket.

## Current Status

This repository contains the Haneulchi desktop implementation:

- Tauri v2 desktop shell with a React, TypeScript, and Vite frontend.
- SQLite-backed projects, sessions, tasks, runs, agents, skills, budgets, knowledge, workflow versions, evidence, policy approvals, and quality records.
- xterm.js terminal panes with WebGL preference, fallback/degraded rendering, ordered PTY streaming, OSC allowlist parsing, and command block extraction.
- Local control API and native command coverage for workspace state, health, projects, sessions, tasks, runs, reviews, policy, secrets, agents, providers, terminal themes, budgets, knowledge, workflows, quality gates, visual harness links, tracker sync, browser automation, and update checks.
- Production-oriented tests across Rust state/control logic, TypeScript service clients, frontend surfaces, design tokens, release packaging, and CI-safe docs-backed compliance checks.

## Product Surfaces

Haneulchi includes terminal-first surfaces for daily development and agent operations:

- Terminal Deck, project sidebar, workspace tabs, compact right rail, command palette, task board, run queue, review queue, file explorer, diff preview, and localhost browser preview.
- roadmap timeline and calendar views for task cycle/module planning, initiative rollups, token totals, and cost summaries.
- skill pack registry and runtime pool summaries that connect durable skill packs, context packs, active workloads, local sessions, SSH sessions, and cloud agent runs.
- historical analytics charts and dashboard widget visibility controls for run lifecycle health, evidence completeness, budget state, and optional Control Tower panels.
- visual workflow debugger and workflow marketplace import for inspecting workflow runtime steps, diagnostics, and built-in preset reloads.
- network sandbox profiles and advanced permission audit filtering in the policy and security diagnostics surface.
- budget forecasts and provider price update workflows for cost runway review and provider price table refreshes.
- visual harness graph canvas with drag-to-create context/tool/task links and persisted manual link creation.
- Linear, GitHub, and Plane tracker sync adapters with binding diagnostics and dry-run sync commands.

## CLI Coverage

The `hc` CLI mirrors the local control API for automation and scripting. Implemented command families include:

- `state --json`, `health --json`
- project list|add|focus|detach|tab-group|files|file|write-file|diff|lsp|patch-export|patch-import|pr-plan|search-files|layout
- session list|new|focus|usage|input|stream|attach-task|detach-task|takeover|release|kill
- task list|create|edit|move|assign|comment|comments|open|workpad|context|planning
- initiative list|create
- block search|show|explain|export|attach
- dispatch
- run list|open|replay|usage|transition|cancel|retry|status|hook
- evidence generate|review
- review list|accept|changes|block|follow-up|pr-plan
- policy approvals|packs|audit|pack set|request|evaluate|decide
- secret list|set
- agent list|scan|register|pause|resume|heartbeat|events ingest
- agent list|scan|register|pause|resume|heartbeat|skill-packs|skill-pack set|runtime-pool|events ingest
- provider-model get|set
- terminal-theme get|set
- budget status|explain|dashboard|forecast|prices|record|set|export|ingest
- knowledge sources|source add|page create|search|open|explorations|exploration create|concepts|obsidian export|chat|lint|compile|ingest
- context list|create|show|attach
- workflow validate|reload|status|negative-tests|negative-test-runs
- release-gate run|list
- terminal smoke|smoke-runs
- task lifecycle-e2e|lifecycle-e2e-runs
- distribution dmg-smoke|dmg-smoke-runs
- recovery drills|drill-runs
- benchmark run|runs
- dogfood telemetry-review|telemetry-reviews
- visual graph|links|link
- tracker bindings|bind|<linear|github|plane> sync
- browser run
- update check

## Architecture

```text
React/Vite desktop UI
  -> Tauri command layer
  -> Rust state store and control API
  -> SQLite, PTY manager, local artifacts, release/quality scripts

hc CLI
  -> Unix-domain-socket HTTP/JSON control API
  -> same Rust state and mutation paths as the desktop app
```

The app is intentionally local-first. Network-facing integrations are modeled as explicit adapters, dry-run syncs, or policy-gated operations.

## Getting Started

Prerequisites:

- Node.js 22 or newer
- Rust stable
- macOS for the desktop release packaging path

Install dependencies:

```sh
npm install
```

Run the desktop web shell:

```sh
npm run dev
```

Run the Tauri desktop app:

```sh
npm run tauri dev
```

Build the frontend:

```sh
npm run build
```

Run tests:

```sh
npm test
cargo test --manifest-path src-tauri/Cargo.toml
```

Visual QA route checks:

```sh
npm run test:visual:qa
```

## Release Tooling

macOS packaging scripts are wired through npm:

```sh
npm run release:macos:app
npm run release:macos:dmg
npm run release:macos:notarize
npm run release:macos:verify
npm run release:update-feed:verify
```

Release workflows require Apple signing/notarization secrets and should be run from protected GitHub environments.

## Security Model

Haneulchi is designed around explicit local control:

- Secret metadata is persisted without exposing plaintext in state snapshots or generated evidence.
- Dangerous terminal input and policy-sensitive actions route through approval records.
- Transcript and evidence generation paths redact common secret patterns.
- Workflow hooks and project file operations enforce workspace/path boundaries.
- Network sandbox profiles distinguish localhost-safe operations from remote network access.

Please report security issues privately. See [SECURITY.md](./SECURITY.md).

## CI and Private Specs

The public repository intentionally excludes `docs/` and `reference/`. Docs-backed compliance tests run when the private docs are present locally and are skipped when those files are absent in CI. Runtime, service, frontend, release-packaging, and Rust tests remain active without the private docs.

## Contributing

Haneulchi is early, but contributions should keep the same bar:

- Prefer local-first behavior and explicit policy gates.
- Keep native, CLI, API, and frontend behavior aligned.
- Add tests for any new state mutation, command surface, or release/security behavior.
- Avoid committing generated private docs, local state, credentials, or build outputs.

## License

MIT. See [LICENSE](./LICENSE).
