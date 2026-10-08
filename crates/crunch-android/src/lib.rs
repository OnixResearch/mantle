mod authority;
use std::collections::BTreeMap;
use std::fs::File;
use std::fs::OpenOptions;
use std::io::Read;
use std::os::unix::fs::MetadataExt;
use std::os::unix::fs::OpenOptionsExt;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::path::PathBuf;

pub use authority::verified_same_run_extractions;
use crunch_android_core::LoweredApk;
use crunch_android_core::Rejection;
use crunch_android_core::SourceIdentity;
use crunch_android_core::Toolchain;
use crunch_android_core::lower;
use crunch_android_core::reviewed::ApkPlan;
use crunch_android_core::reviewed::StepKind;
use serde::Deserialize;
use serde::Serialize;
use sha2::Digest;
use sha2::Sha256;

const MAX_ARCHIVE_BYTES: u64 = 2 * 1024 * 1024 * 1024;
const MAX_SCRIPT_BYTES: usize = 128 * 1024;

#[derive(Debug)]
pub enum Error {
    Plan(Rejection),
    MissingArchive(String),
    DigestDrift(String),
    InvalidInput(String),
    MissingPreviousOutput(StepKind),
    Io(std::io::Error),
    ScriptTooLarge,
}
impl From<std::io::Error> for Error {
    fn from(error: std::io::Error) -> Self {
        Self::Io(error)
    }
}

/// A source is a verified fixed-output *archive*, never a caller-supplied
/// extracted executable tree. Extraction derivations recheck SHA-256 before
/// unpacking; the store authority must verify their outputs before use.
#[derive(Clone, Debug)]
pub struct BoundArchive {
    pub identity: SourceIdentity,
    pub store_path: PathBuf,
}

/// Immutable runtime source identity, reviewed independently of the APK plan.
/// `nar_sha256` is the full tree's NAR SHA-256, not an individual ELF hash.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RuntimeIdentity {
    pub source_cohort: String,
    pub nar_sha256: String,
    pub nar_size: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RuntimeCohort {
    pub glibc: RuntimeIdentity,
    pub libgcc: RuntimeIdentity,
}

#[derive(Clone, Debug)]
pub struct BoundRuntime {
    pub identity: RuntimeIdentity,
    pub store_path: PathBuf,
}
#[derive(Clone, Debug)]
pub struct BuildInputs {
    pub source_root: PathBuf,
    pub build_tools: BoundArchive,
    pub jdk: BoundArchive,
    /// Full signed runtime trees, never ambient host ELF interpreter/libraries.
    pub runtime_glibc: BoundRuntime,
    pub runtime_libgcc: BoundRuntime,
    pub platform: BoundArchive,
    /// Declared, statically linked busybox executable providing sha256sum,
    /// unzip, tar, mkdir, find, sort, cp, and date in the sandbox.
    pub utility: PathBuf,
    pub keystore: Option<PathBuf>,
    pub key_password_file: Option<PathBuf>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct DerivationStep {
    pub name: String,
    pub builder: String,
    pub args: Vec<String>,
    pub system: String,
    pub outputs: Vec<String>,
    pub addressing_mode: String,
    pub env: BTreeMap<String, String>,
    pub inputs: Vec<String>,
    pub kind: StepKind,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ExtractionKind {
    BuildTools,
    Jdk,
    Platform,
}

#[derive(Clone, Debug)]
pub struct ExtractionDerivation {
    pub kind: ExtractionKind,
    pub name: String,
    pub builder: String,
    pub args: Vec<String>,
    pub system: String,
    pub outputs: Vec<String>,
    pub addressing_mode: String,
    pub env: BTreeMap<String, String>,
    pub inputs: Vec<String>,
    pub archive_sha256: String,
    pub record_identity_blake3: String,
    pub unpack_root: String,
}

pub struct ExtractedTrees {
    build: String,
    jdk: String,
    platform: String,
}

pub struct PreparedApk {
    plan: ApkPlan,
    lowered: LoweredApk,
    inputs: BuildInputs,
}

fn store_path(path: &Path, store_prefix: &Path) -> Result<String, Error> {
    if !path.is_absolute()
        || !path.starts_with(store_prefix)
        || path == store_prefix
        || path
            .components()
            .any(|component| !matches!(component, std::path::Component::RootDir | std::path::Component::Normal(_)))
    {
        return Err(Error::InvalidInput(path.display().to_string()));
    }
    let text = path.to_str().ok_or_else(|| Error::InvalidInput(path.display().to_string()))?;
    if text.bytes().any(|byte| byte == b'\'' || byte == b'\n' || byte == b'\r' || byte == 0)
        || text.len() > crunch_android_core::MAX_PATH_BYTES
    {
        return Err(Error::InvalidInput(text.to_string()));
    }
    Ok(text.to_string())
}

fn store_object(path: &Path, store_prefix: &Path) -> Result<String, Error> {
    let relative = path.strip_prefix(store_prefix).map_err(|_| Error::InvalidInput(path.display().to_string()))?;
    let first = relative.components().next().ok_or_else(|| Error::InvalidInput(path.display().to_string()))?;
    store_path(&store_prefix.join(first.as_os_str()), store_prefix)
}

fn declared_member(root: &Path, member: &str, missing: Rejection) -> Result<(), Error> {
    let root_meta = std::fs::symlink_metadata(root).map_err(|_| Error::Plan(missing))?;
    if !root_meta.is_dir() || root_meta.file_type().is_symlink() {
        return Err(Error::Plan(missing));
    }
    let mut current = root.to_path_buf();
    let parts: Vec<_> = member.split('/').collect();
    for (index, part) in parts.iter().enumerate() {
        current.push(part);
        let metadata = std::fs::symlink_metadata(&current).map_err(|_| Error::Plan(missing))?;
        if metadata.file_type().is_symlink() {
            return Err(Error::Plan(missing));
        }
        if index + 1 == parts.len() {
            if !metadata.is_file() {
                return Err(Error::Plan(missing));
            }
        } else if !metadata.is_dir() {
            return Err(Error::Plan(missing));
        }
    }
    Ok(())
}

fn declared_regular_file(path: &Path, store_prefix: &Path) -> Result<File, Error> {
    let text = store_path(path, store_prefix)?;
    let prefix_meta = std::fs::symlink_metadata(store_prefix).map_err(|_| Error::InvalidInput(text.clone()))?;
    if !prefix_meta.is_dir() || prefix_meta.file_type().is_symlink() {
        return Err(Error::InvalidInput(text));
    }
    let mut current = store_prefix.to_path_buf();
    let relative = path.strip_prefix(store_prefix).map_err(|_| Error::InvalidInput(text.clone()))?;
    let parts: Vec<_> = relative.components().collect();
    for (index, part) in parts.iter().enumerate() {
        current.push(part);
        let metadata = std::fs::symlink_metadata(&current).map_err(|_| Error::InvalidInput(text.clone()))?;
        if metadata.file_type().is_symlink() {
            return Err(Error::InvalidInput(text));
        }
        if index + 1 == parts.len() {
            if !metadata.is_file() {
                return Err(Error::InvalidInput(text));
            }
            let opened = OpenOptions::new().read(true).custom_flags(libc::O_NOFOLLOW).open(path)?;
            let observed = opened.metadata()?;
            if metadata.dev() != observed.dev() || metadata.ino() != observed.ino() {
                return Err(Error::InvalidInput(path.display().to_string()));
            }
            return Ok(opened);
        }
        if !metadata.is_dir() {
            return Err(Error::InvalidInput(text));
        }
    }
    Err(Error::InvalidInput(text))
}

fn verify_archive(archive: &BoundArchive, expected: &SourceIdentity, prefix: &Path) -> Result<(), Error> {
    if archive.identity != *expected {
        return Err(Error::DigestDrift(expected.component.clone()));
    }
    let mut file = declared_regular_file(&archive.store_path, prefix)
        .map_err(|_| Error::MissingArchive(archive.store_path.display().to_string()))?;
    if file.metadata()?.len() > MAX_ARCHIVE_BYTES {
        return Err(Error::InvalidInput(expected.component.clone()));
    }
    let mut hash = Sha256::new();
    let mut buffer = [0_u8; 65_536];
    loop {
        let count = file.read(&mut buffer)?;
        if count == 0 {
            break;
        }
        hash.update(&buffer[..count]);
    }
    if format!("{:x}", hash.finalize()) != expected.sha256 {
        return Err(Error::DigestDrift(expected.component.clone()));
    }
    Ok(())
}

// r[impl android_adapter.apk_pipeline_lowering]
impl PreparedApk {
    /// `reviewed` MUST come from an independently approved source manifest,
    /// never from the plan or the admitted archives themselves.
    /// All source-record and FOD checks finish before any tool invocation.
    /// This direct adapter accepts the caller's separately reviewed toolchain;
    /// it does not infer or grant ADR 0089 canonical source-v1 admission.
    pub fn bind(
        plan: ApkPlan,
        inputs: BuildInputs,
        reviewed: &Toolchain,
        reviewed_runtime: &RuntimeCohort,
        store_prefix: &Path,
    ) -> Result<Self, Error> {
        let lowered = lower(&plan).map_err(Error::Plan)?;
        if plan.toolchain != *reviewed {
            return Err(Error::Plan(Rejection::UnboundToolchainIdentity));
        }
        if inputs.runtime_glibc.identity != reviewed_runtime.glibc
            || inputs.runtime_libgcc.identity != reviewed_runtime.libgcc
        {
            return Err(Error::InvalidInput("runtime source cohort drift".into()));
        }
        for runtime in [&inputs.runtime_glibc, &inputs.runtime_libgcc] {
            store_path(&runtime.store_path, store_prefix)?;
            if runtime.identity.source_cohort.is_empty()
                || runtime.identity.source_cohort.len() > 256
                || runtime.identity.nar_size == 0
                || runtime.identity.nar_sha256.len() != 64
                || !runtime.identity.nar_sha256.bytes().all(|byte| byte.is_ascii_hexdigit())
            {
                return Err(Error::InvalidInput("invalid reviewed runtime identity".into()));
            }
        }
        declared_member(
            &inputs.runtime_glibc.store_path,
            "lib/ld-linux-x86-64.so.2",
            Rejection::UnboundToolchainIdentity,
        )?;
        declared_member(&inputs.runtime_libgcc.store_path, "lib/libgcc_s.so.1", Rejection::UnboundToolchainIdentity)?;
        store_path(&inputs.source_root, store_prefix)?;
        let utility = declared_regular_file(&inputs.utility, store_prefix)?;
        if utility.metadata()?.permissions().mode() & 0o111 == 0 {
            return Err(Error::InvalidInput("utility not executable".into()));
        }
        declared_member(&inputs.source_root, &plan.manifest_member, Rejection::MissingManifestMember)?;
        for member in plan.resources.iter().chain(&plan.java_sources) {
            declared_member(&inputs.source_root, member, Rejection::InvalidSourceMember)?;
        }
        verify_archive(&inputs.build_tools, &plan.toolchain.build_tools, store_prefix)?;
        verify_archive(&inputs.jdk, &plan.toolchain.jdk, store_prefix)?;
        verify_archive(&inputs.platform, &plan.toolchain.platform, store_prefix)?;
        if let Some(signing) = &plan.signing {
            let keystore = inputs.keystore.as_ref().ok_or_else(|| Error::InvalidInput("keystore missing".into()))?;
            if *keystore != inputs.source_root.join(&signing.keystore_member) {
                return Err(Error::Plan(Rejection::KeystorePathEscape));
            }
            let password = inputs
                .key_password_file
                .as_ref()
                .ok_or_else(|| Error::InvalidInput("password file missing".into()))?;
            declared_regular_file(keystore, store_prefix)?;
            declared_regular_file(password, store_prefix)?;
        } else if inputs.keystore.is_some() || inputs.key_password_file.is_some() {
            return Err(Error::InvalidInput("unsigned build carries signing input".into()));
        }
        Ok(Self { plan, lowered, inputs })
    }

    pub fn steps(&self) -> &[crunch_android_core::reviewed::StepPlan] {
        &self.lowered.steps
    }

    /// Three deterministic extraction derivations, each tied to one verified
    /// fixed-output archive. They run once, before any SDK executable step.
    pub fn extractions(&self, store_prefix: &Path) -> Result<[ExtractionDerivation; 3], Error> {
        let make = |kind: ExtractionKind, archive: &BoundArchive| -> Result<ExtractionDerivation, Error> {
            let path = store_path(&archive.store_path, store_prefix)?;
            let utility = store_path(&self.inputs.utility, store_prefix)?;
            let utility_source = store_object(&self.inputs.utility, store_prefix)?;
            let unpack = match archive.identity.unpack_shape.archive.as_str() {
                "zip" => "unzip -q \"$ARCHIVE\" -d \"$out\"",
                "tar-gz" => "tar -xzf \"$ARCHIVE\" -C \"$out\"",
                _ => return Err(Error::Plan(Rejection::UnboundToolchainIdentity)),
            };
            let script = format!(
                "set -eu\nexport PATH=/bin HOME=\"$PWD/.android-home\"\n\"$UTILITY\" mkdir -p \"$HOME\" \"$out\"\nobserved=$(\"$UTILITY\" sha256sum \"$ARCHIVE\" | {{ read -r digest rest; printf '%s' \"$digest\"; }})\n[ \"$observed\" = \"$EXPECTED_SHA256\" ] || exit 41\n\"$UTILITY\" {unpack}\n[ -d \"$out/{}\" ] || exit 42\n",
                archive.identity.unpack_shape.root.trim_end_matches('/')
            );
            Ok(ExtractionDerivation {
                kind,
                name: format!("android-extract-{}", archive.identity.component),
                builder: "/bin/sh".into(),
                args: vec!["-eu".into(), "-c".into(), script],
                system: "x86_64-linux".into(),
                outputs: vec!["out".into()],
                addressing_mode: "content-addressed".into(),
                env: BTreeMap::from([
                    ("UTILITY".into(), utility.clone()),
                    ("ARCHIVE".into(), path.clone()),
                    ("EXPECTED_SHA256".into(), archive.identity.sha256.clone()),
                    ("RECORD_IDENTITY_BLAKE3".into(), archive.identity.record_identity_blake3.clone()),
                    ("SOURCE_DATE_EPOCH".into(), self.plan.reproducibility.entry_timestamp_epoch.to_string()),
                    ("LC_ALL".into(), "C".into()),
                    ("TZ".into(), "UTC".into()),
                ]),
                inputs: vec![utility_source, path],
                archive_sha256: archive.identity.sha256.clone(),
                record_identity_blake3: archive.identity.record_identity_blake3.clone(),
                unpack_root: archive.identity.unpack_shape.root.clone(),
            })
        };
        Ok([
            make(ExtractionKind::BuildTools, &self.inputs.build_tools)?,
            make(ExtractionKind::Jdk, &self.inputs.jdk)?,
            make(ExtractionKind::Platform, &self.inputs.platform)?,
        ])
    }

    /// Realize steps in order. Previous CA outputs and extracted trees MUST be
    /// verified by their store authority before being passed to this adapter.
    pub fn derivation(
        &self,
        kind: StepKind,
        previous: &BTreeMap<StepKind, PathBuf>,
        trees: &ExtractedTrees,
        store_prefix: &Path,
    ) -> Result<DerivationStep, Error> {
        let step = self
            .lowered
            .steps
            .iter()
            .find(|step| step.kind == kind)
            .ok_or_else(|| Error::InvalidInput("step is absent".into()))?;
        let mut env = BTreeMap::from([
            ("LC_ALL".into(), "C".into()),
            ("TZ".into(), "UTC".into()),
            ("SOURCE_DATE_EPOCH".into(), self.plan.reproducibility.entry_timestamp_epoch.to_string()),
            (
                "JAVA_TOOL_OPTIONS".into(),
                "-Duser.language=en -Duser.country=US -Duser.timezone=UTC -Dfile.encoding=UTF-8".into(),
            ),
            ("SOURCE_ROOT".into(), store_path(&self.inputs.source_root, store_prefix)?),
            ("UTILITY".into(), store_path(&self.inputs.utility, store_prefix)?),
            ("RUNTIME_GLIBC".into(), store_path(&self.inputs.runtime_glibc.store_path, store_prefix)?),
            ("RUNTIME_LIBGCC".into(), store_path(&self.inputs.runtime_libgcc.store_path, store_prefix)?),
            ("RUNTIME_GLIBC_NAR_SHA256".into(), self.inputs.runtime_glibc.identity.nar_sha256.clone()),
            ("RUNTIME_LIBGCC_NAR_SHA256".into(), self.inputs.runtime_libgcc.identity.nar_sha256.clone()),
        ]);
        let mut declared = vec![
            env["SOURCE_ROOT"].clone(),
            store_object(&self.inputs.utility, store_prefix)?,
            env["RUNTIME_GLIBC"].clone(),
            env["RUNTIME_LIBGCC"].clone(),
        ];
        for (component, key, path, identity) in [
            ("android-build-tools", "EXTRACT_BUILD", &trees.build, &self.plan.toolchain.build_tools),
            ("temurin-jdk", "EXTRACT_JDK", &trees.jdk, &self.plan.toolchain.jdk),
            ("android-platform", "EXTRACT_PLATFORM", &trees.platform, &self.plan.toolchain.platform),
        ] {
            if step.inputs.iter().any(|input| {
                matches!(input,
                crunch_android_core::StepInput::ToolchainIdentity(identity) if identity.component == component)
            }) {
                env.insert(key.into(), path.clone());
                declared.push(path.clone());
                env.insert(format!("{key}_RECORD_IDENTITY_BLAKE3"), identity.record_identity_blake3.clone());
                env.insert(format!("{key}_SHA256"), identity.sha256.clone());
            }
        }
        for (previous_kind, name) in [
            (StepKind::Aapt2Compile, "COMPILED"),
            (StepKind::Aapt2Link, "LINKED"),
            (StepKind::Javac, "CLASSES"),
            (StepKind::D8, "DEX"),
            (StepKind::Zipalign, "UNSIGNED"),
        ] {
            if step.inputs.iter().any(
                |input| matches!(input, crunch_android_core::StepInput::PreviousOutput(kind) if *kind == previous_kind),
            ) {
                let output = previous.get(&previous_kind).ok_or(Error::MissingPreviousOutput(previous_kind))?;
                let path = store_path(output, store_prefix)?;
                env.insert(name.into(), path.clone());
                declared.push(path);
            }
        }
        if kind == StepKind::Apksigner {
            for (key, path) in [
                ("KEYSTORE", self.inputs.keystore.as_ref()),
                ("KEY_PASSWORD_FILE", self.inputs.key_password_file.as_ref()),
            ] {
                let path = path.ok_or_else(|| Error::InvalidInput(format!("{key} missing")))?;
                let path = store_path(path, store_prefix)?;
                env.insert(key.into(), path.clone());
                declared.push(path);
            }
        }
        declared.sort_unstable();
        declared.dedup();
        let script = self.script(kind)?;
        if script.len() > MAX_SCRIPT_BYTES {
            return Err(Error::ScriptTooLarge);
        }
        Ok(DerivationStep {
            name: format!("{}-{kind:?}", self.plan.module).to_ascii_lowercase(),
            builder: "/bin/sh".into(),
            args: vec!["-eu".into(), "-c".into(), script],
            system: "x86_64-linux".into(),
            outputs: vec!["out".into()],
            addressing_mode: "content-addressed".into(),
            env,
            inputs: declared,
            kind,
        })
    }

    fn script(&self, kind: StepKind) -> Result<String, Error> {
        let mut script = format!(
            "set -eu\nexport HOME=\"$PWD/.android-home\"\nexport PATH=/bin\n\"$UTILITY\" mkdir -p \"$HOME\" \"$out\"\nbuild_root=\"${{EXTRACT_BUILD:-}}/{}\"\njdk_root=\"${{EXTRACT_JDK:-}}/{}\"\nplatform_root=\"${{EXTRACT_PLATFORM:-}}/{}\"\nruntime_loader=\"$RUNTIME_GLIBC/lib/ld-linux-x86-64.so.2\"\nruntime_libraries=\"$RUNTIME_GLIBC/lib:$RUNTIME_LIBGCC/lib:$build_root/lib64:$jdk_root/lib:$jdk_root/lib/server\"\nrun_sdk() {{ \"$runtime_loader\" --library-path \"$runtime_libraries\" \"$@\"; }}\n",
            self.plan.toolchain.build_tools.unpack_shape.root.trim_end_matches('/'),
            self.plan.toolchain.jdk.unpack_shape.root.trim_end_matches('/'),
            self.plan.toolchain.platform.unpack_shape.root.trim_end_matches('/')
        );
        script.push_str(&self.command(kind));
        Ok(script)
    }

    fn command(&self, kind: StepKind) -> String {
        match kind {
            StepKind::Aapt2Compile => {
                let mut script = String::from("\"$UTILITY\" mkdir -p \"$out/compiled-resources\"\n");
                for input in &self.lowered.steps[0].inputs {
                    if let crunch_android_core::StepInput::SourceMember(member) = input {
                        script.push_str(&format!("run_sdk \"$build_root/aapt2\" compile -o \"$out/compiled-resources\" \"$SOURCE_ROOT/{member}\"\n"));
                    }
                }
                script
            }
            StepKind::Aapt2Link => format!("\"$UTILITY\" mkdir -p \"$out/gen\"\n\"$UTILITY\" find \"$COMPILED/compiled-resources\" -type f -name '*.flat' | \"$UTILITY\" sort > \"$PWD/resources.list\"\nset --\nwhile IFS= read -r item; do set -- \"$@\" \"$item\"; done < \"$PWD/resources.list\"\nrun_sdk \"$build_root/aapt2\" link -o \"$out/linked-resources.apk\" -I \"$platform_root/android.jar\" --manifest \"$SOURCE_ROOT/{}\" --rename-manifest-package '{}' --version-code '{}' --java \"$out/gen\" \"$@\"\n", self.plan.manifest_member, self.plan.application_id, self.plan.version_code),
            StepKind::Javac => {
                let mut script = String::from("\"$UTILITY\" mkdir -p \"$out/classes\"\nset --\n\"$UTILITY\" find \"$LINKED/gen\" -type f -name '*.java' | \"$UTILITY\" sort > \"$PWD/generated-java.list\"\nwhile IFS= read -r item; do set -- \"$@\" \"$item\"; done < \"$PWD/generated-java.list\"\n");
                for input in &self.lowered.steps[2].inputs {
                    if let crunch_android_core::StepInput::SourceMember(member) = input {
                        script.push_str(&format!("set -- \"$@\" \"$SOURCE_ROOT/{member}\"\n"));
                    }
                }
                script.push_str("run_sdk \"$jdk_root/bin/javac\" -encoding UTF-8 -classpath \"$platform_root/android.jar\" -d \"$out/classes\" \"$@\"\n"); script
            }
            StepKind::D8 => "\"$UTILITY\" find \"$CLASSES/classes\" -type f -name '*.class' | \"$UTILITY\" sort > \"$PWD/classes.list\"\nset --\nwhile IFS= read -r item; do set -- \"$@\" \"$item\"; done < \"$PWD/classes.list\"\nrun_sdk \"$jdk_root/bin/java\" -Xmx2G -cp \"$build_root/lib/d8.jar\" com.android.tools.r8.D8 --lib \"$platform_root/android.jar\" --output \"$out\" \"$@\"\n".into(),
            StepKind::Zipalign => "\"$UTILITY\" cp \"$LINKED/linked-resources.apk\" \"$PWD/with-dex.apk\"\nentry_date=$(\"$UTILITY\" date -u -d \"@$SOURCE_DATE_EPOCH\" +%Y-%m-%dT%H:%M:%SZ)\nrun_sdk \"$jdk_root/bin/jar\" --update --file \"$PWD/with-dex.apk\" --date \"$entry_date\" -C \"$DEX\" classes.dex\nrun_sdk \"$build_root/zipalign\" -f 4 \"$PWD/with-dex.apk\" \"$out/unsigned.apk\"\n".into(),
            StepKind::Apksigner => {
                let signing = self.plan.signing.as_ref().expect("signed step requires signing plan");
                let mut args = String::new();
                for scheme in ["v1", "v2", "v3"] {
                    args.push_str(&format!(" --{scheme}-signing-enabled {}", signing.schemes.iter().any(|enabled| enabled == scheme)));
                }
                format!("run_sdk \"$jdk_root/bin/java\" -Xmx1024M -jar \"$build_root/lib/apksigner.jar\" sign --ks \"$KEYSTORE\" --ks-key-alias '{}' --ks-pass \"file:$KEY_PASSWORD_FILE\" --key-pass \"file:$KEY_PASSWORD_FILE\"{args} --out \"$out/signed.apk\" \"$UNSIGNED/unsigned.apk\"\n", signing.key_alias)
            }
        }
    }
}

#[cfg(test)]
mod integration_tests;
#[cfg(test)]
mod tests;

pub mod pinned {
    //! Adapter for the pinned fixed-output source cohort and input-addressed APK stages.
    //! Offline preflight and native-sandbox derivations for the pinned Android APK cohort.
    //! The archive paths are inspected locally, but never used as build inputs: each
    //! executable is reached through its pinned, flat fixed-output fetch derivation.

    use std::collections::BTreeMap;
    use std::collections::BTreeSet;
    use std::collections::HashMap;
    use std::fmt;
    use std::fs::File;
    use std::io::Read;
    use std::path::Path;
    use std::path::PathBuf;

    use base64::Engine;
    use crunch_android_core::ApkPlan;
    use crunch_android_core::StepKind;
    use crunch_android_core::ToolchainComponent;
    use crunch_android_core::validate_and_lower;
    use crunch_glue::ConversionCache;
    use crunch_glue::CrunchDerivation;
    use crunch_glue::FixedOutput;
    use crunch_glue::Input;
    use crunch_glue::convert;
    use serde::Deserialize;
    use serde::Serialize;
    use sha2::Digest;
    use sha2::Sha256;

    #[derive(Clone, Debug, Deserialize, Serialize)]
    pub struct ApkInputs {
        pub source_root: String,
        pub archives: Vec<ArchiveInput>,
        pub runtime_loader: String,
        pub native_libgcc_root: String,
        pub runtime_library_paths: Vec<String>,
    }

    #[derive(Clone, Debug, Deserialize, Serialize)]
    pub struct ArchiveInput {
        pub component: String,
        pub local_archive: PathBuf,
    }

    #[derive(Clone, Debug)]
    pub struct ApkDerivations {
        /// Ordered APK stages only; the archive fetches and extracts are recursive dependencies.
        pub stages: Vec<CrunchDerivation>,
        pub final_derivation: CrunchDerivation,
        pub final_output_path: String,
    }

    #[derive(Debug)]
    pub struct AdapterError(String);

    impl fmt::Display for AdapterError {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.write_str(&self.0)
        }
    }
    impl std::error::Error for AdapterError {}

    fn error(message: impl Into<String>) -> AdapterError {
        AdapterError(message.into())
    }

    #[derive(Clone, Debug, Deserialize)]
    struct SourceRecord {
        component: String,
        version: String,
        url: String,
        sha256: String,
        record_blake3: String,
        unpack_shape: String,
        platform: String,
    }

    const COMPONENTS: [&str; 3] = ["jdk", "build-tools", "platform-android-jar"];

    /// Evaluate the production Nickel cohort; callers cannot substitute an identity
    /// or fetch URL, even if a forged `ApkPlan` passes the pure shape validator.
    pub fn prepare_apk(plan: &ApkPlan, inputs: &ApkInputs, store_prefix: &str) -> Result<ApkDerivations, AdapterError> {
        let embedded_dir =
            tempfile::tempdir().map_err(|e| error(format!("creating private Android cohort import directory: {e}")))?;
        let import_path = crunch_eval::stdlib::write_stdlib(Some(embedded_dir.path()))
            .map_err(|e| error(format!("materializing pinned Android cohort imports: {e}")))?;
        let json = crunch_eval::evaluate_str_to_json("let android = import \"android.ncl\" in android.cohort", &[
            import_path.into_os_string(),
        ])
        .map_err(|e| error(format!("evaluating pinned Android cohort: {e}")))?;
        let records: Vec<SourceRecord> =
            serde_json::from_str(&json).map_err(|e| error(format!("decoding pinned Android cohort: {e}")))?;
        prepare_with_records(plan, inputs, store_prefix, &records)
    }

    fn archive_sha256(path: &Path) -> Result<String, AdapterError> {
        let mut file =
            File::open(path).map_err(|e| error(format!("opening Android archive {}: {e}", path.display())))?;
        let mut hasher = Sha256::new();
        let mut buffer = [0_u8; 64 * 1024];
        loop {
            let count = file
                .read(&mut buffer)
                .map_err(|e| error(format!("reading Android archive {}: {e}", path.display())))?;
            if count == 0 {
                break;
            }
            hasher.update(&buffer[..count]);
        }
        Ok(format!("sha256-{}", base64::engine::general_purpose::STANDARD.encode(hasher.finalize())))
    }

    fn record_identity(record: &SourceRecord) -> Result<String, AdapterError> {
        let fields = [
            record.component.as_str(),
            record.version.as_str(),
            record.url.as_str(),
            record.sha256.as_str(),
            record.unpack_shape.as_str(),
            record.platform.as_str(),
        ];
        let json = serde_json::to_string(&fields).map_err(|e| error(format!("encoding Android record: {e}")))?;
        let mut hasher = blake3::Hasher::new();
        hasher.update(b"mantle-android-source-v1\n");
        hasher.update(json.as_bytes());
        Ok(blake3::Hasher::finalize(&hasher).to_hex().to_string())
    }

    /// Extract the store object, not a file within it. Every sandbox path used as
    /// input must belong to exactly this configured store namespace.
    fn store_object(path: &str, prefix: &str) -> Result<String, AdapterError> {
        if !prefix.starts_with('/')
            || !prefix.is_ascii()
            || prefix == "/"
            || prefix.ends_with('/')
            || prefix[1..].split('/').any(|part| part.is_empty() || part == "." || part == "..")
        {
            return Err(error("store prefix must be an absolute normalized directory"));
        }
        let rest = path
            .strip_prefix(prefix)
            .and_then(|value| value.strip_prefix('/'))
            .ok_or_else(|| error(format!("path is outside configured store prefix: {path}")))?;
        let mut parts = rest.split('/');
        let object = parts.next().unwrap_or_default();
        if !object.is_ascii()
            || object.len() < 34
            || object.as_bytes().get(32) != Some(&b'-')
            || !object[..32].bytes().all(|byte| b"0123456789abcdfghijklmnpqrsvwxyz".contains(&byte))
            || !object[33..].bytes().all(|byte| byte.is_ascii_alphanumeric() || b"+._?=-".contains(&byte))
            || parts.any(|part| {
                part.is_empty()
                    || part == "."
                    || part == ".."
                    || !part.bytes().all(|byte| byte.is_ascii_alphanumeric() || b"+._?=-".contains(&byte))
            })
        {
            return Err(error(format!("not a normalized store path: {path}")));
        }
        Ok(format!("{prefix}/{object}"))
    }

    fn shell(value: &str) -> Result<String, AdapterError> {
        if value.contains('\0') {
            return Err(error("NUL in APK shell argument"));
        }
        Ok(format!("'{}'", value.replace('\'', "'\\''")))
    }

    fn drv(
        name: &str,
        builder: &str,
        args: Vec<String>,
        inputs: Vec<Input>,
        env: HashMap<String, String>,
        fixed: Option<FixedOutput>,
    ) -> CrunchDerivation {
        CrunchDerivation {
            name: name.to_owned(),
            builder: builder.to_owned(),
            system: "x86_64-linux".to_owned(),
            args,
            outputs: vec!["out".to_owned()],
            dynamic_plan_outputs: vec![],
            env,
            inputs,
            fixed_output: fixed,
            addressing_mode: "input-addressed".to_owned(),
            provenance: None,
        }
    }

    fn output_path(
        derivation: &CrunchDerivation,
        cache: &mut ConversionCache,
        prefix: &str,
    ) -> Result<String, AdapterError> {
        let (_, converted) = convert(derivation, cache)
            .map_err(|e| error(format!("converting Android derivation {}: {e}", derivation.name)))?;
        converted
            .outputs
            .get("out")
            .and_then(|value| value.path.as_ref())
            .map(|value| value.to_absolute_path_with_prefix(prefix))
            .ok_or_else(|| error(format!("missing computed output path for {}", derivation.name)))
    }

    fn step_name(kind: StepKind) -> &'static str {
        match kind {
            StepKind::Aapt2Compile => "android-aapt2-compile",
            StepKind::Aapt2Link => "android-aapt2-link",
            StepKind::Javac => "android-javac",
            StepKind::D8 => "android-d8",
            StepKind::Zipalign => "android-zipalign-unsigned.apk",
            StepKind::Apksigner => "android-apksigner-signed.apk",
        }
    }

    fn tool_record<'a>(records: &BTreeMap<&str, &'a SourceRecord>, component: ToolchainComponent) -> &'a SourceRecord {
        let name = match component {
            ToolchainComponent::Jdk => "jdk",
            ToolchainComponent::BuildTools => "build-tools",
            ToolchainComponent::Platform => "platform-android-jar",
        };
        records.get(name).copied().expect("preflight resolved every required Android component")
    }

    fn stage_script(
        kind: StepKind,
        plan: &ApkPlan,
        source: &str,
        paths: &BTreeMap<String, String>,
        previous: &BTreeMap<&'static str, String>,
        loader: &str,
        libraries: &str,
    ) -> Result<String, AdapterError> {
        let jdk = paths.get("jdk").map(String::as_str).unwrap_or("");
        let tools = paths.get("build-tools").map(String::as_str).unwrap_or("");
        let platform = paths.get("platform-android-jar").map(String::as_str).unwrap_or("");
        let q = |value: &str| shell(value);
        let mut libraries = libraries.to_owned();
        if !jdk.is_empty() {
            libraries.push_str(&format!(":{jdk}/lib:{jdk}/lib/server"));
        }
        if !tools.is_empty() {
            libraries.push_str(&format!(":{tools}/lib64"));
        }
        let run = format!("{} --library-path {}", q(loader)?, q(&libraries)?);
        let java = || -> Result<String, AdapterError> {
            Ok(format!(
                "{run} {} -Duser.language=en -Duser.country=US -Duser.timezone=UTC -Dfile.encoding=UTF-8",
                q(&format!("{jdk}/bin/java"))?
            ))
        };
        let output = |name: &'static str| -> Result<&str, AdapterError> {
            previous
                .get(name)
                .map(String::as_str)
                .ok_or_else(|| error(format!("missing prior Android step output: {name}")))
        };
        let mut script =
            String::from("set -eu\nexport LC_ALL=C TZ=UTC\nexport JAVA_TOOL_OPTIONS=\"-Djava.io.tmpdir=$TMPDIR\"\n");
        match kind {
            StepKind::Aapt2Compile => {
                script.push_str("/bin/busybox mkdir -p \"$out\"\n");
                let mut resources: Vec<_> = plan.resources.iter().map(String::as_str).collect();
                resources.sort_unstable();
                let executable = q(&format!("{tools}/aapt2"))?;
                for resource in resources {
                    script.push_str(&format!(
                        "{run} {executable} compile -o \"$out\" {}\n",
                        q(&format!("{source}/{resource}"))?
                    ));
                }
            }
            StepKind::Aapt2Link => {
                script.push_str("/bin/busybox mkdir -p \"$out/java\"\nset --\n");
                script.push_str(&format!("/bin/busybox find {} -type f -name '*.flat' -print | /bin/busybox sort > \"$TMPDIR/android-flats\"\nwhile IFS= read -r path; do set -- \"$@\" \"$path\"; done < \"$TMPDIR/android-flats\"\n",
                q(output("compile")?)?));
                script.push_str(&format!("{run} {} link -o \"$out/base.apk\" --manifest {} -I {} --java \"$out/java\" --custom-package {} --rename-manifest-package {} --version-code {} --version-name {} \"$@\"\n",
                q(&format!("{tools}/aapt2"))?, q(&format!("{source}/{}", plan.manifest))?, q(&format!("{platform}/android.jar"))?,
                q(&plan.application_id)?, q(&plan.application_id)?, plan.version_code, q(&plan.version_name)?));
            }
            StepKind::Javac => {
                script.push_str(&format!("/bin/busybox mkdir -p \"$out\"\n/bin/busybox find {} -type f -name '*.java' -print | /bin/busybox sort > \"$TMPDIR/android-generated-java\"\nset --\nwhile IFS= read -r path; do set -- \"$@\" \"$path\"; done < \"$TMPDIR/android-generated-java\"\n",
                q(&format!("{}/java", output("link")?))?));
                let mut sources: Vec<_> = plan.java_sources.iter().map(String::as_str).collect();
                sources.sort_unstable();
                for src in sources {
                    script.push_str(&format!("set -- \"$@\" {}\n", q(&format!("{source}/{src}"))?));
                }
                script.push_str(&format!("{run} {} -J-Duser.language=en -J-Duser.country=US -J-Duser.timezone=UTC -J-Dfile.encoding=UTF-8 -source 8 -target 8 -classpath {} -d \"$out\" \"$@\"\n",
                q(&format!("{jdk}/bin/javac"))?, q(&format!("{platform}/android.jar"))?));
            }
            StepKind::D8 => {
                let java = java()?;
                script.push_str(&format!("/bin/busybox mkdir -p \"$out\"\n/bin/busybox find {} -type f -name '*.class' -print | /bin/busybox sort > \"$TMPDIR/android-classes\"\nset --\nwhile IFS= read -r path; do set -- \"$@\" \"$path\"; done < \"$TMPDIR/android-classes\"\n",
                q(output("javac")?)?));
                script.push_str(&format!(
                    "{java} -cp {} com.android.tools.r8.D8 --lib {} --min-api 23 --output \"$out\" \"$@\"\n",
                    q(&format!("{tools}/lib/d8.jar"))?,
                    q(&format!("{platform}/android.jar"))?
                ));
            }
            StepKind::Zipalign => {
                script.push_str(&format!("/bin/busybox mkdir -p \"$TMPDIR/android-merge\"\n/bin/busybox unzip -q {} -d \"$TMPDIR/android-merge\"\n/bin/busybox find {} -type f -name 'classes*.dex' -print | /bin/busybox sort > \"$TMPDIR/android-dex\"\nwhile IFS= read -r dex; do /bin/busybox cp \"$dex\" \"$TMPDIR/android-merge/\"; done < \"$TMPDIR/android-dex\"\nstamp=$(/bin/busybox date -u -d \"@$SOURCE_DATE_EPOCH\" '+%Y%m%d%H%M.%S')\n/bin/busybox find \"$TMPDIR/android-merge\" -exec /bin/busybox touch -t \"$stamp\" {{}} +\n(cd \"$TMPDIR/android-merge\" && /bin/busybox find . -type f -print | /bin/busybox sort > \"$TMPDIR/android-zip-members\")\nset --\nwhile IFS= read -r member; do set -- \"$@\" \"${{member#./}}\"; done < \"$TMPDIR/android-zip-members\"\n(cd \"$TMPDIR/android-merge\" && {run} {} -J-Duser.language=en -J-Duser.country=US -J-Duser.timezone=UTC -J-Dfile.encoding=UTF-8 --create --file \"$TMPDIR/android-merged.apk\" --no-manifest --no-compress \"$@\")\n{run} {} -f 4 \"$TMPDIR/android-merged.apk\" \"$out\"\n",
                q(&format!("{}/base.apk", output("link")?))?, q(output("d8")?)?,
                q(&format!("{jdk}/bin/jar"))?, q(&format!("{tools}/zipalign"))?));
            }
            StepKind::Apksigner => {
                let java = java()?;
                let signing =
                    plan.signing.as_ref().ok_or_else(|| error("signing stage without signing declaration"))?;
                let flags = ["v1", "v2", "v3"]
                    .iter()
                    .map(|scheme| {
                        format!(
                            "--{scheme}-signing-enabled {}",
                            signing.schemes.iter().any(|candidate| candidate == scheme)
                        )
                    })
                    .collect::<Vec<_>>()
                    .join(" ");
                script.push_str(&format!("{java} -cp {} com.android.apksigner.ApkSignerTool sign --ks {} --ks-key-alias {} --ks-pass {} --key-pass {} {flags} --v4-signing-enabled false --out \"$out\" {}\n",
                q(&format!("{tools}/lib/apksigner.jar"))?, q(&signing.keystore)?, q(&signing.alias)?,
                q(&format!("file:{}", signing.store_password_file))?, q(&format!("file:{}", signing.key_password_file))?,
                q(output("zipalign")?)?));
            }
        }
        Ok(script)
    }

    fn prepare_with_records(
        plan: &ApkPlan,
        inputs: &ApkInputs,
        prefix: &str,
        cohort: &[SourceRecord],
    ) -> Result<ApkDerivations, AdapterError> {
        let steps = validate_and_lower(plan).map_err(|e| error(format!("invalid Android plan: {e:?}")))?;
        if store_object(&inputs.source_root, prefix)? != inputs.source_root {
            return Err(error("source_root must name a complete store object"));
        }
        let glibc_root = store_object(&inputs.runtime_loader, prefix)?;
        let libgcc_root = store_object(&inputs.native_libgcc_root, prefix)?;
        if libgcc_root != inputs.native_libgcc_root
            || libgcc_root == glibc_root
            || inputs.runtime_loader == glibc_root
            || inputs.source_root == glibc_root
            || inputs.source_root == libgcc_root
        {
            return Err(error(
                "Android app, glibc loader and independent libgcc root must name distinct store objects",
            ));
        }
        let [first_path, second_path] = inputs.runtime_library_paths.as_slice() else {
            return Err(error("Android tools require one library directory from each native runtime root"));
        };
        let first_root = store_object(first_path, prefix)?;
        let second_root = store_object(second_path, prefix)?;
        if first_path == &first_root
            || second_path == &second_root
            || !((first_root == glibc_root && second_root == libgcc_root)
                || (first_root == libgcc_root && second_root == glibc_root))
        {
            return Err(error("Android runtime library directories must name distinct glibc and libgcc roots"));
        }
        let runtime_inputs = [glibc_root, libgcc_root];
        if cohort.iter().map(|r| r.component.as_str()).collect::<BTreeSet<_>>().len() != cohort.len() {
            return Err(error("duplicate pinned Android source component"));
        }
        let records: BTreeMap<&str, &SourceRecord> =
            cohort.iter().map(|record| (record.component.as_str(), record)).collect();
        let archives: BTreeMap<_, _> =
            inputs.archives.iter().map(|r| (r.component.as_str(), &r.local_archive)).collect();
        if archives.len() != inputs.archives.len()
            || archives.len() != COMPONENTS.len()
            || !archives.keys().all(|name| COMPONENTS.contains(name))
        {
            return Err(error("exactly one local archive per Android tool component is required"));
        }
        // No fetch, extract, or executable script is generated until *all* three
        // identities and observed archive bytes have passed preflight.
        for (identity, component) in [
            (&plan.toolchains.jdk, "jdk"),
            (&plan.toolchains.build_tools, "build-tools"),
            (&plan.toolchains.platform, "platform-android-jar"),
        ] {
            let record = records
                .get(component)
                .copied()
                .ok_or_else(|| error(format!("missing pinned Android source record: {component}")))?;
            if record.component != component
                || record.platform != "x86_64-linux"
                || !record.url.starts_with("https://")
                || record_identity(record)? != record.record_blake3
                || !matches!(
                    (component, record.unpack_shape.as_str()),
                    ("jdk", "jdk-17.0.17+10/")
                        | ("build-tools", "android-15/")
                        | ("platform-android-jar", "android-35/")
                )
            {
                return Err(error(format!("invalid pinned Android source declaration: {component}")));
            }
            if identity.component != component
                || identity.record_blake3 != record.record_blake3
                || identity.sha256 != record.sha256
            {
                return Err(error(format!("Android plan tool identity differs from pinned cohort: {component}")));
            }
            let archive = archives
                .get(component)
                .ok_or_else(|| error(format!("missing local Android archive: {component}")))?;
            let observed = archive_sha256(archive)?;
            if observed != record.sha256 {
                return Err(error(format!(
                    "Android archive SHA-256 drift for {component}: expected {}, observed {observed}",
                    record.sha256
                )));
            }
        }
        if let Some(signing) = &plan.signing {
            for file in [
                &signing.keystore,
                &signing.store_password_file,
                &signing.key_password_file,
            ] {
                let root = store_object(file, prefix)?;
                if root == *file {
                    return Err(error(format!("signing input must name a file within a store object: {file}")));
                }
            }
        }
        let mut cache = ConversionCache::new(prefix);
        let mut fetches = BTreeMap::new();
        let mut extracts = BTreeMap::new();
        let mut extracted_paths = BTreeMap::new();
        for component in COMPONENTS {
            let record = &records[component];
            let fetched = drv(
                &format!("android-{}-{}", component, record.version),
                "builtin:fetchurl",
                vec![],
                vec![],
                HashMap::from([("url".to_owned(), record.url.clone())]),
                Some(FixedOutput {
                    hash: record.sha256.clone(),
                    algo: "sha256".to_owned(),
                    mode: "flat".to_owned(),
                }),
            );
            let archive_path = output_path(&fetched, &mut cache, prefix)?;
            let mut env = HashMap::from([
                ("SOURCE_DATE_EPOCH".to_owned(), plan.reproducibility.entry_timestamp_epoch.to_string()),
                ("TZ".to_owned(), "UTC".to_owned()),
            ]);
            env.insert("MANTLE_ANDROID_TOOLCHAIN_SHA256".to_owned(), record.sha256.clone());
            env.insert("MANTLE_ANDROID_TOOLCHAIN_RECORD_BLAKE3".to_owned(), record.record_blake3.clone());
            env.insert("MANTLE_ANDROID_TOOLCHAIN_COMPONENT".to_owned(), component.to_owned());
            let command = if component == "jdk" {
                format!(
                    "set -eu\n/bin/busybox mkdir -p \"$out\"\n/bin/busybox tar -xzf {} -C \"$out\"\n",
                    shell(&archive_path)?
                )
            } else {
                format!(
                    "set -eu\n/bin/busybox mkdir -p \"$out\"\n/bin/busybox unzip -q {} -d \"$out\"\n",
                    shell(&archive_path)?
                )
            };
            let extracted = drv(
                &format!("android-unpack-{component}"),
                "/bin/sh",
                vec!["-eu".to_owned(), "-c".to_owned(), command],
                vec![Input::Derivation(Box::new(fetched.clone()))],
                env,
                None,
            );
            extracted_paths.insert(component.to_owned(), format!("{}/", output_path(&extracted, &mut cache, prefix)?));
            fetches.insert(component.to_owned(), fetched);
            extracts.insert(component.to_owned(), extracted);
        }
        for component in COMPONENTS {
            let path = extracted_paths.get_mut(component).expect("all extracts exist");
            path.push_str(records[component].unpack_shape.trim_end_matches('/'));
        }
        let libraries = inputs.runtime_library_paths.join(":");
        let mut prior = BTreeMap::new();
        let mut stages: Vec<CrunchDerivation> = Vec::new();
        for step in steps {
            let mut dependencies = vec![Input::Source(inputs.source_root.clone())];
            dependencies.extend(runtime_inputs.iter().cloned().map(Input::Source));
            let mut stage_paths = BTreeMap::new();
            for component in step.required_toolchains.iter().copied() {
                let record = tool_record(&records, component);
                stage_paths.insert(record.component.clone(), extracted_paths[&record.component].clone());
                dependencies.push(Input::Derivation(Box::new(fetches[&record.component].clone())));
                dependencies.push(Input::Derivation(Box::new(extracts[&record.component].clone())));
            }
            let prior_names: &[&str] = match step.kind {
                StepKind::Aapt2Compile => &[],
                StepKind::Aapt2Link => &["compile"],
                StepKind::Javac => &["link"],
                StepKind::D8 => &["javac"],
                StepKind::Zipalign => &["link", "d8"],
                StepKind::Apksigner => &["zipalign"],
            };
            for name in prior_names {
                dependencies.push(Input::Derivation(Box::new(
                    stages
                        .iter()
                        .find(|d: &&CrunchDerivation| {
                            d.name
                                == step_name(match *name {
                                    "compile" => StepKind::Aapt2Compile,
                                    "link" => StepKind::Aapt2Link,
                                    "javac" => StepKind::Javac,
                                    "d8" => StepKind::D8,
                                    _ => StepKind::Zipalign,
                                })
                        })
                        .expect("validated step order")
                        .clone(),
                )));
            }
            if step.kind == StepKind::Apksigner {
                let signing = plan.signing.as_ref().expect("validated signing stage");
                for path in [
                    &signing.keystore,
                    &signing.store_password_file,
                    &signing.key_password_file,
                ] {
                    dependencies.push(Input::Source(store_object(path, prefix)?));
                }
            }
            let script = stage_script(
                step.kind,
                plan,
                &inputs.source_root,
                &stage_paths,
                &prior,
                &inputs.runtime_loader,
                &libraries,
            )?;
            let mut env = HashMap::from([
                ("SOURCE_DATE_EPOCH".to_owned(), plan.reproducibility.entry_timestamp_epoch.to_string()),
                ("TZ".to_owned(), "UTC".to_owned()),
            ]);
            if let Some(jdk) = stage_paths.get("jdk") {
                env.insert("JAVA_HOME".to_owned(), jdk.clone());
            }
            for component in step.required_toolchains {
                let record = tool_record(&records, component);
                let key = record.component.to_ascii_uppercase().replace('-', "_");
                env.insert(format!("MANTLE_ANDROID_{key}_SHA256"), record.sha256.clone());
                env.insert(format!("MANTLE_ANDROID_{key}_RECORD_BLAKE3"), record.record_blake3.clone());
            }
            let derivation = drv(
                step_name(step.kind),
                "/bin/sh",
                vec!["-eu".to_owned(), "-c".to_owned(), script],
                dependencies,
                env,
                None,
            );
            let path = output_path(&derivation, &mut cache, prefix)?;
            let key = match step.kind {
                StepKind::Aapt2Compile => "compile",
                StepKind::Aapt2Link => "link",
                StepKind::Javac => "javac",
                StepKind::D8 => "d8",
                StepKind::Zipalign => "zipalign",
                StepKind::Apksigner => "apksigner",
            };
            prior.insert(key, path);
            stages.push(derivation);
        }
        let final_derivation = stages.last().ok_or_else(|| error("Android plan has no APK stages"))?.clone();
        let final_output_path = prior[if plan.signing.is_some() {
            "apksigner"
        } else {
            "zipalign"
        }]
        .clone();
        Ok(ApkDerivations {
            stages,
            final_derivation,
            final_output_path,
        })
    }

    #[cfg(test)]
    mod tests {
        use std::io::Write;

        use crunch_android_core::ReproducibilityPolicy;
        use crunch_android_core::SigningConfig;
        use crunch_android_core::ToolchainIdentity;
        use crunch_android_core::ToolchainRefs;

        use super::*;

        const PREFIX: &str = "/nix/store";
        const ROOT: &str = "/nix/store/0123456789abcdfghijklmnpqrsvwxyz-android-sources";
        const SIGNING: &str = "/nix/store/0123456789abcdfghijklmnpqrsvwxyz-release";

        fn fixture() -> (tempfile::TempDir, ApkPlan, ApkInputs, Vec<SourceRecord>) {
            let dir = tempfile::tempdir().unwrap();
            let mut records = Vec::new();
            let mut archives = Vec::new();
            for (component, unpack) in [
                ("jdk", "jdk-17.0.17+10/"),
                ("build-tools", "android-15/"),
                ("platform-android-jar", "android-35/"),
            ] {
                let local_archive = dir.path().join(component);
                let mut file = File::create(&local_archive).unwrap();
                file.write_all(format!("archive bytes for {component}").as_bytes()).unwrap();
                let mut record = SourceRecord {
                    component: component.into(),
                    version: "synthetic".into(),
                    url: format!("https://example.invalid/{component}"),
                    sha256: archive_sha256(&local_archive).unwrap(),
                    record_blake3: String::new(),
                    unpack_shape: unpack.into(),
                    platform: "x86_64-linux".into(),
                };
                record.record_blake3 = record_identity(&record).unwrap();
                records.push(record);
                archives.push(ArchiveInput {
                    component: component.into(),
                    local_archive,
                });
            }
            let identity = |name: &str| {
                let record = records.iter().find(|record| record.component == name).unwrap();
                ToolchainIdentity {
                    component: name.into(),
                    sha256: record.sha256.clone(),
                    record_blake3: record.record_blake3.clone(),
                }
            };
            let plan = ApkPlan {
                module: "app".into(),
                application_id: "org.example.app".into(),
                version_code: 7,
                version_name: "1.0".into(),
                manifest: "app/AndroidManifest.xml".into(),
                resources: vec!["app/res/values/strings.xml".into(), "app/res/drawable/icon.xml".into()],
                java_sources: vec!["app/src/Entry.java".into()],
                toolchains: ToolchainRefs {
                    jdk: identity("jdk"),
                    build_tools: identity("build-tools"),
                    platform: identity("platform-android-jar"),
                },
                signing: None,
                reproducibility: ReproducibilityPolicy {
                    entry_timestamp_epoch: 315_532_800,
                    locale: "C".into(),
                    timezone: "UTC".into(),
                },
            };
            let inputs = ApkInputs {
                source_root: ROOT.into(),
                archives,
                runtime_loader: "/nix/store/0123456789abcdfghijklmnpqrsvwxyz-loader/lib/ld-linux-x86-64.so.2".into(),
                native_libgcc_root: "/nix/store/1123456789abcdfghijklmnpqrsvwxyz-libgcc".into(),
                runtime_library_paths: vec![
                    "/nix/store/0123456789abcdfghijklmnpqrsvwxyz-loader/lib".into(),
                    "/nix/store/1123456789abcdfghijklmnpqrsvwxyz-libgcc/lib".into(),
                ],
            };
            (dir, plan, inputs, records)
        }

        #[test]
        fn byte_drift_rejects_entire_graph_before_generating_any_stage() {
            let (_dir, plan, inputs, records) = fixture();
            std::fs::write(&inputs.archives[2].local_archive, b"modified platform archive").unwrap();
            let rejection = prepare_with_records(&plan, &inputs, PREFIX, &records).unwrap_err().to_string();
            assert!(rejection.contains("SHA-256 drift for platform-android-jar"), "{rejection}");
            let mut forged = plan.clone();
            forged.toolchains.jdk.record_blake3 = "b".repeat(64);
            let rejection = prepare_with_records(&forged, &inputs, PREFIX, &records).unwrap_err().to_string();
            assert!(rejection.contains("identity differs from pinned cohort"), "{rejection}");
        }

        #[test]
        fn final_output_identity_is_stable_and_changes_with_apk_inputs() {
            let (_dir, mut plan, inputs, records) = fixture();
            let unsigned = prepare_with_records(&plan, &inputs, PREFIX, &records).unwrap();
            let replay = prepare_with_records(&plan, &inputs, PREFIX, &records).unwrap();
            assert_eq!(unsigned.final_output_path, replay.final_output_path);

            plan.resources.reverse();
            let reordered = prepare_with_records(&plan, &inputs, PREFIX, &records).unwrap();
            assert_eq!(unsigned.final_output_path, reordered.final_output_path);

            plan.resources[0] = "app/res/values/changed.xml".into();
            let changed_source = prepare_with_records(&plan, &inputs, PREFIX, &records).unwrap();
            assert_ne!(unsigned.final_output_path, changed_source.final_output_path);

            plan.signing = Some(SigningConfig {
                keystore: format!("{SIGNING}/release.jks"),
                store_password_file: format!("{SIGNING}/store.pass"),
                key_password_file: format!("{SIGNING}/key.pass"),
                alias: "release".into(),
                schemes: vec!["v2".into(), "v3".into()],
            });
            let signed = prepare_with_records(&plan, &inputs, PREFIX, &records).unwrap();
            assert_ne!(changed_source.final_output_path, signed.final_output_path);

            plan.signing.as_mut().unwrap().alias = "next-release".into();
            let next_signing_identity = prepare_with_records(&plan, &inputs, PREFIX, &records).unwrap();
            assert_ne!(signed.final_output_path, next_signing_identity.final_output_path);
        }

        #[test]
        fn every_sdk_stage_binds_distinct_glibc_and_libgcc_source_objects() {
            let (_dir, plan, mut inputs, records) = fixture();
            let original = prepare_with_records(&plan, &inputs, PREFIX, &records).unwrap();
            let glibc = store_object(&inputs.runtime_loader, PREFIX).unwrap();
            for stage in &original.stages {
                assert!(
                    stage.inputs.iter().any(|input| matches!(input, Input::Source(source) if source == &glibc)),
                    "{} missing glibc",
                    stage.name
                );
                assert!(
                    stage
                        .inputs
                        .iter()
                        .any(|input| matches!(input, Input::Source(source) if source == &inputs.native_libgcc_root)),
                    "{} missing libgcc",
                    stage.name
                );
            }
            inputs.native_libgcc_root = "/nix/store/2123456789abcdfghijklmnpqrsvwxyz-libgcc".into();
            inputs.runtime_library_paths[1] = format!("{}/lib", inputs.native_libgcc_root);
            let changed = prepare_with_records(&plan, &inputs, PREFIX, &records).unwrap();
            assert_ne!(original.final_output_path, changed.final_output_path);
            inputs.native_libgcc_root = glibc.clone();
            assert!(prepare_with_records(&plan, &inputs, PREFIX, &records).is_err());
            inputs.native_libgcc_root = "/usr/lib/libgcc".into();
            assert!(prepare_with_records(&plan, &inputs, PREFIX, &records).is_err());
        }

        #[test]
        fn signed_apk_graph_is_accepted_by_strict_build_environment() {
            fn check(derivation: &CrunchDerivation) {
                for key in derivation.env.keys() {
                    assert!(
                        crunch_build::classify_denied_environment_variable(key).is_none(),
                        "{} cannot build in strict mode because of {key}",
                        derivation.name
                    );
                }
                for input in &derivation.inputs {
                    match input {
                        Input::Derivation(child) => check(child),
                        Input::OutputSelection(output) => check(&output.drv),
                        _ => {}
                    }
                }
            }

            let locale = crunch_build::classify_denied_environment_variable("LC_ALL").unwrap();
            assert_eq!(locale.class, crunch_build::ENV_REJECTION_LOCALE);
            let (_dir, mut plan, inputs, records) = fixture();
            plan.signing = Some(SigningConfig {
                keystore: format!("{SIGNING}/release.jks"),
                store_password_file: format!("{SIGNING}/store.pass"),
                key_password_file: format!("{SIGNING}/key.pass"),
                alias: "release".into(),
                schemes: vec!["v1".into(), "v2".into(), "v3".into()],
            });
            let graph = prepare_with_records(&plan, &inputs, PREFIX, &records).unwrap();
            check(&graph.final_derivation);
        }

        #[test]
        fn declared_store_namespace_rejects_ambient_loader_and_foreign_signing() {
            let (_dir, mut plan, mut inputs, records) = fixture();
            let declared_loader = inputs.runtime_loader.clone();
            inputs.runtime_loader = "/usr/lib/ld-linux.so".into();
            assert!(
                prepare_with_records(&plan, &inputs, PREFIX, &records)
                    .unwrap_err()
                    .to_string()
                    .contains("outside configured store prefix")
            );
            inputs.runtime_loader = format!("{ROOT}/ld.so");
            assert!(
                prepare_with_records(&plan, &inputs, PREFIX, &records)
                    .unwrap_err()
                    .to_string()
                    .contains("distinct store objects")
            );
            inputs.runtime_loader = declared_loader;
            plan.signing = Some(SigningConfig {
                keystore: "/nix/store/0123456789abcdfghijklmnpqrsvwxyz-signing/key.jks".into(),
                store_password_file: "/nix/store/0123456789abcdfghijklmnpqrsvwxyz-signing/store.pass".into(),
                key_password_file: "/nix/store/0123456789abcdfghijklmnpqrsvwxyz-signing/key.pass".into(),
                alias: "release".into(),
                schemes: vec!["v1".into()],
            });
            plan.signing.as_mut().unwrap().keystore =
                "/alternate/store/0123456789abcdfghijklmnpqrsvwxyz-signing/key.jks".into();
            assert!(
                prepare_with_records(&plan, &inputs, PREFIX, &records)
                    .unwrap_err()
                    .to_string()
                    .contains("outside configured store prefix")
            );
        }

        #[test]
        fn malformed_unicode_store_objects_are_rejected_without_panicking() {
            let split_multibyte = format!("{PREFIX}/{}é-release/key.jks", "a".repeat(31));
            assert!(store_object(&split_multibyte, PREFIX).is_err());
            let inside_name = format!("{PREFIX}/0123456789abcdfghijklmnpqrsvwxyz-rélease/key.jks");
            assert!(store_object(&inside_name, PREFIX).is_err());
            assert!(store_object(ROOT, "/nix/störé").is_err());
        }
    }
}

pub use pinned::AdapterError;
pub use pinned::ApkDerivations;
pub use pinned::ApkInputs;
pub use pinned::ArchiveInput;
pub use pinned::prepare_apk;
