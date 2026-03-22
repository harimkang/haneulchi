# RG-04 Workflow Checklist

1. Validate a known-good `WORKFLOW.md` and confirm `state: ok`.
2. Open the workflow drawer and confirm watched path, last reload time, last-known-good hash, review checklist, allowed agents, and hooks are visible.
3. Capture the valid-path screenshot and write `notes/rg04-valid-operator-note.md`.
4. Introduce an invalid reload and confirm `invalid_kept_last_good` is shown.
5. Confirm the previous last-known-good hash remains visible after invalid reload.
6. Confirm the running session is not mutated by reload and that only future launch behavior changes.
7. Capture the invalid-path screenshot and write `notes/rg04-invalid-operator-note.md`.
