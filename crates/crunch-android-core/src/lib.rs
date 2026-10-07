#![no_std]

//! Pure validation and lowering of a declared Android APK plan. These records
//! name pinned third-party tool sources; verifying or executing them is the
//! responsibility of the std-facing adapter, not this crate.

extern crate alloc;

use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;

use serde::Deserialize;
use serde::Serialize;

/// Serializable authoring contract. `manifest`, `resources`, and
/// `java_sources` are declared source-tree members, not host filesystem paths.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ApkPlan {
    pub module: String,
    pub application_id: String,
    pub version_code: u32,
    pub version_name: String,
    pub manifest: String,
    pub resources: Vec<String>,
    pub java_sources: Vec<String>,
    pub toolchains: ToolchainRefs,
    pub signing: Option<SigningConfig>,
    pub reproducibility: ReproducibilityPolicy,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ToolchainRefs {
    pub jdk: ToolchainIdentity,
    pub build_tools: ToolchainIdentity,
    pub platform: ToolchainIdentity,
}

/// A declaration of source-record and downloaded-byte identities, not proof
/// that the installed executable was built from source or has been verified.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ToolchainIdentity {
    pub component: String,
    pub record_blake3: String,
    pub sha256: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SigningConfig {
    pub keystore: String,
    pub store_password_file: String,
    pub key_password_file: String,
    pub alias: String,
    pub schemes: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReproducibilityPolicy {
    pub entry_timestamp_epoch: u64,
    pub locale: String,
    pub timezone: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StepKind {
    Aapt2Compile,
    Aapt2Link,
    Javac,
    D8,
    Zipalign,
    Apksigner,
}

/// Selects a record in `ApkPlan::toolchains`. The adapter must bind each
/// referenced artifact (including JDK jar and Android platform jar) itself.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ToolchainComponent {
    Jdk,
    BuildTools,
    Platform,
}

/// Logical output of a step. These are dependencies, not guessed store paths.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StepOutput {
    CompiledResources,
    LinkedApk,
    GeneratedJava,
    JavaClasses,
    Dex,
    AlignedApk,
    SignedApk,
}

/// `source_inputs` are declared source-tree members or store-shaped signing
/// inputs. `prior_outputs` name all step-produced dependencies. Artifact
/// placement, tool invocation, and sandboxing belong to the adapter.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct StepPlan {
    pub kind: StepKind,
    pub source_inputs: Vec<String>,
    pub required_toolchains: Vec<ToolchainComponent>,
    pub prior_outputs: Vec<StepOutput>,
    pub outputs: Vec<StepOutput>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PlanRejection {
    MissingManifestMember,
    UnboundToolchainIdentity,
    UnpinnedTimestampPolicy,
    KeystorePathEscape,
    UnknownSignatureScheme,
    EmptySourceSet,
}

fn valid_record_blake3(value: &str) -> bool {
    value.len() == 64
        && value.bytes().all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
        && value.bytes().any(|byte| byte != b'0')
}

fn valid_sha256(value: &str) -> bool {
    let Some(digest) = value.strip_prefix("sha256-") else {
        return false;
    };
    digest.len() == 44
        && digest.as_bytes()[43] == b'='
        && digest.as_bytes()[..43]
            .iter()
            .all(|byte| byte.is_ascii_alphanumeric() || *byte == b'+' || *byte == b'/')
        && digest.as_bytes()[..43].iter().any(|byte| *byte != b'A')
}

fn bound_record(identity: &ToolchainIdentity, component: &str) -> bool {
    identity.component == component && valid_record_blake3(&identity.record_blake3) && valid_sha256(&identity.sha256)
}

// Require a normalized, relative source member. Neither lexical `..` nor an
// absolute/drive-qualified path may escape the adapter's declared source tree.
fn safe_member(member: &str) -> bool {
    !member.is_empty()
        && !member.starts_with('/')
        && !member.contains('\\')
        && !member.contains(':')
        && member.split('/').all(|part| !part.is_empty() && part != "." && part != "..")
}

// This only checks lexical store-path shape. The std adapter must verify that
// the input belongs to its exact configured store prefix and was declared.
fn safe_store_input(input: &str) -> bool {
    let Some(rest) = input.strip_prefix('/') else {
        return false;
    };
    let mut parts = rest.split('/');
    let Some(first) = parts.next() else {
        return false;
    };
    if !safe_store_part(first) || matches!(first, "etc" | "run") {
        return false;
    }
    let mut root_found = false;
    for part in parts {
        if !safe_store_part(part) {
            return false;
        }
        if root_found {
            continue;
        }
        if let Some((hash, name)) = part.split_once('-') {
            if hash.len() == 32
                && hash.bytes().all(|byte| b"0123456789abcdfghijklmnpqrsvwxyz".contains(&byte))
                && !name.is_empty()
                && !name.starts_with('.')
            {
                root_found = true;
            }
        }
    }
    root_found
}

fn safe_store_part(part: &str) -> bool {
    !part.is_empty()
        && part != "."
        && part != ".."
        && part.bytes().all(|byte| byte.is_ascii_alphanumeric() || b"+._?=-".contains(&byte))
}

/// Rejects malformed declarations before constructing a dependency-ordered
/// plan. No toolchain binding, byte inspection, clock read, or execution occurs.
pub fn validate_and_lower(plan: &ApkPlan) -> Result<Vec<StepPlan>, PlanRejection> {
    if !safe_member(&plan.manifest) {
        return Err(PlanRejection::MissingManifestMember);
    }
    if plan.resources.is_empty()
        || plan.java_sources.is_empty()
        || !plan.resources.iter().all(|member| safe_member(member))
        || !plan.java_sources.iter().all(|member| safe_member(member))
    {
        return Err(PlanRejection::EmptySourceSet);
    }
    if !bound_record(&plan.toolchains.jdk, "jdk")
        || !bound_record(&plan.toolchains.build_tools, "build-tools")
        || !bound_record(&plan.toolchains.platform, "platform-android-jar")
    {
        return Err(PlanRejection::UnboundToolchainIdentity);
    }
    if plan.reproducibility.entry_timestamp_epoch < 315_532_800
        || plan.reproducibility.entry_timestamp_epoch > 4_354_819_199
        || plan.reproducibility.locale != "C"
        || plan.reproducibility.timezone != "UTC"
    {
        return Err(PlanRejection::UnpinnedTimestampPolicy);
    }
    if let Some(signing) = &plan.signing {
        if !safe_store_input(&signing.keystore)
            || !safe_store_input(&signing.store_password_file)
            || !safe_store_input(&signing.key_password_file)
        {
            return Err(PlanRejection::KeystorePathEscape);
        }
        if signing.alias.is_empty()
            || signing.schemes.is_empty()
            || signing.schemes.iter().enumerate().any(|(index, scheme)| {
                !matches!(scheme.as_str(), "v1" | "v2" | "v3") || signing.schemes[..index].contains(scheme)
            })
        {
            return Err(PlanRejection::UnknownSignatureScheme);
        }
    }

    let mut steps = vec![
        StepPlan {
            kind: StepKind::Aapt2Compile,
            source_inputs: plan.resources.clone(),
            required_toolchains: vec![ToolchainComponent::BuildTools],
            prior_outputs: vec![],
            outputs: vec![StepOutput::CompiledResources],
        },
        StepPlan {
            kind: StepKind::Aapt2Link,
            source_inputs: vec![plan.manifest.clone()],
            required_toolchains: vec![ToolchainComponent::BuildTools, ToolchainComponent::Platform],
            prior_outputs: vec![StepOutput::CompiledResources],
            outputs: vec![StepOutput::LinkedApk, StepOutput::GeneratedJava],
        },
        StepPlan {
            kind: StepKind::Javac,
            source_inputs: plan.java_sources.clone(),
            required_toolchains: vec![ToolchainComponent::Jdk, ToolchainComponent::Platform],
            prior_outputs: vec![StepOutput::GeneratedJava],
            outputs: vec![StepOutput::JavaClasses],
        },
        StepPlan {
            kind: StepKind::D8,
            source_inputs: vec![],
            required_toolchains: vec![
                ToolchainComponent::BuildTools,
                ToolchainComponent::Jdk,
                ToolchainComponent::Platform,
            ],
            prior_outputs: vec![StepOutput::JavaClasses],
            outputs: vec![StepOutput::Dex],
        },
        StepPlan {
            kind: StepKind::Zipalign,
            source_inputs: vec![],
            required_toolchains: vec![
                ToolchainComponent::BuildTools,
                ToolchainComponent::Jdk,
                ToolchainComponent::Platform,
            ],
            prior_outputs: vec![StepOutput::LinkedApk, StepOutput::Dex],
            outputs: vec![StepOutput::AlignedApk],
        },
    ];
    if let Some(signing) = &plan.signing {
        steps.push(StepPlan {
            kind: StepKind::Apksigner,
            source_inputs: vec![
                signing.keystore.clone(),
                signing.store_password_file.clone(),
                signing.key_password_file.clone(),
            ],
            required_toolchains: vec![ToolchainComponent::BuildTools, ToolchainComponent::Jdk],
            prior_outputs: vec![StepOutput::AlignedApk],
            outputs: vec![StepOutput::SignedApk],
        });
    }
    Ok(steps)
}

#[cfg(test)]
mod tests {
    use alloc::string::ToString;

    use super::*;

    fn toolchain(component: &str) -> ToolchainIdentity {
        ToolchainIdentity {
            component: component.to_string(),
            record_blake3: "a".repeat(64),
            sha256: "sha256-mS+W55lQdax2NrsajeUrDGHXHtMTf6/JeauWtKt43XU=".to_string(),
        }
    }

    fn fixture() -> ApkPlan {
        ApkPlan {
            module: "demo".to_string(),
            application_id: "org.example.demo".to_string(),
            version_code: 1,
            version_name: "1.0".to_string(),
            manifest: "app/AndroidManifest.xml".to_string(),
            resources: vec!["app/res/values/strings.xml".to_string()],
            java_sources: vec!["app/src/Main.java".to_string()],
            toolchains: ToolchainRefs {
                jdk: toolchain("jdk"),
                build_tools: toolchain("build-tools"),
                platform: toolchain("platform-android-jar"),
            },
            signing: None,
            reproducibility: ReproducibilityPolicy {
                entry_timestamp_epoch: 315_532_800,
                locale: "C".to_string(),
                timezone: "UTC".to_string(),
            },
        }
    }

    fn signing() -> SigningConfig {
        SigningConfig {
            keystore: "/nix/store/0123456789abcdfghijklmnpqrsvwxyz-signing/release.jks".to_string(),
            store_password_file: "/nix/store/0123456789abcdfghijklmnpqrsvwxyz-signing/store.pass".to_string(),
            key_password_file: "/nix/store/0123456789abcdfghijklmnpqrsvwxyz-signing/key.pass".to_string(),
            alias: "release".to_string(),
            schemes: vec!["v2".to_string(), "v3".to_string()],
        }
    }

    #[test]
    fn unsigned_steps_require_ordered_predecessors_and_toolchains() {
        let plan = fixture();
        let steps = validate_and_lower(&plan).unwrap();
        assert_eq!(steps.iter().map(|step| step.kind).collect::<Vec<_>>(), vec![
            StepKind::Aapt2Compile,
            StepKind::Aapt2Link,
            StepKind::Javac,
            StepKind::D8,
            StepKind::Zipalign,
        ]);
        assert_eq!(steps[1].prior_outputs, vec![StepOutput::CompiledResources]);
        assert_eq!(steps[2].prior_outputs, vec![StepOutput::GeneratedJava]);
        assert_eq!(steps[3].prior_outputs, vec![StepOutput::JavaClasses]);
        assert_eq!(steps[4].prior_outputs, vec![StepOutput::LinkedApk, StepOutput::Dex]);
        assert_eq!(steps[4].outputs, vec![StepOutput::AlignedApk]);
        for step in &steps[3..=4] {
            assert_eq!(step.required_toolchains, vec![
                ToolchainComponent::BuildTools,
                ToolchainComponent::Jdk,
                ToolchainComponent::Platform,
            ]);
        }
    }

    #[test]
    fn signed_step_requires_aligned_apk_and_declared_toolchains() {
        let mut plan = fixture();
        plan.signing = Some(signing());
        let steps = validate_and_lower(&plan).unwrap();
        assert_eq!(steps.iter().map(|step| step.kind).collect::<Vec<_>>(), vec![
            StepKind::Aapt2Compile,
            StepKind::Aapt2Link,
            StepKind::Javac,
            StepKind::D8,
            StepKind::Zipalign,
            StepKind::Apksigner,
        ]);
        assert_eq!(steps[5].prior_outputs, vec![StepOutput::AlignedApk]);
        assert_eq!(steps[5].outputs, vec![StepOutput::SignedApk]);
        assert_eq!(steps[5].required_toolchains, vec![ToolchainComponent::BuildTools, ToolchainComponent::Jdk]);
    }

    #[test]
    fn accepts_configurable_absolute_store_prefix() {
        let mut plan = fixture();
        let mut config = signing();
        config.keystore = "/crunch/store/0123456789abcdfghijklmnpqrsvwxyz-signing/release.jks".to_string();
        config.store_password_file = "/crunch/store/0123456789abcdfghijklmnpqrsvwxyz-signing/store.pass".to_string();
        config.key_password_file = "/crunch/store/0123456789abcdfghijklmnpqrsvwxyz-signing/key.pass".to_string();
        plan.signing = Some(config);
        let steps = validate_and_lower(&plan).unwrap();
        assert_eq!(steps[5].kind, StepKind::Apksigner);
    }

    #[test]
    fn missing_or_unsafe_manifest_is_rejected() {
        for bad in [
            "",
            "/tmp/AndroidManifest.xml",
            "../AndroidManifest.xml",
            "app//AndroidManifest.xml",
        ] {
            let mut plan = fixture();
            plan.manifest = bad.to_string();
            assert_eq!(validate_and_lower(&plan), Err(PlanRejection::MissingManifestMember));
        }
    }

    #[test]
    fn missing_or_unusable_source_set_is_rejected() {
        let mut plan = fixture();
        plan.resources.clear();
        assert_eq!(validate_and_lower(&plan), Err(PlanRejection::EmptySourceSet));
        let mut plan = fixture();
        plan.java_sources.clear();
        assert_eq!(validate_and_lower(&plan), Err(PlanRejection::EmptySourceSet));
        let mut plan = fixture();
        plan.java_sources[0] = "../Main.java".to_string();
        assert_eq!(validate_and_lower(&plan), Err(PlanRejection::EmptySourceSet));
    }

    #[test]
    fn malformed_or_misassigned_toolchain_records_are_not_bound() {
        let mut plan = fixture();
        plan.toolchains.platform.component = "build-tools".to_string();
        assert_eq!(validate_and_lower(&plan), Err(PlanRejection::UnboundToolchainIdentity));
        let mut plan = fixture();
        plan.toolchains.jdk.record_blake3 = "0".repeat(64);
        assert_eq!(validate_and_lower(&plan), Err(PlanRejection::UnboundToolchainIdentity));
        let mut plan = fixture();
        plan.toolchains.build_tools.sha256 = "sha256-AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=".to_string();
        assert_eq!(validate_and_lower(&plan), Err(PlanRejection::UnboundToolchainIdentity));
        let mut plan = fixture();
        plan.toolchains.platform.sha256 = "not-a-digest".to_string();
        assert_eq!(validate_and_lower(&plan), Err(PlanRejection::UnboundToolchainIdentity));
    }

    #[test]
    fn reproducibility_policy_enforces_zip_epoch_and_fixed_locale_timezone() {
        let mut plan = fixture();
        plan.reproducibility.entry_timestamp_epoch -= 1;
        assert_eq!(validate_and_lower(&plan), Err(PlanRejection::UnpinnedTimestampPolicy));
        let mut plan = fixture();
        plan.reproducibility.entry_timestamp_epoch = 4_354_819_200;
        assert_eq!(validate_and_lower(&plan), Err(PlanRejection::UnpinnedTimestampPolicy));
        let mut plan = fixture();
        plan.reproducibility.entry_timestamp_epoch = 4_354_819_199;
        assert!(validate_and_lower(&plan).is_ok());
        let mut plan = fixture();
        plan.reproducibility.locale = "en_US.UTF-8".to_string();
        assert_eq!(validate_and_lower(&plan), Err(PlanRejection::UnpinnedTimestampPolicy));
        let mut plan = fixture();
        plan.reproducibility.timezone = "GMT".to_string();
        assert_eq!(validate_and_lower(&plan), Err(PlanRejection::UnpinnedTimestampPolicy));
        assert!(validate_and_lower(&fixture()).is_ok());
    }

    #[test]
    fn every_signing_file_must_be_store_shaped_and_confined() {
        for bad in [
            "../release.jks",
            "/etc/keys.jks",
            "/run/keys.jks",
            "/etc/0123456789abcdfghijklmnpqrsvwxyz-signing/key.jks",
            "/run/secrets/0123456789abcdfghijklmnpqrsvwxyz-signing/key.jks",
            "file:///nix/store/keys",
            "C:/keys.jks",
            "secrets\\keys.jks",
            "secrets//key.jks",
            "/nix/store/short-signing/key.jks",
            "/nix/store/0123456789abcdfghijklmnpqrsvwxyz-signing/../key.jks",
            "/nix/store/0123456789abcdfghijklmnpqrsvwxyz-signing/./key.jks",
            "/nix/store/0123456789abcdfghijklmnpqrsvwxyz-signing/~/key.jks",
            "/nix/store/0123456789abcdfghijklmnpqrsvwxyz-signing/",
        ] {
            for field in 0..3 {
                let mut plan = fixture();
                let mut config = signing();
                let member = match field {
                    0 => &mut config.keystore,
                    1 => &mut config.store_password_file,
                    _ => &mut config.key_password_file,
                };
                *member = bad.to_string();
                plan.signing = Some(config);
                assert_eq!(validate_and_lower(&plan), Err(PlanRejection::KeystorePathEscape));
            }
        }
    }

    #[test]
    fn only_unique_known_signing_schemes_are_allowed() {
        for schemes in [vec![], vec!["v4"], vec!["V2"], vec!["v1", "v1"]] {
            let mut plan = fixture();
            let mut config = signing();
            config.schemes = schemes.into_iter().map(str::to_string).collect();
            plan.signing = Some(config);
            assert_eq!(validate_and_lower(&plan), Err(PlanRejection::UnknownSignatureScheme));
        }
        let mut plan = fixture();
        let mut config = signing();
        config.schemes = vec!["v1".to_string(), "v2".to_string(), "v3".to_string()];
        plan.signing = Some(config);
        assert_eq!(validate_and_lower(&plan).unwrap().last().unwrap().kind, StepKind::Apksigner);
    }
}
