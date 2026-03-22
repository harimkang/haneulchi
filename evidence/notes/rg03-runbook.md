# RG-03 Operator Runbook

Selected tools: `yazi`, `lazygit`, `vim/nvim`, `tmux` 중 실제 검증 대상 3개 이상

Preflight:
1. Open Haneulchi and route to `Project Focus`.
2. Confirm the selected project is using a live hosted terminal pane.
3. Prepare clipboard text `RG03 paste check`.
4. Keep the output artifact directory open so screenshots can be saved under `evidence/screens/`.

Run each selected tool inside a live Haneulchi `Project Focus / Terminal Deck` pane.

For each tool:
1. Launch it from the hosted terminal.
2. Use the tool long enough to confirm input is not dropped.
3. Confirm alternate screen enter/exit works if the tool uses it.
4. Resize the pane and confirm the tool remains usable.
5. Paste clipboard content and confirm it appears correctly.
6. Quit the tool and confirm the shell prompt returns inside the same pane.

Required evidence per tool:
- one screen capture under `evidence/screens/`
- one checklist note under `evidence/notes/`
- one caveat note under `evidence/notes/`

Recommended launch commands:
- `yazi`: `yazi`
- `lazygit`: `lazygit`
- `vim`: `vim README.md`
- `nvim`: `nvim README.md`
- `tmux`: `tmux new -A -s haneulchi-rg03`
