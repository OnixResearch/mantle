//! Declared Android native outputs consumed by either APK authority domain.
//! Compilation remains the generic Cargo unit-plan producer's responsibility.
use alloc::string::String;
use alloc::vec::Vec;

use serde::Deserialize;
use serde::Serialize;

pub const ABI: &str = "arm64-v8a";
pub const RUST_TARGET: &str = "aarch64-linux-android";
pub const NDK_REVISION: &str = "28.2.13676358";
pub const MIN_API: u32 = 24;
pub const MAX_API: u32 = 35;
pub const PAGE_BYTES: u64 = 16_384;
pub const MAX_LIBRARIES: usize = 128;
pub const MAX_ASSETS: usize = 4096;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Asset {
    pub source_member: String,
    /// Relative to APK assets/, not a host path.
    pub destination: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Library {
    /// A declared, independently admitted producer output member.
    pub store_file: String,
    pub filename: String,
    pub required_jni_exports: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Package {
    pub abi: String,
    pub api_level: u32,
    pub ndk_revision: String,
    /// Exact admitted NDK sysroot. API imports are checked against its stubs,
    /// not an assumed list of platform libraries or a version string.
    pub ndk_sysroot: String,
    pub libraries: Vec<Library>,
}

pub fn member(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 1024
        && value.split('/').all(|part| {
            !part.is_empty()
                && part != "."
                && part != ".."
                && part.bytes().all(|b| b.is_ascii_alphanumeric() || matches!(b, b'.' | b'_' | b'-' | b'+'))
        })
}

pub fn validate_assets(assets: &[Asset]) -> Result<(), &'static str> {
    if assets.len() > MAX_ASSETS {
        return Err("too many Android assets");
    }
    for (index, asset) in assets.iter().enumerate() {
        if !member(&asset.source_member) || !member(&asset.destination) {
            return Err("unsafe Android asset member");
        }
        if assets[..index].iter().any(|prior| {
            prior.destination == asset.destination
                || prior.destination.strip_prefix(&asset.destination).is_some_and(|suffix| suffix.starts_with('/'))
                || asset.destination.strip_prefix(&prior.destination).is_some_and(|suffix| suffix.starts_with('/'))
        }) {
            return Err("colliding Android asset destinations");
        }
    }
    Ok(())
}

pub fn validate(package: &Package) -> Result<(), &'static str> {
    if package.abi != ABI {
        return Err("unsupported Android native ABI");
    }
    if !(MIN_API..=MAX_API).contains(&package.api_level) {
        return Err("unsupported Android native API level");
    }
    if package.ndk_revision != NDK_REVISION {
        return Err("unsupported Android NDK revision");
    }
    if package.libraries.is_empty() || package.libraries.len() > MAX_LIBRARIES {
        return Err("Android native library count must be 1..128");
    }
    if !super::safe_store_input(&package.ndk_sysroot) {
        return Err("NDK sysroot must be a declared store input");
    }
    for (index, library) in package.libraries.iter().enumerate() {
        let is_store_input = super::safe_store_input(&library.store_file);
        let is_member = member(&library.filename);
        let has_separator = library.filename.contains('/');
        let has_lib_prefix = library.filename.starts_with("lib");
        let has_so_suffix = library.filename.ends_with(".so");
        let is_valid_output = is_store_input && is_member && !has_separator && has_lib_prefix && has_so_suffix;
        if !is_valid_output {
            return Err("invalid Android native output");
        }
        if package.libraries[..index].iter().any(|prior| prior.filename == library.filename) {
            return Err("duplicate Android native output");
        }
        if library.required_jni_exports.len() > 256
            || library.required_jni_exports.iter().any(|symbol| {
                symbol.len() > 512
                    || !(symbol.starts_with("Java_") || symbol == "JNI_OnLoad")
                    || !symbol.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'_')
            })
        {
            return Err("invalid required JNI export");
        }
    }
    debug_assert!(package.libraries.len() <= MAX_LIBRARIES, "the library count stayed bounded");
    debug_assert!(package.abi == ABI, "accepted packages use the declared ABI");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn package() -> Package {
        Package {
            abi: ABI.into(),
            api_level: 24,
            ndk_revision: NDK_REVISION.into(),
            ndk_sysroot: "/mantle/store/0123456789abcdfghijklmnpqrsvwxyz-ndk/sysroot".into(),
            libraries: alloc::vec![Library {
                store_file: "/mantle/store/1123456789abcdfghijklmnpqrsvwxyz-native/lib/libdemo.so".into(),
                filename: "libdemo.so".into(),
                required_jni_exports: alloc::vec!["JNI_OnLoad".into()],
            }],
        }
    }

    #[test]
    fn rejects_abi_api_and_ndk_drift_before_execution() {
        let mut plan = package();
        assert_eq!(validate(&plan), Ok(()));
        plan.abi = "x86_64".into();
        assert_eq!(validate(&plan), Err("unsupported Android native ABI"));
        let mut plan = package();
        for api in [0, 23, 36, u32::MAX] {
            plan.api_level = api;
            assert_eq!(validate(&plan), Err("unsupported Android native API level"));
        }
        let mut plan = package();
        plan.ndk_revision = "28".into();
        assert_eq!(validate(&plan), Err("unsupported Android NDK revision"));
    }

    #[test]
    fn rejects_host_paths_duplicate_libraries_and_invalid_jni_names() {
        let mut plan = package();
        plan.libraries[0].store_file = "/home/user/libdemo.so".into();
        assert_eq!(validate(&plan), Err("invalid Android native output"));
        let mut plan = package();
        plan.libraries.push(plan.libraries[0].clone());
        assert_eq!(validate(&plan), Err("duplicate Android native output"));
        let mut plan = package();
        plan.libraries[0].required_jni_exports = alloc::vec!["main".into()];
        assert_eq!(validate(&plan), Err("invalid required JNI export"));
    }

    #[test]
    fn assets_reject_traversal_and_file_directory_collisions() {
        let asset = |destination: &str| Asset {
            source_member: "data/message.txt".into(),
            destination: destination.into(),
        };
        assert_eq!(validate_assets(&[asset("message.txt")]), Ok(()));
        assert_eq!(validate_assets(&[asset("../message.txt")]), Err("unsafe Android asset member"));
        assert_eq!(
            validate_assets(&[asset("data"), asset("data/message.txt")]),
            Err("colliding Android asset destinations")
        );
        assert_eq!(
            validate_assets(&[asset("message.txt"), asset("message.txt")]),
            Err("colliding Android asset destinations")
        );
    }
}
