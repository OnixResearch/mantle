use std::collections::HashSet;

use crate::model::ChunkProfile;
use crate::model::chunk_profile_v1;

pub const PROTOCOL_VERSION_V1: u32 = 1;
pub const MAX_NEGOTIATION_VERSIONS: u32 = 8;
pub const MAX_NEGOTIATION_CHUNK_PROFILES: u32 = 8;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ChunkingAlgorithmWire {
    FastCdc,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ChunkDigestAlgorithmWire {
    Blake3,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ChunkProfileWire {
    pub chunking: ChunkingAlgorithmWire,
    pub chunk_digest: ChunkDigestAlgorithmWire,
    pub min_chunk_bytes: u32,
    pub avg_chunk_bytes: u32,
    pub max_chunk_bytes: u32,
}

impl ChunkProfileWire {
    pub fn validate(self) {
        assert!(self.min_chunk_bytes > 0, "chunk profile min must be positive");
        assert!(self.min_chunk_bytes < self.avg_chunk_bytes, "chunk profile avg must exceed min");
        assert!(self.avg_chunk_bytes < self.max_chunk_bytes, "chunk profile max must exceed avg");
    }

    pub fn matches_runtime_profile(self, runtime: &ChunkProfile) -> bool {
        self.validate();
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

#[derive(Debug, Clone, PartialEq, Eq)]
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

impl std::fmt::Display for NegotiationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
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

impl std::error::Error for NegotiationError {}

pub fn negotiate_protocol(
    sender: &NegotiationOffer,
    receiver: &NegotiationOffer,
) -> Result<NegotiatedProtocol, NegotiationError> {
    validate_offer(sender)?;
    validate_offer(receiver)?;

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
    let version_count = offer.supported_versions.len() as u32;
    if version_count == 0 {
        return Err(NegotiationError::EmptyVersionSet);
    }
    if version_count > MAX_NEGOTIATION_VERSIONS {
        return Err(NegotiationError::TooManyVersions(version_count));
    }

    let profile_count = offer.supported_chunk_profiles.len() as u32;
    if profile_count == 0 {
        return Err(NegotiationError::EmptyChunkProfileSet);
    }
    if profile_count > MAX_NEGOTIATION_CHUNK_PROFILES {
        return Err(NegotiationError::TooManyChunkProfiles(profile_count));
    }

    let mut seen_versions = HashSet::<u32>::new();
    for version in &offer.supported_versions {
        if !seen_versions.insert(*version) {
            return Err(NegotiationError::DuplicateVersion(*version));
        }
    }

    let mut seen_profiles = HashSet::<ChunkProfileWire>::new();
    for profile in &offer.supported_chunk_profiles {
        profile.validate();
        if !seen_profiles.insert(*profile) {
            return Err(NegotiationError::DuplicateChunkProfile(*profile));
        }
    }

    Ok(())
}

fn select_highest_common_version(sender_versions: &[u32], receiver_versions: &[u32]) -> Result<u32, NegotiationError> {
    assert!(!sender_versions.is_empty(), "sender version list must not be empty");
    assert!(!receiver_versions.is_empty(), "receiver version list must not be empty");

    let receiver_version_set = receiver_versions.iter().copied().collect::<HashSet<_>>();
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
    let receiver_profile_set = receiver_profiles.iter().copied().collect::<HashSet<_>>();
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
    use super::*;

    #[test]
    fn protocol_v1_wire_profile_matches_spec() {
        let wire = chunk_profile_wire_v1();
        let runtime = chunk_profile_v1();

        assert_eq!(wire.chunking, ChunkingAlgorithmWire::FastCdc);
        assert_eq!(wire.chunk_digest, ChunkDigestAlgorithmWire::Blake3);
        assert!(wire.matches_runtime_profile(&runtime));
    }

    #[test]
    fn negotiation_agrees_on_protocol_version_one() {
        let sender = NegotiationOffer::protocol_v1();
        let receiver = NegotiationOffer {
            supported_versions: vec![7, PROTOCOL_VERSION_V1],
            supported_chunk_profiles: vec![chunk_profile_wire_v1()],
        };

        let negotiated = negotiate_protocol(&sender, &receiver).expect("protocol version one should negotiate");

        assert_eq!(negotiated.version, PROTOCOL_VERSION_V1);
        assert_eq!(negotiated.chunk_profile, chunk_profile_wire_v1());
    }

    #[test]
    fn negotiation_agrees_on_protocol_v1_chunk_profile() {
        let sender = NegotiationOffer::protocol_v1();
        let receiver = NegotiationOffer::protocol_v1();

        let negotiated = negotiate_protocol(&sender, &receiver).expect("protocol v1 chunk profile should negotiate");

        assert_eq!(negotiated.chunk_profile, chunk_profile_wire_v1());
        assert!(negotiated.chunk_profile.matches_runtime_profile(&chunk_profile_v1()));
    }

    #[test]
    fn negotiation_rejects_version_mismatch() {
        let sender = NegotiationOffer::protocol_v1();
        let receiver = NegotiationOffer {
            supported_versions: vec![2],
            supported_chunk_profiles: vec![chunk_profile_wire_v1()],
        };

        let err = negotiate_protocol(&sender, &receiver).expect_err("version mismatch must fail");

        assert_eq!(err, NegotiationError::VersionMismatch {
            sender_versions: vec![PROTOCOL_VERSION_V1],
            receiver_versions: vec![2],
        });
    }

    #[test]
    fn negotiation_rejects_chunk_profile_mismatch() {
        let sender = NegotiationOffer::protocol_v1();
        let receiver = NegotiationOffer {
            supported_versions: vec![PROTOCOL_VERSION_V1],
            supported_chunk_profiles: vec![ChunkProfileWire {
                chunking: ChunkingAlgorithmWire::FastCdc,
                chunk_digest: ChunkDigestAlgorithmWire::Blake3,
                min_chunk_bytes: 65_536,
                avg_chunk_bytes: 131_072,
                max_chunk_bytes: 262_144,
            }],
        };

        let err = negotiate_protocol(&sender, &receiver).expect_err("chunk profile mismatch must fail");

        assert_eq!(err, NegotiationError::ChunkProfileMismatch {
            version: PROTOCOL_VERSION_V1,
            sender_chunk_profiles: vec![chunk_profile_wire_v1()],
            receiver_chunk_profiles: receiver.supported_chunk_profiles,
        });
    }
}
