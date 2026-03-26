use std::ffi::CString;
use std::os::raw::c_char;

use hc_api::{
    task_move_json as api_task_move_json,
    task_prepare_isolated_launch_json as api_task_prepare_isolated_launch_json,
    task_provision_workspace_json as api_task_provision_workspace_json, tasks_list_json,
};

use crate::HcString;

fn read_required_c_string(value: *const c_char) -> Result<String, String> {
    if value.is_null() {
        return Err("null pointer".to_string());
    }

    let text = unsafe { std::ffi::CStr::from_ptr(value) }
        .to_str()
        .map_err(|error| error.to_string())?;

    Ok(text.to_string())
}

fn read_optional_c_string(value: *const c_char) -> Result<Option<String>, String> {
    if value.is_null() {
        return Ok(None);
    }

    let text = unsafe { std::ffi::CStr::from_ptr(value) }
        .to_str()
        .map_err(|error| error.to_string())?
        .trim()
        .to_string();

    if text.is_empty() {
        Ok(None)
    } else {
        Ok(Some(text))
    }
}

fn string_to_hcstring(value: Result<String, String>) -> HcString {
    let payload = match value {
        Ok(value) => value,
        Err(error) => serde_json::json!({ "error": error }).to_string(),
    };
    let string = CString::new(payload).expect("json payload is nul-free");

    HcString {
        ptr: string.into_raw(),
    }
}

pub fn task_board_json(project_id: Option<String>) -> Result<String, String> {
    tasks_list_json(project_id.as_deref())
}

pub fn task_move_json(task_id: &str, column: &str) -> Result<String, String> {
    api_task_move_json(task_id, column)
}

pub fn task_provision_workspace_json(
    project_root: &str,
    task_id: &str,
    base_root: Option<String>,
) -> Result<String, String> {
    api_task_provision_workspace_json(project_root, task_id, base_root.as_deref())
}

pub fn task_prepare_isolated_launch_json(
    project_root: &str,
    project_name: &str,
    task_id: &str,
    task_title: &str,
    workspace_root: &str,
) -> Result<String, String> {
    api_task_prepare_isolated_launch_json(
        project_root,
        project_name,
        task_id,
        task_title,
        workspace_root,
    )
}

#[unsafe(no_mangle)]
pub extern "C" fn hc_task_board_json(project_id: *const c_char) -> HcString {
    let payload = read_optional_c_string(project_id).and_then(task_board_json);
    string_to_hcstring(payload)
}

#[unsafe(no_mangle)]
pub extern "C" fn hc_task_move_json(task_id: *const c_char, column: *const c_char) -> HcString {
    let payload = read_required_c_string(task_id).and_then(|task_id| {
        read_required_c_string(column).and_then(|column| task_move_json(&task_id, &column))
    });
    string_to_hcstring(payload)
}

#[unsafe(no_mangle)]
pub extern "C" fn hc_task_provision_workspace_json(
    project_root: *const c_char,
    task_id: *const c_char,
    base_root: *const c_char,
) -> HcString {
    let payload = read_required_c_string(project_root).and_then(|project_root| {
        read_required_c_string(task_id).and_then(|task_id| {
            read_optional_c_string(base_root).and_then(|base_root| {
                task_provision_workspace_json(&project_root, &task_id, base_root)
            })
        })
    });
    string_to_hcstring(payload)
}

#[unsafe(no_mangle)]
pub extern "C" fn hc_task_prepare_isolated_launch_json(
    project_root: *const c_char,
    project_name: *const c_char,
    task_id: *const c_char,
    task_title: *const c_char,
    workspace_root: *const c_char,
) -> HcString {
    let payload = read_required_c_string(project_root).and_then(|project_root| {
        read_required_c_string(project_name).and_then(|project_name| {
            read_required_c_string(task_id).and_then(|task_id| {
                read_required_c_string(task_title).and_then(|task_title| {
                    read_required_c_string(workspace_root).and_then(|workspace_root| {
                        task_prepare_isolated_launch_json(
                            &project_root,
                            &project_name,
                            &task_id,
                            &task_title,
                            &workspace_root,
                        )
                    })
                })
            })
        })
    });
    string_to_hcstring(payload)
}
