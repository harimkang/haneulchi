# RG-04 Workflow Checklist

1. Validate a known-good `WORKFLOW.md` and confirm `state: ok`.
2. Open the workflow drawer and confirm path, last-known-good hash, review checklist, allowed agents, and hooks are visible.
3. Introduce an invalid reload and confirm `invalid_kept_last_good` is shown.
4. Confirm the previous last-known-good hash remains visible after invalid reload.
5. Confirm running sessions are not mutated by reload.
6. Capture one screen and one operator note for the valid path.
7. Capture one screen and one operator note for the invalid reload path.
