#include "hc_ffi.h"

#include <stdlib.h>
#include <string.h>

static HcString hc_stub_string(const char *payload) {
    HcString value;
    value.ptr = strdup(payload);
    return value;
}

static HcBytes hc_stub_bytes(void) {
    HcBytes value;
    value.ptr = NULL;
    value.len = 0;
    return value;
}

#define HC_WEAK_STRING0(name, payload) __attribute__((weak)) HcString name(void) { return hc_stub_string(payload); }
#define HC_WEAK_STRING1(name, payload, t1, a1) __attribute__((weak)) HcString name(t1 a1) { (void)a1; return hc_stub_string(payload); }
#define HC_WEAK_STRING2(name, payload, t1, a1, t2, a2) __attribute__((weak)) HcString name(t1 a1, t2 a2) { (void)a1; (void)a2; return hc_stub_string(payload); }
#define HC_WEAK_STRING3(name, payload, t1, a1, t2, a2, t3, a3) __attribute__((weak)) HcString name(t1 a1, t2 a2, t3 a3) { (void)a1; (void)a2; (void)a3; return hc_stub_string(payload); }
#define HC_WEAK_STRING4(name, payload, t1, a1, t2, a2, t3, a3, t4, a4) __attribute__((weak)) HcString name(t1 a1, t2 a2, t3 a3, t4 a4) { (void)a1; (void)a2; (void)a3; (void)a4; return hc_stub_string(payload); }
#define HC_WEAK_STRING5(name, payload, t1, a1, t2, a2, t3, a3, t4, a4, t5, a5) __attribute__((weak)) HcString name(t1 a1, t2 a2, t3 a3, t4 a4, t5 a5) { (void)a1; (void)a2; (void)a3; (void)a4; (void)a5; return hc_stub_string(payload); }
#define HC_WEAK_INT1(name, value, t1, a1) __attribute__((weak)) int name(t1 a1) { (void)a1; return value; }
#define HC_WEAK_INT3(name, value, t1, a1, t2, a2, t3, a3) __attribute__((weak)) int name(t1 a1, t2 a2, t3 a3) { (void)a1; (void)a2; (void)a3; return value; }
#define HC_WEAK_BYTES1(name, t1, a1) __attribute__((weak)) HcBytes name(t1 a1) { (void)a1; return hc_stub_bytes(); }

HC_WEAK_STRING0(hc_runtime_info_json, "{\"renderer_id\":\"preview\",\"transport\":\"preview\",\"demo_mode\":true}")
HC_WEAK_STRING1(hc_api_server_start_json, "{\"status\":\"preview\"}", const char *, socket_path)
HC_WEAK_STRING0(hc_reconcile_now_json, "{\"status\":\"preview\"}")
HC_WEAK_STRING0(hc_state_snapshot_json, "{\"meta\":{\"snapshot_rev\":1,\"runtime_rev\":1,\"projection_rev\":1,\"snapshot_at\":\"2026-03-26T00:00:00Z\"},\"ops\":{\"running_slots\":0,\"max_slots\":1,\"retry_queue_count\":0,\"workflow_health\":\"none\"},\"app\":{\"active_route\":\"project_focus\",\"focused_session_id\":null,\"degraded_flags\":[]},\"projects\":[],\"sessions\":[],\"attention\":[],\"retry_queue\":[],\"warnings\":[]}")
HC_WEAK_STRING0(hc_sessions_list_json, "[]")
HC_WEAK_STRING1(hc_session_details_json, "{\"session_id\":\"preview\",\"title\":\"Preview Session\",\"workflow_binding\":{\"state\":\"none\",\"path\":\"\",\"last_good_hash\":null,\"last_reload_at\":null,\"last_error\":null},\"recent_events\":[]}", const char *, session_id)
HC_WEAK_STRING2(hc_session_attach_task_json, "{\"status\":\"preview\"}", const char *, session_id, const char *, task_id)
HC_WEAK_STRING1(hc_session_detach_task_json, "{\"status\":\"preview\"}", const char *, session_id)
HC_WEAK_STRING2(hc_review_decision_json, "{\"status\":\"preview\"}", const char *, task_id, const char *, decision)
HC_WEAK_STRING0(hc_review_queue_json, "{\"items\":[],\"degraded_reason\":\"preview_stub\"}")
HC_WEAK_STRING1(hc_attention_resolve_json, "{\"status\":\"preview\"}", const char *, attention_id)
HC_WEAK_STRING1(hc_attention_dismiss_json, "{\"status\":\"preview\"}", const char *, attention_id)
HC_WEAK_STRING1(hc_attention_snooze_json, "{\"status\":\"preview\"}", const char *, attention_id)
HC_WEAK_STRING1(hc_task_board_json, "{\"selected_project_id\":null,\"projects\":[],\"columns\":[]}", const char *, project_id)
HC_WEAK_STRING2(hc_task_move_json, "{\"selected_project_id\":null,\"projects\":[],\"columns\":[]}", const char *, task_id, const char *, column)
HC_WEAK_STRING5(hc_task_prepare_isolated_launch_json, "{\"workspace_root\":\"\",\"base_root\":\".\",\"session_cwd\":\"\",\"rendered_prompt_path\":\"\",\"phase_sequence\":[],\"hook_phase_results\":[],\"outcome_code\":\"preview\",\"warning_codes\":[],\"claim_released\":false,\"launch_exit_code\":0,\"last_known_good_hash\":null}", const char *, project_root, const char *, project_name, const char *, task_id, const char *, task_title, const char *, workspace_root)
HC_WEAK_STRING3(hc_task_provision_workspace_json, "{\"task_id\":\"preview\",\"worktree_id\":\"preview\",\"workspace_root\":\"/tmp/preview\",\"base_root\":\".\",\"branch_name\":\"preview\"}", const char *, project_root, const char *, task_id, const char *, base_root)
HC_WEAK_STRING4(hc_dispatch_send_json, "{\"status\":\"preview\"}", const char *, target_session_id, const char *, task_id, _Bool, target_live, const char *, payload)
HC_WEAK_STRING1(hc_terminal_session_spawn_json, "{\"session_id\":\"preview\",\"launch\":{\"program\":\"/bin/zsh\",\"args\":[\"-l\"],\"current_directory\":\"/tmp\"},\"geometry\":{\"cols\":80,\"rows\":24},\"running\":true,\"exit_code\":null,\"shell_metadata\":null}", const char *, config_json)
HC_WEAK_BYTES1(hc_terminal_session_drain, const char *, session_id)
HC_WEAK_INT3(hc_terminal_session_write, 0, const char *, session_id, const unsigned char *, ptr, size_t, len)
HC_WEAK_INT3(hc_terminal_session_resize, 0, const char *, session_id, unsigned short, cols, unsigned short, rows)
HC_WEAK_INT1(hc_terminal_session_terminate, 0, const char *, session_id)
HC_WEAK_STRING1(hc_terminal_session_snapshot_json, "{\"session_id\":\"preview\",\"launch\":{\"program\":\"/bin/zsh\",\"args\":[\"-l\"],\"current_directory\":\"/tmp\"},\"geometry\":{\"cols\":80,\"rows\":24},\"running\":true,\"exit_code\":null,\"shell_metadata\":null}", const char *, session_id)
HC_WEAK_INT1(hc_session_focus, 0, const char *, session_id)
HC_WEAK_INT1(hc_session_takeover, 0, const char *, session_id)
HC_WEAK_INT1(hc_session_release_takeover, 0, const char *, session_id)
HC_WEAK_STRING1(hc_workflow_validate_json, "{\"status\":\"preview\"}", const char *, project_root)
HC_WEAK_STRING1(hc_workflow_reload_json, "{\"status\":\"preview\"}", const char *, project_root)
HC_WEAK_STRING1(hc_inventory_summary_json, "{\"total\":0,\"in_use\":0,\"recoverable\":0,\"safe_to_delete\":0,\"stale\":0}", const char *, project_id)
HC_WEAK_STRING1(hc_inventory_list_json, "[]", const char *, project_id)
HC_WEAK_STRING2(hc_set_worktree_pinned_json, "{\"status\":\"preview\"}", const char *, worktree_id, int, is_pinned)
HC_WEAK_STRING2(hc_update_worktree_lifecycle_json, "{\"status\":\"preview\"}", const char *, worktree_id, const char *, new_state)
HC_WEAK_STRING0(hc_terminal_settings_json, "{\"shell\":\"/bin/zsh\",\"default_cols\":80,\"default_rows\":24,\"scrollback_lines\":10000,\"font_name\":\"\",\"theme\":\"\",\"cursor_style\":\"block\"}")
HC_WEAK_STRING1(hc_upsert_terminal_settings_json, "{\"status\":\"preview\"}", const char *, json)
HC_WEAK_STRING0(hc_list_secret_refs_json, "[]")
HC_WEAK_STRING1(hc_upsert_secret_ref_json, "{\"status\":\"preview\"}", const char *, json)
HC_WEAK_STRING1(hc_delete_secret_ref_json, "{\"status\":\"preview\"}", const char *, ref_id)
HC_WEAK_STRING0(hc_resolve_secret_env_json, "{}")
HC_WEAK_STRING1(hc_degraded_issues_json, "[]", const char *, context_json)
HC_WEAK_STRING1(hc_recovery_action_for_issue_json, "{\"status\":\"preview\"}", const char *, issue_code)
HC_WEAK_STRING0(hc_runtime_info_summary_json, "{\"socket_path\":null,\"transport\":\"preview\",\"status\":\"available\"}")
HC_WEAK_STRING0(hc_load_app_state_json, "null")
HC_WEAK_STRING1(hc_save_app_state_json, "{\"status\":\"preview\"}", const char *, json)
HC_WEAK_STRING1(hc_list_recoverable_sessions_json, "[]", const char *, project_id)

__attribute__((weak)) void hc_string_free(HcString value) {
    free(value.ptr);
}

__attribute__((weak)) void hc_bytes_free(HcBytes value) {
    free(value.ptr);
}
