//! Native closure resolution: walk the reference graph in PathInfo
//! (local redb + optional remote binary cache) instead of shelling
//! out to `nix-store -qR`.
use std::collections::BTreeSet;
use std::collections::VecDeque;

use async_trait::async_trait;
use crunch_gc_core::retention::RuntimeClosureLimits;
use crunch_gc_core::retention::RuntimeClosureMemberInput;
use crunch_gc_core::retention::RuntimeClosureMemberState;
use crunch_gc_core::retention::RuntimeClosureSourceKind;
use crunch_gc_core::retention::RuntimeClosureTrustStatus;
use data_encoding::HEXLOWER;
use nix_compat::narinfo::VerifyingKey;
use nix_compat::narinfo::fingerprint_with_store_dir;
use nix_compat::store_path::StorePath;
use nix_compat::store_path::StorePathRef;
use snix_store::path_info::PathInfo;
use snix_store::pathinfoservice::PathInfoService;
use tracing::warn;

use crate::Error;
use crate::StoreAuditEvent;
use crate::StoreAuditKind;
use crate::StoreFallbackMode;
use crate::layer::Layered;
use crate::layer::StoreLayer;

/// Maximum transitive depth for closure walks. Prevents runaway
/// recursion from cycles the visited-set misses or absurdly deep
/// dependency chains.
pub const MAX_CLOSURE_DEPTH: u32 = 1024;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClosureResolution {
    pub paths: Vec<StorePath<String>>,
    pub audit_events: Vec<StoreAuditEvent>,
}

enum LookupStatus {
    Found(Vec<StorePath<String>>),
    Missing(String),
    Failed(String),
}

enum ReferenceLookup {
    Resolved(Vec<StorePath<String>>),
    Degraded(String),
}

/// Resolve the full runtime closure of `root` by walking PathInfo
/// references.
///
/// Tries `local` first, then `remote` (if provided). Returns the
/// transitive set of all referenced store paths, including `root`
/// itself, plus any practical-mode degraded audit events.
pub async fn resolve_closure(
    root: &StorePath<String>,
    local: &dyn PathInfoService,
    remote: Option<&dyn PathInfoService>,
    fallback_mode: StoreFallbackMode,
    store_dir: &str,
) -> Result<ClosureResolution, Error> {
    const MAX_CLOSURE_MEMBERS: usize = 100_000;
    let mut visited: BTreeSet<[u8; 20]> = BTreeSet::new();
    let mut result: Vec<StorePath<String>> = Vec::with_capacity(64);
    let mut audit_events: Vec<StoreAuditEvent> = Vec::with_capacity(4);
    let mut stack: Vec<(StorePath<String>, u32)> = Vec::with_capacity(64);
    stack.push((root.clone(), 0));

    while let Some((path, depth)) = stack.pop() {
        let digest = *path.digest();
        if !visited.insert(digest) {
            continue;
        }
        assert!(result.len() < MAX_CLOSURE_MEMBERS, "closure exceeded {MAX_CLOSURE_MEMBERS} members");
        result.push(path.clone());
        if depth >= MAX_CLOSURE_DEPTH {
            warn!(path = %path, depth = depth, "closure depth limit ({MAX_CLOSURE_DEPTH}) reached, skipping deeper refs");
            continue;
        }
        match lookup_references(local, remote, digest).await {
            ReferenceLookup::Resolved(refs) => push_unvisited_refs(&mut stack, &visited, refs, depth),
            ReferenceLookup::Degraded(reason) => {
                handle_degraded_lookup(fallback_mode, &path, reason, store_dir, &mut audit_events)?;
            }
        }
    }

    debug_assert!(!result.is_empty(), "closure must contain at least the root");
    Ok(ClosureResolution {
        paths: result,
        audit_events,
    })
}

async fn lookup_references(
    local: &dyn PathInfoService,
    remote: Option<&dyn PathInfoService>,
    digest: [u8; 20],
) -> ReferenceLookup {
    match lookup_local_references(local, digest).await {
        LookupStatus::Found(refs) => ReferenceLookup::Resolved(refs),
        LookupStatus::Missing(local_reason) => lookup_remote_or_degrade(remote, digest, local_reason).await,
        LookupStatus::Failed(local_reason) => lookup_remote_or_degrade(remote, digest, local_reason).await,
    }
}

async fn lookup_local_references(local: &dyn PathInfoService, digest: [u8; 20]) -> LookupStatus {
    match local.get_references(digest).await {
        Ok(Some(refs)) => LookupStatus::Found(refs),
        Ok(None) => LookupStatus::Missing("no local PathInfo".to_string()),
        Err(e) => LookupStatus::Failed(format!("local PathInfo query failed: {e}")),
    }
}

async fn lookup_remote_or_degrade(
    remote: Option<&dyn PathInfoService>,
    digest: [u8; 20],
    local_reason: String,
) -> ReferenceLookup {
    match lookup_remote_references(remote, digest).await {
        LookupStatus::Found(refs) => ReferenceLookup::Resolved(refs),
        LookupStatus::Missing(remote_reason) | LookupStatus::Failed(remote_reason) => {
            ReferenceLookup::Degraded(format!("{local_reason}; {remote_reason}"))
        }
    }
}

async fn lookup_remote_references(remote: Option<&dyn PathInfoService>, digest: [u8; 20]) -> LookupStatus {
    let Some(remote) = remote else {
        return LookupStatus::Missing("no remote narinfo configured".to_string());
    };
    match remote.get_references(digest).await {
        Ok(Some(refs)) => LookupStatus::Found(refs),
        Ok(None) => LookupStatus::Missing("no remote narinfo".to_string()),
        Err(e) => LookupStatus::Failed(format!("remote narinfo query failed: {e}")),
    }
}

fn handle_degraded_lookup(
    fallback_mode: StoreFallbackMode,
    path: &StorePath<String>,
    reason: String,
    store_dir: &str,
    audit_events: &mut Vec<StoreAuditEvent>,
) -> Result<(), Error> {
    assert!(!store_dir.is_empty(), "store_dir must not be empty");
    assert!(store_dir.starts_with('/'), "store_dir must be absolute");
    if fallback_mode.is_strict() {
        return Err(Error::MissingClosureFacts {
            path: path.clone(),
            store_dir: store_dir.to_string(),
            detail: reason,
        });
    }
    let detail = format!("missing closure facts for {}: {reason}", path.to_absolute_path_with_prefix(store_dir));
    warn!(path = %path, detail = %detail, "closure resolution degraded, mounting declared path only");
    audit_events.push(StoreAuditEvent::new(StoreAuditKind::ClosureResolutionDegraded, detail));
    Ok(())
}

fn push_unvisited_refs(
    stack: &mut Vec<(StorePath<String>, u32)>,
    visited: &BTreeSet<[u8; 20]>,
    refs: Vec<StorePath<String>>,
    depth: u32,
) {
    for r in refs {
        if !visited.contains(r.digest()) {
            stack.push((r, depth.saturating_add(1)));
        }
    }
}

#[derive(Clone, Debug)]
pub struct RuntimeClosureMetadataPolicy {
    pub store_dir: String,
    pub limits: RuntimeClosureLimits,
    pub trusted_public_keys: Vec<VerifyingKey>,
    pub is_allow_unsigned_local: bool,
    pub is_allow_remote_metadata: bool,
    pub remote_source_id: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RuntimeClosureCollectedMetadata {
    pub members: Vec<RuntimeClosureMemberInput>,
    pub elapsed_ms: u64,
    pub remote_metadata_request_count: u32,
}

pub(crate) enum LocalMetadataRead {
    Found(Box<Layered<PathInfo>>),
    Missing,
    IdentityConflict(String),
    UntrustedSource(String),
    Failed(String),
}

#[async_trait]
pub(crate) trait RuntimeMetadataSource: Send + Sync {
    async fn read_local(&self, path: &StorePath<String>) -> LocalMetadataRead;
    async fn read_remote(&self, path: &StorePath<String>) -> Result<Option<PathInfo>, String>;
}

struct ServiceMetadataSource<'a> {
    local: &'a dyn PathInfoService,
    remote: Option<&'a dyn PathInfoService>,
}

#[async_trait]
impl RuntimeMetadataSource for ServiceMetadataSource<'_> {
    async fn read_local(&self, path: &StorePath<String>) -> LocalMetadataRead {
        match self.local.get(*path.digest()).await {
            Ok(Some(pi)) => LocalMetadataRead::Found(Box::new(Layered::overlay(pi))),
            Ok(None) => LocalMetadataRead::Missing,
            Err(error) => LocalMetadataRead::Failed(format!("local PathInfo lookup failed: {error}")),
        }
    }

    async fn read_remote(&self, path: &StorePath<String>) -> Result<Option<PathInfo>, String> {
        let Some(remote) = self.remote else {
            return Ok(None);
        };
        remote.get(*path.digest()).await.map_err(|error| format!("remote PathInfo lookup failed: {error}"))
    }
}

pub async fn collect_runtime_closure_metadata(
    roots: &[StorePath<String>],
    local: &dyn PathInfoService,
    remote: Option<&dyn PathInfoService>,
    policy: &RuntimeClosureMetadataPolicy,
) -> Result<RuntimeClosureCollectedMetadata, Error> {
    validate_metadata_policy(roots, policy)?;
    debug_assert!(!roots.is_empty());
    debug_assert!(!policy.store_dir.is_empty());
    let source = ServiceMetadataSource { local, remote };
    let mut now_ms = || {
        crate::roots::current_unix_seconds()
            .ok()
            .and_then(|secs| u64::try_from(secs).ok())
            .map_or(0, |secs| secs.saturating_mul(1_000))
    };
    collect_runtime_closure_metadata_from_source(roots, &source, policy, &mut now_ms).await
}

pub(crate) async fn collect_runtime_closure_metadata_from_source(
    roots: &[StorePath<String>],
    source: &dyn RuntimeMetadataSource,
    policy: &RuntimeClosureMetadataPolicy,
    now_ms: &mut (dyn FnMut() -> u64 + Send),
) -> Result<RuntimeClosureCollectedMetadata, Error> {
    validate_metadata_policy(roots, policy)?;
    let started_ms = now_ms();
    let max_nodes = usize::try_from(policy.limits.max_nodes_count)
        .map_err(|_| Error::Store("runtime closure max_nodes_count exceeds platform usize".to_string()))?;
    let max_steps = policy.limits.max_nodes_count.saturating_add(policy.limits.max_edges_count).saturating_add(1);

    let mut visited: BTreeSet<String> = BTreeSet::new();
    let mut queue: VecDeque<(StorePath<String>, u32)> = VecDeque::new();
    for root in roots {
        queue.push_back((root.clone(), 0));
    }

    let mut members = Vec::with_capacity(max_nodes.min(256));
    let mut remote_metadata_request_count: u32 = 0;
    let mut examined_edge_count: u32 = 0;
    let mut running_nar_bytes: u64 = 0;
    let mut observed_elapsed_ms: u64 = 0;

    for _step in 0..max_steps {
        let Some((path, depth)) = queue.pop_front() else {
            break;
        };
        if !visited.insert(path.to_string()) {
            continue;
        }
        if members.len() >= max_nodes {
            break;
        }
        observed_elapsed_ms = now_ms().saturating_sub(started_ms);
        if observed_elapsed_ms > policy.limits.max_elapsed_ms {
            members.push(budget_exhausted_member(&path, policy));
            break;
        }

        let (member, child_refs) = collect_one_member(&path, source, policy, &mut remote_metadata_request_count).await;
        if let Some(bytes) = member.nar_size_bytes {
            running_nar_bytes = running_nar_bytes.saturating_add(bytes);
        }
        members.push(member);

        if running_nar_bytes > policy.limits.max_total_nar_bytes {
            break;
        }
        enqueue_bounded_children(depth, child_refs, &policy.limits, &mut ChildEnqueueState {
            visited: &visited,
            queue: &mut queue,
            examined_edge_count: &mut examined_edge_count,
        });
    }

    let final_elapsed_ms = now_ms().saturating_sub(started_ms).max(observed_elapsed_ms);
    debug_assert!(!members.is_empty());
    Ok(RuntimeClosureCollectedMetadata {
        members,
        elapsed_ms: final_elapsed_ms,
        remote_metadata_request_count,
    })
}

struct ChildEnqueueState<'a> {
    visited: &'a BTreeSet<String>,
    queue: &'a mut VecDeque<(StorePath<String>, u32)>,
    examined_edge_count: &'a mut u32,
}

fn enqueue_bounded_children(
    depth: u32,
    child_refs: Vec<StorePath<String>>,
    limits: &RuntimeClosureLimits,
    state: &mut ChildEnqueueState<'_>,
) {
    assert!(limits.max_edges_count > 0);
    if depth >= limits.max_depth_count {
        return;
    }
    let next_depth = depth.saturating_add(1);
    for child in child_refs {
        if *state.examined_edge_count >= limits.max_edges_count {
            break;
        }
        *state.examined_edge_count = state.examined_edge_count.saturating_add(1);
        if !state.visited.contains(&child.to_string()) {
            state.queue.push_back((child, next_depth));
        }
    }
    debug_assert!(*state.examined_edge_count <= limits.max_edges_count);
}

fn validate_metadata_policy(roots: &[StorePath<String>], policy: &RuntimeClosureMetadataPolicy) -> Result<(), Error> {
    if roots.is_empty() {
        return Err(Error::Store("runtime closure requires at least one root".to_string()));
    }
    if policy.store_dir.is_empty() || !policy.store_dir.starts_with('/') {
        return Err(Error::Store("runtime closure store_dir must be absolute".to_string()));
    }
    if !policy.store_dir.is_ascii() {
        return Err(Error::Store("runtime closure store_dir must be ASCII".to_string()));
    }
    if policy.limits.max_nodes_count == 0 || policy.limits.max_edges_count == 0 || policy.limits.max_elapsed_ms == 0 {
        return Err(Error::Store("runtime closure bounds must be positive".to_string()));
    }
    debug_assert!(!roots.is_empty());
    debug_assert!(policy.store_dir.is_ascii());
    Ok(())
}

async fn collect_one_member(
    path: &StorePath<String>,
    source: &dyn RuntimeMetadataSource,
    policy: &RuntimeClosureMetadataPolicy,
    remote_request_count: &mut u32,
) -> (RuntimeClosureMemberInput, Vec<StorePath<String>>) {
    assert!(!policy.store_dir.is_empty());
    let path_id = path.to_absolute_path_with_prefix(&policy.store_dir);
    debug_assert!(path_id.starts_with('/'));
    match source.read_local(path).await {
        LocalMetadataRead::Found(layered) => {
            let src = match layered.layer {
                StoreLayer::Overlay => RuntimeClosureSourceKind::LocalOverlay,
                StoreLayer::Base { index } => RuntimeClosureSourceKind::LocalBase {
                    layer_index: crate::gc::saturating_u32(index),
                },
            };
            evaluate_path_info_member(MemberEvalContext {
                requested_path: path,
                path_id: &path_id,
                pi: &layered.value,
                source: src,
                policy,
                is_remote: false,
            })
        }
        LocalMetadataRead::IdentityConflict(detail) => (
            unavailable_member(UnavailableMemberArgs {
                path_id,
                source: RuntimeClosureSourceKind::Unavailable,
                state: RuntimeClosureMemberState::IdentityConflict,
                detail,
            }),
            Vec::new(),
        ),
        LocalMetadataRead::UntrustedSource(detail) => (
            unavailable_member(UnavailableMemberArgs {
                path_id,
                source: RuntimeClosureSourceKind::Unavailable,
                state: RuntimeClosureMemberState::UntrustedSource,
                detail,
            }),
            Vec::new(),
        ),
        LocalMetadataRead::Failed(detail) => (
            unavailable_member(UnavailableMemberArgs {
                path_id,
                source: RuntimeClosureSourceKind::Unavailable,
                state: RuntimeClosureMemberState::LookupFailed,
                detail,
            }),
            Vec::new(),
        ),
        LocalMetadataRead::Missing => {
            collect_missing_local_member(path, &path_id, source, policy, remote_request_count).await
        }
    }
}

async fn collect_missing_local_member(
    path: &StorePath<String>,
    path_id: &str,
    source: &dyn RuntimeMetadataSource,
    policy: &RuntimeClosureMetadataPolicy,
    remote_request_count: &mut u32,
) -> (RuntimeClosureMemberInput, Vec<StorePath<String>>) {
    assert!(!path_id.is_empty());
    assert!(!policy.store_dir.is_empty());
    if !policy.is_allow_remote_metadata {
        return (
            unavailable_member(UnavailableMemberArgs {
                path_id: path_id.to_string(),
                source: RuntimeClosureSourceKind::Unavailable,
                state: RuntimeClosureMemberState::MissingMetadata,
                detail: "no local PathInfo metadata".to_string(),
            }),
            Vec::new(),
        );
    }
    *remote_request_count = remote_request_count.saturating_add(1);
    let source_id = match &policy.remote_source_id {
        Some(id) => id.clone(),
        None => "remote-substituter".to_string(),
    };
    let src = RuntimeClosureSourceKind::RemoteSubstituter { source_id };
    match source.read_remote(path).await {
        Ok(Some(pi)) => evaluate_path_info_member(MemberEvalContext {
            requested_path: path,
            path_id,
            pi: &pi,
            source: src,
            policy,
            is_remote: true,
        }),
        Ok(None) => (
            unavailable_member(UnavailableMemberArgs {
                path_id: path_id.to_string(),
                source: src,
                state: RuntimeClosureMemberState::MissingMetadata,
                detail: "no local or remote PathInfo metadata".to_string(),
            }),
            Vec::new(),
        ),
        Err(detail) => (
            unavailable_member(UnavailableMemberArgs {
                path_id: path_id.to_string(),
                source: src,
                state: RuntimeClosureMemberState::LookupFailed,
                detail,
            }),
            Vec::new(),
        ),
    }
}

struct MemberEvalContext<'a> {
    requested_path: &'a StorePath<String>,
    path_id: &'a str,
    pi: &'a PathInfo,
    source: RuntimeClosureSourceKind,
    policy: &'a RuntimeClosureMetadataPolicy,
    is_remote: bool,
}

fn evaluate_path_info_member(ctx: MemberEvalContext<'_>) -> (RuntimeClosureMemberInput, Vec<StorePath<String>>) {
    assert!(!ctx.path_id.is_empty());
    assert!(!ctx.policy.store_dir.is_empty());
    if ctx.pi.store_path != *ctx.requested_path {
        let detail = format!("digest collision: requested {}, observed {}", ctx.requested_path, ctx.pi.store_path);
        return (
            unavailable_member(UnavailableMemberArgs {
                path_id: ctx.path_id.to_string(),
                source: ctx.source,
                state: RuntimeClosureMemberState::IdentityConflict,
                detail,
            }),
            Vec::new(),
        );
    }
    let (trust, state, detail) = evaluate_member_trust(ctx.pi, ctx.policy, ctx.is_remote);
    if state != RuntimeClosureMemberState::Observed {
        let reason = match &detail {
            Some(text) => text.clone(),
            None => "untrusted-metadata".to_string(),
        };
        return (
            RuntimeClosureMemberInput {
                path_id: ctx.path_id.to_string(),
                nar_sha256_hex: None,
                nar_size_bytes: None,
                size_unavailable_reason: Some(reason.clone()),
                references: None,
                references_unavailable_reason: Some(reason),
                source: ctx.source,
                trust,
                state,
                detail,
            },
            Vec::new(),
        );
    }

    let nar_sha256_hex = Some(HEXLOWER.encode(&ctx.pi.nar_sha256));
    let nar_size_bytes = Some(ctx.pi.nar_size);

    let mut child_refs = ctx.pi.references.clone();
    child_refs.sort_by_key(StorePath::to_string);
    child_refs.dedup();
    let ref_strings: Vec<String> = child_refs
        .iter()
        .map(|ref_path| ref_path.to_absolute_path_with_prefix(&ctx.policy.store_dir))
        .collect();
    (
        RuntimeClosureMemberInput {
            path_id: ctx.path_id.to_string(),
            nar_sha256_hex,
            nar_size_bytes,
            size_unavailable_reason: None,
            references: Some(ref_strings),
            references_unavailable_reason: None,
            source: ctx.source,
            trust,
            state: RuntimeClosureMemberState::Observed,
            detail,
        },
        child_refs,
    )
}

fn evaluate_member_trust(
    pi: &PathInfo,
    policy: &RuntimeClosureMetadataPolicy,
    is_remote: bool,
) -> (RuntimeClosureTrustStatus, RuntimeClosureMemberState, Option<String>) {
    assert!(!policy.store_dir.is_empty());
    assert!(!pi.store_path.to_string().is_empty());
    let total_signature_count = crate::gc::saturating_u32(pi.signatures.len());
    if pi.signatures.is_empty() {
        let is_admitted = !is_remote && policy.is_allow_unsigned_local;
        let state = if is_admitted {
            RuntimeClosureMemberState::Observed
        } else {
            RuntimeClosureMemberState::UntrustedSource
        };
        let detail = if is_admitted {
            None
        } else {
            Some("unsigned metadata is not admitted by policy".to_string())
        };
        return (RuntimeClosureTrustStatus::Unsigned, state, detail);
    }

    let refs: Vec<StorePathRef<'_>> = pi.references.iter().map(StorePath::as_ref).collect();
    let fp = fingerprint_with_store_dir(
        &pi.store_path.as_ref(),
        &pi.nar_sha256,
        pi.nar_size,
        refs.iter(),
        &policy.store_dir,
    );
    let trusted_count = pi
        .signatures
        .iter()
        .filter(|sig| policy.trusted_public_keys.iter().any(|key| key.verify(&fp, &sig.as_ref())))
        .count();
    let trusted_signature_count = crate::gc::saturating_u32(trusted_count);

    if trusted_signature_count > 0 {
        return (
            RuntimeClosureTrustStatus::Trusted {
                trusted_signature_count,
                total_signature_count,
            },
            RuntimeClosureMemberState::Observed,
            None,
        );
    }
    if policy.trusted_public_keys.is_empty() && !is_remote && policy.is_allow_unsigned_local {
        return (
            RuntimeClosureTrustStatus::NotVerified {
                reason: "no-trusted-keys-configured".to_string(),
            },
            RuntimeClosureMemberState::Observed,
            None,
        );
    }
    if !is_remote && policy.is_allow_unsigned_local {
        return (
            RuntimeClosureTrustStatus::Untrusted { total_signature_count },
            RuntimeClosureMemberState::Observed,
            None,
        );
    }
    (
        RuntimeClosureTrustStatus::Untrusted { total_signature_count },
        RuntimeClosureMemberState::UntrustedSource,
        Some("no signature matches configured trusted public keys".to_string()),
    )
}

struct UnavailableMemberArgs {
    path_id: String,
    source: RuntimeClosureSourceKind,
    state: RuntimeClosureMemberState,
    detail: String,
}

fn unavailable_member(args: UnavailableMemberArgs) -> RuntimeClosureMemberInput {
    assert!(!args.path_id.is_empty());
    assert!(!args.detail.is_empty());
    RuntimeClosureMemberInput {
        path_id: args.path_id,
        nar_sha256_hex: None,
        nar_size_bytes: None,
        size_unavailable_reason: Some(args.detail.clone()),
        references: None,
        references_unavailable_reason: Some(args.detail.clone()),
        source: args.source,
        trust: RuntimeClosureTrustStatus::NotVerified {
            reason: String::from(args.state.as_str()),
        },
        state: args.state,
        detail: Some(args.detail),
    }
}

fn budget_exhausted_member(
    path: &StorePath<String>,
    policy: &RuntimeClosureMetadataPolicy,
) -> RuntimeClosureMemberInput {
    assert!(!policy.store_dir.is_empty());
    let path_id = path.to_absolute_path_with_prefix(&policy.store_dir);
    unavailable_member(UnavailableMemberArgs {
        path_id,
        source: RuntimeClosureSourceKind::Unavailable,
        state: RuntimeClosureMemberState::BudgetExhausted,
        detail: format!("elapsed time exceeded {}ms", policy.limits.max_elapsed_ms),
    })
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;
    use std::collections::BTreeSet;

    use async_trait::async_trait;
    use futures::stream::StreamExt;
    use snix_store::path_info::PathInfo;
    use snix_store::pathinfoservice::PathInfoService;
    use snix_store::pathinfoservice::{self};

    use super::*;

    #[derive(Default, Clone)]
    struct MockPathInfoService {
        entries: std::sync::Arc<std::sync::Mutex<BTreeMap<[u8; 20], PathInfo>>>,
        failed_gets: std::sync::Arc<std::sync::Mutex<BTreeMap<[u8; 20], String>>>,
        get_counts: std::sync::Arc<std::sync::Mutex<BTreeMap<[u8; 20], u32>>>,
        reference_counts: std::sync::Arc<std::sync::Mutex<BTreeMap<[u8; 20], u32>>>,
    }

    impl MockPathInfoService {
        fn insert(&self, pi: PathInfo) {
            self.entries.lock().unwrap().insert(*pi.store_path.digest(), pi);
        }

        fn fail_get(&self, sp: &StorePath<String>, message: &str) {
            self.failed_gets.lock().unwrap().insert(*sp.digest(), message.to_string());
        }

        fn get_count(&self, sp: &StorePath<String>) -> u32 {
            self.get_counts.lock().unwrap().get(sp.digest()).copied().unwrap_or(0)
        }

        fn reference_count(&self, sp: &StorePath<String>) -> u32 {
            self.reference_counts.lock().unwrap().get(sp.digest()).copied().unwrap_or(0)
        }

        fn bump_count(counts: &std::sync::Mutex<BTreeMap<[u8; 20], u32>>, digest: [u8; 20]) {
            let mut counts = counts.lock().unwrap();
            let next = counts.get(&digest).copied().unwrap_or(0).saturating_add(1);
            counts.insert(digest, next);
        }
    }

    #[async_trait]
    impl PathInfoService for MockPathInfoService {
        async fn get(&self, digest: [u8; 20]) -> Result<Option<PathInfo>, pathinfoservice::Error> {
            Self::bump_count(&self.get_counts, digest);
            if let Some(message) = self.failed_gets.lock().unwrap().get(&digest).cloned() {
                return Err(std::io::Error::other(message).into());
            }
            Ok(self.entries.lock().unwrap().get(&digest).cloned())
        }

        async fn get_references(
            &self,
            digest: [u8; 20],
        ) -> Result<Option<Vec<StorePath<String>>>, pathinfoservice::Error> {
            Self::bump_count(&self.reference_counts, digest);
            if let Some(message) = self.failed_gets.lock().unwrap().get(&digest).cloned() {
                return Err(std::io::Error::other(message).into());
            }
            Ok(self.entries.lock().unwrap().get(&digest).map(|pi| pi.references.clone()))
        }

        async fn put(&self, pi: PathInfo) -> Result<PathInfo, pathinfoservice::Error> {
            self.entries.lock().unwrap().insert(*pi.store_path.digest(), pi.clone());
            Ok(pi)
        }

        fn list(&self) -> futures::stream::BoxStream<'static, Result<PathInfo, pathinfoservice::Error>> {
            let entries: Vec<_> = self.entries.lock().unwrap().values().cloned().collect();
            futures::stream::iter(entries.into_iter().map(Ok)).boxed()
        }
    }

    fn make_sp(name: &str, seed: u8) -> StorePath<String> {
        let mut digest = [0u8; 20];
        digest[0] = seed;
        for (i, b) in name.as_bytes().iter().enumerate() {
            digest[(i + 1) % 20] = *b;
        }
        StorePath::from_name_and_digest_fixed(name, digest).expect("valid store path")
    }

    fn make_pi(sp: &StorePath<String>, refs: Vec<StorePath<String>>) -> PathInfo {
        PathInfo {
            store_path: sp.clone(),
            node: snix_castore::Node::Symlink {
                target: snix_castore::SymlinkTarget::try_from("x").unwrap(),
            },
            references: refs,
            nar_size: 0,
            nar_sha256: [0u8; 32],
            signatures: vec![],
            deriver: None,
            ca: None,
        }
    }

    #[tokio::test]
    async fn single_path_no_refs() {
        let local = MockPathInfoService::default();
        let sp = make_sp("hello", 1);
        local.insert(make_pi(&sp, vec![]));

        let closure = resolve_closure(&sp, &local, None, StoreFallbackMode::Practical, "/nix/store").await.unwrap();
        assert!(closure.audit_events.is_empty());
        assert_eq!(closure.paths.len(), 1);
        assert_eq!(closure.paths[0], sp);
    }

    #[tokio::test]
    async fn linear_chain() {
        let local = MockPathInfoService::default();
        let c = make_sp("c", 3);
        let b = make_sp("b", 2);
        let a = make_sp("a", 1);

        local.insert(make_pi(&c, vec![]));
        local.insert(make_pi(&b, vec![c.clone()]));
        local.insert(make_pi(&a, vec![b.clone()]));

        let closure = resolve_closure(&a, &local, None, StoreFallbackMode::Practical, "/nix/store").await.unwrap();
        assert!(closure.audit_events.is_empty());
        assert_eq!(closure.paths.len(), 3);
        let digests: BTreeSet<_> = closure.paths.iter().map(|p| *p.digest()).collect();
        assert!(digests.contains(a.digest()));
        assert!(digests.contains(b.digest()));
        assert!(digests.contains(c.digest()));
    }

    #[tokio::test]
    async fn diamond_dedup() {
        let local = MockPathInfoService::default();
        let d = make_sp("d", 4);
        let c = make_sp("c", 3);
        let b = make_sp("b", 2);
        let a = make_sp("a", 1);

        local.insert(make_pi(&d, vec![]));
        local.insert(make_pi(&c, vec![d.clone()]));
        local.insert(make_pi(&b, vec![d.clone()]));
        local.insert(make_pi(&a, vec![b.clone(), c.clone()]));

        let closure = resolve_closure(&a, &local, None, StoreFallbackMode::Practical, "/nix/store").await.unwrap();
        assert!(closure.audit_events.is_empty());
        assert_eq!(closure.paths.len(), 4, "diamond should dedup D");
    }

    #[tokio::test]
    async fn cycle_terminates() {
        let local = MockPathInfoService::default();
        let b = make_sp("b", 2);
        let a = make_sp("a", 1);

        local.insert(make_pi(&a, vec![b.clone()]));
        local.insert(make_pi(&b, vec![a.clone()]));

        let closure = resolve_closure(&a, &local, None, StoreFallbackMode::Practical, "/nix/store").await.unwrap();
        assert!(closure.audit_events.is_empty());
        assert_eq!(closure.paths.len(), 2);
    }

    #[tokio::test]
    async fn practical_mode_records_degraded_child_lookup() {
        let local = MockPathInfoService::default();
        let b = make_sp("b", 2);
        let a = make_sp("a", 1);

        local.insert(make_pi(&a, vec![b.clone()]));

        let closure = resolve_closure(&a, &local, None, StoreFallbackMode::Practical, "/crunch/store").await.unwrap();
        assert_eq!(closure.paths.len(), 2);
        assert_eq!(closure.audit_events.len(), 1);
        assert_eq!(closure.audit_events[0].kind, StoreAuditKind::ClosureResolutionDegraded);
        assert!(closure.audit_events[0].detail.contains(&b.to_absolute_path_with_prefix("/crunch/store")));
        assert!(!closure.audit_events[0].detail.contains("/nix/store"));
    }

    #[tokio::test]
    async fn remote_fallback() {
        let local = MockPathInfoService::default();
        let remote = MockPathInfoService::default();
        let b = make_sp("b", 2);
        let a = make_sp("a", 1);

        local.insert(make_pi(&a, vec![b.clone()]));
        remote.insert(make_pi(&b, vec![]));

        let closure = resolve_closure(&a, &local, Some(&remote), StoreFallbackMode::Practical, "/nix/store")
            .await
            .unwrap();
        assert!(closure.audit_events.is_empty());
        assert_eq!(closure.paths.len(), 2);
        assert_eq!(remote.reference_count(&b), 1);
        assert_eq!(remote.get_count(&b), 0);
    }

    #[tokio::test]
    async fn practical_mode_records_degraded_root_lookup() {
        let local = MockPathInfoService::default();
        let a = make_sp("a", 1);

        let closure = resolve_closure(&a, &local, None, StoreFallbackMode::Practical, "/crunch/store").await.unwrap();
        assert_eq!(closure.paths.len(), 1);
        assert_eq!(closure.paths[0], a);
        assert_eq!(closure.audit_events.len(), 1);
        assert_eq!(closure.audit_events[0].kind, StoreAuditKind::ClosureResolutionDegraded);
        assert!(closure.audit_events[0].detail.contains("/crunch/store"));
    }

    #[tokio::test]
    async fn strict_mode_rejects_missing_root_closure_facts() {
        let local = MockPathInfoService::default();
        let a = make_sp("a", 1);

        let err = resolve_closure(&a, &local, None, StoreFallbackMode::Strict, "/crunch/store").await.unwrap_err();
        assert!(
            matches!(err, Error::MissingClosureFacts { ref path, ref store_dir, .. } if *path == a && store_dir == "/crunch/store")
        );
        assert!(err.to_string().contains("/crunch/store"));
        assert!(!err.to_string().contains("/nix/store"));
    }

    #[tokio::test]
    async fn self_reference() {
        let local = MockPathInfoService::default();
        let a = make_sp("a", 1);
        local.insert(make_pi(&a, vec![a.clone()]));

        let closure = resolve_closure(&a, &local, None, StoreFallbackMode::Practical, "/nix/store").await.unwrap();
        assert!(closure.audit_events.is_empty());
        assert_eq!(closure.paths.len(), 1);
    }

    #[tokio::test]
    async fn depth_limit_enforced() {
        let local = MockPathInfoService::default();
        let depth = MAX_CLOSURE_DEPTH + 10;
        let mut paths: Vec<StorePath<String>> = Vec::new();

        for i in 0..=depth {
            let sp = make_sp(&format!("n{i}"), (i % 255) as u8);
            paths.push(sp);
        }

        for i in 0..depth as usize {
            local.insert(make_pi(&paths[i], vec![paths[i + 1].clone()]));
        }
        local.insert(make_pi(&paths[depth as usize], vec![]));

        let closure =
            resolve_closure(&paths[0], &local, None, StoreFallbackMode::Practical, "/nix/store").await.unwrap();
        assert!(closure.audit_events.is_empty());
        assert!(
            closure.paths.len() <= (MAX_CLOSURE_DEPTH as usize) + 2,
            "closure too large: {} (limit {})",
            closure.paths.len(),
            MAX_CLOSURE_DEPTH + 2
        );
    }

    #[tokio::test]
    async fn local_query_error_can_still_use_remote_refs() {
        let local = MockPathInfoService::default();
        let remote = MockPathInfoService::default();
        let a = make_sp("a", 1);

        local.fail_get(&a, "disk offline");
        remote.insert(make_pi(&a, vec![]));

        let closure = resolve_closure(&a, &local, Some(&remote), StoreFallbackMode::Practical, "/nix/store")
            .await
            .unwrap();
        assert_eq!(closure.paths.len(), 1);
        assert!(closure.audit_events.is_empty());
        assert_eq!(remote.reference_count(&a), 1);
        assert_eq!(remote.get_count(&a), 0);
    }

    #[tokio::test]
    async fn remote_metadata_lookup_failure_degrades_practical_mode() {
        let local = MockPathInfoService::default();
        let remote = MockPathInfoService::default();
        let b = make_sp("b", 2);
        let a = make_sp("a", 1);

        local.insert(make_pi(&a, vec![b.clone()]));
        remote.fail_get(&b, "narinfo offline");

        let closure = resolve_closure(&a, &local, Some(&remote), StoreFallbackMode::Practical, "/crunch/store")
            .await
            .unwrap();
        assert_eq!(closure.paths.len(), 2);
        assert_eq!(closure.audit_events.len(), 1);
        assert_eq!(closure.audit_events[0].kind, StoreAuditKind::ClosureResolutionDegraded);
        assert!(closure.audit_events[0].detail.contains("narinfo offline"));
        assert_eq!(remote.reference_count(&b), 1);
        assert_eq!(remote.get_count(&b), 0);
    }

    #[tokio::test]
    async fn runtime_closure_metadata_reads_metadata_only_and_enforces_trust_and_collision() {
        let local = MockPathInfoService::default();
        let remote = MockPathInfoService::default();
        let (signing_key, verifying_key) = nix_compat::narinfo::parse_keypair(
            "cache.example.com-1:cCta2MEsRNuYCgWYyeRXLyfoFpKhQJKn8gLMeXWAb7vIpRKKo/3JoxJ24OYa3DxT2JVV38KjK/1ywHWuMe2JEw==",
        )
        .unwrap();
        let store_dir = "/mantle/store";
        let root = make_sp("root", 1);
        let remote_dep = make_sp("remote-dep", 2);
        let untrusted_dep = make_sp("untrusted-dep", 3);
        let drv = make_sp("build-only.drv", 4);

        let mut root_pi = make_pi(&root, vec![remote_dep.clone(), untrusted_dep.clone()]);
        root_pi.nar_size = 120;
        root_pi.deriver = Some(drv.clone());
        local.insert(root_pi);

        let mut remote_pi = make_pi(&remote_dep, vec![]);
        remote_pi.nar_size = 300;
        let refs: Vec<_> = remote_pi.references.iter().map(StorePath::as_ref).collect();
        let fp = fingerprint_with_store_dir(
            &remote_pi.store_path.as_ref(),
            &remote_pi.nar_sha256,
            remote_pi.nar_size,
            refs.iter(),
            store_dir,
        );
        remote_pi.signatures.push(signing_key.sign(fp.as_bytes()).to_owned());
        remote.insert(remote_pi);

        // Unsigned remote metadata must be refused as UntrustedSource.
        let mut bad_remote_pi = make_pi(&untrusted_dep, vec![]);
        bad_remote_pi.nar_size = 999;
        remote.insert(bad_remote_pi);

        let policy = RuntimeClosureMetadataPolicy {
            store_dir: store_dir.to_string(),
            limits: RuntimeClosureLimits::standard(),
            trusted_public_keys: vec![verifying_key],
            is_allow_unsigned_local: true,
            is_allow_remote_metadata: true,
            remote_source_id: Some("https://cache.example.com".to_string()),
        };

        let collected = collect_runtime_closure_metadata(std::slice::from_ref(&root), &local, Some(&remote), &policy)
            .await
            .unwrap();
        assert_eq!(collected.remote_metadata_request_count, 2);
        assert_eq!(collected.members.len(), 3);
        assert!(collected.members.iter().all(|m| !m.path_id.ends_with("build-only.drv")));

        let untrusted_member = collected.members.iter().find(|m| m.path_id.ends_with("untrusted-dep")).unwrap();
        assert_eq!(untrusted_member.state, RuntimeClosureMemberState::UntrustedSource);
        assert!(untrusted_member.references.is_none());

        let explained =
            crunch_gc_core::retention::explain_runtime_closure(crunch_gc_core::retention::RuntimeClosureRequest {
                store_prefix: store_dir.to_string(),
                roots: vec![root.to_absolute_path_with_prefix(store_dir)],
                limits: policy.limits,
                elapsed_ms: collected.elapsed_ms,
                members: collected.members,
            })
            .unwrap();
        assert_eq!(explained.completeness, crunch_gc_core::retention::RuntimeClosureCompleteness::Partial);
        assert!(
            explained
                .issues
                .iter()
                .any(|i| i.code == crunch_gc_core::retention::RuntimeClosureIssueCode::UntrustedSource)
        );
    }

    #[tokio::test]
    async fn runtime_closure_metadata_detects_digest_collision_and_time_budget() {
        let local = MockPathInfoService::default();
        let store_dir = "/mantle/store";
        let requested = make_sp("requested", 7);
        let mut colliding = make_pi(&make_sp("other-name", 7), vec![]);
        colliding.store_path = StorePath::from_name_and_digest_fixed("other-name", *requested.digest()).unwrap();
        colliding.nar_size = 55;
        local.insert(colliding);

        let policy = RuntimeClosureMetadataPolicy {
            store_dir: store_dir.to_string(),
            limits: RuntimeClosureLimits::standard(),
            trusted_public_keys: Vec::new(),
            is_allow_unsigned_local: true,
            is_allow_remote_metadata: false,
            remote_source_id: None,
        };
        let collected = collect_runtime_closure_metadata(std::slice::from_ref(&requested), &local, None, &policy)
            .await
            .unwrap();
        assert_eq!(collected.members[0].state, RuntimeClosureMemberState::IdentityConflict);
        assert!(collected.members[0].references.is_none());

        // Time budget test with deterministic clock callback.
        let source = ServiceMetadataSource {
            local: &local,
            remote: None,
        };
        let mut ticks = [0_u64, 10_000_u64].into_iter();
        let mut clock = || ticks.next().unwrap_or(10_000);
        let timed_out = collect_runtime_closure_metadata_from_source(&[requested], &source, &policy, &mut clock)
            .await
            .unwrap();
        assert_eq!(timed_out.members[0].state, RuntimeClosureMemberState::BudgetExhausted);
    }

    #[tokio::test]
    async fn runtime_closure_metadata_strips_size_and_hash_from_untrusted_members() {
        let local = MockPathInfoService::default();
        let remote = MockPathInfoService::default();
        let store_dir = "/mantle/store";

        let raw_signing = ed25519_dalek::SigningKey::from_bytes(&[15_u8; 32]);
        let verifying_key = VerifyingKey::new("trusted-1".to_string(), raw_signing.verifying_key());
        let signing_key = nix_compat::narinfo::SigningKey::new("trusted-1".to_string(), raw_signing);

        let untrusted_root = make_sp("untrusted-root", 11);
        let mut untrusted_root_pi = make_pi(&untrusted_root, vec![]);
        untrusted_root_pi.nar_size = u64::MAX;
        remote.insert(untrusted_root_pi);

        let local_root = make_sp("local-root", 12);
        let trusted_child = make_sp("trusted-child", 13);
        let untrusted_child = make_sp("untrusted-child", 14);

        let mut local_root_pi = make_pi(&local_root, vec![trusted_child.clone(), untrusted_child.clone()]);
        local_root_pi.nar_size = 100;
        local.insert(local_root_pi);

        let mut trusted_child_pi = make_pi(&trusted_child, vec![]);
        trusted_child_pi.nar_size = 250;
        let refs: Vec<_> = trusted_child_pi.references.iter().map(StorePath::as_ref).collect();
        let fp = fingerprint_with_store_dir(
            &trusted_child_pi.store_path.as_ref(),
            &trusted_child_pi.nar_sha256,
            trusted_child_pi.nar_size,
            refs.iter(),
            store_dir,
        );
        trusted_child_pi.signatures.push(signing_key.sign(fp.as_bytes()).to_owned());
        remote.insert(trusted_child_pi);

        let mut untrusted_child_pi = make_pi(&untrusted_child, vec![]);
        untrusted_child_pi.nar_size = u64::MAX;
        remote.insert(untrusted_child_pi);

        let policy = RuntimeClosureMetadataPolicy {
            store_dir: store_dir.to_string(),
            limits: RuntimeClosureLimits::standard(),
            trusted_public_keys: vec![verifying_key],
            is_allow_unsigned_local: true,
            is_allow_remote_metadata: true,
            remote_source_id: Some("https://cache.example.com".to_string()),
        };

        let root_collected =
            collect_runtime_closure_metadata(&[untrusted_root], &local, Some(&remote), &policy).await.unwrap();
        assert_eq!(root_collected.members[0].state, RuntimeClosureMemberState::UntrustedSource);
        assert!(root_collected.members[0].nar_size_bytes.is_none());
        assert!(root_collected.members[0].nar_sha256_hex.is_none());
        assert!(root_collected.members[0].size_unavailable_reason.is_some());

        let child_collected =
            collect_runtime_closure_metadata(std::slice::from_ref(&local_root), &local, Some(&remote), &policy)
                .await
                .unwrap();
        let untrusted_member = child_collected.members.iter().find(|m| m.path_id.ends_with("untrusted-child")).unwrap();
        assert_eq!(untrusted_member.state, RuntimeClosureMemberState::UntrustedSource);
        assert!(untrusted_member.nar_size_bytes.is_none());
        assert!(untrusted_member.nar_sha256_hex.is_none());
        assert!(untrusted_member.size_unavailable_reason.is_some());

        let explained =
            crunch_gc_core::retention::explain_runtime_closure(crunch_gc_core::retention::RuntimeClosureRequest {
                store_prefix: store_dir.to_string(),
                roots: vec![local_root.to_absolute_path_with_prefix(store_dir)],
                limits: policy.limits,
                elapsed_ms: child_collected.elapsed_ms,
                members: child_collected.members,
            })
            .unwrap();
        assert_eq!(explained.completeness, crunch_gc_core::retention::RuntimeClosureCompleteness::Partial);
        assert_eq!(explained.observed_known_nar_bytes, 350);
        assert!(
            explained
                .issues
                .iter()
                .any(|i| i.code == crunch_gc_core::retention::RuntimeClosureIssueCode::UntrustedSource)
        );
    }

    #[tokio::test]
    async fn collect_runtime_closure_metadata_rejects_empty_roots_or_invalid_store_dir_without_panicking() {
        let local = MockPathInfoService::default();
        let root = make_sp("root", 21);

        let empty_roots_policy = RuntimeClosureMetadataPolicy {
            store_dir: "/mantle/store".to_string(),
            limits: RuntimeClosureLimits::standard(),
            trusted_public_keys: Vec::new(),
            is_allow_unsigned_local: true,
            is_allow_remote_metadata: false,
            remote_source_id: None,
        };
        let empty_err = collect_runtime_closure_metadata(&[], &local, None, &empty_roots_policy).await.unwrap_err();
        assert!(empty_err.to_string().contains("at least one root"), "{empty_err}");

        let empty_dir_policy = RuntimeClosureMetadataPolicy {
            store_dir: String::new(),
            ..empty_roots_policy.clone()
        };
        let empty_dir_err =
            collect_runtime_closure_metadata(std::slice::from_ref(&root), &local, None, &empty_dir_policy)
                .await
                .unwrap_err();
        assert!(empty_dir_err.to_string().contains("store_dir must be absolute"), "{empty_dir_err}");

        let non_ascii_dir_policy = RuntimeClosureMetadataPolicy {
            store_dir: "/mantle/st\u{00f6}re".to_string(),
            ..empty_roots_policy
        };
        let ascii_err =
            collect_runtime_closure_metadata(&[root], &local, None, &non_ascii_dir_policy).await.unwrap_err();
        assert!(ascii_err.to_string().contains("ASCII"), "{ascii_err}");
    }

    #[tokio::test]
    async fn collect_runtime_closure_metadata_keys_visited_by_full_store_path_and_enforces_max_edges_count() {
        let local = MockPathInfoService::default();
        let store_dir = "/mantle/store";

        // Two distinct store paths sharing the same 20-byte digest.
        let same_digest_a = StorePath::from_name_and_digest_fixed("alpha", [33_u8; 20]).unwrap();
        let same_digest_b = StorePath::from_name_and_digest_fixed("beta", [33_u8; 20]).unwrap();
        let root = make_sp("digest-root", 34);

        local.insert(make_pi(&same_digest_a, vec![]));
        local.insert(make_pi(&root, vec![same_digest_a.clone(), same_digest_b.clone()]));

        let policy = RuntimeClosureMetadataPolicy {
            store_dir: store_dir.to_string(),
            limits: RuntimeClosureLimits::standard(),
            trusted_public_keys: Vec::new(),
            is_allow_unsigned_local: true,
            is_allow_remote_metadata: false,
            remote_source_id: None,
        };
        let collected = collect_runtime_closure_metadata(&[root], &local, None, &policy).await.unwrap();
        assert_eq!(collected.members.len(), 3);
        let alpha = collected.members.iter().find(|m| m.path_id.ends_with("alpha")).unwrap();
        let beta = collected.members.iter().find(|m| m.path_id.ends_with("beta")).unwrap();
        assert_eq!(alpha.state, RuntimeClosureMemberState::Observed);
        assert_eq!(beta.state, RuntimeClosureMemberState::IdentityConflict);

        // Enforce max_edges_count = 1 even when max_nodes_count is large and remote has 5 children.
        let remote = MockPathInfoService::default();
        let edge_local = MockPathInfoService::default();
        let mut children = Vec::new();
        for idx in 1..=5_u8 {
            let child = make_sp(&format!("child-{idx}"), 40 + idx);
            remote.insert(make_pi(&child, vec![]));
            children.push(child);
        }
        let edge_root = make_sp("edge-root", 40);
        edge_local.insert(make_pi(&edge_root, children));

        let mut edge_limits = RuntimeClosureLimits::standard();
        edge_limits.max_edges_count = 1;
        edge_limits.max_nodes_count = 100;
        let edge_policy = RuntimeClosureMetadataPolicy {
            store_dir: store_dir.to_string(),
            limits: edge_limits,
            trusted_public_keys: Vec::new(),
            is_allow_unsigned_local: true,
            is_allow_remote_metadata: true,
            remote_source_id: Some("https://cache.example.com".to_string()),
        };
        let bounded = collect_runtime_closure_metadata(&[edge_root], &edge_local, Some(&remote), &edge_policy)
            .await
            .unwrap();
        assert_eq!(bounded.remote_metadata_request_count, 1);
        assert_eq!(bounded.members.len(), 2);
    }

    #[test]
    fn evaluate_member_trust_admits_local_snix_other_signer_when_unsigned_local_allowed_while_remote_refuses() {
        let store_dir = "/mantle/store";
        let raw_trusted = ed25519_dalek::SigningKey::from_bytes(&[51_u8; 32]);
        let trusted_verifying = VerifyingKey::new("local-key-1".to_string(), raw_trusted.verifying_key());

        let raw_other = ed25519_dalek::SigningKey::from_bytes(&[52_u8; 32]);
        let other_signing = nix_compat::narinfo::SigningKey::new("imported-other-1".to_string(), raw_other);

        let sp = make_sp("other-signed", 53);
        let mut pi = make_pi(&sp, vec![]);
        let empty_refs: [StorePathRef<'_>; 0] = [];
        let fp = fingerprint_with_store_dir(
            &pi.store_path.as_ref(),
            &pi.nar_sha256,
            pi.nar_size,
            empty_refs.iter(),
            store_dir,
        );
        pi.signatures.push(other_signing.sign(fp.as_bytes()).to_owned());

        let local_policy = RuntimeClosureMetadataPolicy {
            store_dir: store_dir.to_string(),
            limits: RuntimeClosureLimits::standard(),
            trusted_public_keys: vec![trusted_verifying.clone()],
            is_allow_unsigned_local: true,
            is_allow_remote_metadata: false,
            remote_source_id: None,
        };
        let (local_trust, local_state, local_detail) = evaluate_member_trust(&pi, &local_policy, false);
        assert_eq!(local_trust, RuntimeClosureTrustStatus::Untrusted {
            total_signature_count: 1,
        });
        assert_eq!(local_state, RuntimeClosureMemberState::Observed);
        assert!(local_detail.is_none());

        let (remote_trust, remote_state, remote_detail) = evaluate_member_trust(&pi, &local_policy, true);
        assert_eq!(remote_trust, RuntimeClosureTrustStatus::Untrusted {
            total_signature_count: 1,
        });
        assert_eq!(remote_state, RuntimeClosureMemberState::UntrustedSource);
        assert!(remote_detail.is_some());

        let strict_local_policy = RuntimeClosureMetadataPolicy {
            is_allow_unsigned_local: false,
            ..local_policy
        };
        let (_, strict_state, strict_detail) = evaluate_member_trust(&pi, &strict_local_policy, false);
        assert_eq!(strict_state, RuntimeClosureMemberState::UntrustedSource);
        assert!(strict_detail.is_some());
    }
}
