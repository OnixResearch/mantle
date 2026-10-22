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

#[derive(Clone, Debug)]
pub struct ReleaseSelector {
    pub owner: String,
    pub path: String,
    pub reason: Option<String>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum InterestError {
    TooManyRecords,
    EmptyField,
    FieldTooLong,
    UnorderedIdentity,
    MissingReleaseScope,
    ForeignOwner,
    AmbiguousRelease,
}

impl core::fmt::Display for InterestError {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter.write_str(match self {
            Self::TooManyRecords => "retention record count exceeds limit",
            Self::EmptyField => "retention record has empty owner, path, or reason",
            Self::FieldTooLong => "retention record field exceeds limit",
            Self::UnorderedIdentity => "duplicate or unordered retention record identity",
            Self::MissingReleaseScope => "release requires owner and path",
            Self::ForeignOwner => "foreign-owner release denied",
            Self::AmbiguousRelease => "ambiguous release: specify an exact reason",
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InterestIdentity {
    pub canonical_bytes: Vec<u8>,
    pub digest: [u8; 32],
}

#[derive(Clone, Debug)]
pub struct InterestRelease {
    pub selector: ReleaseSelector,
    pub identities: Vec<String>,
}
/// Inputs are validated records sorted by canonical identity. All owners of a
/// path survive the deterministic grouping. The store adapter selects the
/// retained policy representative after evaluating every declaration.
pub fn merge(facts: Vec<InterestFact>) -> Result<BTreeMap<String, Vec<InterestFact>>, InterestError> {
    validate_facts(&facts)?;
    let fact_count = facts.len();
    let mut by_path = BTreeMap::<String, Vec<InterestFact>>::new();
    for fact in facts {
        let path = fact.path.clone();
        let path_facts = by_path.entry(path).or_insert_with(|| Vec::with_capacity(fact_count));
        path_facts.push(fact);
    }
    assert_eq!(by_path.is_empty(), fact_count == 0);
    assert!(by_path.len() <= fact_count);
    Ok(by_path)
}

fn validate_facts(facts: &[InterestFact]) -> Result<(), InterestError> {
    if facts.len() > MAX_INTEREST_RECORDS {
        return Err(InterestError::TooManyRecords);
    }
    let mut prior_identity = None;
    for fact in facts {
        if fact.owner.is_empty() || fact.path.is_empty() || fact.reason.is_empty() {
            return Err(InterestError::EmptyField);
        }
        if fact.owner.len() > 256 || fact.path.len() > 4_096 || fact.reason.len() > 256 {
            return Err(InterestError::FieldTooLong);
        }
        if prior_identity.is_some_and(|previous: &str| previous >= fact.identity.as_str()) {
            return Err(InterestError::UnorderedIdentity);
        }
        prior_identity = Some(fact.identity.as_str());
    }
    Ok(())
}

/// Select exactly one declaration for a release.
///
/// A release that names no reason is refused when the owner holds more than one
/// declaration for the path, because the caller must choose between reasons it
/// declared itself.
pub fn release_identities(
    facts: Vec<InterestFact>,
    selector: ReleaseSelector,
) -> Result<InterestRelease, InterestError> {
    if selector.owner.is_empty() || selector.path.is_empty() {
        return Err(InterestError::MissingReleaseScope);
    }
    let mut grouped = merge(facts)?;
    let Some(path_facts) = grouped.remove(&selector.path) else {
        return Ok(InterestRelease {
            selector,
            identities: Vec::new(),
        });
    };
    if path_facts.iter().all(|fact| fact.owner != selector.owner) {
        return Err(InterestError::ForeignOwner);
    }
    let path_fact_count = path_facts.len();
    let matching = path_facts
        .into_iter()
        .filter(|fact| {
            fact.owner == selector.owner && selector.reason.as_ref().is_none_or(|wanted| *wanted == fact.reason)
        })
        .map(|fact| fact.identity)
        .collect::<Vec<_>>();
    if matching.len() > 1 {
        return Err(InterestError::AmbiguousRelease);
    }
    assert!(matching.len() <= path_fact_count);
    assert!(matching.len() <= 1);
    Ok(InterestRelease {
        selector,
        identities: matching,
    })
}

/// Select every declaration of one owner for one exact path.
///
/// A private owner scope holds only its own declarations, so releasing that
/// owner for one path is unambiguous even when the declarations carry distinct
/// reasons, for example after a renewed pin. The shared operator scope keeps
/// the ambiguous refusal, because there the caller must choose between reasons
/// it declared itself.
pub fn release_owner_identities(
    facts: Vec<InterestFact>,
    selector: ReleaseSelector,
) -> Result<InterestRelease, InterestError> {
    if selector.owner.is_empty() || selector.path.is_empty() {
        return Err(InterestError::MissingReleaseScope);
    }
    let mut grouped = merge(facts)?;
    let Some(path_facts) = grouped.remove(&selector.path) else {
        return Ok(InterestRelease {
            selector,
            identities: Vec::new(),
        });
    };
    if path_facts.iter().all(|fact| fact.owner != selector.owner) {
        return Err(InterestError::ForeignOwner);
    }
    let path_fact_count = path_facts.len();
    let owner_fact_count = path_facts.iter().filter(|fact| fact.owner == selector.owner).count();
    let mut identities = Vec::with_capacity(owner_fact_count);
    for fact in path_facts {
        if fact.owner == selector.owner {
            identities.push(fact.identity);
        }
    }
    assert_eq!(identities.len(), owner_fact_count);
    assert!(identities.len() <= path_fact_count);
    Ok(InterestRelease { selector, identities })
}

#[must_use]
pub fn identity(canonical_bytes: Vec<u8>) -> InterestIdentity {
    let digest = *blake3::hash(&canonical_bytes).as_bytes();
    InterestIdentity {
        canonical_bytes,
        digest,
    }
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
        assert_eq!(merge(facts.clone()).unwrap()["p"].len(), 2);
        assert_eq!(
            release_identities(facts.clone(), ReleaseSelector {
                owner: "a".into(),
                path: "p".into(),
                reason: None
            })
            .unwrap()
            .identities,
            vec!["a"]
        );
        assert_eq!(
            release_identities(facts.clone(), ReleaseSelector {
                owner: "c".into(),
                path: "p".into(),
                reason: None
            })
            .unwrap_err(),
            InterestError::ForeignOwner
        );
        assert!(merge(vec![facts[0].clone(), facts[0].clone()]).is_err());
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
        assert_eq!(merge(facts), Err(InterestError::TooManyRecords));
    }
}
