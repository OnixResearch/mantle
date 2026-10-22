//! Lowers a content-bound CPU conversion plan into an ordinary derivation.
//! No scheduler/store/trust implementation lives here. Execution and signed
//! input/output admission are the existing `mantle build` path's authority.
use std::collections::BTreeMap;

use crunch_model_conversion_core::Plan;
use crunch_model_conversion_core::Rejection;
use crunch_model_conversion_core::validate;
use serde::Serialize;

const READER: &str = include_str!("gguf_reader.py");
const RUNNER: &str = include_str!("runner.py");

#[derive(Debug)]
pub enum Error {
    Plan(Rejection),
    StoreInput,
    Json(serde_json::Error),
}
#[derive(Debug, Serialize)]
pub struct Derivation {
    pub name: String,
    pub builder: String,
    pub args: Vec<String>,
    pub env: BTreeMap<String, String>,
    pub inputs: Vec<String>,
    pub outputs: Vec<String>,
}

/// A store path together with the prefix it must live under.
struct StoreRootRequest<'a> {
    value: &'a str,
    prefix: &'a str,
}

fn store_root(request: StoreRootRequest<'_>) -> Result<String, Error> {
    let StoreRootRequest { value, prefix } = request;
    let relative = value.strip_prefix(prefix).and_then(|s| s.strip_prefix('/')).ok_or(Error::StoreInput)?;
    let root = relative.split('/').next().ok_or(Error::StoreInput)?;
    let (hash, name) = root.split_once('-').ok_or(Error::StoreInput)?;
    if hash.len() != 32 || !hash.bytes().all(|b| b"0123456789abcdfghijklmnpqrsvwxyz".contains(&b)) || name.is_empty() {
        return Err(Error::StoreInput);
    }
    assert!(!relative.starts_with('/'));
    assert!(!name.is_empty());
    Ok(format!("{prefix}/{root}"))
}

/// `prefix` must be the same logical store prefix selected for `mantle build`.
/// Every root is passed through the generic signed input admission boundary;
/// lexical validation here is deliberately not an attestation substitute.
pub fn lower(plan: Plan, prefix: &str) -> Result<Derivation, Error> {
    let plan = validate(plan).map_err(Error::Plan)?;
    if !prefix.starts_with('/') || prefix.ends_with('/') {
        return Err(Error::StoreInput);
    }
    if prefix.contains("..") || prefix.contains('\n') {
        return Err(Error::StoreInput);
    }
    let mut inputs = vec![store_root(StoreRootRequest {
        value: &plan.model_root,
        prefix,
    })?];
    for tool in &plan.tools {
        let root = store_root(StoreRootRequest {
            value: &tool.path,
            prefix,
        })?;
        if root != tool.path {
            return Err(Error::StoreInput);
        }
        inputs.push(root);
    }
    inputs.sort();
    inputs.dedup();
    let json = serde_json::to_string(&plan).map_err(Error::Json)?;
    let identity = blake3::hash(json.as_bytes()).to_hex();
    let mut env = BTreeMap::from([
        ("MANTLE_MODEL_PLAN".into(), json),
        ("PYTHONPATH".into(), plan.python_site),
        ("PYTHONHASHSEED".into(), "0".into()),
        ("PYTHONNOUSERSITE".into(), "1".into()),
        ("PYTHONDONTWRITEBYTECODE".into(), "1".into()),
        ("OMP_NUM_THREADS".into(), "1".into()),
        ("OPENBLAS_NUM_THREADS".into(), "1".into()),
        ("MKL_NUM_THREADS".into(), "1".into()),
        ("NUMEXPR_NUM_THREADS".into(), "1".into()),
        ("VECLIB_MAXIMUM_THREADS".into(), "1".into()),
        ("CUDA_VISIBLE_DEVICES".into(), String::new()),
        ("HF_HUB_OFFLINE".into(), "1".into()),
        ("TRANSFORMERS_OFFLINE".into(), "1".into()),
        ("LC_ALL".into(), "C".into()),
        ("TZ".into(), "UTC".into()),
        ("SOURCE_DATE_EPOCH".into(), "0".into()),
        ("HOME".into(), "/build/disabled-home".into()),
        ("PATH".into(), String::new()),
    ]);
    env.insert("MANTLE_MODEL_PLAN_BLAKE3".into(), identity.to_string());
    env.insert("MANTLE_MODEL_READER_BLAKE3".into(), blake3::hash(READER.as_bytes()).to_hex().to_string());
    env.insert("MANTLE_MODEL_RUNNER_BLAKE3".into(), blake3::hash(RUNNER.as_bytes()).to_hex().to_string());
    assert!(!inputs.is_empty());
    assert!(env.contains_key("MANTLE_MODEL_PLAN"));
    Ok(Derivation {
        name: format!("model-conversion-{}", &identity.as_str()[..16]),
        builder: plan.python,
        args: vec!["-s".into(), "-c".into(), format!("{READER}\n{RUNNER}")],
        env,
        inputs,
        outputs: vec!["out".into()],
    })
}
