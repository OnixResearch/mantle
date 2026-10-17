//! Owner-only Nix worker-protocol store reads. This module intentionally has
//! no public network binding and does not submit builds or import NARs.

#[cfg(target_os = "linux")]
use std::fs;
#[cfg(target_os = "linux")]
use std::os::unix::fs::DirBuilderExt;
#[cfg(target_os = "linux")]
use std::path::Path;
#[cfg(target_os = "linux")]
use std::sync::Arc;
#[cfg(target_os = "linux")]
use std::sync::atomic::{AtomicBool, Ordering};

use crate::RunContext;
use crate::errors::RunError;

#[cfg(target_os = "linux")]
const PRIVATE_SOCKET_NAME: &str = "nix-worker.sock";

#[cfg(target_os = "linux")]
fn open_read_authority(ctx: &RunContext) -> Result<crunch_nix_gateway::store::VerifiedStore, RunError> {
    if ctx.store_prefix != "/nix/store" {
        return Err(RunError::Internal("gateway-store-prefix-unsupported".into()));
    }
    let trusted = crate::signing_key::load_configured_trusted_public_keys(None, &ctx.resolved_state_dir)?
        .filter(|keys| !keys.is_empty())
        .ok_or_else(|| RunError::Internal("gateway-trusted-signers-required".into()))?;
    let mut config = crunch_store::StoreConfig::new(
        ctx.store_backend,
        ctx.resolved_state_dir.clone(),
        ctx.store.clone(),
        ctx.store_prefix.clone(),
    ).with_base_state_dirs(ctx.base_state_dirs.clone());
    config.fallback_mode = crunch_store::StoreFallbackMode::Strict;
    let runtime = tokio::runtime::Builder::new_current_thread().enable_all().build()
        .map_err(|_| RunError::Internal("gateway-store-runtime-unavailable".into()))?;
    let handle = runtime.block_on(crunch_store::StoreHandle::open_overlay(config))
        .map_err(|_| RunError::Internal("gateway-store-open-denied".into()))?;
    crunch_nix_gateway::store::VerifiedStore::new(handle, trusted)
        .map_err(|reason| RunError::Internal(reason.code().into()))
}

/// A caller's OS peer UID is checked by the socket transport. `nix` clients
/// cannot carry Mantle UCAN/ticket authority on this wire, so this command is
/// deliberately limited to verified store reads. It never acknowledges a
/// build, upload or administrative request.
#[cfg(target_os = "linux")]
pub(crate) fn serve_private(ctx: &RunContext) -> Result<(), RunError> {
    let store = open_read_authority(ctx)?;
    let private_dir = ctx.resolved_state_dir.join("nix-gateway");
    let mut builder = fs::DirBuilder::new();
    builder.mode(0o700);
    if let Err(error) = builder.create(&private_dir)
        && error.kind() != std::io::ErrorKind::AlreadyExists
    {
        return Err(RunError::Internal("gateway-private-directory-unavailable".into()));
    }
    let socket = private_dir.join(PRIVATE_SOCKET_NAME);
    let stop = Arc::new(AtomicBool::new(false));
    let worker_stop = Arc::clone(&stop);
    let runtime = tokio::runtime::Builder::new_current_thread().enable_all().build()
        .map_err(|_| RunError::Internal("gateway-service-runtime-unavailable".into()))?;
    runtime.block_on(async move {
        let mut term = tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .map_err(|_| RunError::Internal("gateway-signal-unavailable".into()))?;
        let mut worker = tokio::task::spawn_blocking(move || {
            crunch_nix_gateway::server::serve_private(Path::new(&socket), store, &worker_stop)
        });
        let ended = tokio::select! {
            outcome = &mut worker => Some(outcome),
            _ = tokio::signal::ctrl_c() => None,
            _ = term.recv() => None,
        };
        stop.store(true, Ordering::Release);
        let outcome = match ended {
            Some(outcome) => outcome,
            None => worker.await,
        }.map_err(|_| RunError::Internal("gateway-service-worker-failed".into()))?;
        outcome.map_err(|_| RunError::Internal("gateway-private-socket-unavailable".into()))
    })
}

#[cfg(not(target_os = "linux"))]
pub(crate) fn serve_private(_ctx: &RunContext) -> Result<(), RunError> {
    Err(RunError::Internal("gateway-private-linux-only".into()))
}
