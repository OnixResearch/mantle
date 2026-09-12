//! Adapter: entropy-backed remote-failure replay execution identities.
//!
//! The derivation is pure over a source-bundle digest and explicit entropy
//! bytes; this adapter is the only place that acquires entropy from the host,
//! so a command path no longer reaches the operating-system random source.

use crunch_build::distributed::RemoteFailureDebugDigest;

use crate::RunError;

/// Domain separation tag for the replay execution identity.
pub(crate) const REPLAY_IDENTITY_DOMAIN: &[u8] = b"mantle-remote-failure-replay-execution-v1\0";

/// Entropy bytes drawn for one replay execution identity.
pub(crate) const REMOTE_FAILURE_REPLAY_RANDOM_BYTES: usize = 16;

/// Derive one replay execution identity from a bundle digest and entropy.
///
/// The caller supplies the entropy, so the derivation is deterministic and
/// testable. Exactly [`REMOTE_FAILURE_REPLAY_RANDOM_BYTES`] bytes are admitted.
pub(crate) fn derive_identity(
    source_bundle: &RemoteFailureDebugDigest,
    entropy: &[u8],
) -> Result<RemoteFailureDebugDigest, RunError> {
    if entropy.len() != REMOTE_FAILURE_REPLAY_RANDOM_BYTES {
        return Err(RunError::Internal(format!(
            "remote failure replay entropy must be {REMOTE_FAILURE_REPLAY_RANDOM_BYTES} bytes, got {}",
            entropy.len()
        )));
    }
    debug_assert!(!source_bundle.as_str().is_empty());
    debug_assert_eq!(entropy.len(), REMOTE_FAILURE_REPLAY_RANDOM_BYTES);
    let mut hasher = blake3::Hasher::new();
    hasher.update(REPLAY_IDENTITY_DOMAIN);
    hasher.update(source_bundle.as_str().as_bytes());
    hasher.update(entropy);
    RemoteFailureDebugDigest::new(hasher.finalize().to_hex().to_string())
        .map_err(|reason| RunError::Internal(reason.as_str().to_string()))
}

/// Derive one replay execution identity from freshly drawn host entropy.
pub(crate) fn new_identity(source_bundle: &RemoteFailureDebugDigest) -> Result<RemoteFailureDebugDigest, RunError> {
    use rand::RngCore as _;

    let mut entropy = [0_u8; REMOTE_FAILURE_REPLAY_RANDOM_BYTES];
    rand::rngs::OsRng
        .try_fill_bytes(&mut entropy)
        .map_err(|error| RunError::Internal(format!("remote failure replay randomness: {error}")))?;
    let identity = derive_identity(source_bundle, &entropy)?;
    debug_assert_eq!(identity.as_str().len(), 64);
    debug_assert_ne!(identity.as_str(), source_bundle.as_str());
    Ok(identity)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn bundle() -> RemoteFailureDebugDigest {
        RemoteFailureDebugDigest::new("a".repeat(64)).expect("sample bundle digest")
    }

    fn entropy(byte: u8) -> Vec<u8> {
        vec![byte; REMOTE_FAILURE_REPLAY_RANDOM_BYTES]
    }

    #[test]
    fn the_derivation_is_deterministic_for_fixed_entropy() {
        let first = derive_identity(&bundle(), &entropy(7)).expect("derivation succeeds");
        let second = derive_identity(&bundle(), &entropy(7)).expect("derivation succeeds");
        assert_eq!(first, second);
        assert_eq!(first.as_str().len(), 64);
        assert_ne!(first.as_str(), bundle().as_str());
    }

    #[test]
    fn different_entropy_produces_a_different_identity() {
        let first = derive_identity(&bundle(), &entropy(1)).expect("derivation succeeds");
        let second = derive_identity(&bundle(), &entropy(2)).expect("derivation succeeds");
        assert_ne!(first, second);
        assert_ne!(first.as_str(), second.as_str());
    }

    #[test]
    fn a_short_entropy_input_is_rejected() {
        let error = derive_identity(&bundle(), &entropy(3)[..1]).expect_err("short entropy is rejected");
        assert!(format!("{error}").contains("entropy must be"));
        let empty = derive_identity(&bundle(), &[]).expect_err("empty entropy is rejected");
        assert!(format!("{empty}").contains("got 0"));
    }

    #[test]
    fn a_fresh_identity_comes_from_host_entropy() {
        let first = new_identity(&bundle()).expect("identity is created");
        let second = new_identity(&bundle()).expect("identity is created");
        assert_ne!(first, second);
        assert_eq!(first.as_str().len(), 64);
    }
}
