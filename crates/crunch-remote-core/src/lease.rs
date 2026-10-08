use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;

pub const MAX_SESSION_LEASE_REFS: usize = 4_096;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SessionLeasePlan {
    pub session_id: String,
    pub leased_refs: Vec<String>,
    pub release_when_done: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LeaseRefCountExceeded;

impl LeaseRefCountExceeded {
    pub fn diagnostic(self) -> String {
        format!("session-lease-ref-count-exceeds-{MAX_SESSION_LEASE_REFS}")
    }
}

/// Plan a lease over uploaded and produced references, without reserving,
/// persisting, observing, or releasing a lease. Both source lists count against
/// the bound *before* deduplication, matching remote session admission.
// r[impl remote_builds.hexagonal_core]
pub fn plan_session_lease(
    session_id: &str,
    uploaded_refs: &[String],
    output_refs: &[String],
) -> Result<SessionLeasePlan, LeaseRefCountExceeded> {
    let count = uploaded_refs.len().saturating_add(output_refs.len());
    if count > MAX_SESSION_LEASE_REFS {
        return Err(LeaseRefCountExceeded);
    }
    let mut leased_refs = Vec::with_capacity(count);
    leased_refs.extend_from_slice(uploaded_refs);
    leased_refs.extend_from_slice(output_refs);
    leased_refs.sort_unstable();
    leased_refs.dedup();
    debug_assert!(leased_refs.len() <= count);
    debug_assert!(leased_refs.len() <= MAX_SESSION_LEASE_REFS);
    Ok(SessionLeasePlan {
        session_id: String::from(session_id),
        leased_refs,
        release_when_done: true,
    })
}

#[cfg(test)]
mod tests {
    use alloc::vec;

    use super::*;

    #[test]
    fn overlapping_refs_are_sorted_and_lease_release_is_only_a_plan() {
        let plan = plan_session_lease("session-a", &[String::from("z"), String::from("a")], &[
            String::from("a"),
            String::from("b"),
        ])
        .unwrap();
        assert_eq!(plan.session_id, "session-a");
        assert_eq!(plan.leased_refs, vec!["a", "b", "z"]);
        assert!(plan.release_when_done);
    }

    #[test]
    fn pre_dedup_count_is_bounded() {
        let refs = vec![String::from("same"); MAX_SESSION_LEASE_REFS];
        let accepted = plan_session_lease("session", &refs, &[]).unwrap();
        assert_eq!(accepted.leased_refs, vec!["same"]);
        let error = plan_session_lease("session", &refs, &[String::from("same")]).unwrap_err();
        assert_eq!(error.diagnostic(), "session-lease-ref-count-exceeds-4096");
    }
}
