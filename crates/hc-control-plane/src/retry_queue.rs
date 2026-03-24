use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum DispatchFailureClass {
    Retryable,
    NonRetryable,
}

pub fn classify_dispatch_failure(reason_code: &str) -> DispatchFailureClass {
    match reason_code {
        "adapter_timeout" | "transport_error" | "provider_unavailable" => {
            DispatchFailureClass::Retryable
        }
        _ => DispatchFailureClass::NonRetryable,
    }
}
