//! Checked representation migration. No payload, signature, CA, or path rewrite.
//! The adapter supplies cryptographically checked observations of the SAME NAR.

extern crate alloc;
use self::alloc::{
    collections::{BTreeMap, BTreeSet},
    string::String,
    vec::Vec,
};

pub struct ClosureRecord {
    pub identity: String,
    pub root: bool,
    pub references: Vec<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ClosureRejection {
    Bounds,
    DuplicateMember,
    DuplicateReference,
    RootRole,
    MissingMember,
    UnreachableMember,
}

pub fn validate_closed_archive(roots: Vec<String>, records: Vec<ClosureRecord>) -> Result<(), ClosureRejection> {
    if roots.is_empty()
        || roots.len() > 64
        || records.len() > 64
        || roots.iter().collect::<BTreeSet<_>>().len() != roots.len()
    {
        return Err(ClosureRejection::Bounds);
    }
    let mut by_name = BTreeMap::new();
    for record in &records {
        if record.references.len() > 64 {
            return Err(ClosureRejection::Bounds);
        }
        if by_name.insert(record.identity.clone(), record).is_some() {
            return Err(ClosureRejection::DuplicateMember);
        }
        if record.root != roots.contains(&record.identity) {
            return Err(ClosureRejection::RootRole);
        }
        if record.references.iter().collect::<BTreeSet<_>>().len() != record.references.len() {
            return Err(ClosureRejection::DuplicateReference);
        }
    }
    let mut seen = BTreeSet::new();
    let mut pending = roots;
    while let Some(name) = pending.pop() {
        if !seen.insert(name.clone()) {
            continue;
        }
        let record = by_name.get(&name).ok_or(ClosureRejection::MissingMember)?;
        pending.extend(record.references.iter().cloned());
    }
    if seen.len() != records.len() {
        return Err(ClosureRejection::UnreachableMember);
    }
    Ok(())
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DirectoryIdentity {
    pub digest: [u8; 32],
    pub descendants: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CaEvidence {
    Absent,
    FinalNar,
    MarkerNar { replacements: u32 },
    Rejected,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MigrationFacts {
    pub trusted_signature: bool,
    pub final_nar_matches: bool,
    pub payload_blake3_matches: bool,
    pub ca_path_matches: bool,
    pub ca: CaEvidence,
    pub recorded: DirectoryIdentity,
    pub current: DirectoryIdentity,
    pub legacy: DirectoryIdentity,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Disposition {
    Unchanged,
    CanonicalizeDoubledDirectoryCounts,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Rejection {
    Untrusted,
    FinalNarMismatch,
    PayloadMismatch,
    CaPathMismatch,
    CaMismatch,
    UnknownDirectoryRepresentation,
}

pub fn plan_directory_migration(facts: MigrationFacts) -> Result<Disposition, Rejection> {
    if !facts.trusted_signature {
        return Err(Rejection::Untrusted);
    }
    if !facts.final_nar_matches {
        return Err(Rejection::FinalNarMismatch);
    }
    if !facts.payload_blake3_matches {
        return Err(Rejection::PayloadMismatch);
    }
    if !facts.ca_path_matches {
        return Err(Rejection::CaPathMismatch);
    }
    match facts.ca {
        CaEvidence::Rejected | CaEvidence::MarkerNar { replacements: 0 } => return Err(Rejection::CaMismatch),
        _ => {}
    }
    if facts.recorded == facts.current {
        return Ok(Disposition::Unchanged);
    }
    if facts.recorded != facts.legacy || facts.current.descendants.checked_mul(2) != Some(facts.legacy.descendants) {
        return Err(Rejection::UnknownDirectoryRepresentation);
    }
    Ok(Disposition::CanonicalizeDoubledDirectoryCounts)
}

#[cfg(test)]
mod tests {
    use super::*;
    fn facts() -> MigrationFacts {
        let current = DirectoryIdentity {
            digest: [1; 32],
            descendants: 14,
        };
        let legacy = DirectoryIdentity {
            digest: [2; 32],
            descendants: 28,
        };
        MigrationFacts {
            trusted_signature: true,
            final_nar_matches: true,
            payload_blake3_matches: true,
            ca_path_matches: true,
            ca: CaEvidence::MarkerNar { replacements: 2 },
            recorded: legacy,
            current,
            legacy,
        }
    }
    #[test]
    fn accepts_only_proven_representations() {
        let f = facts();
        assert_eq!(plan_directory_migration(f), Ok(Disposition::CanonicalizeDoubledDirectoryCounts));
        assert_eq!(
            plan_directory_migration(MigrationFacts {
                recorded: f.current,
                ..f
            }),
            Ok(Disposition::Unchanged)
        );
    }
    #[test]
    fn rejects_every_missing_proof_before_migration() {
        let f = facts();
        for (bad, reason) in [
            (
                MigrationFacts {
                    trusted_signature: false,
                    ..f
                },
                Rejection::Untrusted,
            ),
            (
                MigrationFacts {
                    final_nar_matches: false,
                    ..f
                },
                Rejection::FinalNarMismatch,
            ),
            (
                MigrationFacts {
                    payload_blake3_matches: false,
                    ..f
                },
                Rejection::PayloadMismatch,
            ),
            (
                MigrationFacts {
                    ca_path_matches: false,
                    ..f
                },
                Rejection::CaPathMismatch,
            ),
            (
                MigrationFacts {
                    ca: CaEvidence::Rejected,
                    ..f
                },
                Rejection::CaMismatch,
            ),
            (
                MigrationFacts {
                    ca: CaEvidence::MarkerNar { replacements: 0 },
                    ..f
                },
                Rejection::CaMismatch,
            ),
            (
                MigrationFacts {
                    recorded: DirectoryIdentity {
                        digest: [3; 32],
                        ..f.legacy
                    },
                    ..f
                },
                Rejection::UnknownDirectoryRepresentation,
            ),
            (
                MigrationFacts {
                    legacy: DirectoryIdentity {
                        descendants: 29,
                        ..f.legacy
                    },
                    ..f
                },
                Rejection::UnknownDirectoryRepresentation,
            ),
        ] {
            assert_eq!(plan_directory_migration(bad), Err(reason));
        }
    }
}
