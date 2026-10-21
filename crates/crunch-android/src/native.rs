//! Native input inspection shared by the published and reviewed APK routes.
//! Store signatures and execution provenance remain the existing builder's job.
use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::io::Read;
use std::path::Path;

use crunch_android_core::elf;
use crunch_android_core::native::Asset;
use crunch_android_core::native::Package;

use crate::Error;
use crate::Rejection;
use crate::declared_member;
use crate::declared_regular_file;
use crate::store_object;

fn invalid(message: impl Into<String>) -> Error {
    Error::InvalidInput(message.into())
}

pub(crate) fn reject_signing_overlap<'a>(
    package: Option<&Package>,
    files: impl IntoIterator<Item = &'a Path>,
    prefix: &Path,
) -> Result<(), Error> {
    let Some(package) = package else {
        return Ok(());
    };
    let signing_roots = files.into_iter().map(|file| store_object(file, prefix)).collect::<Result<BTreeSet<_>, _>>()?;
    for file in std::iter::once(package.ndk_sysroot.as_str())
        .chain(package.libraries.iter().map(|library| library.store_file.as_str()))
    {
        if signing_roots.contains(&store_object(Path::new(file), prefix)?) {
            return Err(invalid("native input shares a signing secret store object"));
        }
    }
    Ok(())
}

fn read_elf(path: &Path, prefix: &Path, aligned: bool) -> Result<elf::Elf, Error> {
    let file = declared_regular_file(path, prefix)?;
    let size = file.metadata()?.len();
    if size > elf::MAX_ELF_BYTES as u64 {
        return Err(invalid("oversized native ELF"));
    }
    let mut data = Vec::with_capacity(size as usize);
    file.take(elf::MAX_ELF_BYTES as u64 + 1).read_to_end(&mut data)?;
    elf::inspect(&data, aligned).map_err(|message| invalid(format!("{}: {message}", path.display())))
}

pub fn inspect_inputs(source: &Path, assets: &[Asset], package: Option<&Package>, prefix: &Path) -> Result<(), Error> {
    crunch_android_core::native::validate_assets(assets).map_err(invalid)?;
    for asset in assets {
        declared_member(source, &asset.source_member, Rejection::InvalidAssetInput)?;
    }
    let Some(package) = package else {
        return Ok(());
    };
    crunch_android_core::native::validate(package).map_err(invalid)?;
    store_object(Path::new(&package.ndk_sysroot), prefix)?;
    let mut closure = BTreeMap::new();
    for library in &package.libraries {
        let parsed = read_elf(Path::new(&library.store_file), prefix, true)?;
        if parsed.android_api != Some(package.api_level) {
            return Err(invalid(format!(
                "Android ELF API note {:?} differs from declared API {}: {}",
                parsed.android_api, package.api_level, library.filename
            )));
        }
        if parsed.soname.as_ref().is_some_and(|name| name != &library.filename) {
            return Err(invalid(format!("ELF SONAME differs from declared APK name: {}", library.filename)));
        }
        for symbol in &library.required_jni_exports {
            if !parsed.exports.contains(symbol) {
                return Err(invalid(format!("missing JNI export: {symbol}")));
            }
        }
        closure.insert(library.filename.clone(), parsed);
    }
    let mut pending: BTreeSet<_> = closure.values().flat_map(|elf| elf.needed.iter().cloned()).collect();
    while let Some(name) = pending.pop_first() {
        if closure.contains_key(&name) {
            continue;
        }
        if closure.len() >= 256 {
            return Err(invalid("Android dependency closure exceeds 256 libraries"));
        }
        if !crunch_android_core::native::member(&name) || name.contains('/') {
            return Err(invalid("unsafe ELF dependency name"));
        }
        let stub = Path::new(&package.ndk_sysroot)
            .join("usr/lib/aarch64-linux-android")
            .join(package.api_level.to_string())
            .join(&name);
        let parsed = read_elf(&stub, prefix, false).map_err(|error| {
            invalid(format!("undeclared dependency or absent API {} stub {name}: {error:?}", package.api_level))
        })?;
        pending.extend(parsed.needed.iter().cloned());
        closure.insert(name, parsed);
    }
    // A sibling APK library is not loaded merely because it was packaged.
    // Resolve each root's DT_NEEDED graph, not the union of all APK exports.
    for library in &package.libraries {
        let mut exports = BTreeSet::new();
        let mut pending = BTreeSet::from([library.filename.as_str()]);
        let mut visited = BTreeSet::new();
        while let Some(name) = pending.pop_first() {
            if !visited.insert(name) {
                continue;
            }
            let parsed = &closure[name];
            exports.extend(parsed.exports.iter().map(String::as_str));
            pending.extend(parsed.needed.iter().map(String::as_str));
        }
        for symbol in &closure[&library.filename].imports {
            if !exports.contains(symbol.as_str()) {
                return Err(invalid(format!(
                    "unresolved Android API {} import {}: {symbol}",
                    package.api_level, library.filename
                )));
            }
        }
    }
    Ok(())
}

/// Copy only declared members. Caller has validated all strings and mapped
/// library paths to complete signed store inputs before rendering.
pub(crate) fn copy_script(
    source: &str,
    assets: &[Asset],
    package: Option<&Package>,
    utility: &str,
    destination: &str,
) -> String {
    let mut script = String::new();
    for asset in assets {
        let target = format!("{destination}/assets/{}", asset.destination);
        let parent = target.rsplit_once('/').expect("asset target has parent").0;
        script.push_str(&format!(
            "{utility} mkdir -p \"{parent}\"\n{utility} cp '{source}/{}' \"{target}\"\n",
            asset.source_member
        ));
    }
    if let Some(package) = package {
        script.push_str(&format!("{utility} mkdir -p \"{destination}/lib/{}\"\n", package.abi));
        for library in &package.libraries {
            script.push_str(&format!(
                "{utility} cp '{}' \"{destination}/lib/{}/{}\"\n",
                library.store_file, package.abi, library.filename
            ));
        }
    }
    script
}

#[cfg(test)]
mod tests {
    use crunch_android_core::native::ABI;
    use crunch_android_core::native::Library;
    use crunch_android_core::native::NDK_REVISION;

    use super::*;

    #[test]
    fn native_and_signing_files_cannot_share_an_object_even_at_different_members() {
        let prefix = Path::new("/mantle/store");
        let mut package = Package {
            abi: ABI.into(),
            api_level: 24,
            ndk_revision: NDK_REVISION.into(),
            ndk_sysroot: "/mantle/store/00000000000000000000000000000000-ndk/sysroot".into(),
            libraries: vec![Library {
                store_file: "/mantle/store/11111111111111111111111111111111-native/lib/libapp.so".into(),
                filename: "libapp.so".into(),
                required_jni_exports: vec!["JNI_OnLoad".into()],
            }],
        };
        let secret = Path::new("/mantle/store/22222222222222222222222222222222-secrets/private/key.p12");
        assert!(reject_signing_overlap(Some(&package), [secret], prefix).is_ok());
        package.libraries[0].store_file = "/mantle/store/22222222222222222222222222222222-secrets/lib/libapp.so".into();
        assert!(
            matches!(reject_signing_overlap(Some(&package), [secret], prefix), Err(Error::InvalidInput(message)) if message.contains("signing secret"))
        );
        package.libraries[0].store_file = "/mantle/store/11111111111111111111111111111111-native/lib/libapp.so".into();
        package.ndk_sysroot = "/mantle/store/22222222222222222222222222222222-secrets/sysroot".into();
        assert!(
            matches!(reject_signing_overlap(Some(&package), [secret], prefix), Err(Error::InvalidInput(message)) if message.contains("signing secret"))
        );
    }
}
