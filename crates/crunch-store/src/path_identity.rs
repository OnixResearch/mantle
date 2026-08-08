use nix_compat::store_path::DIGEST_SIZE;
use nix_compat::store_path::StorePath;
use nix_compat::store_path::build_ca_path_with_store_dir;
use snix_store::path_info::PathInfo;

const REMOTE_PATHINFO_IDENTITY_MISMATCH: &str = "remote-pathinfo-request-identity-mismatch";

/// Require returned remote metadata to carry the requested store-path digest.
// r[impl cache_substitution.requested_path_identity]
pub(crate) fn require_requested_path_identity(
    requested_digest: [u8; DIGEST_SIZE],
    requested_path: &StorePath<String>,
    observed: &PathInfo,
) -> Result<(), String> {
    assert_eq!(requested_digest, *requested_path.digest(), "requested digest must match the requested store path");
    assert!(!requested_path.name().is_empty(), "requested store path name must not be empty");
    if requested_digest == *observed.store_path.digest() {
        return Ok(());
    }
    Err(format!(
        "{REMOTE_PATHINFO_IDENTITY_MISMATCH}: requested {requested_path}, observed {}",
        observed.store_path
    ))
}

/// Require any declared CA metadata to derive the signed logical store path.
pub(crate) fn require_ca_path_identity(path_info: &PathInfo, store_dir: &str) -> Result<(), String> {
    assert!(!path_info.store_path.name().is_empty(), "store path name must not be empty");
    assert!(store_dir.starts_with('/'), "logical store prefix must be absolute");
    if ca_path_identity_matches(path_info, store_dir)? {
        return Ok(());
    }
    Err(format!("CA metadata for {} does not derive its signed store-path identity", path_info.store_path))
}

fn ca_path_identity_matches(path_info: &PathInfo, store_dir: &str) -> Result<bool, String> {
    assert!(!path_info.store_path.name().is_empty(), "store path name must not be empty");
    assert!(store_dir.starts_with('/'), "logical store prefix must be absolute");
    let Some(ca_hash) = path_info.ca.as_ref() else {
        return Ok(true);
    };
    let marker_path: StorePath<String> =
        build_ca_path_with_store_dir(path_info.store_path.name(), ca_hash, Vec::<String>::new(), false, store_dir)
            .map_err(|error| format!("deriving marker-normalized CA path for {}: {error}", path_info.store_path))?;
    if marker_path == path_info.store_path {
        return Ok(true);
    }

    let is_self_reference = path_info.references.iter().any(|reference| reference == &path_info.store_path);
    let references = path_info
        .references
        .iter()
        .filter(|reference| *reference != &path_info.store_path)
        .map(ToString::to_string)
        .collect::<Vec<_>>();
    let standard_path: Result<StorePath<String>, _> =
        build_ca_path_with_store_dir(path_info.store_path.name(), ca_hash, references, is_self_reference, store_dir);
    Ok(standard_path.is_ok_and(|candidate| candidate == path_info.store_path))
}

#[cfg(test)]
mod tests {
    use nix_compat::nixhash::CAHash;
    use nix_compat::nixhash::NixHash;
    use snix_castore::Node;
    use snix_castore::SymlinkTarget;

    use super::*;

    const SHA256_DIGEST_BYTES: usize = 32;
    const CA_HASH_BYTE: u8 = 0xA5;
    const WRONG_CA_HASH_BYTE: u8 = 0x5A;
    const STORE_DIR: &str = "/mantle/store";

    fn path_info(ca_hash: CAHash) -> PathInfo {
        let store_path =
            build_ca_path_with_store_dir("identity-test", &ca_hash, Vec::<String>::new(), false, STORE_DIR).unwrap();
        PathInfo {
            store_path,
            node: Node::Symlink {
                target: SymlinkTarget::try_from("target").unwrap(),
            },
            references: Vec::new(),
            nar_size: 1,
            nar_sha256: [1; SHA256_DIGEST_BYTES],
            signatures: Vec::new(),
            deriver: None,
            ca: Some(ca_hash),
        }
    }

    #[test]
    fn matching_requested_path_digest_is_accepted() {
        // r[verify cache_substitution.requested_path_identity]
        let requested_path = StorePath::from_name_and_digest_fixed("requested", [CA_HASH_BYTE; DIGEST_SIZE])
            .expect("valid requested store path");
        let observed_path = StorePath::from_name_and_digest_fixed("observed", [CA_HASH_BYTE; DIGEST_SIZE])
            .expect("valid observed store path");
        let observed = PathInfo {
            store_path: observed_path,
            node: Node::Symlink {
                target: SymlinkTarget::try_from("target").unwrap(),
            },
            references: Vec::new(),
            nar_size: 1,
            nar_sha256: [1; SHA256_DIGEST_BYTES],
            signatures: Vec::new(),
            deriver: None,
            ca: None,
        };

        assert!(require_requested_path_identity(*requested_path.digest(), &requested_path, &observed).is_ok());
        assert_eq!(requested_path.digest(), observed.store_path.digest());
    }

    #[test]
    fn different_requested_path_digest_is_rejected() {
        // r[verify cache_substitution.requested_path_identity]
        let requested_path = StorePath::from_name_and_digest_fixed("requested", [CA_HASH_BYTE; DIGEST_SIZE])
            .expect("valid requested store path");
        let observed_path = StorePath::from_name_and_digest_fixed("observed", [WRONG_CA_HASH_BYTE; DIGEST_SIZE])
            .expect("valid observed store path");
        let observed = PathInfo {
            store_path: observed_path,
            node: Node::Symlink {
                target: SymlinkTarget::try_from("target").unwrap(),
            },
            references: Vec::new(),
            nar_size: 1,
            nar_sha256: [1; SHA256_DIGEST_BYTES],
            signatures: Vec::new(),
            deriver: None,
            ca: None,
        };

        let error = require_requested_path_identity(*requested_path.digest(), &requested_path, &observed).unwrap_err();
        assert_eq!(
            error,
            format!(
                "{REMOTE_PATHINFO_IDENTITY_MISMATCH}: requested {requested_path}, observed {}",
                observed.store_path
            )
        );
        assert_ne!(requested_path.digest(), observed.store_path.digest());
    }

    #[test]
    fn marker_normalized_ca_path_is_accepted() {
        let ca_hash = CAHash::Nar(NixHash::Sha256([CA_HASH_BYTE; SHA256_DIGEST_BYTES]));
        let path_info = path_info(ca_hash);

        assert!(require_ca_path_identity(&path_info, STORE_DIR).is_ok());
        assert_eq!(path_info.store_path.digest().len(), DIGEST_SIZE);
    }

    #[test]
    fn mismatched_ca_path_is_rejected() {
        let ca_hash = CAHash::Nar(NixHash::Sha256([CA_HASH_BYTE; SHA256_DIGEST_BYTES]));
        let mut path_info = path_info(ca_hash);
        path_info.ca = Some(CAHash::Nar(NixHash::Sha256([WRONG_CA_HASH_BYTE; SHA256_DIGEST_BYTES])));

        let error = require_ca_path_identity(&path_info, STORE_DIR).unwrap_err();
        assert!(error.contains("does not derive"));
        assert!(error.contains(path_info.store_path.name()));
    }
}
