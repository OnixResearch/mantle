use crate::model::ChunkProfile;
use crate::model::core_chunk_profile;

pub const PROTOCOL_VERSION_V1: u32 = 1;

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
        validate_chunk_profile_wire(self);
    }

    pub fn matches_runtime_profile(self, runtime: &ChunkProfile) -> bool {
        matches_runtime_profile(self, runtime)
    }
}

pub fn chunk_profile_wire_v1() -> ChunkProfileWire {
    chunk_profile_wire_from_core(crunch_delta_core::chunk_profile_wire_v1())
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

impl From<crunch_delta_core::NegotiationError> for NegotiationError {
    fn from(value: crunch_delta_core::NegotiationError) -> Self {
        match value {
            crunch_delta_core::NegotiationError::EmptyVersionSet => Self::EmptyVersionSet,
            crunch_delta_core::NegotiationError::EmptyChunkProfileSet => Self::EmptyChunkProfileSet,
            crunch_delta_core::NegotiationError::TooManyVersions(count) => Self::TooManyVersions(count),
            crunch_delta_core::NegotiationError::TooManyChunkProfiles(count) => Self::TooManyChunkProfiles(count),
            crunch_delta_core::NegotiationError::DuplicateVersion(version) => Self::DuplicateVersion(version),
            crunch_delta_core::NegotiationError::DuplicateChunkProfile(profile) => {
                Self::DuplicateChunkProfile(chunk_profile_wire_from_core(profile))
            }
            crunch_delta_core::NegotiationError::VersionMismatch {
                sender_versions,
                receiver_versions,
            } => Self::VersionMismatch {
                sender_versions,
                receiver_versions,
            },
            crunch_delta_core::NegotiationError::ChunkProfileMismatch {
                version,
                sender_chunk_profiles,
                receiver_chunk_profiles,
            } => Self::ChunkProfileMismatch {
                version,
                sender_chunk_profiles: sender_chunk_profiles
                    .into_iter()
                    .map(chunk_profile_wire_from_core)
                    .collect::<Vec<_>>(),
                receiver_chunk_profiles: receiver_chunk_profiles
                    .into_iter()
                    .map(chunk_profile_wire_from_core)
                    .collect::<Vec<_>>(),
            },
        }
    }
}

pub(crate) fn validate_chunk_profile_wire(profile: ChunkProfileWire) {
    crunch_delta_core::validate_chunk_profile_wire(core_chunk_profile_wire(profile));
}

pub(crate) fn matches_runtime_profile(profile: ChunkProfileWire, runtime: &ChunkProfile) -> bool {
    core_chunk_profile_wire(profile).matches_runtime_profile(core_chunk_profile(runtime))
}

pub fn negotiate_protocol(
    sender: &NegotiationOffer,
    receiver: &NegotiationOffer,
) -> Result<NegotiatedProtocol, NegotiationError> {
    let negotiated =
        crunch_delta_core::negotiate_protocol(core_negotiation_offer(sender), core_negotiation_offer(receiver))
            .map_err(NegotiationError::from)?;
    Ok(negotiated_protocol_from_core(negotiated))
}

fn negotiated_protocol_from_core(protocol: crunch_delta_core::NegotiatedProtocol) -> NegotiatedProtocol {
    NegotiatedProtocol {
        version: protocol.version,
        chunk_profile: chunk_profile_wire_from_core(protocol.chunk_profile),
    }
}

fn chunk_profile_wire_from_core(profile: crunch_delta_core::ChunkProfileWire) -> ChunkProfileWire {
    ChunkProfileWire {
        chunking: chunking_algorithm_from_core(profile.chunking),
        chunk_digest: chunk_digest_algorithm_from_core(profile.chunk_digest),
        min_chunk_bytes: profile.min_chunk_bytes,
        avg_chunk_bytes: profile.avg_chunk_bytes,
        max_chunk_bytes: profile.max_chunk_bytes,
    }
}

fn core_negotiation_offer(offer: &NegotiationOffer) -> crunch_delta_core::NegotiationOffer {
    crunch_delta_core::NegotiationOffer {
        supported_versions: offer.supported_versions.clone(),
        supported_chunk_profiles: offer
            .supported_chunk_profiles
            .iter()
            .copied()
            .map(core_chunk_profile_wire)
            .collect::<Vec<_>>(),
    }
}

fn core_chunk_profile_wire(profile: ChunkProfileWire) -> crunch_delta_core::ChunkProfileWire {
    crunch_delta_core::ChunkProfileWire {
        chunking: core_chunking_algorithm(profile.chunking),
        chunk_digest: core_chunk_digest_algorithm(profile.chunk_digest),
        min_chunk_bytes: profile.min_chunk_bytes,
        avg_chunk_bytes: profile.avg_chunk_bytes,
        max_chunk_bytes: profile.max_chunk_bytes,
    }
}

fn chunking_algorithm_from_core(value: crunch_delta_core::ChunkingAlgorithmWire) -> ChunkingAlgorithmWire {
    match value {
        crunch_delta_core::ChunkingAlgorithmWire::FastCdc => ChunkingAlgorithmWire::FastCdc,
    }
}

fn chunk_digest_algorithm_from_core(value: crunch_delta_core::ChunkDigestAlgorithmWire) -> ChunkDigestAlgorithmWire {
    match value {
        crunch_delta_core::ChunkDigestAlgorithmWire::Blake3 => ChunkDigestAlgorithmWire::Blake3,
    }
}

fn core_chunking_algorithm(value: ChunkingAlgorithmWire) -> crunch_delta_core::ChunkingAlgorithmWire {
    match value {
        ChunkingAlgorithmWire::FastCdc => crunch_delta_core::ChunkingAlgorithmWire::FastCdc,
    }
}

fn core_chunk_digest_algorithm(value: ChunkDigestAlgorithmWire) -> crunch_delta_core::ChunkDigestAlgorithmWire {
    match value {
        ChunkDigestAlgorithmWire::Blake3 => crunch_delta_core::ChunkDigestAlgorithmWire::Blake3,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chunk_profile_v1;

    #[test]
    fn protocol_v1_wire_profile_matches_spec() {
        let runtime = chunk_profile_v1();
        let wire = chunk_profile_wire_v1();
        validate_chunk_profile_wire(wire);
        assert!(matches_runtime_profile(wire, &runtime));
    }

    #[test]
    fn negotiation_agrees_on_protocol_version_one() {
        let sender = NegotiationOffer::protocol_v1();
        let receiver = NegotiationOffer::protocol_v1();
        let negotiated = negotiate_protocol(&sender, &receiver).expect("protocol v1 negotiation");
        assert_eq!(negotiated.version, PROTOCOL_VERSION_V1);
        assert_eq!(negotiated.chunk_profile, chunk_profile_wire_v1());
    }

    #[test]
    fn negotiation_rejects_version_mismatch() {
        let sender = NegotiationOffer::protocol_v1();
        let receiver = NegotiationOffer {
            supported_versions: vec![PROTOCOL_VERSION_V1.saturating_add(1)],
            supported_chunk_profiles: vec![chunk_profile_wire_v1()],
        };
        let err = negotiate_protocol(&sender, &receiver).expect_err("version mismatch must fail");
        assert_eq!(err, NegotiationError::VersionMismatch {
            sender_versions: vec![PROTOCOL_VERSION_V1],
            receiver_versions: vec![PROTOCOL_VERSION_V1.saturating_add(1)],
        });
    }

    #[test]
    fn negotiation_error_keeps_std_error_in_facade() {
        fn assert_error<E: std::error::Error>() {}
        assert_error::<NegotiationError>();
    }

    #[test]
    fn validate_method_matches_free_function() {
        let mut seen = std::collections::HashSet::new();
        let wire = chunk_profile_wire_v1();
        wire.validate();
        assert!(seen.insert(wire));
        assert!(wire.matches_runtime_profile(&chunk_profile_v1()));
    }
}
