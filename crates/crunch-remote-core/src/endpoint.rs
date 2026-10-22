//! Bounded name-to-session decisions, not bearer authentication or payload trust.
//!
//! Adapters supply canonical local identifiers (not raw URLs), current time and
//! existing ticket/transfer/cache/layer policy facts. Neither a descriptor nor a
//! resolved handle verifies a credential, signature, proof chain or content.

use alloc::string::String;
use alloc::vec::Vec;

pub const MAX_ENDPOINT_BINDINGS: usize = 4_096;
pub const MAX_ENDPOINT_NAME_BYTES: usize = 256;
pub const MAX_ENDPOINT_SCOPE_BYTES: usize = 256;
pub const MAX_ENDPOINT_TRUST_BYTES: usize = 512;
pub const MAX_ENDPOINT_PREFIX_BYTES: usize = 256;

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "kebab-case"))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EndpointKind {
    BuilderTicket,
    Transfer,
    Substituter,
    BaseLayer,
}

/// Exact application-owned requirements; these strings never create policy.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(deny_unknown_fields))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EndpointBindingSpec {
    pub kind: EndpointKind,
    pub name: String,
    pub scope: String,
    pub trust_requirement: String,
    pub store_prefix: String,
    pub expires_unix_s: u64,
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(deny_unknown_fields))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EndpointBinding {
    pub spec: EndpointBindingSpec,
    pub generation: u64,
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(deny_unknown_fields))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EndpointBindingRecord {
    pub binding: EndpointBinding,
    pub revoked: bool,
}

/// Tombstones count toward the bound and cannot be deleted or reset by this API.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct EndpointBindingTable {
    records: Vec<EndpointBindingRecord>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EndpointBindingError {
    InvalidName,
    InvalidScope,
    InvalidTrustRequirement,
    InvalidStorePrefix,
    InvalidExpiry,
    InvalidGeneration,
    TableFull,
    DuplicateBinding,
    UnknownBinding,
    AlreadyActive,
    Revoked,
    Expired,
    StaleGeneration,
    BindingMismatch,
    GenerationExhausted,
}

/// Local admission view only. It is deliberately borrowed, not a portable token.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ResolvedEndpoint<'a> {
    binding: &'a EndpointBinding,
}

impl<'a> ResolvedEndpoint<'a> {
    pub fn binding(self) -> &'a EndpointBinding {
        self.binding
    }
}

impl EndpointBindingSpec {
    pub fn validate(&self) -> Result<(), EndpointBindingError> {
        validate_endpoint_name(&self.name)?;
        if !bounded_identity(&self.scope, MAX_ENDPOINT_SCOPE_BYTES) {
            return Err(EndpointBindingError::InvalidScope);
        }
        if !bounded_identity(&self.trust_requirement, MAX_ENDPOINT_TRUST_BYTES) {
            return Err(EndpointBindingError::InvalidTrustRequirement);
        }
        validate_store_prefix(&self.store_prefix)?;
        if self.expires_unix_s == 0 {
            return Err(EndpointBindingError::InvalidExpiry);
        }
        Ok(())
    }
}

impl EndpointBinding {
    pub fn validate(&self) -> Result<(), EndpointBindingError> {
        self.spec.validate()?;
        if self.generation == 0 {
            return Err(EndpointBindingError::InvalidGeneration);
        }
        Ok(())
    }
}

impl EndpointBindingTable {
    pub fn from_records(records: Vec<EndpointBindingRecord>) -> Result<Self, EndpointBindingError> {
        if records.len() > MAX_ENDPOINT_BINDINGS {
            return Err(EndpointBindingError::TableFull);
        }
        for (index, record) in records.iter().enumerate() {
            record.binding.validate()?;
            if records[..index].iter().any(|prior| same_name(&prior.binding.spec, &record.binding.spec)) {
                return Err(EndpointBindingError::DuplicateBinding);
            }
        }
        Ok(Self { records })
    }

    pub fn records(&self) -> &[EndpointBindingRecord] {
        &self.records
    }

    pub fn into_records(self) -> Vec<EndpointBindingRecord> {
        self.records
    }
    // r[impl remote_builds.single_binding_resolution]
    // r[impl remote_builds.binding_revocation]
    pub fn create(
        &mut self,
        spec: EndpointBindingSpec,
        expected_previous_generation: Option<u64>,
        now_unix_s: u64,
    ) -> Result<&EndpointBinding, EndpointBindingError> {
        spec.validate()?;
        if now_unix_s >= spec.expires_unix_s {
            return Err(EndpointBindingError::Expired);
        }
        if let Some(index) = self.records.iter().position(|record| same_name(&record.binding.spec, &spec)) {
            let record = &mut self.records[index];
            if expected_previous_generation != Some(record.binding.generation) {
                return Err(EndpointBindingError::StaleGeneration);
            }
            if !record.revoked {
                return Err(EndpointBindingError::AlreadyActive);
            }
            let generation =
                record.binding.generation.checked_add(1).ok_or(EndpointBindingError::GenerationExhausted)?;
            *record = EndpointBindingRecord {
                binding: EndpointBinding { spec, generation },
                revoked: false,
            };
            return Ok(&record.binding);
        }
        if expected_previous_generation.is_some() {
            return Err(EndpointBindingError::UnknownBinding);
        }
        if self.records.len() >= MAX_ENDPOINT_BINDINGS {
            return Err(EndpointBindingError::TableFull);
        }
        self.records.push(EndpointBindingRecord {
            binding: EndpointBinding { spec, generation: 1 },
            revoked: false,
        });
        let created = self.records.last().ok_or(EndpointBindingError::UnknownBinding)?;
        Ok(&created.binding)
    }

    /// Retract exactly the supplied descriptor, never another generation or scope.
    /// This touches no store, receipt, admitted content or content verification.
    // r[impl remote_builds.binding_revocation]
    pub fn revoke(&mut self, exact: &EndpointBinding) -> Result<(), EndpointBindingError> {
        exact.validate()?;
        let record = self
            .records
            .iter_mut()
            .find(|record| same_name(&record.binding.spec, &exact.spec))
            .ok_or(EndpointBindingError::UnknownBinding)?;
        compare_binding(&record.binding, exact)?;
        record.revoked = true;
        Ok(())
    }

    // r[impl remote_builds.single_binding_resolution]
    pub fn resolve(
        &self,
        presented: &EndpointBinding,
        now_unix_s: u64,
    ) -> Result<ResolvedEndpoint<'_>, EndpointBindingError> {
        presented.validate()?;
        let record = self
            .records
            .iter()
            .find(|record| same_name(&record.binding.spec, &presented.spec))
            .ok_or(EndpointBindingError::UnknownBinding)?;
        compare_binding(&record.binding, presented)?;
        if record.revoked {
            return Err(EndpointBindingError::Revoked);
        }
        if now_unix_s >= record.binding.spec.expires_unix_s {
            return Err(EndpointBindingError::Expired);
        }
        Ok(ResolvedEndpoint {
            binding: &record.binding,
        })
    }

    /// Re-resolve at the receiver against its own current table and expected site
    /// requirements before manifest/checkpoint/payload work. Caller must still
    /// authenticate and enforce existing source-specific admission independently.
    // r[impl remote_builds.single_binding_resolution]
    pub fn verify_received(
        &self,
        presented: &EndpointBinding,
        expected: EndpointReceiverRequirements<'_>,
        now_unix_s: u64,
    ) -> Result<ResolvedEndpoint<'_>, EndpointBindingError> {
        let has_matching_kind = presented.spec.kind == expected.kind;
        let has_matching_scope = presented.spec.scope == expected.scope;
        let has_matching_trust = presented.spec.trust_requirement == expected.trust_requirement;
        let has_matching_prefix = presented.spec.store_prefix == expected.store_prefix;
        let is_matching = has_matching_kind && has_matching_scope && has_matching_trust && has_matching_prefix;
        if !is_matching {
            return Err(EndpointBindingError::BindingMismatch);
        }
        self.resolve(presented, now_unix_s)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct EndpointReceiverRequirements<'a> {
    pub kind: EndpointKind,
    pub scope: &'a str,
    pub trust_requirement: &'a str,
    pub store_prefix: &'a str,
}

pub fn validate_endpoint_name(name: &str) -> Result<(), EndpointBindingError> {
    if name.is_empty() || name.len() > MAX_ENDPOINT_NAME_BYTES {
        return Err(EndpointBindingError::InvalidName);
    }
    let edge = |byte: u8| byte.is_ascii_lowercase() || byte.is_ascii_digit();
    let bytes = name.as_bytes();
    let has_valid_edge = bytes.first().is_some_and(|byte| edge(*byte)) && bytes.last().is_some_and(|byte| edge(*byte));
    if !has_valid_edge {
        return Err(EndpointBindingError::InvalidName);
    }
    if !name.bytes().all(|byte| edge(byte) || matches!(byte, b'-' | b'_' | b'.')) || name.contains("..") {
        return Err(EndpointBindingError::InvalidName);
    }
    Ok(())
}

fn bounded_identity(value: &str, maximum: usize) -> bool {
    !value.is_empty() && value.len() <= maximum && value.bytes().all(|byte| byte.is_ascii_graphic())
}

fn validate_store_prefix(prefix: &str) -> Result<(), EndpointBindingError> {
    if prefix.len() > MAX_ENDPOINT_PREFIX_BYTES || !prefix.starts_with('/') || prefix.ends_with('/') {
        return Err(EndpointBindingError::InvalidStorePrefix);
    }
    for component in prefix[1..].split('/') {
        let is_empty = component.is_empty();
        let is_current = component == ".";
        let is_parent = component == "..";
        let has_safe_bytes =
            component.bytes().all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.'));
        let is_valid = !is_empty && !is_current && !is_parent && has_safe_bytes;
        if !is_valid {
            return Err(EndpointBindingError::InvalidStorePrefix);
        }
    }
    Ok(())
}

fn same_name(left: &EndpointBindingSpec, right: &EndpointBindingSpec) -> bool {
    left.kind == right.kind && left.name == right.name
}

fn compare_binding(current: &EndpointBinding, presented: &EndpointBinding) -> Result<(), EndpointBindingError> {
    if current.generation != presented.generation {
        return Err(EndpointBindingError::StaleGeneration);
    }
    if current != presented {
        return Err(EndpointBindingError::BindingMismatch);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn spec(kind: EndpointKind, name: &str) -> EndpointBindingSpec {
        EndpointBindingSpec {
            kind,
            name: name.into(),
            scope: "farm-a".into(),
            trust_requirement: "existing-policy-a".into(),
            store_prefix: "/mantle/store".into(),
            expires_unix_s: 100,
        }
    }

    fn expected() -> EndpointReceiverRequirements<'static> {
        EndpointReceiverRequirements {
            kind: EndpointKind::BuilderTicket,
            scope: "farm-a",
            trust_requirement: "existing-policy-a",
            store_prefix: "/mantle/store",
        }
    }

    #[test]
    fn resolves_all_entry_kinds_and_receiver_rechecks_every_binding_fact() {
        for kind in [
            EndpointKind::BuilderTicket,
            EndpointKind::Transfer,
            EndpointKind::Substituter,
            EndpointKind::BaseLayer,
        ] {
            let mut table = EndpointBindingTable::default();
            let binding = table.create(spec(kind, "entry-a"), None, 1).unwrap().clone();
            assert_eq!(table.resolve(&binding, 2).unwrap().binding(), &binding);
            assert_eq!(table.resolve(&binding, 3).unwrap().binding().generation, 1);
            let requirements = EndpointReceiverRequirements { kind, ..expected() };
            assert_eq!(table.verify_received(&binding, requirements, 3).unwrap().binding(), &binding);
            assert_eq!(table.resolve(&binding, 100), Err(EndpointBindingError::Expired));
            let mut unknown = binding.clone();
            unknown.spec.name = "unknown".into();
            assert_eq!(table.resolve(&unknown, 3), Err(EndpointBindingError::UnknownBinding));
            for mutate in 0..5 {
                let mut forged = binding.clone();
                match mutate {
                    0 => forged.spec.scope = "farm-b".into(),
                    1 => forged.spec.trust_requirement = "weaker".into(),
                    2 => forged.spec.store_prefix = "/nix/store".into(),
                    3 => forged.spec.expires_unix_s += 1,
                    _ => forged.generation += 1,
                }
                assert!(table.resolve(&forged, 3).is_err());
            }
            let wrong_prefix = EndpointReceiverRequirements {
                store_prefix: "/nix/store",
                ..requirements
            };
            assert_eq!(table.verify_received(&binding, wrong_prefix, 3), Err(EndpointBindingError::BindingMismatch));
        }
    }

    #[test]
    fn exact_revoke_retains_tombstone_and_recreation_fences_old_presentations() {
        let mut table = EndpointBindingTable::default();
        let old = table.create(spec(EndpointKind::BuilderTicket, "entry-a"), None, 1).unwrap().clone();
        let other = table.create(spec(EndpointKind::BuilderTicket, "entry-b"), None, 1).unwrap().clone();
        let mut forged = old.clone();
        forged.spec.scope = "farm-b".into();
        assert_eq!(table.revoke(&forged), Err(EndpointBindingError::BindingMismatch));
        table.revoke(&old).unwrap();
        assert_eq!(table.resolve(&old, 2), Err(EndpointBindingError::Revoked));
        assert_eq!(table.records().len(), 2);
        assert_eq!(table.resolve(&other, 2).unwrap().binding(), &other);
        assert_eq!(table.create(old.spec.clone(), None, 2), Err(EndpointBindingError::StaleGeneration));
        let new = table.create(old.spec.clone(), Some(1), 2).unwrap().clone();
        assert_eq!(new.generation, 2);
        assert_eq!(table.resolve(&old, 2), Err(EndpointBindingError::StaleGeneration));
        assert_eq!(table.revoke(&old), Err(EndpointBindingError::StaleGeneration));
        assert_eq!(table.resolve(&new, 2).unwrap().binding(), &new);
    }

    #[test]
    fn rejects_noncanonical_names_corrupt_tables_and_generation_overflow() {
        for name in ["", "Entry", " entry", "entry/child", "a..b", "a%2fb", "a.", "é", "a\n"] {
            assert_eq!(validate_endpoint_name(name), Err(EndpointBindingError::InvalidName));
        }
        assert_eq!(
            validate_endpoint_name(&"a".repeat(MAX_ENDPOINT_NAME_BYTES + 1)),
            Err(EndpointBindingError::InvalidName)
        );
        let binding = EndpointBinding {
            spec: spec(EndpointKind::Transfer, "entry-a"),
            generation: u64::MAX,
        };
        let record = EndpointBindingRecord {
            binding: binding.clone(),
            revoked: true,
        };
        assert_eq!(
            EndpointBindingTable::from_records(alloc::vec![record.clone(), record.clone()]),
            Err(EndpointBindingError::DuplicateBinding)
        );
        assert_eq!(
            EndpointBindingTable::from_records(alloc::vec![record.clone(); MAX_ENDPOINT_BINDINGS + 1]),
            Err(EndpointBindingError::TableFull)
        );
        let mut table = EndpointBindingTable::from_records(alloc::vec![record]).unwrap();
        assert_eq!(table.create(binding.spec, Some(u64::MAX), 1), Err(EndpointBindingError::GenerationExhausted));
        assert!(table.records()[0].revoked);
    }
}
