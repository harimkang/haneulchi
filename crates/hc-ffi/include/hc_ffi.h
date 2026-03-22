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
HcString hc_state_snapshot_json(void);
HcString hc_sessions_list_json(void);
HcString hc_task_board_json(const char *project_id);
HcString hc_task_move_json(const char *task_id, const char *column);
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
void hc_string_free(HcString value);
void hc_bytes_free(HcBytes value);

#ifdef __cplusplus
}
#endif

#endif /* HC_FFI_H */
