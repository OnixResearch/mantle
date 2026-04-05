//! Build orchestration: recursively build derivations, check cache,
//! persist outputs.

use std::collections::{BTreeMap, HashMap};
use std::path::PathBuf;
use std::sync::Arc;

use nix_compat::derivation::Derivation;
use nix_compat::store_path::StorePath;
use snix_build::buildservice::BuildService;
use snix_castore::blobservice::BlobService;
use snix_castore::directoryservice::DirectoryService;
use snix_castore::import::fs::ingest_path;
use snix_castore::Node;
use snix_store::nar::{NarCalculationService, SimpleRenderer};
use snix_store::path_info::PathInfo;
use tracing::{debug, info};

use crunch_glue::KnownPaths;
use snix_store::pathinfoservice::PathInfoService;

use crate::build_request::{collect_input_paths, derivation_to_build_request};
use crate::ca_mapping::CaMappings;
use crate::export::export_castore_to_disk;
use crate::fod::verify_fod_hash;
use crate::references::{parse_store_path, resolve_nix_closure, resolve_references};
use crate::Error;


/// The result of building a single derivation.
#[derive(Debug, Clone)]
pub struct BuildOutcome {
    /// The derivation store path (the .drv path).
    pub drv_path: StorePath<String>,
    /// Output name → PathInfo for each output.
    pub outputs: HashMap<String, PathInfo>,
    /// Whether the build was served from cache (output already existed).
    pub cached: bool,
    /// Captured build stdout+stderr, if available.
    pub log: Option<String>,
}

/// The logical store prefix. Derivation paths, output hashing, sandbox
/// layout, and KnownPaths lookups always use this. Matches Nix convention.
const LOGICAL_STORE_DIR: &str = "/nix/store";

/// Metadata saved during `prepare_build`, consumed by `finish_build`.
pub(crate) struct PreparedBuild {
    pub(crate) drv_path: StorePath<String>,
    pub(crate) drv_name: String,
    pub(crate) derivation: Derivation,
    pub(crate) build_request: snix_build::buildservice::BuildRequest,
    pub(crate) sandbox_inputs: BTreeMap<StorePath<String>, Node>,
    pub(crate) input_rewrites: Vec<(String, String)>,
    pub(crate) is_ca: bool,
}

/// Result of `prepare_build`: either already done (cached/fetcher) or
/// needs a sandbox build.
pub(crate) enum PrepareResult {
    Done(BuildOutcome),
    NeedsBuild(PreparedBuild),
}

/// Orchestrates the build pipeline: evaluating dependencies, checking
/// cache, running builds, persisting results.
pub struct Builder<BS, DS, BServ, PIS> {
    blob_service: BS,
    directory_service: DS,
    build_service: Arc<BServ>,
    pathinfo_service: PIS,
    /// Physical output directory on the host filesystem. This is where
    /// crunch writes build outputs (from `--store`). May differ from
    /// `LOGICAL_STORE_DIR` — e.g., `/tmp/mystore` while derivation
    /// paths still use `/nix/store`.
    #[allow(dead_code)] // used indirectly via output_dir_str
    output_dir: PathBuf,
    output_dir_str: String,
    /// Output store path → PathInfo for outputs built in this session.
    built_outputs: HashMap<String, PathInfo>,
    /// Output store path → Node (castore root node) for outputs built or
    /// ingested in this session. Needed by downstream builds that reference
    /// these as inputs.
    output_nodes: HashMap<StorePath<String>, Node>,
    /// Persistent CA derivation → output path mapping.
    ca_mappings: CaMappings,
    /// State directory for persisting ca_mappings.
    state_dir: Option<PathBuf>,
    verbose: bool,
}

impl<BS, DS, BServ, PIS> Builder<BS, DS, BServ, PIS>
where
    BS: BlobService + Clone + 'static,
    DS: DirectoryService + Clone + 'static,
    BServ: BuildService + 'static,
    PIS: PathInfoService,
{
    pub fn new(
        blob_service: BS,
        directory_service: DS,
        build_service: BServ,
        pathinfo_service: PIS,
        output_dir: PathBuf,
        verbose: bool,
    ) -> Self {
        Self::with_state_dir(blob_service, directory_service, build_service,
            pathinfo_service, output_dir, None, verbose)
    }

    /// Create a Builder with a state directory for persistent CA mappings.
    pub fn with_state_dir(
        blob_service: BS,
        directory_service: DS,
        build_service: BServ,
        pathinfo_service: PIS,
        output_dir: PathBuf,
        state_dir: Option<PathBuf>,
        verbose: bool,
    ) -> Self {
        let output_dir_str = output_dir.to_str()
            .unwrap_or(LOGICAL_STORE_DIR).to_string();
        let ca_mappings = state_dir
            .as_ref()
            .map(|d| CaMappings::load(d))
            .unwrap_or_default();
        Self {
            blob_service,
            directory_service,
            build_service: Arc::new(build_service),
            pathinfo_service,
            output_dir,
            output_dir_str,
            built_outputs: HashMap::new(),
            output_nodes: HashMap::new(),
            ca_mappings,
            state_dir,
            verbose,
        }
    }

    /// Build a derivation and all its dependencies. Returns the outcome.
    ///
    /// Delegates to `build_all` with a single root and max_jobs=1.
    /// Get a cloned Arc to the build service (for spawning tasks).
    pub(crate) fn build_service(&self) -> Arc<BServ> {
        self.build_service.clone()
    }

    /// Read the full content of a blob from castore.
    ///
    /// Used by the Worker to read `.drv` files from build outputs
    /// (dynamic derivation detection).
    pub(crate) async fn read_blob(
        &self,
        node: &snix_castore::Node,
    ) -> Result<Vec<u8>, Error> {
        use tokio::io::AsyncReadExt;

        let digest = match node {
            snix_castore::Node::File { digest, size, .. } => {
                // Tiger Style: fixed limit.
                const MAX_BLOB_READ: u64 = 8 * 1024 * 1024;
                if *size > MAX_BLOB_READ {
                    return Err(Error::Store(format!(
                        "blob too large to read: {size} bytes (limit: {MAX_BLOB_READ})"
                    )));
                }
                digest.clone()
            }
            other => {
                return Err(Error::Store(format!(
                    "read_blob called on non-file node: {other:?}"
                )));
            }
        };

        let mut reader = self.blob_service.open_read(&digest).await
            .map_err(|e| Error::Store(format!("opening blob: {e}")))?
            .ok_or_else(|| Error::Store(format!(
                "blob not found in castore: {}",
                data_encoding::HEXLOWER.encode(digest.as_slice())
            )))?;

        let mut buf = Vec::new();
        reader.read_to_end(&mut buf).await
            .map_err(|e| Error::Store(format!("reading blob: {e}")))?;

        Ok(buf)
    }

    /// Prefer `build_all` when building multiple roots.
    pub async fn build(
        &mut self,
        drv_path: &StorePath<String>,
        known_paths: &mut KnownPaths,
    ) -> Result<BuildOutcome, Error> {
        let mut outcomes = self.build_all(&[drv_path.clone()], known_paths, 1).await?;
        outcomes.pop().ok_or_else(|| Error::DerivationNotFound {
            path: drv_path.clone(),
        })
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
        known_paths: &mut KnownPaths,
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
            return Err(Error::Store(format!(
                "{} root build(s) failed",
                result.failed.len(),
            )));
        }

        Ok(result.outcomes)
    }

    /// Prepare a single derivation for building. Handles cache hits and
    /// fetchers inline; for sandbox builds, returns the prepared metadata
    /// needed to dispatch `do_build` and later `finish_build`.
    pub(crate) async fn prepare_build(
        &mut self,
        drv_path: &StorePath<String>,
        derivation: &Derivation,
        known_paths: &mut KnownPaths,
    ) -> Result<PrepareResult, Error> {
        let drv_name = drv_path.name().to_string();

        // 1. Cache check
        if let Some(cached_outputs) = self.check_cache(drv_path, derivation).await? {
            info!(drv = %drv_name, "all outputs cached, skipping build");
            return Ok(PrepareResult::Done(BuildOutcome {
                drv_path: drv_path.clone(),
                outputs: cached_outputs,
                cached: true,
                log: None,
            }));
        }

        // 2. Builtin fetcher bypass (runs inline, not dispatched).
        if crate::fetcher::is_builtin_fetcher(derivation) {
            let outcome = self
                .build_fetcher(drv_path, derivation, known_paths)
                .await?;
            return Ok(PrepareResult::Done(outcome));
        }

        // 3. Ensure input derivation outputs are in castore.
        for (input_drv_path, _output_names) in &derivation.input_derivations {
            let input_abs = input_drv_path.to_absolute_path_with_prefix(LOGICAL_STORE_DIR);
            if let Some(entry) = known_paths.get_by_drv_path(&input_abs) {
                let input_drv = entry.derivation.clone();
                self.ensure_input_nodes(&input_drv).await?;
            }
        }

        // 4. Resolve source inputs + closures.
        let all_source_paths = self.resolve_and_ingest_sources(derivation).await?;

        // 5. Collect sandbox inputs.
        let sandbox_inputs = self
            .collect_sandbox_inputs(derivation, known_paths, &all_source_paths)
            .await?;

        // 6. Create build request.
        let build_request = derivation_to_build_request(
            derivation, &sandbox_inputs, LOGICAL_STORE_DIR,
        )?;

        info!(drv = %drv_name, "building");
        if self.verbose {
            debug!(
                drv = %drv_name,
                inputs = sandbox_inputs.len(),
                outputs = build_request.outputs.len(),
                "dispatching sandbox build"
            );
        }

        let input_rewrites = self.collect_ca_input_rewrites(derivation, known_paths);
        let is_ca = derivation.outputs.values()
            .all(|o| o.path.is_none() && o.ca_hash.is_none());

        Ok(PrepareResult::NeedsBuild(PreparedBuild {
            drv_path: drv_path.clone(),
            drv_name,
            derivation: derivation.clone(),
            build_request,
            sandbox_inputs,
            input_rewrites,
            is_ca,
        }))
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
        known_paths: &mut KnownPaths,
    ) -> Result<BuildOutcome, Error> {
        let mut output_infos: HashMap<String, PathInfo> = HashMap::new();
        let output_names: Vec<String> = prepared.derivation.outputs.keys().cloned().collect();
        let is_multi_ca = prepared.is_ca && prepared.derivation.outputs.len() > 1;

        if is_multi_ca {
            output_infos = self.finish_build_multi_ca(
                prepared, &build_result, known_paths,
            ).await?;
        } else {
            for (i, (output_name, output)) in prepared.derivation.outputs.iter().enumerate() {
                let build_output = build_result.outputs.get(i).ok_or_else(|| {
                    Error::OutputMissing {
                        output: output_name.clone(),
                    }
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
                        &prepared.build_request,
                        &prepared.derivation,
                        known_paths,
                        prepared.is_ca,
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
                        pi.store_path.to_absolute_path_with_prefix(&self.output_dir_str)
                    })
                })
                .collect::<Vec<_>>(),
            "build succeeded"
        );

        Ok(BuildOutcome {
            drv_path: prepared.drv_path.clone(),
            outputs: output_infos,
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
        known_paths: &mut KnownPaths,
    ) -> Result<HashMap<String, PathInfo>, Error> {
        let nar_renderer = SimpleRenderer::new(
            self.blob_service.clone(),
            self.directory_service.clone(),
        );

        // Collect all provisionals for this derivation's outputs.
        let provisionals: Vec<(String, String)> = prepared.derivation.outputs.keys()
            .map(|name| {
                let prov = prepared.derivation.environment
                    .get(name)
                    .map(|v| String::from_utf8_lossy(v).to_string())
                    .unwrap_or_default();
                (name.clone(), prov)
            })
            .collect();

        // Generate distinct markers per output. Use the output name's
        // SHA-256 hash truncated to the provisional length. This gives
        // unique markers even when provisionals have the same length.
        let markers: Vec<(String, Vec<u8>, usize)> = provisionals.iter()
            .map(|(name, prov)| {
                let prov_len = prov.len();
                let hash = *blake3::hash(format!("crunch-ca-marker:{name}").as_bytes()).as_bytes();
                let mut marker = vec![0u8; prov_len];
                for (i, b) in hash.iter().cycle().enumerate().take(prov_len) {
                    marker[i] = *b;
                }
                (name.clone(), marker, prov_len)
            })
            .collect();

        // Pass 1: Apply input rewrites, replace all provisionals with
        // markers, compute CA paths.
        struct CaOutputIntermediate {
            name: String,
            marked_node: Node,
            ca_path: StorePath<String>,
            nar_size: u64,
            nar_sha256: [u8; 32],
        }

        let mut intermediates: Vec<CaOutputIntermediate> = Vec::new();
        let drv_name = &prepared.drv_name;
        let base_name = drv_name.strip_suffix(".drv").unwrap_or(drv_name);

        for (i, (output_name, _output)) in prepared.derivation.outputs.iter().enumerate() {
            let build_output = build_result.outputs.get(i).ok_or_else(|| {
                Error::OutputMissing { output: output_name.clone() }
            })?;

            // Apply transitive CA input rewrites.
            let mut node = build_output.node.clone();
            for (old_placeholder, new_path) in &prepared.input_rewrites {
                let (rewritten, _) = crate::rewrite::rewrite_node(
                    &node, old_placeholder.as_bytes(), new_path.as_bytes(),
                    &self.blob_service, &self.directory_service,
                ).await?;
                node = rewritten;
            }

            // Replace ALL provisionals with their markers.
            for ((_prov_name, prov), (_marker_name, marker, _len)) in
                provisionals.iter().zip(markers.iter())
            {
                let (rewritten, _) = crate::rewrite::rewrite_node(
                    &node, prov.as_bytes(), marker,
                    &self.blob_service, &self.directory_service,
                ).await?;
                node = rewritten;
            }

            // Compute NAR hash of the marker-replaced content.
            let (nar_size, nar_sha256) = nar_renderer
                .calculate_nar(&node)
                .await
                .map_err(|e| Error::NarCalculation(e.to_string()))?;

            // Compute CA store path.
            let ca_hash = nix_compat::nixhash::CAHash::Nar(
                nix_compat::nixhash::NixHash::Sha256(nar_sha256),
            );
            let path_name = if output_name == "out" {
                base_name.to_string()
            } else {
                format!("{base_name}-{output_name}")
            };
            let ca_path: StorePath<String> = nix_compat::store_path::build_ca_path_with_store_dir(
                &path_name, &ca_hash, Vec::<&str>::new(), false, LOGICAL_STORE_DIR,
            ).map_err(|e| Error::Store(format!("computing CA path: {e}")))?;

            intermediates.push(CaOutputIntermediate {
                name: output_name.clone(),
                marked_node: node,
                ca_path,
                nar_size,
                nar_sha256,
            });
        }

        // Build marker → final path replacement map.
        let final_rewrites: Vec<(&[u8], Vec<u8>)> = markers.iter()
            .zip(intermediates.iter())
            .map(|((_name, marker, _len), intermediate)| {
                let final_abs = intermediate.ca_path
                    .to_absolute_path_with_prefix(LOGICAL_STORE_DIR);
                (marker.as_slice(), final_abs.into_bytes())
            })
            .collect();

        // Pass 2: Replace all markers with final CA paths, persist.
        let drv_abs = prepared.drv_path.to_absolute_path_with_prefix(LOGICAL_STORE_DIR);
        let mut output_infos: HashMap<String, PathInfo> = HashMap::new();

        for intermediate in &intermediates {
            let mut final_node = intermediate.marked_node.clone();

            for (marker, final_bytes) in &final_rewrites {
                if marker.len() == final_bytes.len() {
                    let (rewritten, _) = crate::rewrite::rewrite_node(
                        &final_node, marker, final_bytes,
                        &self.blob_service, &self.directory_service,
                    ).await?;
                    final_node = rewritten;
                }
            }

            // Register resolved output.
            known_paths.resolve_output(
                &drv_abs, &intermediate.name, intermediate.ca_path.clone(),
            );

            let final_abs = intermediate.ca_path
                .to_absolute_path_with_prefix(LOGICAL_STORE_DIR);
            self.ca_mappings.insert(&drv_abs, &intermediate.name, &final_abs);
            if let Some(ref sd) = self.state_dir {
                self.ca_mappings.save(sd);
            }

            let display_abs = intermediate.ca_path
                .to_absolute_path_with_prefix(&self.output_dir_str);
            info!(
                drv = %drv_name,
                output = %intermediate.name,
                ca_path = %display_abs,
                "CA output path resolved"
            );

            // Resolve references.
            let build_output = build_result.outputs
                .get(intermediates.iter().position(|x| x.name == intermediate.name).unwrap())
                .unwrap();
            let references = resolve_references(
                &build_output.output_needles,
                &prepared.build_request.refscan_needles,
                &prepared.derivation,
                &prepared.sandbox_inputs,
            );

            let ca_field = Some(nix_compat::nixhash::CAHash::Nar(
                nix_compat::nixhash::NixHash::Sha256(intermediate.nar_sha256),
            ));

            let path_info = self.persist_and_export_output(
                &prepared.drv_path,
                &intermediate.ca_path,
                final_node,
                references,
                intermediate.nar_size,
                intermediate.nar_sha256,
                ca_field,
            ).await?;

            output_infos.insert(intermediate.name.clone(), path_info);
        }

        Ok(output_infos)
    }

    /// Validate source inputs, resolve Nix closures, and ingest all
    /// paths into castore. Returns the full set of source paths
    /// (declared + transitive closure).
    async fn resolve_and_ingest_sources(
        &mut self,
        derivation: &Derivation,
    ) -> Result<Vec<StorePath<String>>, Error> {
        let mut all_source_paths: Vec<StorePath<String>> =
            derivation.input_sources.iter().cloned().collect();

        // Source inputs are seed packages — always physically at
        // /nix/store/ regardless of --store output dir.
        for source_path in &derivation.input_sources {
            let abs = source_path.to_absolute_path();
            if !PathBuf::from(&abs).exists() {
                return Err(Error::SourceNotFound {
                    path: source_path.clone(),
                });
            }
            for closure_abs in resolve_nix_closure(&abs) {
                if let Some(sp) = parse_store_path(&closure_abs, LOGICAL_STORE_DIR) {
                    if !all_source_paths.contains(&sp) {
                        all_source_paths.push(sp);
                    }
                }
            }
        }

        // Ingest all source paths (declared + closure) into castore.
        // Source/closure paths live at /nix/store/ on the host.
        for source_path in &all_source_paths {
            let abs = PathBuf::from(source_path.to_absolute_path());
            if !abs.exists() {
                debug!(path = %source_path, "closure path not found on disk, skipping");
                continue;
            }
            if !self.output_nodes.contains_key(source_path) {
                let node = ingest_path::<_, _, _, &[u8]>(
                    self.blob_service.clone(),
                    self.directory_service.clone(),
                    &abs,
                    None,
                )
                .await
                .map_err(|e| Error::Sandbox(std::io::Error::other(format!(
                    "failed to ingest source {}: {e}",
                    source_path
                ))))?;
                self.output_nodes.insert(source_path.clone(), node);
            }
        }

        Ok(all_source_paths)
    }

    /// Gather all castore nodes needed as sandbox inputs: built
    /// dependency outputs, declared source paths, and closure paths.
    async fn collect_sandbox_inputs(
        &mut self,
        derivation: &Derivation,
        known_paths: &KnownPaths,
        source_paths: &[StorePath<String>],
    ) -> Result<BTreeMap<StorePath<String>, Node>, Error> {
        let mut input_paths = collect_input_paths(derivation, known_paths)?;
        for sp in source_paths {
            input_paths.insert(sp.clone());
        }

        let mut sandbox_inputs: BTreeMap<StorePath<String>, Node> = BTreeMap::new();
        for input_path in &input_paths {
            if let Some(node) = self.output_nodes.get(input_path) {
                sandbox_inputs.insert(input_path.clone(), node.clone());
            } else {
                // Try ingesting from disk as fallback.
                // Source inputs live at /nix/store/, built outputs at output_dir.
                let abs = self.resolve_host_path(input_path, derivation);
                if abs.exists() {
                    let node = ingest_path::<_, _, _, &[u8]>(
                        self.blob_service.clone(),
                        self.directory_service.clone(),
                        &abs,
                        None,
                    )
                    .await
                    .map_err(|e| Error::Sandbox(std::io::Error::other(format!(
                        "failed to ingest input {}: {e}",
                        input_path
                    ))))?;
                    self.output_nodes.insert(input_path.clone(), node.clone());
                    sandbox_inputs.insert(input_path.clone(), node);
                } else {
                    return Err(Error::SourceNotFound {
                        path: input_path.clone(),
                    });
                }
            }
        }
        Ok(sandbox_inputs)
    }

    /// Collect (old, new) path pairs for transitive CA input rewriting.
    /// If any input derivation is CA, its provisional env path may appear
    /// in our output and needs rewriting to the final resolved path.
    fn collect_ca_input_rewrites(
        &self,
        derivation: &Derivation,
        known_paths: &KnownPaths,
    ) -> Vec<(String, String)> {
        let mut rewrites: Vec<(String, String)> = Vec::new();
        for (input_drv_path, output_names) in &derivation.input_derivations {
            let input_abs = input_drv_path.to_absolute_path_with_prefix(LOGICAL_STORE_DIR);
            if let Some(entry) = known_paths.get_by_drv_path(&input_abs) {
                if entry.content_addressed {
                    for on in output_names {
                        let placeholder = entry.derivation.environment
                            .get(on)
                            .map(|v| String::from_utf8_lossy(v).to_string())
                            .unwrap_or_default();
                        if let Some(resolved) = entry.resolved_outputs.get(on) {
                            // CA rewrites operate in sandbox space (logical prefix).
                            let resolved_abs = resolved.to_absolute_path_with_prefix(LOGICAL_STORE_DIR);
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
        build_request: &snix_build::buildservice::BuildRequest,
        derivation: &Derivation,
        known_paths: &mut KnownPaths,
        is_ca: bool,
    ) -> Result<PathInfo, Error> {
        let nar_renderer = SimpleRenderer::new(
            self.blob_service.clone(),
            self.directory_service.clone(),
        );

        // Apply transitive CA input rewrites.
        let mut working_node = build_output.node.clone();
        for (old_placeholder, new_path) in input_rewrites {
            let (rewritten, _) = crate::rewrite::rewrite_node(
                &working_node,
                old_placeholder.as_bytes(),
                new_path.as_bytes(),
                &self.blob_service,
                &self.directory_service,
            ).await?;
            working_node = rewritten;
        }

        // Determine output store path and final node.
        let (output_path, final_node, nar_size, nar_sha256) = if is_ca {
            self.compute_ca_output(
                drv_path, drv_name, output_name, &working_node,
                derivation, known_paths, &nar_renderer,
            ).await?
        } else {
            let path = output.path.as_ref().ok_or_else(|| Error::OutputNoPath {
                output: output_name.to_string(),
                drv_name: drv_name.to_string(),
            })?.clone();

            let (nar_size, nar_sha256) = nar_renderer
                .calculate_nar(&build_output.node)
                .await
                .map_err(|e| Error::NarCalculation(e.to_string()))?;

            (path, working_node, nar_size, nar_sha256)
        };

        // FOD hash verification (only for FODs, not CA).
        if let Some(ca_hash) = &output.ca_hash {
            verify_fod_hash(
                drv_name, output_name, ca_hash, nar_size, &nar_sha256,
                &final_node, &self.blob_service, &self.directory_service,
            ).await?;
        }

        // Resolve references from refscan needles.
        let references = resolve_references(
            &build_output.output_needles,
            &build_request.refscan_needles,
            derivation,
            sandbox_inputs,
        );

        let ca_field = if is_ca {
            Some(nix_compat::nixhash::CAHash::Nar(
                nix_compat::nixhash::NixHash::Sha256(nar_sha256),
            ))
        } else {
            output.ca_hash.clone()
        };

        // Persist and export.
        self.persist_and_export_output(
            drv_path, &output_path, final_node, references,
            nar_size, nar_sha256, ca_field,
        ).await
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
        known_paths: &mut KnownPaths,
        nar_renderer: &SimpleRenderer<BS, DS>,
    ) -> Result<(StorePath<String>, Node, u64, [u8; 32]), Error> {
        // 1. Get the provisional (input-addressed) path from the env.
        let provisional = derivation.environment
            .get(output_name)
            .map(|v| String::from_utf8_lossy(v).to_string())
            .unwrap_or_default();
        let provisional_bytes = provisional.as_bytes();
        let marker = vec![0u8; provisional_bytes.len()];

        // 2. Replace provisional with zero marker.
        let (marked_node, _has_self_refs) = crate::rewrite::rewrite_node(
            working_node,
            provisional_bytes,
            &marker,
            &self.blob_service,
            &self.directory_service,
        ).await?;

        // 3. Compute NAR hash of marker-replaced content (canonical form).
        let (marker_nar_size, marker_nar_sha256) = nar_renderer
            .calculate_nar(&marked_node)
            .await
            .map_err(|e| Error::NarCalculation(e.to_string()))?;

        // 4. Compute CA store path from marker-replaced hash.
        let ca_hash = nix_compat::nixhash::CAHash::Nar(
            nix_compat::nixhash::NixHash::Sha256(marker_nar_sha256),
        );
        let base_name = drv_name.strip_suffix(".drv").unwrap_or(drv_name);
        let path_name = if output_name == "out" {
            base_name.to_string()
        } else {
            format!("{base_name}-{output_name}")
        };
        let ca_path: StorePath<String> = nix_compat::store_path::build_ca_path_with_store_dir(
            &path_name,
            &ca_hash,
            Vec::<&str>::new(),
            false,
            LOGICAL_STORE_DIR,
        )
        .map_err(|e| Error::Store(format!("computing CA path: {e}")))?;

        // 5. Replace zero markers with the final CA path (in sandbox/logical space).
        let final_abs = ca_path.to_absolute_path_with_prefix(LOGICAL_STORE_DIR);
        let final_bytes = final_abs.as_bytes();
        let final_node = if marker.len() == final_bytes.len() {
            let (node, _) = crate::rewrite::rewrite_node(
                &marked_node,
                &marker,
                final_bytes,
                &self.blob_service,
                &self.directory_service,
            ).await?;
            node
        } else {
            marked_node
        };

        // Register the resolved path.
        let drv_abs = drv_path.to_absolute_path_with_prefix(LOGICAL_STORE_DIR);
        known_paths.resolve_output(&drv_abs, output_name, ca_path.clone());

        // Persist CA mapping for cache across restarts.
        self.ca_mappings.insert(&drv_abs, output_name, &final_abs);
        if let Some(ref sd) = self.state_dir {
            self.ca_mappings.save(sd);
        }

        let display_abs = ca_path.to_absolute_path_with_prefix(&self.output_dir_str);
        info!(
            drv = %drv_name,
            output = %output_name,
            ca_path = %display_abs,
            "CA output path resolved"
        );

        Ok((ca_path, final_node, marker_nar_size, marker_nar_sha256))
    }

    /// Create PathInfo, persist to PathInfoService, and export to disk.
    async fn persist_and_export_output(
        &mut self,
        drv_path: &StorePath<String>,
        output_path: &StorePath<String>,
        final_node: Node,
        references: Vec<StorePath<String>>,
        nar_size: u64,
        nar_sha256: [u8; 32],
        ca: Option<nix_compat::nixhash::CAHash>,
    ) -> Result<PathInfo, Error> {
        // Tiger Style: assert the NAR hash is not all zeros
        // (would indicate a hashing bug or uninitialized memory).
        debug_assert!(
            nar_sha256 != [0u8; 32],
            "NAR sha256 must not be all zeros for {}",
            output_path.name()
        );

        let path_info = PathInfo {
            store_path: output_path.clone(),
            node: final_node.clone(),
            references,
            nar_size,
            nar_sha256,
            signatures: vec![],
            deriver: Some(drv_path.clone()),
            ca,
        };

        self.pathinfo_service
            .put(path_info.clone())
            .await
            .map_err(|e| Error::Store(format!("persisting PathInfo: {e}")))?;

        // Host filesystem: write outputs to the physical output dir.
        let abs_path = output_path.to_absolute_path_with_prefix(&self.output_dir_str);
        self.built_outputs.insert(abs_path.clone(), path_info.clone());
        self.output_nodes
            .insert(output_path.clone(), final_node.clone());

        if !PathBuf::from(&abs_path).exists() {
            match export_castore_to_disk(
                &final_node, &abs_path,
                &self.blob_service, &self.directory_service,
            ).await {
                Ok(()) => {}
                Err(e) if e.contains("Read-only file system")
                       || e.contains("Permission denied") => {
                    debug!(
                        path = %abs_path,
                        "store dir not writable, output stays in castore only"
                    );
                }
                Err(e) => {
                    return Err(Error::Store(format!(
                        "exporting output {abs_path} to disk: {e}"
                    )));
                }
            }
        }

        Ok(path_info)
    }

    /// Execute a builtin fetcher derivation (e.g., `builtin:fetchurl`).
    ///
    /// Bypasses the sandbox entirely: parses the derivation environment
    /// into a `Fetch`, downloads the resource, verifies the hash, then
    /// runs the standard post-build pipeline (ingest, NAR hash, PathInfo).
    async fn build_fetcher(
        &mut self,
        drv_path: &StorePath<String>,
        derivation: &Derivation,
        _known_paths: &mut KnownPaths,
    ) -> Result<BuildOutcome, Error> {
        let drv_name = drv_path.name().to_string();

        let fetch = crate::fetcher::parse_fetch(derivation).map_err(|e| {
            Error::BuildFailed {
                name: drv_name.clone(),
                exit_code: "parse".to_string(),
                log: e.to_string(),
            }
        })?;

        let out_output = derivation.outputs.get("out").ok_or_else(|| {
            Error::OutputMissing {
                output: "out".to_string(),
            }
        })?;
        let out_path = out_output.path.as_ref().ok_or_else(|| Error::OutputNoPath {
            output: "out".to_string(),
            drv_name: drv_name.clone(),
        })?;
        // Fetcher outputs land on disk at the physical output dir.
        let out_abs = out_path.to_absolute_path_with_prefix(&self.output_dir_str);

        info!(drv = %drv_name, "fetching");
        let fetch_clone = fetch.clone();
        let out_abs_clone = out_abs.clone();
        tokio::task::spawn_blocking(move || {
            crate::fetcher::fetch_to_store(&fetch_clone, &out_abs_clone)
        })
        .await
        .map_err(|e| Error::Sandbox(std::io::Error::other(format!("spawn_blocking: {e}"))))?
        .map_err(|e| Error::BuildFailed {
            name: drv_name.clone(),
            exit_code: "fetch".to_string(),
            log: e.to_string(),
        })?;

        if !PathBuf::from(&out_abs).exists() {
            return Err(Error::BuildFailed {
                name: drv_name.clone(),
                exit_code: "fetch".to_string(),
                log: format!("fetcher did not produce output at {out_abs}"),
            });
        }

        // Verify flat hash for URL fetches (before ingest).
        if let crate::fetcher::Fetch::Url { exp_hash: Some(ref expected), .. } = fetch {
            crate::fetcher::verify_flat_hash(&out_abs, expected, &drv_name).map_err(
                |e| match e {
                    crate::fetcher::FetchError::HashMismatch {
                        name,
                        expected,
                        actual,
                    } => Error::FodHashMismatch {
                        name,
                        expected_sri: expected,
                        actual_sri: actual,
                    },
                    other => Error::BuildFailed {
                        name: drv_name.clone(),
                        exit_code: "verify".to_string(),
                        log: other.to_string(),
                    },
                },
            )?;
        }

        // Ingest output into castore.
        let node = ingest_path::<_, _, _, &[u8]>(
            self.blob_service.clone(),
            self.directory_service.clone(),
            PathBuf::from(&out_abs).as_path(),
            None,
        )
        .await
        .map_err(|e| {
            Error::Sandbox(std::io::Error::other(format!(
                "failed to ingest fetcher output {out_abs}: {e}"
            )))
        })?;

        // Compute NAR hash.
        let nar_renderer = SimpleRenderer::new(
            self.blob_service.clone(),
            self.directory_service.clone(),
        );
        let (nar_size, nar_sha256) = nar_renderer
            .calculate_nar(&node)
            .await
            .map_err(|e| Error::NarCalculation(e.to_string()))?;

        // Verify FOD hash (NAR-based, for tarballs/git/NAR fetches).
        if let Some(ca_hash) = &out_output.ca_hash {
            verify_fod_hash(
                &drv_name, "out", ca_hash, nar_size, &nar_sha256,
                &node, &self.blob_service, &self.directory_service,
            ).await?;
        }

        // Persist via the shared helper (no duplication).
        let path_info = self
            .persist_and_export_output(
                drv_path,
                out_path,
                node,
                vec![],  // fetcher outputs have no references
                nar_size,
                nar_sha256,
                out_output.ca_hash.clone(),
            )
            .await?;

        let mut output_infos = HashMap::new();
        output_infos.insert("out".to_string(), path_info);

        info!(drv = %drv_name, path = %out_abs, "fetch succeeded");

        Ok(BuildOutcome {
            drv_path: drv_path.clone(),
            outputs: output_infos,
            cached: false,
            log: None,
        })
    }

    /// Check cache: every output must have PathInfo AND exist on disk.
    /// For CA derivations, uses ca_mappings to find the resolved path.
    /// Returns Some(outputs) on full cache hit, None on any miss.
    async fn check_cache(
        &mut self,
        drv_path: &StorePath<String>,
        derivation: &Derivation,
    ) -> Result<Option<HashMap<String, PathInfo>>, Error> {
        let mut infos = HashMap::new();
        let drv_abs = drv_path.to_absolute_path_with_prefix(LOGICAL_STORE_DIR);

        for (output_name, output) in &derivation.outputs {
            let output_path: StorePath<String> = match output.path.as_ref() {
                Some(p) => p.clone(),
                None => {
                    match self.ca_mappings.get(&drv_abs, output_name) {
                        Some(ca_abs) => {
                            StorePath::from_absolute_path(ca_abs.as_bytes())
                                .map_err(|_| Error::Store(format!(
                                    "invalid CA mapping path: {ca_abs}"
                                )))?
                        }
                        None => return Ok(None),
                    }
                }
            };

            // Cache check: output must exist on host at the physical output dir.
            let abs = PathBuf::from(output_path.to_absolute_path_with_prefix(&self.output_dir_str));
            let digest = *output_path.digest();

            let stored = self.pathinfo_service.get(digest).await
                .map_err(|e| Error::Store(format!("PathInfo lookup: {e}")))?;

            match (stored, abs.exists()) {
                (Some(path_info), true) => {
                    self.output_nodes.insert(output_path.clone(), path_info.node.clone());
                    self.built_outputs.insert(
                        output_path.to_absolute_path_with_prefix(&self.output_dir_str),
                        path_info.clone(),
                    );
                    infos.insert(output_name.clone(), path_info);
                }
                (Some(_), false) => {
                    tracing::warn!(
                        path = %output_path.to_absolute_path_with_prefix(&self.output_dir_str),
                        "PathInfo exists but output missing from disk, rebuilding"
                    );
                    return Ok(None);
                }
                (None, true) => {
                    debug!(
                        path = %output_path.to_absolute_path_with_prefix(&self.output_dir_str),
                        "output exists on disk but no PathInfo, rebuilding"
                    );
                    return Ok(None);
                }
                (None, false) => {
                    return Ok(None);
                }
            }
        }

        Ok(Some(infos))
    }

    /// Resolve a store path to its host filesystem location.
    /// Source inputs (from input_sources) live at /nix/store/.
    /// Built outputs live at the physical output dir.
    fn resolve_host_path(&self, path: &StorePath<String>, derivation: &Derivation) -> PathBuf {
        let is_source = derivation.input_sources.contains(path);
        if is_source {
            PathBuf::from(path.to_absolute_path())
        } else {
            PathBuf::from(path.to_absolute_path_with_prefix(&self.output_dir_str))
        }
    }

    /// Make sure we have castore nodes for inputs that exist on disk
    /// (needed for passing as sandbox inputs to downstream builds).
    async fn ensure_input_nodes(&mut self, derivation: &Derivation) -> Result<(), Error> {
        for output in derivation.outputs.values() {
            if let Some(path) = &output.path {
                if !self.output_nodes.contains_key(path) {
                    // Built outputs are at the physical output dir.
                    let abs = PathBuf::from(path.to_absolute_path_with_prefix(&self.output_dir_str));
                    if abs.exists() {
                        let node = ingest_path::<_, _, _, &[u8]>(
                            self.blob_service.clone(),
                            self.directory_service.clone(),
                            &abs,
                            None,
                        )
                        .await
                        .map_err(|e| Error::Sandbox(std::io::Error::other(format!(
                            "failed to ingest existing output {}: {e}",
                            path
                        ))))?;
                        self.output_nodes.insert(path.clone(), node);
                    }
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;
    use std::sync::{Arc, Mutex};

    use nix_compat::derivation::Output;
    use nix_compat::store_path::StorePath;
    use snix_build::buildservice::{BuildRequest, BuildResult, BuildOutput};
    use snix_castore::blobservice::MemoryBlobService;
    use snix_castore::blobservice::BlobService;
    use snix_castore::directoryservice::{RedbDirectoryService, RedbDirectoryServiceConfig};
    use snix_store::pathinfoservice::LruPathInfoService;
    use tokio::io::AsyncWriteExt;

    fn tmp_ds() -> RedbDirectoryService {
        RedbDirectoryService::new_temporary(
            "test".to_string(),
            RedbDirectoryServiceConfig::default(),
        ).unwrap()
    }

    fn test_pis() -> LruPathInfoService {
        LruPathInfoService::with_capacity(
            "test".to_string(),
            std::num::NonZeroUsize::new(128).unwrap(),
        )
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
            (Self { calls: calls.clone(), blob_service }, calls)
        }
    }

    #[tonic::async_trait]
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

    /// Build a nix_compat::Derivation, compute paths, register in KnownPaths.
    fn build_and_register(
        name: &str,
        input_drvs: &[(StorePath<String>, &str)],
        kp: &mut crunch_glue::KnownPaths,
    ) -> (StorePath<String>, Derivation) {
        let mut outputs = BTreeMap::new();
        outputs.insert("out".to_string(), nix_compat::derivation::Output {
            path: None, ca_hash: None,
        });
        let mut environment = BTreeMap::new();
        environment.insert("name".to_string(), name.into());
        environment.insert("system".to_string(), "x86_64-linux".into());
        environment.insert("builder".to_string(), "/bin/sh".into());
        environment.insert("out".to_string(), "".into());

        let mut input_derivations = BTreeMap::new();
        for (dp, on) in input_drvs {
            input_derivations
                .entry(dp.clone())
                .or_insert_with(BTreeSet::new)
                .insert(on.to_string());
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
            kp.get_hdm_by_drv_path(&parent_path.to_absolute_path())
                .expect("parent should be in known_paths")
        });
        drv.calculate_output_paths(name, &hdm).unwrap();
        let drv_path = drv.calculate_derivation_path(name).unwrap();

        let mut fake_hash = [0u8; 32];
        for (i, b) in name.bytes().enumerate().take(32) {
            fake_hash[i] = b;
        }
        kp.insert(fake_hash, drv_path.clone(), hdm, drv.clone());

        (drv_path, drv)
    }

    #[tokio::test]
    async fn builder_single_drv_calls_do_build_once() {
        let bs = MemoryBlobService::default();
        let ds = tmp_ds();
        let (mock, calls) = MockBuildService::new(bs.clone());

        let mut builder = Builder::new(
            bs, ds, mock, test_pis(), PathBuf::from("/nix/store"), false,
        );

        let mut kp = crunch_glue::KnownPaths::default();
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
            bs, ds, mock, test_pis(), PathBuf::from("/nix/store"), false,
        );

        let mut kp = crunch_glue::KnownPaths::default();
        let (dep_path, _) = build_and_register("dep", &[], &mut kp);
        let (top_path, _) = build_and_register("top", &[(dep_path.clone(), "out")], &mut kp);

        let outcome = builder.build(&top_path, &mut kp).await.unwrap();
        assert!(!outcome.cached);

        let recorded = calls.lock().unwrap();
        assert_eq!(recorded.len(), 2, "should build dep + top");
        assert!(
            recorded[0].iter().any(|a| a.contains("dep")),
            "first build should be dep: {:?}", recorded[0]
        );
        assert!(
            recorded[1].iter().any(|a| a.contains("top")),
            "second build should be top: {:?}", recorded[1]
        );
    }

    #[tokio::test]
    async fn builder_diamond_builds_shared_dep_once() {
        let bs = MemoryBlobService::default();
        let ds = tmp_ds();
        let (mock, calls) = MockBuildService::new(bs.clone());

        let mut builder = Builder::new(
            bs, ds, mock, test_pis(), PathBuf::from("/nix/store"), false,
        );

        let mut kp = crunch_glue::KnownPaths::default();
        let (a_path, _) = build_and_register("aaa", &[], &mut kp);
        let (b_path, _) = build_and_register("bbb", &[(a_path.clone(), "out")], &mut kp);
        let (c_path, _) = build_and_register("ccc", &[(a_path.clone(), "out")], &mut kp);
        let (d_path, _) = build_and_register(
            "ddd",
            &[(b_path.clone(), "out"), (c_path.clone(), "out")],
            &mut kp,
        );

        let outcome = builder.build(&d_path, &mut kp).await.unwrap();
        assert!(!outcome.cached);

        let recorded = calls.lock().unwrap();
        assert_eq!(recorded.len(), 4, "should build A+B+C+D: {:?}", *recorded);
        let a_builds = recorded.iter()
            .filter(|args| args.iter().any(|a| a.contains("aaa")))
            .count();
        assert_eq!(a_builds, 1, "shared dep A should build exactly once");
    }

    // ── Cache tests ────────────────────────────────────────────

    #[tokio::test]
    async fn builder_skips_build_when_output_exists() {
        let output_tmp = tempfile::tempdir().unwrap();
        let output_dir = output_tmp.path().to_str().unwrap().to_string();

        let bs = MemoryBlobService::default();
        let ds = tmp_ds();
        let (mock, calls) = MockBuildService::new(bs.clone());
        let pis = test_pis();

        // KnownPaths uses /nix/store (logical); output_dir is the
        // physical location where the Builder checks for cached files.
        let mut kp = crunch_glue::KnownPaths::default();
        let (drv_path, drv) = build_and_register("cached-test", &[], &mut kp);

        let out_path = drv.outputs["out"].path.as_ref().unwrap();
        let abs = PathBuf::from(out_path.to_absolute_path_with_prefix(&output_dir));
        std::fs::create_dir_all(abs.parent().unwrap()).unwrap();
        std::fs::write(&abs, "cached content").unwrap();

        let path_info = PathInfo {
            store_path: out_path.clone(),
            node: Node::File {
                digest: snix_castore::B3Digest::from(&[0u8; 32]),
                size: 14,
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
            bs, ds, mock, pis, output_tmp.path().to_path_buf(), false,
        );

        let outcome = builder.build(&drv_path, &mut kp).await.unwrap();
        assert!(outcome.cached, "should report as cached");

        let recorded = calls.lock().unwrap();
        assert_eq!(recorded.len(), 0, "should NOT call do_build for cached output");
    }

    #[tokio::test]
    async fn cache_miss_when_pathinfo_but_no_file() {
        let bs = MemoryBlobService::default();
        let ds = tmp_ds();
        let (mock, calls) = MockBuildService::new(bs.clone());
        let pis = test_pis();

        let mut kp = crunch_glue::KnownPaths::default();
        let (drv_path, drv) = build_and_register("orphan", &[], &mut kp);

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
            bs, ds, mock, pis, PathBuf::from("/nix/store"), false,
        );

        let outcome = builder.build(&drv_path, &mut kp).await.unwrap();
        assert!(!outcome.cached, "should NOT be cached (file missing)");

        let recorded = calls.lock().unwrap();
        assert_eq!(recorded.len(), 1, "should call do_build");
    }

    #[tokio::test]
    async fn cache_miss_when_file_but_no_pathinfo() {
        let output_tmp = tempfile::tempdir().unwrap();
        let output_dir = output_tmp.path().to_str().unwrap().to_string();

        let bs = MemoryBlobService::default();
        let ds = tmp_ds();
        let (mock, calls) = MockBuildService::new(bs.clone());
        let pis = test_pis();

        let mut kp = crunch_glue::KnownPaths::default();
        let (drv_path, drv) = build_and_register("untracked", &[], &mut kp);

        let out_path = drv.outputs["out"].path.as_ref().unwrap();
        let abs = PathBuf::from(out_path.to_absolute_path_with_prefix(&output_dir));
        std::fs::create_dir_all(abs.parent().unwrap()).unwrap();
        std::fs::write(&abs, "untracked content").unwrap();

        let mut builder = Builder::new(
            bs, ds, mock, pis, output_tmp.path().to_path_buf(), false,
        );

        let outcome = builder.build(&drv_path, &mut kp).await.unwrap();
        assert!(!outcome.cached, "should NOT be cached (no PathInfo)");

        let recorded = calls.lock().unwrap();
        assert_eq!(recorded.len(), 1, "should call do_build");
    }

    // ── CA derivation tests ──────────────────────────────────────

    fn build_and_register_ca(
        name: &str,
        kp: &mut crunch_glue::KnownPaths,
    ) -> (StorePath<String>, Derivation) {
        let mut outputs = BTreeMap::new();
        outputs.insert("out".to_string(), nix_compat::derivation::Output {
            path: None, ca_hash: None,
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
        kp.insert_ca(fake_hash, drv_path.clone(), hdm, drv.clone(), true);

        (drv_path, drv)
    }

    #[tokio::test]
    async fn ca_derivation_gets_content_based_path() {
        let bs = MemoryBlobService::default();
        let ds = tmp_ds();
        let (mock, _calls) = MockBuildService::new(bs.clone());

        let mut builder = Builder::new(
            bs, ds, mock, test_pis(), PathBuf::from("/nix/store"), false,
        );

        let mut kp = crunch_glue::KnownPaths::default();
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
            bs.clone(), ds.clone(), mock1, test_pis(), PathBuf::from("/nix/store"), false,
        );
        let mut kp1 = crunch_glue::KnownPaths::default();
        let (drv_path1, _) = build_and_register_ca("ca-a", &mut kp1);
        let outcome1 = builder1.build(&drv_path1, &mut kp1).await.unwrap();

        let (mock2, _) = MockBuildService::new(bs.clone());
        let mut builder2 = Builder::new(
            bs.clone(), ds.clone(), mock2, test_pis(), PathBuf::from("/nix/store"), false,
        );
        let mut kp2 = crunch_glue::KnownPaths::default();
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
            bs.clone(), ds.clone(), mock1, test_pis(), PathBuf::from("/nix/store"), false,
        );
        let mut kp1 = crunch_glue::KnownPaths::default();
        let (drv_path1, _) = build_and_register_ca("ca-same", &mut kp1);
        let outcome1 = builder1.build(&drv_path1, &mut kp1).await.unwrap();

        let (mock2, _) = MockBuildService::new(bs.clone());
        let mut builder2 = Builder::new(
            bs.clone(), ds.clone(), mock2, test_pis(), PathBuf::from("/nix/store"), false,
        );
        let mut kp2 = crunch_glue::KnownPaths::default();
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
        // KnownPaths uses /nix/store (default/logical).
        // Verifies output lands in the custom dir, not /nix/store.
        let output_tmp = tempfile::tempdir().unwrap();

        let bs = MemoryBlobService::default();
        let ds = tmp_ds();
        let (mock, _calls) = MockBuildService::new(bs.clone());

        let mut builder = Builder::new(
            bs, ds, mock, test_pis(),
            output_tmp.path().to_path_buf(), false,
        );

        let mut kp = crunch_glue::KnownPaths::default();
        let (drv_path, _) = build_and_register("custom-dir-test", &[], &mut kp);

        let outcome = builder.build(&drv_path, &mut kp).await.unwrap();
        assert!(!outcome.cached);

        // Output should exist under the custom output dir.
        let out_info = &outcome.outputs["out"];
        let custom_abs = out_info.store_path
            .to_absolute_path_with_prefix(output_tmp.path().to_str().unwrap());
        assert!(
            PathBuf::from(&custom_abs).exists(),
            "output should exist at custom dir: {custom_abs}"
        );

        // And should NOT exist at the logical /nix/store path.
        let logical_abs = out_info.store_path.to_absolute_path();
        // (Only check if /nix/store is not the output_dir, which it
        // isn't since we used a temp dir.)
        assert_ne!(
            output_tmp.path().to_str().unwrap(), "/nix/store",
            "test requires output_dir != /nix/store"
        );
        assert!(
            !PathBuf::from(&logical_abs).exists()
                || logical_abs == custom_abs,
            "output should NOT exist at /nix/store: {logical_abs}"
        );
    }

    #[tokio::test]
    async fn custom_output_dir_cache_hit() {
        // Pre-populate output in a custom dir, verify cache hit.
        let output_tmp = tempfile::tempdir().unwrap();
        let output_dir = output_tmp.path().to_str().unwrap().to_string();

        let bs = MemoryBlobService::default();
        let ds = tmp_ds();
        let (mock, calls) = MockBuildService::new(bs.clone());
        let pis = test_pis();

        let mut kp = crunch_glue::KnownPaths::default();
        let (drv_path, drv) = build_and_register("cached-custom", &[], &mut kp);

        // Write the output file at output_dir, NOT /nix/store.
        let out_path = drv.outputs["out"].path.as_ref().unwrap();
        let abs = PathBuf::from(out_path.to_absolute_path_with_prefix(&output_dir));
        std::fs::create_dir_all(abs.parent().unwrap()).unwrap();
        std::fs::write(&abs, "cached").unwrap();

        // Also insert PathInfo.
        let path_info = PathInfo {
            store_path: out_path.clone(),
            node: Node::File {
                digest: snix_castore::B3Digest::from(&[0u8; 32]),
                size: 6,
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
            bs, ds, mock, pis, output_tmp.path().to_path_buf(), false,
        );

        let outcome = builder.build(&drv_path, &mut kp).await.unwrap();
        assert!(outcome.cached, "should be cached from custom output dir");

        let recorded = calls.lock().unwrap();
        assert_eq!(recorded.len(), 0, "no build needed for cached output");
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
            bs, ds, mock, test_pis(), PathBuf::from("/nix/store"), false,
        );

        let mut kp = crunch_glue::KnownPaths::default();
        let (a_path, _) = build_and_register("shared", &[], &mut kp);
        let (b_path, _) = build_and_register("root-b", &[(a_path.clone(), "out")], &mut kp);
        let (c_path, _) = build_and_register("root-c", &[(a_path.clone(), "out")], &mut kp);

        let outcomes = builder
            .build_all(&[b_path.clone(), c_path.clone()], &mut kp, 2)
            .await
            .unwrap();

        // Should return outcomes for roots only.
        assert_eq!(outcomes.len(), 2, "should return 2 root outcomes");
        let outcome_drv_names: Vec<String> = outcomes.iter()
            .map(|o| o.drv_path.name().to_string())
            .collect();
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
        let shared_builds = recorded.iter()
            .filter(|args| args.iter().any(|a| a.contains("shared")))
            .count();
        assert_eq!(shared_builds, 1, "shared dep should build once");
    }

    #[tokio::test]
    async fn build_all_empty_roots() {
        let bs = MemoryBlobService::default();
        let ds = tmp_ds();
        let (mock, _calls) = MockBuildService::new(bs.clone());

        let mut builder = Builder::new(
            bs, ds, mock, test_pis(), PathBuf::from("/nix/store"), false,
        );

        let mut kp = crunch_glue::KnownPaths::default();
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
            bs, ds, mock, test_pis(), PathBuf::from("/nix/store"), false,
        );

        let mut kp = crunch_glue::KnownPaths::default();
        let (a_path, _) = build_and_register("leaf-a", &[], &mut kp);
        let (b_path, _) = build_and_register("mid-b", &[(a_path.clone(), "out")], &mut kp);
        let (c_path, _) = build_and_register("top-c", &[(b_path.clone(), "out")], &mut kp);

        let outcomes = builder
            .build_all(&[c_path.clone()], &mut kp, 2)
            .await
            .unwrap();

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
            bs, ds, mock, test_pis(), PathBuf::from("/nix/store"), false,
        );

        let mut kp = crunch_glue::KnownPaths::default();
        let (a_path, _) = build_and_register("tree1-leaf", &[], &mut kp);
        let (b_path, _) = build_and_register("tree1-root", &[(a_path.clone(), "out")], &mut kp);
        let (c_path, _) = build_and_register("tree2-leaf", &[], &mut kp);
        let (d_path, _) = build_and_register("tree2-root", &[(c_path.clone(), "out")], &mut kp);

        let outcomes = builder
            .build_all(&[b_path.clone(), d_path.clone()], &mut kp, 2)
            .await
            .unwrap();

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
            bs, ds, mock, test_pis(), PathBuf::from("/nix/store"), false,
        );

        let mut kp = crunch_glue::KnownPaths::default();
        let (drv_path, drv) = build_and_register_multi(
            "multi-out", &["out", "dev", "lib"], &[], &mut kp,
        );

        let outcome = builder.build(&drv_path, &mut kp).await.unwrap();
        assert!(!outcome.cached);

        // Should have PathInfo for all 3 outputs.
        assert_eq!(outcome.outputs.len(), 3, "expected 3 outputs: {:?}", outcome.outputs.keys().collect::<Vec<_>>());
        assert!(outcome.outputs.contains_key("out"));
        assert!(outcome.outputs.contains_key("dev"));
        assert!(outcome.outputs.contains_key("lib"));

        // Each output should have a distinct store path.
        let paths: Vec<String> = outcome.outputs.values()
            .map(|pi| pi.store_path.to_absolute_path())
            .collect();
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

        let mut kp = crunch_glue::KnownPaths::default();
        let (_drv_path, drv) = build_and_register_multi(
            "env-test", &["out", "dev", "man"], &[], &mut kp,
        );

        // The outputs env var should list all output names.
        let outputs_env = drv.environment.get("outputs").unwrap();
        let outputs_str = std::str::from_utf8(outputs_env.as_ref()).unwrap();
        assert_eq!(outputs_str, "out dev man");

        // Each output name should have a store path env var.
        for name in &["out", "dev", "man"] {
            let val = drv.environment.get(*name).unwrap();
            let s = std::str::from_utf8(val.as_ref()).unwrap();
            assert!(
                s.starts_with("/nix/store/"),
                "${name} should be a store path: {s}"
            );
        }
    }

    #[tokio::test]
    async fn multi_output_distinct_store_paths() {
        use crate::test_support::build_and_register_multi;

        let mut kp = crunch_glue::KnownPaths::default();
        let (_drv_path, drv) = build_and_register_multi(
            "paths-test", &["out", "dev", "lib"], &[], &mut kp,
        );

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
            bs, ds, mock, test_pis(), PathBuf::from("/nix/store"), false,
        );

        let mut kp = crunch_glue::KnownPaths::default();
        let (dep_path, dep_drv) = build_and_register_multi(
            "multi-dep", &["out", "dev"], &[], &mut kp,
        );

        // Parent depends on both "out" and "dev" of the dep.
        let mut outputs = BTreeMap::new();
        outputs.insert("out".to_string(), Output { path: None, ca_hash: None });
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
            kp.get_hdm_by_drv_path(&parent_path.to_absolute_path())
                .expect("parent should be in known_paths")
        });
        parent_drv.calculate_output_paths("consumer", &hdm).unwrap();
        let parent_drv_path = parent_drv.calculate_derivation_path("consumer").unwrap();

        let mut fake_hash = [0u8; 32];
        for (i, b) in "consumer".bytes().enumerate().take(32) {
            fake_hash[i] = b;
        }
        kp.insert(fake_hash, parent_drv_path.clone(), hdm, parent_drv.clone());

        let outcome = builder.build(&parent_drv_path, &mut kp).await.unwrap();
        assert!(!outcome.cached);

        // Should have built the dep and the consumer (2 builds total).
        let recorded = calls.lock().unwrap();
        assert_eq!(recorded.len(), 2, "dep + consumer: {:?}", *recorded);
    }

    #[tokio::test]
    async fn multi_output_cache_requires_all_outputs() {
        // If only some outputs are cached, it should be a cache miss.
        use crate::test_support::build_and_register_multi;

        let output_tmp = tempfile::tempdir().unwrap();
        let output_dir = output_tmp.path().to_str().unwrap().to_string();

        let bs = MemoryBlobService::default();
        let ds = tmp_ds();
        let (mock, calls) = MockBuildService::new(bs.clone());
        let pis = test_pis();

        let mut kp = crunch_glue::KnownPaths::default();
        let (drv_path, drv) = build_and_register_multi(
            "partial-cache", &["out", "dev"], &[], &mut kp,
        );

        // Write only the "out" output to disk + PathInfo. "dev" is missing.
        let out_path = drv.outputs["out"].path.as_ref().unwrap();
        let abs = PathBuf::from(out_path.to_absolute_path_with_prefix(&output_dir));
        std::fs::create_dir_all(abs.parent().unwrap()).unwrap();
        std::fs::write(&abs, "cached out").unwrap();

        let path_info = PathInfo {
            store_path: out_path.clone(),
            node: Node::File {
                digest: snix_castore::B3Digest::from(&[0u8; 32]),
                size: 10,
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
            bs, ds, mock, pis, output_tmp.path().to_path_buf(), false,
        );

        let outcome = builder.build(&drv_path, &mut kp).await.unwrap();
        // Cache miss because "dev" is missing.
        assert!(!outcome.cached, "partial cache should not count as hit");

        let recorded = calls.lock().unwrap();
        assert_eq!(recorded.len(), 1, "should rebuild");
    }
}
