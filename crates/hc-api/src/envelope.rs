use hc_domain::AppSnapshot;
use serde::Serialize;
use serde_json::Value;

#[derive(Clone, Debug, Serialize)]
pub struct ApiMeta {
    pub api_version: &'static str,
    pub snapshot_rev: u64,
    pub runtime_rev: u64,
    pub projection_rev: u64,
    pub snapshot_at: Option<String>,
    pub request_id: String,
}

#[derive(Clone, Debug, Serialize)]
pub struct SuccessEnvelope<T: Serialize> {
    pub ok: bool,
    pub data: T,
    pub meta: ApiMeta,
}

#[derive(Clone, Debug, Serialize)]
pub struct ErrorBody {
    pub code: String,
    pub message: String,
    pub retryable: bool,
    pub details: Value,
}

#[derive(Clone, Debug, Serialize)]
pub struct ErrorEnvelope {
    pub ok: bool,
    pub error: ErrorBody,
    pub meta: ApiMeta,
}

pub fn meta_from_snapshot(snapshot: &AppSnapshot, request_id: impl Into<String>) -> ApiMeta {
    ApiMeta {
        api_version: "1",
        snapshot_rev: snapshot.meta.snapshot_rev,
        runtime_rev: snapshot.meta.runtime_rev,
        projection_rev: snapshot.meta.projection_rev,
        snapshot_at: snapshot.meta.snapshot_at.clone(),
        request_id: request_id.into(),
    }
}

pub fn success_json<T: Serialize>(
    data: T,
    snapshot: &AppSnapshot,
    request_id: impl Into<String>,
) -> Result<String, String> {
    serde_json::to_string(&SuccessEnvelope {
        ok: true,
        data,
        meta: meta_from_snapshot(snapshot, request_id),
    })
    .map_err(|error| error.to_string())
}

pub fn error_json(
    code: &str,
    message: &str,
    snapshot: Option<&AppSnapshot>,
    request_id: impl Into<String>,
    retryable: bool,
    details: Value,
) -> Result<String, String> {
    let request_id = request_id.into();
    let meta = snapshot
        .map(|snapshot| meta_from_snapshot(snapshot, request_id.clone()))
        .unwrap_or(ApiMeta {
            api_version: "1",
            snapshot_rev: 0,
            runtime_rev: 0,
            projection_rev: 0,
            snapshot_at: None,
            request_id,
        });
    serde_json::to_string(&ErrorEnvelope {
        ok: false,
        error: ErrorBody {
            code: code.to_string(),
            message: message.to_string(),
            retryable,
            details,
        },
        meta,
    })
    .map_err(|error| error.to_string())
}
