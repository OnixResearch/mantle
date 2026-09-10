// HARDENING-BACKLOG 2026-09-09: pre-existing tigerstyle findings in this file are
// recorded in .cairn/changes/complete-store-capability-migration/evidence/
// tigerstyle-remaining-2026-09-09.log and scheduled for the standalone store-shell
// hardening pass. Scoped to the lint categories present at recording time.
#![allow(tigerstyle::usize_in_public_api)]

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
use snix_store::path_info::PathInfo;

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

/// Ordered, bounded publication work requested by output admission.
///
/// An effect plan requests publication work. It does not prove that
/// publication occurred; observations carry that evidence.
#[derive(Debug, Default)]
pub struct PublicationEffectPlan {
    pub steps: Vec<PublicationEffectStep>,
}

impl PublicationEffectPlan {
    pub fn is_empty(&self) -> bool {
        self.steps.is_empty()
    }

    pub fn len(&self) -> usize {
        self.steps.len()
    }
}

/// One requested publication effect: one publisher for one admitted output.
#[derive(Debug)]
pub struct PublicationEffectStep {
    pub publisher_index: u32,
    pub logical_path: String,
    pub path_info: std::sync::Arc<PathInfo>,
    publisher: std::sync::Arc<dyn Publisher>,
}

impl PublicationEffectStep {
    pub(crate) fn new(
        publisher_index: u32,
        logical_path: String,
        path_info: std::sync::Arc<PathInfo>,
        publisher: std::sync::Arc<dyn Publisher>,
    ) -> Self {
        Self {
            publisher_index,
            logical_path,
            path_info,
            publisher,
        }
    }

    /// Execute this effect and return a typed observation.
    ///
    /// Execution never panics on publisher failure; failures are observed.
    pub async fn execute(&self) -> PublicationObservation {
        let outcome = self.publisher.publish(&self.path_info).await;
        PublicationObservation {
            publisher_index: self.publisher_index,
            logical_path: self.logical_path.clone(),
            outcome,
        }
    }
}

/// A typed success or failure observation for one publication effect.
///
/// A failed observation does not erase the underlying output admission.
#[derive(Debug, Clone)]
pub struct PublicationObservation {
    pub publisher_index: u32,
    pub logical_path: String,
    pub outcome: Result<(), String>,
}

impl PublicationObservation {
    pub fn is_success(&self) -> bool {
        self.outcome.is_ok()
    }
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
