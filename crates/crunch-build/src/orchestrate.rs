//! Build orchestration: recursively build derivations, check cache,
//! persist outputs.

use std::collections::{BTreeMap, HashMap};
use std::path::PathBuf;

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

/// Maximum recursive build depth. Prevents stack overflow from cyclic
/// or pathologically deep dependency graphs.
const MAX_BUILD_DEPTH: u32 = 256;

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

/// Orchestrates the build pipeline: evaluating dependencies, checking
/// cache, running builds, persisting results.
pub struct Builder<BS, DS, BServ, PIS> {
    blob_service: BS,
    directory_service: DS,
    build_service: BServ,
    pathinfo_service: PIS,
    store_dir: PathBuf,
    /// The store dir as a string, for path serialization.
    store_dir_str: String,
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
    BServ: BuildService,
    PIS: PathInfoService,
{
    pub fn new(
        blob_service: BS,
        directory_service: DS,
        build_service: BServ,
        pathinfo_service: PIS,
        store_dir: PathBuf,
        verbose: bool,
    ) -> Self {
        Self::with_state_dir(blob_service, directory_service, build_service,
            pathinfo_service, store_dir, None, verbose)
    }

    /// Create a Builder with a state directory for persistent CA mappings.
    pub fn with_state_dir(
        blob_service: BS,
        directory_service: DS,
        build_service: BServ,
        pathinfo_service: PIS,
        store_dir: PathBuf,
        state_dir: Option<PathBuf>,
        verbose: bool,
    ) -> Self {
        let store_dir_str = store_dir.to_str().unwrap_or("/nix/store").to_string();
        let ca_mappings = state_dir
            .as_ref()
            .map(|d| CaMappings::load(d))
            .unwrap_or_default();
        Self {
            blob_service,
            directory_service,
            build_service,
            pathinfo_service,
            store_dir,
            store_dir_str,
            built_outputs: HashMap::new(),
            output_nodes: HashMap::new(),
            ca_mappings,
            state_dir,
            verbose,
        }
    }

    /// Build a derivation and all its dependencies. Returns the outcome.
    ///
    /// `known_paths` must contain the derivation and all its transitive
    /// input derivations (populated by crunch-glue's `convert()`).
    pub async fn build(
        &mut self,
        drv_path: &StorePath<String>,
        known_paths: &mut KnownPaths,
    ) -> Result<BuildOutcome, Error> {
        let entry = known_paths
            .get_by_drv_path(&drv_path.to_absolute_path_with_prefix(&self.store_dir_str))
            .ok_or_else(|| Error::DerivationNotFound {
                path: drv_path.clone(),
            })?;
        let derivation = entry.derivation.clone();

        self.build_derivation_at_depth(drv_path, &derivation, known_paths, 0)
            .await
    }

    /// Recursive build implementation.
    ///
    /// Boxed because it's a recursive async fn — Rust can't compute the
    /// layout of the future without indirection.
    fn build_derivation_at_depth<'a>(
        &'a mut self,
        drv_path: &'a StorePath<String>,
        derivation: &'a Derivation,
        known_paths: &'a mut KnownPaths,
        depth: u32,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<BuildOutcome, Error>> + 'a>> {
        Box::pin(self.build_derivation_inner(drv_path, derivation, known_paths, depth))
    }

    /// Core build logic. Phases:
    /// 1. Check cache
    /// 2. Handle builtin fetchers
    /// 3. Build input derivations recursively
    /// 4. Resolve source inputs and Nix closures
    /// 5. Collect sandbox inputs
    /// 6. Execute the sandbox build
    /// 7. Process each output (rewrite, hash, persist, export)
    async fn build_derivation_inner(
        &mut self,
        drv_path: &StorePath<String>,
        derivation: &Derivation,
        known_paths: &mut KnownPaths,
        depth: u32,
    ) -> Result<BuildOutcome, Error> {
        // Tiger Style: fixed limit on recursion depth.
        if depth >= MAX_BUILD_DEPTH {
            return Err(Error::Store(format!(
                "build depth limit ({MAX_BUILD_DEPTH}) exceeded at {}",
                drv_path.name()
            )));
        }

        let drv_name = drv_path.name().to_string();

        // 1. Cache check
        if let Some(cached_outputs) = self.check_cache(drv_path, derivation).await? {
            info!(drv = %drv_name, "all outputs cached, skipping build");
            return Ok(BuildOutcome {
                drv_path: drv_path.clone(),
                outputs: cached_outputs,
                cached: true,
                log: None,
            });
        }

        // 2. Builtin fetcher bypass
        if crate::fetcher::is_builtin_fetcher(derivation) {
            return self
                .build_fetcher(drv_path, derivation, known_paths)
                .await;
        }

        // 3. Build input derivations
        self.build_input_derivations(derivation, known_paths, depth).await?;

        // 4. Resolve source inputs + closures, ingest into castore
        let all_source_paths = self.resolve_and_ingest_sources(derivation).await?;

        // 5. Collect sandbox inputs
        let sandbox_inputs = self
            .collect_sandbox_inputs(derivation, known_paths, &all_source_paths)
            .await?;

        // 6. Build
        let build_request = derivation_to_build_request(derivation, &sandbox_inputs, &self.store_dir_str)?;

        info!(drv = %drv_name, "building");
        if self.verbose {
            debug!(
                drv = %drv_name,
                inputs = sandbox_inputs.len(),
                outputs = build_request.outputs.len(),
                "starting sandbox build"
            );
        }

        let input_rewrites = self.collect_ca_input_rewrites(derivation, known_paths);

        let build_result = self
            .build_service
            .do_build(build_request.clone())
            .await
            .map_err(|e| Error::BuildFailed {
                name: drv_name.clone(),
                exit_code: "unknown".to_string(),
                log: e.to_string(),
            })?;

        // 7. Process outputs
        let is_ca = derivation.outputs.values()
            .all(|o| o.path.is_none() && o.ca_hash.is_none());

        let mut output_infos: HashMap<String, PathInfo> = HashMap::new();
        let output_names: Vec<String> = derivation.outputs.keys().cloned().collect();

        for (i, (output_name, output)) in derivation.outputs.iter().enumerate() {
            let build_output = build_result.outputs.get(i).ok_or_else(|| {
                Error::OutputMissing {
                    output: output_name.clone(),
                }
            })?;

            let path_info = self
                .process_output(
                    drv_path,
                    &drv_name,
                    output_name,
                    output,
                    build_output,
                    &input_rewrites,
                    &sandbox_inputs,
                    &build_request,
                    derivation,
                    known_paths,
                    is_ca,
                )
                .await?;

            output_infos.insert(output_name.clone(), path_info);
        }

        info!(
            drv = %drv_name,
            outputs = ?output_names.iter()
                .filter_map(|n| derivation.outputs.get(n)?.path.as_ref())
                .map(|p| p.to_absolute_path_with_prefix(&self.store_dir_str))
                .collect::<Vec<_>>(),
            "build succeeded"
        );

        Ok(BuildOutcome {
            drv_path: drv_path.clone(),
            outputs: output_infos,
            cached: false,
            log: build_result.log,
        })
    }

    /// Recursively build all input derivations. Skips inputs whose
    /// outputs already exist (built earlier this session or on disk).
    async fn build_input_derivations(
        &mut self,
        derivation: &Derivation,
        known_paths: &mut KnownPaths,
        depth: u32,
    ) -> Result<(), Error> {
        for (input_drv_path, _output_names) in &derivation.input_derivations {
            let input_entry = known_paths
                .get_by_drv_path(&input_drv_path.to_absolute_path_with_prefix(&self.store_dir_str))
                .ok_or_else(|| Error::DerivationNotFound {
                    path: input_drv_path.clone(),
                })?;
            let input_drv = input_entry.derivation.clone();

            let input_abs = input_drv_path.to_absolute_path_with_prefix(&self.store_dir_str);
            let already_built = input_drv.outputs.keys().all(|output_name| {
                if let Some(resolved) = known_paths.get_output_path(&input_abs, output_name) {
                    self.output_nodes.contains_key(&resolved) || self.path_exists_on_disk(&resolved)
                } else {
                    false
                }
            });

            if !already_built {
                self.build_derivation_at_depth(
                    input_drv_path, &input_drv, known_paths, depth.saturating_add(1),
                ).await?;
            } else {
                self.ensure_input_nodes(&input_drv).await?;
            }
        }
        Ok(())
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

        // Resolve closures for each declared source input.
        for source_path in &derivation.input_sources {
            let abs = source_path.to_absolute_path_with_prefix(&self.store_dir_str);
            if !PathBuf::from(&abs).exists() {
                return Err(Error::SourceNotFound {
                    path: source_path.clone(),
                });
            }
            for closure_abs in resolve_nix_closure(&abs) {
                if let Some(sp) = parse_store_path(&closure_abs, &self.store_dir_str) {
                    if !all_source_paths.contains(&sp) {
                        all_source_paths.push(sp);
                    }
                }
            }
        }

        // Ingest all source paths (declared + closure) into castore.
        for source_path in &all_source_paths {
            let abs = PathBuf::from(source_path.to_absolute_path_with_prefix(&self.store_dir_str));
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
                let abs = PathBuf::from(input_path.to_absolute_path_with_prefix(&self.store_dir_str));
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
            let input_abs = input_drv_path.to_absolute_path_with_prefix(&self.store_dir_str);
            if let Some(entry) = known_paths.get_by_drv_path(&input_abs) {
                if entry.content_addressed {
                    for on in output_names {
                        let placeholder = entry.derivation.environment
                            .get(on)
                            .map(|v| String::from_utf8_lossy(v).to_string())
                            .unwrap_or_default();
                        if let Some(resolved) = entry.resolved_outputs.get(on) {
                            let resolved_abs = resolved.to_absolute_path_with_prefix(&self.store_dir_str);
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
            &self.store_dir_str,
        )
        .map_err(|e| Error::Store(format!("computing CA path: {e}")))?;

        // 5. Replace zero markers with the final CA path.
        let final_abs = ca_path.to_absolute_path_with_prefix(&self.store_dir_str);
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
        let drv_abs = drv_path.to_absolute_path_with_prefix(&self.store_dir_str);
        known_paths.resolve_output(&drv_abs, output_name, ca_path.clone());

        // Persist CA mapping for cache across restarts.
        self.ca_mappings.insert(&drv_abs, output_name, &final_abs);
        if let Some(ref sd) = self.state_dir {
            self.ca_mappings.save(sd);
        }

        info!(
            drv = %drv_name,
            output = %output_name,
            ca_path = %final_abs,
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

        let abs_path = output_path.to_absolute_path_with_prefix(&self.store_dir_str);
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
        let out_abs = out_path.to_absolute_path_with_prefix(&self.store_dir_str);

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
        let drv_abs = drv_path.to_absolute_path_with_prefix(&self.store_dir_str);

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

            let abs = PathBuf::from(output_path.to_absolute_path_with_prefix(&self.store_dir_str));
            let digest = *output_path.digest();

            let stored = self.pathinfo_service.get(digest).await
                .map_err(|e| Error::Store(format!("PathInfo lookup: {e}")))?;

            match (stored, abs.exists()) {
                (Some(path_info), true) => {
                    self.output_nodes.insert(output_path.clone(), path_info.node.clone());
                    self.built_outputs.insert(
                        output_path.to_absolute_path_with_prefix(&self.store_dir_str),
                        path_info.clone(),
                    );
                    infos.insert(output_name.clone(), path_info);
                }
                (Some(_), false) => {
                    tracing::warn!(
                        path = %output_path.to_absolute_path_with_prefix(&self.store_dir_str),
                        "PathInfo exists but output missing from disk, rebuilding"
                    );
                    return Ok(None);
                }
                (None, true) => {
                    debug!(
                        path = %output_path.to_absolute_path_with_prefix(&self.store_dir_str),
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

    /// Check if a store path exists on the filesystem.
    fn path_exists_on_disk(&self, path: &StorePath<String>) -> bool {
        let abs = PathBuf::from(path.to_absolute_path_with_prefix(&self.store_dir_str));
        abs.exists()
    }

    /// Make sure we have castore nodes for inputs that exist on disk
    /// (needed for passing as sandbox inputs to downstream builds).
    async fn ensure_input_nodes(&mut self, derivation: &Derivation) -> Result<(), Error> {
        for output in derivation.outputs.values() {
            if let Some(path) = &output.path {
                if !self.output_nodes.contains_key(path) {
                    let abs = PathBuf::from(path.to_absolute_path_with_prefix(&self.store_dir_str));
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
        let tmp = tempfile::tempdir().unwrap();
        let store_dir = tmp.path().to_str().unwrap().to_string();

        let bs = MemoryBlobService::default();
        let ds = tmp_ds();
        let (mock, calls) = MockBuildService::new(bs.clone());
        let pis = test_pis();

        let mut kp = crunch_glue::KnownPaths::new(&store_dir);
        let (drv_path, drv) = build_and_register("cached-test", &[], &mut kp);

        let out_path = drv.outputs["out"].path.as_ref().unwrap();
        let abs = PathBuf::from(out_path.to_absolute_path_with_prefix(&store_dir));
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
            bs, ds, mock, pis, tmp.path().to_path_buf(), false,
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
        let tmp = tempfile::tempdir().unwrap();
        let store_dir = tmp.path().to_str().unwrap().to_string();

        let bs = MemoryBlobService::default();
        let ds = tmp_ds();
        let (mock, calls) = MockBuildService::new(bs.clone());
        let pis = test_pis();

        let mut kp = crunch_glue::KnownPaths::new(&store_dir);
        let (drv_path, drv) = build_and_register("untracked", &[], &mut kp);

        let out_path = drv.outputs["out"].path.as_ref().unwrap();
        let abs = PathBuf::from(out_path.to_absolute_path_with_prefix(&store_dir));
        std::fs::create_dir_all(abs.parent().unwrap()).unwrap();
        std::fs::write(&abs, "untracked content").unwrap();

        let mut builder = Builder::new(
            bs, ds, mock, pis, tmp.path().to_path_buf(), false,
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
}
