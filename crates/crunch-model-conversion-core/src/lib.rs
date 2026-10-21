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
    if plan.schema != "mantle-model-conversion-v1"
        || plan.converter_revision != CONVERTER_REVISION
        || plan.system != "x86_64-linux"
        || plan.hardware != "x86_64-cpu-little-endian-no-accelerator"
    {
        return Err(Rejection::Cohort);
    }
    if plan.format != "gguf-v3-f32"
        || plan.quantization != "Q8_0"
        || !plan.calibration.is_empty()
        || plan.seed != 0
        || plan.threads != 1
    {
        return Err(Rejection::Options);
    }
    validate_files(plan)?;
    if plan.tools.is_empty() || plan.tools.len() > 64 || !path(&plan.model_root) {
        return Err(Rejection::Identity);
    }
    if plan.cpu_model.is_empty()
        || plan.cpu_model.len() > 256
        || !plan.cpu_model.is_ascii()
        || plan.cpu_model.bytes().any(|b| b.is_ascii_control())
        || plan.cpu_flags.is_empty()
        || plan.cpu_flags.len() > 256
        || plan.cpu_flags.iter().any(|s| {
            s.is_empty()
                || s.len() > 64
                || !s.bytes().all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'_')
        })
        || plan.cpu_flags.windows(2).any(|w| w[0] >= w[1])
    {
        return Err(Rejection::Cohort);
    }
    for (i, tree) in plan.tools.iter().enumerate() {
        if !path(&tree.path)
            || !digest(&tree.blake3)
            || tree.bytes_max == 0
            || tree.bytes_max > 8 << 30
            || plan.tools[..i].iter().any(|t| t.path == tree.path)
        {
            return Err(Rejection::Identity);
        }
    }
    for tool in [&plan.python, &plan.python_site, &plan.converter, &plan.quantizer] {
        if !bound_tool(plan, tool) {
            return Err(Rejection::UndeclaredTool);
        }
    }
    let l = &plan.limits;
    if !(1 << 30..=8 << 30).contains(&l.address_space_bytes)
        || !(256 << 20..=2 << 30).contains(&l.storage_bytes)
        || !(1..=300).contains(&l.cpu_seconds)
        || !(1..=600).contains(&l.wall_seconds)
    {
        return Err(Rejection::Bounds);
    }
    assert!(plan.files.len() <= MAX_FILES);
    assert!(plan.calibration.is_empty());
    Ok(())
}
fn validate_files(plan: &Plan) -> Result<(), Rejection> {
    if plan.files.len() < 5 || plan.files.len() > MAX_FILES {
        return Err(Rejection::Files);
    }
    let mut total = 0_u64;
    for (i, file) in plan.files.iter().enumerate() {
        let allowed = matches!(
            file.member.as_str(),
            "config.json" | "tokenizer.model" | "tokenizer_config.json" | "model.safetensors.index.json"
        ) || (file.member.starts_with("model-")
            && file.member.ends_with(".safetensors")
            && file.member.bytes().all(|b| b.is_ascii_alphanumeric() || b".-".contains(&b)));
        if !allowed
            || !digest(&file.blake3)
            || file.bytes == 0
            || plan.files[..i].iter().any(|f| f.member == file.member)
        {
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
