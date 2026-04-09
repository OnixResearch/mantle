use data_encoding::HEXLOWER;
// crunch: SHA-256 removed from derivation-level hashing (BLAKE3 via sha256! macro)
use thiserror;

use crate::nixbase32;
use crate::nixhash::CAHash;
use crate::nixhash::NixHash;
use crate::store_path::Error;
use crate::store_path::STORE_DIR;
use crate::store_path::StorePath;

/// Errors that can occur when creating a content-addressed store path.
///
/// This wraps the main [crate::store_path::Error]..
#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum BuildStorePathError {
    #[error("Invalid Store Path: {0}")]
    InvalidStorePath(Error),
    /// This error occurs when we have references outside the SHA-256 +
    /// Recursive case. The restriction comes from upstream Nix. It may be
    /// lifted at some point but there isn't a pressing need to anticipate that.
    #[error("References were not supported as much as requested")]
    InvalidReference(),
}

/// compress_hash takes an arbitrarily long sequence of bytes (usually
/// a hash digest), and returns a sequence of bytes of length
/// OUTPUT_SIZE.
///
/// It's calculated by rotating through the bytes in the output buffer
/// (zero- initialized), and XOR'ing with each byte of the passed
/// input. It consumes 1 byte at a time, and XOR's it with the current
/// value in the output buffer.
///
/// This mimics equivalent functionality in C++ Nix.
pub fn compress_hash<const OUTPUT_SIZE: usize>(input: &[u8]) -> [u8; OUTPUT_SIZE] {
    let mut output = [0; OUTPUT_SIZE];

    for (ii, ch) in input.iter().enumerate() {
        output[ii % OUTPUT_SIZE] ^= ch;
    }

    output
}

/// This builds a store path, by calculating the text_hash_string of either a
/// derivation or a literal text file that may contain references.
/// If you don't want to have to pass the entire contents, you might want to use
/// [build_ca_path] instead.
pub fn build_text_path<'a, S, SP, I, C>(
    name: &'a str,
    content: C,
    references: I,
) -> Result<StorePath<SP>, BuildStorePathError>
where
    S: AsRef<str>,
    SP: AsRef<str> + std::convert::From<&'a str>,
    I: IntoIterator<Item = S>,
    C: AsRef<[u8]>,
{
    build_text_path_with_store_dir(name, content, references, STORE_DIR)
}

/// Like [build_text_path] but with a custom store dir.
pub fn build_text_path_with_store_dir<'a, S, SP, I, C>(
    name: &'a str,
    content: C,
    references: I,
    store_dir: &str,
) -> Result<StorePath<SP>, BuildStorePathError>
where
    S: AsRef<str>,
    SP: AsRef<str> + std::convert::From<&'a str>,
    I: IntoIterator<Item = S>,
    C: AsRef<[u8]>,
{
    // produce the BLAKE3 digest of the contents (crunch: was SHA-256)
    let content_digest = *blake3::hash(content.as_ref()).as_bytes();

    build_ca_path_with_store_dir(name, &CAHash::Text(content_digest), references, false, store_dir)
}

/// This builds a store path from a [CAHash] and a list of references.
pub fn build_ca_path<'a, S, SP, I>(
    name: &'a str,
    ca_hash: &CAHash,
    references: I,
    self_reference: bool,
) -> Result<StorePath<SP>, BuildStorePathError>
where
    S: AsRef<str>,
    SP: AsRef<str> + std::convert::From<&'a str>,
    I: IntoIterator<Item = S>,
{
    build_ca_path_with_store_dir(name, ca_hash, references, self_reference, STORE_DIR)
}

/// Like [build_ca_path] but with a custom store dir.
pub fn build_ca_path_with_store_dir<'a, S, SP, I>(
    name: &'a str,
    ca_hash: &CAHash,
    references: I,
    self_reference: bool,
    store_dir: &str,
) -> Result<StorePath<SP>, BuildStorePathError>
where
    S: AsRef<str>,
    SP: AsRef<str> + std::convert::From<&'a str>,
    I: IntoIterator<Item = S>,
{
    // self references are only allowed for CAHash::Nar(NixHash::Sha256(_)).
    if self_reference && matches!(ca_hash, CAHash::Nar(NixHash::Sha256(_))) {
        return Err(BuildStorePathError::InvalidReference());
    }

    /// Helper function, used for the non-sha256 [CAHash::Nar] and all [CAHash::Flat].
    fn fixed_out_digest(prefix: &str, hash: &NixHash) -> [u8; 32] {
        sha256!("{}:{}:", prefix, hash.to_nix_lowerhex_string())
    }

    let (ty, inner_digest) = match &ca_hash {
        CAHash::Text(digest) => (make_references_string("text", references, false), *digest),
        CAHash::Nar(NixHash::Sha256(digest)) => (make_references_string("source", references, self_reference), *digest),

        // for all other CAHash::Nar, another custom scheme is used.
        CAHash::Nar(hash) => {
            if references.into_iter().next().is_some() {
                return Err(BuildStorePathError::InvalidReference());
            }

            ("output:out".to_string(), fixed_out_digest("fixed:out:r", hash))
        }
        // CaHash::Flat is using something very similar, except the `r:` prefix.
        CAHash::Flat(hash) => {
            if references.into_iter().next().is_some() {
                return Err(BuildStorePathError::InvalidReference());
            }

            ("output:out".to_string(), fixed_out_digest("fixed:out", hash))
        }
    };

    build_store_path_from_fingerprint_parts_with_store_dir(&ty, &inner_digest, name, store_dir)
        .map_err(BuildStorePathError::InvalidStorePath)
}

/// This builds an input-addressed store path.
///
/// Input-addresed store paths are always derivation outputs, the "input" in question is the
/// derivation and its closure.
pub fn build_output_path<'a, SP>(
    drv_sha256: &[u8; 32],
    output_name: &str,
    output_path_name: &'a str,
) -> Result<StorePath<SP>, Error>
where
    SP: AsRef<str> + std::convert::From<&'a str>,
{
    build_output_path_with_store_dir(drv_sha256, output_name, output_path_name, STORE_DIR)
}

/// Like [build_output_path] but with a custom store dir.
pub fn build_output_path_with_store_dir<'a, SP>(
    drv_sha256: &[u8; 32],
    output_name: &str,
    output_path_name: &'a str,
    store_dir: &str,
) -> Result<StorePath<SP>, Error>
where
    SP: AsRef<str> + std::convert::From<&'a str>,
{
    build_store_path_from_fingerprint_parts_with_store_dir(
        &(String::from("output:") + output_name),
        drv_sha256,
        output_path_name,
        store_dir,
    )
}

/// This builds a store path from fingerprint parts.
/// Usually, that function is used from [build_text_path] and
/// passed a "text hash string" (starting with "text:" as fingerprint),
/// but other fingerprints starting with "output:" are also used in Derivation
/// output path calculation.
///
/// The fingerprint is hashed with sha256, and its digest is compressed to 20
/// bytes.
/// Inside a StorePath, that digest is printed nixbase32-encoded
/// (32 characters).
#[allow(dead_code)] // wrapper over _with_store_dir; kept for API parity with upstream snix
fn build_store_path_from_fingerprint_parts<'a, SP>(
    ty: &str,
    inner_digest: &[u8; 32],
    name: &'a str,
) -> Result<StorePath<SP>, Error>
where
    SP: AsRef<str> + std::convert::From<&'a str>,
{
    build_store_path_from_fingerprint_parts_with_store_dir(ty, inner_digest, name, STORE_DIR)
}

/// Like [build_store_path_from_fingerprint_parts] but with a custom store dir.
///
/// The store dir is embedded in the fingerprint hash, so changing it produces
/// different store paths. This is the mechanism for configurable store prefixes.
pub fn build_store_path_from_fingerprint_parts_with_store_dir<'a, SP>(
    ty: &str,
    inner_digest: &[u8; 32],
    name: &'a str,
    store_dir: &str,
) -> Result<StorePath<SP>, Error>
where
    SP: AsRef<str> + std::convert::From<&'a str>,
{
    let fingerprint_hash = sha256!("{ty}:sha256:{}:{store_dir}:{name}", HEXLOWER.encode(inner_digest));
    // name validation happens in here.
    StorePath::from_name_and_digest_fixed(name, compress_hash(&fingerprint_hash))
}

/// This contains the Nix logic to create "text hash strings", which are used
/// in `builtins.toFile`, as well as in Derivation Path calculation.
///
/// A text hash is calculated by concatenating the following fields, separated by a `:`:
///
///  - text
///  - references, individually joined by `:`
///  - the nix_hash_string representation of the sha256 digest of some contents
///  - the value of `storeDir`
///  - the name
fn make_references_string<S: AsRef<str>, I: IntoIterator<Item = S>>(ty: &str, references: I, self_ref: bool) -> String {
    let mut s = String::from(ty);

    for reference in references {
        s.push(':');
        s.push_str(reference.as_ref());
    }

    if self_ref {
        s.push_str(":self");
    }

    s
}

/// Nix placeholders (i.e. values returned by `builtins.placeholder`)
/// are used to populate outputs with paths that must be
/// string-replaced with the actual placeholders later, at runtime.
///
/// The actual placeholder is basically just a SHA256 hash encoded in
/// cppnix format.
pub fn hash_placeholder(name: &str) -> String {
    format!("/{}", nixbase32::encode(&sha256!("nix-output:{name}")))
}

#[cfg(test)]
mod test {
    use hex_literal::hex;

    use super::*;
    use crate::nixhash::CAHash;
    use crate::nixhash::NixHash;
    use crate::store_path::StorePathRef;

    // crunch: expected paths differ from Nix because we use BLAKE3 instead of SHA-256.
    // These values were computed by running the BLAKE3-modified code.

    #[test]
    fn build_text_path_with_zero_references() {
        let store_path: StorePathRef =
            build_text_path("foo", "bar", Vec::<String>::new()).expect("build_store_path() should succeed");

        assert_eq!(store_path.to_absolute_path().as_str(), "/nix/store/2134hymrrc6z3nm2w3mha41w883yj7pq-foo");
    }

    #[test]
    fn build_text_path_with_non_zero_references() {
        let inner: StorePathRef =
            build_text_path("foo", "bar", Vec::<String>::new()).expect("path_with_references() should succeed");
        let inner_path = inner.to_absolute_path();

        let outer: StorePathRef = build_text_path("baz", &inner_path, vec![inner_path.as_str()])
            .expect("path_with_references() should succeed");

        assert_eq!(outer.to_absolute_path().as_str(), "/nix/store/fw62ahw6h9bnfm2ygmagsc6hm602n9qd-baz");
    }

    #[test]
    fn build_sha1_path() {
        let outer: StorePathRef = build_ca_path(
            "bar",
            &CAHash::Nar(NixHash::Sha1(hex!("0beec7b5ea3f0fdbc95d0dd47f3c5bc275da8a33"))),
            Vec::<String>::new(),
            false,
        )
        .expect("path_with_references() should succeed");

        assert_eq!(outer.to_absolute_path().as_str(), "/nix/store/v20wf8r1xgdfy20jjfhswazf1l76rzhm-bar");
    }

    #[test]
    fn build_store_path_with_non_zero_references() {
        let outer: StorePathRef = build_ca_path(
            "baz",
            &CAHash::Nar(NixHash::Sha256(
                nixbase32::decode(b"1xqkzcb3909fp07qngljr4wcdnrh1gdam1m2n29i6hhrxlmkgkv1")
                    .expect("nixbase32 should decode")
                    .try_into()
                    .expect("should have right len"),
            )),
            vec!["/nix/store/dxwkwjzdaq7ka55pkk252gh32bgpmql4-foo"],
            false,
        )
        .expect("path_with_references() should succeed");

        assert_eq!(outer.to_absolute_path().as_str(), "/nix/store/57rxb32ssn02s9zp6m3n8sph18s2b95m-baz");
    }

    /// Non-default store dir produces different paths.
    #[test]
    fn build_text_path_custom_store_dir() {
        let default: StorePathRef = build_text_path("foo", "bar", Vec::<String>::new()).expect("should succeed");
        let custom: StorePathRef =
            build_text_path_with_store_dir("foo", "bar", Vec::<String>::new(), "/opt/crunch").expect("should succeed");

        assert_ne!(default, custom, "different store dir must produce different path");
    }

    /// Default store dir matches existing build_text_path.
    #[test]
    fn build_text_path_default_store_dir_matches() {
        let via_default: StorePathRef = build_text_path("foo", "bar", Vec::<String>::new()).expect("should succeed");
        let via_explicit: StorePathRef =
            build_text_path_with_store_dir("foo", "bar", Vec::<String>::new(), "/nix/store").expect("should succeed");

        assert_eq!(via_default, via_explicit);
    }

    /// Non-default store dir on build_output_path.
    #[test]
    fn build_output_path_custom_store_dir() {
        let hash = [0x42u8; 32];
        let default: StorePathRef = build_output_path(&hash, "out", "test").expect("should succeed");
        let custom: StorePathRef =
            build_output_path_with_store_dir(&hash, "out", "test", "/opt/crunch").expect("should succeed");

        assert_ne!(default, custom, "different store dir must produce different path");
    }

    /// Non-default store dir on build_ca_path.
    #[test]
    fn build_ca_path_custom_store_dir() {
        let ca = CAHash::Nar(NixHash::Sha256([0x42u8; 32]));
        let default: StorePathRef = build_ca_path("test", &ca, Vec::<String>::new(), false).expect("should succeed");
        let custom: StorePathRef =
            build_ca_path_with_store_dir("test", &ca, Vec::<String>::new(), false, "/opt/crunch")
                .expect("should succeed");

        assert_ne!(default, custom, "different store dir must produce different path");
    }

    /// Verify BLAKE3 path computation is deterministic: same inputs → same path.
    #[test]
    fn blake3_determinism() {
        let path1: StorePathRef = build_text_path("test", "content", Vec::<String>::new()).expect("should succeed");
        let path2: StorePathRef = build_text_path("test", "content", Vec::<String>::new()).expect("should succeed");
        assert_eq!(path1, path2);
    }

    /// Verify different content produces different paths.
    #[test]
    fn blake3_different_content_different_path() {
        let path1: StorePathRef = build_text_path("test", "content-a", Vec::<String>::new()).expect("should succeed");
        let path2: StorePathRef = build_text_path("test", "content-b", Vec::<String>::new()).expect("should succeed");
        assert_ne!(path1, path2);
    }
}
