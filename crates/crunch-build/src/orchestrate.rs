//! Build orchestration: recursively build derivations, check cache,
//! persist outputs.

use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::path::PathBuf;

use digest::Digest;
use nix_compat::derivation::Derivation;
use nix_compat::nixhash::{CAHash, NixHash};
use nix_compat::store_path::StorePath;
use snix_build::buildservice::BuildService;
use snix_castore::blobservice::BlobService;
use snix_castore::directoryservice::DirectoryService;
use snix_castore::import::fs::ingest_path;
use snix_castore::Node;
use snix_store::nar::{NarCalculationService, SimpleRenderer, write_nar};
use snix_store::path_info::PathInfo;
use snix_store::utils::AsyncIoBridge;
use tokio::io::AsyncReadExt;
use tracing::{debug, info};

use crunch_glue::KnownPaths;
use snix_store::pathinfoservice::PathInfoService;

use crate::build_request::{collect_input_paths, derivation_to_build_request};
use crate::ca_mapping::CaMappings;
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

        self.build_derivation(drv_path, &derivation, known_paths)
            .await
    }

    /// Recursive build implementation.
    ///
    /// Boxed because it's a recursive async fn — Rust can't compute the
    /// layout of the future without indirection.
    fn build_derivation<'a>(
        &'a mut self,
        drv_path: &'a StorePath<String>,
        derivation: &'a Derivation,
        known_paths: &'a mut KnownPaths,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<BuildOutcome, Error>> + 'a>> {
        Box::pin(self.build_derivation_inner(drv_path, derivation, known_paths))
    }

    async fn build_derivation_inner(
        &mut self,
        drv_path: &StorePath<String>,
        derivation: &Derivation,
        known_paths: &mut KnownPaths,
    ) -> Result<BuildOutcome, Error> {
        let drv_name = drv_path.name().to_string();

        // 1. Check cache: PathInfoService + filesystem
        if let Some(cached_outputs) = self.check_cache(drv_path, derivation).await? {
            info!(drv = %drv_name, "all outputs cached, skipping build");
            return Ok(BuildOutcome {
                drv_path: drv_path.clone(),
                outputs: cached_outputs,
                cached: true,
                log: None,
            });
        }

        // 1b. Builtin fetcher bypass: download instead of sandbox.
        if crate::fetcher::is_builtin_fetcher(derivation) {
            return self
                .build_fetcher(drv_path, derivation, known_paths)
                .await;
        }

        // 2. Recursively build all input derivations
        for (input_drv_path, _output_names) in &derivation.input_derivations {
            let input_entry = known_paths
                .get_by_drv_path(&input_drv_path.to_absolute_path_with_prefix(&self.store_dir_str))
                .ok_or_else(|| Error::DerivationNotFound {
                    path: input_drv_path.clone(),
                })?;
            let input_drv = input_entry.derivation.clone();

            // Skip if we've already built this in the current session.
            // For CA inputs, check resolved_outputs in known_paths.
            let input_abs = input_drv_path.to_absolute_path_with_prefix(&self.store_dir_str);
            let already_built = input_drv.outputs.keys().all(|output_name| {
                if let Some(resolved) = known_paths.get_output_path(&input_abs, output_name) {
                    self.output_nodes.contains_key(&resolved) || self.path_exists_on_disk(&resolved)
                } else {
                    // No resolved path yet — not built
                    false
                }
            });

            if !already_built {
                self.build_derivation(input_drv_path, &input_drv, known_paths)
                    .await?;
            } else {
                // Even if built, make sure we have nodes for inputs that
                // exist on disk but weren't built this session.
                self.ensure_input_nodes(&input_drv).await?;
            }
        }

        // 3. Validate source inputs exist on disk and resolve closures.
        //    For Nix store paths, query the runtime closure so all
        //    transitive dependencies (glibc, etc.) are mounted.
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
            // Query Nix for the runtime closure.
            let closure_paths = resolve_nix_closure(&abs);
            for closure_abs in closure_paths {
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
                // Closure member missing — skip silently (may have been
                // garbage collected). The build will fail if it's actually
                // needed at runtime.
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

        // 4. Collect input nodes for the sandbox (declared + closure)
        let mut input_paths = collect_input_paths(derivation, known_paths)?;
        for sp in &all_source_paths {
            input_paths.insert(sp.clone());
        }
        let mut sandbox_inputs: BTreeMap<StorePath<String>, Node> = BTreeMap::new();
        for input_path in &input_paths {
            if let Some(node) = self.output_nodes.get(input_path) {
                sandbox_inputs.insert(input_path.clone(), node.clone());
            } else {
                // Try ingesting from disk
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

        // 5. Build the BuildRequest
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

        // 5b. Collect CA input rewrite pairs for transitive rewriting.
        // If any input derivation is CA, its placeholder in $env may
        // appear in our output and needs rewriting to the final path.
        let mut input_rewrites: Vec<(String, String)> = Vec::new();
        for (input_drv_path, output_names) in &derivation.input_derivations {
            let input_abs = input_drv_path.to_absolute_path_with_prefix(&self.store_dir_str);
            if let Some(entry) = known_paths.get_by_drv_path(&input_abs) {
                if entry.content_addressed {
                    for on in output_names {
                        let raw = nix_compat::store_path::hash_placeholder(on);
                        let placeholder = format!(
                            "{}/{}",
                            self.store_dir_str,
                            raw.strip_prefix('/').unwrap_or(&raw)
                        );
                        if let Some(resolved) = entry.resolved_outputs.get(on) {
                            let resolved_abs = resolved.to_absolute_path_with_prefix(&self.store_dir_str);
                            if placeholder.len() == resolved_abs.len() {
                                input_rewrites.push((placeholder, resolved_abs));
                            }
                        }
                    }
                }
            }
        }

        // 6. Execute the build
        let build_result = self
            .build_service
            .do_build(build_request.clone())
            .await
            .map_err(|e| {
                // The bwrap builder returns io::Error for failures.
                // Try to extract build log from the error message.
                Error::BuildFailed {
                    name: drv_name.clone(),
                    exit_code: "unknown".to_string(),
                    log: e.to_string(),
                }
            })?;

        // 7. Process outputs: compute NAR hash, find references, create PathInfo
        let nar_renderer = SimpleRenderer::new(
            self.blob_service.clone(),
            self.directory_service.clone(),
        );

        let mut output_infos: HashMap<String, PathInfo> = HashMap::new();
        let output_names: Vec<String> = derivation.outputs.keys().cloned().collect();

        // Determine if this is a CA derivation (output paths are None).
        let is_ca = derivation.outputs.values()
            .all(|o| o.path.is_none() && o.ca_hash.is_none());

        for (i, (output_name, output)) in derivation.outputs.iter().enumerate() {
            let build_output = build_result.outputs.get(i).ok_or_else(|| {
                Error::OutputMissing {
                    output: output_name.clone(),
                }
            })?;

            // Apply transitive CA input rewrites if any.
            let mut working_node = build_output.node.clone();
            for (old_placeholder, new_path) in &input_rewrites {
                let (rewritten, _) = crate::rewrite::rewrite_node(
                    &working_node,
                    old_placeholder.as_bytes(),
                    new_path.as_bytes(),
                    &self.blob_service,
                    &self.directory_service,
                ).await?;
                working_node = rewritten;
            }

            // Determine the output store path and final node.
            let (output_path, final_node, nar_size, nar_sha256) = if is_ca {
                // CA derivation: self-reference rewriting + content-based path.
                //
                // 1. Get the provisional placeholder this output was built with
                let raw_provisional = nix_compat::store_path::hash_placeholder(output_name);
                let provisional = format!(
                    "{}/{}",
                    self.store_dir_str,
                    raw_provisional.strip_prefix('/').unwrap_or(&raw_provisional)
                );
                let provisional_bytes = provisional.as_bytes();
                let marker = vec![0u8; provisional_bytes.len()];

                // 2. Replace provisional with zero marker in the output tree
                let (marked_node, _has_self_refs) = crate::rewrite::rewrite_node(
                    &working_node,
                    provisional_bytes,
                    &marker,
                    &self.blob_service,
                    &self.directory_service,
                ).await?;

                // 3. Compute NAR hash of marker-replaced content (canonical form)
                let (marker_nar_size, marker_nar_sha256) = nar_renderer
                    .calculate_nar(&marked_node)
                    .await
                    .map_err(|e| Error::NarCalculation(e.to_string()))?;

                // 4. Compute CA store path from the marker-replaced hash
                let ca_hash = nix_compat::nixhash::CAHash::Nar(
                    nix_compat::nixhash::NixHash::Sha256(marker_nar_sha256),
                );
                // Use the derivation name without .drv suffix.
                let base_name = drv_name.strip_suffix(".drv").unwrap_or(&drv_name);
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

                // 5. Replace zero markers with the final CA path in the output
                let final_abs = ca_path.to_absolute_path_with_prefix(&self.store_dir_str);
                let final_bytes = final_abs.as_bytes();
                // Marker and final path may differ in length. If so, skip
                // rewriting (the output didn't contain self-references anyway
                // if the provisional wasn't found in step 2).
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
                    // Different lengths — self-refs were already zero-replaced
                    // in the canonical form. Use the marked node as-is for
                    // on-disk content (the zeros will be there, which is the
                    // Nix convention for CA self-references).
                    marked_node
                };

                // Register the resolved path
                let drv_abs = drv_path.to_absolute_path_with_prefix(&self.store_dir_str);
                known_paths.resolve_output(&drv_abs, output_name, ca_path.clone());

                // Persist CA mapping for cache across restarts
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

                // The canonical NAR hash (for PathInfo) is the marker-replaced one.
                (ca_path, final_node, marker_nar_size, marker_nar_sha256)
            } else {
                // Input-addressed or FOD: path was computed at convert() time.
                let path = output.path.as_ref().ok_or_else(|| Error::OutputNoPath {
                    output: output_name.clone(),
                    drv_name: drv_name.clone(),
                })?.clone();

                let (nar_size, nar_sha256) = nar_renderer
                    .calculate_nar(&build_output.node)
                    .await
                    .map_err(|e| Error::NarCalculation(e.to_string()))?;

                (path, working_node, nar_size, nar_sha256)
            };

            // FOD hash verification (only for FODs, not CA)
            if let Some(ca_hash) = &output.ca_hash {
                verify_fod_hash(
                    &drv_name,
                    output_name,
                    ca_hash,
                    nar_size,
                    &nar_sha256,
                    &build_output.node,
                    &self.blob_service,
                    &self.directory_service,
                )
                .await?;
            }

            // Resolve references from refscan needles
            let references = resolve_references(
                &build_output.output_needles,
                &build_request.refscan_needles,
                derivation,
                &sandbox_inputs,
            );

            let ca_field = if is_ca {
                Some(nix_compat::nixhash::CAHash::Nar(
                    nix_compat::nixhash::NixHash::Sha256(nar_sha256),
                ))
            } else {
                output.ca_hash.clone()
            };

            let path_info = PathInfo {
                store_path: output_path.clone(),
                node: final_node.clone(),
                references,
                nar_size,
                nar_sha256,
                signatures: vec![],
                deriver: Some(drv_path.clone()),
                ca: ca_field,
            };

            // Persist to PathInfoService
            self.pathinfo_service
                .put(path_info.clone())
                .await
                .map_err(|e| Error::Store(format!("persisting PathInfo: {e}")))?;

            // Register in our session state
            let abs_path = output_path.to_absolute_path_with_prefix(&self.store_dir_str);
            self.built_outputs.insert(abs_path.clone(), path_info.clone());
            self.output_nodes
                .insert(output_path.clone(), final_node.clone());

            // Export the output from castore to the store directory on disk.
            // Skip if the path already exists (cache hit) or the store is
            // read-only (output stays in castore + PathInfo db only).
            if !PathBuf::from(&abs_path).exists() {
                match export_castore_to_disk(
                    &final_node,
                    &abs_path,
                    &self.blob_service,
                    &self.directory_service,
                )
                .await
                {
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

    /// Execute a builtin fetcher derivation (e.g., `builtin:fetchurl`).
    ///
    /// Bypasses the sandbox entirely: parses the derivation environment
    /// into a `Fetch`, downloads the resource, verifies the hash, then
    /// runs the standard post-build pipeline (ingest, NAR hash, PathInfo).
    async fn build_fetcher(
        &mut self,
        drv_path: &StorePath<String>,
        derivation: &Derivation,
        known_paths: &mut KnownPaths,
    ) -> Result<BuildOutcome, Error> {
        let drv_name = drv_path.name().to_string();

        // 1. Parse the derivation into a typed Fetch.
        let fetch = crate::fetcher::parse_fetch(derivation).map_err(|e| {
            Error::BuildFailed {
                name: drv_name.clone(),
                exit_code: "parse".to_string(),
                log: e.to_string(),
            }
        })?;

        // 2. Validate: fetchers must be fixed-output derivations.
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

        // 3. Execute the fetch in a blocking task (ureq is sync).
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

        // 4. Verify output was created.
        if !PathBuf::from(&out_abs).exists() {
            return Err(Error::BuildFailed {
                name: drv_name.clone(),
                exit_code: "fetch".to_string(),
                log: format!("fetcher did not produce output at {out_abs}"),
            });
        }

        // 5. Verify flat hash for URL fetches (before ingest).
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

        // 6. Ingest output into castore.
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

        // 7. Compute NAR hash.
        let nar_renderer = SimpleRenderer::new(
            self.blob_service.clone(),
            self.directory_service.clone(),
        );
        let (nar_size, nar_sha256) = nar_renderer
            .calculate_nar(&node)
            .await
            .map_err(|e| Error::NarCalculation(e.to_string()))?;

        // 8. Verify FOD hash (NAR-based, for tarballs/git/NAR fetches).
        if let Some(ca_hash) = &out_output.ca_hash {
            verify_fod_hash(
                &drv_name,
                "out",
                ca_hash,
                nar_size,
                &nar_sha256,
                &node,
                &self.blob_service,
                &self.directory_service,
            )
            .await?;
        }

        // 9. Build PathInfo and persist.
        let path_info = PathInfo {
            store_path: out_path.clone(),
            node: node.clone(),
            references: vec![], // fetcher outputs have no references
            nar_size,
            nar_sha256,
            signatures: vec![],
            deriver: Some(drv_path.clone()),
            ca: out_output.ca_hash.clone(),
        };

        self.pathinfo_service
            .put(path_info.clone())
            .await
            .map_err(|e| Error::Store(format!("persisting PathInfo: {e}")))?;

        self.built_outputs
            .insert(out_abs.clone(), path_info.clone());
        self.output_nodes.insert(out_path.clone(), node);

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
            // For input-addressed: path is in the derivation.
            // For CA: look up in ca_mappings from a previous session.
            let output_path: StorePath<String> = match output.path.as_ref() {
                Some(p) => p.clone(),
                None => {
                    // CA derivation — check persistent mapping
                    match self.ca_mappings.get(&drv_abs, output_name) {
                        Some(ca_abs) => {
                            StorePath::from_absolute_path(ca_abs.as_bytes())
                                .map_err(|_| Error::Store(format!(
                                    "invalid CA mapping path: {ca_abs}"
                                )))?
                        }
                        None => return Ok(None), // no mapping = must build
                    }
                }
            };

            let abs = PathBuf::from(output_path.to_absolute_path_with_prefix(&self.store_dir_str));
            let digest = *output_path.digest();

            // Query PathInfoService
            let stored = self.pathinfo_service.get(digest).await
                .map_err(|e| Error::Store(format!("PathInfo lookup: {e}")))?;

            match (stored, abs.exists()) {
                (Some(path_info), true) => {
                    // Full cache hit: PathInfo + file on disk
                    self.output_nodes.insert(output_path.clone(), path_info.node.clone());
                    self.built_outputs.insert(
                        output_path.to_absolute_path_with_prefix(&self.store_dir_str),
                        path_info.clone(),
                    );
                    infos.insert(output_name.clone(), path_info);
                }
                (Some(_), false) => {
                    // PathInfo exists but file gone — inconsistent store
                    tracing::warn!(
                        path = %output_path.to_absolute_path_with_prefix(&self.store_dir_str),
                        "PathInfo exists but output missing from disk, rebuilding"
                    );
                    return Ok(None);
                }
                (None, true) => {
                    // File exists but no PathInfo — untracked, rebuild
                    debug!(
                        path = %output_path.to_absolute_path_with_prefix(&self.store_dir_str),
                        "output exists on disk but no PathInfo, rebuilding"
                    );
                    return Ok(None);
                }
                (None, false) => {
                    // Nothing — cache miss
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

/// Verify that a fixed-output derivation produced the expected hash.
///
/// Flat mode: hash the raw file bytes with the declared algorithm.
/// NAR mode: hash the NAR serialization with the declared algorithm.
/// Text mode: equivalent to NAR sha256 for verification purposes.
async fn verify_fod_hash(
    drv_name: &str,
    _output_name: &str,
    expected_ca: &CAHash,
    _nar_size: u64,
    nar_sha256: &[u8; 32],
    node: &Node,
    blob_service: &(impl BlobService + Clone),
    directory_service: &(impl DirectoryService + Clone),
) -> Result<(), Error> {
    match expected_ca {
        CAHash::Flat(expected_hash) => {
            let digest = match node {
                Node::File { digest, .. } => digest,
                _ => {
                    return Err(Error::FodFlatNotFile {
                        name: drv_name.to_string(),
                    });
                }
            };

            let actual = hash_blob(blob_service, digest, expected_hash.algo()).await?;
            if actual.digest_as_bytes() != expected_hash.digest_as_bytes() {
                return Err(Error::FodHashMismatch {
                    name: drv_name.to_string(),
                    expected_sri: crate::fetcher::nix_hash_to_sri(expected_hash),
                    actual_sri: crate::fetcher::nix_hash_to_sri(&actual),
                });
            }
        }
        CAHash::Nar(NixHash::Sha256(expected_digest)) => {
            if nar_sha256 != expected_digest {
                let expected_h = NixHash::Sha256(*expected_digest);
                let actual_h = NixHash::Sha256(*nar_sha256);
                return Err(Error::FodHashMismatch {
                    name: drv_name.to_string(),
                    expected_sri: crate::fetcher::nix_hash_to_sri(&expected_h),
                    actual_sri: crate::fetcher::nix_hash_to_sri(&actual_h),
                });
            }
        }
        CAHash::Nar(expected_hash) => {
            let actual = nar_hash(
                node,
                expected_hash.algo(),
                blob_service.clone(),
                directory_service.clone(),
            )
            .await?;
            if actual.digest_as_bytes() != expected_hash.digest_as_bytes() {
                return Err(Error::FodHashMismatch {
                    name: drv_name.to_string(),
                    expected_sri: crate::fetcher::nix_hash_to_sri(expected_hash),
                    actual_sri: crate::fetcher::nix_hash_to_sri(&actual),
                });
            }
        }
        CAHash::Text(expected_digest) => {
            if nar_sha256 != expected_digest {
                let expected_h = NixHash::Sha256(*expected_digest);
                let actual_h = NixHash::Sha256(*nar_sha256);
                return Err(Error::FodHashMismatch {
                    name: drv_name.to_string(),
                    expected_sri: crate::fetcher::nix_hash_to_sri(&expected_h),
                    actual_sri: crate::fetcher::nix_hash_to_sri(&actual_h),
                });
            }
        }
    }
    Ok(())
}

/// Serialize a node to NAR and hash the byte stream with the given algorithm.
async fn nar_hash(
    node: &Node,
    algo: nix_compat::nixhash::HashAlgo,
    blob_service: impl BlobService + Send,
    directory_service: impl DirectoryService + Send,
) -> Result<NixHash, Error> {
    use nix_compat::nixhash::HashAlgo;

    match algo {
        HashAlgo::Md5 => {
            let mut hasher = md5::Md5::new();
            write_nar(AsyncIoBridge(&mut hasher), node, blob_service, directory_service)
                .await
                .map_err(|e| Error::NarCalculation(e.to_string()))?;
            let hash: [u8; 16] = hasher.finalize().into();
            Ok(NixHash::Md5(hash))
        }
        HashAlgo::Sha1 => {
            let mut hasher = sha1::Sha1::new();
            write_nar(AsyncIoBridge(&mut hasher), node, blob_service, directory_service)
                .await
                .map_err(|e| Error::NarCalculation(e.to_string()))?;
            let hash: [u8; 20] = hasher.finalize().into();
            Ok(NixHash::Sha1(hash))
        }
        HashAlgo::Sha256 => {
            let mut hasher = sha2::Sha256::new();
            write_nar(AsyncIoBridge(&mut hasher), node, blob_service, directory_service)
                .await
                .map_err(|e| Error::NarCalculation(e.to_string()))?;
            let hash: [u8; 32] = hasher.finalize().into();
            Ok(NixHash::Sha256(hash))
        }
        HashAlgo::Sha512 => {
            let mut hasher = sha2::Sha512::new();
            write_nar(AsyncIoBridge(&mut hasher), node, blob_service, directory_service)
                .await
                .map_err(|e| Error::NarCalculation(e.to_string()))?;
            let hash: [u8; 64] = hasher.finalize().into();
            Ok(NixHash::Sha512(Box::new(hash)))
        }
        HashAlgo::Blake3 => {
            let mut hasher = blake3::Hasher::new();
            write_nar(AsyncIoBridge(&mut hasher), node, blob_service, directory_service)
                .await
                .map_err(|e| Error::NarCalculation(e.to_string()))?;
            let hash = blake3::Hasher::finalize(&hasher);
            Ok(NixHash::Blake3(*hash.as_bytes()))
        }
    }
}

/// Read a blob from the blob service and hash it with the given algorithm.
async fn hash_blob(
    blob_service: &impl BlobService,
    digest: &snix_castore::B3Digest,
    algo: nix_compat::nixhash::HashAlgo,
) -> Result<NixHash, Error> {
    use nix_compat::nixhash::HashAlgo;

    let mut reader = blob_service
        .open_read(digest)
        .await
        .map_err(|e| Error::Store(format!("blob read for FOD verification: {e}")))?
        .ok_or_else(|| Error::Store(format!("blob {digest} not found for FOD verification")))?;

    let mut buf = vec![0u8; 64 * 1024];

    match algo {
        HashAlgo::Md5 => {
            let mut hasher = md5::Md5::new();
            loop {
                let n = reader.read(&mut buf).await
                    .map_err(|e| Error::Store(format!("blob read: {e}")))?;
                if n == 0 { break; }
                hasher.update(&buf[..n]);
            }
            let hash: [u8; 16] = hasher.finalize().into();
            Ok(NixHash::Md5(hash))
        }
        HashAlgo::Sha1 => {
            let mut hasher = sha1::Sha1::new();
            loop {
                let n = reader.read(&mut buf).await
                    .map_err(|e| Error::Store(format!("blob read: {e}")))?;
                if n == 0 { break; }
                hasher.update(&buf[..n]);
            }
            let hash: [u8; 20] = hasher.finalize().into();
            Ok(NixHash::Sha1(hash))
        }
        HashAlgo::Sha256 => {
            let mut hasher = sha2::Sha256::new();
            loop {
                let n = reader.read(&mut buf).await
                    .map_err(|e| Error::Store(format!("blob read: {e}")))?;
                if n == 0 { break; }
                hasher.update(&buf[..n]);
            }
            let hash: [u8; 32] = hasher.finalize().into();
            Ok(NixHash::Sha256(hash))
        }
        HashAlgo::Sha512 => {
            let mut hasher = sha2::Sha512::new();
            loop {
                let n = reader.read(&mut buf).await
                    .map_err(|e| Error::Store(format!("blob read: {e}")))?;
                if n == 0 { break; }
                hasher.update(&buf[..n]);
            }
            let hash: [u8; 64] = hasher.finalize().into();
            Ok(NixHash::Sha512(Box::new(hash)))
        }
        HashAlgo::Blake3 => {
            let mut hasher = blake3::Hasher::new();
            loop {
                let n = reader.read(&mut buf).await
                    .map_err(|e| Error::Store(format!("blob read: {e}")))?;
                if n == 0 { break; }
                hasher.update(&buf[..n]);
            }
            let hash = blake3::Hasher::finalize(&hasher);
            Ok(NixHash::Blake3(*hash.as_bytes()))
        }
    }
}

/// Map refscan needle indices back to store path references.
/// Export a castore Node to a filesystem path.
///
/// Reconstructs files, directories, and symlinks on disk from the
/// content-addressed store. This is the inverse of `ingest_path`.
async fn export_castore_to_disk(
    node: &Node,
    dest: &str,
    blob_service: &(impl BlobService + Clone),
    directory_service: &(impl DirectoryService + Clone),
) -> Result<(), String> {
    match node {
        Node::File { digest, executable, .. } => {
            // Read blob content and write to disk.
            let mut reader = blob_service
                .open_read(digest)
                .await
                .map_err(|e| format!("opening blob {digest}: {e}"))?
                .ok_or_else(|| format!("blob {digest} not found in castore"))?;

            if let Some(parent) = std::path::Path::new(dest).parent() {
                std::fs::create_dir_all(parent)
                    .map_err(|e| format!("creating parent dir: {e}"))?;
            }
            let mut file = std::fs::File::create(dest)
                .map_err(|e| format!("creating {dest}: {e}"))?;
            let mut buf = vec![0u8; 64 * 1024];
            loop {
                let n = reader.read(&mut buf).await
                    .map_err(|e| format!("reading blob: {e}"))?;
                if n == 0 { break; }
                std::io::Write::write_all(&mut file, &buf[..n])
                    .map_err(|e| format!("writing {dest}: {e}"))?;
            }
            #[cfg(unix)]
            if *executable {
                use std::os::unix::fs::PermissionsExt;
                std::fs::set_permissions(dest, std::fs::Permissions::from_mode(0o555))
                    .map_err(|e| format!("setting executable: {e}"))?;
            }
        }
        Node::Symlink { target, .. } => {
            if let Some(parent) = std::path::Path::new(dest).parent() {
                std::fs::create_dir_all(parent)
                    .map_err(|e| format!("creating parent dir: {e}"))?;
            }
            #[cfg(unix)]
            {
                use std::os::unix::ffi::OsStrExt;
                let target_os = std::ffi::OsStr::from_bytes(target.as_ref());
                std::os::unix::fs::symlink(target_os, dest)
                    .map_err(|e| format!("creating symlink {dest}: {e}"))?;
            }
        }
        Node::Directory { digest, .. } => {
            std::fs::create_dir_all(dest)
                .map_err(|e| format!("creating dir {dest}: {e}"))?;

            let dir = directory_service
                .get(digest)
                .await
                .map_err(|e| format!("fetching directory {digest}: {e}"))?
                .ok_or_else(|| format!("directory {digest} not found in castore"))?;

            for (name, child_node) in dir.nodes() {
                let name_str = std::str::from_utf8(name.as_ref())
                    .map_err(|e| format!("non-UTF8 filename in directory: {e}"))?;
                let child_dest = format!("{dest}/{name_str}");
                Box::pin(export_castore_to_disk(
                    child_node,
                    &child_dest,
                    blob_service,
                    directory_service,
                ))
                .await?;
            }
        }
    }
    Ok(())
}

/// Query the Nix store for the runtime closure of a store path.
///
/// Runs `nix-store -qR <path>` and returns the list of absolute paths.
/// Returns an empty vec if nix-store is not available or the query fails
/// (graceful degradation — the build may still work if dependencies are
/// statically linked or the closure is incomplete).
fn resolve_nix_closure(abs_path: &str) -> Vec<String> {
    let output = std::process::Command::new("nix-store")
        .args(["-qR", abs_path])
        .output();

    match output {
        Ok(out) if out.status.success() => {
            String::from_utf8_lossy(&out.stdout)
                .lines()
                .filter(|l| !l.is_empty())
                .map(|l| l.to_string())
                .collect()
        }
        _ => vec![],
    }
}

/// Parse an absolute store path string into a `StorePath`, given the
/// store directory prefix. Returns `None` if the path doesn't start
/// with the prefix or can't be parsed.
fn parse_store_path(abs: &str, store_dir: &str) -> Option<StorePath<String>> {
    let suffix = abs.strip_prefix(store_dir)?.strip_prefix('/')?;
    StorePath::<String>::from_bytes(suffix.as_bytes()).ok().map(|sp| sp.to_owned())
}

fn resolve_references(
    found_needles: &BTreeSet<u64>,
    _all_needles: &[String],
    derivation: &Derivation,
    inputs: &BTreeMap<StorePath<String>, Node>,
) -> Vec<StorePath<String>> {
    // The needle list is: [output paths...] ++ [input paths...]
    // We build the reverse mapping.
    let output_paths: Vec<StorePath<String>> = derivation
        .outputs
        .values()
        .filter_map(|o| o.path.clone())
        .collect();
    let input_paths: Vec<StorePath<String>> = inputs.keys().cloned().collect();

    let all_paths: Vec<StorePath<String>> = output_paths
        .into_iter()
        .chain(input_paths.into_iter())
        .collect();

    found_needles
        .iter()
        .filter_map(|&idx| all_paths.get(idx as usize).cloned())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use nix_compat::nixhash::{CAHash, HashAlgo, NixHash};
    use snix_castore::blobservice::MemoryBlobService;
    use snix_castore::blobservice::BlobService;
    use snix_castore::directoryservice::RedbDirectoryService;
    use tokio::io::AsyncWriteExt;

    /// Insert bytes into a MemoryBlobService and return the (B3Digest, Node).
    async fn insert_blob(bs: &MemoryBlobService, data: &[u8]) -> (snix_castore::B3Digest, Node) {
        let mut writer = bs.open_write().await;
        writer.write_all(data).await.unwrap();
        let digest = writer.close().await.unwrap();
        let node = Node::File {
            digest: digest.clone(),
            size: data.len() as u64,
            executable: false,
        };
        (digest, node)
    }

    /// Compute the expected NixHash for given bytes and algo.
    fn expected_hash(data: &[u8], algo: HashAlgo) -> NixHash {
        use digest::Digest;
        match algo {
            HashAlgo::Md5 => {
                let h: [u8; 16] = md5::Md5::digest(data).into();
                NixHash::Md5(h)
            }
            HashAlgo::Sha1 => {
                let h: [u8; 20] = sha1::Sha1::digest(data).into();
                NixHash::Sha1(h)
            }
            HashAlgo::Sha256 => {
                let h: [u8; 32] = sha2::Sha256::digest(data).into();
                NixHash::Sha256(h)
            }
            HashAlgo::Sha512 => {
                let h: [u8; 64] = sha2::Sha512::digest(data).into();
                NixHash::Sha512(Box::new(h))
            }
            HashAlgo::Blake3 => {
                let h = blake3::hash(data);
                NixHash::Blake3(*h.as_bytes())
            }
        }
    }

    /// Create a temporary directory service for tests.
    fn tmp_ds() -> RedbDirectoryService {
        use snix_castore::directoryservice::RedbDirectoryServiceConfig;
        RedbDirectoryService::new_temporary(
            "test".to_string(),
            RedbDirectoryServiceConfig::default(),
        ).unwrap()
    }

    #[tokio::test]
    async fn flat_sha256_match() {
        let bs = MemoryBlobService::default();
        let ds = tmp_ds();
        let data = b"hello world";
        let (_, node) = insert_blob(&bs, data).await;
        let hash = expected_hash(data, HashAlgo::Sha256);
        let ca = CAHash::Flat(hash);
        let nar_sha256 = [0u8; 32]; // unused for flat

        verify_fod_hash("test-drv", "out", &ca, 0, &nar_sha256, &node, &bs, &ds)
            .await
            .expect("matching flat sha256 should pass");
    }

    #[tokio::test]
    async fn flat_sha256_mismatch() {
        let bs = MemoryBlobService::default();
        let ds = tmp_ds();
        let data = b"hello world";
        let (_, node) = insert_blob(&bs, data).await;
        let wrong = NixHash::Sha256([0xab; 32]);
        let ca = CAHash::Flat(wrong);

        let err = verify_fod_hash("test-drv", "out", &ca, 0, &[0; 32], &node, &bs, &ds)
            .await
            .unwrap_err();
        assert!(matches!(err, Error::FodHashMismatch { .. }));
    }

    #[tokio::test]
    async fn flat_sha1_match() {
        let bs = MemoryBlobService::default();
        let ds = tmp_ds();
        let data = b"sha1 test content";
        let (_, node) = insert_blob(&bs, data).await;
        let hash = expected_hash(data, HashAlgo::Sha1);
        let ca = CAHash::Flat(hash);

        verify_fod_hash("test-drv", "out", &ca, 0, &[0; 32], &node, &bs, &ds)
            .await
            .expect("matching flat sha1 should pass");
    }

    #[tokio::test]
    async fn flat_sha512_match() {
        let bs = MemoryBlobService::default();
        let ds = tmp_ds();
        let data = b"sha512 test content";
        let (_, node) = insert_blob(&bs, data).await;
        let hash = expected_hash(data, HashAlgo::Sha512);
        let ca = CAHash::Flat(hash);

        verify_fod_hash("test-drv", "out", &ca, 0, &[0; 32], &node, &bs, &ds)
            .await
            .expect("matching flat sha512 should pass");
    }

    #[tokio::test]
    async fn flat_md5_match() {
        let bs = MemoryBlobService::default();
        let ds = tmp_ds();
        let data = b"md5 test content";
        let (_, node) = insert_blob(&bs, data).await;
        let hash = expected_hash(data, HashAlgo::Md5);
        let ca = CAHash::Flat(hash);

        verify_fod_hash("test-drv", "out", &ca, 0, &[0; 32], &node, &bs, &ds)
            .await
            .expect("matching flat md5 should pass");
    }

    /// Helper to make a dummy B3Digest for tests that don't read from the blob service.
    fn dummy_b3() -> snix_castore::B3Digest {
        snix_castore::B3Digest::from(&[0u8; 32])
    }

    #[tokio::test]
    async fn flat_rejects_directory_node() {
        let bs = MemoryBlobService::default();
        let ds = tmp_ds();
        let node = Node::Directory {
            digest: dummy_b3(),
            size: 0,
        };
        let hash = NixHash::Sha256([0; 32]);
        let ca = CAHash::Flat(hash);

        let err = verify_fod_hash("test-drv", "out", &ca, 0, &[0; 32], &node, &bs, &ds)
            .await
            .unwrap_err();
        assert!(matches!(err, Error::FodFlatNotFile { .. }));
    }

    #[tokio::test]
    async fn flat_rejects_symlink_node() {
        let bs = MemoryBlobService::default();
        let ds = tmp_ds();
        let node = Node::Symlink {
            target: snix_castore::SymlinkTarget::try_from("target").unwrap(),
        };
        let hash = NixHash::Sha256([0; 32]);
        let ca = CAHash::Flat(hash);

        let err = verify_fod_hash("test-drv", "out", &ca, 0, &[0; 32], &node, &bs, &ds)
            .await
            .unwrap_err();
        assert!(matches!(err, Error::FodFlatNotFile { .. }));
    }

    #[tokio::test]
    async fn nar_sha256_match() {
        let bs = MemoryBlobService::default();
        let ds = tmp_ds();
        let expected = [0x42u8; 32];
        let ca = CAHash::Nar(NixHash::Sha256(expected));
        let node = Node::Directory {
            digest: dummy_b3(),
            size: 0,
        };

        verify_fod_hash("test-drv", "out", &ca, 100, &expected, &node, &bs, &ds)
            .await
            .expect("matching NAR sha256 should pass");
    }

    #[tokio::test]
    async fn nar_sha256_mismatch() {
        let bs = MemoryBlobService::default();
        let ds = tmp_ds();
        let expected = [0x42u8; 32];
        let actual = [0x00u8; 32];
        let ca = CAHash::Nar(NixHash::Sha256(expected));
        let node = Node::Directory {
            digest: dummy_b3(),
            size: 0,
        };

        let err = verify_fod_hash("test-drv", "out", &ca, 100, &actual, &node, &bs, &ds)
            .await
            .unwrap_err();
        assert!(matches!(err, Error::FodHashMismatch { .. }));
    }

    #[tokio::test]
    async fn text_sha256_match() {
        let bs = MemoryBlobService::default();
        let ds = tmp_ds();
        let expected = [0x42u8; 32];
        let ca = CAHash::Text(expected);
        let node = Node::Directory {
            digest: dummy_b3(),
            size: 0,
        };

        verify_fod_hash("test-drv", "out", &ca, 100, &expected, &node, &bs, &ds)
            .await
            .expect("matching text sha256 should pass");
    }

    #[tokio::test]
    async fn flat_empty_blob() {
        let bs = MemoryBlobService::default();
        let ds = tmp_ds();
        let data = b"";
        let (_, node) = insert_blob(&bs, data).await;
        let hash = expected_hash(data, HashAlgo::Sha256);
        let ca = CAHash::Flat(hash);

        verify_fod_hash("test-drv", "out", &ca, 0, &[0; 32], &node, &bs, &ds)
            .await
            .expect("empty blob flat sha256 should pass");
    }

    #[tokio::test]
    async fn flat_large_blob() {
        let bs = MemoryBlobService::default();
        let ds = tmp_ds();
        // 256KB — larger than the 64KB read buffer
        let data = vec![0xffu8; 256 * 1024];
        let (_, node) = insert_blob(&bs, &data).await;
        let hash = expected_hash(&data, HashAlgo::Sha256);
        let ca = CAHash::Flat(hash);

        verify_fod_hash("test-drv", "out", &ca, 0, &[0; 32], &node, &bs, &ds)
            .await
            .expect("large blob should hash correctly across buffer boundaries");
    }

    // --- NAR non-sha256 tests ---

    /// Compute the NAR hash for a file node using nar_hash() to get the
    /// "correct" value, then verify verify_fod_hash accepts it.
    #[tokio::test]
    async fn nar_sha1_match() {
        let bs = MemoryBlobService::default();
        let ds = tmp_ds();
        let data = b"nar sha1 test";
        let (_, node) = insert_blob(&bs, data).await;

        let hash = nar_hash(&node, HashAlgo::Sha1, bs.clone(), ds.clone())
            .await
            .unwrap();
        let ca = CAHash::Nar(hash);

        verify_fod_hash("test-drv", "out", &ca, 0, &[0; 32], &node, &bs, &ds)
            .await
            .expect("matching NAR sha1 should pass");
    }

    #[tokio::test]
    async fn nar_sha1_mismatch() {
        let bs = MemoryBlobService::default();
        let ds = tmp_ds();
        let data = b"nar sha1 test";
        let (_, node) = insert_blob(&bs, data).await;

        let ca = CAHash::Nar(NixHash::Sha1([0xaa; 20]));

        let err = verify_fod_hash("test-drv", "out", &ca, 0, &[0; 32], &node, &bs, &ds)
            .await
            .unwrap_err();
        assert!(matches!(err, Error::FodHashMismatch { .. }));
    }

    #[tokio::test]
    async fn nar_md5_match() {
        let bs = MemoryBlobService::default();
        let ds = tmp_ds();
        let data = b"nar md5 test";
        let (_, node) = insert_blob(&bs, data).await;

        let hash = nar_hash(&node, HashAlgo::Md5, bs.clone(), ds.clone())
            .await
            .unwrap();
        let ca = CAHash::Nar(hash);

        verify_fod_hash("test-drv", "out", &ca, 0, &[0; 32], &node, &bs, &ds)
            .await
            .expect("matching NAR md5 should pass");
    }

    #[tokio::test]
    async fn nar_md5_mismatch() {
        let bs = MemoryBlobService::default();
        let ds = tmp_ds();
        let data = b"nar md5 test";
        let (_, node) = insert_blob(&bs, data).await;

        let ca = CAHash::Nar(NixHash::Md5([0xbb; 16]));

        let err = verify_fod_hash("test-drv", "out", &ca, 0, &[0; 32], &node, &bs, &ds)
            .await
            .unwrap_err();
        assert!(matches!(err, Error::FodHashMismatch { .. }));
    }

    #[tokio::test]
    async fn nar_sha512_match() {
        let bs = MemoryBlobService::default();
        let ds = tmp_ds();
        let data = b"nar sha512 test";
        let (_, node) = insert_blob(&bs, data).await;

        let hash = nar_hash(&node, HashAlgo::Sha512, bs.clone(), ds.clone())
            .await
            .unwrap();
        let ca = CAHash::Nar(hash);

        verify_fod_hash("test-drv", "out", &ca, 0, &[0; 32], &node, &bs, &ds)
            .await
            .expect("matching NAR sha512 should pass");
    }

    #[tokio::test]
    async fn nar_sha512_mismatch() {
        let bs = MemoryBlobService::default();
        let ds = tmp_ds();
        let data = b"nar sha512 test";
        let (_, node) = insert_blob(&bs, data).await;

        let ca = CAHash::Nar(NixHash::Sha512(Box::new([0xcc; 64])));

        let err = verify_fod_hash("test-drv", "out", &ca, 0, &[0; 32], &node, &bs, &ds)
            .await
            .unwrap_err();
        assert!(matches!(err, Error::FodHashMismatch { .. }));
    }

    /// NAR sha1 on a directory node (multi-file output).
    /// Inserts a directory with two files, computes NAR sha1, verifies.
    #[tokio::test]
    async fn nar_sha1_directory_node() {
        use snix_castore::directoryservice::DirectoryService;
        use snix_castore::Directory;

        let bs = MemoryBlobService::default();
        let ds = tmp_ds();

        // Insert two file blobs
        let (digest_a, _) = insert_blob(&bs, b"file-a-content").await;
        let (digest_b, _) = insert_blob(&bs, b"file-b-content").await;

        // Build a directory containing two files
        let mut dir = Directory::new();
        dir.add(
            "a.txt".try_into().unwrap(),
            Node::File {
                digest: digest_a,
                size: 14,
                executable: false,
            },
        )
        .unwrap();
        dir.add(
            "b.txt".try_into().unwrap(),
            Node::File {
                digest: digest_b,
                size: 14,
                executable: false,
            },
        )
        .unwrap();

        let dir_digest = dir.digest();
        let dir_size = dir.size();
        ds.put(dir).await.unwrap();

        let dir_node = Node::Directory {
            digest: dir_digest,
            size: dir_size,
        };

        // Compute NAR sha1 for the directory
        let hash = nar_hash(&dir_node, HashAlgo::Sha1, bs.clone(), ds.clone())
            .await
            .unwrap();
        let ca = CAHash::Nar(hash);

        verify_fod_hash("test-drv", "out", &ca, 0, &[0; 32], &dir_node, &bs, &ds)
            .await
            .expect("matching NAR sha1 on directory should pass");
    }

    /// Cross-check: NAR sha256 via nar_hash() matches calculate_size_and_sha256().
    /// This validates that nar_hash produces the same NAR serialization as
    /// the existing code path.
    #[tokio::test]
    async fn nar_hash_sha256_matches_calculate() {
        let bs = MemoryBlobService::default();
        let ds = tmp_ds();
        let data = b"cross-check content";
        let (_, node) = insert_blob(&bs, data).await;

        // Existing path
        let renderer = SimpleRenderer::new(bs.clone(), ds.clone());
        let (_size, sha256_from_calc) = renderer.calculate_nar(&node).await.unwrap();

        // New path
        let hash_from_nar_hash = nar_hash(&node, HashAlgo::Sha256, bs.clone(), ds.clone())
            .await
            .unwrap();

        assert_eq!(
            &sha256_from_calc[..],
            hash_from_nar_hash.digest_as_bytes(),
            "nar_hash(Sha256) should produce the same digest as calculate_nar"
        );
    }

    // ── Phase 2: resolve_references tests ─────────────────────

    fn make_test_drv_for_refs() -> (Derivation, BTreeMap<StorePath<String>, Node>) {
        let mut outputs = BTreeMap::new();
        outputs.insert(
            "out".to_string(),
            nix_compat::derivation::Output {
                path: Some(
                    StorePath::from_absolute_path(
                        b"/nix/store/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa-out",
                    ).unwrap(),
                ),
                ca_hash: None,
            },
        );
        let drv = Derivation {
            arguments: vec![],
            builder: "/bin/sh".to_string(),
            environment: BTreeMap::new(),
            input_derivations: BTreeMap::new(),
            input_sources: BTreeSet::new(),
            outputs,
            system: "x86_64-linux".to_string(),
        };

        let input_path: StorePath<String> = StorePath::from_absolute_path(
            b"/nix/store/bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb-input",
        ).unwrap();
        let mut inputs = BTreeMap::new();
        inputs.insert(
            input_path,
            Node::Symlink {
                target: snix_castore::SymlinkTarget::try_from("x").unwrap(),
            },
        );
        (drv, inputs)
    }

    #[test]
    fn resolve_refs_index_zero_maps_to_output() {
        let (drv, inputs) = make_test_drv_for_refs();
        let found = BTreeSet::from([0u64]);
        let needles = vec!["a".to_string(), "b".to_string()];

        let refs = resolve_references(&found, &needles, &drv, &inputs);
        assert_eq!(refs.len(), 1);
        assert_eq!(
            refs[0].to_absolute_path(),
            "/nix/store/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa-out"
        );
    }

    #[test]
    fn resolve_refs_index_past_outputs_maps_to_input() {
        let (drv, inputs) = make_test_drv_for_refs();
        // Index 1 = first input (after 1 output)
        let found = BTreeSet::from([1u64]);
        let needles = vec!["a".to_string(), "b".to_string()];

        let refs = resolve_references(&found, &needles, &drv, &inputs);
        assert_eq!(refs.len(), 1);
        assert_eq!(
            refs[0].to_absolute_path(),
            "/nix/store/bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb-input"
        );
    }

    #[test]
    fn resolve_refs_out_of_range_ignored() {
        let (drv, inputs) = make_test_drv_for_refs();
        let found = BTreeSet::from([999u64]);
        let needles = vec!["a".to_string()];

        let refs = resolve_references(&found, &needles, &drv, &inputs);
        assert!(refs.is_empty());
    }

    #[test]
    fn resolve_refs_empty_needles() {
        let (drv, inputs) = make_test_drv_for_refs();
        let found = BTreeSet::new();
        let needles: Vec<String> = vec![];

        let refs = resolve_references(&found, &needles, &drv, &inputs);
        assert!(refs.is_empty());
    }

    // ── Phase 3 & 4: MockBuildService + Builder orchestration ────

    use std::sync::{Arc, Mutex};
    use snix_build::buildservice::{BuildRequest, BuildResult, BuildOutput};
    use snix_store::pathinfoservice::LruPathInfoService;

    /// Create an in-memory LruPathInfoService for tests.
    fn test_pis() -> LruPathInfoService {
        LruPathInfoService::with_capacity(
            "test".to_string(),
            std::num::NonZeroUsize::new(128).unwrap(),
        )
    }

    /// A mock BuildService that records requests and returns a synthetic
    /// output node for each requested output. Uses a shared blob service
    /// so the Builder can find the blobs during NAR calculation.
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

            // Write blob to the shared blob service so NAR calc can find it
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
        input_drvs: &[(StorePath<String>, &str)], // (drv_path, output_name)
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

        // hdm needs to look up parent derivation modulos
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
        // First call should be for "dep" (contains "dep" in args)
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
        // A is shared dep
        let (a_path, _) = build_and_register("aaa", &[], &mut kp);
        // B and C both depend on A
        let (b_path, _) = build_and_register("bbb", &[(a_path.clone(), "out")], &mut kp);
        let (c_path, _) = build_and_register("ccc", &[(a_path.clone(), "out")], &mut kp);
        // D depends on B and C
        let (d_path, _) = build_and_register(
            "ddd",
            &[(b_path.clone(), "out"), (c_path.clone(), "out")],
            &mut kp,
        );

        let outcome = builder.build(&d_path, &mut kp).await.unwrap();
        assert!(!outcome.cached);

        let recorded = calls.lock().unwrap();
        // A, B, C, D = 4 builds. A should appear exactly once.
        assert_eq!(recorded.len(), 4, "should build A+B+C+D: {:?}", *recorded);
        let a_builds = recorded.iter()
            .filter(|args| args.iter().any(|a| a.contains("aaa")))
            .count();
        assert_eq!(a_builds, 1, "shared dep A should build exactly once");
    }

    // ── Phase 4: Cache tests ────────────────────────────────────

    /// Test that the cache check works: if the output path exists on disk,
    /// the builder skips the build and returns cached: true.
    ///
    /// Uses a tempdir as store_dir so no /nix/store write access is needed.
    /// Cache hit requires BOTH PathInfo in the service AND file on disk.
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

        // Pre-create the output path on disk.
        let out_path = drv.outputs["out"].path.as_ref().unwrap();
        let abs = PathBuf::from(out_path.to_absolute_path_with_prefix(&store_dir));
        std::fs::create_dir_all(abs.parent().unwrap()).unwrap();
        std::fs::write(&abs, "cached content").unwrap();

        // Pre-populate PathInfoService with a dummy PathInfo.
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

    /// PathInfo exists but file doesn't = cache miss, rebuild.
    #[tokio::test]
    async fn cache_miss_when_pathinfo_but_no_file() {
        let bs = MemoryBlobService::default();
        let ds = tmp_ds();
        let (mock, calls) = MockBuildService::new(bs.clone());
        let pis = test_pis();

        let mut kp = crunch_glue::KnownPaths::default();
        let (drv_path, drv) = build_and_register("orphan", &[], &mut kp);

        // Put PathInfo but DON'T create the file on disk.
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

    /// File exists but no PathInfo = cache miss, rebuild.
    #[tokio::test]
    async fn cache_miss_when_file_but_no_pathinfo() {
        let tmp = tempfile::tempdir().unwrap();
        let store_dir = tmp.path().to_str().unwrap().to_string();

        let bs = MemoryBlobService::default();
        let ds = tmp_ds();
        let (mock, calls) = MockBuildService::new(bs.clone());
        let pis = test_pis(); // empty — no PathInfo stored

        let mut kp = crunch_glue::KnownPaths::new(&store_dir);
        let (drv_path, drv) = build_and_register("untracked", &[], &mut kp);

        // Create file on disk but DON'T put PathInfo.
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

    /// Build a CA derivation (output paths None, placeholder in env).
    fn build_and_register_ca(
        name: &str,
        kp: &mut crunch_glue::KnownPaths,
    ) -> (StorePath<String>, Derivation) {
        let raw = nix_compat::store_path::hash_placeholder("out");
        let placeholder = format!(
            "{}/{}",
            kp.store_dir(),
            raw.strip_prefix('/').unwrap_or(&raw)
        );
        let mut outputs = BTreeMap::new();
        outputs.insert("out".to_string(), nix_compat::derivation::Output {
            path: None, ca_hash: None,
        });
        let mut environment = BTreeMap::new();
        environment.insert("name".to_string(), name.into());
        environment.insert("system".to_string(), "x86_64-linux".into());
        environment.insert("builder".to_string(), "/bin/sh".into());
        environment.insert("out".to_string(), placeholder.into());

        let drv = Derivation {
            arguments: vec!["-c".into(), format!("echo {name} > $out")],
            builder: "/bin/sh".to_string(),
            environment,
            input_derivations: BTreeMap::new(),
            input_sources: BTreeSet::new(),
            outputs,
            system: "x86_64-linux".to_string(),
        };

        let hdm = drv.hash_derivation_modulo(|_| panic!("CA drv has no input derivations"));
        let drv_path = drv.calculate_derivation_path(name).unwrap();

        let mut fake_hash = [0u8; 32];
        for (i, b) in name.bytes().enumerate().take(32) {
            fake_hash[i] = b;
        }
        kp.insert_ca(fake_hash, drv_path.clone(), hdm, drv.clone(), true);

        (drv_path, drv)
    }

    /// CA derivation: output path is content-based, not from inputs.
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
        // The output path should be content-based, not input-based
        let path_str = pi.store_path.to_absolute_path();
        assert!(path_str.starts_with("/nix/store/"), "CA path in store: {path_str}");
        // The PathInfo should have a ca field
        assert!(pi.ca.is_some(), "CA PathInfo should have ca field");
    }

    /// Two CA derivations with identical build output get the same CA path.
    #[tokio::test]
    async fn ca_identical_outputs_same_path() {
        // Both use the same MockBuildService which produces "mock output"
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

        // Different derivation names but same build output content
        assert_ne!(drv_path1, drv_path2, "different drvs");

        let path1 = outcome1.outputs["out"].store_path.to_absolute_path();
        let path2 = outcome2.outputs["out"].store_path.to_absolute_path();
        // CA paths should be identical (same content → same hash)
        // BUT: the path name component includes the drv name, so paths
        // differ even with same content hash (CA path = hash + name).
        // This is correct behavior — same content but different names
        // produce different store paths.
        // To truly test identical paths, we'd need same name + same content.
        // Instead, verify both have ca field set.
        assert!(outcome1.outputs["out"].ca.is_some());
        assert!(outcome2.outputs["out"].ca.is_some());
    }

    /// Same name + same content = same CA path.
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


