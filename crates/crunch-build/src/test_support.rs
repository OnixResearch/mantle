//! Shared test helpers for crunch-build tests.
//!
//! Provides MockBuildService, build_and_register, and factory functions
//! for in-memory blob/directory/pathinfo services.

#![cfg(test)]

use std::collections::{BTreeMap, BTreeSet};
use std::sync::{Arc, Mutex};

use nix_compat::derivation::{Derivation, Output};
use nix_compat::store_path::StorePath;
use snix_build::buildservice::{BuildOutput, BuildRequest, BuildResult, BuildService};
use snix_castore::blobservice::{BlobService, MemoryBlobService};
use snix_castore::directoryservice::{RedbDirectoryService, RedbDirectoryServiceConfig};
use snix_castore::Node;
use snix_store::pathinfoservice::LruPathInfoService;
use tokio::io::AsyncWriteExt;

use crunch_glue::KnownPaths;

/// Create a temporary in-memory directory service.
pub fn tmp_ds() -> RedbDirectoryService {
    RedbDirectoryService::new_temporary(
        "test".to_string(),
        RedbDirectoryServiceConfig::default(),
    )
    .unwrap()
}

/// Create an in-memory LRU path info service.
pub fn test_pis() -> LruPathInfoService {
    LruPathInfoService::with_capacity(
        "test".to_string(),
        std::num::NonZeroUsize::new(128).unwrap(),
    )
}

/// A mock BuildService that records command_args from each request and
/// returns a synthetic output node for each requested output.
pub struct MockBuildService {
    calls: Arc<Mutex<Vec<Vec<String>>>>,
    blob_service: MemoryBlobService,
}

impl MockBuildService {
    pub fn new(blob_service: MemoryBlobService) -> (Self, Arc<Mutex<Vec<Vec<String>>>>) {
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

#[tonic::async_trait]
impl BuildService for MockBuildService {
    async fn do_build(&self, request: BuildRequest) -> std::io::Result<BuildResult> {
        self.calls
            .lock()
            .unwrap()
            .push(request.command_args.clone());

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
///
/// `input_drvs`: `&[(parent_store_path, output_name)]` — parents must
/// already be registered in `kp`.
pub fn build_and_register(
    name: &str,
    input_drvs: &[(StorePath<String>, &str)],
    kp: &mut KnownPaths,
) -> (StorePath<String>, Derivation) {
    let mut outputs = BTreeMap::new();
    outputs.insert(
        "out".to_string(),
        Output {
            path: None,
            ca_hash: None,
        },
    );
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
