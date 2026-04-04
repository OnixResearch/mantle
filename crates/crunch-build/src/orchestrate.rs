//! Build orchestration: recursively build derivations, check cache,
//! persist outputs.

use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::path::PathBuf;

use nix_compat::derivation::Derivation;
use nix_compat::nixhash::{CAHash, NixHash};
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

use crate::build_request::{collect_input_paths, derivation_to_build_request};
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
}

/// Orchestrates the build pipeline: evaluating dependencies, checking
/// cache, running builds, persisting results.
pub struct Builder<BS, DS, BServ> {
    blob_service: BS,
    directory_service: DS,
    build_service: BServ,
    store_dir: PathBuf,
    /// Output store path → PathInfo for outputs built in this session.
    built_outputs: HashMap<String, PathInfo>,
    /// Output store path → Node (castore root node) for outputs built or
    /// ingested in this session. Needed by downstream builds that reference
    /// these as inputs.
    output_nodes: HashMap<StorePath<String>, Node>,
    verbose: bool,
}

impl<BS, DS, BServ> Builder<BS, DS, BServ>
where
    BS: BlobService + Clone + 'static,
    DS: DirectoryService + Clone + 'static,
    BServ: BuildService,
{
    pub fn new(
        blob_service: BS,
        directory_service: DS,
        build_service: BServ,
        store_dir: PathBuf,
        verbose: bool,
    ) -> Self {
        Self {
            blob_service,
            directory_service,
            build_service,
            store_dir,
            built_outputs: HashMap::new(),
            output_nodes: HashMap::new(),
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
        known_paths: &KnownPaths,
    ) -> Result<BuildOutcome, Error> {
        let entry = known_paths
            .get_by_drv_path(&drv_path.to_absolute_path())
            .ok_or_else(|| Error::DerivationNotFound {
                path: drv_path.clone(),
            })?;
        let derivation = entry.derivation.clone();

        self.build_derivation(drv_path, &derivation, known_paths)
            .await
    }

    /// Recursive build implementation.
    async fn build_derivation(
        &mut self,
        drv_path: &StorePath<String>,
        derivation: &Derivation,
        known_paths: &KnownPaths,
    ) -> Result<BuildOutcome, Error> {
        let drv_name = drv_path.name().to_string();

        // 1. Check if all outputs already exist (cache hit)
        if self.all_outputs_exist(derivation) {
            info!(drv = %drv_name, "all outputs cached, skipping build");
            let outputs = self.load_cached_outputs(drv_path, derivation).await?;
            return Ok(BuildOutcome {
                drv_path: drv_path.clone(),
                outputs,
                cached: true,
            });
        }

        // 2. Recursively build all input derivations
        for (input_drv_path, _output_names) in &derivation.input_derivations {
            let input_entry = known_paths
                .get_by_drv_path(&input_drv_path.to_absolute_path())
                .ok_or_else(|| Error::DerivationNotFound {
                    path: input_drv_path.clone(),
                })?;
            let input_drv = input_entry.derivation.clone();

            // Skip if we've already built this in the current session
            let already_built = input_drv
                .outputs
                .values()
                .filter_map(|o| o.path.as_ref())
                .all(|p| self.output_nodes.contains_key(p) || self.path_exists_on_disk(p));

            if !already_built {
                self.build_derivation(input_drv_path, &input_drv, known_paths)
                    .await?;
            } else {
                // Even if built, make sure we have nodes for inputs that
                // exist on disk but weren't built this session.
                self.ensure_input_nodes(&input_drv).await?;
            }
        }

        // 3. Validate source inputs exist on disk
        for source_path in &derivation.input_sources {
            let abs = PathBuf::from(source_path.to_absolute_path());
            if !abs.exists() {
                return Err(Error::SourceNotFound {
                    path: source_path.clone(),
                });
            }

            // Ingest source into castore if we don't have its node yet
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

        // 4. Collect input nodes for the sandbox
        let input_paths = collect_input_paths(derivation, known_paths)?;
        let mut sandbox_inputs: BTreeMap<StorePath<String>, Node> = BTreeMap::new();
        for input_path in &input_paths {
            if let Some(node) = self.output_nodes.get(input_path) {
                sandbox_inputs.insert(input_path.clone(), node.clone());
            } else {
                // Try ingesting from disk
                let abs = PathBuf::from(input_path.to_absolute_path());
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
        let build_request = derivation_to_build_request(derivation, &sandbox_inputs)?;

        info!(drv = %drv_name, "building");
        if self.verbose {
            debug!(
                drv = %drv_name,
                inputs = sandbox_inputs.len(),
                outputs = build_request.outputs.len(),
                "starting sandbox build"
            );
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

        for (i, (output_name, output)) in derivation.outputs.iter().enumerate() {
            let output_path = output.path.as_ref().ok_or_else(|| Error::OutputNoPath {
                output: output_name.clone(),
                drv_name: drv_name.clone(),
            })?;

            let build_output = build_result.outputs.get(i).ok_or_else(|| {
                Error::OutputMissing {
                    output: output_name.clone(),
                }
            })?;

            // Compute NAR size and sha256
            let (nar_size, nar_sha256) = nar_renderer
                .calculate_nar(&build_output.node)
                .await
                .map_err(|e| Error::NarCalculation(e.to_string()))?;

            // FOD hash verification
            if let Some(ca_hash) = &output.ca_hash {
                verify_fod_hash(
                    &drv_name,
                    output_name,
                    ca_hash,
                    nar_size,
                    &nar_sha256,
                    &build_output.node,
                )?;
            }

            // Resolve references from refscan needles
            let references = resolve_references(
                &build_output.output_needles,
                &build_request.refscan_needles,
                derivation,
                &sandbox_inputs,
            );

            let path_info = PathInfo {
                store_path: output_path.clone(),
                node: build_output.node.clone(),
                references,
                nar_size,
                nar_sha256,
                signatures: vec![],
                deriver: Some(drv_path.clone()),
                ca: output.ca_hash.clone(),
            };

            // Register in our session state
            let abs_path = output_path.to_absolute_path();
            self.built_outputs.insert(abs_path, path_info.clone());
            self.output_nodes
                .insert(output_path.clone(), build_output.node.clone());

            output_infos.insert(output_name.clone(), path_info);
        }

        info!(
            drv = %drv_name,
            outputs = ?output_names.iter()
                .filter_map(|n| derivation.outputs.get(n)?.path.as_ref())
                .map(|p| p.to_absolute_path())
                .collect::<Vec<_>>(),
            "build succeeded"
        );

        Ok(BuildOutcome {
            drv_path: drv_path.clone(),
            outputs: output_infos,
            cached: false,
        })
    }

    /// Check if all outputs of a derivation exist on disk.
    fn all_outputs_exist(&self, derivation: &Derivation) -> bool {
        derivation.outputs.values().all(|output| {
            output
                .path
                .as_ref()
                .is_some_and(|p| self.path_exists_on_disk(p))
        })
    }

    /// Check if a store path exists on the filesystem.
    fn path_exists_on_disk(&self, path: &StorePath<String>) -> bool {
        let abs = PathBuf::from(path.to_absolute_path());
        abs.exists()
    }

    /// For cached outputs, construct PathInfo from what's on disk.
    async fn load_cached_outputs(
        &mut self,
        drv_path: &StorePath<String>,
        derivation: &Derivation,
    ) -> Result<HashMap<String, PathInfo>, Error> {
        let mut infos = HashMap::new();

        for (output_name, output) in &derivation.outputs {
            let output_path = output.path.as_ref().ok_or_else(|| Error::OutputNoPath {
                output: output_name.clone(),
                drv_name: drv_path.name().to_string(),
            })?;

            // Ingest from disk to compute node, NAR hash, etc.
            let abs = PathBuf::from(output_path.to_absolute_path());
            let node = ingest_path::<_, _, _, &[u8]>(
                self.blob_service.clone(),
                self.directory_service.clone(),
                &abs,
                None,
            )
            .await
            .map_err(|e| Error::Sandbox(std::io::Error::other(format!(
                "failed to ingest cached output {}: {e}",
                output_path
            ))))?;

            let nar_renderer = SimpleRenderer::new(
                self.blob_service.clone(),
                self.directory_service.clone(),
            );
            let (nar_size, nar_sha256) = nar_renderer
                .calculate_nar(&node)
                .await
                .map_err(|e| Error::NarCalculation(e.to_string()))?;

            let path_info = PathInfo {
                store_path: output_path.clone(),
                node: node.clone(),
                references: vec![], // no refscan for cached outputs
                nar_size,
                nar_sha256,
                signatures: vec![],
                deriver: Some(drv_path.clone()),
                ca: output.ca_hash.clone(),
            };

            self.output_nodes.insert(output_path.clone(), node);
            self.built_outputs
                .insert(output_path.to_absolute_path(), path_info.clone());

            infos.insert(output_name.clone(), path_info);
        }

        Ok(infos)
    }

    /// Make sure we have castore nodes for inputs that exist on disk
    /// (needed for passing as sandbox inputs to downstream builds).
    async fn ensure_input_nodes(&mut self, derivation: &Derivation) -> Result<(), Error> {
        for output in derivation.outputs.values() {
            if let Some(path) = &output.path {
                if !self.output_nodes.contains_key(path) {
                    let abs = PathBuf::from(path.to_absolute_path());
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
fn verify_fod_hash(
    drv_name: &str,
    _output_name: &str,
    expected_ca: &CAHash,
    _nar_size: u64,
    nar_sha256: &[u8; 32],
    _node: &Node,
) -> Result<(), Error> {
    // For FODs, the output hash must match the declared content address.
    // The actual verification depends on the hash mode:
    // - Flat: hash the file contents directly
    // - Nar (recursive): hash the NAR representation
    //
    // For v0 we verify NAR-hashed FODs (mode = recursive) by comparing
    // the NAR sha256. Flat-mode FODs would need the raw file content hash,
    // which we don't compute here yet — the sandbox already handles this.
    //
    // TODO: implement flat-mode FOD verification
    match expected_ca {
        CAHash::Nar(NixHash::Sha256(expected_digest)) => {
            if nar_sha256 != expected_digest {
                return Err(Error::FodHashMismatch {
                    name: drv_name.to_string(),
                    expected: data_encoding::HEXLOWER.encode(expected_digest),
                    actual: data_encoding::HEXLOWER.encode(nar_sha256),
                });
            }
        }
        _ => {
            // Other hash types / modes: trust the sandbox for now
            tracing::debug!("FOD verification for non-NAR-sha256 not yet implemented");
        }
    }
    Ok(())
}

/// Map refscan needle indices back to store path references.
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


