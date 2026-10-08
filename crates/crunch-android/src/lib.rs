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
use crunch_android_core::ApkPlan;
use crunch_android_core::LoweredApk;
use crunch_android_core::Rejection;
use crunch_android_core::SourceIdentity;
use crunch_android_core::StepKind;
use crunch_android_core::Toolchain;
use crunch_android_core::lower;
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
        if plan.signing.is_some() {
            let keystore = inputs.keystore.as_ref().ok_or_else(|| Error::InvalidInput("keystore missing".into()))?;
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

    pub fn steps(&self) -> &[crunch_android_core::StepPlan] {
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
