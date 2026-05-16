use alloc::collections::BTreeSet;
use alloc::vec;
use alloc::vec::Vec;
use core::fmt;

use crate::model::ChunkProfile;
use crate::model::chunk_profile_v1;

pub const PROTOCOL_VERSION_V1: u32 = 1;
pub const MAX_NEGOTIATION_VERSIONS: u32 = 8;
pub const MAX_NEGOTIATION_CHUNK_PROFILES: u32 = 8;

const _: () = assert!(MAX_NEGOTIATION_VERSIONS > 0);
const _: () = assert!(MAX_NEGOTIATION_CHUNK_PROFILES > 0);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ChunkingAlgorithmWire {
    FastCdc,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ChunkDigestAlgorithmWire {
    Blake3,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ChunkProfileWire {
    pub chunking: ChunkingAlgorithmWire,
    pub chunk_digest: ChunkDigestAlgorithmWire,
    pub min_chunk_bytes: u32,
    pub avg_chunk_bytes: u32,
    pub max_chunk_bytes: u32,
}

impl ChunkProfileWire {
    pub fn matches_runtime_profile(self, runtime: ChunkProfile) -> bool {
        validate_chunk_profile_wire(self);
        assert!(runtime.min_chunk_bytes > 0, "runtime chunk profile min must be positive");
        assert!(runtime.min_chunk_bytes < runtime.avg_chunk_bytes, "runtime chunk profile avg must exceed min");
        self.chunking == ChunkingAlgorithmWire::FastCdc
            && self.chunk_digest == ChunkDigestAlgorithmWire::Blake3
            && self.min_chunk_bytes == runtime.min_chunk_bytes
            && self.avg_chunk_bytes == runtime.avg_chunk_bytes
            && self.max_chunk_bytes == runtime.max_chunk_bytes
    }
}

pub fn chunk_profile_wire_v1() -> ChunkProfileWire {
    let runtime = chunk_profile_v1();
    ChunkProfileWire {
        chunking: ChunkingAlgorithmWire::FastCdc,
        chunk_digest: ChunkDigestAlgorithmWire::Blake3,
        min_chunk_bytes: runtime.min_chunk_bytes,
        avg_chunk_bytes: runtime.avg_chunk_bytes,
        max_chunk_bytes: runtime.max_chunk_bytes,
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NegotiationOffer {
    pub supported_versions: Vec<u32>,
    pub supported_chunk_profiles: Vec<ChunkProfileWire>,
}

impl NegotiationOffer {
    pub fn protocol_v1() -> Self {
        Self {
            supported_versions: vec![PROTOCOL_VERSION_V1],
            supported_chunk_profiles: vec![chunk_profile_wire_v1()],
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NegotiatedProtocol {
    pub version: u32,
    pub chunk_profile: ChunkProfileWire,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NegotiationError {
    EmptyVersionSet,
    EmptyChunkProfileSet,
    TooManyVersions(u32),
    TooManyChunkProfiles(u32),
    DuplicateVersion(u32),
    DuplicateChunkProfile(ChunkProfileWire),
    VersionMismatch {
        sender_versions: Vec<u32>,
        receiver_versions: Vec<u32>,
    },
    ChunkProfileMismatch {
        version: u32,
        sender_chunk_profiles: Vec<ChunkProfileWire>,
        receiver_chunk_profiles: Vec<ChunkProfileWire>,
    },
}

impl fmt::Display for NegotiationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyVersionSet => write!(f, "negotiation offer must include at least one protocol version"),
            Self::EmptyChunkProfileSet => write!(f, "negotiation offer must include at least one chunk profile"),
            Self::TooManyVersions(count) => write!(f, "negotiation offer exceeds version limit: {count}"),
            Self::TooManyChunkProfiles(count) => write!(f, "negotiation offer exceeds chunk profile limit: {count}"),
            Self::DuplicateVersion(version) => write!(f, "negotiation offer repeats protocol version {version}"),
            Self::DuplicateChunkProfile(profile) => write!(
                f,
                "negotiation offer repeats chunk profile {:?}/{:?}/{:?}/{:?}/{:?}",
                profile.chunking,
                profile.chunk_digest,
                profile.min_chunk_bytes,
                profile.avg_chunk_bytes,
                profile.max_chunk_bytes,
            ),
            Self::VersionMismatch {
                sender_versions,
                receiver_versions,
            } => write!(f, "no shared protocol version: sender={sender_versions:?} receiver={receiver_versions:?}"),
            Self::ChunkProfileMismatch {
                version,
                sender_chunk_profiles,
                receiver_chunk_profiles,
            } => write!(
                f,
                "no shared chunk profile for protocol version {version}: sender={sender_chunk_profiles:?} receiver={receiver_chunk_profiles:?}"
            ),
        }
    }
}

pub fn negotiate_protocol(
    sender: NegotiationOffer,
    receiver: NegotiationOffer,
) -> Result<NegotiatedProtocol, NegotiationError> {
    validate_offer(&sender)?;
    validate_offer(&receiver)?;

    let agreed_version = select_highest_common_version(&sender.supported_versions, &receiver.supported_versions)?;
    let agreed_profile = select_shared_chunk_profile(
        agreed_version,
        &sender.supported_chunk_profiles,
        &receiver.supported_chunk_profiles,
    )?;

    Ok(NegotiatedProtocol {
        version: agreed_version,
        chunk_profile: agreed_profile,
    })
}

fn validate_offer(offer: &NegotiationOffer) -> Result<(), NegotiationError> {
    let version_count = match u32::try_from(offer.supported_versions.len()) {
        Ok(count) => count,
        Err(_) => return Err(NegotiationError::TooManyVersions(MAX_NEGOTIATION_VERSIONS.saturating_add(1))),
    };
    if version_count == 0 {
        return Err(NegotiationError::EmptyVersionSet);
    }
    if version_count > MAX_NEGOTIATION_VERSIONS {
        return Err(NegotiationError::TooManyVersions(version_count));
    }
    debug_assert!((1..=MAX_NEGOTIATION_VERSIONS).contains(&version_count));

    let profile_count = match u32::try_from(offer.supported_chunk_profiles.len()) {
        Ok(count) => count,
        Err(_) => {
            return Err(NegotiationError::TooManyChunkProfiles(MAX_NEGOTIATION_CHUNK_PROFILES.saturating_add(1)));
        }
    };
    if profile_count == 0 {
        return Err(NegotiationError::EmptyChunkProfileSet);
    }
    if profile_count > MAX_NEGOTIATION_CHUNK_PROFILES {
        return Err(NegotiationError::TooManyChunkProfiles(profile_count));
    }
    debug_assert!((1..=MAX_NEGOTIATION_CHUNK_PROFILES).contains(&profile_count));

    let mut seen_versions = BTreeSet::<u32>::new();
    for version in &offer.supported_versions {
        if !seen_versions.insert(*version) {
            return Err(NegotiationError::DuplicateVersion(*version));
        }
    }

    let mut seen_profiles = BTreeSet::<ChunkProfileWire>::new();
    for profile in &offer.supported_chunk_profiles {
        validate_chunk_profile_wire(*profile);
        if !seen_profiles.insert(*profile) {
            return Err(NegotiationError::DuplicateChunkProfile(*profile));
        }
    }

    Ok(())
}

pub fn validate_chunk_profile_wire(profile: ChunkProfileWire) {
    assert!(profile.min_chunk_bytes > 0, "chunk profile min must be positive");
    assert!(profile.min_chunk_bytes < profile.avg_chunk_bytes, "chunk profile avg must exceed min");
    assert!(profile.avg_chunk_bytes < profile.max_chunk_bytes, "chunk profile max must exceed avg");
}

fn select_highest_common_version(sender_versions: &[u32], receiver_versions: &[u32]) -> Result<u32, NegotiationError> {
    assert!(!sender_versions.is_empty(), "sender version list must not be empty");
    assert!(!receiver_versions.is_empty(), "receiver version list must not be empty");

    let receiver_version_set = receiver_versions.iter().copied().collect::<BTreeSet<_>>();
    let mut common_versions = sender_versions
        .iter()
        .copied()
        .filter(|version| receiver_version_set.contains(version))
        .collect::<Vec<_>>();
    if common_versions.is_empty() {
        return Err(NegotiationError::VersionMismatch {
            sender_versions: sender_versions.to_vec(),
            receiver_versions: receiver_versions.to_vec(),
        });
    }

    common_versions.sort_unstable();
    let agreed_version = *common_versions.last().ok_or(NegotiationError::VersionMismatch {
        sender_versions: sender_versions.to_vec(),
        receiver_versions: receiver_versions.to_vec(),
    })?;
    assert!(agreed_version > 0, "protocol version must be positive");
    Ok(agreed_version)
}

fn select_shared_chunk_profile(
    version: u32,
    sender_profiles: &[ChunkProfileWire],
    receiver_profiles: &[ChunkProfileWire],
) -> Result<ChunkProfileWire, NegotiationError> {
    assert!(!sender_profiles.is_empty(), "sender chunk profile list must not be empty");
    assert!(!receiver_profiles.is_empty(), "receiver chunk profile list must not be empty");

    let expected_profile = chunk_profile_for_version(version)?;
    let receiver_profile_set = receiver_profiles.iter().copied().collect::<BTreeSet<_>>();
    let shared_profiles = sender_profiles
        .iter()
        .copied()
        .filter(|profile| *profile == expected_profile)
        .filter(|profile| receiver_profile_set.contains(profile))
        .collect::<Vec<_>>();
    if shared_profiles.is_empty() {
        return Err(NegotiationError::ChunkProfileMismatch {
            version,
            sender_chunk_profiles: sender_profiles.to_vec(),
            receiver_chunk_profiles: receiver_profiles.to_vec(),
        });
    }

    assert_eq!(shared_profiles[0], expected_profile, "shared chunk profile must match version pin");
    Ok(shared_profiles[0])
}

fn chunk_profile_for_version(version: u32) -> Result<ChunkProfileWire, NegotiationError> {
    if version == PROTOCOL_VERSION_V1 {
        return Ok(chunk_profile_wire_v1());
    }

    Err(NegotiationError::ChunkProfileMismatch {
        version,
        sender_chunk_profiles: Vec::new(),
        receiver_chunk_profiles: Vec::new(),
    })
}

#[cfg(test)]
mod tests {
    use alloc::vec;

    use super::*;

    #[test]
    fn chunk_profile_wire_v1_matches_runtime_profile() {
        let runtime = chunk_profile_v1();
        let wire = chunk_profile_wire_v1();
        assert!(wire.matches_runtime_profile(runtime));
        assert_eq!(wire.min_chunk_bytes, runtime.min_chunk_bytes);
    }

    #[test]
    fn duplicate_version_offer_is_rejected() {
        let duplicate_version = PROTOCOL_VERSION_V1;
        let err = negotiate_protocol(
            NegotiationOffer {
                supported_versions: vec![duplicate_version, duplicate_version],
                supported_chunk_profiles: vec![chunk_profile_wire_v1()],
            },
            NegotiationOffer::protocol_v1(),
        )
        .expect_err("duplicate versions must fail");
        assert_eq!(err, NegotiationError::DuplicateVersion(duplicate_version));
    }
}
