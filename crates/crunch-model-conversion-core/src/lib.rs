#![no_std]
//! Offline, bounded Llama/SentencePiece safetensors → GGUF v3/F32 → Q8_0.
extern crate alloc;
use alloc::string::String;
use alloc::vec::Vec;

use serde::Deserialize;
use serde::Serialize;

pub const CONVERTER_REVISION: &str = "ec7f3ac9ab33e46b136eb5ab6a76c4d81f57c7f1";
pub const MAX_MODEL_BYTES: u64 = 64 << 20;
pub const MAX_FILES: usize = 32;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FileIdentity {
    pub member: String,
    pub blake3: String,
    pub bytes: u64,
}
/// Full admitted store trees, including native and Python runtime dependencies.
/// `blake3` uses the documented mantle-model-tree-v1 preimage, not NAR hashing.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TreeIdentity {
    pub path: String,
    pub blake3: String,
    pub bytes_max: u64,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Limits {
    pub address_space_bytes: u64,
    pub storage_bytes: u64,
    pub cpu_seconds: u32,
    pub wall_seconds: u32,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Plan {
    pub schema: String,
    pub converter_revision: String,
    pub system: String,
    pub hardware: String,
    pub cpu_model: String,
    pub cpu_flags: Vec<String>,
    pub model_root: String,
    pub files: Vec<FileIdentity>,
    pub tools: Vec<TreeIdentity>,
    pub python: String,
    pub python_site: String,
    pub converter: String,
    pub quantizer: String,
    pub format: String,
    pub quantization: String,
    /// This cohort supports no importance matrix/calibration input. A changed
    /// nonempty declaration fails rather than being silently ignored.
    pub calibration: Vec<FileIdentity>,
    pub seed: u32,
    pub threads: u32,
    pub limits: Limits,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Rejection {
    Cohort,
    Identity,
    Files,
    Bounds,
    Options,
    UndeclaredTool,
}

fn digest(value: &str) -> bool {
    value.len() == 64
        && value.bytes().all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
        && value.bytes().any(|b| b != b'0')
}
fn path(value: &str) -> bool {
    value.starts_with('/')
        && value.len() <= 4096
        && !value.ends_with('/')
        && value[1..].split('/').all(|s| {
            !s.is_empty()
                && s != "."
                && s != ".."
                && s.bytes().all(|b| b.is_ascii_alphanumeric() || b"+._-".contains(&b))
        })
}
fn bound_tool(plan: &Plan, value: &str) -> bool {
    path(value) && plan.tools.iter().any(|t| value.strip_prefix(&t.path).is_some_and(|s| s.starts_with('/')))
}
/// Pure early rejection only. Byte and signed-store authority stay in the
/// existing generic build admission boundary, never in this declaration.
pub fn validate(plan: Plan) -> Result<Plan, Rejection> {
    validate_plan(&plan)?;
    Ok(plan)
}

fn validate_plan(plan: &Plan) -> Result<(), Rejection> {
    validate_cohort(plan)?;
    validate_options(plan)?;
    validate_files(plan)?;
    validate_identity(plan)?;
    validate_cpu_facts(plan)?;
    validate_tool_trees(plan)?;
    validate_tool_bindings(plan)?;
    validate_limits(plan)?;
    assert!(plan.files.len() <= MAX_FILES);
    assert!(plan.calibration.is_empty());
    Ok(())
}

/// The admitted cohort is fixed: schema, converter revision, system and hardware.
fn validate_cohort(plan: &Plan) -> Result<(), Rejection> {
    if plan.schema != "mantle-model-conversion-v1" || plan.converter_revision != CONVERTER_REVISION {
        return Err(Rejection::Cohort);
    }
    if plan.system != "x86_64-linux" || plan.hardware != "x86_64-cpu-little-endian-no-accelerator" {
        return Err(Rejection::Cohort);
    }
    Ok(())
}

/// The admitted conversion options are fixed: format, quantization and seed.
fn validate_options(plan: &Plan) -> Result<(), Rejection> {
    if plan.format != "gguf-v3-f32" || plan.quantization != "Q8_0" {
        return Err(Rejection::Options);
    }
    if !plan.calibration.is_empty() || plan.seed != 0 || plan.threads != 1 {
        return Err(Rejection::Options);
    }
    Ok(())
}

/// The model root and the declared tool set must exist and stay bounded.
fn validate_identity(plan: &Plan) -> Result<(), Rejection> {
    if plan.tools.is_empty() || plan.tools.len() > 64 || !path(&plan.model_root) {
        return Err(Rejection::Identity);
    }
    Ok(())
}

/// The CPU description is part of the admitted cohort.
fn validate_cpu_facts(plan: &Plan) -> Result<(), Rejection> {
    if !is_admissible_cpu_model(&plan.cpu_model) {
        return Err(Rejection::Cohort);
    }
    if plan.cpu_flags.is_empty() || plan.cpu_flags.len() > 256 {
        return Err(Rejection::Cohort);
    }
    if !plan.cpu_flags.iter().all(|flag| is_admissible_cpu_flag(flag.as_str())) {
        return Err(Rejection::Cohort);
    }
    if plan.cpu_flags.windows(2).any(|pair| pair[0] >= pair[1]) {
        return Err(Rejection::Cohort);
    }
    Ok(())
}

fn is_admissible_cpu_model(cpu_model: &str) -> bool {
    if cpu_model.is_empty() {
        return false;
    }
    if cpu_model.len() > 256 {
        return false;
    }
    if !cpu_model.is_ascii() {
        return false;
    }
    if cpu_model.bytes().any(|byte| byte.is_ascii_control()) {
        return false;
    }
    true
}

fn is_admissible_cpu_flag(flag: &str) -> bool {
    if flag.is_empty() {
        return false;
    }
    if flag.len() > 64 {
        return false;
    }
    if !flag.bytes().all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_') {
        return false;
    }
    true
}

/// Every declared tool tree must be well formed and uniquely addressed.
fn validate_tool_trees(plan: &Plan) -> Result<(), Rejection> {
    for (index, tree) in plan.tools.iter().enumerate() {
        if !is_admissible_tool_tree(tree) {
            return Err(Rejection::Identity);
        }
        if plan.tools[..index].iter().any(|other| other.path == tree.path) {
            return Err(Rejection::Identity);
        }
    }
    Ok(())
}

fn is_admissible_tool_tree(tree: &TreeIdentity) -> bool {
    if !path(&tree.path) {
        return false;
    }
    if !digest(&tree.blake3) {
        return false;
    }
    if tree.bytes_max == 0 {
        return false;
    }
    if tree.bytes_max > 8 << 30 {
        return false;
    }
    true
}

/// The interpreter, site directory, converter and quantizer live inside a declared tree.
fn validate_tool_bindings(plan: &Plan) -> Result<(), Rejection> {
    for tool in [&plan.python, &plan.python_site, &plan.converter, &plan.quantizer] {
        if !bound_tool(plan, tool) {
            return Err(Rejection::UndeclaredTool);
        }
    }
    Ok(())
}

/// The declared resource limits must stay inside the admitted bounds.
fn validate_limits(plan: &Plan) -> Result<(), Rejection> {
    let bounds = &plan.limits;
    if !(1 << 30..=8 << 30).contains(&bounds.address_space_bytes) {
        return Err(Rejection::Bounds);
    }
    if !(256 << 20..=2 << 30).contains(&bounds.storage_bytes) {
        return Err(Rejection::Bounds);
    }
    if !(1..=300).contains(&bounds.cpu_seconds) {
        return Err(Rejection::Bounds);
    }
    if !(1..=600).contains(&bounds.wall_seconds) {
        return Err(Rejection::Bounds);
    }
    Ok(())
}
fn validate_files(plan: &Plan) -> Result<(), Rejection> {
    if plan.files.len() < 5 || plan.files.len() > MAX_FILES {
        return Err(Rejection::Files);
    }
    let mut total = 0_u64;
    for (index, file) in plan.files.iter().enumerate() {
        let is_allowed = is_allowed_member(&file.member);
        if !is_allowed || !digest(&file.blake3) {
            return Err(Rejection::Files);
        }
        if file.bytes == 0 {
            return Err(Rejection::Files);
        }
        if plan.files[..index].iter().any(|other| other.member == file.member) {
            return Err(Rejection::Files);
        }
        total = total.checked_add(file.bytes).ok_or(Rejection::Bounds)?;
    }
    for name in [
        "config.json",
        "tokenizer.model",
        "tokenizer_config.json",
        "model.safetensors.index.json",
    ] {
        if !plan.files.iter().any(|f| f.member == name) {
            return Err(Rejection::Files);
        }
    }
    if total > MAX_MODEL_BYTES {
        return Err(Rejection::Bounds);
    }
    assert!(total > 0);
    assert!(!plan.files.is_empty());
    Ok(())
}

/// The admitted member names are the fixed metadata files plus safetensors shards.
fn is_allowed_member(member: &str) -> bool {
    if matches!(member, "config.json" | "tokenizer.model" | "tokenizer_config.json" | "model.safetensors.index.json") {
        return true;
    }
    if !member.starts_with("model-") {
        return false;
    }
    if !member.ends_with(".safetensors") {
        return false;
    }
    member.bytes().all(|byte| byte.is_ascii_alphanumeric() || b".-".contains(&byte))
}

#[cfg(test)]
mod tests {
    use alloc::vec;

    use super::*;
    fn fixture() -> Plan {
        Plan {
            schema: "mantle-model-conversion-v1".into(),
            converter_revision: CONVERTER_REVISION.into(),
            system: "x86_64-linux".into(),
            hardware: "x86_64-cpu-little-endian-no-accelerator".into(),
            cpu_model: "synthetic-contract-only".into(),
            cpu_flags: vec!["sse2".into()],
            model_root: "/mantle/store/model".into(),
            files: [
                "config.json",
                "tokenizer.model",
                "tokenizer_config.json",
                "model.safetensors.index.json",
                "model-00001-of-00001.safetensors",
            ]
            .into_iter()
            .map(|member| FileIdentity {
                member: member.into(),
                blake3: "a".repeat(64),
                bytes: 1,
            })
            .collect(),
            tools: vec![TreeIdentity {
                path: "/mantle/store/tool".into(),
                blake3: "a".repeat(64),
                bytes_max: 4 << 30,
            }],
            python: "/mantle/store/tool/bin/python".into(),
            python_site: "/mantle/store/tool/site".into(),
            converter: "/mantle/store/tool/convert_hf_to_gguf.py".into(),
            quantizer: "/mantle/store/tool/bin/llama-quantize".into(),
            format: "gguf-v3-f32".into(),
            quantization: "Q8_0".into(),
            calibration: vec![],
            seed: 0,
            threads: 1,
            limits: Limits {
                address_space_bytes: 4 << 30,
                storage_bytes: 512 << 20,
                cpu_seconds: 60,
                wall_seconds: 120,
            },
        }
    }
    #[test]
    fn rejects_execution_and_calibration_not_in_supported_fragment() {
        let mut p = fixture();
        p = validate(p).unwrap();
        p.calibration.push(p.files[0].clone());
        assert_eq!(validate(p).unwrap_err(), Rejection::Options);
        p = fixture();
        p.files[4].member = "pytorch_model.bin".into();
        assert_eq!(validate(p).unwrap_err(), Rejection::Files);
        p = fixture();
        p.quantizer = "/usr/bin/llama-quantize".into();
        assert_eq!(validate(p).unwrap_err(), Rejection::UndeclaredTool);
    }
    #[test]
    fn missing_shard_and_duplicate_members_fail_early() {
        let mut p = fixture();
        p.files.pop();
        assert_eq!(validate(p).unwrap_err(), Rejection::Files);
        p = fixture();
        p.files.push(p.files[4].clone());
        assert_eq!(validate(p).unwrap_err(), Rejection::Files);
        p = fixture();
        p.files[4].bytes = MAX_MODEL_BYTES;
        assert_eq!(validate(p).unwrap_err(), Rejection::Bounds);
    }
}
