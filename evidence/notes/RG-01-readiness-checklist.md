# RG-01 Readiness Checklist

Automated regression run on `2026-03-22`:
- `bash scripts/smoke.sh readiness` passed.
- `bash scripts/smoke.sh launcher` passed.

Validation status:
1. `WF-01` first-launch entry surface:
Automated: `AppShellBootstrapTests.appShellBootstrapsLauncherWhenNoProjectExists`
Manual: pending operator screenshot in `screens/S-01-welcome.png`
2. Readiness rows for `shell`, `git`, `preset`, `keychain`, and `workflow`:
Automated: `ReadinessProbeRunnerTests`, `WelcomeReadinessViewModelTests`
Manual: pending launcher screenshot confirmation
3. `Add Folder` / selected-project summary handoff:
Automated: launcher bootstrap and view-model coverage passed
Manual: pending `NSOpenPanel` operator run
4. `Continue with Generic Shell` on degraded preset/workflow state:
Automated: `ReadinessProbeRunnerTests.readinessReportAllowsGenericShellFallback`, `AppShellBootstrapTests.liveDefaultKeepsShellEntryForInformationalGaps`
Manual: pending first shell prompt capture
5. `Open Settings` and `Retry` launcher actions:
Automated: settings route coverage passed; launcher smoke passed
Manual: pending interactive retry click-through
6. First shell command transcript in `logs/S-01-first-run.log`:
Automated: `scripts/qa/readiness/run-rg01-pack.sh` seeds the acceptance-pack log with suite output and leaves a manual capture marker
Manual: pending operator run
