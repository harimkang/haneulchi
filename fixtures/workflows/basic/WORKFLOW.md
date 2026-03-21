---
name: Basic Scaffold Workflow
goal: Keep implementation aligned to the terminal-first control plane.
review:
  checklist:
    - Confirm tests relevant to the touched layer were run.
    - Confirm no secret material was committed.
workspace:
  strategy: worktree
  base_root: .
---

Focus on the requested task and keep UI, CLI, and API vocabulary aligned.
