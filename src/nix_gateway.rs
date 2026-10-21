//! Private Nix gateway host projections. Read-only service remains the default;
//! mutating callers require explicit current remote admission and host policy.

#[cfg(target_os = "linux")]
use std::fs;
#[cfg(target_os = "linux")]
use std::os::unix::fs::DirBuilderExt;
#[cfg(target_os = "linux")]
use std::sync::Arc;
#[cfg(target_os = "linux")]
use std::sync::atomic::AtomicBool;
#[cfg(target_os = "linux")]
use std::sync::atomic::Ordering;

use crate::RunContext;
use crate::errors::RunError;

#[cfg(target_os = "linux")]
pub(crate) mod api_host;
#[cfg(target_os = "linux")]
pub(crate) mod authority;
#[cfg(target_os = "linux")]
pub(crate) mod campaign;
#[cfg(target_os = "linux")]
pub(crate) mod evidence;
#[cfg(target_os = "linux")]
pub(crate) mod native;
#[cfg(target_os = "linux")]
pub(crate) mod resource_host;

#[cfg(target_os = "linux")]
const PRIVATE_SOCKET_NAME: &str = "nix-worker.sock";

#[cfg(target_os = "linux")]
async fn open_read_authority_async(ctx: &RunContext) -> Result<crunch_nix_gateway::store::VerifiedStore, RunError> {
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
    )
    .with_base_state_dirs(ctx.base_state_dirs.clone());
    config.source_admission = ctx.store_source_admission();
    config.fallback_mode = crunch_store::StoreFallbackMode::Strict;
    let handle = crunch_store::StoreHandle::open_overlay(config)
        .await
        .map_err(|_| RunError::Internal("gateway-store-open-denied".into()))?;
    crunch_nix_gateway::store::VerifiedStore::new(handle, trusted)
        .map_err(|reason| RunError::Internal(reason.code().into()))
}

/// Project signed, measured concrete derivations into the existing remote
/// dispatch input. This performs no evaluator work, no ticket redemption and
/// no execution. The host must use the ordinary remote dispatch runner, whose
/// admission, lease, transfer, fencing and signed-output import remain intact.
#[cfg(target_os = "linux")]
pub(crate) async fn prepare_build_paths(
    store: &crunch_nix_gateway::store::VerifiedStore,
    targets: &[String],
) -> Result<Vec<crate::remote_build::RemoteClientDerivationInput>, RunError> {
    let projected = store
        .project_build_paths_for_remote(targets)
        .await
        .map_err(|reason| RunError::Internal(reason.code().into()))?;
    Ok(projected
        .into_iter()
        .map(|proven| {
            let (drv_path, crunch_derivation, nix_derivation) = proven.into_derivations();
            crate::remote_build::RemoteClientDerivationInput {
                label: drv_path.to_absolute_path(),
                drv_path,
                crunch_derivation,
                nix_derivation,
            }
        })
        .collect())
}

/// A caller's OS peer UID is checked by the socket transport. `nix` clients
/// cannot carry Mantle UCAN/ticket authority on this wire, so this command is
/// deliberately limited to verified store reads. It never acknowledges a
/// build, upload or administrative request.
#[cfg(target_os = "linux")]
pub(crate) fn serve_private(
    ctx: &RunContext,
) -> Result<mantle_application_contract::gateway_operations::GatewayOperationEvidence, RunError> {
    let store_runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|_| RunError::Internal("gateway-store-runtime-unavailable".into()))?;
    let store = store_runtime.block_on(open_read_authority_async(ctx))?;
    let private_dir = ctx.resolved_state_dir.join("nix-gateway");
    let mut builder = fs::DirBuilder::new();
    builder.mode(0o700);
    if let Err(error) = builder.create(&private_dir)
        && error.kind() != std::io::ErrorKind::AlreadyExists
    {
        return Err(RunError::Internal("gateway-private-directory-unavailable".into()));
    }
    let socket = private_dir.join(PRIVATE_SOCKET_NAME);
    // Bind before launching the worker so completion is grounded in the actual
    // listener identity, not merely in the worker returning Ok after disconnect.
    let (listener, socket_guard) =
        api_host::bind_socket(&socket).map_err(|error| RunError::Internal(error.code().into()))?;
    let socket_inode = socket_guard.ino;
    let stop = Arc::new(AtomicBool::new(false));
    let worker_stop = Arc::clone(&stop);
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|_| RunError::Internal("gateway-service-runtime-unavailable".into()))?;
    runtime.block_on(async move {
        let mut term = tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .map_err(|_| RunError::Internal("gateway-signal-unavailable".into()))?;
        let mut worker = tokio::task::spawn_blocking(move || {
            let _socket_guard = socket_guard;
            while !worker_stop.load(Ordering::Acquire) {
                match listener.accept() {
                    Ok((stream, _)) => {
                        // Connection rejection is not a listener failure. Keep
                        // the existing bounded owner-authenticated read server.
                        let _ = crunch_nix_gateway::server::serve_connection(stream, &store, &store_runtime);
                    }
                    Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                        std::thread::sleep(std::time::Duration::from_millis(25));
                    }
                    Err(error) if error.kind() == std::io::ErrorKind::Interrupted => continue,
                    Err(error) => return Err(error),
                }
            }
            Ok(())
        });
        let ended = tokio::select! {
            outcome = &mut worker => Some(outcome),
            _ = tokio::signal::ctrl_c() => None,
            _ = term.recv() => None,
        };
        let shutdown_requested = ended.is_none();
        stop.store(true, Ordering::Release);
        let outcome = match ended {
            Some(outcome) => outcome,
            None => worker.await,
        }
        .map_err(|_| RunError::Internal("gateway-service-worker-failed".into()))?;
        outcome.map_err(|_| RunError::Internal("gateway-private-socket-unavailable".into()))?;
        Ok(mantle_application_contract::gateway_operations::GatewayOperationEvidence::Service {
            socket_inode,
            shutdown_requested,
            workers_started: 1,
            workers_joined: 1,
        })
    })
}

#[cfg(not(target_os = "linux"))]
pub(crate) fn serve_private(_ctx: &RunContext) -> Result<(), RunError> {
    Err(RunError::Internal("gateway-private-linux-only".into()))
}
