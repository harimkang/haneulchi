# Haneulchi Architecture vs Codebase Alignment Review

작성일: 2026-03-26
업데이트: Task 1-8A implementation 반영

## 현재 상태
- `hc-workflow`는 contract parsing, repo-confinement, `haneulchi` front matter normalization, documented scalar template rendering, hook timeout/optional semantics, `HC_*` env를 반영한다.
- orchestrator/runtime snapshot은 sample 값보다 persisted/runtime-derived 값에 가깝게 정렬됐다.
- API/CLI는 `request_id`, typed error body, `/v1/automation`, filtered state export, response headers를 포함하는 richer contract를 가진다.
- FFI와 Swift shell은 expanded state/session-details contract를 공유하고, isolated launch preflight bootstrap은 Rust core가 담당한다.
- command palette는 operator workflow commands를 포함한다.
- workflow/settings/task operator surfaces는 이전보다 richer workflow binding, lineage, watch context를 노출한다.

## 해결된 주요 gap

### 문서가 더 맞았고 코드가 따라온 항목
- Swift local bootstrap ownership 제거
  현재는 isolated launch setup phase를 Rust prelaunch bootstrap helper가 수행하고, Swift는 returned `session_cwd`를 소비한다.
- workflow runtime semantics closure
  repo-confined hook validation, last-known-good reload, larger stdio capture, optional hook semantics, documented scalar rendering이 정리됐다.
- authoritative orchestrator snapshot
  control-plane snapshot은 explicit orchestrator runtime values와 retry aggregates를 사용한다.
- API/CLI contract
  `/v1/automation`, richer error envelope, `X-HC-*` headers, compact/filter behavior가 추가됐다.
- FFI parity surface
  session details export와 Swift bridge decoding이 추가됐다.
- WF-10 palette operator commands
  refresh/reconcile/reload/copy-state/open-automation-panel이 추가됐다.

### 코드가 더 나은 방향이라 문서 업데이트가 더 적절한 항목
- tracker external ref는 `tasks.external_ref_json`가 아니라 `tracker_bindings` 단일 source를 유지하는 편이 더 적절하다.
- timestamp는 DB/API/FFI 전체에서 TEXT ISO-8601 중심으로 정렬하는 편이 일관적이다.
- `/v1/tasks`는 flat list보다 board projection 설명으로 문서화하는 편이 실제 제품과 맞다.

## 아직 남아 있는 gap

### 문서가 더 맞고 아직 미완인 항목
- FFI event subscription family는 여전히 없다. 현재는 query/command + refresh model 중심이다.
- API `details` payload는 richer해졌지만 아직 일부 endpoint는 placeholder 성격의 값이 남아 있다.
- `Task Drawer`/`Workflow Drawer`는 wireframe보다 더 나아졌지만 validate/open-file 같은 직접 action은 아직 제한적이다.

### 문서 업데이트가 필요한 항목
- DB schema spec는 projection view/index 추가와 실제 timestamp/storage 전략을 기준으로 정리할 필요가 있다.
- CLI/API spec는 expanded envelope/header/query surface와 `/v1/tasks` projection model을 기준으로 갱신할 필요가 있다.
- FFI spec는 새 prelaunch bootstrap helper와 session-details export를 반영해야 한다.

## 구현 커밋 기준
- `acec6af` test: lock missing workflow contract semantics
- `d54f112` fix: close workflow runtime contract gaps
- `a70cfcb` fix: make orchestrator snapshot state authoritative
- `f7f3e02` fix: align control api and cli contracts
- `56f74f1` feat: expand ffi parity surface for swift shell
- `c51f44b` fix: move isolated launch bootstrap into rust core
- `1ee7199` feat: add missing operator commands to palette
- `ff4ab59` feat: complete workflow and automation operator surfaces

## 후속 문서 작업
- ignored `docs/architecture` 본문은 아직 직접 수정하지 않았다.
- 본 branch에서는 tracked `docs/superpowers/specs/2026-03-26-architecture-alignment-doc-deltas.md`에 필요한 delta만 정리한다.
