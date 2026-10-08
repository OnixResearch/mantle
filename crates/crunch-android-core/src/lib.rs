#![no_std]
extern crate alloc;

use alloc::string::String;
use alloc::vec::Vec;

use serde::Deserialize;
use serde::Serialize;

pub const MAX_MEMBERS: usize = 256;
pub const MAX_PATH_BYTES: usize = 1024;
pub const MAX_NAME_BYTES: usize = 128;
pub const MIN_ZIP_EPOCH: u64 = 315_532_800;
pub const MAX_ZIP_EPOCH: u64 = 4_354_819_199;

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct UnpackShape {
    pub archive: String,
    pub root: String,
}

/// Matches the reviewed `lib/android/sources.ncl` source-record wire shape.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct SourceIdentity {
    pub component: String,
    pub version: String,
    pub url: String,
    pub sha256: String,
    pub record_identity_blake3: String,
    pub unpack_shape: UnpackShape,
    pub platform: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Toolchain {
    pub build_tools: SourceIdentity,
    pub jdk: SourceIdentity,
    pub platform: SourceIdentity,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Signing {
    pub keystore_member: String,
    pub key_alias: String,
    pub schemes: Vec<String>,
    // Passwords are never part of the plan: the signing executable receives a
    // separately declared password file via the adapter's explicit input.
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Reproducibility {
    pub entry_timestamp_epoch: u64,
    pub locale: String,
    pub timezone: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ApkPlan {
    pub module: String,
    pub application_id: String,
    pub version_code: u32,
    pub manifest_member: String,
    pub resources: Vec<String>,
    pub java_sources: Vec<String>,
    pub toolchain: Toolchain,
    pub signing: Option<Signing>,
    pub reproducibility: Reproducibility,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Rejection {
    InvalidModule,
    InvalidApplicationId,
    InvalidVersion,
    MissingManifestMember,
    UnboundToolchainIdentity,
    UnpinnedTimestampPolicy,
    KeystorePathEscape,
    UnknownSignatureScheme,
    EmptySourceSet,
    InvalidSourceMember,
    TooManyMembers,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StepKind {
    Aapt2Compile,
    Aapt2Link,
    Javac,
    D8,
    Zipalign,
    Apksigner,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum StepInput {
    SourceMember(String),
    ToolchainIdentity(SourceIdentity),
    PreviousOutput(StepKind),
    KeystoreMember(String),
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct StepPlan {
    pub kind: StepKind,
    pub inputs: Vec<StepInput>,
    pub toolchain_identity: SourceIdentity,
    pub output: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct LoweredApk {
    pub steps: Vec<StepPlan>,
    pub final_output: StepKind,
}

fn safe_member(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= MAX_PATH_BYTES
        && value.split('/').all(|part| {
            !part.is_empty()
                && part != "."
                && part != ".."
                && part.bytes().all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b'-' | b'+'))
        })
}

fn identity_bound(identity: &SourceIdentity, component: &str) -> bool {
    identity.component == component
        && (1..=64).contains(&identity.version.len())
        && identity.version.bytes().next().is_some_and(|byte| byte.is_ascii_digit())
        && identity
            .version
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'+' | b'_' | b'-'))
        && (identity.url.starts_with("https://") || identity.url.starts_with("file:///"))
        && identity.url.len() <= MAX_PATH_BYTES
        && identity.sha256.len() == 64
        && identity.sha256.bytes().any(|byte| byte != b'0')
        && identity.sha256.bytes().all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
        && identity.record_identity_blake3.len() == 64
        && identity.record_identity_blake3.bytes().any(|byte| byte != b'0')
        && identity
            .record_identity_blake3
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
        && (identity.unpack_shape.archive == "zip" || identity.unpack_shape.archive == "tar-gz")
        && identity.unpack_shape.root.len() <= 256
        && identity.unpack_shape.root.ends_with('/')
        && safe_member(&identity.unpack_shape.root[..identity.unpack_shape.root.len() - 1])
        && identity.platform == "x86_64-linux"
}

fn validate(plan: &ApkPlan) -> Result<(), Rejection> {
    if plan.module.is_empty()
        || plan.module.len() > MAX_NAME_BYTES
        || !plan.module.bytes().all(|byte| byte.is_ascii_alphanumeric() || byte == b'-' || byte == b'_')
    {
        return Err(Rejection::InvalidModule);
    }
    if plan.application_id.len() > MAX_NAME_BYTES
        || plan.application_id.split('.').count() < 2
        || !plan
            .application_id
            .split('.')
            .all(|part| !part.is_empty() && part.bytes().all(|byte| byte.is_ascii_alphanumeric() || byte == b'_'))
    {
        return Err(Rejection::InvalidApplicationId);
    }
    if plan.version_code == 0 {
        return Err(Rejection::InvalidVersion);
    }
    if !safe_member(&plan.manifest_member) {
        return Err(Rejection::MissingManifestMember);
    }
    if plan.resources.is_empty() || plan.java_sources.is_empty() {
        return Err(Rejection::EmptySourceSet);
    }
    if plan.resources.len() > MAX_MEMBERS || plan.java_sources.len() > MAX_MEMBERS {
        return Err(Rejection::TooManyMembers);
    }
    if !plan.resources.iter().chain(plan.java_sources.iter()).all(|member| safe_member(member)) {
        return Err(Rejection::InvalidSourceMember);
    }
    if !identity_bound(&plan.toolchain.build_tools, "android-build-tools")
        || !identity_bound(&plan.toolchain.jdk, "temurin-jdk")
        || !identity_bound(&plan.toolchain.platform, "android-platform")
    {
        return Err(Rejection::UnboundToolchainIdentity);
    }
    let policy = &plan.reproducibility;
    if !(MIN_ZIP_EPOCH..=MAX_ZIP_EPOCH).contains(&policy.entry_timestamp_epoch)
        || policy.locale != "C"
        || policy.timezone != "UTC"
    {
        return Err(Rejection::UnpinnedTimestampPolicy);
    }
    if let Some(signing) = &plan.signing {
        if !safe_member(&signing.keystore_member) || !signing.keystore_member.starts_with("keys/") {
            return Err(Rejection::KeystorePathEscape);
        }
        if signing.key_alias.is_empty()
            || signing.key_alias.len() > MAX_NAME_BYTES
            || !signing.key_alias.bytes().all(|byte| byte.is_ascii_alphanumeric() || byte == b'_' || byte == b'-')
        {
            return Err(Rejection::KeystorePathEscape);
        }
        if signing.schemes.is_empty()
            || signing.schemes.len() > 3
            || !signing.schemes.iter().all(|scheme| matches!(scheme.as_str(), "v1" | "v2" | "v3"))
        {
            return Err(Rejection::UnknownSignatureScheme);
        }
    }
    Ok(())
}

// r[impl android_adapter.apk_plan_functional_core]
pub fn lower(plan: &ApkPlan) -> Result<LoweredApk, Rejection> {
    validate(plan)?;
    let tools = &plan.toolchain.build_tools;
    let platform = &plan.toolchain.platform;
    let mut resources = plan.resources.clone();
    resources.sort_unstable();
    resources.dedup();
    let mut java = plan.java_sources.clone();
    java.sort_unstable();
    java.dedup();
    let mut compile_inputs: Vec<_> = resources.into_iter().map(StepInput::SourceMember).collect();
    compile_inputs.push(StepInput::ToolchainIdentity(tools.clone()));
    let mut steps = alloc::vec![
        StepPlan {
            kind: StepKind::Aapt2Compile,
            inputs: compile_inputs,
            toolchain_identity: tools.clone(),
            output: "compiled-resources".into()
        },
        StepPlan {
            kind: StepKind::Aapt2Link,
            inputs: alloc::vec![
                StepInput::SourceMember(plan.manifest_member.clone()),
                StepInput::PreviousOutput(StepKind::Aapt2Compile),
                StepInput::ToolchainIdentity(tools.clone()),
                StepInput::ToolchainIdentity(platform.clone())
            ],
            toolchain_identity: tools.clone(),
            output: "linked-resources.apk".into()
        },
    ];
    let mut javac_inputs: Vec<_> = java.into_iter().map(StepInput::SourceMember).collect();
    javac_inputs.extend([
        StepInput::PreviousOutput(StepKind::Aapt2Link),
        StepInput::ToolchainIdentity(plan.toolchain.jdk.clone()),
        StepInput::ToolchainIdentity(platform.clone()),
    ]);
    steps.push(StepPlan {
        kind: StepKind::Javac,
        inputs: javac_inputs,
        toolchain_identity: plan.toolchain.jdk.clone(),
        output: "classes".into(),
    });
    steps.push(StepPlan {
        kind: StepKind::D8,
        inputs: alloc::vec![
            StepInput::PreviousOutput(StepKind::Javac),
            StepInput::ToolchainIdentity(tools.clone()),
            StepInput::ToolchainIdentity(plan.toolchain.jdk.clone()),
            StepInput::ToolchainIdentity(platform.clone())
        ],
        toolchain_identity: tools.clone(),
        output: "classes.dex".into(),
    });
    steps.push(StepPlan {
        kind: StepKind::Zipalign,
        inputs: alloc::vec![
            StepInput::PreviousOutput(StepKind::D8),
            StepInput::PreviousOutput(StepKind::Aapt2Link),
            StepInput::ToolchainIdentity(tools.clone()),
            StepInput::ToolchainIdentity(plan.toolchain.jdk.clone())
        ],
        toolchain_identity: tools.clone(),
        output: "unsigned.apk".into(),
    });
    if let Some(signing) = &plan.signing {
        steps.push(StepPlan {
            kind: StepKind::Apksigner,
            inputs: alloc::vec![
                StepInput::PreviousOutput(StepKind::Zipalign),
                StepInput::KeystoreMember(signing.keystore_member.clone()),
                StepInput::ToolchainIdentity(tools.clone()),
                StepInput::ToolchainIdentity(plan.toolchain.jdk.clone())
            ],
            toolchain_identity: tools.clone(),
            output: "signed.apk".into(),
        });
    }
    Ok(LoweredApk {
        final_output: if plan.signing.is_some() {
            StepKind::Apksigner
        } else {
            StepKind::Zipalign
        },
        steps,
    })
}

#[cfg(test)]
mod tests;
