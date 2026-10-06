#![no_std]
//! Bounded, ephemeral build observations; never store or evidence authority.

extern crate alloc;

use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec::Vec;
use core::fmt;

use serde::Deserialize;
use serde::Serialize;

pub const VERSION: u32 = 1;
pub const MAX_FACT_BYTES: usize = 2_048;
pub const MAX_FACTS: usize = 4_096;
pub const MAX_FILTER_BYTES: usize = 256;
pub const MAX_SUBSCRIBERS: usize = 128;
pub const MAX_PENDING_EVENTS: usize = 64;
pub const MAX_FRAME_BYTES: usize = 4_096;

const FACT_DOMAIN: &[u8] = b"mantle.live-state.fact.v1\0";
const ID_BYTES: usize = 64;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FactKind {
    Goal,
    Worker,
    Reservation,
    Outcome,
    ServiceReadiness,
}

impl FactKind {
    const fn as_str(self) -> &'static str {
        match self {
            Self::Goal => "goal",
            Self::Worker => "worker",
            Self::Reservation => "reservation",
            Self::Outcome => "outcome",
            Self::ServiceReadiness => "service_readiness",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Fact {
    pub version: u32,
    pub id: String,
    pub owner: String,
    pub kind: FactKind,
    pub subject: String,
    pub state: String,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Filter {
    pub owner: Option<String>,
    pub kind: Option<FactKind>,
    pub subject_prefix: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Error {
    UnsupportedVersion,
    EmptyField(&'static str),
    ControlCharacter(&'static str),
    FactTooLarge,
    FilterTooLarge,
    InvalidId,
    CapacityExceeded,
    IdentityCollision,
    OwnerMismatch,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnsupportedVersion => f.write_str("unsupported live fact version"),
            Self::EmptyField(field) => write!(f, "empty {field}"),
            Self::ControlCharacter(field) => write!(f, "control character in {field}"),
            Self::FactTooLarge => f.write_str("live fact exceeds byte bound"),
            Self::FilterTooLarge => f.write_str("live filter exceeds byte bound"),
            Self::InvalidId => f.write_str("invalid live fact identity"),
            Self::CapacityExceeded => f.write_str("live fact count exceeds bound"),
            Self::IdentityCollision => f.write_str("live fact identity collision"),
            Self::OwnerMismatch => f.write_str("live fact owner mismatch"),
        }
    }
}

#[derive(Clone, Copy)]
enum TextField {
    Owner,
    Subject,
    State,
}

impl TextField {
    const fn name(self) -> &'static str {
        match self {
            Self::Owner => "owner",
            Self::Subject => "subject",
            Self::State => "state",
        }
    }
}

fn check_text(value: &str, field: TextField, bound: usize, too_large: Error) -> Result<(), Error> {
    if value.is_empty() {
        return Err(Error::EmptyField(field.name()));
    }
    if value.len() > bound {
        return Err(too_large);
    }
    if value.chars().any(char::is_control) {
        return Err(Error::ControlCharacter(field.name()));
    }
    Ok(())
}

// All control characters are rejected, so JSON only expands quotes and backslashes.
fn json_text_bytes(value: &str) -> Option<usize> {
    let escapes = value.as_bytes().iter().filter(|&&byte| byte == b'"' || byte == b'\\').count();
    value.len().checked_add(escapes)
}

struct FactFields<'a> {
    owner: &'a str,
    kind: FactKind,
    subject: &'a str,
    state: &'a str,
}

fn fact_wire_bytes(fields: FactFields<'_>) -> Option<usize> {
    // Exact JSON envelope, with its one version digit and 64 hexadecimal ID bytes.
    let mut bytes = r#"{"version":,"id":"","owner":"","kind":"","subject":"","state":""}"#.len();
    bytes = bytes.checked_add(1)?.checked_add(ID_BYTES)?;
    bytes = bytes.checked_add(json_text_bytes(fields.owner)?)?;
    bytes = bytes.checked_add(fields.kind.as_str().len())?;
    bytes = bytes.checked_add(json_text_bytes(fields.subject)?)?;
    bytes = bytes.checked_add(json_text_bytes(fields.state)?)?;
    Some(bytes)
}

fn filter_wire_bytes(filter: &Filter) -> Option<usize> {
    let field_bytes = |value: Option<&str>| match value {
        Some(text) => json_text_bytes(text)?.checked_add(2),
        None => Some(4), // null
    };
    let mut bytes = r#"{"owner":,"kind":,"subject_prefix":}"#.len();
    bytes = bytes.checked_add(field_bytes(filter.owner.as_deref())?)?;
    bytes = bytes.checked_add(field_bytes(filter.kind.map(FactKind::as_str))?)?;
    bytes = bytes.checked_add(field_bytes(filter.subject_prefix.as_deref())?)?;
    Some(bytes)
}

fn hash_field(hasher: &mut blake3::Hasher, field: &str) {
    hasher.update(&(field.len() as u64).to_le_bytes());
    hasher.update(field.as_bytes());
}

/// Stable identity for an owner/kind/subject, independent of the changing state.
pub fn fact_id(owner: &str, kind: FactKind, subject: &str) -> String {
    let mut hasher = blake3::Hasher::new();
    hasher.update(FACT_DOMAIN);
    hash_field(&mut hasher, owner);
    hash_field(&mut hasher, kind.as_str());
    hash_field(&mut hasher, subject);
    hasher.finalize().to_hex().as_str().into()
}

impl Fact {
    pub fn new(owner: &str, kind: FactKind, subject: &str, state: &str) -> Result<Self, Error> {
        check_text(owner, TextField::Owner, MAX_FACT_BYTES, Error::FactTooLarge)?;
        check_text(subject, TextField::Subject, MAX_FACT_BYTES, Error::FactTooLarge)?;
        check_text(state, TextField::State, MAX_FACT_BYTES, Error::FactTooLarge)?;
        if fact_wire_bytes(FactFields {
            owner,
            kind,
            subject,
            state,
        })
        .is_none_or(|bytes| bytes > MAX_FACT_BYTES)
        {
            return Err(Error::FactTooLarge);
        }
        let fact = Self {
            version: VERSION,
            id: fact_id(owner, kind, subject),
            owner: owner.into(),
            kind,
            subject: subject.into(),
            state: state.into(),
        };
        // Inputs and serialized size have been checked before any string allocation.
        Ok(fact)
    }

    pub fn validate(&self) -> Result<(), Error> {
        if self.version != VERSION {
            return Err(Error::UnsupportedVersion);
        }
        if self.id.len() != ID_BYTES || !self.id.bytes().all(|byte| matches!(byte, b'0'..=b'9' | b'a'..=b'f')) {
            return Err(Error::InvalidId);
        }
        check_text(&self.owner, TextField::Owner, MAX_FACT_BYTES, Error::FactTooLarge)?;
        check_text(&self.subject, TextField::Subject, MAX_FACT_BYTES, Error::FactTooLarge)?;
        check_text(&self.state, TextField::State, MAX_FACT_BYTES, Error::FactTooLarge)?;
        if fact_wire_bytes(FactFields {
            owner: &self.owner,
            kind: self.kind,
            subject: &self.subject,
            state: &self.state,
        })
        .is_none_or(|bytes| bytes > MAX_FACT_BYTES)
        {
            return Err(Error::FactTooLarge);
        }
        if self.id != fact_id(&self.owner, self.kind, &self.subject) {
            return Err(Error::InvalidId);
        }
        Ok(())
    }
}

impl Filter {
    pub fn validate(&self) -> Result<(), Error> {
        if let Some(owner) = &self.owner {
            check_text(owner, TextField::Owner, MAX_FILTER_BYTES, Error::FilterTooLarge)?;
        }
        if let Some(prefix) = &self.subject_prefix {
            if prefix.len() > MAX_FILTER_BYTES {
                return Err(Error::FilterTooLarge);
            }
            if prefix.chars().any(char::is_control) {
                return Err(Error::ControlCharacter("subject_prefix"));
            }
        }
        if filter_wire_bytes(self).is_none_or(|bytes| bytes > MAX_FILTER_BYTES) {
            return Err(Error::FilterTooLarge);
        }
        Ok(())
    }

    /// Call `validate` on wire input before accepting a subscription.
    pub fn matches(&self, fact: &Fact) -> bool {
        self.owner.as_ref().is_none_or(|owner| fact.owner == *owner)
            && self.kind.is_none_or(|kind| fact.kind == kind)
            && self.subject_prefix.as_ref().is_none_or(|prefix| fact.subject.starts_with(prefix))
    }
}

/// Current facts keyed by their stable identity, with no durable backing store.
#[derive(Debug, Default)]
pub struct LiveSet {
    facts: BTreeMap<String, Fact>,
}

impl LiveSet {
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns true only for a new fact or a changed state; identical publications do nothing.
    pub fn publish(&mut self, fact: Fact) -> Result<bool, Error> {
        fact.validate()?;
        if let Some(previous) = self.facts.get(&fact.id) {
            if previous.owner != fact.owner {
                return Err(Error::OwnerMismatch);
            }
            if previous.kind != fact.kind || previous.subject != fact.subject {
                return Err(Error::IdentityCollision);
            }
            if previous.state == fact.state {
                return Ok(false);
            }
            self.facts.insert(fact.id.clone(), fact);
            return Ok(true);
        }
        if self.facts.len() >= MAX_FACTS {
            return Err(Error::CapacityExceeded);
        }
        self.facts.insert(fact.id.clone(), fact);
        Ok(true)
    }

    pub fn retract(&mut self, id: &str) -> Option<Fact> {
        self.facts.remove(id)
    }

    /// Removes only this owner's facts; unaffected owners retain their state.
    pub fn retract_owner(&mut self, owner: &str) -> Vec<Fact> {
        let ids: Vec<_> = self.facts.iter().filter(|(_, fact)| fact.owner == owner).map(|(id, _)| id.clone()).collect();
        ids.into_iter().filter_map(|id| self.facts.remove(&id)).collect()
    }

    /// Deterministic identity order, without copying retained facts.
    pub fn snapshot<'a>(&'a self, filter: &'a Filter) -> impl Iterator<Item = &'a Fact> + 'a {
        self.facts.values().filter(move |fact| filter.matches(fact))
    }
}

#[cfg(test)]
mod tests {
    use alloc::string::ToString;
    use alloc::vec;

    use super::*;

    fn fact(owner: &str, kind: FactKind, subject: &str, state: &str) -> Fact {
        Fact::new(owner, kind, subject, state).unwrap()
    }

    #[test]
    fn identity_is_unambiguous_and_state_independent() {
        let first = fact("ab", FactKind::Goal, "c", "discovered");
        assert_eq!(first.id, fact("ab", FactKind::Goal, "c", "running").id);
        assert_ne!(first.id, fact("a", FactKind::Goal, "bc", "discovered").id);
        assert_ne!(first.id, fact("ab", FactKind::Worker, "c", "discovered").id);
        assert_ne!(first.id, fact("ac", FactKind::Goal, "b", "discovered").id);
        assert_eq!(first.id.len(), ID_BYTES);
        assert_eq!(serde_json::to_string(&FactKind::ServiceReadiness).unwrap(), "\"service_readiness\"");
        first.validate().unwrap();
    }

    #[test]
    fn malformed_and_oversized_facts_are_rejected() {
        assert_eq!(Fact::new("", FactKind::Goal, "s", "ready"), Err(Error::EmptyField("owner")));
        assert_eq!(Fact::new("o", FactKind::Goal, "s\n", "ready"), Err(Error::ControlCharacter("subject")));
        assert_eq!(Fact::new("o", FactKind::Goal, "s", "\u{7f}"), Err(Error::ControlCharacter("state")));
        let mut malformed = fact("o", FactKind::Goal, "s", "ready");
        malformed.id = fact_id("other", FactKind::Goal, "s");
        assert_eq!(malformed.validate(), Err(Error::InvalidId));
        malformed.id = fact_id("o", FactKind::Goal, "s");
        malformed.version += 1;
        assert_eq!(malformed.validate(), Err(Error::UnsupportedVersion));
        assert_eq!(Fact::new("o", FactKind::Goal, "s", &"x".repeat(MAX_FACT_BYTES)), Err(Error::FactTooLarge));
        let escaped = fact("o", FactKind::Goal, "s", &"\\".repeat(900));
        assert!(serde_json::to_vec(&escaped).unwrap().len() <= MAX_FACT_BYTES);
        assert_eq!(Fact::new("o", FactKind::Goal, "s", &"\\".repeat(1_050)), Err(Error::FactTooLarge));
        let baseline = fact("o", FactKind::Goal, "s", "x");
        let available = MAX_FACT_BYTES.checked_sub(serde_json::to_vec(&baseline).unwrap().len()).unwrap();
        let mut exact_state = "x".repeat(available.checked_add(1).unwrap());
        let exact = fact("o", FactKind::Goal, "s", &exact_state);
        assert_eq!(serde_json::to_vec(&exact).unwrap().len(), MAX_FACT_BYTES);
        exact.validate().unwrap();
        exact_state.push('x');
        assert_eq!(Fact::new("o", FactKind::Goal, "s", &exact_state), Err(Error::FactTooLarge));
    }

    #[test]
    fn malformed_and_oversized_filters_are_rejected() {
        let mut filter = Filter {
            owner: Some("builder".into()),
            kind: Some(FactKind::Goal),
            subject_prefix: Some("root/".into()),
        };
        filter.validate().unwrap();
        let initial_bytes = serde_json::to_vec(&filter).unwrap().len();
        let available = MAX_FILTER_BYTES.checked_sub(initial_bytes).unwrap();
        let mut prefix = String::with_capacity("root/".len().checked_add(available).unwrap());
        prefix.push_str("root/");
        prefix.extend(core::iter::repeat_n('x', available));
        filter.subject_prefix = Some(prefix);
        assert_eq!(serde_json::to_vec(&filter).unwrap().len(), MAX_FILTER_BYTES);
        filter.validate().unwrap();
        filter.subject_prefix.as_mut().unwrap().push('x');
        assert_eq!(filter.validate(), Err(Error::FilterTooLarge));
        filter.subject_prefix = Some("root/\n".into());
        assert_eq!(filter.validate(), Err(Error::ControlCharacter("subject_prefix")));
        filter.subject_prefix = Some("\\".repeat(MAX_FILTER_BYTES));
        assert_eq!(filter.validate(), Err(Error::FilterTooLarge));
        filter.owner = Some(String::new());
        filter.subject_prefix = None;
        assert_eq!(filter.validate(), Err(Error::EmptyField("owner")));
    }

    #[test]
    fn snapshots_match_in_identity_order_and_retain_unrelated_facts() {
        let mut set = LiveSet::new();
        let first = fact("owner-a", FactKind::Goal, "root/a", "discovered");
        let other = fact("owner-b", FactKind::Goal, "root/b", "waiting");
        let second = fact("owner-a", FactKind::Goal, "root/c", "running");
        for current in [&second, &other, &first] {
            assert_eq!(set.publish(current.clone()), Ok(true));
        }
        assert_eq!(set.publish(first.clone()), Ok(false));
        let filter = Filter {
            owner: Some("owner-a".into()),
            kind: Some(FactKind::Goal),
            subject_prefix: Some("root/".into()),
        };
        let expected: Vec<_> = set.snapshot(&filter).map(|fact| fact.id.as_str()).collect();
        let mut sorted = vec![first.id.as_str(), second.id.as_str()];
        sorted.sort_unstable();
        assert_eq!(expected, sorted);
        assert_eq!(set.publish(fact("owner-a", FactKind::Goal, "root/a", "running")), Ok(true));
        assert_eq!(
            set.snapshot(&Filter {
                owner: Some("owner-b".into()),
                ..Filter::default()
            })
            .next(),
            Some(&other)
        );
        assert_eq!(set.retract(&first.id).unwrap().state, "running");
        assert!(set.retract(&first.id).is_none());
        assert_eq!(set.snapshot(&Filter::default()).count(), 2);
    }

    #[test]
    fn owner_loss_retracts_only_that_owner_in_identity_order() {
        let mut set = LiveSet::new();
        let first = fact("worker-1", FactKind::Worker, "worker-1", "present");
        let second = fact("worker-1", FactKind::Goal, "build", "dispatched");
        let unrelated = fact("worker-2", FactKind::Worker, "worker-2", "present");
        for current in [&first, &unrelated, &second] {
            set.publish(current.clone()).unwrap();
        }
        let removed = set.retract_owner("worker-1");
        let mut expected = [first.id, second.id];
        expected.sort_unstable();
        assert_eq!(removed.iter().map(|fact| &fact.id).collect::<Vec<_>>(), expected.iter().collect::<Vec<_>>());
        assert!(set.retract_owner("worker-1").is_empty());
        assert_eq!(set.snapshot(&Filter::default()).next(), Some(&unrelated));
    }

    #[test]
    fn full_set_rejects_new_id_but_allows_existing_transition() {
        let mut set = LiveSet::new();
        for index in 0..MAX_FACTS {
            let subject = index.to_string();
            set.publish(fact("worker", FactKind::Goal, &subject, "pending")).unwrap();
        }
        assert_eq!(set.publish(fact("worker", FactKind::Goal, "overflow", "pending")), Err(Error::CapacityExceeded));
        assert_eq!(set.publish(fact("worker", FactKind::Goal, "0", "running")), Ok(true));
        assert_eq!(set.snapshot(&Filter::default()).count(), MAX_FACTS);
    }
}
