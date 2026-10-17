//! Pure, bounded retention interest identity and owner-scoped release decisions.
use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec::Vec;

pub const MAX_INTEREST_RECORDS: usize = 65_536;
pub const MAX_INTEREST_BYTES: usize = 4_096;
pub const INTEREST_SCHEMA_VERSION: u32 = 1;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InterestFact {
    pub identity: String,
    pub owner: String,
    pub path: String,
    pub reason: String,
}

/// Inputs are validated records sorted by canonical identity. All owners of a
/// path survive the deterministic grouping. The store adapter selects the
/// retained policy representative after evaluating every declaration.
pub fn merge(facts: &[InterestFact]) -> Result<BTreeMap<String, Vec<InterestFact>>, &'static str> {
    if facts.len() > MAX_INTEREST_RECORDS {
        return Err("retention record count exceeds limit");
    }
    let mut by_path = BTreeMap::<String, Vec<InterestFact>>::new();
    let mut prior_identity = None;
    for fact in facts {
        if fact.owner.is_empty() || fact.path.is_empty() || fact.reason.is_empty() {
            return Err("retention record has empty owner, path, or reason");
        }
        if fact.owner.len() > 256 || fact.path.len() > 4_096 || fact.reason.len() > 256 {
            return Err("retention record field exceeds limit");
        }
        if prior_identity.is_some_and(|previous: &str| previous >= fact.identity.as_str()) {
            return Err("duplicate or unordered retention record identity");
        }
        prior_identity = Some(&fact.identity);
        by_path.entry(fact.path.clone()).or_default().push(fact.clone());
    }
    Ok(by_path)
}

pub fn release_identities(
    facts: &[InterestFact],
    owner: &str,
    path: &str,
    reason: Option<&str>,
) -> Result<Vec<String>, &'static str> {
    if owner.is_empty() || path.is_empty() {
        return Err("release requires owner and path");
    }
    let grouped = merge(facts)?;
    let Some(path_facts) = grouped.get(path) else {
        return Ok(Vec::new());
    };
    if path_facts.iter().all(|fact| fact.owner != owner) {
        return Err("foreign-owner release denied");
    }
    let matching = path_facts
        .iter()
        .filter(|fact| fact.owner == owner && reason.is_none_or(|wanted| wanted == fact.reason))
        .map(|fact| fact.identity.clone())
        .collect::<Vec<_>>();
    if matching.len() > 1 {
        return Err("ambiguous release: specify an exact reason");
    }
    Ok(matching)
}

#[must_use]
pub fn identity(canonical_bytes: &[u8]) -> [u8; 32] {
    *blake3::hash(canonical_bytes).as_bytes()
}

#[cfg(test)]
mod tests {
    use alloc::vec;

    use super::*;

    #[test]
    fn merge_preserves_two_owners_and_release_is_scoped() {
        let first = InterestFact {
            identity: "a".into(),
            owner: "a".into(),
            path: "p".into(),
            reason: "build".into(),
        };
        let second = InterestFact {
            identity: "b".into(),
            owner: "b".into(),
            path: "p".into(),
            reason: "ci".into(),
        };
        let facts = vec![first, second];
        assert_eq!(merge(&facts).unwrap()["p"].len(), 2);
        assert_eq!(release_identities(&facts, "a", "p", None).unwrap(), vec!["a"]);
        assert_eq!(release_identities(&facts, "c", "p", None), Err("foreign-owner release denied"));
        assert!(merge(&[facts[0].clone(), facts[0].clone()]).is_err());
    }
    #[test]
    fn record_count_above_bound_is_rejected() {
        let fact = InterestFact {
            identity: "a".into(),
            owner: "a".into(),
            path: "p".into(),
            reason: "build".into(),
        };
        let facts = vec![fact; MAX_INTEREST_RECORDS + 1];
        assert_eq!(merge(&facts), Err("retention record count exceeds limit"));
    }
}
