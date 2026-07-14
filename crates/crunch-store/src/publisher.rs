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
