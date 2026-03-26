# Architecture Alignment Doc Deltas

작성일: 2026-03-26

## 목적
현재 branch에서 코드로 반영된 내용 중, 이후 ignored `docs/architecture` 본문을 갱신할 때 반영해야 할 delta를 추린다.

## 문서별 delta

## `Haneulchi_Workflow_Contract_Spec_v2.md`
- `haneulchi` top-level normalization이 실제 구현에 반영됐다.
- hook path는 repo-confinement validation을 거친다.

## `Haneulchi_WORKFLOW_md_Runtime_Semantics_v1.md`
- setup-only prelaunch bootstrap helper가 추가됐다.
- documented scalar template rendering과 `HC_*` env naming이 구현 기준이 됐다.
- hook capture limit은 더 큰 실사용 값으로 정렬됐다.

## `Haneulchi_DB_Schema_Spec_v2.md`
- `v_control_tower_ops_strip`
- `v_task_drawer_automation`
- `v_automation_health`
- `idx_tasks_automation_mode`
- `idx_retry_queue_due_state`
- timestamp storage는 TEXT ISO-8601 중심 설명으로 갱신 필요
- tracker external ref는 `tracker_bindings` 중심 설명으로 정리 필요

## `Haneulchi_CLI_API_Spec_v2.md`
- `/v1/automation` 추가
- success/error envelope에 `request_id`, `retryable`, `details` 추가
- response headers에 `X-HC-Api-Version`, `X-HC-Snapshot-Rev`, `X-HC-Request-Id` 추가
- `/v1/state` compact/filter semantics 갱신
- `/v1/tasks`는 board projection contract로 설명 갱신

## `Haneulchi_State_Snapshot_and_Session_Control_API_v1.md`
- richer `session.details`
- filtered `/v1/state`
- request-id carrying envelope
- header/meta parity

## `Haneulchi_FFI_Interface_Spec_v3.md`
- session-details export 추가
- prelaunch isolated bootstrap helper 추가
- Swift shell이 prelaunch workflow setup을 재구현하지 않고 FFI helper를 소비하는 방향으로 갱신 필요

## `Haneulchi_LoFi_Wireframe_Spec_v5.md`
- command palette operator commands가 반영됐다.
- workflow/settings/task context surfaces는 richer watch/binding/lineage 정보를 보여주도록 정리할 수 있다.

## 비고
- 이 branch에서는 ignored `docs/architecture` 파일을 직접 추적하지 않는다.
- architecture 본문 갱신은 tracking-policy를 유지한 채 별도 문서 정리 또는 사용자 명시 요청 이후에 수행한다.
