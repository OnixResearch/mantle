//! Build orchestration: recursively build derivations, check cache,
//! persist outputs.
use std::collections::BTreeMap;
use std::collections::HashMap;
use std::collections::HashSet;
use std::path::Path;
use std::path::PathBuf;
use std::sync::Arc;

use crunch_store::ArtifactProvenance;
use crunch_store::GcRootSource;
use crunch_store::OutputSubstitutionReport;
use nix_compat::derivation::Derivation;
use nix_compat::narinfo::VerifyingKey;
use nix_compat::store_path::StorePath;
use snix_build::buildservice::BuildService;
use snix_castore::Node;
use snix_castore::blobservice::BlobService;
use snix_castore::directoryservice::DirectoryService;
use snix_castore::import::fs::ingest_path;
use snix_store::nar::NarCalculationService;
use snix_store::nar::SimpleRenderer;
use snix_store::path_info::PathInfo;
use snix_store::pathinfoservice::PathInfoService;
use tracing::debug;
use tracing::info;

use crate::Error;
use crate::HermeticityAuditEvent;
use crate::HermeticityMode;
use crate::build_request::collect_input_paths;
use crate::build_request::derivation_to_build_request;
use crate::fod::verify_fod_hash;
use crate::references::resolve_references;
use crate::registry::DerivationRegistry;
use crate::signing::KeyPair;
use crate::signing::{self};

/// Apply a sequence of byte-level rewrites to a castore node.
/// Used for transitive CA input path replacement.
async fn apply_input_rewrites(
    node: &Node,
    rewrites: &[(String, String)],
    blob_service: &(impl BlobService + Clone),
    directory_service: &(impl DirectoryService + Clone),
) -> Result<Node, Error> {
    let mut current = node.clone();
    for (old_placeholder, new_path) in rewrites {
        let (rewritten, _) = crate::rewrite::rewrite_node(
            &current,
            old_placeholder.as_bytes(),
            new_path.as_bytes(),
            blob_service,
            directory_service,
        )
        .await?;
        current = rewritten;
    }
    Ok(current)
}

/// Compute the CA field stored in PathInfo for an output.
fn output_ca_field(
    is_ca: bool,
    output: &nix_compat::derivation::Output,
    nar_sha256: [u8; 32],
) -> Option<nix_compat::nixhash::CAHash> {
    if is_ca {
        Some(nix_compat::nixhash::CAHash::Nar(nix_compat::nixhash::NixHash::Sha256(nar_sha256)))
    } else {
        output.ca_hash.clone()
    }
}

fn push_unique_store_paths(
    ordered_paths: &mut Vec<StorePath<String>>,
    seen_paths: &mut HashSet<StorePath<String>>,
    new_paths: impl IntoIterator<Item = StorePath<String>>,
) {
    for path in new_paths {
        let inserted = seen_paths.insert(path.clone());
        if inserted {
            ordered_paths.push(path);
        }
    }
}

/// The result of building a single derivation.
#[derive(Debug, Clone)]
pub struct BuildOutcome {
    /// The derivation store path (the .drv path).
    pub drv_path: StorePath<String>,
    /// Output name → PathInfo for each output.
    pub outputs: HashMap<String, PathInfo>,
    /// Output name → substitution reporting for successful remote cache hits.
    pub substitutions: HashMap<String, OutputSubstitutionReport>,
    /// Whether the build was served from cache (output already existed).
    pub cached: bool,
    /// Captured build stdout+stderr, if available.
    pub log: Option<String>,
}

struct CacheCheckHit {
    infos: HashMap<String, PathInfo>,
    substitutions: HashMap<String, OutputSubstitutionReport>,
}

/// Metadata saved during `prepare_build`, consumed by `finish_build`.
/// Intermediate state for a single CA output during multi-output
/// CA derivation finalization (between pass 1 and pass 2).
struct CaOutputIntermediate {
    name: String,
    marked_node: Node,
    ca_path: StorePath<String>,
    nar_size: u64,
    nar_sha256: [u8; 32],
}

pub(crate) struct PreparedBuild {
    pub(crate) drv_path: StorePath<String>,
    pub(crate) drv_name: String,
    pub(crate) derivation: Arc<Derivation>,
    pub(crate) refscan_needles: Vec<String>,
    pub(crate) sandbox_inputs: BTreeMap<StorePath<String>, Node>,
    pub(crate) input_rewrites: Vec<(String, String)>,
    pub(crate) is_ca: bool,
    /// Whether this derivation is a user-requested root.
    /// Root outputs get exported to disk; intermediate deps stay in castore.
    pub(crate) is_root: bool,
}

/// Result of `prepare_build`: either already done (cached/fetcher) or
/// needs a sandbox build.
pub(crate) enum PrepareResult {
    Done(BuildOutcome),
    NeedsBuild {
        prepared: PreparedBuild,
        build_request: snix_build::buildservice::BuildRequest,
    },
}

/// Orchestrates the build pipeline: evaluating dependencies, checking
/// cache, running builds, persisting results.
///
/// `BServ` is the only remaining generic: the sandbox dispatch point.
/// All store operations (blob, directory, pathinfo) go through
/// `Arc<dyn ...>` services extracted from constructor args or a
/// `StoreHandle`.
pub struct Builder<BServ> {
    /// All store operations (blob, directory, pathinfo, cache, export,
    /// session caches, CA mappings) go through the StoreHandle.
    pub(crate) store: crunch_store::StoreHandle,
    build_service: Arc<BServ>,
    /// Signing keypair — every PathInfo gets signed before persistence.
    keypair: KeyPair,
    /// Trusted public keys for signature verification on cache hits.
    trusted_keys: Vec<VerifyingKey>,
    /// When true, skip signature verification on cache hits.
    trust_unsigned: bool,
    verbose: bool,
    hermeticity_mode: HermeticityMode,
    hermeticity_audit_events: Vec<HermeticityAuditEvent>,
    source_closure_cache: HashMap<StorePath<String>, Vec<StorePath<String>>>,
    root_retention_source: Option<GcRootSource>,
}

impl<BServ> Builder<BServ>
where BServ: BuildService + 'static
{
    /// Create a Builder from individual services.
    ///
    /// Accepts any types that implement the service traits. Internally
    /// wraps them in `Arc<dyn ...>` and constructs a StoreHandle.
    pub fn new<BS, DS, PIS>(
        blob_service: BS,
        directory_service: DS,
        build_service: BServ,
        pathinfo_service: PIS,
        output_dir: PathBuf,
        store_dir: &str,
        keypair: KeyPair,
        trusted_keys: Vec<VerifyingKey>,
        trust_unsigned: bool,
        verbose: bool,
    ) -> Self
    where
        BS: BlobService + 'static,
        DS: DirectoryService + 'static,
        PIS: PathInfoService + 'static,
    {
        let output_dir_str = output_dir.to_str().unwrap_or(store_dir).to_string();
        let store = crunch_store::StoreHandle::from_services_with_store_dir(
            Arc::new(blob_service) as Arc<dyn BlobService>,
            Arc::new(directory_service) as Arc<dyn DirectoryService>,
            Arc::new(pathinfo_service) as Arc<dyn PathInfoService>,
            None,
            PathBuf::from("/tmp/crunch-test"),
            output_dir_str,
            store_dir.to_string(),
        );
        Self {
            store,
            build_service: Arc::new(build_service),
            keypair,
            trusted_keys,
            trust_unsigned,
            verbose,
            hermeticity_mode: HermeticityMode::Practical,
            hermeticity_audit_events: Vec::new(),
            source_closure_cache: HashMap::new(),
            root_retention_source: None,
        }
    }

    /// Create a Builder with a state directory for persistent CA mappings
    /// and an optional remote PathInfoService for binary cache substitution.
    ///
    /// Accepts pre-wrapped `Arc<dyn ...>` services (e.g., from a StoreHandle).
    pub fn with_state_dir(
        blob_service: Arc<dyn BlobService>,
        directory_service: Arc<dyn DirectoryService>,
        build_service: BServ,
        pathinfo_service: Arc<dyn PathInfoService>,
        output_dir: PathBuf,
        state_dir: Option<PathBuf>,
        remote_pathinfo: Option<Arc<dyn PathInfoService>>,
        store_dir: &str,
        keypair: KeyPair,
        trusted_keys: Vec<VerifyingKey>,
        trust_unsigned: bool,
        verbose: bool,
    ) -> Self {
        let output_dir_str = output_dir.to_str().unwrap_or(store_dir).to_string();
        let sd = state_dir.unwrap_or_else(|| PathBuf::from("/tmp/crunch-no-state"));
        let store = crunch_store::StoreHandle::from_services_with_store_dir(
            blob_service,
            directory_service,
            pathinfo_service,
            remote_pathinfo,
            sd,
            output_dir_str,
            store_dir.to_string(),
        );
        Self {
            store,
            build_service: Arc::new(build_service),
            keypair,
            trusted_keys,
            trust_unsigned,
            verbose,
            hermeticity_mode: HermeticityMode::Practical,
            hermeticity_audit_events: Vec::new(),
            source_closure_cache: HashMap::new(),
            root_retention_source: None,
        }
    }

    /// Build a derivation and all its dependencies. Returns the outcome.
    ///
    /// Delegates to `build_all` with a single root and max_jobs=1.
    /// Get a cloned Arc to the build service (for spawning tasks).
    pub(crate) fn build_service(&self) -> Arc<BServ> {
        self.build_service.clone()
    }

    /// The logical store directory prefix.
    pub fn store_dir(&self) -> &str {
        self.store.store_dir()
    }

    pub fn set_hermeticity_mode(&mut self, hermeticity_mode: HermeticityMode) {
        self.hermeticity_mode = hermeticity_mode;
    }

    pub fn set_root_retention_source(&mut self, root_retention_source: Option<GcRootSource>) {
        self.root_retention_source = root_retention_source;
    }

    pub fn take_hermeticity_audit_events(&mut self) -> Vec<HermeticityAuditEvent> {
        std::mem::take(&mut self.hermeticity_audit_events)
    }

    /// Read the full content of a blob from castore.
    ///
    /// Used by the Worker to read `.drv` files from build outputs
    /// (dynamic derivation detection).
    pub(crate) async fn read_blob(&self, node: &snix_castore::Node) -> Result<Vec<u8>, Error> {
        use tokio::io::AsyncReadExt;

        let digest = match node {
            snix_castore::Node::File { digest, size, .. } => {
                // Tiger Style: fixed limit.
                const MAX_BLOB_READ: u64 = 8 * 1024 * 1024;
                if *size > MAX_BLOB_READ {
                    return Err(Error::Store(format!("blob too large to read: {size} bytes (limit: {MAX_BLOB_READ})")));
                }
                digest.clone()
            }
            other => {
                return Err(Error::Store(format!("read_blob called on non-file node: {other:?}")));
            }
        };

        let mut reader = self
            .store
            .blob_service()
            .open_read(&digest)
            .await
            .map_err(|e| Error::Store(format!("opening blob: {e}")))?
            .ok_or_else(|| {
                Error::Store(format!(
                    "blob not found in castore: {}",
                    data_encoding::HEXLOWER.encode(digest.as_slice())
                ))
            })?;

        let mut buf = Vec::new();
        reader.read_to_end(&mut buf).await.map_err(|e| Error::Store(format!("reading blob: {e}")))?;

        Ok(buf)
    }

    /// Prefer `build_all` when building multiple roots.
    pub async fn build(
        &mut self,
        drv_path: &StorePath<String>,
        known_paths: &mut DerivationRegistry,
    ) -> Result<BuildOutcome, Error> {
        let mut outcomes = self.build_all(&[drv_path.clone()], known_paths, 1).await?;
        outcomes.pop().ok_or_else(|| Error::DerivationNotFound { path: drv_path.clone() })
    }

    /// Build multiple root derivations and all their dependencies,
    /// using the lazy goal-based scheduler.
    ///
    /// Creates goals lazily via `Worker::want()`, then runs the
    /// dispatch loop which handles cache hits inline and spawns
    /// sandbox builds concurrently up to `max_jobs`.
    ///
    /// Returns outcomes for root derivations only.
    pub async fn build_all(
        &mut self,
        roots: &[StorePath<String>],
        known_paths: &mut DerivationRegistry,
        max_jobs: u32,
    ) -> Result<Vec<BuildOutcome>, Error> {
        use crate::worker::Worker;

        if roots.is_empty() {
            return Ok(Vec::new());
        }

        debug_assert!(max_jobs >= 1, "max_jobs must be at least 1");

        let mut worker = Worker::new(max_jobs);

        for root in roots {
            worker.want(root, known_paths, true)?;
        }

        let result = worker.run(self, known_paths).await?;

        if !result.failed.is_empty() {
            return Err(Error::Store(format!("{} root build(s) failed", result.failed.len(),)));
        }

        Ok(result.outcomes)
    }

    /// Prepare a single derivation for building. Handles cache hits and
    /// fetchers inline; for sandbox builds, returns the prepared metadata
    /// needed to dispatch `do_build` and later `finish_build`.
    pub(crate) async fn prepare_build(
        &mut self,
        drv_path: &StorePath<String>,
        derivation: Arc<Derivation>,
        known_paths: &mut DerivationRegistry,
        is_root: bool,
    ) -> Result<PrepareResult, Error> {
        let drv_name = drv_path.name().to_string();
        let derivation_ref = derivation.as_ref();

        // 1. Cache check
        if let Some(cached_hit) = self.check_cache(drv_path, derivation_ref, is_root).await? {
            self.record_cached_output_paths(drv_path, derivation_ref, &cached_hit.infos, known_paths)?;
            info!(drv = %drv_name, "all outputs cached, skipping build");
            return Ok(PrepareResult::Done(BuildOutcome {
                drv_path: drv_path.clone(),
                outputs: cached_hit.infos,
                substitutions: cached_hit.substitutions,
                cached: true,
                log: None,
            }));
        }

        // 2. Ensure input derivation outputs are in castore.
        for (input_drv_path, _output_names) in &derivation_ref.input_derivations {
            let input_abs = input_drv_path.to_absolute_path_with_prefix(self.store.store_dir());
            if let Some(entry) = known_paths.get_by_drv_path(&input_abs) {
                self.ensure_input_nodes(entry.derivation.as_ref()).await?;
            }
        }

        // 3. Resolve source inputs + closures.
        let all_source_paths = self.resolve_and_ingest_sources(derivation_ref).await?;

        // 4. Collect sandbox inputs.
        let sandbox_inputs = self.collect_sandbox_inputs(derivation_ref, known_paths, &all_source_paths).await?;

        // 5. Create build request.
        let request_envelope = derivation_to_build_request(
            derivation_ref,
            &sandbox_inputs,
            self.store.store_dir(),
            self.hermeticity_mode,
        )?;
        self.hermeticity_audit_events.extend(request_envelope.audit_events.iter().cloned());
        let build_request = request_envelope.build_request;
        let refscan_needles = build_request.refscan_needles.clone();

        info!(drv = %drv_name, "building");
        if self.verbose {
            debug!(
                drv = %drv_name,
                inputs = sandbox_inputs.len(),
                outputs = build_request.outputs.len(),
                "dispatching sandbox build"
            );
        }

        let input_rewrites = self.collect_ca_input_rewrites(derivation_ref, known_paths);
        let is_ca = derivation_ref.outputs.values().all(|o| o.path.is_none() && o.ca_hash.is_none());

        Ok(PrepareResult::NeedsBuild {
            prepared: PreparedBuild {
                drv_path: drv_path.clone(),
                drv_name,
                derivation,
                refscan_needles,
                sandbox_inputs,
                input_rewrites,
                is_ca,
                is_root,
            },
            build_request,
        })
    }

    /// Process a completed sandbox build: apply rewrites, hash outputs,
    /// persist PathInfo, export to disk.
    ///
    /// For multi-output CA derivations, uses a two-pass approach:
    /// 1. Apply input rewrites + compute CA paths for all outputs
    /// 2. Rewrite cross-output references in all outputs
    pub(crate) async fn finish_build(
        &mut self,
        prepared: &PreparedBuild,
        build_result: snix_build::buildservice::BuildResult,
        known_paths: &mut DerivationRegistry,
    ) -> Result<BuildOutcome, Error> {
        let mut output_infos: HashMap<String, PathInfo> = HashMap::new();
        let output_names: Vec<String> = prepared.derivation.outputs.keys().cloned().collect();
        let is_multi_ca = prepared.is_ca && prepared.derivation.outputs.len() > 1;
        let artifact_provenance = self.build_artifact_provenance(&prepared.derivation, known_paths)?;

        if is_multi_ca {
            output_infos =
                self.finish_build_multi_ca(prepared, &build_result, known_paths, &artifact_provenance).await?;
        } else {
            for (i, (output_name, output)) in prepared.derivation.outputs.iter().enumerate() {
                let build_output = build_result.outputs.get(i).ok_or_else(|| Error::OutputMissing {
                    output: output_name.clone(),
                })?;

                let path_info = self
                    .process_output(
                        &prepared.drv_path,
                        &prepared.drv_name,
                        output_name,
                        output,
                        build_output,
                        &prepared.input_rewrites,
                        &prepared.sandbox_inputs,
                        &prepared.refscan_needles,
                        &prepared.derivation,
                        known_paths,
                        prepared.is_ca,
                        &artifact_provenance,
                        prepared.is_root,
                    )
                    .await?;

                output_infos.insert(output_name.clone(), path_info);
            }
        }

        info!(
            drv = %prepared.drv_name,
            outputs = ?output_names.iter()
                .filter_map(|n| {
                    output_infos.get(n).map(|pi| {
                        pi.store_path.to_absolute_path_with_prefix(&self.store.output_dir_str())
                    })
                })
                .collect::<Vec<_>>(),
            "build succeeded"
        );

        Ok(BuildOutcome {
            drv_path: prepared.drv_path.clone(),
            outputs: output_infos,
            substitutions: HashMap::new(),
            cached: false,
            log: build_result.log,
        })
    }

    /// Multi-output CA: two-pass processing.
    ///
    /// Pass 1: For each output, apply input rewrites, then replace ALL
    ///         of the derivation's own output provisionals with distinct
    ///         markers. Hash the canonical form to get the CA path.
    /// Pass 2: For each output, replace all markers with the final CA
    ///         paths (handles cross-output references).
    async fn finish_build_multi_ca(
        &mut self,
        prepared: &PreparedBuild,
        build_result: &snix_build::buildservice::BuildResult,
        known_paths: &mut DerivationRegistry,
        artifact_provenance: &ArtifactProvenance,
    ) -> Result<HashMap<String, PathInfo>, Error> {
        let ca_plans = crate::ca_plan::plan_ca_outputs(
            &prepared.drv_name,
            &prepared.derivation.outputs,
            &prepared.derivation.environment,
        );
        assert_eq!(ca_plans.len(), prepared.derivation.outputs.len(), "CA plan count must match output count",);

        // Pass 1: marker replacement + CA path computation.
        let intermediates = self.compute_ca_intermediates(prepared, build_result, &ca_plans).await?;
        assert_eq!(
            intermediates.len(),
            prepared.derivation.outputs.len(),
            "intermediate count must match output count",
        );

        // Pass 2: replace markers with final paths, persist.
        self.finalize_ca_outputs(prepared, build_result, known_paths, artifact_provenance, &ca_plans, &intermediates)
            .await
    }

    /// Pass 1: apply input rewrites, replace provisionals with markers,
    /// compute NAR hashes, and derive CA store paths.
    async fn compute_ca_intermediates(
        &self,
        prepared: &PreparedBuild,
        build_result: &snix_build::buildservice::BuildResult,
        ca_plans: &[crate::ca_plan::CaOutputPlan],
    ) -> Result<Vec<CaOutputIntermediate>, Error> {
        let nar_renderer = SimpleRenderer::new(self.store.blob_service(), self.store.directory_service());
        let mut intermediates: Vec<CaOutputIntermediate> = Vec::new();

        for (i, (output_name, _output)) in prepared.derivation.outputs.iter().enumerate() {
            let build_output = build_result.outputs.get(i).ok_or_else(|| Error::OutputMissing {
                output: output_name.clone(),
            })?;

            let mut node = apply_input_rewrites(
                &build_output.node,
                &prepared.input_rewrites,
                &self.store.blob_service(),
                &self.store.directory_service(),
            )
            .await?;

            for plan in ca_plans {
                if plan.provisional.is_empty() {
                    continue;
                }
                let (rewritten, _) = crate::rewrite::rewrite_node(
                    &node,
                    plan.provisional.as_bytes(),
                    &plan.marker,
                    &self.store.blob_service(),
                    &self.store.directory_service(),
                )
                .await?;
                node = rewritten;
            }

            let (nar_size, nar_sha256) =
                nar_renderer.calculate_nar(&node).await.map_err(|e| Error::NarCalculation(e.to_string()))?;

            let plan = ca_plans
                .iter()
                .find(|p| p.output_name == *output_name)
                .ok_or_else(|| Error::Store(format!("no CA plan for output '{output_name}'")))?;
            let ca_path = crate::ca_plan::compute_ca_store_path(&plan.path_name, nar_sha256, self.store.store_dir())?;

            intermediates.push(CaOutputIntermediate {
                name: output_name.clone(),
                marked_node: node,
                ca_path,
                nar_size,
                nar_sha256,
            });
        }

        Ok(intermediates)
    }

    /// Pass 2: replace markers with final CA paths, register outputs,
    /// persist PathInfo.
    async fn finalize_ca_outputs(
        &mut self,
        prepared: &PreparedBuild,
        build_result: &snix_build::buildservice::BuildResult,
        known_paths: &mut DerivationRegistry,
        artifact_provenance: &ArtifactProvenance,
        ca_plans: &[crate::ca_plan::CaOutputPlan],
        intermediates: &[CaOutputIntermediate],
    ) -> Result<HashMap<String, PathInfo>, Error> {
        let final_rewrites: Vec<(&[u8], Vec<u8>)> = ca_plans
            .iter()
            .filter_map(|plan| {
                let intermediate = intermediates.iter().find(|i| i.name == plan.output_name)?;
                let final_abs = intermediate.ca_path.to_absolute_path_with_prefix(self.store.store_dir());
                Some((plan.marker.as_slice(), final_abs.into_bytes()))
            })
            .collect();

        let drv_abs = prepared.drv_path.to_absolute_path_with_prefix(self.store.store_dir());
        let mut output_infos: HashMap<String, PathInfo> = HashMap::new();

        for (idx, intermediate) in intermediates.iter().enumerate() {
            let final_node = self.rewrite_markers_to_final(&intermediate.marked_node, &final_rewrites).await?;

            self.register_ca_output(
                &drv_abs,
                &prepared.drv_name,
                &intermediate.name,
                &intermediate.ca_path,
                known_paths,
            )?;

            let build_output = build_result.outputs.get(idx).ok_or_else(|| Error::OutputMissing {
                output: intermediate.name.clone(),
            })?;
            let references = resolve_references(
                &build_output.output_needles,
                &prepared.refscan_needles,
                &prepared.derivation,
                &prepared.sandbox_inputs,
            );

            let ca_field =
                Some(nix_compat::nixhash::CAHash::Nar(nix_compat::nixhash::NixHash::Sha256(intermediate.nar_sha256)));

            let path_info = self
                .persist_and_export_output(
                    &prepared.drv_path,
                    &intermediate.name,
                    &intermediate.ca_path,
                    final_node,
                    references,
                    intermediate.nar_size,
                    intermediate.nar_sha256,
                    ca_field,
                    Some(artifact_provenance.clone()),
                    prepared.is_root,
                )
                .await?;

            output_infos.insert(intermediate.name.clone(), path_info);
        }

        Ok(output_infos)
    }

    /// Rewrite all same-length CA markers to their final absolute paths.
    async fn rewrite_markers_to_final(&self, node: &Node, final_rewrites: &[(&[u8], Vec<u8>)]) -> Result<Node, Error> {
        let mut final_node = node.clone();
        for (marker, final_bytes) in final_rewrites {
            if marker.len() == final_bytes.len() {
                let (rewritten, _) = crate::rewrite::rewrite_node(
                    &final_node,
                    marker,
                    final_bytes,
                    &self.store.blob_service(),
                    &self.store.directory_service(),
                )
                .await?;
                final_node = rewritten;
            }
        }
        Ok(final_node)
    }

    fn closure_fallback_mode(&self) -> crunch_store::StoreFallbackMode {
        if self.hermeticity_mode.is_strict() {
            crunch_store::StoreFallbackMode::Strict
        } else {
            crunch_store::StoreFallbackMode::Practical
        }
    }

    fn ensure_declared_source_exists(&self, source_path: &StorePath<String>) -> Result<(), Error> {
        assert!(!source_path.name().is_empty(), "source path name must not be empty");
        assert!(source_path.to_string().contains('-'), "source path text must include a digest/name separator");

        let abs = PathBuf::from(source_path.to_absolute_path());
        if abs.exists() {
            return Ok(());
        }
        Err(Error::SourceNotFound {
            path: source_path.clone(),
        })
    }

    async fn resolve_source_closure_paths(
        &mut self,
        source_path: &StorePath<String>,
    ) -> Result<Vec<StorePath<String>>, Error> {
        assert!(!source_path.name().is_empty(), "source path name must not be empty");
        assert!(source_path.to_string().contains('-'), "source path text must include a digest/name separator");

        if let Some(cached_paths) = self.source_closure_cache.get(source_path) {
            return Ok(cached_paths.clone());
        }

        let remote_ref = self.store.remote_pathinfo();
        let remote_dyn: Option<&dyn snix_store::pathinfoservice::PathInfoService> = remote_ref.as_deref();
        let closure = crunch_store::resolve_closure(
            source_path,
            self.store.pathinfo_service().as_ref(),
            remote_dyn,
            self.closure_fallback_mode(),
            self.store.store_dir(),
        )
        .await
        .map_err(|e| Error::Store(format!("closure resolution failed for {}: {e}", source_path)))?;
        self.hermeticity_audit_events
            .extend(closure.audit_events.into_iter().map(HermeticityAuditEvent::from));
        assert!(!closure.paths.is_empty(), "closure must contain at least the root source path");

        let resolved_paths = closure.paths;
        self.source_closure_cache.insert(source_path.clone(), resolved_paths.clone());
        Ok(resolved_paths)
    }

    fn preferred_source_host_path(&self, source_path: &StorePath<String>) -> PathBuf {
        let crunch_abs = PathBuf::from(source_path.to_absolute_path_with_prefix(&self.store.output_dir_str()));
        if crunch_abs.exists() {
            return crunch_abs;
        }
        PathBuf::from(source_path.to_absolute_path())
    }

    async fn cached_or_ingested_node_for_path(
        &mut self,
        path: &StorePath<String>,
        host_path: &Path,
    ) -> Result<Option<Node>, Error> {
        assert!(!path.name().is_empty(), "store path name must not be empty");
        assert!(host_path.is_absolute(), "host path must be absolute");

        if let Some(node) = self
            .store
            .cached_node_for_path(path)
            .await
            .map_err(|e| Error::Store(format!("reusing cached node for {path}: {e}")))?
        {
            return Ok(Some(node));
        }
        if !host_path.exists() {
            return Ok(None);
        }

        let node =
            ingest_path::<_, _, _, &[u8]>(self.store.blob_service(), self.store.directory_service(), host_path, None)
                .await
                .map_err(|e| Error::Sandbox(std::io::Error::other(format!("failed to ingest input {}: {e}", path))))?;
        self.store.output_nodes.insert(path.clone(), node.clone());
        Ok(Some(node))
    }

    /// Validate source inputs, resolve closures via PathInfo/narinfo,
    /// and ingest all paths into castore. Returns the full set of
    /// source paths (declared + transitive closure).
    async fn resolve_and_ingest_sources(&mut self, derivation: &Derivation) -> Result<Vec<StorePath<String>>, Error> {
        let mut all_source_paths = Vec::new();
        let mut seen_source_paths = HashSet::new();
        push_unique_store_paths(
            &mut all_source_paths,
            &mut seen_source_paths,
            derivation.input_sources.iter().cloned(),
        );

        for source_path in &derivation.input_sources {
            if self.is_crunch_built(source_path) {
                debug!(path = %source_path, "skipping closure resolution (crunch-built)");
                continue;
            }
            self.ensure_declared_source_exists(source_path)?;
            let closure_paths = self.resolve_source_closure_paths(source_path).await?;
            push_unique_store_paths(&mut all_source_paths, &mut seen_source_paths, closure_paths);
        }

        for source_path in &all_source_paths {
            let host_path = self.preferred_source_host_path(source_path);
            if self.cached_or_ingested_node_for_path(source_path, &host_path).await?.is_none() {
                debug!(path = %source_path, "source path not found on disk, skipping");
            }
        }

        Ok(all_source_paths)
    }

    /// Check if a source path is crunch-built (not from the host Nix store).
    ///
    /// A path is crunch-built if:
    /// - It already has a node in output_nodes (built in this session), or
    /// - It exists in the crunch output dir (when --store != /nix/store)
    fn is_crunch_built(&self, path: &StorePath<String>) -> bool {
        // Already built in this session.
        if self.store.output_nodes.contains_key(path) {
            return true;
        }
        // If --store is a custom dir, check if the path exists there.
        // When output_dir == /nix/store, we can't distinguish, so
        // fall through to Nix closure resolution (safe default).
        if self.store.output_dir_str() != self.store.store_dir() {
            let custom_abs = path.to_absolute_path_with_prefix(&self.store.output_dir_str());
            return PathBuf::from(&custom_abs).exists();
        }
        false
    }

    /// Gather all castore nodes needed as sandbox inputs: built
    /// dependency outputs, declared source paths, and closure paths.
    async fn collect_sandbox_inputs(
        &mut self,
        derivation: &Derivation,
        known_paths: &DerivationRegistry,
        source_paths: &[StorePath<String>],
    ) -> Result<BTreeMap<StorePath<String>, Node>, Error> {
        let mut input_paths = collect_input_paths(derivation, known_paths)?;
        for sp in source_paths {
            input_paths.insert(sp.clone());
        }

        let mut sandbox_inputs: BTreeMap<StorePath<String>, Node> = BTreeMap::new();
        for input_path in &input_paths {
            let abs = self.resolve_host_path(input_path, derivation);
            let Some(node) = self.cached_or_ingested_node_for_path(input_path, &abs).await? else {
                return Err(Error::SourceNotFound {
                    path: input_path.clone(),
                });
            };
            sandbox_inputs.insert(input_path.clone(), node);
        }
        Ok(sandbox_inputs)
    }

    /// Collect (old, new) path pairs for transitive CA input rewriting.
    /// If any input derivation is CA, its provisional env path may appear
    /// in our output and needs rewriting to the final resolved path.
    fn collect_ca_input_rewrites(
        &self,
        derivation: &Derivation,
        known_paths: &DerivationRegistry,
    ) -> Vec<(String, String)> {
        let mut rewrites: Vec<(String, String)> = Vec::new();
        for (input_drv_path, output_names) in &derivation.input_derivations {
            let input_abs = input_drv_path.to_absolute_path_with_prefix(self.store.store_dir());
            if let Some(entry) = known_paths.get_by_drv_path(&input_abs) {
                if entry.content_addressed {
                    for on in output_names {
                        let placeholder = entry
                            .derivation
                            .environment
                            .get(on)
                            .map(|v| String::from_utf8_lossy(v).to_string())
                            .unwrap_or_default();
                        if let Some(resolved) = entry.resolved_outputs.get(on) {
                            // CA rewrites operate in sandbox space (logical prefix).
                            let resolved_abs = resolved.to_absolute_path_with_prefix(self.store.store_dir());
                            if placeholder.len() == resolved_abs.len() {
                                rewrites.push((placeholder, resolved_abs));
                            }
                        }
                    }
                }
            }
        }
        rewrites
    }

    /// Process a single build output: apply rewrites, compute paths,
    /// verify FOD hash, create PathInfo, persist, and export to disk.
    #[allow(clippy::too_many_arguments)]
    async fn process_output(
        &mut self,
        drv_path: &StorePath<String>,
        drv_name: &str,
        output_name: &str,
        output: &nix_compat::derivation::Output,
        build_output: &snix_build::buildservice::BuildOutput,
        input_rewrites: &[(String, String)],
        sandbox_inputs: &BTreeMap<StorePath<String>, Node>,
        refscan_needles: &[String],
        derivation: &Derivation,
        known_paths: &mut DerivationRegistry,
        is_ca: bool,
        artifact_provenance: &ArtifactProvenance,
        is_root: bool,
    ) -> Result<PathInfo, Error> {
        let nar_renderer = SimpleRenderer::new(self.store.blob_service(), self.store.directory_service());
        let working_node = apply_input_rewrites(
            &build_output.node,
            input_rewrites,
            &self.store.blob_service(),
            &self.store.directory_service(),
        )
        .await?;
        let (output_path, final_node, nar_size, nar_sha256) = self
            .resolve_output_node(
                drv_path,
                drv_name,
                output_name,
                output,
                build_output,
                &working_node,
                derivation,
                known_paths,
                is_ca,
                &nar_renderer,
            )
            .await?;
        self.verify_output_hash_if_needed(drv_name, output_name, output, nar_size, &nar_sha256, &final_node)
            .await?;
        let references = resolve_references(&build_output.output_needles, refscan_needles, derivation, sandbox_inputs);
        self.persist_and_export_output(
            drv_path,
            output_name,
            &output_path,
            final_node,
            references,
            nar_size,
            nar_sha256,
            output_ca_field(is_ca, output, nar_sha256),
            Some(artifact_provenance.clone()),
            is_root,
        )
        .await
    }

    /// Resolve the final output path, node, and NAR hash for either a
    /// CA or input-addressed output.
    async fn resolve_output_node(
        &mut self,
        drv_path: &StorePath<String>,
        drv_name: &str,
        output_name: &str,
        output: &nix_compat::derivation::Output,
        build_output: &snix_build::buildservice::BuildOutput,
        working_node: &Node,
        derivation: &Derivation,
        known_paths: &mut DerivationRegistry,
        is_ca: bool,
        nar_renderer: &SimpleRenderer<Arc<dyn BlobService>, Arc<dyn DirectoryService>>,
    ) -> Result<(StorePath<String>, Node, u64, [u8; 32]), Error> {
        if is_ca {
            return self
                .compute_ca_output(drv_path, drv_name, output_name, working_node, derivation, known_paths, nar_renderer)
                .await;
        }

        let path = output
            .path
            .as_ref()
            .ok_or_else(|| Error::OutputNoPath {
                output: output_name.to_string(),
                drv_name: drv_name.to_string(),
            })?
            .clone();
        let (nar_size, nar_sha256) = nar_renderer
            .calculate_nar(&build_output.node)
            .await
            .map_err(|e| Error::NarCalculation(e.to_string()))?;
        Ok((path, working_node.clone(), nar_size, nar_sha256))
    }

    /// Verify a fixed-output hash when the output declares one.
    async fn verify_output_hash_if_needed(
        &self,
        drv_name: &str,
        output_name: &str,
        output: &nix_compat::derivation::Output,
        nar_size: u64,
        nar_sha256: &[u8; 32],
        final_node: &Node,
    ) -> Result<(), Error> {
        let Some(ca_hash) = &output.ca_hash else {
            return Ok(());
        };
        verify_fod_hash(
            drv_name,
            output_name,
            ca_hash,
            nar_size,
            nar_sha256,
            final_node,
            &self.store.blob_service(),
            &self.store.directory_service(),
        )
        .await
    }

    /// Compute the final output path and node for a CA derivation output.
    ///
    /// Performs self-reference rewriting:
    /// 1. Replace provisional path with zero marker
    /// 2. Hash the marker-replaced NAR for the content address
    /// 3. Compute the CA store path
    /// 4. Replace zero markers with the final path
    async fn compute_ca_output(
        &mut self,
        drv_path: &StorePath<String>,
        drv_name: &str,
        output_name: &str,
        working_node: &Node,
        derivation: &Derivation,
        known_paths: &mut DerivationRegistry,
        nar_renderer: &SimpleRenderer<Arc<dyn BlobService>, Arc<dyn DirectoryService>>,
    ) -> Result<(StorePath<String>, Node, u64, [u8; 32]), Error> {
        // 1. Get the provisional (input-addressed) path from the env.
        let provisional = derivation
            .environment
            .get(output_name)
            .map(|v| String::from_utf8_lossy(v).to_string())
            .unwrap_or_default();
        let provisional_bytes = provisional.as_bytes();
        let marker = crate::ca_plan::generate_ca_marker(output_name, provisional_bytes.len());

        // 2. Replace provisional with zero marker.
        let (marked_node, _has_self_refs) = crate::rewrite::rewrite_node(
            working_node,
            provisional_bytes,
            &marker,
            &self.store.blob_service(),
            &self.store.directory_service(),
        )
        .await?;

        // 3. Compute NAR hash of marker-replaced content (canonical form).
        let (marker_nar_size, marker_nar_sha256) =
            nar_renderer.calculate_nar(&marked_node).await.map_err(|e| Error::NarCalculation(e.to_string()))?;

        // 4. Compute CA store path from marker-replaced hash.
        let path_name = crate::ca_plan::ca_output_path_name(drv_name, output_name);
        let ca_path = crate::ca_plan::compute_ca_store_path(&path_name, marker_nar_sha256, self.store.store_dir())?;

        // 5. Replace zero markers with the final CA path (in sandbox/logical space).
        let final_abs = ca_path.to_absolute_path_with_prefix(self.store.store_dir());
        let final_bytes = final_abs.as_bytes();
        let final_node = if marker.len() == final_bytes.len() {
            let (node, _) = crate::rewrite::rewrite_node(
                &marked_node,
                &marker,
                final_bytes,
                &self.store.blob_service(),
                &self.store.directory_service(),
            )
            .await?;
            node
        } else {
            marked_node
        };

        let drv_abs = drv_path.to_absolute_path_with_prefix(self.store.store_dir());
        self.register_ca_output(&drv_abs, drv_name, output_name, &ca_path, known_paths)?;

        Ok((ca_path, final_node, marker_nar_size, marker_nar_sha256))
    }

    /// Register a resolved CA output path, persist the CA mapping, and log it.
    fn register_ca_output(
        &mut self,
        drv_abs: &str,
        drv_name: &str,
        output_name: &str,
        ca_path: &StorePath<String>,
        known_paths: &mut DerivationRegistry,
    ) -> Result<(), Error> {
        known_paths.resolve_output(drv_abs, output_name, ca_path.clone())?;
        let final_abs = ca_path.to_absolute_path_with_prefix(self.store.store_dir());
        self.store.insert_ca_mapping(drv_abs, output_name, &final_abs);
        let display_abs = ca_path.to_absolute_path_with_prefix(self.store.output_dir_str());
        info!(drv = %drv_name, output = %output_name, ca_path = %display_abs, "CA output path resolved");
        Ok(())
    }

    /// Build PathInfo, sign it, then delegate persistence to the StoreHandle.
    async fn persist_and_export_output(
        &mut self,
        drv_path: &StorePath<String>,
        output_name: &str,
        output_path: &StorePath<String>,
        final_node: Node,
        references: Vec<StorePath<String>>,
        nar_size: u64,
        nar_sha256: [u8; 32],
        ca: Option<nix_compat::nixhash::CAHash>,
        provenance: Option<ArtifactProvenance>,
        is_root: bool,
    ) -> Result<PathInfo, Error> {
        // Build the PathInfo locally so we can sign it before handing
        // it to the store for persistence.
        let mut path_info = PathInfo {
            store_path: output_path.clone(),
            node: final_node.clone(),
            references,
            nar_size,
            nar_sha256,
            signatures: vec![],
            deriver: Some(drv_path.clone()),
            ca,
        };

        signing::sign_pathinfo(&mut path_info, &self.keypair.signing_key);

        self.store
            .persist_and_export_signed_output(
                output_name,
                output_path,
                path_info,
                final_node,
                provenance,
                is_root,
                self.root_retention_source,
            )
            .await
            .map_err(|e| Error::Store(format!("{e}")))
    }

    /// Check cache: every output must have PathInfo AND exist on disk.
    /// For CA derivations, uses ca_mappings to find the resolved path.
    /// Delegates to `self.store.check_cache()`, then verifies signatures.
    async fn check_cache(
        &mut self,
        drv_path: &StorePath<String>,
        derivation: &Derivation,
        is_root: bool,
    ) -> Result<Option<CacheCheckHit>, Error> {
        let cached = self
            .store
            .check_cache(drv_path, derivation, is_root, self.root_retention_source)
            .await
            .map_err(|e| Error::Store(format!("{e}")))?;

        let Some(infos) = cached else {
            return Ok(None);
        };

        let substitutions = infos
            .iter()
            .filter_map(|(output_name, path_info)| {
                self.store
                    .take_output_substitution_report(&path_info.store_path)
                    .map(|report| (output_name.clone(), report))
            })
            .collect::<HashMap<_, _>>();

        if self.trust_unsigned {
            return Ok(Some(CacheCheckHit { infos, substitutions }));
        }

        // Verify signatures on every cached output.
        for (output_name, path_info) in &infos {
            let result = signing::verify_pathinfo_signatures(path_info, &self.trusted_keys);
            if !result.is_trusted() {
                if result.total_sigs == 0 {
                    tracing::warn!(
                        drv = %drv_path.name(),
                        output = %output_name,
                        "cache hit rejected: unsigned PathInfo (run `crunch store sign --all` to fix)"
                    );
                } else {
                    tracing::warn!(
                        drv = %drv_path.name(),
                        output = %output_name,
                        untrusted = ?result.untrusted_names,
                        "cache hit rejected: no trusted signature"
                    );
                }
                return Ok(None);
            }
        }

        Ok(Some(CacheCheckHit { infos, substitutions }))
    }

    fn record_cached_output_paths(
        &self,
        drv_path: &StorePath<String>,
        derivation: &Derivation,
        infos: &HashMap<String, PathInfo>,
        known_paths: &mut DerivationRegistry,
    ) -> Result<(), Error> {
        let is_ca = derivation.outputs.values().all(|output| output.path.is_none() && output.ca_hash.is_none());
        if !is_ca {
            return Ok(());
        }

        let drv_abs = drv_path.to_absolute_path_with_prefix(self.store.store_dir());
        for (output_name, path_info) in infos {
            known_paths.resolve_output(&drv_abs, output_name, path_info.store_path.clone())?;
        }
        Ok(())
    }

    /// Collect build-input provenance for native attestations.
    fn build_artifact_provenance(
        &self,
        derivation: &Derivation,
        known_paths: &DerivationRegistry,
    ) -> Result<ArtifactProvenance, Error> {
        let declared_inputs = collect_input_paths(derivation, known_paths)?;
        let drv_abs = self.derivation_path_from_nix_derivation(derivation, known_paths.store_dir())?;
        let claims = known_paths.get_by_drv_path(&drv_abs).and_then(|entry| entry.provenance_claims.clone());
        let mut input_sources = Vec::new();
        let mut input_artifacts = Vec::new();

        for input_path in declared_inputs {
            if derivation.input_sources.contains(&input_path) {
                input_sources.push(input_path);
                continue;
            }
            input_artifacts.push(input_path);
        }

        Ok(ArtifactProvenance {
            claims,
            input_sources,
            input_artifacts,
        })
    }

    fn derivation_path_from_nix_derivation(&self, derivation: &Derivation, store_dir: &str) -> Result<String, Error> {
        let name = derivation
            .environment
            .get("name")
            .map(|value| String::from_utf8_lossy(value.as_ref()).to_string())
            .ok_or_else(|| Error::Store("claims lookup requires derivation.environment.name".to_string()))?;
        let drv_path = derivation
            .calculate_derivation_path_with_store_dir(&name, store_dir)
            .map_err(|e| Error::Store(format!("calculating derivation path for claims lookup: {e}")))?;
        Ok(drv_path.to_absolute_path_with_prefix(store_dir))
    }

    /// Resolve a store path to its host filesystem location.
    /// Source inputs (from input_sources) live at /nix/store/.
    /// Built outputs live at the physical output dir.
    fn resolve_host_path(&self, path: &StorePath<String>, derivation: &Derivation) -> PathBuf {
        let is_source = derivation.input_sources.contains(path);
        if is_source {
            PathBuf::from(path.to_absolute_path())
        } else {
            PathBuf::from(path.to_absolute_path_with_prefix(&self.store.output_dir_str()))
        }
    }

    /// Make sure we have castore nodes for inputs that exist on disk
    /// (needed for passing as sandbox inputs to downstream builds).
    async fn ensure_input_nodes(&mut self, derivation: &Derivation) -> Result<(), Error> {
        for output in derivation.outputs.values() {
            let Some(path) = &output.path else {
                continue;
            };
            let abs = PathBuf::from(path.to_absolute_path_with_prefix(&self.store.output_dir_str()));
            let _ = self.cached_or_ingested_node_for_path(path, &abs).await?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;
    use std::sync::Arc;
    use std::sync::Mutex;

    use async_trait::async_trait;
    use futures::stream::StreamExt;
    use nix_compat::derivation::Output;
    use nix_compat::store_path::StorePath;
    use snix_build::buildservice::BuildOutput;
    use snix_build::buildservice::BuildRequest;
    use snix_build::buildservice::BuildResult;
    use snix_castore::SymlinkTarget;
    use snix_castore::blobservice::BlobService;
    use snix_castore::blobservice::MemoryBlobService;
    use snix_castore::directoryservice::RedbDirectoryService;
    use snix_castore::directoryservice::RedbDirectoryServiceConfig;
    use snix_store::pathinfoservice::LruPathInfoService;
    use snix_store::pathinfoservice::PathInfoService;
    use snix_store::pathinfoservice::{self};
    use tokio::io::AsyncWriteExt;

    use super::*;

    fn tmp_ds() -> RedbDirectoryService {
        RedbDirectoryService::new_temporary("test".to_string(), RedbDirectoryServiceConfig::default()).unwrap()
    }

    /// Write bytes into the blob service and return the resulting
    /// Node::File. Used by cache tests to populate the castore so
    /// that `castore_has_content` finds the blob.
    async fn put_blob(bs: &MemoryBlobService, content: &[u8]) -> Node {
        let mut writer = BlobService::open_write(bs).await;
        writer.write_all(content).await.unwrap();
        let digest = writer.close().await.unwrap();
        Node::File {
            digest,
            size: content.len() as u64,
            executable: false,
        }
    }

    fn test_pis() -> LruPathInfoService {
        LruPathInfoService::with_capacity("test".to_string(), std::num::NonZeroUsize::new(128).unwrap())
    }

    fn test_keypair() -> crate::signing::KeyPair {
        crate::signing::load_keypair(
            "cache.example.com-1:cCta2MEsRNuYCgWYyeRXLyfoFpKhQJKn8gLMeXWAb7vIpRKKo/3JoxJ24OYa3DxT2JVV38KjK/1ywHWuMe2JEw==",
        ).unwrap()
    }

    fn test_trusted_keys() -> Vec<nix_compat::narinfo::VerifyingKey> {
        crate::signing::build_trusted_keys(&test_keypair(), None)
    }

    fn sign_with_test_key(path_info: &mut PathInfo) {
        let keypair = test_keypair();
        crate::signing::sign_pathinfo(path_info, &keypair.signing_key);
    }

    fn corrupt_signature_bytes(path_info: &mut PathInfo) {
        use nix_compat::narinfo::Signature;

        assert!(!path_info.signatures.is_empty(), "signature must exist before corruption");
        let bad_bytes = [0u8; 64];
        let bad_sig_str = format!("{}:{}", path_info.signatures[0].name(), data_encoding::BASE64.encode(&bad_bytes),);
        path_info.signatures[0] = Signature::<String>::parse(&bad_sig_str).unwrap();
    }

    /// A mock BuildService that records requests and returns a synthetic
    /// output node for each requested output.
    struct MockBuildService {
        calls: Arc<Mutex<Vec<Vec<String>>>>,
        blob_service: MemoryBlobService,
    }

    impl MockBuildService {
        fn new(blob_service: MemoryBlobService) -> (Self, Arc<Mutex<Vec<Vec<String>>>>) {
            let calls = Arc::new(Mutex::new(Vec::new()));
            (
                Self {
                    calls: calls.clone(),
                    blob_service,
                },
                calls,
            )
        }
    }

    #[async_trait]
    impl BuildService for MockBuildService {
        async fn do_build(&self, request: BuildRequest) -> std::io::Result<BuildResult> {
            self.calls.lock().unwrap().push(request.command_args.clone());

            let mut writer = BlobService::open_write(&self.blob_service).await;
            writer.write_all(b"mock output").await.unwrap();
            let digest = writer.close().await.unwrap();

            let outputs: Vec<BuildOutput> = request
                .outputs
                .iter()
                .map(|_| BuildOutput {
                    node: Node::File {
                        digest: digest.clone(),
                        size: 11,
                        executable: false,
                    },
                    output_needles: BTreeSet::new(),
                })
                .collect();

            Ok(BuildResult { outputs, log: None })
        }
    }

    #[derive(Default, Clone)]
    struct CountingPathInfoService {
        entries: Arc<Mutex<HashMap<[u8; 20], PathInfo>>>,
        get_counts: Arc<Mutex<HashMap<[u8; 20], u32>>>,
    }

    impl CountingPathInfoService {
        fn insert(&self, path_info: PathInfo) {
            self.entries.lock().unwrap().insert(*path_info.store_path.digest(), path_info);
        }

        fn get_count(&self, path: &StorePath<String>) -> u32 {
            self.get_counts.lock().unwrap().get(path.digest()).copied().unwrap_or(0)
        }
    }

    #[async_trait]
    impl PathInfoService for CountingPathInfoService {
        async fn get(&self, digest: [u8; 20]) -> Result<Option<PathInfo>, pathinfoservice::Error> {
            let mut counts = self.get_counts.lock().unwrap();
            let next_count = counts.get(&digest).copied().unwrap_or(0).saturating_add(1);
            counts.insert(digest, next_count);
            drop(counts);
            Ok(self.entries.lock().unwrap().get(&digest).cloned())
        }

        async fn put(&self, path_info: PathInfo) -> Result<PathInfo, pathinfoservice::Error> {
            self.entries.lock().unwrap().insert(*path_info.store_path.digest(), path_info.clone());
            Ok(path_info)
        }

        fn list(&self) -> futures::stream::BoxStream<'static, Result<PathInfo, pathinfoservice::Error>> {
            let entries: Vec<_> = self.entries.lock().unwrap().values().cloned().collect();
            futures::stream::iter(entries.into_iter().map(Ok)).boxed()
        }
    }

    fn make_source_path(name: &str, digest_byte: u8) -> StorePath<String> {
        StorePath::from_name_and_digest_fixed(name, [digest_byte; 20]).unwrap()
    }

    fn make_source_path_info(path: &StorePath<String>, references: Vec<StorePath<String>>) -> PathInfo {
        PathInfo {
            store_path: path.clone(),
            node: Node::Symlink {
                target: SymlinkTarget::try_from("source-target").unwrap(),
            },
            references,
            nar_size: 0,
            nar_sha256: [0u8; 32],
            signatures: vec![],
            deriver: None,
            ca: None,
        }
    }

    /// Build a nix_compat::Derivation, compute paths, register in DerivationRegistry.
    fn build_and_register(
        name: &str,
        input_drvs: &[(StorePath<String>, &str)],
        kp: &mut crate::registry::DerivationRegistry,
    ) -> (StorePath<String>, Derivation) {
        let mut outputs = BTreeMap::new();
        outputs.insert("out".to_string(), nix_compat::derivation::Output {
            path: None,
            ca_hash: None,
        });
        let mut environment = BTreeMap::new();
        environment.insert("name".to_string(), name.into());
        environment.insert("system".to_string(), "x86_64-linux".into());
        environment.insert("builder".to_string(), "/bin/sh".into());
        environment.insert("out".to_string(), "".into());

        let mut input_derivations = BTreeMap::new();
        for (dp, on) in input_drvs {
            input_derivations.entry(dp.clone()).or_insert_with(BTreeSet::new).insert(on.to_string());
        }

        let mut drv = Derivation {
            arguments: vec!["-c".into(), format!("echo {name} > $out")],
            builder: "/bin/sh".to_string(),
            environment,
            input_derivations,
            input_sources: BTreeSet::new(),
            outputs,
            system: "x86_64-linux".to_string(),
        };

        let hdm = drv.hash_derivation_modulo(|parent_path| {
            kp.get_hdm_by_drv_path(&parent_path.to_absolute_path()).expect("parent should be in known_paths")
        });
        drv.calculate_output_paths(name, &hdm).unwrap();
        let drv_path = drv.calculate_derivation_path(name).unwrap();

        let mut fake_hash = [0u8; 32];
        for (i, b) in name.bytes().enumerate().take(32) {
            fake_hash[i] = b;
        }
        kp.insert(drv_path.clone(), hdm, drv.clone(), false, None);

        (drv_path, drv)
    }

    #[tokio::test]
    async fn source_closure_resolution_is_memoized_per_session() {
        let bs = MemoryBlobService::default();
        let ds = tmp_ds();
        let (mock, _calls) = MockBuildService::new(bs.clone());
        let pis = CountingPathInfoService::default();

        let root = make_source_path("shared-source", 41);
        let dep = make_source_path("shared-source-ref", 42);
        pis.insert(make_source_path_info(&dep, vec![]));
        pis.insert(make_source_path_info(&root, vec![dep.clone()]));

        let mut builder = Builder::new(
            bs,
            ds,
            mock,
            pis.clone(),
            PathBuf::from("/nix/store"),
            nix_compat::store_path::STORE_DIR,
            test_keypair(),
            test_trusted_keys(),
            true,
            false,
        );

        let first = builder.resolve_source_closure_paths(&root).await.unwrap();
        let second = builder.resolve_source_closure_paths(&root).await.unwrap();

        assert_eq!(first, second, "memoized closure must match the first resolution");
        assert_eq!(pis.get_count(&root), 1, "root closure should be queried once");
        assert_eq!(pis.get_count(&dep), 1, "transitive reference should be queried once");
        assert_eq!(builder.source_closure_cache.len(), 1, "session should cache one source root");
    }

    #[tokio::test]
    async fn collect_sandbox_inputs_reuses_pathinfo_node_before_disk_ingest() {
        use snix_store::pathinfoservice::PathInfoService;

        let bs = MemoryBlobService::default();
        let ds = tmp_ds();
        let (mock, _calls) = MockBuildService::new(bs.clone());
        let output_dir = tempfile::tempdir().unwrap();
        let pis = test_pis();

        let mut kp = DerivationRegistry::default();
        let (dep_drv_path, dep_drv) = build_and_register("reuse-dep", &[], &mut kp);
        let (_root_drv_path, root_drv) = build_and_register("reuse-root", &[(dep_drv_path.clone(), "out")], &mut kp);
        let dep_output_path = dep_drv.outputs["out"].path.as_ref().unwrap().clone();

        let cached_node = put_blob(&bs, b"cached dependency output").await;
        let path_info = PathInfo {
            store_path: dep_output_path.clone(),
            node: cached_node.clone(),
            references: vec![],
            nar_size: 0,
            nar_sha256: [0u8; 32],
            signatures: vec![],
            deriver: Some(dep_drv_path.clone()),
            ca: None,
        };
        pis.put(path_info).await.unwrap();

        let disk_path =
            PathBuf::from(dep_output_path.to_absolute_path_with_prefix(output_dir.path().to_str().unwrap()));
        std::fs::create_dir_all(disk_path.parent().unwrap()).unwrap();
        std::fs::write(&disk_path, b"disk fallback should stay unused").unwrap();

        let mut builder = Builder::new(
            bs,
            ds,
            mock,
            pis,
            output_dir.path().to_path_buf(),
            nix_compat::store_path::STORE_DIR,
            test_keypair(),
            test_trusted_keys(),
            true,
            false,
        );

        let sandbox_inputs = builder.collect_sandbox_inputs(&root_drv, &kp, &[]).await.unwrap();

        assert_eq!(sandbox_inputs.get(&dep_output_path), Some(&cached_node));
        assert_eq!(builder.store.output_nodes.get(&dep_output_path), Some(&cached_node));
    }

    #[tokio::test]
    async fn builder_single_drv_calls_do_build_once() {
        let bs = MemoryBlobService::default();
        let ds = tmp_ds();
        let (mock, calls) = MockBuildService::new(bs.clone());

        let mut builder = Builder::new(
            bs,
            ds,
            mock,
            test_pis(),
            PathBuf::from("/nix/store"),
            nix_compat::store_path::STORE_DIR,
            test_keypair(),
            test_trusted_keys(),
            true,
            false,
        );

        let mut kp = DerivationRegistry::default();
        let (drv_path, _) = build_and_register("solo", &[], &mut kp);

        let outcome = builder.build(&drv_path, &mut kp).await.unwrap();
        assert!(!outcome.cached);
        assert_eq!(outcome.outputs.len(), 1);

        let recorded = calls.lock().unwrap();
        assert_eq!(recorded.len(), 1, "should call do_build exactly once");
    }

    #[tokio::test]
    async fn builder_chain_builds_dep_first() {
        let bs = MemoryBlobService::default();
        let ds = tmp_ds();
        let (mock, calls) = MockBuildService::new(bs.clone());

        let mut builder = Builder::new(
            bs,
            ds,
            mock,
            test_pis(),
            PathBuf::from("/nix/store"),
            nix_compat::store_path::STORE_DIR,
            test_keypair(),
            test_trusted_keys(),
            true,
            false,
        );

        let mut kp = DerivationRegistry::default();
        let (dep_path, _) = build_and_register("dep", &[], &mut kp);
        let (top_path, _) = build_and_register("top", &[(dep_path.clone(), "out")], &mut kp);

        let outcome = builder.build(&top_path, &mut kp).await.unwrap();
        assert!(!outcome.cached);

        let recorded = calls.lock().unwrap();
        assert_eq!(recorded.len(), 2, "should build dep + top");
        assert!(recorded[0].iter().any(|a| a.contains("dep")), "first build should be dep: {:?}", recorded[0]);
        assert!(recorded[1].iter().any(|a| a.contains("top")), "second build should be top: {:?}", recorded[1]);
    }

    #[tokio::test]
    async fn builder_diamond_builds_shared_dep_once() {
        let bs = MemoryBlobService::default();
        let ds = tmp_ds();
        let (mock, calls) = MockBuildService::new(bs.clone());

        let mut builder = Builder::new(
            bs,
            ds,
            mock,
            test_pis(),
            PathBuf::from("/nix/store"),
            nix_compat::store_path::STORE_DIR,
            test_keypair(),
            test_trusted_keys(),
            true,
            false,
        );

        let mut kp = DerivationRegistry::default();
        let (a_path, _) = build_and_register("aaa", &[], &mut kp);
        let (b_path, _) = build_and_register("bbb", &[(a_path.clone(), "out")], &mut kp);
        let (c_path, _) = build_and_register("ccc", &[(a_path.clone(), "out")], &mut kp);
        let (d_path, _) = build_and_register("ddd", &[(b_path.clone(), "out"), (c_path.clone(), "out")], &mut kp);

        let outcome = builder.build(&d_path, &mut kp).await.unwrap();
        assert!(!outcome.cached);

        let recorded = calls.lock().unwrap();
        assert_eq!(recorded.len(), 4, "should build A+B+C+D: {:?}", *recorded);
        let a_builds = recorded.iter().filter(|args| args.iter().any(|a| a.contains("aaa"))).count();
        assert_eq!(a_builds, 1, "shared dep A should build exactly once");
    }

    // ── Cache tests ────────────────────────────────────────────

    #[tokio::test]
    async fn builder_skips_build_when_castore_cached() {
        let bs = MemoryBlobService::default();
        let ds = tmp_ds();
        let (mock, calls) = MockBuildService::new(bs.clone());
        let pis = test_pis();

        let mut kp = DerivationRegistry::default();
        let (drv_path, drv) = build_and_register("cached-test", &[], &mut kp);

        // Put content into castore (no file on disk needed).
        let out_path = drv.outputs["out"].path.as_ref().unwrap();
        let node = put_blob(&bs, b"cached content").await;

        let path_info = PathInfo {
            store_path: out_path.clone(),
            node,
            references: vec![],
            nar_size: 0,
            nar_sha256: [0u8; 32],
            signatures: vec![],
            deriver: Some(drv_path.clone()),
            ca: None,
        };
        use snix_store::pathinfoservice::PathInfoService;
        pis.put(path_info).await.unwrap();

        let mut builder = Builder::new(
            bs,
            ds,
            mock,
            pis,
            PathBuf::from("/nix/store"),
            nix_compat::store_path::STORE_DIR,
            test_keypair(),
            test_trusted_keys(),
            true,
            false,
        );

        let outcome = builder.build(&drv_path, &mut kp).await.unwrap();
        assert!(outcome.cached, "should report as cached");

        let recorded = calls.lock().unwrap();
        assert_eq!(recorded.len(), 0, "should NOT call do_build for cached output");
    }

    #[tokio::test]
    async fn build_persists_signed_pathinfo_that_verifies_on_reread() {
        use snix_store::pathinfoservice::PathInfoService;

        let bs = MemoryBlobService::default();
        let ds = tmp_ds();
        let (mock, _calls) = MockBuildService::new(bs.clone());
        let pis = Arc::new(test_pis());

        let mut builder = Builder::new(
            bs,
            ds,
            mock,
            pis.clone(),
            PathBuf::from("/nix/store"),
            nix_compat::store_path::STORE_DIR,
            test_keypair(),
            test_trusted_keys(),
            false,
            false,
        );

        let mut kp = DerivationRegistry::default();
        let (drv_path, drv) = build_and_register("signed-roundtrip", &[], &mut kp);
        let out_path = drv.outputs["out"].path.as_ref().unwrap().clone();
        let digest = *out_path.digest();

        let outcome = builder.build(&drv_path, &mut kp).await.unwrap();
        assert!(!outcome.cached);
        let signed_output = outcome.outputs.get("out").unwrap();
        assert_eq!(signed_output.signatures.len(), 1);

        let reread = pis.get(digest).await.unwrap().unwrap();
        assert_eq!(reread.signatures.len(), 1);

        let verify = crate::signing::verify_pathinfo_signatures(&reread, &test_trusted_keys());
        assert!(verify.is_trusted(), "persisted signature should verify on reread");
        assert_eq!(verify.trusted_count, 1);
    }

    #[tokio::test]
    async fn build_persists_attestation_with_declared_inputs() {
        let bs = MemoryBlobService::default();
        let ds = tmp_ds();
        let (mock, _calls) = MockBuildService::new(bs.clone());
        let pis = Arc::new(test_pis()) as Arc<dyn snix_store::pathinfoservice::PathInfoService>;
        let output_dir = tempfile::tempdir().unwrap();
        let state_dir = tempfile::tempdir().unwrap();

        let mut builder = Builder::with_state_dir(
            Arc::new(bs) as Arc<dyn BlobService>,
            Arc::new(ds) as Arc<dyn DirectoryService>,
            mock,
            pis,
            output_dir.path().to_path_buf(),
            Some(state_dir.path().to_path_buf()),
            None,
            nix_compat::store_path::STORE_DIR,
            test_keypair(),
            test_trusted_keys(),
            false,
            false,
        );

        let mut kp = DerivationRegistry::default();
        let (dep_drv_path, dep_drv) = build_and_register("attestation-dep", &[], &mut kp);
        let source_path = StorePath::from_name_and_digest_fixed("attestation-src", [42u8; 20]).unwrap();
        let source_disk_path =
            PathBuf::from(source_path.to_absolute_path_with_prefix(output_dir.path().to_str().unwrap()));
        std::fs::create_dir_all(source_disk_path.parent().unwrap()).unwrap();
        std::fs::write(&source_disk_path, b"source input").unwrap();

        let root_drv = {
            let mut outputs = BTreeMap::new();
            outputs.insert("out".to_string(), nix_compat::derivation::Output {
                path: None,
                ca_hash: None,
            });
            let mut environment = BTreeMap::new();
            environment.insert("name".to_string(), "attestation-root".into());
            environment.insert("system".to_string(), "x86_64-linux".into());
            environment.insert("builder".to_string(), "/bin/sh".into());
            environment.insert("out".to_string(), "".into());
            let mut input_derivations = BTreeMap::new();
            input_derivations.insert(dep_drv_path.clone(), BTreeSet::from(["out".to_string()]));
            let mut input_sources = BTreeSet::new();
            input_sources.insert(source_path.clone());
            let mut drv = Derivation {
                arguments: vec!["-c".into(), "echo root > $out".into()],
                builder: "/bin/sh".to_string(),
                environment,
                input_derivations,
                input_sources,
                outputs,
                system: "x86_64-linux".to_string(),
            };
            let hdm = drv.hash_derivation_modulo(|parent_path| {
                kp.get_hdm_by_drv_path(&parent_path.to_absolute_path()).expect("parent should be in known_paths")
            });
            drv.calculate_output_paths("attestation-root", &hdm).unwrap();
            let drv_path = drv.calculate_derivation_path("attestation-root").unwrap();
            kp.insert(
                drv_path.clone(),
                hdm,
                drv.clone(),
                false,
                Some(crunch_attestation::Claims {
                    supplier: Some("Example Supplier".to_string()),
                    homepage: Some("https://example.invalid/root".to_string()),
                    ..Default::default()
                }),
            );
            (drv_path, drv)
        };

        let outcome = builder.build(&root_drv.0, &mut kp).await.unwrap();
        assert!(!outcome.cached, "root should build locally");

        let root_output_path = root_drv.1.outputs["out"].path.as_ref().unwrap();
        let attestation = builder.store.get_artifact_attestation(root_output_path).await.unwrap().unwrap();
        let dep_output_path = dep_drv.outputs["out"].path.as_ref().unwrap().to_absolute_path();
        let source_logical_path = source_path.to_absolute_path();

        assert!(attestation.attestation.edges.iter().any(|edge| {
            edge.kind == crunch_attestation::EdgeKind::BuildInput
                && edge.to_node_id == format!("artifact:{dep_output_path}")
        }));
        assert!(attestation.attestation.edges.iter().any(|edge| {
            edge.kind == crunch_attestation::EdgeKind::FetchedFrom
                && edge.from_node_id == format!("source:{source_logical_path}")
        }));
        assert_eq!(attestation.attestation.claims.supplier.as_deref(), Some("Example Supplier"));
        assert_eq!(attestation.attestation.claims.homepage.as_deref(), Some("https://example.invalid/root"));
    }

    #[tokio::test]
    async fn corrupted_local_signature_triggers_rebuild() {
        use snix_store::pathinfoservice::PathInfoService;

        let bs = MemoryBlobService::default();
        let ds = tmp_ds();
        let (mock, calls) = MockBuildService::new(bs.clone());
        let pis = test_pis();

        let mut kp = DerivationRegistry::default();
        let (drv_path, drv) = build_and_register("corrupt-sig", &[], &mut kp);

        let out_path = drv.outputs["out"].path.as_ref().unwrap();
        let node = put_blob(&bs, b"cached content").await;
        let mut path_info = PathInfo {
            store_path: out_path.clone(),
            node,
            references: vec![],
            nar_size: 0,
            nar_sha256: [9u8; 32],
            signatures: vec![],
            deriver: Some(drv_path.clone()),
            ca: None,
        };
        sign_with_test_key(&mut path_info);
        corrupt_signature_bytes(&mut path_info);
        pis.put(path_info).await.unwrap();

        let mut builder = Builder::new(
            bs,
            ds,
            mock,
            pis,
            PathBuf::from("/nix/store"),
            nix_compat::store_path::STORE_DIR,
            test_keypair(),
            test_trusted_keys(),
            false,
            false,
        );

        let outcome = builder.build(&drv_path, &mut kp).await.unwrap();
        assert!(!outcome.cached, "corrupted local signature must be treated as cache miss");

        let recorded = calls.lock().unwrap();
        assert_eq!(recorded.len(), 1, "builder should rebuild after signature verification fails");
    }

    #[tokio::test]
    async fn cache_miss_when_pathinfo_but_no_castore_content() {
        let bs = MemoryBlobService::default();
        let ds = tmp_ds();
        let (mock, calls) = MockBuildService::new(bs.clone());
        let pis = test_pis();

        let mut kp = DerivationRegistry::default();
        let (drv_path, drv) = build_and_register("orphan", &[], &mut kp);

        // PathInfo references a blob digest that doesn't exist in the
        // blob service — castore content is missing.
        let out_path = drv.outputs["out"].path.as_ref().unwrap();
        let path_info = PathInfo {
            store_path: out_path.clone(),
            node: Node::File {
                digest: snix_castore::B3Digest::from(&[0u8; 32]),
                size: 0,
                executable: false,
            },
            references: vec![],
            nar_size: 0,
            nar_sha256: [0u8; 32],
            signatures: vec![],
            deriver: Some(drv_path.clone()),
            ca: None,
        };
        use snix_store::pathinfoservice::PathInfoService;
        pis.put(path_info).await.unwrap();

        let mut builder = Builder::new(
            bs,
            ds,
            mock,
            pis,
            PathBuf::from("/nix/store"),
            nix_compat::store_path::STORE_DIR,
            test_keypair(),
            test_trusted_keys(),
            true,
            false,
        );

        let outcome = builder.build(&drv_path, &mut kp).await.unwrap();
        assert!(!outcome.cached, "should NOT be cached (castore content missing)");

        let recorded = calls.lock().unwrap();
        assert_eq!(recorded.len(), 1, "should call do_build");
    }

    #[tokio::test]
    async fn cache_miss_when_no_pathinfo() {
        // No PathInfo in the service at all — always a miss,
        // regardless of what's on disk or in the castore.
        let bs = MemoryBlobService::default();
        let ds = tmp_ds();
        let (mock, calls) = MockBuildService::new(bs.clone());
        let pis = test_pis();

        let mut kp = DerivationRegistry::default();
        let (drv_path, _drv) = build_and_register("untracked", &[], &mut kp);

        let mut builder = Builder::new(
            bs,
            ds,
            mock,
            pis,
            PathBuf::from("/nix/store"),
            nix_compat::store_path::STORE_DIR,
            test_keypair(),
            test_trusted_keys(),
            true,
            false,
        );

        let outcome = builder.build(&drv_path, &mut kp).await.unwrap();
        assert!(!outcome.cached, "should NOT be cached (no PathInfo)");

        let recorded = calls.lock().unwrap();
        assert_eq!(recorded.len(), 1, "should call do_build");
    }

    // ── CA derivation tests ──────────────────────────────────────

    fn build_and_register_ca(
        name: &str,
        kp: &mut crate::registry::DerivationRegistry,
    ) -> (StorePath<String>, Derivation) {
        let mut outputs = BTreeMap::new();
        outputs.insert("out".to_string(), nix_compat::derivation::Output {
            path: None,
            ca_hash: None,
        });
        let mut environment = BTreeMap::new();
        environment.insert("name".to_string(), name.into());
        environment.insert("system".to_string(), "x86_64-linux".into());
        environment.insert("builder".to_string(), "/bin/sh".into());
        environment.insert("out".to_string(), "".into());

        let mut drv = Derivation {
            arguments: vec!["-c".into(), format!("echo {name} > $out")],
            builder: "/bin/sh".to_string(),
            environment,
            input_derivations: BTreeMap::new(),
            input_sources: BTreeSet::new(),
            outputs,
            system: "x86_64-linux".to_string(),
        };

        let hdm = drv.hash_derivation_modulo(|_| panic!("CA drv has no input derivations"));

        drv.calculate_output_paths(name, &hdm).unwrap();
        for (_, output) in drv.outputs.iter_mut() {
            output.path = None;
        }

        let drv_path = drv.calculate_derivation_path(name).unwrap();

        let mut fake_hash = [0u8; 32];
        for (i, b) in name.bytes().enumerate().take(32) {
            fake_hash[i] = b;
        }
        kp.insert(drv_path.clone(), hdm, drv.clone(), true, None);

        (drv_path, drv)
    }

    #[tokio::test]
    async fn cached_ca_dependency_records_resolved_output_for_later_builds() {
        let bs = MemoryBlobService::default();
        let ds = tmp_ds();
        let (mock, calls) = MockBuildService::new(bs.clone());
        let pis = test_pis();

        let mut builder = Builder::new(
            bs,
            ds,
            mock,
            pis,
            PathBuf::from("/nix/store"),
            nix_compat::store_path::STORE_DIR,
            test_keypair(),
            test_trusted_keys(),
            true,
            false,
        );

        let mut seed_registry = DerivationRegistry::default();
        let (seed_drv_path, _) = build_and_register_ca("cached-seed", &mut seed_registry);
        let seed_outcome = builder.build(&seed_drv_path, &mut seed_registry).await.unwrap();
        let seed_output_path = seed_outcome.outputs["out"].store_path.clone();

        let mut dependent_registry = DerivationRegistry::default();
        let (seed_drv_path_again, _) = build_and_register_ca("cached-seed", &mut dependent_registry);
        assert_eq!(seed_drv_path_again, seed_drv_path, "same CA derivation must keep same drv path");
        let (root_drv_path, _) =
            build_and_register("uses-cached-seed", &[(seed_drv_path.clone(), "out")], &mut dependent_registry);

        let root_outcome = builder.build(&root_drv_path, &mut dependent_registry).await.unwrap();
        assert!(!root_outcome.outputs.is_empty(), "dependent root must still build");

        let seed_abs = seed_drv_path.to_absolute_path();
        let recorded_output_path = dependent_registry
            .get_output_path(&seed_abs, "out")
            .expect("cached CA dependency must record resolved output path");
        assert_eq!(recorded_output_path, seed_output_path, "cached CA output path must match first build");

        let recorded = calls.lock().unwrap();
        assert_eq!(recorded.len(), 2, "only the seed build and dependent root build should dispatch");
    }

    #[tokio::test]
    async fn ca_derivation_gets_content_based_path() {
        let bs = MemoryBlobService::default();
        let ds = tmp_ds();
        let (mock, _calls) = MockBuildService::new(bs.clone());

        let mut builder = Builder::new(
            bs,
            ds,
            mock,
            test_pis(),
            PathBuf::from("/nix/store"),
            nix_compat::store_path::STORE_DIR,
            test_keypair(),
            test_trusted_keys(),
            true,
            false,
        );

        let mut kp = DerivationRegistry::default();
        let (drv_path, _drv) = build_and_register_ca("ca-test", &mut kp);

        let outcome = builder.build(&drv_path, &mut kp).await.unwrap();
        assert!(!outcome.cached);
        assert_eq!(outcome.outputs.len(), 1);

        let pi = outcome.outputs.get("out").unwrap();
        let path_str = pi.store_path.to_absolute_path();
        assert!(path_str.starts_with("/nix/store/"), "CA path in store: {path_str}");
        assert!(pi.ca.is_some(), "CA PathInfo should have ca field");
    }

    #[tokio::test]
    async fn ca_identical_outputs_same_path() {
        let bs = MemoryBlobService::default();
        let ds = tmp_ds();
        let (mock1, _) = MockBuildService::new(bs.clone());
        let mut builder1 = Builder::new(
            bs.clone(),
            ds.clone(),
            mock1,
            test_pis(),
            PathBuf::from("/nix/store"),
            nix_compat::store_path::STORE_DIR,
            test_keypair(),
            test_trusted_keys(),
            true,
            false,
        );
        let mut kp1 = DerivationRegistry::default();
        let (drv_path1, _) = build_and_register_ca("ca-a", &mut kp1);
        let outcome1 = builder1.build(&drv_path1, &mut kp1).await.unwrap();

        let (mock2, _) = MockBuildService::new(bs.clone());
        let mut builder2 = Builder::new(
            bs.clone(),
            ds.clone(),
            mock2,
            test_pis(),
            PathBuf::from("/nix/store"),
            nix_compat::store_path::STORE_DIR,
            test_keypair(),
            test_trusted_keys(),
            true,
            false,
        );
        let mut kp2 = DerivationRegistry::default();
        let (drv_path2, _) = build_and_register_ca("ca-b", &mut kp2);
        let outcome2 = builder2.build(&drv_path2, &mut kp2).await.unwrap();

        assert_ne!(drv_path1, drv_path2, "different drvs");

        // CA paths should differ because the path name includes the drv name.
        // Both should have ca field set.
        assert!(outcome1.outputs["out"].ca.is_some());
        assert!(outcome2.outputs["out"].ca.is_some());
    }

    #[tokio::test]
    async fn ca_same_name_same_content_same_path() {
        let bs = MemoryBlobService::default();
        let ds = tmp_ds();

        let (mock1, _) = MockBuildService::new(bs.clone());
        let mut builder1 = Builder::new(
            bs.clone(),
            ds.clone(),
            mock1,
            test_pis(),
            PathBuf::from("/nix/store"),
            nix_compat::store_path::STORE_DIR,
            test_keypair(),
            test_trusted_keys(),
            true,
            false,
        );
        let mut kp1 = DerivationRegistry::default();
        let (drv_path1, _) = build_and_register_ca("ca-same", &mut kp1);
        let outcome1 = builder1.build(&drv_path1, &mut kp1).await.unwrap();

        let (mock2, _) = MockBuildService::new(bs.clone());
        let mut builder2 = Builder::new(
            bs.clone(),
            ds.clone(),
            mock2,
            test_pis(),
            PathBuf::from("/nix/store"),
            nix_compat::store_path::STORE_DIR,
            test_keypair(),
            test_trusted_keys(),
            true,
            false,
        );
        let mut kp2 = DerivationRegistry::default();
        let (drv_path2, _) = build_and_register_ca("ca-same", &mut kp2);
        let outcome2 = builder2.build(&drv_path2, &mut kp2).await.unwrap();

        let path1 = outcome1.outputs["out"].store_path.to_absolute_path();
        let path2 = outcome2.outputs["out"].store_path.to_absolute_path();
        assert_eq!(path1, path2, "same name + same content = same CA path");
    }

    // ── Custom output_dir tests ───────────────────────────────────

    #[tokio::test]
    async fn custom_output_dir_writes_output_there() {
        // Builder with output_dir = temp dir (not /nix/store).
        // DerivationRegistry uses /nix/store (default/logical).
        // Verifies output lands in the custom dir, not /nix/store.
        let output_tmp = tempfile::tempdir().unwrap();

        let bs = MemoryBlobService::default();
        let ds = tmp_ds();
        let (mock, _calls) = MockBuildService::new(bs.clone());

        let mut builder = Builder::new(
            bs,
            ds,
            mock,
            test_pis(),
            output_tmp.path().to_path_buf(),
            nix_compat::store_path::STORE_DIR,
            test_keypair(),
            test_trusted_keys(),
            true,
            false,
        );

        let mut kp = DerivationRegistry::default();
        let (drv_path, _) = build_and_register("custom-dir-test", &[], &mut kp);

        let outcome = builder.build(&drv_path, &mut kp).await.unwrap();
        assert!(!outcome.cached);

        // Output should exist under the custom output dir.
        let out_info = &outcome.outputs["out"];
        let custom_abs = out_info.store_path.to_absolute_path_with_prefix(output_tmp.path().to_str().unwrap());
        assert!(PathBuf::from(&custom_abs).exists(), "output should exist at custom dir: {custom_abs}");

        // And should NOT exist at the logical /nix/store path.
        let logical_abs = out_info.store_path.to_absolute_path();
        // (Only check if /nix/store is not the output_dir, which it
        // isn't since we used a temp dir.)
        assert_ne!(output_tmp.path().to_str().unwrap(), "/nix/store", "test requires output_dir != /nix/store");
        assert!(
            !PathBuf::from(&logical_abs).exists() || logical_abs == custom_abs,
            "output should NOT exist at /nix/store: {logical_abs}"
        );
    }

    #[tokio::test]
    async fn custom_output_dir_cache_hit_via_castore() {
        // Cache hit works even with a custom output dir — castore content
        // is the authority, not disk.
        let output_tmp = tempfile::tempdir().unwrap();

        let bs = MemoryBlobService::default();
        let ds = tmp_ds();
        let (mock, calls) = MockBuildService::new(bs.clone());
        let pis = test_pis();

        let mut kp = DerivationRegistry::default();
        let (drv_path, drv) = build_and_register("cached-custom", &[], &mut kp);

        // Put content in castore, nothing on disk.
        let out_path = drv.outputs["out"].path.as_ref().unwrap();
        let node = put_blob(&bs, b"cached").await;

        let path_info = PathInfo {
            store_path: out_path.clone(),
            node,
            references: vec![],
            nar_size: 0,
            nar_sha256: [0u8; 32],
            signatures: vec![],
            deriver: Some(drv_path.clone()),
            ca: None,
        };
        use snix_store::pathinfoservice::PathInfoService;
        pis.put(path_info).await.unwrap();

        let mut builder = Builder::new(
            bs,
            ds,
            mock,
            pis,
            output_tmp.path().to_path_buf(),
            nix_compat::store_path::STORE_DIR,
            test_keypair(),
            test_trusted_keys(),
            true,
            false,
        );

        let outcome = builder.build(&drv_path, &mut kp).await.unwrap();
        assert!(outcome.cached, "should be cached via castore");

        let recorded = calls.lock().unwrap();
        assert_eq!(recorded.len(), 0, "no build needed for cached output");
    }

    // ── Root-only export tests ──────────────────────────────────

    #[tokio::test]
    async fn only_root_output_exported_to_disk() {
        // dep (non-root) → root. Only root's output should appear on disk.
        let output_tmp = tempfile::tempdir().unwrap();
        let output_dir = output_tmp.path().to_str().unwrap().to_string();

        let bs = MemoryBlobService::default();
        let ds = tmp_ds();
        let (mock, _calls) = MockBuildService::new(bs.clone());

        let mut builder = Builder::new(
            bs,
            ds,
            mock,
            test_pis(),
            output_tmp.path().to_path_buf(),
            nix_compat::store_path::STORE_DIR,
            test_keypair(),
            test_trusted_keys(),
            true,
            false,
        );

        let mut kp = DerivationRegistry::default();
        let (dep_path, dep_drv) = build_and_register("dep-lib", &[], &mut kp);
        let (root_path, root_drv) = build_and_register("root-app", &[(dep_path.clone(), "out")], &mut kp);

        let outcomes = builder.build_all(&[root_path.clone()], &mut kp, 1).await.unwrap();

        assert_eq!(outcomes.len(), 1);
        assert!(!outcomes[0].cached);

        // Root output should exist on disk.
        let root_out = root_drv.outputs["out"].path.as_ref().unwrap();
        let root_abs = PathBuf::from(root_out.to_absolute_path_with_prefix(&output_dir));
        assert!(root_abs.exists(), "root output should be on disk");

        // Dep output should NOT exist on disk (stays in castore).
        let dep_out = dep_drv.outputs["out"].path.as_ref().unwrap();
        let dep_abs = PathBuf::from(dep_out.to_absolute_path_with_prefix(&output_dir));
        assert!(!dep_abs.exists(), "dep output should stay in castore only");
    }

    #[tokio::test]
    async fn read_only_output_dir_still_succeeds() {
        // With a non-writable output dir, builds should still succeed
        // (output lives in castore). No panic, no error.
        let bs = MemoryBlobService::default();
        let ds = tmp_ds();
        let (mock, _calls) = MockBuildService::new(bs.clone());

        // Use a path that doesn't exist and can't be created.
        let fake_dir = PathBuf::from("/nonexistent/read-only-store");

        let mut builder = Builder::new(
            bs,
            ds,
            mock,
            test_pis(),
            fake_dir,
            nix_compat::store_path::STORE_DIR,
            test_keypair(),
            test_trusted_keys(),
            true,
            false,
        );

        let mut kp = DerivationRegistry::default();
        let (drv_path, _drv) = build_and_register("ro-test", &[], &mut kp);

        let outcome = builder.build(&drv_path, &mut kp).await.unwrap();
        assert!(!outcome.cached);
        assert!(outcome.outputs.contains_key("out"));
    }

    // ── build_all tests ─────────────────────────────────────────

    #[tokio::test]
    async fn build_all_multiple_roots_shared_dep() {
        // A (leaf) → B (root), A → C (root).
        // build_all([B, C]) should build A once, then B and C.
        let bs = MemoryBlobService::default();
        let ds = tmp_ds();
        let (mock, calls) = MockBuildService::new(bs.clone());

        let mut builder = Builder::new(
            bs,
            ds,
            mock,
            test_pis(),
            PathBuf::from("/nix/store"),
            nix_compat::store_path::STORE_DIR,
            test_keypair(),
            test_trusted_keys(),
            true,
            false,
        );

        let mut kp = DerivationRegistry::default();
        let (a_path, _) = build_and_register("shared", &[], &mut kp);
        let (b_path, _) = build_and_register("root-b", &[(a_path.clone(), "out")], &mut kp);
        let (c_path, _) = build_and_register("root-c", &[(a_path.clone(), "out")], &mut kp);

        let outcomes = builder.build_all(&[b_path.clone(), c_path.clone()], &mut kp, 2).await.unwrap();

        // Should return outcomes for roots only.
        assert_eq!(outcomes.len(), 2, "should return 2 root outcomes");
        let outcome_drv_names: Vec<String> = outcomes.iter().map(|o| o.drv_path.name().to_string()).collect();
        assert!(
            outcome_drv_names.iter().any(|n| n.contains("root-b")),
            "should include root-b: {outcome_drv_names:?}"
        );
        assert!(
            outcome_drv_names.iter().any(|n| n.contains("root-c")),
            "should include root-c: {outcome_drv_names:?}"
        );

        // Shared dep 'shared' should build exactly once.
        let recorded = calls.lock().unwrap();
        assert_eq!(recorded.len(), 3, "should build shared + root-b + root-c: {:?}", *recorded);
        let shared_builds = recorded.iter().filter(|args| args.iter().any(|a| a.contains("shared"))).count();
        assert_eq!(shared_builds, 1, "shared dep should build once");
    }

    #[tokio::test]
    async fn build_all_empty_roots() {
        let bs = MemoryBlobService::default();
        let ds = tmp_ds();
        let (mock, _calls) = MockBuildService::new(bs.clone());

        let mut builder = Builder::new(
            bs,
            ds,
            mock,
            test_pis(),
            PathBuf::from("/nix/store"),
            nix_compat::store_path::STORE_DIR,
            test_keypair(),
            test_trusted_keys(),
            true,
            false,
        );

        let mut kp = DerivationRegistry::default();
        let outcomes = builder.build_all(&[], &mut kp, 2).await.unwrap();
        assert!(outcomes.is_empty());
    }

    #[tokio::test]
    async fn build_all_ordering_deps_before_dependents() {
        // A (leaf) → B → C (root).
        // Build order must be A, B, C.
        let bs = MemoryBlobService::default();
        let ds = tmp_ds();
        let (mock, calls) = MockBuildService::new(bs.clone());

        let mut builder = Builder::new(
            bs,
            ds,
            mock,
            test_pis(),
            PathBuf::from("/nix/store"),
            nix_compat::store_path::STORE_DIR,
            test_keypair(),
            test_trusted_keys(),
            true,
            false,
        );

        let mut kp = DerivationRegistry::default();
        let (a_path, _) = build_and_register("leaf-a", &[], &mut kp);
        let (b_path, _) = build_and_register("mid-b", &[(a_path.clone(), "out")], &mut kp);
        let (c_path, _) = build_and_register("top-c", &[(b_path.clone(), "out")], &mut kp);

        let outcomes = builder.build_all(&[c_path.clone()], &mut kp, 2).await.unwrap();

        assert_eq!(outcomes.len(), 1, "one root");
        assert!(outcomes[0].drv_path.name().contains("top-c"));

        let recorded = calls.lock().unwrap();
        assert_eq!(recorded.len(), 3);

        // Verify ordering: leaf-a before mid-b before top-c.
        let pos_a = recorded.iter().position(|args| args.iter().any(|a| a.contains("leaf-a"))).unwrap();
        let pos_b = recorded.iter().position(|args| args.iter().any(|a| a.contains("mid-b"))).unwrap();
        let pos_c = recorded.iter().position(|args| args.iter().any(|a| a.contains("top-c"))).unwrap();
        assert!(pos_a < pos_b, "leaf-a must build before mid-b");
        assert!(pos_b < pos_c, "mid-b must build before top-c");
    }

    #[tokio::test]
    async fn build_all_disjoint_trees() {
        // Two independent trees: A→B and C→D.
        let bs = MemoryBlobService::default();
        let ds = tmp_ds();
        let (mock, calls) = MockBuildService::new(bs.clone());

        let mut builder = Builder::new(
            bs,
            ds,
            mock,
            test_pis(),
            PathBuf::from("/nix/store"),
            nix_compat::store_path::STORE_DIR,
            test_keypair(),
            test_trusted_keys(),
            true,
            false,
        );

        let mut kp = DerivationRegistry::default();
        let (a_path, _) = build_and_register("tree1-leaf", &[], &mut kp);
        let (b_path, _) = build_and_register("tree1-root", &[(a_path.clone(), "out")], &mut kp);
        let (c_path, _) = build_and_register("tree2-leaf", &[], &mut kp);
        let (d_path, _) = build_and_register("tree2-root", &[(c_path.clone(), "out")], &mut kp);

        let outcomes = builder.build_all(&[b_path.clone(), d_path.clone()], &mut kp, 2).await.unwrap();

        assert_eq!(outcomes.len(), 2);

        let recorded = calls.lock().unwrap();
        assert_eq!(recorded.len(), 4, "all 4 nodes should build");

        // Each leaf must build before its root.
        let pos_a = recorded.iter().position(|args| args.iter().any(|a| a.contains("tree1-leaf"))).unwrap();
        let pos_b = recorded.iter().position(|args| args.iter().any(|a| a.contains("tree1-root"))).unwrap();
        let pos_c = recorded.iter().position(|args| args.iter().any(|a| a.contains("tree2-leaf"))).unwrap();
        let pos_d = recorded.iter().position(|args| args.iter().any(|a| a.contains("tree2-root"))).unwrap();
        assert!(pos_a < pos_b, "tree1-leaf before tree1-root");
        assert!(pos_c < pos_d, "tree2-leaf before tree2-root");
    }

    // ── Multi-output tests ─────────────────────────────────────────

    #[tokio::test]
    async fn multi_output_build_returns_all_outputs() {
        use crate::test_support::build_and_register_multi;

        let bs = MemoryBlobService::default();
        let ds = tmp_ds();
        let (mock, calls) = MockBuildService::new(bs.clone());

        let mut builder = Builder::new(
            bs,
            ds,
            mock,
            test_pis(),
            PathBuf::from("/nix/store"),
            nix_compat::store_path::STORE_DIR,
            test_keypair(),
            test_trusted_keys(),
            true,
            false,
        );

        let mut kp = DerivationRegistry::default();
        let (drv_path, _drv) = build_and_register_multi("multi-out", &["out", "dev", "lib"], &[], &mut kp);

        let outcome = builder.build(&drv_path, &mut kp).await.unwrap();
        assert!(!outcome.cached);

        // Should have PathInfo for all 3 outputs.
        assert_eq!(outcome.outputs.len(), 3, "expected 3 outputs: {:?}", outcome.outputs.keys().collect::<Vec<_>>());
        assert!(outcome.outputs.contains_key("out"));
        assert!(outcome.outputs.contains_key("dev"));
        assert!(outcome.outputs.contains_key("lib"));

        // Each output should have a distinct store path.
        let paths: Vec<String> = outcome.outputs.values().map(|pi| pi.store_path.to_absolute_path()).collect();
        assert_eq!(paths.len(), 3);
        assert_ne!(paths[0], paths[1]);
        assert_ne!(paths[1], paths[2]);
        assert_ne!(paths[0], paths[2]);

        // Build should have been called exactly once.
        let recorded = calls.lock().unwrap();
        assert_eq!(recorded.len(), 1);
    }

    #[tokio::test]
    async fn multi_output_env_has_outputs_var() {
        use crate::test_support::build_and_register_multi;

        let mut kp = DerivationRegistry::default();
        let (_drv_path, drv) = build_and_register_multi("env-test", &["out", "dev", "man"], &[], &mut kp);

        // The outputs env var should list all output names.
        let outputs_env = drv.environment.get("outputs").unwrap();
        let outputs_str = std::str::from_utf8(outputs_env.as_ref()).unwrap();
        assert_eq!(outputs_str, "out dev man");

        // Each output name should have a store path env var.
        for name in &["out", "dev", "man"] {
            let val = drv.environment.get(*name).unwrap();
            let s = std::str::from_utf8(val.as_ref()).unwrap();
            assert!(s.starts_with("/nix/store/"), "${name} should be a store path: {s}");
        }
    }

    #[tokio::test]
    async fn multi_output_distinct_store_paths() {
        use crate::test_support::build_and_register_multi;

        let mut kp = DerivationRegistry::default();
        let (_drv_path, drv) = build_and_register_multi("paths-test", &["out", "dev", "lib"], &[], &mut kp);

        let out = drv.outputs["out"].path.as_ref().unwrap().to_absolute_path();
        let dev = drv.outputs["dev"].path.as_ref().unwrap().to_absolute_path();
        let lib = drv.outputs["lib"].path.as_ref().unwrap().to_absolute_path();

        // "out" output uses the base name; others get a suffix.
        assert!(out.ends_with("-paths-test"), "out: {out}");
        assert!(dev.ends_with("-paths-test-dev"), "dev: {dev}");
        assert!(lib.ends_with("-paths-test-lib"), "lib: {lib}");

        assert_ne!(out, dev);
        assert_ne!(dev, lib);
    }

    #[tokio::test]
    async fn multi_output_dep_mounts_all_outputs() {
        // Parent depends on a multi-output dep. All outputs should be
        // available as sandbox inputs.
        use crate::test_support::build_and_register_multi;

        let bs = MemoryBlobService::default();
        let ds = tmp_ds();
        let (mock, calls) = MockBuildService::new(bs.clone());

        let mut builder = Builder::new(
            bs,
            ds,
            mock,
            test_pis(),
            PathBuf::from("/nix/store"),
            nix_compat::store_path::STORE_DIR,
            test_keypair(),
            test_trusted_keys(),
            true,
            false,
        );

        let mut kp = DerivationRegistry::default();
        let (dep_path, _dep_drv) = build_and_register_multi("multi-dep", &["out", "dev"], &[], &mut kp);

        // Parent depends on both "out" and "dev" of the dep.
        let mut outputs = BTreeMap::new();
        outputs.insert("out".to_string(), Output {
            path: None,
            ca_hash: None,
        });
        let mut environment = BTreeMap::new();
        environment.insert("name".to_string(), "consumer".into());
        environment.insert("system".to_string(), "x86_64-linux".into());
        environment.insert("builder".to_string(), "/bin/sh".into());
        environment.insert("out".to_string(), "".into());
        environment.insert("outputs".to_string(), "out".into());

        let mut input_derivations = BTreeMap::new();
        let mut dep_outputs = BTreeSet::new();
        dep_outputs.insert("out".to_string());
        dep_outputs.insert("dev".to_string());
        input_derivations.insert(dep_path.clone(), dep_outputs);

        let mut parent_drv = Derivation {
            arguments: vec!["-c".into(), "echo consumer > $out".into()],
            builder: "/bin/sh".to_string(),
            environment,
            input_derivations,
            input_sources: BTreeSet::new(),
            outputs,
            system: "x86_64-linux".to_string(),
        };

        let hdm = parent_drv.hash_derivation_modulo(|parent_path| {
            kp.get_hdm_by_drv_path(&parent_path.to_absolute_path()).expect("parent should be in known_paths")
        });
        parent_drv.calculate_output_paths("consumer", &hdm).unwrap();
        let parent_drv_path = parent_drv.calculate_derivation_path("consumer").unwrap();

        let mut fake_hash = [0u8; 32];
        for (i, b) in "consumer".bytes().enumerate().take(32) {
            fake_hash[i] = b;
        }
        kp.insert(parent_drv_path.clone(), hdm, parent_drv.clone(), false, None);

        let outcome = builder.build(&parent_drv_path, &mut kp).await.unwrap();
        assert!(!outcome.cached);

        // Should have built the dep and the consumer (2 builds total).
        let recorded = calls.lock().unwrap();
        assert_eq!(recorded.len(), 2, "dep + consumer: {:?}", *recorded);
    }

    #[tokio::test]
    async fn multi_output_cache_requires_all_outputs() {
        // If only some outputs are cached (PathInfo + castore),
        // the derivation is a cache miss.
        use crate::test_support::build_and_register_multi;

        let bs = MemoryBlobService::default();
        let ds = tmp_ds();
        let (mock, calls) = MockBuildService::new(bs.clone());
        let pis = test_pis();

        let mut kp = DerivationRegistry::default();
        let (drv_path, drv) = build_and_register_multi("partial-cache", &["out", "dev"], &[], &mut kp);

        // Put only the "out" output in castore + PathInfo. "dev" has no PathInfo.
        let out_path = drv.outputs["out"].path.as_ref().unwrap();
        let node = put_blob(&bs, b"cached out").await;

        let path_info = PathInfo {
            store_path: out_path.clone(),
            node,
            references: vec![],
            nar_size: 0,
            nar_sha256: [0u8; 32],
            signatures: vec![],
            deriver: Some(drv_path.clone()),
            ca: None,
        };
        use snix_store::pathinfoservice::PathInfoService;
        pis.put(path_info).await.unwrap();

        let mut builder = Builder::new(
            bs,
            ds,
            mock,
            pis,
            PathBuf::from("/nix/store"),
            nix_compat::store_path::STORE_DIR,
            test_keypair(),
            test_trusted_keys(),
            true,
            false,
        );

        let outcome = builder.build(&drv_path, &mut kp).await.unwrap();
        // Cache miss because "dev" has no PathInfo.
        assert!(!outcome.cached, "partial cache should not count as hit");

        let recorded = calls.lock().unwrap();
        assert_eq!(recorded.len(), 1, "should rebuild");
    }

    // ── Remote substitution tests ─────────────────────────────────

    #[tokio::test]
    async fn remote_cache_hit_skips_build() {
        let bs = MemoryBlobService::default();
        let ds = tmp_ds();
        let (mock, calls) = MockBuildService::new(bs.clone());
        let local_pis = test_pis();
        let remote_pis = test_pis();

        let mut kp = DerivationRegistry::default();
        let (drv_path, drv) = build_and_register("remote-hit", &[], &mut kp);

        // Put content in castore + remote PathInfo.
        let out_path = drv.outputs["out"].path.as_ref().unwrap();
        let node = put_blob(&bs, b"remote content").await;

        let path_info = PathInfo {
            store_path: out_path.clone(),
            node,
            references: vec![],
            nar_size: 14,
            nar_sha256: [1u8; 32],
            signatures: vec![],
            deriver: Some(drv_path.clone()),
            ca: None,
        };
        use snix_store::pathinfoservice::PathInfoService as _;
        remote_pis.put(path_info).await.unwrap();

        let remote: Arc<dyn snix_store::pathinfoservice::PathInfoService> = Arc::new(remote_pis);
        let mut builder = Builder::with_state_dir(
            Arc::new(bs) as Arc<dyn BlobService>,
            Arc::new(ds) as Arc<dyn DirectoryService>,
            mock,
            Arc::new(local_pis) as Arc<dyn snix_store::pathinfoservice::PathInfoService>,
            PathBuf::from("/nix/store"),
            None,
            Some(remote),
            nix_compat::store_path::STORE_DIR,
            test_keypair(),
            test_trusted_keys(),
            true,
            false,
        );

        let outcome = builder.build(&drv_path, &mut kp).await.unwrap();
        assert!(outcome.cached, "should be cached via remote substitution");

        let recorded = calls.lock().unwrap();
        assert_eq!(recorded.len(), 0, "should NOT call do_build");
    }

    #[tokio::test]
    async fn remote_signed_cache_hit_verifies_and_skips_build() {
        use snix_store::pathinfoservice::PathInfoService as _;

        let bs = MemoryBlobService::default();
        let ds = tmp_ds();
        let (mock, calls) = MockBuildService::new(bs.clone());
        let local_pis = test_pis();
        let remote_pis = test_pis();

        let mut kp = DerivationRegistry::default();
        let (drv_path, drv) = build_and_register("remote-signed-hit", &[], &mut kp);

        let remote_keypair = crate::signing::generate_keypair().0;
        let out_path = drv.outputs["out"].path.as_ref().unwrap();
        let node = put_blob(&bs, b"remote signed content").await;
        let mut path_info = PathInfo {
            store_path: out_path.clone(),
            node,
            references: vec![],
            nar_size: 21,
            nar_sha256: [4u8; 32],
            signatures: vec![],
            deriver: Some(drv_path.clone()),
            ca: None,
        };
        crate::signing::sign_pathinfo(&mut path_info, &remote_keypair.signing_key);
        remote_pis.put(path_info).await.unwrap();

        let trusted_keys = crate::signing::build_trusted_keys(
            &test_keypair(),
            Some(std::slice::from_ref(&remote_keypair.verifying_key)),
        );
        let remote: Arc<dyn snix_store::pathinfoservice::PathInfoService> = Arc::new(remote_pis);
        let mut builder = Builder::with_state_dir(
            Arc::new(bs) as Arc<dyn BlobService>,
            Arc::new(ds) as Arc<dyn DirectoryService>,
            mock,
            Arc::new(local_pis) as Arc<dyn snix_store::pathinfoservice::PathInfoService>,
            PathBuf::from("/nix/store"),
            None,
            Some(remote),
            nix_compat::store_path::STORE_DIR,
            test_keypair(),
            trusted_keys,
            false,
            false,
        );

        let outcome = builder.build(&drv_path, &mut kp).await.unwrap();
        assert!(outcome.cached, "signed remote substitution should verify and skip build");

        let recorded = calls.lock().unwrap();
        assert_eq!(recorded.len(), 0, "should NOT call do_build when remote signature is trusted");
    }

    #[tokio::test]
    async fn unsigned_remote_path_rejected_without_trust_unsigned() {
        use snix_store::pathinfoservice::PathInfoService as _;

        let bs = MemoryBlobService::default();
        let ds = tmp_ds();
        let (mock, calls) = MockBuildService::new(bs.clone());
        let local_pis = test_pis();
        let remote_pis = test_pis();

        let mut kp = DerivationRegistry::default();
        let (drv_path, drv) = build_and_register("remote-unsigned", &[], &mut kp);

        let out_path = drv.outputs["out"].path.as_ref().unwrap();
        let node = put_blob(&bs, b"remote unsigned content").await;
        let path_info = PathInfo {
            store_path: out_path.clone(),
            node,
            references: vec![],
            nar_size: 23,
            nar_sha256: [5u8; 32],
            signatures: vec![],
            deriver: Some(drv_path.clone()),
            ca: None,
        };
        remote_pis.put(path_info).await.unwrap();

        let remote: Arc<dyn snix_store::pathinfoservice::PathInfoService> = Arc::new(remote_pis);
        let mut builder = Builder::with_state_dir(
            Arc::new(bs) as Arc<dyn BlobService>,
            Arc::new(ds) as Arc<dyn DirectoryService>,
            mock,
            Arc::new(local_pis) as Arc<dyn snix_store::pathinfoservice::PathInfoService>,
            PathBuf::from("/nix/store"),
            None,
            Some(remote),
            nix_compat::store_path::STORE_DIR,
            test_keypair(),
            test_trusted_keys(),
            false,
            false,
        );

        let outcome = builder.build(&drv_path, &mut kp).await.unwrap();
        assert!(!outcome.cached, "unsigned remote path must be rejected when trust_unsigned is false");

        let recorded = calls.lock().unwrap();
        assert_eq!(recorded.len(), 1, "builder should fall through to local build");
    }

    #[tokio::test]
    async fn remote_miss_falls_through_to_build() {
        let bs = MemoryBlobService::default();
        let ds = tmp_ds();
        let (mock, calls) = MockBuildService::new(bs.clone());
        let local_pis = test_pis();
        // Remote is empty — no PathInfo.
        let remote_pis = test_pis();

        let mut kp = DerivationRegistry::default();
        let (drv_path, _drv) = build_and_register("remote-miss", &[], &mut kp);

        let remote: Arc<dyn snix_store::pathinfoservice::PathInfoService> = Arc::new(remote_pis);
        let mut builder = Builder::with_state_dir(
            Arc::new(bs) as Arc<dyn BlobService>,
            Arc::new(ds) as Arc<dyn DirectoryService>,
            mock,
            Arc::new(local_pis) as Arc<dyn snix_store::pathinfoservice::PathInfoService>,
            PathBuf::from("/nix/store"),
            None,
            Some(remote),
            nix_compat::store_path::STORE_DIR,
            test_keypair(),
            test_trusted_keys(),
            true,
            false,
        );

        let outcome = builder.build(&drv_path, &mut kp).await.unwrap();
        assert!(!outcome.cached, "should build locally on remote miss");

        let recorded = calls.lock().unwrap();
        assert_eq!(recorded.len(), 1, "should call do_build");
    }

    #[tokio::test]
    async fn remote_hit_persisted_to_local() {
        // After a remote cache hit, the PathInfo is written to the
        // local pathinfo service. A second build should hit local.
        let bs = MemoryBlobService::default();
        let ds = tmp_ds();
        let remote_pis = test_pis();

        // Wrap local PIS in Arc so both builders share it.
        let local_pis = Arc::new(test_pis());

        let mut kp = DerivationRegistry::default();
        let (drv_path, drv) = build_and_register("persist-local", &[], &mut kp);

        let out_path = drv.outputs["out"].path.as_ref().unwrap();
        let node = put_blob(&bs, b"persist me").await;

        let path_info = PathInfo {
            store_path: out_path.clone(),
            node,
            references: vec![],
            nar_size: 10,
            nar_sha256: [2u8; 32],
            signatures: vec![],
            deriver: Some(drv_path.clone()),
            ca: None,
        };
        use snix_store::pathinfoservice::PathInfoService as _;
        remote_pis.put(path_info).await.unwrap();

        let remote: Arc<dyn snix_store::pathinfoservice::PathInfoService> = Arc::new(remote_pis);
        let (mock1, _) = MockBuildService::new(bs.clone());
        let mut builder = Builder::with_state_dir(
            Arc::new(bs.clone()) as Arc<dyn BlobService>,
            Arc::new(ds.clone()) as Arc<dyn DirectoryService>,
            mock1,
            Arc::new(local_pis.clone()) as Arc<dyn snix_store::pathinfoservice::PathInfoService>,
            PathBuf::from("/nix/store"),
            None,
            Some(remote),
            nix_compat::store_path::STORE_DIR,
            test_keypair(),
            test_trusted_keys(),
            true,
            false,
        );

        // First build: remote hit.
        let outcome = builder.build(&drv_path, &mut kp).await.unwrap();
        assert!(outcome.cached);

        // Second build: fresh builder with NO remote. Local should have it.
        let (mock2, calls2) = MockBuildService::new(bs.clone());
        let mut builder2 = Builder::new(
            bs,
            ds,
            mock2,
            local_pis,
            PathBuf::from("/nix/store"),
            nix_compat::store_path::STORE_DIR,
            test_keypair(),
            test_trusted_keys(),
            true,
            false,
        );

        let outcome2 = builder2.build(&drv_path, &mut kp).await.unwrap();
        assert!(outcome2.cached, "should hit local cache after write-through");

        let recorded2 = calls2.lock().unwrap();
        assert_eq!(recorded2.len(), 0, "no build on second pass");
    }

    #[tokio::test]
    async fn fod_skips_remote_cache() {
        let bs = MemoryBlobService::default();
        let ds = tmp_ds();
        let (mock, calls) = MockBuildService::new(bs.clone());
        let local_pis = test_pis();
        let remote_pis = test_pis();

        // Build a FOD derivation (has ca_hash on the output).
        let mut kp = DerivationRegistry::default();
        let mut outputs = BTreeMap::new();
        outputs.insert("out".to_string(), nix_compat::derivation::Output {
            path: None,
            ca_hash: Some(nix_compat::nixhash::CAHash::Flat(nix_compat::nixhash::NixHash::Sha256([42u8; 32]))),
        });
        let mut environment = BTreeMap::new();
        environment.insert("name".to_string(), "fod-test".into());
        environment.insert("system".to_string(), "x86_64-linux".into());
        environment.insert("builder".to_string(), "/bin/sh".into());
        environment.insert("out".to_string(), "".into());
        environment.insert("outputs".to_string(), "out".into());

        let mut drv = nix_compat::derivation::Derivation {
            arguments: vec!["-c".into(), "echo fod > $out".into()],
            builder: "/bin/sh".to_string(),
            environment,
            input_derivations: BTreeMap::new(),
            input_sources: BTreeSet::new(),
            outputs,
            system: "x86_64-linux".to_string(),
        };

        let hdm = drv.hash_derivation_modulo(|_| panic!("no input drvs"));
        drv.calculate_output_paths("fod-test", &hdm).unwrap();
        let drv_path = drv.calculate_derivation_path("fod-test").unwrap();

        let mut fake_hash = [0u8; 32];
        for (i, b) in "fod-test".bytes().enumerate().take(32) {
            fake_hash[i] = b;
        }
        kp.insert(drv_path.clone(), hdm, drv.clone(), false, None);

        // Put PathInfo in remote for the FOD output path.
        let out_path = drv.outputs["out"].path.as_ref().unwrap();
        let node = put_blob(&bs, b"fod content").await;
        let pi = PathInfo {
            store_path: out_path.clone(),
            node,
            references: vec![],
            nar_size: 11,
            nar_sha256: [3u8; 32],
            signatures: vec![],
            deriver: Some(drv_path.clone()),
            ca: None,
        };
        use snix_store::pathinfoservice::PathInfoService as _;
        remote_pis.put(pi).await.unwrap();

        let remote: Arc<dyn snix_store::pathinfoservice::PathInfoService> = Arc::new(remote_pis);
        let mut builder = Builder::with_state_dir(
            Arc::new(bs) as Arc<dyn BlobService>,
            Arc::new(ds) as Arc<dyn DirectoryService>,
            mock,
            Arc::new(local_pis) as Arc<dyn snix_store::pathinfoservice::PathInfoService>,
            PathBuf::from("/nix/store"),
            None,
            Some(remote),
            nix_compat::store_path::STORE_DIR,
            test_keypair(),
            test_trusted_keys(),
            true,
            false,
        );

        // FOD should bypass remote and attempt a local build.
        // The build itself may fail (mock output won't match ca_hash)
        // but the critical assertion is that do_build was called,
        // proving substitution was skipped.
        let _result = builder.build(&drv_path, &mut kp).await;

        let recorded = calls.lock().unwrap();
        assert_eq!(recorded.len(), 1, "FOD should trigger sandbox build, not substitution");
    }

    /// A PathInfoService that always fails on get().
    struct FailingRemoteService;

    #[async_trait]
    impl snix_store::pathinfoservice::PathInfoService for FailingRemoteService {
        async fn get(&self, _digest: [u8; 20]) -> Result<Option<PathInfo>, snix_store::pathinfoservice::Error> {
            Err(Box::new(std::io::Error::new(std::io::ErrorKind::ConnectionRefused, "simulated network error")))
        }
        async fn put(&self, _pi: PathInfo) -> Result<PathInfo, snix_store::pathinfoservice::Error> {
            unimplemented!()
        }
        fn list(&self) -> futures::stream::BoxStream<'static, Result<PathInfo, snix_store::pathinfoservice::Error>> {
            unimplemented!()
        }
    }

    #[tokio::test]
    async fn remote_error_treated_as_miss() {
        let bs = MemoryBlobService::default();
        let ds = tmp_ds();
        let (mock, calls) = MockBuildService::new(bs.clone());
        let local_pis = test_pis();

        let mut kp = DerivationRegistry::default();
        let (drv_path, _drv) = build_and_register("remote-err", &[], &mut kp);

        let remote: Arc<dyn snix_store::pathinfoservice::PathInfoService> = Arc::new(FailingRemoteService);
        let mut builder = Builder::with_state_dir(
            Arc::new(bs) as Arc<dyn BlobService>,
            Arc::new(ds) as Arc<dyn DirectoryService>,
            mock,
            Arc::new(local_pis) as Arc<dyn snix_store::pathinfoservice::PathInfoService>,
            PathBuf::from("/nix/store"),
            None,
            Some(remote),
            nix_compat::store_path::STORE_DIR,
            test_keypair(),
            test_trusted_keys(),
            true,
            false,
        );

        // Remote error should be swallowed — build proceeds.
        let outcome = builder.build(&drv_path, &mut kp).await.unwrap();
        assert!(!outcome.cached, "should build locally on remote error");

        let recorded = calls.lock().unwrap();
        assert_eq!(recorded.len(), 1, "should call do_build");
    }

    // ── Blob persistence across Builder instances ─────────────

    /// Helper: write bytes into an ObjectStoreBlobService-backed store.
    async fn put_blob_obj(bs: &snix_castore::blobservice::ObjectStoreBlobService, content: &[u8]) -> Node {
        let mut writer = BlobService::open_write(bs).await;
        writer.write_all(content).await.unwrap();
        let digest = writer.close().await.unwrap();
        Node::File {
            digest,
            size: content.len() as u64,
            executable: false,
        }
    }

    #[tokio::test]
    async fn persistent_blob_cache_hit_across_builder_instances() {
        use snix_castore::blobservice::ObjectStoreBlobService;
        use snix_store::pathinfoservice::PathInfoService;
        use snix_store::pathinfoservice::RedbPathInfoService;
        use snix_store::pathinfoservice::RedbPathInfoServiceConfig;

        let tmp = tempfile::tempdir().unwrap();
        let blob_dir = tmp.path().join("blobs");
        let pis_path = tmp.path().join("pathinfo.redb");
        std::fs::create_dir_all(&blob_dir).unwrap();

        let mut kp = DerivationRegistry::default();
        let (drv_path, drv) = build_and_register("persist-test", &[], &mut kp);
        let out_path = drv.outputs["out"].path.as_ref().unwrap();

        // --- First Builder instance: populate castore + pathinfo ---
        {
            let bs = std::sync::Arc::new(ObjectStoreBlobService::new_local(&blob_dir).unwrap());
            let pis = RedbPathInfoService::new("test".to_string(), RedbPathInfoServiceConfig {
                path: Some(pis_path.clone()),
                read_only: false,
                cache_size: None,
            })
            .await
            .unwrap();

            let node = put_blob_obj(&bs, b"persistent content").await;

            let path_info = PathInfo {
                store_path: out_path.clone(),
                node,
                references: vec![],
                nar_size: 0,
                nar_sha256: [0u8; 32],
                signatures: vec![],
                deriver: Some(drv_path.clone()),
                ca: None,
            };
            pis.put(path_info).await.unwrap();
            // Builder and services dropped here
        }

        // --- Second Builder instance: same dirs, fresh services ---
        {
            let bs = std::sync::Arc::new(ObjectStoreBlobService::new_local(&blob_dir).unwrap());
            let ds = tmp_ds();
            let (mock, calls) = MockBuildService::new(MemoryBlobService::default());
            let pis = RedbPathInfoService::new("test".to_string(), RedbPathInfoServiceConfig {
                path: Some(pis_path.clone()),
                read_only: false,
                cache_size: None,
            })
            .await
            .unwrap();

            let mut builder = Builder::new(
                bs,
                ds,
                mock,
                pis,
                PathBuf::from("/nix/store"),
                nix_compat::store_path::STORE_DIR,
                test_keypair(),
                test_trusted_keys(),
                true,
                false,
            );

            let mut kp2 = DerivationRegistry::default();
            let (drv_path2, _) = build_and_register("persist-test", &[], &mut kp2);

            let outcome = builder.build(&drv_path2, &mut kp2).await.unwrap();
            assert!(outcome.cached, "should be cached from persistent blob store");

            let recorded = calls.lock().unwrap();
            assert_eq!(recorded.len(), 0, "should NOT call do_build — blobs persisted on disk",);
        }
    }

    // ── Fetcher through DispatchBuildService + finish_build ──────────────
    //
    // These verify that fetcher derivations flow through the full
    // prepare → DispatchBuildService → FetchBuildService → finish_build
    // pipeline for both flat and recursive (NAR) hash modes.

    /// Build a fetchurl-style derivation. No `unpack` flag set.
    fn make_fetcher_drv_for_build(
        name: &str,
        url: &str,
        ca_hash: Option<nix_compat::nixhash::CAHash>,
        kp: &mut DerivationRegistry,
    ) -> (StorePath<String>, Derivation) {
        let mut outputs = BTreeMap::new();
        outputs.insert("out".to_string(), Output { path: None, ca_hash });
        let mut environment = BTreeMap::new();
        environment.insert("name".to_string(), name.into());
        environment.insert("system".to_string(), "builtin".into());
        environment.insert("builder".to_string(), "builtin:fetchurl".into());
        environment.insert("url".to_string(), url.into());
        environment.insert("out".to_string(), "".into());

        let mut drv = Derivation {
            arguments: vec![],
            builder: "builtin:fetchurl".to_string(),
            environment,
            input_derivations: BTreeMap::new(),
            input_sources: BTreeSet::new(),
            outputs,
            system: "builtin".to_string(),
        };
        let hdm = drv.hash_derivation_modulo(|_| panic!("no parent"));
        drv.calculate_output_paths(name, &hdm).unwrap();
        let drv_path = drv.calculate_derivation_path(name).unwrap();
        kp.insert(drv_path.clone(), hdm, drv.clone(), false, None);
        (drv_path, drv)
    }

    /// Build a fetchurl-style derivation with `unpack=1` (tarball mode).
    /// Uses `CAHash::Nar` for recursive hash verification.
    fn make_fetcher_drv_for_build_unpack(
        name: &str,
        url: &str,
        ca_hash: Option<nix_compat::nixhash::CAHash>,
        kp: &mut DerivationRegistry,
    ) -> (StorePath<String>, Derivation) {
        let mut outputs = BTreeMap::new();
        outputs.insert("out".to_string(), Output { path: None, ca_hash });
        let mut environment = BTreeMap::new();
        environment.insert("name".to_string(), name.into());
        environment.insert("system".to_string(), "builtin".into());
        environment.insert("builder".to_string(), "builtin:fetchurl".into());
        environment.insert("url".to_string(), url.into());
        environment.insert("unpack".to_string(), "1".into());
        environment.insert("out".to_string(), "".into());

        let mut drv = Derivation {
            arguments: vec![],
            builder: "builtin:fetchurl".to_string(),
            environment,
            input_derivations: BTreeMap::new(),
            input_sources: BTreeSet::new(),
            outputs,
            system: "builtin".to_string(),
        };
        let hdm = drv.hash_derivation_modulo(|_| panic!("no parent"));
        drv.calculate_output_paths(name, &hdm).unwrap();
        let drv_path = drv.calculate_derivation_path(name).unwrap();
        kp.insert(drv_path.clone(), hdm, drv.clone(), false, None);
        (drv_path, drv)
    }

    /// Create a gzipped tarball with a single file, return the path.
    fn create_test_tarball(file_name: &str, content: &[u8]) -> tempfile::NamedTempFile {
        let tmp_src = tempfile::tempdir().unwrap();
        let inner = tmp_src.path().join("project-v1");
        std::fs::create_dir(&inner).unwrap();
        std::fs::write(inner.join(file_name), content).unwrap();

        let tar_file = tempfile::NamedTempFile::new().unwrap();
        {
            let gz = flate2::write::GzEncoder::new(
                std::fs::File::create(tar_file.path()).unwrap(),
                flate2::Compression::fast(),
            );
            let mut builder = tar::Builder::new(gz);
            builder.append_dir_all("project-v1", &inner).unwrap();
            builder.into_inner().unwrap().finish().unwrap();
        }
        tar_file
    }

    /// Compute the NAR sha256 of a tarball's unpacked content, using
    /// the same extract+ingest path that FetchBuildService uses.
    async fn nar_sha256_of_tarball(tarball_path: &std::path::Path) -> [u8; 32] {
        let extract_dir = tempfile::tempdir().unwrap();
        let url = format!("file://{}", tarball_path.display());
        crate::fetcher::fetch_and_unpack(&url, extract_dir.path().to_str().unwrap()).unwrap();

        let bs = MemoryBlobService::default();
        let ds = tmp_ds();
        let node =
            snix_castore::import::fs::ingest_path::<_, _, _, &[u8]>(bs.clone(), ds.clone(), extract_dir.path(), None)
                .await
                .unwrap();

        use snix_store::nar::NarCalculationService;
        let renderer = snix_store::nar::SimpleRenderer::new(
            std::sync::Arc::new(bs) as std::sync::Arc<dyn snix_castore::blobservice::BlobService>,
            std::sync::Arc::new(ds) as std::sync::Arc<dyn snix_castore::directoryservice::DirectoryService>,
        );
        let (_size, sha256) = renderer.calculate_nar(&node).await.unwrap();
        sha256
    }

    /// A mock sandbox service that panics if called. Proves the
    /// DispatchBuildService never routes fetcher requests here.
    struct PanicSandboxService;

    #[async_trait]
    impl BuildService for PanicSandboxService {
        async fn do_build(&self, _request: BuildRequest) -> std::io::Result<BuildResult> {
            panic!("sandbox service must NOT be called for fetcher derivations");
        }
    }

    #[tokio::test]
    async fn fetcher_through_dispatch_service_hash_match() {
        use nix_compat::nixhash::CAHash;
        use nix_compat::nixhash::NixHash;
        use sha2::Digest;

        let content = b"fetcher integration test content";
        let digest: [u8; 32] = sha2::Sha256::digest(content).into();
        let ca = CAHash::Flat(NixHash::Sha256(digest));

        let tmp = tempfile::NamedTempFile::new().unwrap();
        std::fs::write(tmp.path(), content).unwrap();
        let url = format!("file://{}", tmp.path().display());

        let bs = MemoryBlobService::default();
        let ds = tmp_ds();

        // DispatchBuildService wrapping FetchBuildService + PanicSandboxService.
        // If DispatchBuildService ever routes the fetcher to the sandbox,
        // PanicSandboxService will panic and fail the test.
        let fetch_svc = crate::fetch_build_service::FetchBuildService::new(bs.clone(), ds.clone());
        let dispatch = crate::dispatch_build_service::DispatchBuildService::new(fetch_svc, PanicSandboxService);

        let output_dir = tempfile::tempdir().unwrap();
        let mut builder = Builder::new(
            bs,
            ds,
            dispatch,
            test_pis(),
            output_dir.path().to_path_buf(),
            nix_compat::store_path::STORE_DIR,
            test_keypair(),
            test_trusted_keys(),
            true,
            false,
        );

        let mut kp = DerivationRegistry::default();
        let (drv_path, _drv) = make_fetcher_drv_for_build("fetch-hash-match", &url, Some(ca), &mut kp);

        // Build through the full prepare → dispatch → finish pipeline.
        let outcome = builder.build(&drv_path, &mut kp).await.unwrap();

        // finish_build verified the FOD hash (would have returned
        // Error::FodHashMismatch otherwise) and persisted PathInfo.
        assert!(!outcome.cached, "first build should not be cached");
        assert!(outcome.outputs.contains_key("out"), "must produce 'out' output");
        let path_info = &outcome.outputs["out"];
        assert!(path_info.nar_size > 0, "PathInfo must have non-zero NAR size");
        // The CA field must match the declared flat hash.
        assert!(path_info.ca.is_some(), "PathInfo must have CA field for FOD output",);
    }

    #[tokio::test]
    async fn fetcher_through_dispatch_service_hash_mismatch() {
        use nix_compat::nixhash::CAHash;
        use nix_compat::nixhash::NixHash;

        use crate::worker::Worker;

        // Deliberately wrong hash.
        let wrong_hash = CAHash::Flat(NixHash::Sha256([0xAA; 32]));

        let tmp = tempfile::NamedTempFile::new().unwrap();
        std::fs::write(tmp.path(), b"mismatch test content").unwrap();
        let url = format!("file://{}", tmp.path().display());

        let bs = MemoryBlobService::default();
        let ds = tmp_ds();

        let fetch_svc = crate::fetch_build_service::FetchBuildService::new(bs.clone(), ds.clone());
        let dispatch = crate::dispatch_build_service::DispatchBuildService::new(fetch_svc, PanicSandboxService);

        let output_dir = tempfile::tempdir().unwrap();
        let mut builder = Builder::new(
            bs,
            ds,
            dispatch,
            test_pis(),
            output_dir.path().to_path_buf(),
            nix_compat::store_path::STORE_DIR,
            test_keypair(),
            test_trusted_keys(),
            true,
            false,
        );

        let mut kp = DerivationRegistry::default();
        let (drv_path, drv) = make_fetcher_drv_for_build("fetch-hash-mismatch", &url, Some(wrong_hash), &mut kp);

        // Use Worker directly to access FailedGoal.error (build_all
        // wraps it into a generic Store error).
        let mut worker = Worker::new(1);
        worker.want(&drv_path, &kp, true).unwrap();
        let result = worker.run(&mut builder, &mut kp).await.unwrap();

        // No successful outcomes.
        assert!(result.outcomes.is_empty(), "mismatch build must not succeed: {:?}", result.outcomes,);

        // Exactly one failure.
        assert_eq!(result.failed.len(), 1, "exactly one root must fail");
        let err_str = &result.failed[0].error;

        // Verify the error is specifically a FOD hash mismatch from
        // verify_fod_hash in finish_build, not a generic build failure.
        assert!(
            err_str.contains("FOD hash mismatch"),
            "error must be a FOD hash mismatch from verify_fod_hash, got: {err_str}"
        );
        // Verify expected hash is reported (the [0xAA; 32] we declared).
        assert!(
            err_str.contains("expected sha256-"),
            "error must report expected hash in SRI format, got: {err_str}"
        );
        // Verify actual hash is reported.
        assert!(err_str.contains(", got sha256-"), "error must report actual hash in SRI format, got: {err_str}");

        // Verify output was NOT persisted: verify_fod_hash returns Err
        // before persist_and_export_output runs in process_output,
        // so no PathInfo exists for the bad content. Check via pathinfo.
        let out_path = drv.outputs["out"].path.as_ref().unwrap();
        let pi = builder.store.pathinfo_service().get(*out_path.digest()).await;
        assert!(pi.is_ok() && pi.unwrap().is_none(), "PathInfo must NOT be persisted for hash-mismatched output");
        // Verify the output node was NOT stored in the session cache.
        assert!(
            !builder.store.output_nodes.contains_key(out_path),
            "output_nodes must NOT contain the mismatched output"
        );
    }

    // ── Recursive (NAR) fetcher tests ─────────────────────────────

    #[tokio::test]
    async fn fetcher_recursive_through_dispatch_hash_match() {
        use nix_compat::nixhash::CAHash;
        use nix_compat::nixhash::NixHash;

        // Create a tarball and pre-compute its NAR sha256.
        let tarball = create_test_tarball("hello.txt", b"recursive test content");
        let nar_sha256 = nar_sha256_of_tarball(tarball.path()).await;
        let ca = CAHash::Nar(NixHash::Sha256(nar_sha256));
        let url = format!("file://{}", tarball.path().display());

        let bs = MemoryBlobService::default();
        let ds = tmp_ds();

        let fetch_svc = crate::fetch_build_service::FetchBuildService::new(bs.clone(), ds.clone());
        let dispatch = crate::dispatch_build_service::DispatchBuildService::new(fetch_svc, PanicSandboxService);

        let output_dir = tempfile::tempdir().unwrap();
        let mut builder = Builder::new(
            bs,
            ds,
            dispatch,
            test_pis(),
            output_dir.path().to_path_buf(),
            nix_compat::store_path::STORE_DIR,
            test_keypair(),
            test_trusted_keys(),
            true,
            false,
        );

        let mut kp = DerivationRegistry::default();
        let (drv_path, _drv) = make_fetcher_drv_for_build_unpack("fetch-recursive-match", &url, Some(ca), &mut kp);

        let outcome = builder.build(&drv_path, &mut kp).await.unwrap();

        assert!(!outcome.cached);
        assert!(outcome.outputs.contains_key("out"));
        let path_info = &outcome.outputs["out"];
        assert!(path_info.nar_size > 0);
        // CA must be NAR (recursive), not Flat.
        match &path_info.ca {
            Some(nix_compat::nixhash::CAHash::Nar(NixHash::Sha256(h))) => {
                assert_eq!(h, &nar_sha256, "persisted CA hash must match declared NAR hash");
            }
            other => panic!("expected CAHash::Nar(Sha256), got {other:?}"),
        }
    }

    #[tokio::test]
    async fn fetcher_recursive_through_dispatch_hash_mismatch() {
        use nix_compat::nixhash::CAHash;
        use nix_compat::nixhash::NixHash;

        use crate::worker::Worker;

        // Tarball with wrong NAR hash.
        let tarball = create_test_tarball("data.txt", b"recursive mismatch content");
        let wrong_ca = CAHash::Nar(NixHash::Sha256([0xBB; 32]));
        let url = format!("file://{}", tarball.path().display());

        let bs = MemoryBlobService::default();
        let ds = tmp_ds();

        let fetch_svc = crate::fetch_build_service::FetchBuildService::new(bs.clone(), ds.clone());
        let dispatch = crate::dispatch_build_service::DispatchBuildService::new(fetch_svc, PanicSandboxService);

        let output_dir = tempfile::tempdir().unwrap();
        let mut builder = Builder::new(
            bs,
            ds,
            dispatch,
            test_pis(),
            output_dir.path().to_path_buf(),
            nix_compat::store_path::STORE_DIR,
            test_keypair(),
            test_trusted_keys(),
            true,
            false,
        );

        let mut kp = DerivationRegistry::default();
        let (drv_path, drv) =
            make_fetcher_drv_for_build_unpack("fetch-recursive-mismatch", &url, Some(wrong_ca), &mut kp);

        let mut worker = Worker::new(1);
        worker.want(&drv_path, &kp, true).unwrap();
        let result = worker.run(&mut builder, &mut kp).await.unwrap();

        assert!(result.outcomes.is_empty());
        assert_eq!(result.failed.len(), 1);
        let err_str = &result.failed[0].error;

        assert!(
            err_str.contains("FOD hash mismatch"),
            "recursive mismatch must report FOD hash mismatch, got: {err_str}"
        );
        assert!(err_str.contains("expected sha256-"), "must report expected NAR hash in SRI, got: {err_str}");
        assert!(err_str.contains(", got sha256-"), "must report actual NAR hash in SRI, got: {err_str}");

        // No PathInfo persisted.
        let out_path = drv.outputs["out"].path.as_ref().unwrap();
        let pi = builder.store.pathinfo_service().get(*out_path.digest()).await;
        assert!(pi.is_ok() && pi.unwrap().is_none(), "PathInfo must NOT be persisted for NAR hash mismatch");
        // No output node stored.
        assert!(
            !builder.store.output_nodes.contains_key(out_path),
            "output_nodes must NOT contain the mismatched recursive output"
        );
    }
}
