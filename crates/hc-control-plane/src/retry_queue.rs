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

/// Compute exponential backoff delay for a given attempt number.
///
/// Uses a 30 s base with power-of-two multiplier capped at ~8 minutes.
#[allow(dead_code)]
fn backoff_delay_ms(attempt: u32) -> u64 {
    let base_ms: u64 = 30_000;
    let multiplier = 2_u64.saturating_pow(attempt.saturating_sub(1));
    base_ms.saturating_mul(multiplier).min(480_000)
}

/// Compute the ISO 8601 due-at timestamp for the next retry attempt given the
/// current epoch-millis and attempt number.
#[allow(dead_code)]
fn compute_due_at(now_epoch_ms: u64, attempt: u32) -> String {
    let delay = backoff_delay_ms(attempt);
    let due_ms = now_epoch_ms.saturating_add(delay);
    // Store as epoch millis string (consistent with hc-storage schedule_retry_entry).
    due_ms.to_string()
}

/// Determine whether a retry entry should transition from `BackingOff` to
/// `Due` based on the current time, or to `Exhausted` if the stall timeout
/// has been reached.
#[allow(dead_code)]
fn should_promote_retry(
    due_at_ms_str: Option<&str>,
    now_ms: u64,
    max_attempts: u32,
    current_attempt: u32,
) -> RetryPromotion {
    if current_attempt >= max_attempts {
        return RetryPromotion::Exhausted;
    }
    match due_at_ms_str.and_then(|v| v.parse::<u64>().ok()) {
        Some(due_at) if now_ms >= due_at => RetryPromotion::Due,
        _ => RetryPromotion::StillBackingOff,
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[allow(dead_code)]
enum RetryPromotion {
    Due,
    StillBackingOff,
    Exhausted,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn backoff_delay_doubles_per_attempt() {
        assert_eq!(backoff_delay_ms(1), 30_000);
        assert_eq!(backoff_delay_ms(2), 60_000);
        assert_eq!(backoff_delay_ms(3), 120_000);
        assert_eq!(backoff_delay_ms(4), 240_000);
        assert_eq!(backoff_delay_ms(5), 480_000); // cap
        assert_eq!(backoff_delay_ms(6), 480_000); // cap
    }

    #[test]
    fn classify_retryable_and_non_retryable() {
        assert_eq!(
            classify_dispatch_failure("adapter_timeout"),
            DispatchFailureClass::Retryable
        );
        assert_eq!(
            classify_dispatch_failure("user_cancelled"),
            DispatchFailureClass::NonRetryable
        );
    }

    #[test]
    fn retry_promotion_respects_due_time_and_max_attempts() {
        assert_eq!(
            should_promote_retry(Some("1000"), 2000, 5, 1),
            RetryPromotion::Due
        );
        assert_eq!(
            should_promote_retry(Some("3000"), 2000, 5, 1),
            RetryPromotion::StillBackingOff
        );
        assert_eq!(
            should_promote_retry(Some("1000"), 2000, 3, 3),
            RetryPromotion::Exhausted
        );
    }
}
