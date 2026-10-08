//! Admit exactly the app, full glibc, full libgcc, and test signing trees for one signed APK.
//! This command never fetches tools, generates keys, or claims that an APK exists.

use std::error::Error;
use std::fs;
use std::path::Component;
use std::path::Path;
use std::path::PathBuf;

use clap::Args;
use clap::Parser;
use clap::Subcommand;
use crunch_android::ApkInputs;
use crunch_android_core::ApkPlan;
use crunch_android_core::validate_and_lower;
use crunch_nar::CaseHackPolicy;
use crunch_nar::FilesystemNarRequest;
use crunch_nar::observe_path_blocking;
use crunch_store::StoreBackend;
use crunch_store::StoreConfig;
use crunch_store::StoreHandle;
use crunch_store::VerifiedSourceIngestRequest;
use nix_compat::narinfo::VerifyingKey;
use nix_compat::nixhash::HashAlgo;
use nix_compat::store_path::StorePath;
use serde::Deserialize;
use serde::Serialize;

const NAR_BYTES_MAX: u64 = 1_073_741_824;
const INPUT_COUNT: usize = 4;
const NON_CLAIM: &str =
    "test-only signer and prebuilt inputs; no APK, SDK provenance, source-built toolchain, or release claim";

type AdmissionResult<T> = Result<T, Box<dyn Error>>;

#[derive(Parser)]
#[command(
    name = "crunch-android-admit",
    about = "Observe or admit exactly four signed APK input trees"
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Observe actual local source NARs without creating store state or readiness.
    Observe(Common),
    /// Require independently reviewed NAR facts, then sign and verify real store objects.
    Admit(AdmitArgs),
}

#[derive(Args, Clone)]
struct Common {
    #[arg(long)]
    plan: PathBuf,
    #[arg(long)]
    inputs: PathBuf,
    /// Exactly four local source directories and their declared logical paths.
    #[arg(long)]
    manifest: PathBuf,
    #[arg(long)]
    store_prefix: String,
}

#[derive(Args, Clone)]
struct AdmitArgs {
    #[command(flatten)]
    common: Common,
    /// Writable physical store root, distinct for independent A/B replays.
    #[arg(long)]
    store: PathBuf,
    /// Writable Snix state directory, distinct for independent A/B replays.
    #[arg(long)]
    state_dir: PathBuf,
    /// Existing Mantle Nix-format Ed25519 private signing key, never generated here.
    #[arg(long)]
    signing_key: PathBuf,
    /// Exact corresponding trusted Mantle Nix-format public key.
    #[arg(long)]
    trusted_public_key: PathBuf,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, Eq, Ord, PartialEq, PartialOrd)]
#[serde(rename_all = "kebab-case")]
enum Role {
    AppSource,
    NativeGlibc,
    NativeLibgcc,
    TestSigning,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Manifest {
    entries: [LocalInput; INPUT_COUNT],
}

#[derive(Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct LocalInput {
    role: Role,
    source_path: PathBuf,
    store_path: String,
    expected_nar_sha256: Option<String>,
    expected_nar_blake3: Option<String>,
    expected_nar_size: Option<u64>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
struct NarFacts {
    nar_sha256: String,
    nar_blake3: String,
    nar_size: u64,
}

#[derive(Serialize)]
struct InputReceipt {
    role: Role,
    store_path: String,
    #[serde(flatten)]
    nar: NarFacts,
    signer: Option<String>,
    physical_path: Option<PathBuf>,
}

#[derive(Serialize)]
struct AdmissionReceipt {
    schema: &'static str,
    ready_class: &'static str,
    records: Vec<InputReceipt>,
    non_claim: &'static str,
}

fn local_member<'a>(full: &'a str, root: &str) -> AdmissionResult<&'a str> {
    let relative = full
        .strip_prefix(root)
        .and_then(|rest| rest.strip_prefix('/'))
        .ok_or("APK member is outside its declared logical source object")?;
    if relative.is_empty() || !Path::new(relative).components().all(|part| matches!(part, Component::Normal(_))) {
        return Err(format!("APK member is not a normalized path within {root}: {full}").into());
    }
    Ok(relative)
}

fn check_local_member(
    root: &Path,
    relative: &str,
    file_required: bool,
    allow_relative_symlinks: bool,
) -> AdmissionResult<()> {
    let canonical_root = fs::canonicalize(root)?;
    let mut member = canonical_root.clone();
    for part in Path::new(relative).components() {
        let Component::Normal(name) = part else {
            return Err("APK local member path is not normalized".into());
        };
        member.push(name);
        let metadata = fs::symlink_metadata(&member)?;
        if metadata.file_type().is_symlink() {
            let target = fs::read_link(&member)?;
            if !allow_relative_symlinks || target.is_absolute() {
                return Err(format!("APK local member follows forbidden symlink: {}", member.display()).into());
            }
        }
    }
    let resolved = fs::canonicalize(&member)?;
    if !resolved.starts_with(&canonical_root) {
        return Err(format!("APK local member escapes its source object: {}", member.display()).into());
    }
    let metadata = fs::metadata(&member)?;
    if file_required && !metadata.is_file() {
        return Err(format!("APK local member is not a file: {}", member.display()).into());
    }
    if !file_required && !metadata.is_file() && !metadata.is_dir() {
        return Err(format!("APK local runtime member is not a file or directory: {}", member.display()).into());
    }
    Ok(())
}

fn validate_native_runtime(inputs: &ApkInputs, glibc: &LocalInput, libgcc: &LocalInput) -> AdmissionResult<()> {
    let loader = local_member(&inputs.runtime_loader, &glibc.store_path)?;
    check_local_member(&glibc.source_path, loader, true, true)?;
    if inputs.runtime_library_paths.len() != 2 {
        return Err("APK runtime requires exactly one glibc and one libgcc library directory".into());
    }
    for (entry, required_library) in [(glibc, "libc.so.6"), (libgcc, "libgcc_s.so.1")] {
        let mut matches =
            inputs.runtime_library_paths.iter().filter_map(|path| local_member(path, &entry.store_path).ok());
        let relative = matches.next().ok_or("APK runtime missing declared library directory")?;
        if matches.next().is_some() {
            return Err(format!("APK runtime library directory is ambiguous under {}", entry.store_path).into());
        }
        check_local_member(&entry.source_path, relative, false, true)?;
        if !fs::metadata(entry.source_path.join(relative))?.is_dir() {
            return Err(format!("APK runtime library path is not a directory: {relative}").into());
        }
        check_local_member(&entry.source_path, &format!("{relative}/{required_library}"), true, true)?;
    }
    Ok(())
}

fn validate_bindings(
    plan: &ApkPlan,
    inputs: &ApkInputs,
    manifest: Manifest,
    prefix: &str,
) -> AdmissionResult<Vec<LocalInput>> {
    validate_and_lower(plan).map_err(|error| format!("invalid APK plan: {error:?}"))?;
    let signing = plan.signing.as_ref().ok_or("Android local admission requires the signed test plan")?;
    let mut entries = manifest.entries.into_iter().collect::<Vec<_>>();
    entries.sort_by_key(|entry| entry.role);
    let [app, glibc, libgcc, signing_root] = entries.as_slice() else {
        unreachable!("four required roles")
    };
    if [app.role, glibc.role, libgcc.role, signing_root.role]
        != [
            Role::AppSource,
            Role::NativeGlibc,
            Role::NativeLibgcc,
            Role::TestSigning,
        ]
    {
        return Err("expected exactly one app-source, native-glibc, native-libgcc and test-signing input".into());
    }
    let mut physical_roots: [Option<PathBuf>; INPUT_COUNT] = std::array::from_fn(|_| None);
    for (index, entry) in entries.iter().enumerate() {
        let parsed: StorePath<String> = StorePath::from_absolute_path_with_prefix(entry.store_path.as_bytes(), prefix)
            .map_err(|error| format!("invalid APK logical source path {}: {error}", entry.store_path))?;
        if parsed.to_absolute_path_with_prefix(prefix) != entry.store_path
            || entries[..index].iter().any(|prior| prior.store_path == entry.store_path)
        {
            return Err("APK source bindings require four distinct complete store objects".into());
        }
        if !entry.source_path.is_absolute() {
            return Err("APK local source paths must be absolute".into());
        }
        let metadata = fs::symlink_metadata(&entry.source_path)
            .map_err(|error| format!("missing APK local source {}: {error}", entry.source_path.display()))?;
        if !metadata.is_dir() || metadata.file_type().is_symlink() {
            return Err(format!("APK source root must be a real directory: {}", entry.source_path.display()).into());
        }
        let physical = fs::canonicalize(&entry.source_path)?;
        if physical_roots[..index].iter().flatten().any(|prior| prior == &physical) {
            return Err("APK source bindings require four distinct physical source trees".into());
        }
        physical_roots[index] = Some(physical);
    }
    if app.store_path != inputs.source_root || libgcc.store_path != inputs.native_libgcc_root {
        return Err("APK app or native-libgcc root differs from declared inputs".into());
    }
    for member in std::iter::once(&plan.manifest).chain(&plan.resources).chain(&plan.java_sources) {
        let full_member = format!("{}/{}", app.store_path, member);
        let member = local_member(&full_member, &app.store_path)?;
        check_local_member(&app.source_path, member, true, true)?;
    }
    validate_native_runtime(inputs, glibc, libgcc)?;
    for member in [
        &signing.keystore,
        &signing.store_password_file,
        &signing.key_password_file,
    ] {
        let member = local_member(member, &signing_root.store_path)?;
        check_local_member(&signing_root.source_path, member, true, false)?;
    }
    Ok(entries)
}

async fn observe_nar(path: &Path) -> AdmissionResult<NarFacts> {
    let metadata = fs::symlink_metadata(path)?;
    if metadata.file_type().is_symlink() {
        return Err(format!("refusing symlink NAR root: {}", path.display()).into());
    }
    let sha = observe_path_blocking(
        path.to_path_buf(),
        FilesystemNarRequest::new(HashAlgo::Sha256, CaseHackPolicy::native(), NAR_BYTES_MAX),
    )
    .await?;
    let blake = observe_path_blocking(
        path.to_path_buf(),
        FilesystemNarRequest::new(HashAlgo::Blake3, CaseHackPolicy::native(), NAR_BYTES_MAX),
    )
    .await?;
    let confirmed = observe_path_blocking(
        path.to_path_buf(),
        FilesystemNarRequest::new(HashAlgo::Sha256, CaseHackPolicy::native(), NAR_BYTES_MAX),
    )
    .await?;
    if sha.nar_size != blake.nar_size
        || sha.nar_size != confirmed.nar_size
        || sha.digest.digest_as_bytes() != confirmed.digest.digest_as_bytes()
    {
        return Err("APK source changed during NAR observations".into());
    }
    Ok(NarFacts {
        nar_sha256: data_encoding::HEXLOWER.encode(sha.digest.digest_as_bytes()),
        nar_blake3: data_encoding::HEXLOWER.encode(blake.digest.digest_as_bytes()),
        nar_size: sha.nar_size,
    })
}

fn check_expected(entry: &LocalInput, observed: &NarFacts) -> AdmissionResult<()> {
    if entry.expected_nar_sha256.as_deref() != Some(observed.nar_sha256.as_str())
        || entry.expected_nar_blake3.as_deref() != Some(observed.nar_blake3.as_str())
        || entry.expected_nar_size != Some(observed.nar_size)
    {
        return Err(format!("APK {:?} expected NAR facts missing or different from source bytes", entry.role).into());
    }
    Ok(())
}

fn check_signed(
    info: &snix_store::pathinfoservice::PathInfo,
    logical: &str,
    prefix: &str,
    trusted: &VerifyingKey,
) -> AdmissionResult<()> {
    if info.store_path.to_absolute_path_with_prefix(prefix) != logical {
        return Err(format!("APK source PathInfo has a different logical path: {logical}").into());
    }
    if info.ca.is_some() {
        return Err(format!("APK local source must have signer-bound, non-CA PathInfo: {logical}").into());
    }
    let verified =
        crunch_build::signing::verify_pathinfo_signatures_with_store_dir(info, std::slice::from_ref(trusted), prefix);
    if verified.trusted_count < 1 {
        return Err(format!("APK source signer is not trusted: {logical}").into());
    }
    Ok(())
}

async fn prepare(common: &Common) -> AdmissionResult<(Vec<LocalInput>, Vec<NarFacts>)> {
    let plan: ApkPlan = crunch_eval::evaluate_and_deserialize(&common.plan, &[])?;
    let inputs: ApkInputs = serde_json::from_slice(&fs::read(&common.inputs)?)?;
    let manifest: Manifest = serde_json::from_slice(&fs::read(&common.manifest)?)?;
    let entries = validate_bindings(&plan, &inputs, manifest, &common.store_prefix)?;
    let mut facts = Vec::with_capacity(INPUT_COUNT);
    for entry in &entries {
        facts.push(observe_nar(&entry.source_path).await?);
    }
    Ok((entries, facts))
}

async fn admit(args: AdmitArgs) -> AdmissionResult<AdmissionReceipt> {
    let (entries, facts) = prepare(&args.common).await?;
    admit_prepared(args, entries, facts).await
}

async fn preflight_inputs(
    handle: &mut StoreHandle,
    args: &AdmitArgs,
    entries: &[LocalInput],
    facts: &[NarFacts],
    keypair: &crunch_build::KeyPair,
    public_key: &VerifyingKey,
) -> AdmissionResult<()> {
    // A cached PathInfo alone never establishes physical source readiness.
    for (entry, expected) in entries.iter().zip(facts) {
        let path: StorePath<String> =
            StorePath::from_absolute_path_with_prefix(entry.store_path.as_bytes(), &args.common.store_prefix)?;
        let actual_path = args.store.join(path.to_string());
        if let Some(existing) = handle.pathinfo_service().get(*path.digest()).await.map_err(std::io::Error::other)? {
            check_signed(&existing, &entry.store_path, &args.common.store_prefix, public_key)?;
            if observe_nar(&actual_path).await? != *expected {
                return Err(format!("cached APK source physical NAR differs: {}", entry.store_path).into());
            }
            handle
                .adopt_verified_local_output(&entry.store_path, path.name(), &keypair.signing_key, None)
                .await?;
        } else {
            match fs::symlink_metadata(&actual_path) {
                Ok(_) => {
                    return Err(
                        format!("APK source physical object lacks a signed PathInfo: {}", entry.store_path).into()
                    );
                }
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
                Err(error) => {
                    return Err(
                        format!("inspecting APK source physical object {}: {error}", actual_path.display()).into()
                    );
                }
            }
        }
        let candidate = handle
            .preflight_verified_source(VerifiedSourceIngestRequest {
                source_path: &entry.source_path,
                logical_store_path: &entry.store_path,
                source_name: path.name(),
                signing_key: &keypair.signing_key,
            })
            .await?;
        check_signed(&candidate, &entry.store_path, &args.common.store_prefix, public_key)?;
        if candidate.nar_size != expected.nar_size
            || data_encoding::HEXLOWER.encode(&candidate.nar_sha256) != expected.nar_sha256
        {
            return Err(format!("APK source candidate NAR differs: {}", entry.store_path).into());
        }
    }
    Ok(())
}

async fn admit_prepared(
    args: AdmitArgs,
    entries: Vec<LocalInput>,
    facts: Vec<NarFacts>,
) -> AdmissionResult<AdmissionReceipt> {
    for (entry, observed) in entries.iter().zip(&facts) {
        check_expected(entry, observed)?;
    }
    let keypair = crunch_build::signing::load_keypair(&fs::read_to_string(&args.signing_key)?)
        .map_err(|error| format!("existing Mantle signer invalid: {error}"))?;
    let public_key = VerifyingKey::parse(fs::read_to_string(&args.trusted_public_key)?.trim())
        .map_err(|error| format!("trusted Mantle public key invalid: {error}"))?;
    if keypair.verifying_key != public_key {
        return Err("Mantle signer is not the exact trusted public key".into());
    }
    if !args.store.is_absolute() || !args.state_dir.is_absolute() || args.store == args.state_dir {
        return Err("APK physical store and state must be different absolute paths".into());
    }
    let mut handle = StoreHandle::open(StoreConfig::new(
        StoreBackend::Snix,
        args.state_dir.clone(),
        args.store.clone(),
        args.common.store_prefix.clone(),
    ))
    .await?;
    // Preflight all four before publishing any signed PathInfo.
    preflight_inputs(&mut handle, &args, &entries, &facts, &keypair, &public_key).await?;
    let mut records = Vec::with_capacity(INPUT_COUNT);
    for (entry, expected) in entries.iter().zip(&facts) {
        let path: StorePath<String> =
            StorePath::from_absolute_path_with_prefix(entry.store_path.as_bytes(), &args.common.store_prefix)?;
        let request = VerifiedSourceIngestRequest {
            source_path: &entry.source_path,
            logical_store_path: &entry.store_path,
            source_name: path.name(),
            signing_key: &keypair.signing_key,
        };
        let info = handle.ingest_verified_source(request).await?;
        check_signed(&info, &entry.store_path, &args.common.store_prefix, &public_key)?;
        let exported = handle
            .export_cached_path_info(&path)
            .await?
            .ok_or("APK source PathInfo lacks complete castore content")?;
        if exported != info {
            return Err("APK source PathInfo changed during physical export".into());
        }
        // This adoption MUST only read back an already-present PathInfo. Its
        // absent-PathInfo branch signs a new one, so never use it as a probe.
        let persisted = handle
            .pathinfo_service()
            .get(*path.digest())
            .await
            .map_err(std::io::Error::other)?
            .ok_or("APK source PathInfo missing after ingest")?;
        if persisted != info {
            return Err("APK source persisted PathInfo differs from ingested PathInfo".into());
        }
        let observed = handle
            .adopt_verified_local_output(&entry.store_path, path.name(), &keypair.signing_key, None)
            .await?;
        if observed != persisted {
            return Err("APK source physical NAR or node differs from signed PathInfo".into());
        }
        let slice = handle.observe_source_slice(&observed.node, "").await?;
        if slice.nar_size() != expected.nar_size
            || data_encoding::HEXLOWER.encode(&slice.nar_sha256()) != expected.nar_sha256
            || data_encoding::HEXLOWER.encode(&slice.nar_blake3()) != expected.nar_blake3
        {
            return Err("APK source castore NAR differs from preflighted source bytes".into());
        }
        let actual_path = args.store.join(path.to_string());
        if observe_nar(&actual_path).await? != *expected {
            return Err(format!("APK source physical NAR differs after export: {}", entry.store_path).into());
        }
        check_signed(&observed, &entry.store_path, &args.common.store_prefix, &public_key)?;
        records.push(InputReceipt {
            role: entry.role,
            store_path: entry.store_path.clone(),
            nar: expected.clone(),
            signer: Some(public_key.to_string()),
            physical_path: Some(actual_path),
        });
    }
    Ok(AdmissionReceipt {
        schema: "mantle-android-local-input-admission-v1",
        ready_class: "signed-physical",
        records,
        non_claim: NON_CLAIM,
    })
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let report = match Cli::parse().command {
        Command::Observe(common) => {
            let (entries, facts) = prepare(&common).await?;
            AdmissionReceipt {
                schema: "mantle-android-local-input-admission-v1",
                ready_class: "observed-only",
                records: entries
                    .into_iter()
                    .zip(facts)
                    .map(|(entry, nar)| InputReceipt {
                        role: entry.role,
                        store_path: entry.store_path,
                        nar,
                        signer: None,
                        physical_path: None,
                    })
                    .collect(),
                non_claim: NON_CLAIM,
            }
        }
        Command::Admit(args) => admit(args).await?,
    };
    println!("{}", serde_json::to_string_pretty(&report)?);
    Ok(())
}

#[cfg(test)]
mod tests {
    use crunch_android_core::ReproducibilityPolicy;
    use crunch_android_core::SigningConfig;
    use crunch_android_core::ToolchainIdentity;
    use crunch_android_core::ToolchainRefs;

    use super::*;

    const PREFIX: &str = "/mantle/store";
    const APP: &str = "/mantle/store/0123456789abcdfghijklmnpqrsvwxyz-android-app";
    const GLIBC: &str = "/mantle/store/1123456789abcdfghijklmnpqrsvwxyz-android-glibc";
    const LIBGCC: &str = "/mantle/store/3123456789abcdfghijklmnpqrsvwxyz-android-libgcc";
    const SIGNING: &str = "/mantle/store/2123456789abcdfghijklmnpqrsvwxyz-android-test-signing";
    const TEST_SIGNER: &str =
        "cache.example.com-1:cCta2MEsRNuYCgWYyeRXLyfoFpKhQJKn8gLMeXWAb7vIpRKKo/3JoxJ24OYa3DxT2JVV38KjK/1ywHWuMe2JEw==";

    struct FixtureRoot(Option<tempfile::TempDir>);

    impl FixtureRoot {
        fn path(&self) -> &Path {
            self.0.as_ref().unwrap().path()
        }
    }

    impl Drop for FixtureRoot {
        fn drop(&mut self) {
            let Some(root) = self.0.take() else {
                return;
            };
            let path = root.path().to_path_buf();
            if let Err(error) = prepare_fixture_cleanup(&path).and_then(|()| root.close()) {
                if std::thread::panicking() {
                    eprintln!("cleaning owned APK fixture {}: {error}", path.display());
                } else {
                    panic!("cleaning owned APK fixture {}: {error}", path.display());
                }
            }
        }
    }

    fn prepare_fixture_cleanup(root: &Path) -> std::io::Result<()> {
        use std::os::unix::fs::MetadataExt;
        use std::os::unix::fs::PermissionsExt;

        let owner = fs::symlink_metadata(root)?;
        let mut pending = vec![root.to_path_buf()];
        while let Some(path) = pending.pop() {
            let metadata = fs::symlink_metadata(&path)?;
            if metadata.file_type().is_symlink() || !metadata.is_dir() {
                continue;
            }
            if metadata.uid() != owner.uid() || metadata.dev() != owner.dev() {
                return Err(std::io::Error::other(format!("non-owned APK fixture directory: {}", path.display())));
            }
            let mut permissions = metadata.permissions();
            permissions.set_mode(permissions.mode() | 0o700);
            fs::set_permissions(&path, permissions)?;
            for entry in fs::read_dir(&path)? {
                pending.push(entry?.path());
            }
        }
        Ok(())
    }

    fn write_file(path: &Path, bytes: &[u8]) {
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, bytes).unwrap();
    }

    fn fixture() -> (FixtureRoot, AdmitArgs, Vec<LocalInput>, ApkPlan, ApkInputs) {
        let root = FixtureRoot(Some(tempfile::tempdir().unwrap()));
        let app = root.path().join("app");
        let glibc = root.path().join("glibc");
        let libgcc = root.path().join("libgcc");
        let signing = root.path().join("signing");
        write_file(&app.join("AndroidManifest.xml"), b"<manifest package=\"org.example.test\"/>");
        write_file(&app.join("res/values/strings.xml"), b"<resources/>");
        write_file(&app.join("java/Main.java"), b"class Main {}");
        write_file(&glibc.join("lib/ld-linux.so"), b"native-test-loader");
        write_file(&glibc.join("lib/libc.so.6"), b"native-test-libc");
        write_file(&libgcc.join("lib/libgcc_s.so.1"), b"native-test-libgcc");
        write_file(&signing.join("release.jks"), b"test-only keystore fixture, never a signing authority");
        write_file(&signing.join("store.pass"), b"test-only-store-pass");
        write_file(&signing.join("key.pass"), b"test-only-key-pass");
        let toolchain = |name: &str| ToolchainIdentity {
            component: name.into(),
            record_blake3: "a".repeat(64),
            sha256: "sha256-mS+W55lQdax2NrsajeUrDGHXHtMTf6/JeauWtKt43XU=".into(),
        };
        let plan = ApkPlan {
            module: "test".into(),
            application_id: "org.example.test".into(),
            version_code: 1,
            version_name: "1.0".into(),
            manifest: "AndroidManifest.xml".into(),
            resources: vec!["res/values/strings.xml".into()],
            java_sources: vec!["java/Main.java".into()],
            toolchains: ToolchainRefs {
                jdk: toolchain("jdk"),
                build_tools: toolchain("build-tools"),
                platform: toolchain("platform-android-jar"),
            },
            signing: Some(SigningConfig {
                keystore: format!("{SIGNING}/release.jks"),
                store_password_file: format!("{SIGNING}/store.pass"),
                key_password_file: format!("{SIGNING}/key.pass"),
                alias: "test".into(),
                schemes: vec!["v2".into()],
            }),
            reproducibility: ReproducibilityPolicy {
                entry_timestamp_epoch: 315_532_800,
                locale: "C".into(),
                timezone: "UTC".into(),
            },
        };
        let inputs = ApkInputs {
            source_root: APP.into(),
            archives: Vec::new(),
            runtime_loader: format!("{GLIBC}/lib/ld-linux.so"),
            native_libgcc_root: LIBGCC.into(),
            runtime_library_paths: vec![format!("{GLIBC}/lib"), format!("{LIBGCC}/lib")],
        };
        let manifest = Manifest {
            entries: [
                LocalInput {
                    role: Role::TestSigning,
                    source_path: signing,
                    store_path: SIGNING.into(),
                    expected_nar_sha256: None,
                    expected_nar_blake3: None,
                    expected_nar_size: None,
                },
                LocalInput {
                    role: Role::AppSource,
                    source_path: app,
                    store_path: APP.into(),
                    expected_nar_sha256: None,
                    expected_nar_blake3: None,
                    expected_nar_size: None,
                },
                LocalInput {
                    role: Role::NativeGlibc,
                    source_path: glibc,
                    store_path: GLIBC.into(),
                    expected_nar_sha256: None,
                    expected_nar_blake3: None,
                    expected_nar_size: None,
                },
                LocalInput {
                    role: Role::NativeLibgcc,
                    source_path: libgcc,
                    store_path: LIBGCC.into(),
                    expected_nar_sha256: None,
                    expected_nar_blake3: None,
                    expected_nar_size: None,
                },
            ],
        };
        let entries = validate_bindings(&plan, &inputs, manifest, PREFIX).unwrap();
        let keypair = crunch_build::load_keypair(TEST_SIGNER).unwrap();
        let signer_file = root.path().join("mantle-signer.key");
        let trusted_file = root.path().join("mantle-trusted.pub");
        write_file(&signer_file, TEST_SIGNER.as_bytes());
        write_file(&trusted_file, keypair.verifying_key.to_string().as_bytes());
        let args = AdmitArgs {
            common: Common {
                plan: root.path().join("unused-test-plan.ncl"),
                inputs: root.path().join("unused-test-inputs.json"),
                manifest: root.path().join("unused-test-manifest.json"),
                store_prefix: PREFIX.into(),
            },
            store: root.path().join("store"),
            state_dir: root.path().join("state"),
            signing_key: signer_file,
            trusted_public_key: trusted_file,
        };
        (root, args, entries, plan, inputs)
    }

    async fn observed(mut entries: Vec<LocalInput>) -> (Vec<LocalInput>, Vec<NarFacts>) {
        let mut facts = Vec::new();
        for entry in &mut entries {
            let fact = observe_nar(&entry.source_path).await.unwrap();
            entry.expected_nar_sha256 = Some(fact.nar_sha256.clone());
            entry.expected_nar_blake3 = Some(fact.nar_blake3.clone());
            entry.expected_nar_size = Some(fact.nar_size);
            facts.push(fact);
        }
        (entries, facts)
    }
    fn validate_fixture(
        plan: &ApkPlan,
        inputs: &ApkInputs,
        entries: &[LocalInput],
    ) -> AdmissionResult<Vec<LocalInput>> {
        assert_eq!(entries.len(), INPUT_COUNT);
        let manifest = Manifest {
            entries: std::array::from_fn(|index| entries[index].clone()),
        };
        validate_bindings(plan, inputs, manifest, PREFIX)
    }

    #[test]
    fn four_native_bindings_reject_missing_duplicate_ambiguous_and_escaped_roots() {
        let (_root, _args, entries, plan, inputs) = fixture();
        let mut missing = entries.clone();
        missing[2].role = Role::NativeGlibc;
        assert!(validate_fixture(&plan, &inputs, &missing).is_err());
        let mut aliased = entries.clone();
        aliased[2].store_path = GLIBC.into();
        assert!(validate_fixture(&plan, &inputs, &aliased).is_err());
        let mut aliased = entries.clone();
        aliased[2].source_path = aliased[1].source_path.clone();
        assert!(validate_fixture(&plan, &inputs, &aliased).is_err());
        let mut wrong = inputs.clone();
        wrong.native_libgcc_root = format!("{LIBGCC}/lib");
        assert!(validate_fixture(&plan, &wrong, &entries).is_err());
        let mut wrong = inputs.clone();
        wrong.runtime_library_paths = vec![format!("{GLIBC}/lib"), format!("{GLIBC}/lib")];
        assert!(validate_fixture(&plan, &wrong, &entries).is_err());
        wrong.runtime_library_paths = vec![format!("{GLIBC}/lib"), format!("{LIBGCC}/../lib")];
        assert!(validate_fixture(&plan, &wrong, &entries).is_err());
        wrong.runtime_library_paths = vec![format!("{GLIBC}/lib"), "/usr/lib".into()];
        assert!(validate_fixture(&plan, &wrong, &entries).is_err());
        wrong.runtime_library_paths = vec![format!("{LIBGCC}/lib"), format!("{GLIBC}/lib")];
        assert!(validate_fixture(&plan, &wrong, &entries).is_ok(), "library search order is caller-owned");
    }

    #[test]
    fn runtime_members_must_exist_inside_their_own_full_source_trees() {
        use std::os::unix::fs::symlink;
        let (root, _args, entries, plan, inputs) = fixture();
        fs::remove_file(entries[2].source_path.join("lib/libgcc_s.so.1")).unwrap();
        assert!(validate_fixture(&plan, &inputs, &entries).is_err());
        let outside = root.path().join("outside-libgcc");
        write_file(&outside, b"host libgcc");
        symlink(&outside, entries[2].source_path.join("lib/libgcc_s.so.1")).unwrap();
        assert!(validate_fixture(&plan, &inputs, &entries).is_err());
        fs::remove_file(entries[2].source_path.join("lib/libgcc_s.so.1")).unwrap();
        write_file(&entries[2].source_path.join("lib/libgcc_s.so.1"), b"native-test-libgcc");
        fs::remove_file(entries[1].source_path.join("lib/ld-linux.so")).unwrap();
        assert!(validate_fixture(&plan, &inputs, &entries).is_err());
    }

    #[test]
    fn source_members_reject_host_escape_and_preserve_relative_runtime_symlinks() {
        use std::os::unix::fs::symlink;

        let root = tempfile::tempdir().unwrap();
        let source = root.path().join("source");
        let outside = root.path().join("outside");
        write_file(&source.join("lib/loader"), b"local loader bytes");
        write_file(&outside, b"host-only bytes");
        symlink(&outside, source.join("host-link")).unwrap();
        symlink(source.join("lib/loader"), source.join("absolute-local")).unwrap();
        symlink("loader", source.join("lib/relative-loader")).unwrap();
        assert!(check_local_member(&source, "host-link", true, true).is_err());
        assert!(check_local_member(&source, "absolute-local", true, true).is_err());
        check_local_member(&source, "lib/relative-loader", true, true).unwrap();
        assert!(check_local_member(&source, "lib/relative-loader", true, false).is_err());
    }

    #[tokio::test]
    async fn signed_input_admission_proves_exact_path_signature_physical_bytes_and_replay() {
        let (_root, args, entries, _, _) = fixture();
        let (entries, facts) = observed(entries).await;
        let report = admit_prepared(args.clone(), entries.clone(), facts.clone()).await.unwrap();
        assert_eq!(report.ready_class, "signed-physical");
        assert_eq!(report.records.len(), INPUT_COUNT);
        let keypair = crunch_build::load_keypair(TEST_SIGNER).unwrap();
        let handle = StoreHandle::open(StoreConfig::new(
            StoreBackend::Snix,
            args.state_dir.clone(),
            args.store.clone(),
            PREFIX.into(),
        ))
        .await
        .unwrap();
        for (entry, expected) in entries.iter().zip(&facts) {
            let path: StorePath<String> =
                StorePath::from_absolute_path_with_prefix(entry.store_path.as_bytes(), PREFIX).unwrap();
            let persisted = handle.pathinfo_service().get(*path.digest()).await.unwrap().unwrap();
            check_signed(&persisted, &entry.store_path, PREFIX, &keypair.verifying_key).unwrap();
            assert_eq!(persisted.nar_size, expected.nar_size);
            assert_eq!(data_encoding::HEXLOWER.encode(&persisted.nar_sha256), expected.nar_sha256);
            assert_eq!(observe_nar(&args.store.join(path.to_string())).await.unwrap(), *expected);
        }
        drop(handle);
        assert_eq!(
            fs::read(
                args.store.join(APP.trim_start_matches(PREFIX).trim_start_matches('/')).join("AndroidManifest.xml")
            )
            .unwrap(),
            b"<manifest package=\"org.example.test\"/>"
        );
        let replay = admit_prepared(args, entries, facts).await.unwrap();
        assert_eq!(serde_json::to_value(report).unwrap(), serde_json::to_value(replay).unwrap());
    }

    #[tokio::test]
    async fn reviewed_nar_mismatch_and_changed_source_reject_before_store_mutation() {
        let (_root, args, entries, _, _) = fixture();
        let (entries, facts) = observed(entries).await;
        let mut drifted = entries.clone();
        drifted[2].expected_nar_blake3 = Some("0".repeat(64));
        assert!(admit_prepared(args.clone(), drifted, facts.clone()).await.is_err());
        assert!(!args.state_dir.exists(), "invalid declarations must not create store state");
        fs::remove_file(entries[2].source_path.join("lib/libgcc_s.so.1")).unwrap();
        let changed = observe_nar(&entries[2].source_path).await.unwrap();
        assert_ne!(changed, facts[2], "changed libgcc bytes must change its own NAR identity");
        let mut observed_after_change = facts;
        observed_after_change[2] = changed;
        assert!(admit_prepared(args.clone(), entries, observed_after_change).await.is_err());
        assert!(!args.state_dir.exists(), "drifted source must not create store state");
    }

    #[tokio::test]
    async fn cached_sources_fail_on_absent_or_mutated_physical_bytes_and_untrusted_signer() {
        let (root, args, entries, _, _) = fixture();
        let (entries, facts) = observed(entries).await;
        admit_prepared(args.clone(), entries.clone(), facts.clone()).await.unwrap();
        let path: StorePath<String> = StorePath::from_absolute_path_with_prefix(APP.as_bytes(), PREFIX).unwrap();
        let handle = StoreHandle::open(StoreConfig::new(
            StoreBackend::Snix,
            args.state_dir.clone(),
            args.store.clone(),
            PREFIX.into(),
        ))
        .await
        .unwrap();
        let initial = handle.pathinfo_service().get(*path.digest()).await.unwrap().unwrap();
        drop(handle);
        let physical = args.store.join(path.to_string());
        let quarantined = root.path().join("quarantined-store");
        fs::rename(&args.store, &quarantined).unwrap();
        fs::create_dir(&args.store).unwrap();
        assert!(admit_prepared(args.clone(), entries.clone(), facts.clone()).await.is_err());
        assert!(!physical.exists(), "cached source failure must not silently restore absent physical bytes");
        fs::create_dir_all(&physical).unwrap();
        write_file(&physical.join("AndroidManifest.xml"), b"mutated after successful admission");
        assert!(admit_prepared(args.clone(), entries.clone(), facts.clone()).await.is_err());
        assert_eq!(fs::read(physical.join("AndroidManifest.xml")).unwrap(), b"mutated after successful admission");
        prepare_fixture_cleanup(&args.store).unwrap();
        fs::remove_dir_all(&args.store).unwrap();
        fs::rename(&quarantined, &args.store).unwrap();
        assert_eq!(observe_nar(&physical).await.unwrap(), facts[0]);
        let (other, private_line) = crunch_build::generate_keypair();
        let other_private = root.path().join("other.key");
        let other_public = root.path().join("other.pub");
        write_file(&other_private, private_line.as_bytes());
        write_file(&other_public, other.verifying_key.to_string().as_bytes());
        let mut wrong = args.clone();
        wrong.signing_key = other_private;
        wrong.trusted_public_key = other_public;
        assert!(admit_prepared(wrong, entries, facts).await.is_err());
        check_signed(&initial, APP, PREFIX, &other.verifying_key).unwrap_err();
        let handle = StoreHandle::open(StoreConfig::new(StoreBackend::Snix, args.state_dir, args.store, PREFIX.into()))
            .await
            .unwrap();
        assert_eq!(handle.pathinfo_service().get(*path.digest()).await.unwrap().unwrap(), initial);
        drop(handle);
    }

    #[tokio::test]
    async fn missing_signing_source_fails_before_any_signed_input_pathinfo() {
        let (_root, args, entries, _, _) = fixture();
        let (entries, facts) = observed(entries).await;
        fs::remove_dir_all(&entries[3].source_path).unwrap();
        assert!(admit_prepared(args.clone(), entries, facts).await.is_err());
        let handle = StoreHandle::open(StoreConfig::new(StoreBackend::Snix, args.state_dir, args.store, PREFIX.into()))
            .await
            .unwrap();
        let path: StorePath<String> = StorePath::from_absolute_path_with_prefix(APP.as_bytes(), PREFIX).unwrap();
        assert!(handle.pathinfo_service().get(*path.digest()).await.unwrap().is_none());
    }

    #[tokio::test]
    async fn orphan_physical_object_without_signed_pathinfo_is_not_ready() {
        let (root, mut args, entries, _, _) = fixture();
        let (entries, facts) = observed(entries).await;
        admit_prepared(args.clone(), entries.clone(), facts.clone()).await.unwrap();
        args.state_dir = root.path().join("fresh-empty-state");
        assert!(admit_prepared(args.clone(), entries, facts).await.is_err());
        let path: StorePath<String> = StorePath::from_absolute_path_with_prefix(APP.as_bytes(), PREFIX).unwrap();
        let handle = StoreHandle::open(StoreConfig::new(StoreBackend::Snix, args.state_dir, args.store, PREFIX.into()))
            .await
            .unwrap();
        assert!(handle.pathinfo_service().get(*path.digest()).await.unwrap().is_none());
    }
}
