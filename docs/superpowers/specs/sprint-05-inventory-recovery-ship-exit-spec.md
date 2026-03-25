# Sprint 05 — Inventory, Recovery & Ship Exit — Implementation Spec

> 기준 문서: Sprint 05 Execution v3, PRD v8, MVP Backlog v5, Architecture v2, DB Schema v2, FFI v3, CLI/API v2, `WORKFLOW.md` Runtime Semantics v1, Release Gate / Evidence Spec v1, Wireframe v5, UI Theme  
> 작성일: 2026-03-24  
> 목적: Sprint 5 작업을 위한 code-close implementation spec. 수동 operator 검증, evidence pack 작성, hosted release-gate promotion은 본 문서 범위에서 제외한다.

---

## 1. Sprint 목표

Sprint 5는 EP-07, EP-08, EP-09를 "구현 가능한 코드와 자동 검증" 범위까지 닫는 단계다. 이 sprint에서 닫아야 하는 핵심은 다음 네 가지다.

- Worktree / cache / restore 정보를 하나의 recovery-first inventory projection으로 통합한다.
- Settings / Secrets / Automation & Integrations surface를 실제 control-plane 상태와 연결한다.
- 실제 Keychain-backed secret injection, Rust-backed persistence, restore, degraded recovery를 구현한다.
- 수동 evidence 수집은 미루되, release gate를 뒷받침하는 자동 테스트와 보안/성능 측정 훅을 준비한다.

Sprint 5의 code-close 목표는 다음 기준을 직접 겨냥한다.

- AC-08: worktree & cache inventory에서 pin / clean / open이 가능해야 한다.
- AC-13: secrets가 Keychain ref를 통해 launch에 주입되어야 한다.
- AC-14: 프로젝트 / 태스크 / 세션 / 레이아웃 metadata가 재시작 후 복원되어야 한다.
- AC-15: degraded scenario에서 next safe action까지 3 actions 이내에 도달할 수 있어야 한다.
- WF-09: Settings가 diagnostics, secrets, worktree/recovery, automation/integrations를 실제 상태와 연결해야 한다.
- WF-13: Inventory가 cleanup 도구이면서 recovery 도구로 동작해야 한다.

---

## 2. Scope Lock

### 2.1 Sprint 5에 포함되는 범위

- `state.snapshot` 기반의 single-authority inventory / settings / recovery data path
- recovery-first grouping을 갖는 WF-13 Inventory UI
- WF-09 `Automation & Integrations` 탭과 workflow reload error 설명 surface
- 실제 macOS Keychain-backed secret ref CRUD 및 session launch env injection
- Rust SQLite 기반의 project / layout / session / worktree / app restore persistence
- startup restore, dead session archiving, stale claim reconcile
- degraded recovery code path
  - `preset_missing`
  - `missing_project_path`
  - `deleted_repo`
  - `keychain_ref_missing`
  - `invalid_workflow_reload`
  - `before_run_hook_failure`
  - `worktree_unreachable`
  - `crashed_restore`
  - `stale_claim_reconcile`
- local-only control surface guardrail
  - Unix domain socket only
  - user-scoped permission
  - secrets redaction from snapshot / session JSON / logs
- automated regression suites and machine-runnable smoke/metric hooks

### 2.2 Sprint 5에서 제외되는 범위

- 수동 TUI operator certification
  - `yazi`, `lazygit`, `vim` or `nvim`, `tmux`, IME 최종 실사용 판정
- screen capture, screenshot, video, runbook, checklist, evidence manifest 작성
- RG-01 ~ RG-10 최종 hosted promotion
- ship/no-ship 최종 판정
- subjective release wording 정리

### 2.3 Sprint 5 산출물의 성격

Sprint 5는 "ship verdict" sprint가 아니라 "ship verdict를 자동/운영 기준으로 판정할 수 있게 만드는 구현" sprint다. 따라서 release gate의 manual evidence는 후속 단계로 미루지만, 그 evidence를 채울 코드 경계와 자동화된 확인 경로는 Sprint 5 안에서 만들어야 한다.

---

## 3. 구현 원칙

### 3.1 `state.snapshot`는 계속 authoritative source다

- UI, CLI, API가 보는 운영 상태의 truth source는 계속 `state.snapshot` 이다.
- Inventory, Settings, Recovery는 derived query를 추가할 수 있지만 alternate truth source를 만들면 안 된다.
- 새 derived query는 snapshot 또는 Rust-side projection / repository에서 파생돼야 하며, enum 의미를 새로 정의하면 안 된다.

### 3.2 Inventory는 cleanup-first가 아니라 recovery-first다

- WF-13은 단순 파일 목록 화면이 아니다.
- inventory row는 "지울 수 있는가" 이전에 "복구 가능 상태인가 / 현재 사용 중인가 / stale 인가"를 먼저 보여줘야 한다.
- row grouping 순서는 `in_use -> recoverable -> safe_to_delete -> stale` 로 고정한다.

### 3.3 Workflow reload semantics는 기존 runtime contract를 그대로 따른다

- valid reload는 future launch / future retry에만 반영된다.
- invalid reload는 `last-known-good` 을 유지한다.
- running session은 reload로 바뀌지 않는다.
- `before_run` failure는 typed degraded scenario로 surface되어야 한다.

### 3.4 Secrets는 local-only boundary를 절대 넘지 않는다

- secret value는 Keychain 외의 저장소에 쓰지 않는다.
- `/v1/state`, `/v1/sessions`, FFI snapshot JSON, logs, restore metadata에 secret value가 나오면 안 된다.
- UI는 ref metadata만 보여주고 secret value는 절대 렌더링하지 않는다.

### 3.5 Recovery는 cause-first, next-safe-action-first여야 한다

- error copy 순서는 원인 -> 영향 -> next action 이다.
- 각 degraded scenario는 3 actions 이내에 next safe action으로 이어져야 한다.
- generic shell fallback이 가능한 경로에서는 fallback을 숨기지 않는다.

### 3.6 Rust persistence가 restore source of truth다

- Sprint 5 이후 restore-critical metadata의 source of truth는 Rust SQLite store가 된다.
- 현재 Swift JSON stores(`AppShellPreferencesStore`, `TerminalSessionRestoreStore`)는 migration fallback으로만 남는다.
- restore, stale claim reconcile, inventory recoverable rows는 Rust persistence를 기준으로 동작해야 한다.

---

## 4. 현재 베이스라인

| 영역 | 현재 상태 | Sprint 5에서 확장할 점 |
|---|---|---|
| `hc-control-plane::worktrees` | 실제 git worktree provisioning 구현 완료 | lifecycle, inventory projection, cleanup / stale handling 추가 |
| `hc-workflow` | contract load, validate, reload, bootstrap hook runtime 이미 존재 | invalid reload / before_run failure를 recovery matrix와 UI surface에 연결 |
| `hc-api` / `hc-cli` | state / sessions / workflow / reconcile parity 기본형 존재 | secret redaction / local-only guardrail / snapshot optional fields 검증 강화 |
| Swift Settings / Workflow Drawer | 개요 수준 summary surface 존재 | WF-09 섹션 구조, actual settings forms, workflow watch/reload diagnostics 추가 |
| Swift restore path | file-backed JSON restore bundle 기반 | Rust persistence 기반 restore와 degraded startup 흐름으로 전환 |

---

## 5. Sprint 5에서 추가되는 핵심 데이터 모델

| 모델 | 목적 | 비고 |
|---|---|---|
| `WorktreeLifecycleState` | worktree 운영 상태 표현 | `active`, `review`, `archived`, `gc_candidate`, `pinned` |
| `InventoryDisposition` | inventory의 recovery-first grouping | `in_use`, `recoverable`, `safe_to_delete`, `stale` |
| `InventoryRow` | worktree/cache/restore/degraded 정보를 합친 projection row | WF-13용 derived view |
| `CacheRoot`, `CacheEntry`, `CacheQuota` | disk usage / fingerprint / quota 관리 | repo cache coordinator v1 |
| `RestorePointSummary` | recoverable row의 restore metadata 요약 | session metadata + restore bundle linkage |
| `TerminalSettings` | terminal font / theme / scrollback / cursor 설정 | launch-time 적용, running session 소급 적용 없음 |
| `SecretRef` | Keychain-backed ref metadata | scope는 `global`, `project`, `preset` |
| `WorktreePolicy`, `NotificationRule` | archive / GC / quota / alert 정책 | project/global scope 허용 |
| `ProjectRecord`, `LayoutRecord`, `SessionMetadataRecord`, `AppStateRecord` | Rust-backed restore source of truth | app restore 및 inventory recoverable row에 사용 |
| `DegradedIssue`, `RecoveryAction` | typed degraded recovery contract | UI / automation / tests가 동일 code를 사용 |

---

## 6. Task 상세 Spec

### 6.1 EP-07 — Worktree & Cache Ops

#### MVP2-038: repo cache coordinator v1

**Outcome**

- repo별 worktree/cache root를 스캔해 disk usage, fingerprint, quota 상태를 authoritative metadata로 저장한다.
- inventory projection은 더 이상 Swift-side restore root 추정치로 만들어지지 않는다.

**필수 구현**

- `cache_roots`는 project별 cache root, total size, entry count, last scan time을 가진다.
- `cache_entries`는 worktree / cache artifact 단위 fingerprint, size, last accessed, scan status를 가진다.
- `cache_quotas`는 global / project scope quota와 used bytes를 가진다.
- fingerprint는 최소 `branch + HEAD commit` 기준으로 계산한다.
- scan error는 silent drop이 아니라 row-level degraded reason으로 남겨야 한다.

**설계 규칙**

- cache coordinator는 raw filesystem scan 결과를 저장하지만, `safe_to_delete` 같은 UI용 의미는 inventory projection에서 파생한다.
- scan이 실패해도 snapshot 전체가 깨지면 안 되고, 부분 degraded row로 남겨야 한다.

**자동 검증**

- quota 계산
- fingerprint 계산
- scan error degradation
- project/global quota aggregation

---

#### MVP2-039: worktree lifecycle + cleanup suggestion

**Outcome**

- worktree는 lifecycle을 갖고, inventory row는 lifecycle + live binding + restore binding + filesystem health를 합쳐 disposition을 갖는다.
- cleanup suggestion은 `AttentionEvent` 로 발행된다.

**필수 구현**

- `worktrees` 테이블에 `lifecycle_state`, `last_accessed_at`, `size_bytes`, `pinned` 추가
- lifecycle transition
  - review-ready evidence가 생기면 `review`
  - review 완료 후 `archived`
  - grace period 경과 후 `gc_candidate`
  - operator pin 시 `pinned`
- cleanup suggestion event는 `gc_candidate` 또는 stale cleanup failure row에서 발행

**Disposition 파생 규칙**

- `in_use`
  - running session이 연결된 worktree
  - active claim 또는 launch target으로 쓰이는 worktree
  - pinned worktree가 live task/session과 연결된 경우
- `recoverable`
  - crashed/dead session metadata가 연결된 worktree
  - restore point가 남아 있어 operator가 다시 들어갈 수 있는 row
  - workflow setup failure 후 workspace가 남아 있는 row
- `safe_to_delete`
  - archived / gc_candidate 상태
  - pinned 아님
  - live session / claim / restore dependency 없음
- `stale`
  - path missing
  - repo missing
  - orphaned restore metadata
  - cleanup failure residue
  - task / worktree / claim linkage mismatch

**자동 검증**

- lifecycle transition validity
- pinned override
- stale row derivation
- cleanup suggestion event emission

---

#### MVP2-040: Worktree & Cache Inventory UI (WF-13)

**Outcome**

- WF-13은 recovery-first grouping을 갖는 dedicated inventory surface로 구현된다.
- Swift shell은 Rust inventory projection을 직접 읽고, 더 이상 restore roots를 임의 추정해 inventory처럼 보이지 않는다.

**Entry points**

- `Cmd+Shift+I`
- Command Palette inventory section
- Settings > Worktree & Recovery deep-link

**Surface type**

- global route가 아니라 dedicated overlay / panel surface로 구현한다.
- 열고 닫을 때 origin surface로 복귀해야 한다.

**UI 구성**

- Summary strip
  - total rows
  - total size
  - pinned count
  - safe-to-delete count
  - stale count
  - quota usage
- Filters
  - disposition
  - lifecycle state
  - project
  - pinned
  - size range
  - last used
- Primary grouping sections
  - `In Use`
  - `Recoverable`
  - `Safe to Delete`
  - `Stale`

**Row contract**

각 row는 최소 아래 정보를 가져야 한다.

- owner project
- linked task
- linked session or restore point
- workspace path
- branch
- lifecycle state
- disposition
- size
- last used
- degraded reason or restore badge
- available actions

**Restore point semantics**

- recoverable row는 단순 경로가 아니라 "다시 이어서 들어갈 수 있는 세션/레이아웃/워크스페이스 metadata" 를 뜻한다.
- 따라서 inventory projection은 `worktrees` 뿐 아니라 `session_metadata`, `app_state`, legacy restore bundle fallback을 함께 합쳐 recoverable row를 만든다.

**자동 검증**

- grouping order
- row action gating
- empty state / filtered state rendering data
- recoverable row creation from restore metadata

---

#### MVP2-041: Inventory actions

**Outcome**

- inventory row에서 operator가 바로 pin / clean / open 행동을 수행할 수 있다.

**Action contract**

| Action | 허용 상태 | 결과 |
|---|---|---|
| `pin` | `in_use`, `recoverable`, `safe_to_delete` | `pinned=true`, lifecycle `pinned` |
| `unpin` | pinned row | 이전 lifecycle로 복원 |
| `clean` | `safe_to_delete`, `stale` | git worktree remove, disk cleanup, restore metadata cleanup, DB cleanup |
| `open_session` | live session 또는 recoverable session metadata 존재 | linked target으로 focus / open |
| `open_task` | linked task 존재 | task drawer deep-link |
| `open_finder` | path exists | Finder open |

**설계 규칙**

- `clean` 은 destructive action이므로 live session / claim / restore dependency가 있으면 허용되면 안 된다.
- `stale` row의 `clean` 실패는 다시 stale degraded reason으로 남겨야 한다.
- action 실행 후 inventory projection, attention, snapshot이 같은 vocabulary로 즉시 갱신되어야 한다.

**자동 검증**

- allowed / blocked action matrix
- clean side effects
- stale cleanup retry path

---

### 6.2 EP-08 — Settings, Secrets & Recovery

#### MVP2-042: terminal settings

**Outcome**

- terminal settings는 실제 persisted preference가 되고, new session / restore launch에 적용된다.

**설정 항목**

- `font_family`
- `font_size`
- `theme_name`
- `scrollback_lines`
- `cursor_style`
- `cursor_blink`

**적용 규칙**

- 변경은 persistence에 즉시 반영한다.
- running session에는 소급 적용하지 않고, future launch / restore에서 사용한다.
- invalid value는 typed validation error로 거절한다.

**UI**

- Settings > Terminal section
- 변경 전후 값이 한 화면에서 읽혀야 한다.

**자동 검증**

- defaults
- validation
- persistence round-trip
- new launch 적용

---

#### MVP2-043: Settings diagnostics + Automation & Integrations

**Outcome**

- WF-09 Settings가 실제 runtime / readiness / local API / workflow watch 상태를 설명하는 surface가 된다.

**Settings 정보 구조**

- `Terminal`
- `Diagnostics`
- `Secrets`
- `Worktree & Recovery`
- `Automation & Integrations`

**Diagnostics section**

- shell integration on/off
- shell marker status per shell
- preset install state per preset
- workflow contract detect / valid / invalid
- readiness probe 결과 재사용

**Automation & Integrations section**

- local API transport
- socket path
- local-only boundary 설명
- `hc` CLI install path / version
- workflow watch 상태
- last reload time
- last reload error
- cadence / max slots / retry cap defaults
- tracker binding summary
- actions
  - refresh snapshot
  - reconcile now
  - reload workflow
  - export snapshot JSON

**중요 규칙**

- workflow reload error는 Settings와 Workflow Drawer 양쪽에서 설명 가능해야 한다.
- local API는 port-first가 아니라 Unix domain socket / local-only boundary로 설명해야 한다.

**자동 검증**

- Settings view model payload shape
- workflow reload error dual-surface visibility
- API runtime info serialization

---

#### MVP2-044: Keychain secrets injection

**Outcome**

- 실제 macOS Keychain을 사용해 secret ref를 저장/조회하고, session launch env에 주입한다.

**저장 모델**

- `secret_refs` 테이블에는 metadata만 저장
  - `id`
  - `name`
  - `keychain_service`
  - `keychain_account`
  - `scope`
- secret value는 Keychain에만 저장

**Launch integration**

- generic / preset / isolated launch는 applicable scope의 `SecretRef` 를 resolve해 environment map으로 주입한다.
- launch config는 environment map을 support해야 한다.
- missing Keychain value는 typed degraded issue(`keychain_ref_missing`)로 surface하고 launch를 실패시킨다.

**보안 규칙**

- secret value는 snapshot / session JSON / logs / restore metadata에 기록되면 안 된다.
- UI는 ref metadata만 보여준다.

**자동 검증**

- ref CRUD
- restart 이후 lookup
- env injection
- redaction from state/session/log payloads

---

#### MVP2-045: worktree policy / cache quota / notification rules

**Outcome**

- worktree archive / GC / quota / alert 정책이 persisted setting이 된다.

**정책 항목**

- `auto_archive_after_days`
- `gc_grace_days`
- `max_worktrees_per_project`
- project/global cache quota
- notification rules
  - cleanup suggestion
  - workflow invalid
  - restore degraded
  - keychain missing

**UI**

- Settings > Worktree & Recovery section
- inventory deep-link 제공
- effective policy summary 표시

**자동 검증**

- policy persistence
- coordinator / lifecycle가 policy를 실제 사용함
- notification rule gating

---

#### MVP2-046: full metadata persistence

**Outcome**

- restore-critical metadata의 source of truth가 Rust SQLite store로 이동한다.

**Persistence 대상**

- projects
  - id, name, root_path, last_opened_at
- layouts
  - layout preset, split config, selected inspector context
- session metadata
  - session_id
  - launch program / args / current directory
  - geometry
  - task/worktree linkage
  - started_at / ended_at
  - exit_code
  - last runtime state
- worktree metadata
  - lifecycle, size, last accessed, pinned
- app state
  - last project
  - last layout
  - last route
  - saved_at

**migration / compatibility 규칙**

- Sprint 5 동안 Swift JSON stores는 fallback으로만 유지한다.
- Rust persistence가 값이 있으면 Swift fallback보다 우선한다.

**자동 검증**

- file-backed DB reopen
- save / reopen / restore round-trip
- no secret persistence

---

#### MVP2-047: last project + last layout restore

**Outcome**

- 앱 재시작 시 last project, layout, route, recoverable session metadata를 복원한다.

**동작**

1. 앱 상태 변경 시 Rust app state와 session metadata를 저장한다.
2. 앱 시작 시 Rust restore state를 읽는다.
3. live PTY가 없는 세션은 archived / recoverable metadata로 surface한다.
4. stale running claim은 startup reconcile에서 정리한다.
5. saved route가 invalid하면 `project_focus` 로 fallback한다.

**UI 규칙**

- restore는 "조용한 재실행" 이 아니라 "복원 가능한 상태를 operator에게 설명하는 것" 이어야 한다.
- dead session은 recoverable or archived badge를 갖고 inventory / launcher / settings에서 보인다.

**자동 검증**

- last project / route / layout round-trip
- dead session archived handling
- restart restore from persisted metadata
- stale claim reconcile on startup

---

#### MVP2-048: degraded recovery flows

**Outcome**

- Sprint 5는 recovery matrix를 typed code path로 구현한다.
- original backlog의 4개 scenario만이 아니라, PRD AC-15와 RG-08이 요구하는 workflow / restore / reconcile degraded path까지 포함한다.

**Recovery matrix**

| Scenario code | Detect source | Primary surface | Next safe actions |
|---|---|---|---|
| `preset_missing` | readiness / launch | Welcome, Settings, New Session | `Continue with Generic Shell`, `Open Settings`, `Retry` |
| `missing_project_path` | startup restore / launcher | Welcome, Settings, Inventory | `Re-locate`, `Remove`, `Open Settings` |
| `deleted_repo` | repo health check | Settings, Inventory | `Re-initialize`, `Continue without Git`, `Remove` |
| `keychain_ref_missing` | secret resolve | Settings, launch error | `Re-enter secret`, `Remove ref`, `Open Settings` |
| `invalid_workflow_reload` | workflow watch / reload | Settings, Workflow Drawer, Attention | `Keep last-known-good`, `Open Workflow`, `Retry validate` |
| `before_run_hook_failure` | workflow bootstrap | Task / Attention / Inventory | `Open Workflow`, `Retry launch`, `Launch Generic Shell` |
| `worktree_unreachable` | filesystem probe / clean action | Inventory, Attention | `Clean stale worktree`, `Retry`, `Open Finder` |
| `crashed_restore` | startup restore | Project Focus, Inventory | `Archive metadata`, `Reopen workspace`, `Open Settings` |
| `stale_claim_reconcile` | startup reconcile / tick | Task Drawer, Attention | `Release claim`, `Requeue task`, `Open task` |

**규칙**

- 모든 scenario는 cause-first copy를 사용한다.
- 각 scenario는 3 actions 이내에 next safe action에 도달해야 한다.
- terminal work를 계속할 수 있는 generic fallback이 있으면 숨기지 않는다.

**자동 검증**

- 모든 scenario의 typed detection
- recovery action dispatch
- next-action availability
- no data loss on degraded transition

---

### 6.3 EP-09 — QA, Compatibility & Exit

#### MVP2-049: readiness acceptance suite

**Outcome**

- readiness, launcher, fallback, workflow presence, Keychain availability를 자동 테스트로 검증한다.

**Sprint 5 범위**

- `git`
- `shell`
- login-shell PATH hydration
- preset binaries
- shell integration markers
- workflow presence / validity summary
- Keychain availability / secret ref resolution
- generic shell fallback path

**Sprint 5에서 제외**

- screenshot / runbook / S-01 evidence 작성

**자동 검증**

- missing git
- preset missing
- shell integration disabled
- workflow missing vs valid
- keychain available vs missing ref

---

#### MVP2-050: compatibility regression suite

**Outcome**

- Sprint 5에서는 "manual certification" 이 아니라 "machine-runnable compatibility harness" 까지만 닫는다.

**Sprint 5 범위**

- PTY launch / resize / terminate smoke
- split-safe geometry propagation
- optional installed-tool smoke harness
  - `yazi`
  - `lazygit`
  - `vim` or `nvim`
  - `tmux`
- tool missing 시 skip-based execution 허용

**Sprint 5에서 제외**

- IME 최종 operator certification
- TUI screen recording
- RG-03 evidence pack 작성

**자동 검증**

- generic PTY smoke
- installed tool launch / quit if present
- resize path does not crash

---

#### MVP2-051: degraded-state regression suite

**Outcome**

- degraded recovery, security redaction, snapshot-safe restore path를 자동 테스트로 닫는다.

**테스트 범위**

- `preset_missing`
- `missing_project_path`
- `deleted_repo`
- `keychain_ref_missing`
- `invalid_workflow_reload`
- `before_run_hook_failure`
- `worktree_unreachable`
- `crashed_restore`
- `stale_claim_reconcile`
- cleanup permission error
- secret redaction from state / session payloads

**Sprint 5에서 제외**

- S-07 recovery video / runbook 작성

---

## 7. Cross-Cutting Exit Requirements Included in Sprint 5

Sprint 5 task 번호는 049~051이지만, 아래 code-close 요구사항은 task 내부에 같이 포함해야 한다.

| 상위 기준 | Sprint 5 code-close deliverable | Sprint 5에서 defer되는 것 |
|---|---|---|
| RG-06 | snapshot/session parity regression tests, optional field compatibility, same vocabulary across UI/FFI/API/CLI | parity video / hosted operator proof |
| RG-07 | ops diagnostics data path(last tick, last reconcile, slots, retry count, workflow health, tracker health)와 workflow invalid attention wiring | ops-strip video / notes |
| RG-09 | secret redaction tests, UDS 0600 permission test, local-only transport summary | security checklist 문서화 |
| RG-10 | automated measurement hooks for snapshot / workflow validate / restore latency, no blocker/major regression tracking hooks | final threshold sign-off, ship-exit note |

---

## 8. 구현 순서

Sprint 5는 아래 순서로 구현해야 한다.

1. domain / storage vocabulary 확장
2. terminal launch environment support 추가
3. inventory projection / lifecycle / cleanup action 구현
4. settings / persistence / recovery service 구현
5. FFI / API / redaction / runtime info integration
6. Swift Settings / Workflow / Inventory surface 연결
7. Rust-backed restore 경로로 Swift shell 이관
8. automated regression / parity / security / metric hook 추가

이 순서를 바꾸면 UI가 임시 local model에 다시 의존하거나, recovery matrix가 snapshot semantics와 어긋날 가능성이 높다.

---

## 9. Sprint 5 종료 시점의 명시적 defer 목록

아래 항목은 본 sprint에서 일부러 완료로 주장하지 않는다.

- manual TUI compatibility operator sign-off
- IME final validation
- screenshots / videos / runbook / checklist / manifest 작성
- RG-01 ~ RG-10 최종 pass 선언
- ship/no-ship 최종 판정

Sprint 5 완료의 의미는 "수동 gate promotion을 실행할 코드와 자동 검증 경계가 준비됨" 이지, "release evidence가 모두 채워짐" 이 아니다.

---

## Sprint 5 Execution Complete (2026-03-25)

코드 경계와 자동 검증이 준비됨. `cargo test --workspace` 0 failures, `swift test` 133/133 통과.

**Post-Sprint 5 defer (변경 없음):**
- manual TUI operator sign-off (yazi, lazygit, vim/nvim, tmux, IME)
- screenshots / videos / runbook / checklist / evidence manifest 작성
- RG-01 ~ RG-10 최종 pass 선언
- Final ship/no-ship 판정
