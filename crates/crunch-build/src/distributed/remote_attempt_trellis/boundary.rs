use super::model::TRELLIS_EVENT_LIMIT;
use crate::distributed::MAX_REMOTE_ATTEMPT_EVENTS;

pub const TRELLIS_REMOTE_ADMISSION_CLAIM: &str =
    "the named abstract fenced-attempt safety properties agree for the recorded supported projection cases";
pub const TRELLIS_REMOTE_ADMISSION_NON_CLAIMS: &[&str] = &[
    "not full implementation equivalence",
    "not persistence atomicity",
    "not transport reliability",
    "not cryptographic correctness",
    "not remote worker correctness",
    "not liveness or availability",
    "not whole-build correctness",
    "not release eligibility",
];

const _: () = assert!(MAX_REMOTE_ATTEMPT_EVENTS < TRELLIS_EVENT_LIMIT);

#[must_use]
pub fn trellis_claim_text_is_bounded(value: &str) -> bool {
    let lower = value.to_ascii_lowercase();
    let forbidden = [
        "full implementation equivalence",
        "persistence atomicity",
        "transport reliability",
        "cryptographic correctness",
        "remote worker correctness",
        "worker correctness",
        "liveness",
        "availability",
        "whole-build correctness",
        "release eligibility",
    ];
    !forbidden.iter().any(|fragment| lower.contains(fragment))
}
