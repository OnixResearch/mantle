//! Output publication adapters for remote build results.
//!
//! A [`Publisher`] is called after local output admission has accepted the
//! PathInfo, castore objects, and attestation evidence. Publication is
//! separate from realization success — publisher errors are diagnostic
//! warnings, not build failures (unless policy requires publication).
//!
//! r[impl remote_builds.production_verified_publication]

use std::fmt;
#[cfg(test)]
use std::sync::Mutex;

use async_trait::async_trait;
use serde::Deserialize;
use serde::Serialize;
use snix_store::path_info::PathInfo;

const PUBLICATION_PLAN_SCHEMA: &str = "mantle-output-publication-plan-v1";
const PUBLICATION_EFFECT_DOMAIN: &str = "mantle-output-publication-effect-v1";
const PUBLICATION_PLAN_DOMAIN: &str = "mantle-output-publication-plan-identity-v1";
const MAX_OUTPUT_PUBLISHERS: u32 = 1_024;

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PublicationEffect {
    pub effect_id_blake3: String,
    pub publisher_index: u32,
    pub logical_path: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PublicationEffectPlan {
    pub schema: String,
    pub plan_blake3: String,
    pub logical_path: String,
    pub effects: Vec<PublicationEffect>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AdmittedOutput {
    pub path_info: PathInfo,
    pub publication_plan: PublicationEffectPlan,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum PublicationDisposition {
    Succeeded,
    Failed,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PublicationObservation {
    pub effect_id_blake3: String,
    pub publisher_index: u32,
    pub logical_path: String,
    pub disposition: PublicationDisposition,
    pub error: Option<String>,
}

pub(crate) fn plan_publication(path_info: &PathInfo, publisher_count: u32) -> Result<PublicationEffectPlan, String> {
    if publisher_count > MAX_OUTPUT_PUBLISHERS {
        return Err(format!("publisher count exceeds {MAX_OUTPUT_PUBLISHERS}"));
    }
    let logical_path = path_info.store_path.to_string();
    let effects_len = usize::try_from(publisher_count).map_err(|_| "publisher count does not fit usize".to_string())?;
    let mut effects = Vec::with_capacity(effects_len);
    for publisher_index in 0..publisher_count {
        effects.push(PublicationEffect {
            effect_id_blake3: publication_effect_identity(&logical_path, publisher_index),
            publisher_index,
            logical_path: logical_path.clone(),
        });
    }
    debug_assert!(publisher_count <= MAX_OUTPUT_PUBLISHERS);
    debug_assert_eq!(effects.len(), effects_len);
    let plan_blake3 = publication_plan_identity(&logical_path, &effects)?;
    Ok(PublicationEffectPlan {
        schema: PUBLICATION_PLAN_SCHEMA.to_string(),
        plan_blake3,
        logical_path,
        effects,
    })
}

pub(crate) fn validate_publication_plan(
    plan: &PublicationEffectPlan,
    path_info: &PathInfo,
    publisher_count: usize,
) -> Result<(), String> {
    if plan.schema != PUBLICATION_PLAN_SCHEMA || plan.logical_path != path_info.store_path.to_string() {
        return Err("publication plan identity mismatch".to_string());
    }
    let planned_publisher_count = u32::try_from(plan.effects.len())
        .map_err(|_| "publication plan publisher count does not fit u32".to_string())?;
    let configured_publisher_count =
        u32::try_from(publisher_count).map_err(|_| "configured publisher count does not fit u32".to_string())?;
    if planned_publisher_count != configured_publisher_count || planned_publisher_count > MAX_OUTPUT_PUBLISHERS {
        return Err("publication plan publisher count mismatch".to_string());
    }
    if plan.plan_blake3 != publication_plan_identity(&plan.logical_path, &plan.effects)? {
        return Err("publication plan BLAKE3 mismatch".to_string());
    }
    debug_assert!(planned_publisher_count <= MAX_OUTPUT_PUBLISHERS);
    debug_assert_eq!(planned_publisher_count, configured_publisher_count);
    for (expected_index, effect) in plan.effects.iter().enumerate() {
        let expected_index =
            u32::try_from(expected_index).map_err(|_| "publication plan index does not fit u32".to_string())?;
        if effect.publisher_index != expected_index
            || effect.logical_path != plan.logical_path
            || effect.effect_id_blake3 != publication_effect_identity(&plan.logical_path, expected_index)
        {
            return Err("publication effect identity mismatch".to_string());
        }
    }
    Ok(())
}

fn publication_plan_identity(logical_path: &str, effects: &[PublicationEffect]) -> Result<String, String> {
    let mut hasher = blake3::Hasher::new();
    hash_framed(&mut hasher, PUBLICATION_PLAN_DOMAIN.as_bytes())?;
    hash_framed(&mut hasher, logical_path.as_bytes())?;
    for effect in effects {
        hash_framed(&mut hasher, effect.effect_id_blake3.as_bytes())?;
        hasher.update(&effect.publisher_index.to_le_bytes());
    }
    Ok(hasher.finalize().to_hex().to_string())
}

fn publication_effect_identity(logical_path: &str, publisher_index: u32) -> String {
    let mut hasher = blake3::Hasher::new();
    hasher.update(PUBLICATION_EFFECT_DOMAIN.as_bytes());
    hasher.update(logical_path.as_bytes());
    hasher.update(&publisher_index.to_le_bytes());
    hasher.finalize().to_hex().to_string()
}

fn hash_framed(hasher: &mut blake3::Hasher, bytes: &[u8]) -> Result<(), String> {
    let length_bytes = u64::try_from(bytes.len()).map_err(|_| "publication identity frame length does not fit u64")?;
    hasher.update(&length_bytes.to_le_bytes());
    hasher.update(bytes);
    Ok(())
}

/// A publisher exports an admitted build output to an external cache or
/// artifact store.
///
/// Publisher adapters MAY target local archives, Nix-compatible binary
/// caches, HTTP/S3 artifact stores, or future provider backends.
#[async_trait]
pub trait Publisher: Send + Sync + fmt::Debug {
    /// Publish an admitted output.
    ///
    /// Called after `persist_and_export_signed_output` has accepted the
    /// PathInfo. Returns `Ok(())` on success or `Err(message)` on failure.
    /// Errors are logged as warnings — they do not retroactively fail the
    /// build unless the caller enforces strict publication policy.
    async fn publish(&self, path_info: &PathInfo) -> Result<(), String>;
}

/// A no-op publisher that skips all outputs.
/// Used as the default when no publishers are configured.
#[derive(Debug)]
pub struct NoopPublisher;

#[async_trait]
impl Publisher for NoopPublisher {
    async fn publish(&self, _path_info: &PathInfo) -> Result<(), String> {
        Ok(())
    }
}

/// A publisher that records every PathInfo it receives and optionally
/// fails on request. Used for testing publication hooks.
#[cfg(test)]
#[derive(Debug)]
pub struct RecordingPublisher {
    calls: Mutex<Vec<String>>,
    fail_on_next: Mutex<bool>,
}

#[cfg(test)]
impl Default for RecordingPublisher {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
impl RecordingPublisher {
    pub fn new() -> Self {
        Self {
            calls: Mutex::new(Vec::new()),
            fail_on_next: Mutex::new(false),
        }
    }

    pub fn fail_next(&self) {
        *self.fail_on_next.lock().unwrap() = true;
    }

    pub fn calls(&self) -> Vec<String> {
        self.calls.lock().unwrap().clone()
    }

    pub fn call_count(&self) -> usize {
        self.calls.lock().unwrap().len()
    }
}

#[cfg(test)]
#[async_trait]
impl Publisher for RecordingPublisher {
    async fn publish(&self, path_info: &PathInfo) -> Result<(), String> {
        let store_path = path_info.store_path.to_string();
        if *self.fail_on_next.lock().unwrap() {
            *self.fail_on_next.lock().unwrap() = false;
            return Err(format!("simulated publication failure for {store_path}"));
        }
        self.calls.lock().unwrap().push(store_path);
        Ok(())
    }
}
