#![cfg_attr(docsrs, feature(doc_cfg))]

extern crate self as nix_compat;

/// Hashes formatted string data with BLAKE3, without an intermediate buffer.
/// Analogous to [`std::fmt::format`].
///
/// crunch uses BLAKE3 for derivation-level hashing instead of Nix's SHA-256.
/// The macro name is kept as `sha256!` to minimize churn in vendored code,
/// but the actual hash function is BLAKE3.
pub(crate) fn derivation_hash_fmt(fmt: std::fmt::Arguments<'_>) -> [u8; 32] {
    use std::fmt::Write;
    let mut s = String::new();
    write!(&mut s, "{fmt}").unwrap();
    *blake3::hash(s.as_bytes()).as_bytes()
}

/// Hashes a formatted string with BLAKE3 for derivation-level operations.
///
/// Named `sha256!` for compatibility with vendored nix-compat code.
/// The actual hash function is BLAKE3 (crunch design decision #10).
macro_rules! sha256 {
    ($($args:tt)*) => {
        $crate::derivation_hash_fmt(format_args!($($args)*))
    };
}

pub(crate) mod aterm;
pub mod derivation;
pub mod log;
pub mod nar;
pub mod narinfo;
pub mod nix_http;
pub mod nixbase32;
pub mod nixcpp;
pub mod nixhash;
pub mod path_info;
pub mod store_path;

#[cfg(feature = "wire")]
pub mod wire;

#[cfg(feature = "daemon")]
pub mod nix_daemon;
#[cfg(feature = "daemon")]
pub use nix_daemon::worker_protocol;
#[cfg(feature = "flakeref")]
pub mod flakeref;
