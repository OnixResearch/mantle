#![feature(register_tool)]
#![register_tool(tigerstyle)]
// HARDENING-BACKLOG 2026-10-22: pre-existing tigerstyle findings in this file are
// recorded in .cairn/archive/2026-10-22-schedule-tigerstyle-hardening-backlog/evidence/
// tigerstyle-backlog-2026-10-22.log and scheduled for the standalone hardening
// pass. Scoped to the lint categories present at recording time.
#![allow(
    tigerstyle::ambiguous_params,
    reason = "the published consume entry points keep their documented profile and receipt path arguments"
)]
//! Filesystem adapter for exact-byte evidence handoff, with no process/network effects.
use std::collections::BTreeSet;
use std::fs::File;
use std::fs::{self};
use std::io::Read;
use std::io::Write;
use std::path::Path;

pub use crunch_verification_bridge_core;
use crunch_verification_bridge_core::Material;
use crunch_verification_bridge_core::Observation;
use crunch_verification_bridge_core::ObservationRequest;
use crunch_verification_bridge_core::PackagingInputs;
use crunch_verification_bridge_core::identity;
use crunch_verification_bridge_core::relative_path;
use serde_json::Value;

#[derive(Clone, Copy)]
pub enum Producer {
    Octet,
    OctetVerified,
    Trellis,
}

pub struct Handoff {
    pub observation: Observation,
    pub profile: Vec<u8>,
    pub receipt: Vec<u8>,
    pub materials: Vec<Material>,
    pub producer_binding: Option<crunch_verification_bridge_core::ProducerBinding>,
}
pub struct ProfileInputs {
    pub inputs: PackagingInputs,
    pub profile: Vec<u8>,
}

fn read_bounded(root: &Path, relative: String, limit_bytes: usize) -> Result<Material, String> {
    let relative = relative_path(relative).map_err(|error| error.to_string())?;
    if fs::symlink_metadata(root).map_err(|error| error.to_string())?.file_type().is_symlink() {
        return Err("handoff root may not be a symlink".into());
    }
    let mut path = root.to_path_buf();
    for component in relative.split('/') {
        path.push(component);
        if fs::symlink_metadata(&path)
            .map_err(|error| format!("{}: {error}", path.display()))?
            .file_type()
            .is_symlink()
        {
            return Err(format!("symlink artifact rejected: {}", path.display()));
        }
    }
    let file = File::open(&path).map_err(|error| format!("{}: {error}", path.display()))?;
    let metadata = file.metadata().map_err(|error| error.to_string())?;
    if !metadata.is_file() || metadata.len() > limit_bytes as u64 {
        return Err(format!("artifact is not a bounded regular file: {relative}"));
    }
    let mut bytes = Vec::new();
    let read_limit_bytes = (limit_bytes as u64).saturating_add(1);
    file.take(read_limit_bytes).read_to_end(&mut bytes).map_err(|error| error.to_string())?;
    if bytes.len() > limit_bytes {
        return Err(format!("artifact grew beyond byte bound: {relative}"));
    }
    debug_assert!(metadata.is_file());
    debug_assert!(bytes.len() <= limit_bytes);
    Ok(Material { path: relative, bytes })
}

fn json(bytes: &[u8]) -> Result<Value, String> {
    serde_json::from_slice(bytes).map_err(|error| error.to_string())
}

fn text<'a>(value: &'a Value, pointer: &str) -> Result<&'a str, String> {
    value.pointer(pointer).and_then(Value::as_str).ok_or_else(|| format!("missing {pointer}"))
}

/// Derives packaging expectations from an explicit caller-selected profile, never
/// an ambient checkout. These observations confer no external proof acceptance.
pub fn packaging_inputs(producer: Producer, profile: Vec<u8>, revision: &str) -> Result<ProfileInputs, String> {
    if profile.len() > crunch_verification_bridge_core::MAX_DOCUMENT_BYTES {
        return Err("profile byte bound exceeded".into());
    }
    let identified = identity(profile);
    let value = json(&identified.bytes)?;
    let consumer = match producer {
        Producer::Octet | Producer::OctetVerified => &value,
        Producer::Trellis => value.get("consumer").ok_or("missing consumer")?,
    };
    let provider = text(consumer, "/provider/profile_identity")?.to_owned();
    let tool = match producer {
        Producer::Octet | Producer::OctetVerified => provider.clone(),
        Producer::Trellis => {
            identity(
                serde_json::to_vec(value.get("provider_tools").ok_or("missing provider tools")?)
                    .map_err(|error| error.to_string())?,
            )
            .identity
        }
    };
    let options = match producer {
        Producer::Octet | Producer::OctetVerified => consumer.get("resource_policy"),
        Producer::Trellis => value.get("extraction"),
    }
    .ok_or("missing producer options")?;
    debug_assert!(identified.bytes.len() <= crunch_verification_bridge_core::MAX_DOCUMENT_BYTES);
    debug_assert!(!identified.identity.is_empty());
    let inputs = PackagingInputs {
        source_revision: revision.to_owned(),
        source_identity: text(consumer, "/source/paths/0/identity")?.to_owned(),
        profile_identity: identified.identity,
        provider_identity: provider,
        tool_contract_identity: tool,
        options_identity: identity(serde_json::to_vec(options).map_err(|error| error.to_string())?).identity,
    };
    Ok(ProfileInputs {
        inputs,
        profile: identified.bytes,
    })
}

/// Select a profile from an explicit immutable handoff root.
pub fn consume_selected(
    producer: Producer,
    root: &Path,
    profile_path: &str,
    receipt_path: &str,
    revision: &str,
) -> Result<Handoff, String> {
    let profile =
        read_bounded(root, profile_path.to_owned(), crunch_verification_bridge_core::MAX_DOCUMENT_BYTES)?.bytes;
    let ProfileInputs { inputs, profile } = packaging_inputs(producer, profile, revision)?;
    consume_profile(producer, root, profile, receipt_path, inputs)
}

/// Read only paths explicitly named in the selected producer contracts.
pub fn consume(
    producer: Producer,
    root: &Path,
    profile_path: &str,
    receipt_path: &str,
    inputs: PackagingInputs,
) -> Result<Handoff, String> {
    let profile =
        read_bounded(root, profile_path.to_owned(), crunch_verification_bridge_core::MAX_DOCUMENT_BYTES)?.bytes;
    consume_profile(producer, root, profile, receipt_path, inputs)
}
fn consume_profile(
    producer: Producer,
    root: &Path,
    profile: Vec<u8>,
    receipt_path: &str,
    inputs: PackagingInputs,
) -> Result<Handoff, String> {
    let receipt =
        read_bounded(root, receipt_path.to_owned(), crunch_verification_bridge_core::MAX_DOCUMENT_BYTES)?.bytes;
    let parsed = json(&receipt)?;
    let mut paths = material_paths(producer, &parsed)?;
    let verification = if matches!(producer, Producer::OctetVerified) {
        verified_report(root, &parsed, &mut paths)?
    } else {
        None
    };
    let materials = read_materials(root, paths, verification)?;
    let request = ObservationRequest {
        inputs,
        profile,
        receipt,
        materials,
    };
    let observed = match producer {
        Producer::Octet => crunch_verification_bridge_core::observe_octet(request),
        Producer::OctetVerified => crunch_verification_bridge_core::observe_octet_verified(request),
        Producer::Trellis => crunch_verification_bridge_core::observe_trellis(request),
    }
    .map_err(|error| error.to_string())?;
    debug_assert!(!observed.observation.schema.is_empty());
    debug_assert!(!observed.observation.producer.is_empty());
    Ok(Handoff {
        observation: observed.observation,
        profile: observed.profile,
        receipt: observed.receipt,
        materials: observed.materials,
        producer_binding: None,
    })
}

/// The paths the selected producer receipt names for its own materials.
fn material_paths(producer: Producer, parsed: &Value) -> Result<BTreeSet<String>, String> {
    let mut paths = BTreeSet::new();
    match producer {
        Producer::Octet | Producer::OctetVerified => octet_material_paths(parsed, &mut paths)?,
        Producer::Trellis => trellis_material_paths(parsed, &mut paths)?,
    }
    Ok(paths)
}

/// The Octet receipt names its source, the provider profile, and one artifact
/// per generated stage.
fn octet_material_paths(parsed: &Value, paths: &mut BTreeSet<String>) -> Result<(), String> {
    paths.insert(text(parsed, "/consumer/source_path")?.to_owned());
    paths.insert("proofs/aeneas/provider-profile.json".to_owned());
    let stages = parsed
        .pointer("/consumer/stage_artifacts")
        .and_then(Value::as_array)
        .ok_or("missing stage artifacts")?;
    let artifacts = parsed.get("artifact_identities").and_then(Value::as_array).ok_or("missing artifact identities")?;
    if stages.len() > crunch_verification_bridge_core::MAX_MATERIALS
        || artifacts.len() > crunch_verification_bridge_core::MAX_MATERIALS
    {
        return Err("external artifact count exceeds bound".into());
    }
    for stage in stages {
        let digest = text(stage, "/blake3")?.trim_start_matches("b3:");
        for row in artifacts {
            if text(row, "/blake3")?.trim_start_matches("b3:") == digest {
                paths.insert(text(row, "/locator")?.to_owned());
                break;
            }
        }
    }
    debug_assert!(stages.len() <= crunch_verification_bridge_core::MAX_MATERIALS);
    debug_assert!(artifacts.len() <= crunch_verification_bridge_core::MAX_MATERIALS);
    Ok(())
}

/// The Trellis receipt names its source, the verifier receipt, and every
/// artifact of the pilot profile.
fn trellis_material_paths(parsed: &Value, paths: &mut BTreeSet<String>) -> Result<(), String> {
    paths.insert(text(parsed, "/source_path")?.to_owned());
    paths.insert(text(parsed, "/verus_receipt_path")?.to_owned());
    let rows = parsed.get("artifacts").and_then(Value::as_array).ok_or("missing artifacts")?;
    if rows.len() > crunch_verification_bridge_core::MAX_MATERIALS {
        return Err("external artifact count exceeds bound".into());
    }
    for row in rows {
        paths.insert(text(row, "/path")?.to_owned());
    }
    Ok(())
}

/// The artifact-verification report of a verified Octet handoff, together with
/// every payload it and the receipt envelope name.
fn verified_report(root: &Path, parsed: &Value, paths: &mut BTreeSet<String>) -> Result<Option<Material>, String> {
    let verification = read_bounded(
        root,
        "producer-artifact-verification.json".to_owned(),
        crunch_verification_bridge_core::MAX_DOCUMENT_BYTES,
    )?;
    let value = json(&verification.bytes)?;
    let rows = value.get("artifacts").and_then(Value::as_array).ok_or("missing verified artifacts")?;
    if rows.len() > crunch_verification_bridge_core::MAX_MATERIALS {
        return Err("verified artifact count exceeds bound".into());
    }
    for row in rows {
        paths.insert(text(row, "/artifact")?.to_owned());
    }
    let artifacts = parsed.get("artifact_identities").and_then(Value::as_array).ok_or("missing envelope artifacts")?;
    for row in artifacts {
        paths.insert(text(row, "/locator")?.to_owned());
    }
    debug_assert!(rows.len() <= crunch_verification_bridge_core::MAX_MATERIALS);
    debug_assert!(verification.bytes.len() <= crunch_verification_bridge_core::MAX_DOCUMENT_BYTES);
    Ok(Some(verification))
}

/// Reads every named payload, including the verification report, within the
/// bridge material and aggregate byte bounds.
fn read_materials(
    root: &Path,
    paths: BTreeSet<String>,
    verification: Option<Material>,
) -> Result<Vec<Material>, String> {
    let material_count = paths.len().saturating_add(usize::from(verification.is_some()));
    if material_count > crunch_verification_bridge_core::MAX_MATERIALS {
        return Err("material count exceeds bound".into());
    }
    let mut materials = Vec::with_capacity(material_count);
    let mut total_bytes = 0usize;
    if let Some(report) = verification {
        total_bytes = report.bytes.len();
        materials.push(report);
    }
    for path in paths {
        let material = read_bounded(root, path, crunch_verification_bridge_core::MAX_MATERIAL_BYTES)?;
        total_bytes = total_bytes.checked_add(material.bytes.len()).ok_or("material byte overflow")?;
        if total_bytes > crunch_verification_bridge_core::MAX_TOTAL_BYTES {
            return Err("aggregate material byte bound exceeded".into());
        }
        materials.push(material);
    }
    debug_assert!(materials.len() <= crunch_verification_bridge_core::MAX_MATERIALS);
    debug_assert!(total_bytes <= crunch_verification_bridge_core::MAX_TOTAL_BYTES);
    Ok(materials)
}

/// Bind the real producer output to an acquisition-only frozen executable graph.
/// Both roots are explicit inputs; execution never discovers or invokes Nix.
pub fn consume_octet_job(root: &Path, producer_export: &Path, source_revision: &str) -> Result<Handoff, String> {
    use crunch_verification_bridge_core::ObservedHandoff;
    use crunch_verification_bridge_core::ProducerBinding;
    use crunch_verification_bridge_core::bind_octet_job;
    let handoff = consume_selected(
        Producer::OctetVerified,
        root,
        "external-profile.json",
        "external-receipt.json",
        source_revision,
    )?;
    let job =
        read_bounded(producer_export, "job.json".into(), crunch_verification_bridge_core::MAX_DOCUMENT_BYTES)?.bytes;
    let graph = read_bounded(
        producer_export,
        "tool-closure.derivation-json.json".into(),
        crunch_verification_bridge_core::MAX_TOOL_GRAPH_BYTES,
    )?
    .bytes;
    let root_derivation = read_bounded(producer_export, "root-derivation".into(), 4096)?.bytes;
    let flake_metadata = read_bounded(
        producer_export,
        "flake-metadata.json".into(),
        crunch_verification_bridge_core::MAX_DOCUMENT_BYTES,
    )?
    .bytes;
    let flake_lock =
        read_bounded(producer_export, "flake.lock".into(), crunch_verification_bridge_core::MAX_DOCUMENT_BYTES)?.bytes;
    let root_derivation = String::from_utf8(root_derivation).map_err(|error| error.to_string())?.trim().to_owned();
    let bound = bind_octet_job(
        ObservedHandoff {
            observation: handoff.observation,
            profile: handoff.profile,
            receipt: handoff.receipt,
            materials: handoff.materials,
        },
        ProducerBinding {
            job,
            graph,
            flake_metadata,
            flake_lock,
            root_derivation,
        },
    )
    .map_err(|error| error.to_string())?;
    debug_assert!(bound.handoff.observation.producer_job_identity.is_some());
    debug_assert!(bound.handoff.observation.tool_graph_identity.is_some());
    Ok(Handoff {
        observation: bound.handoff.observation,
        profile: bound.handoff.profile,
        receipt: bound.handoff.receipt,
        materials: bound.handoff.materials,
        producer_binding: Some(bound.binding),
    })
}

/// Publish exact external bytes without normalization. The destination must not
/// exist. Mantle store/cache/export handles the resulting ordinary output tree.
pub fn export(mut handoff: Handoff, destination: &Path) -> Result<Handoff, String> {
    if destination.exists() {
        return Err("handoff destination already exists".into());
    }
    let parent = destination.parent().filter(|path| !path.as_os_str().is_empty()).unwrap_or_else(|| Path::new("."));
    fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    let staging = tempfile::tempdir_in(parent).map_err(|error| error.to_string())?;
    write_new(staging.path(), "external-profile.json".into(), &handoff.profile)?;
    write_new(staging.path(), "external-receipt.json".into(), &handoff.receipt)?;
    let observation = serde_json::to_vec_pretty(&handoff.observation).map_err(|error| error.to_string())?;
    write_new(staging.path(), "mantle-observation.json".into(), &observation)?;
    if let Some(binding) = &handoff.producer_binding {
        write_new(staging.path(), "job.json".into(), &binding.job)?;
        write_new(staging.path(), "tool-closure.derivation-json.json".into(), &binding.graph)?;
        write_new(staging.path(), "root-derivation".into(), binding.root_derivation.as_bytes())?;
        write_new(staging.path(), "flake-metadata.json".into(), &binding.flake_metadata)?;
        write_new(staging.path(), "flake.lock".into(), &binding.flake_lock)?;
        write_new(staging.path(), "source-revision".into(), handoff.observation.packaging.source_revision.as_bytes())?;
    }
    for material in &mut handoff.materials {
        material.path = write_new(staging.path(), std::mem::take(&mut material.path), &material.bytes)?;
    }
    fs::rename(staging.path(), destination).map_err(|error| error.to_string())?;
    debug_assert!(!handoff.observation.schema.is_empty());
    debug_assert!(!handoff.observation.producer.is_empty());
    Ok(handoff)
}

fn write_new(root: &Path, relative: String, bytes: &[u8]) -> Result<String, String> {
    let relative = relative_path(relative).map_err(|_| "unsafe export path".to_owned())?;
    let path = root.join(&relative);
    fs::create_dir_all(path.parent().ok_or("missing export parent")?).map_err(|error| error.to_string())?;
    let mut file = File::options().create_new(true).write(true).open(path).map_err(|error| error.to_string())?;
    file.write_all(bytes).map_err(|error| error.to_string())?;
    Ok(relative)
}
