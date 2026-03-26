#ifndef HC_FFI_H
#define HC_FFI_H

#include <stddef.h>

#ifdef __cplusplus
extern "C" {
#endif

typedef struct HcString {
    char *ptr;
} HcString;

typedef struct HcBytes {
    unsigned char *ptr;
    size_t len;
} HcBytes;

HcString hc_runtime_info_json(void);
HcString hc_api_server_start_json(const char *socket_path);
HcString hc_reconcile_now_json(void);
HcString hc_state_snapshot_json(void);
HcString hc_sessions_list_json(void);
HcString hc_session_details_json(const char *session_id);
HcString hc_session_attach_task_json(const char *session_id, const char *task_id);
HcString hc_session_detach_task_json(const char *session_id);
HcString hc_review_decision_json(const char *task_id, const char *decision);
HcString hc_review_queue_json(void);
HcString hc_attention_resolve_json(const char *attention_id);
HcString hc_attention_dismiss_json(const char *attention_id);
HcString hc_attention_snooze_json(const char *attention_id);
HcString hc_task_board_json(const char *project_id);
HcString hc_task_move_json(const char *task_id, const char *column);
HcString hc_task_prepare_isolated_launch_json(const char *project_root, const char *project_name, const char *task_id, const char *task_title, const char *workspace_root);
HcString hc_task_provision_workspace_json(const char *project_root, const char *task_id, const char *base_root);
HcString hc_dispatch_send_json(const char *target_session_id, const char *task_id, _Bool target_live, const char *payload);
HcString hc_terminal_session_spawn_json(const char *config_json);
HcBytes hc_terminal_session_drain(const char *session_id);
int hc_terminal_session_write(const char *session_id, const unsigned char *ptr, size_t len);
int hc_terminal_session_resize(const char *session_id, unsigned short cols, unsigned short rows);
int hc_terminal_session_terminate(const char *session_id);
HcString hc_terminal_session_snapshot_json(const char *session_id);
int hc_session_focus(const char *session_id);
int hc_session_takeover(const char *session_id);
int hc_session_release_takeover(const char *session_id);
HcString hc_workflow_validate_json(const char *project_root);
HcString hc_workflow_reload_json(const char *project_root);
HcString hc_inventory_summary_json(const char *project_id);
HcString hc_inventory_list_json(const char *project_id);
HcString hc_set_worktree_pinned_json(const char *worktree_id, int is_pinned);
HcString hc_update_worktree_lifecycle_json(const char *worktree_id, const char *new_state);
HcString hc_terminal_settings_json(void);
HcString hc_upsert_terminal_settings_json(const char *json);
HcString hc_list_secret_refs_json(void);
HcString hc_upsert_secret_ref_json(const char *json);
HcString hc_delete_secret_ref_json(const char *ref_id);
HcString hc_resolve_secret_env_json(void);
HcString hc_degraded_issues_json(const char *context_json);
HcString hc_recovery_action_for_issue_json(const char *issue_code);
HcString hc_runtime_info_summary_json(void);
HcString hc_load_app_state_json(void);
HcString hc_save_app_state_json(const char *json);
HcString hc_list_recoverable_sessions_json(const char *project_id);
void hc_string_free(HcString value);
void hc_bytes_free(HcBytes value);

#ifdef __cplusplus
}
#endif

#endif /* HC_FFI_H */
