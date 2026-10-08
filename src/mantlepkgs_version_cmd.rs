// r[impl mantlepkgs_versions.observation_status]
// r[impl mantlepkgs_versions.producer_recheck]
// r[impl mantlepkgs_versions.revision_grouping]
// r[impl mantlepkgs_versions.claim_boundary]
// r[verify mantlepkgs_versions.observation_status]
// r[verify mantlepkgs_versions.producer_recheck]
// r[verify mantlepkgs_versions.revision_grouping]
// r[verify mantlepkgs_versions.claim_boundary]

use std::ffi::OsString;
use std::fs;
use std::io::Read;
use std::io::Write;
use std::os::unix::ffi::OsStrExt;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::path::PathBuf;
use std::process::Output;

use clap::Subcommand;
use mantle_application_contract::ApplicationOutcome;
use mantle_application_contract::CommandFamily;
use mantle_application_contract::EffectId;
use mantle_application_contract::EffectKind;
use mantle_application_contract::EffectMeasure;
use mantle_application_contract::EffectOutput;
use mantle_application_contract::EffectPlan;
use mantle_application_contract::EffectSpec;
use mantle_application_contract::ExpectedOutput;
use mantle_application_contract::Observation;
use mantle_application_contract::ObservationStatus;
use mantle_application_contract::classify_observations;
use mantle_application_contract::plan_effects;
use mantlepkgs_core::MantlepkgsManifest;
use mantlepkgs_core::VERSION_OBSERVATION_SCHEMA;
use mantlepkgs_core::VERSION_RECHECK_SCHEMA;
use mantlepkgs_core::VERSION_RECHECK_SET_SCHEMA;
use mantlepkgs_core::VersionCohort;
use mantlepkgs_core::VersionCohortRevision;
use mantlepkgs_core::VersionGroupRecheck;
use mantlepkgs_core::VersionIndex;
use mantlepkgs_core::VersionObservation;
use mantlepkgs_core::VersionObservationMethod;
use mantlepkgs_core::VersionObservationSet;
use mantlepkgs_core::VersionObservationStatus;
use mantlepkgs_core::VersionProductionPlan;
use mantlepkgs_core::VersionRecheckEntry;
use mantlepkgs_core::VersionRecheckSet;
use mantlepkgs_core::VersionRequestSet;
use mantlepkgs_core::VersionSelectionPolicy;
use mantlepkgs_core::build_version_group_manifests;
use mantlepkgs_core::build_version_index;
use mantlepkgs_core::build_version_observation_set;
use mantlepkgs_core::group_version_resolutions;
use mantlepkgs_core::resolve_version_requests;
use mantlepkgs_core::seal_version_cohort;
use mantlepkgs_core::seal_version_production_plan;
use mantlepkgs_core::seal_version_recheck_set;
use mantlepkgs_core::seal_version_request_set;
use mantlepkgs_core::seal_version_selection_policy;
use serde::Deserialize;
use serde::Serialize;
use serde::de::DeserializeOwned;

use crate::errors::RunError;
use crate::linux_rename::rename_path_no_replace;
use crate::mantlepkgs_cmd::core_eval_error;
use crate::mantlepkgs_cmd::run_nix;

const VERSION_INDEX_STDOUT_BYTES_MAX: usize = 1_048_576;
const VERSION_INPUT_BYTES_MAX: u64 = 67_108_864;
const NIX_EXECUTABLE_BYTES_MAX: u64 = 268_435_456;
const NIX_EXPERIMENTAL_FEATURES: &str = "nix-command flakes";
const VERSION_OBSERVATIONS_FILE: &str = "observations.json";
const VERSION_COHORT_FILE: &str = "cohort.json";
const VERSION_POLICY_FILE: &str = "policy.json";
const VERSION_REQUESTS_FILE: &str = "requests.json";
const VERSION_RESOLUTIONS_FILE: &str = "resolutions.json";
const VERSION_PLAN_FILE: &str = "production-plan.json";
const VERSION_RECHECKS_FILE: &str = "rechecks.json";
const VERSION_RECEIPTS_DIRECTORY: &str = "receipts";
const VERSION_MANIFESTS_DIRECTORY: &str = "manifests";
const VERSION_MANIFEST_JSON_FILE: &str = "manifest.json";
const VERSION_MANIFEST_NICKEL_FILE: &str = "manifest.ncl";
const VERSION_MANIFEST_CONTRACT_FILE: &str = "contracts.ncl";
const VERSION_MANIFEST_CONTRACT: &str = include_str!("../mantlepkgs/contracts.ncl");
const FAILURE_EXIT_CODE: u8 = 1;
const REQUIRED_OWNER_EXECUTE_BITS: u32 = 0o100;
const STAGE_SEPARATOR: &str = ".stage-";
const SOURCE_TREE_IDENTITY_DOMAIN: &[u8] = b"mantle.mantlepkgs.nixpkgs-source-tree.v1";
const SOURCE_TREE_DOMAIN_SEPARATOR: u8 = 0;
const SOURCE_TREE_ENTRY_MAX: u32 = 1_000_000;
const SOURCE_TREE_BYTES_MAX: u64 = 8_589_934_592;
const SOURCE_TREE_DEPTH_MAX: u32 = 128;
const SOURCE_TREE_READ_BUFFER_BYTES: usize = 65_536;
const SOURCE_TREE_EXECUTABLE_BITS: u32 = 0o111;
const SOURCE_TREE_FILE_TAG: &[u8] = b"file";
const SOURCE_TREE_DIRECTORY_TAG: &[u8] = b"directory";
const SOURCE_TREE_SYMLINK_TAG: &[u8] = b"symlink";

#[derive(Subcommand, Debug, Clone)]
pub(crate) enum MantlepkgsVersionAction {
    /// Observe one exact revision cohort and publish a compact per-system index
    Index {
        #[arg(long)]
        cohort: PathBuf,

        #[arg(long = "nix-program")]
        nix_program: PathBuf,

        #[arg(long = "output-root")]
        output_root: PathBuf,
    },

    /// Replay a saved index and publish deterministic receipts and revision groups
    Resolve {
        #[arg(long)]
        index: PathBuf,

        #[arg(long)]
        policy: PathBuf,

        #[arg(long)]
        requests: PathBuf,

        #[arg(long = "output-root")]
        output_root: PathBuf,
    },

    /// Recheck selected revisions and emit existing Mantlepkgs manifests
    Recheck {
        #[arg(long)]
        plan: PathBuf,

        #[arg(long = "template-manifest")]
        template_manifest: PathBuf,

        #[arg(long = "nix-program")]
        nix_program: PathBuf,

        #[arg(long = "output-root")]
        output_root: PathBuf,
    },
}

#[derive(Clone, Debug, Deserialize)]
struct NixFlakeMetadata {
    path: String,
    locked: NixLockedSource,
}

#[derive(Clone, Debug, Deserialize)]
struct NixFlakeArchive {
    path: String,
}

#[derive(Clone, Debug, Deserialize)]
struct NixLockedSource {
    #[serde(rename = "narHash")]
    nar_hash: String,
    rev: String,
    #[serde(rename = "type")]
    source_type: String,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct NixVersionResult {
    status: String,
    version: Option<String>,
}

#[derive(Debug)]
struct ObservedVersion {
    status: VersionObservationStatus,
    version: Option<String>,
    response_identity_blake3: Option<String>,
    reason_codes: Vec<String>,
}

// Each declaration is complete before the first Nickel evaluation, filesystem read, or Nix
// invocation. The final read is a distinct capability: publication/rename alone cannot certify its
// bytes.
const INDEX_EFFECTS: &[EffectSpec<'static>] = &[
    EffectSpec {
        effect_id: "version-index-input",
        kind: EffectKind::ReadFiles,
        limit: EffectMeasure::Calls(1),
        expected_output: ExpectedOutput::None,
    },
    EffectSpec {
        effect_id: "version-index-producer",
        kind: EffectKind::RunProcess,
        limit: EffectMeasure::Calls(1),
        expected_output: ExpectedOutput::None,
    },
    EffectSpec {
        effect_id: "version-index-publish",
        kind: EffectKind::WriteFiles,
        limit: EffectMeasure::Calls(1),
        expected_output: ExpectedOutput::None,
    },
    EffectSpec {
        effect_id: "version-index-readback",
        kind: EffectKind::ReadFiles,
        limit: EffectMeasure::Calls(1),
        expected_output: ExpectedOutput::None,
    },
];
const RESOLVE_EFFECTS: &[EffectSpec<'static>] = &[
    EffectSpec {
        effect_id: "version-resolve-inputs",
        kind: EffectKind::ReadFiles,
        limit: EffectMeasure::Calls(1),
        expected_output: ExpectedOutput::None,
    },
    EffectSpec {
        effect_id: "version-resolve-publish",
        kind: EffectKind::WriteFiles,
        limit: EffectMeasure::Calls(1),
        expected_output: ExpectedOutput::None,
    },
    EffectSpec {
        effect_id: "version-resolve-readback",
        kind: EffectKind::ReadFiles,
        limit: EffectMeasure::Calls(1),
        expected_output: ExpectedOutput::None,
    },
];
const RECHECK_EFFECTS: &[EffectSpec<'static>] = &[
    EffectSpec {
        effect_id: "version-recheck-inputs",
        kind: EffectKind::ReadFiles,
        limit: EffectMeasure::Calls(1),
        expected_output: ExpectedOutput::None,
    },
    EffectSpec {
        effect_id: "version-recheck-producer",
        kind: EffectKind::RunProcess,
        limit: EffectMeasure::Calls(1),
        expected_output: ExpectedOutput::None,
    },
    EffectSpec {
        effect_id: "version-recheck-publish",
        kind: EffectKind::WriteFiles,
        limit: EffectMeasure::Calls(1),
        expected_output: ExpectedOutput::None,
    },
    EffectSpec {
        effect_id: "version-recheck-readback",
        kind: EffectKind::ReadFiles,
        limit: EffectMeasure::Calls(1),
        expected_output: ExpectedOutput::None,
    },
];

fn version_plan(specs: &[EffectSpec<'_>]) -> Result<EffectPlan, RunError> {
    plan_effects(CommandFamily::Project, specs)
        .map_err(|error| RunError::Internal(format!("planning version effects: {}", error.code())))
}

struct VersionEffects {
    plan: EffectPlan,
    observations: Vec<Observation>,
}

impl VersionEffects {
    fn new(specs: &[EffectSpec<'_>]) -> Result<Self, RunError> {
        Ok(Self {
            plan: version_plan(specs)?,
            observations: Vec::with_capacity(specs.len()),
        })
    }

    fn execute<T>(
        &mut self,
        effect_id: &'static str,
        kind: EffectKind,
        usage: impl FnOnce() -> u32,
        action: impl FnOnce() -> Result<(T, EffectOutput), RunError>,
    ) -> Result<T, RunError> {
        let Some(expected) = self.plan.effects.get(self.observations.len()) else {
            return Err(RunError::Internal(format!("unplanned version effect: {effect_id}")));
        };
        if expected.effect_id.0 != effect_id || expected.kind != kind {
            return Err(RunError::Internal(format!("out-of-order version effect: {effect_id}")));
        }
        let (result, output): (Result<T, RunError>, EffectOutput) = match action() {
            Ok((value, output)) => (Ok(value), output),
            Err(error) => (Err(error), EffectOutput::None),
        };
        self.observations.push(Observation {
            effect_id: EffectId(effect_id.into()),
            kind,
            status: if result.is_ok() {
                ObservationStatus::Succeeded
            } else {
                ObservationStatus::Failed
            },
            output,
            usage: EffectMeasure::Calls(usage()),
            diagnostics_code: result.as_ref().err().map(|_| "version-effect-failed".into()),
        });
        if result.is_err() {
            self.finish_failure()?;
        }
        result
    }

    fn finish_failure(&mut self) -> Result<(), RunError> {
        for effect in &self.plan.effects[self.observations.len()..] {
            self.observations.push(Observation {
                effect_id: effect.effect_id.clone(),
                kind: effect.kind,
                status: ObservationStatus::Skipped,
                output: EffectOutput::None,
                usage: EffectMeasure::Calls(0),
                diagnostics_code: None,
            });
        }
        match classify_observations(&self.plan, &self.observations) {
            ApplicationOutcome::Failed { .. } => Ok(()),
            outcome => Err(RunError::Internal(format!("version effects contradicted plan: {outcome:?}"))),
        }
    }

    fn finish(&self) -> Result<(), RunError> {
        match classify_observations(&self.plan, &self.observations) {
            ApplicationOutcome::Completed => Ok(()),
            outcome => Err(RunError::Internal(format!("version effects contradicted plan: {outcome:?}"))),
        }
    }
}

// The port is intentionally version-specific: the core cannot open a path, launch
// Nix, or rename a stage. The concrete adapter retains those capabilities.
trait VersionPort {
    fn index_input(&self, path: &Path) -> Result<VersionCohort, RunError>;
    fn index_observations(&self, cohort: &VersionCohort, nix: &Path) -> Result<Vec<VersionObservation>, RunError>;
    fn resolution_inputs(
        &self,
        index: &Path,
        policy: &Path,
        requests: &Path,
    ) -> Result<(VersionIndex, VersionSelectionPolicy, VersionRequestSet), RunError>;
    fn recheck_inputs(
        &self,
        plan: &Path,
        template: &Path,
    ) -> Result<(VersionProductionPlan, MantlepkgsManifest), RunError>;
    fn recheck_observations(
        &self,
        plan: &VersionProductionPlan,
        nix: &Path,
    ) -> Result<Vec<VersionGroupRecheck>, RunError>;
    fn publish_index(
        &self,
        root: &Path,
        cohort: &VersionCohort,
        observations: &VersionObservationSet,
        index: &VersionIndex,
    ) -> Result<(), RunError>;
    fn publish_resolution(
        &self,
        root: &Path,
        policy: &VersionSelectionPolicy,
        requests: &VersionRequestSet,
        resolutions: &mantlepkgs_core::VersionResolutionSet,
        plan: &VersionProductionPlan,
    ) -> Result<(), RunError>;
    fn publish_recheck(
        &self,
        root: &Path,
        template: &Path,
        rechecks: &VersionRecheckSet,
        manifests: Option<&Vec<(String, MantlepkgsManifest)>>,
    ) -> Result<(), RunError>;
    fn verify_index(
        &self,
        root: &Path,
        cohort: &VersionCohort,
        observations: &VersionObservationSet,
        index: &VersionIndex,
    ) -> Result<EffectOutput, RunError>;
    fn verify_resolution(
        &self,
        root: &Path,
        policy: &VersionSelectionPolicy,
        requests: &VersionRequestSet,
        resolutions: &mantlepkgs_core::VersionResolutionSet,
        plan: &VersionProductionPlan,
    ) -> Result<EffectOutput, RunError>;
    fn verify_recheck(
        &self,
        root: &Path,
        rechecks: &VersionRecheckSet,
        manifests: Option<&Vec<(String, MantlepkgsManifest)>>,
    ) -> Result<EffectOutput, RunError>;
}

struct FsVersionPort;

fn publish_version(root: &Path, write_stage: impl FnOnce(&Path) -> Result<(), RunError>) -> Result<(), RunError> {
    let stage = prepare_stage(root)?;
    let result = write_stage(&stage).and_then(|()| publish_stage(&stage, root));
    cleanup_failed_stage(&stage, result.is_err());
    result
}

fn verify_published<T: DeserializeOwned + PartialEq>(path: &Path, expected: &T) -> Result<(), RunError> {
    let actual: T = read_json_bounded(path)?;
    if actual != *expected {
        return Err(RunError::Eval(format!(
            "published version artifact differs from planned output: {}",
            path.display()
        )));
    }
    Ok(())
}

impl VersionPort for FsVersionPort {
    fn index_input(&self, path: &Path) -> Result<VersionCohort, RunError> {
        seal_version_cohort(&evaluate_nickel(path, "version cohort")?).map_err(core_eval_error)
    }
    fn index_observations(&self, cohort: &VersionCohort, nix: &Path) -> Result<Vec<VersionObservation>, RunError> {
        validate_nix_program(nix)?;
        observe_cohort(cohort, nix)
    }
    fn resolution_inputs(
        &self,
        index: &Path,
        policy: &Path,
        requests: &Path,
    ) -> Result<(VersionIndex, VersionSelectionPolicy, VersionRequestSet), RunError> {
        Ok((
            read_json_bounded(index)?,
            evaluate_nickel(policy, "version selection policy")?,
            evaluate_nickel(requests, "version request set")?,
        ))
    }
    fn recheck_inputs(
        &self,
        plan: &Path,
        template: &Path,
    ) -> Result<(VersionProductionPlan, MantlepkgsManifest), RunError> {
        Ok((
            seal_version_production_plan(&read_json_bounded::<VersionProductionPlan>(plan)?)
                .map_err(core_eval_error)?,
            evaluate_nickel(template, "Mantlepkgs template manifest")?,
        ))
    }
    fn recheck_observations(
        &self,
        plan: &VersionProductionPlan,
        nix: &Path,
    ) -> Result<Vec<VersionGroupRecheck>, RunError> {
        validate_nix_program(nix)?;
        observe_rechecks(plan, nix)
    }
    fn publish_index(
        &self,
        root: &Path,
        cohort: &VersionCohort,
        observations: &VersionObservationSet,
        index: &VersionIndex,
    ) -> Result<(), RunError> {
        publish_version(root, |stage| write_index_stage(stage, cohort, observations, index))
    }
    fn publish_resolution(
        &self,
        root: &Path,
        policy: &VersionSelectionPolicy,
        requests: &VersionRequestSet,
        resolutions: &mantlepkgs_core::VersionResolutionSet,
        plan: &VersionProductionPlan,
    ) -> Result<(), RunError> {
        publish_version(root, |stage| write_resolution_stage(stage, policy, requests, resolutions, plan))
    }
    fn publish_recheck(
        &self,
        root: &Path,
        template: &Path,
        rechecks: &VersionRecheckSet,
        manifests: Option<&Vec<(String, MantlepkgsManifest)>>,
    ) -> Result<(), RunError> {
        let template_root = template
            .parent()
            .ok_or_else(|| RunError::Eval("the template manifest must have a parent directory".into()))?;
        publish_version(root, |stage| write_recheck_stage(stage, template_root, rechecks, manifests))
    }
    fn verify_index(
        &self,
        root: &Path,
        cohort: &VersionCohort,
        observations: &VersionObservationSet,
        index: &VersionIndex,
    ) -> Result<EffectOutput, RunError> {
        verify_published(&root.join(VERSION_COHORT_FILE), cohort)?;
        verify_published(&root.join(VERSION_OBSERVATIONS_FILE), observations)?;
        verify_published(&root.join(&cohort.index_relative_path), index)?;
        Ok(EffectOutput::None)
    }
    fn verify_resolution(
        &self,
        root: &Path,
        policy: &VersionSelectionPolicy,
        requests: &VersionRequestSet,
        resolutions: &mantlepkgs_core::VersionResolutionSet,
        plan: &VersionProductionPlan,
    ) -> Result<EffectOutput, RunError> {
        verify_published(&root.join(VERSION_POLICY_FILE), policy)?;
        verify_published(&root.join(VERSION_REQUESTS_FILE), requests)?;
        verify_published(&root.join(VERSION_RESOLUTIONS_FILE), resolutions)?;
        verify_published(&root.join(VERSION_PLAN_FILE), plan)?;
        for receipt in &resolutions.receipts {
            verify_published(
                &root.join(VERSION_RECEIPTS_DIRECTORY).join(format!("{}.json", receipt.receipt_identity_blake3)),
                receipt,
            )?;
        }
        Ok(EffectOutput::None)
    }
    fn verify_recheck(
        &self,
        root: &Path,
        rechecks: &VersionRecheckSet,
        manifests: Option<&Vec<(String, MantlepkgsManifest)>>,
    ) -> Result<EffectOutput, RunError> {
        verify_published(&root.join(VERSION_RECHECKS_FILE), rechecks)?;
        if let Some(manifests) = manifests {
            for (identity, manifest) in manifests {
                let group = root.join(VERSION_MANIFESTS_DIRECTORY).join(identity);
                verify_published(&group.join(VERSION_MANIFEST_JSON_FILE), manifest)?;
                let expected_nickel = render_manifest_nickel(manifest)?;
                verify_published_bytes(&group.join(VERSION_MANIFEST_NICKEL_FILE), expected_nickel.as_bytes())?;
                verify_published_bytes(
                    &group.join(VERSION_MANIFEST_CONTRACT_FILE),
                    VERSION_MANIFEST_CONTRACT.as_bytes(),
                )?;
                for artifact in [
                    &manifest.conversion_policy.translation_policy,
                    &manifest.conversion_policy.execution_profile,
                ] {
                    let path = group.join(&artifact.path);
                    let bytes = read_published_bytes(&path)?;
                    if blake3::hash(&bytes).to_hex().as_str() != artifact.digest_blake3 {
                        return Err(RunError::Eval(format!(
                            "published manifest policy digest mismatch: {}",
                            artifact.path
                        )));
                    }
                }
            }
        } else if root.join(VERSION_MANIFESTS_DIRECTORY).exists() {
            return Err(RunError::Eval("published blocked recheck unexpectedly contains manifests".into()));
        }
        Ok(EffectOutput::None)
    }
}

fn read_published_bytes(path: &Path) -> Result<Vec<u8>, RunError> {
    let metadata = fs::symlink_metadata(path).map_err(|error| {
        RunError::Internal(format!("reading published version artifact {}: {error}", path.display()))
    })?;
    if !metadata.is_file() || metadata.file_type().is_symlink() || metadata.len() > VERSION_INPUT_BYTES_MAX {
        return Err(RunError::Eval(format!(
            "published version artifact is not one bounded regular file: {}",
            path.display()
        )));
    }
    fs::read(path)
        .map_err(|error| RunError::Internal(format!("reading published version artifact {}: {error}", path.display())))
}

fn verify_published_bytes(path: &Path, expected: &[u8]) -> Result<(), RunError> {
    let actual = read_published_bytes(path)?;
    if actual != expected {
        return Err(RunError::Eval(format!(
            "published version artifact differs from planned output: {}",
            path.display()
        )));
    }
    Ok(())
}

pub(crate) fn cmd_mantlepkgs_version(action: MantlepkgsVersionAction, json: bool) -> Result<(), RunError> {
    match action {
        MantlepkgsVersionAction::Index {
            cohort,
            nix_program,
            output_root,
        } => run_index(&cohort, &nix_program, &output_root, json),
        MantlepkgsVersionAction::Resolve {
            index,
            policy,
            requests,
            output_root,
        } => run_resolve(&index, &policy, &requests, &output_root, json),
        MantlepkgsVersionAction::Recheck {
            plan,
            template_manifest,
            nix_program,
            output_root,
        } => run_recheck(&plan, &template_manifest, &nix_program, &output_root, json),
    }
}

fn run_index(cohort_path: &Path, nix_program: &Path, output_root: &Path, json: bool) -> Result<(), RunError> {
    run_index_with_port(&FsVersionPort, cohort_path, nix_program, output_root, json)
}

fn run_index_with_port(
    port: &impl VersionPort,
    cohort_path: &Path,
    nix_program: &Path,
    output_root: &Path,
    json: bool,
) -> Result<(), RunError> {
    let mut effects = VersionEffects::new(INDEX_EFFECTS)?;
    let cohort = effects.execute(
        "version-index-input",
        EffectKind::ReadFiles,
        || 1,
        || Ok((port.index_input(cohort_path)?, EffectOutput::None)),
    )?;
    let (observation_set, index) = effects.execute(
        "version-index-producer",
        EffectKind::RunProcess,
        || 1,
        || {
            let observations = port.index_observations(&cohort, nix_program)?;
            let set = build_version_observation_set(&cohort, &observations).map_err(core_eval_error)?;
            let index = build_version_index(&cohort, &set).map_err(core_eval_error)?;
            Ok(((set, index), EffectOutput::None))
        },
    )?;
    effects.execute(
        "version-index-publish",
        EffectKind::WriteFiles,
        || 1,
        || {
            port.publish_index(output_root, &cohort, &observation_set, &index)?;
            Ok(((), EffectOutput::None))
        },
    )?;
    effects.execute(
        "version-index-readback",
        EffectKind::ReadFiles,
        || 1,
        || {
            let output = port.verify_index(output_root, &cohort, &observation_set, &index)?;
            Ok(((), output))
        },
    )?;
    effects.finish()?;
    emit_index_result(&index, output_root, json)
}

fn observe_cohort(cohort: &VersionCohort, nix_program: &Path) -> Result<Vec<VersionObservation>, RunError> {
    let mut observations = Vec::new();
    for revision in &cohort.revisions {
        let metadata = observe_metadata(nix_program, revision)?;
        for attribute in &cohort.attributes {
            observations.push(observe_one(cohort, revision, attribute, nix_program, metadata.as_ref())?);
        }
    }
    let expected = cohort
        .revisions
        .len()
        .checked_mul(cohort.attributes.len())
        .ok_or_else(|| RunError::Internal("version observation count overflow".into()))?;
    assert_eq!(observations.len(), expected, "the shell must emit one observation for each declared pair");
    assert!(!observations.is_empty(), "the validated cohort must produce observations");
    Ok(observations)
}

fn observe_metadata(
    nix_program: &Path,
    revision: &VersionCohortRevision,
) -> Result<Option<NixFlakeMetadata>, RunError> {
    let output = run_nix(
        nix_program,
        &[
            "--extra-experimental-features",
            NIX_EXPERIMENTAL_FEATURES,
            "flake",
            "metadata",
            "--json",
            "--no-write-lock-file",
            &revision.reference,
        ],
        VERSION_INDEX_STDOUT_BYTES_MAX,
    )?;
    if !output.status.success() {
        return Ok(None);
    }
    let metadata = parse_flake_metadata(&output.stdout)?;
    if metadata.locked.source_type != "github"
        || metadata.locked.rev != revision.revision
        || metadata.locked.nar_hash != revision.nar_hash
    {
        return Ok(None);
    }
    assert_eq!(metadata.locked.rev, revision.revision);
    assert_eq!(metadata.locked.nar_hash, revision.nar_hash);
    Ok(Some(metadata))
}

fn observe_one(
    cohort: &VersionCohort,
    revision: &VersionCohortRevision,
    attribute: &str,
    nix_program: &Path,
    metadata: Option<&NixFlakeMetadata>,
) -> Result<VersionObservation, RunError> {
    let observed = if metadata.is_some() {
        observe_version(nix_program, &revision.reference, &cohort.system, attribute)?
    } else {
        ObservedVersion {
            status: VersionObservationStatus::Failed,
            version: None,
            response_identity_blake3: None,
            reason_codes: vec!["source-metadata-mismatch".into()],
        }
    };
    let observation = VersionObservation {
        schema: VERSION_OBSERVATION_SCHEMA.into(),
        observation_identity_blake3: String::new(),
        cohort_identity_blake3: cohort.cohort_identity_blake3.clone(),
        system: cohort.system.clone(),
        source_reference: revision.reference.clone(),
        revision: revision.revision.clone(),
        published_order: revision.published_order,
        nar_hash: revision.nar_hash.clone(),
        attribute: attribute.into(),
        method: VersionObservationMethod::PackageVersionAttribute,
        status: observed.status,
        reported_version: observed.version,
        response_identity_blake3: observed.response_identity_blake3,
        reason_codes: observed.reason_codes,
    };
    assert_eq!(observation.revision, revision.revision);
    assert_eq!(observation.attribute, attribute);
    Ok(observation)
}

fn observe_version(
    nix_program: &Path,
    reference: &str,
    system: &str,
    attribute: &str,
) -> Result<ObservedVersion, RunError> {
    let expression = version_expression(reference, system, attribute);
    let output = run_nix(
        nix_program,
        &[
            "--extra-experimental-features",
            NIX_EXPERIMENTAL_FEATURES,
            "eval",
            "--json",
            "--expr",
            &expression,
        ],
        VERSION_INDEX_STDOUT_BYTES_MAX,
    )?;
    if !output.status.success() {
        return Ok(ObservedVersion {
            status: VersionObservationStatus::Failed,
            version: None,
            response_identity_blake3: None,
            reason_codes: vec!["nix-evaluation-failed".into()],
        });
    }
    parse_version_output(&output)
}

fn version_expression(reference: &str, system: &str, attribute: &str) -> String {
    let attribute_path = attribute.split('.').map(|segment| format!(".\"{segment}\"")).collect::<String>();
    let expression = format!(
        "let flake = builtins.getFlake \"{reference}\"; \
         package = builtins.tryEval flake.legacyPackages.\"{system}\"{attribute_path}; in \
         if !package.success then {{ status = \"attribute-unavailable\"; version = null; }} \
         else let version = builtins.tryEval package.value.version; in \
         if !version.success then {{ status = \"version-unavailable\"; version = null; }} \
         else if builtins.isString version.value then {{ status = \"success\"; version = version.value; }} \
         else {{ status = \"malformed-version\"; version = null; }}"
    );
    assert!(expression.contains(reference));
    assert!(expression.contains(system));
    expression
}

fn parse_flake_metadata(bytes: &[u8]) -> Result<NixFlakeMetadata, RunError> {
    let metadata = serde_json::from_slice::<NixFlakeMetadata>(bytes)
        .map_err(|error| RunError::Eval(format!("decoding exact Nixpkgs metadata: {error}")))?;
    if metadata.path.is_empty() || !Path::new(&metadata.path).is_absolute() {
        return Err(RunError::Eval("exact Nixpkgs metadata returned an invalid source path".into()));
    }
    assert!(!metadata.locked.rev.is_empty());
    assert!(!metadata.locked.nar_hash.is_empty());
    Ok(metadata)
}

fn parse_version_output(output: &Output) -> Result<ObservedVersion, RunError> {
    let result = serde_json::from_slice::<NixVersionResult>(&output.stdout)
        .map_err(|error| RunError::Eval(format!("decoding Nixpkgs version observation: {error}")))?;
    let response_identity_blake3 = blake3::hash(&output.stdout).to_hex().to_string();
    let observed = match result.status.as_str() {
        "success" => {
            let version = result
                .version
                .filter(|value| !value.is_empty())
                .ok_or_else(|| RunError::Eval("successful Nixpkgs version observation omitted version text".into()))?;
            ObservedVersion {
                status: VersionObservationStatus::Success,
                version: Some(version),
                response_identity_blake3: Some(response_identity_blake3),
                reason_codes: Vec::new(),
            }
        }
        "attribute-unavailable" => unavailable_version("attribute-unavailable", response_identity_blake3),
        "version-unavailable" => unavailable_version("version-unavailable", response_identity_blake3),
        "malformed-version" => ObservedVersion {
            status: VersionObservationStatus::Failed,
            version: None,
            response_identity_blake3: Some(response_identity_blake3),
            reason_codes: vec!["malformed-version".into()],
        },
        _ => return Err(RunError::Eval("Nixpkgs version observation returned an unknown status".into())),
    };
    assert!(observed.status != VersionObservationStatus::Success || observed.version.is_some());
    assert!(observed.status == VersionObservationStatus::Success || !observed.reason_codes.is_empty());
    Ok(observed)
}

fn unavailable_version(reason: &str, response_identity_blake3: String) -> ObservedVersion {
    let observed = ObservedVersion {
        status: VersionObservationStatus::Unavailable,
        version: None,
        response_identity_blake3: Some(response_identity_blake3),
        reason_codes: vec![reason.into()],
    };
    assert!(observed.version.is_none());
    assert_eq!(observed.reason_codes.len(), 1);
    observed
}

fn run_resolve(
    index_path: &Path,
    policy_path: &Path,
    request_path: &Path,
    output_root: &Path,
    json: bool,
) -> Result<(), RunError> {
    run_resolve_with_port(&FsVersionPort, index_path, policy_path, request_path, output_root, json)
}

fn run_resolve_with_port(
    port: &impl VersionPort,
    index_path: &Path,
    policy_path: &Path,
    request_path: &Path,
    output_root: &Path,
    json: bool,
) -> Result<(), RunError> {
    let mut effects = VersionEffects::new(RESOLVE_EFFECTS)?;
    let (policy, requests, resolutions, plan) = effects.execute(
        "version-resolve-inputs",
        EffectKind::ReadFiles,
        || 1,
        || {
            let (index, mut policy_template, mut request_template) =
                port.resolution_inputs(index_path, policy_path, request_path)?;
            bind_policy_template_to_index(&mut policy_template, &index);
            let policy = seal_version_selection_policy(&policy_template).map_err(core_eval_error)?;
            if request_template.policy_identity_blake3.is_empty() {
                request_template.policy_identity_blake3 = policy.policy_identity_blake3.clone();
            }
            let requests = seal_version_request_set(&policy, &request_template).map_err(core_eval_error)?;
            let resolutions = resolve_version_requests(&index, &policy, &requests).map_err(core_eval_error)?;
            let plan = group_version_resolutions(&resolutions, &policy.limits).map_err(core_eval_error)?;
            Ok(((policy, requests, resolutions, plan), EffectOutput::None))
        },
    )?;
    effects.execute(
        "version-resolve-publish",
        EffectKind::WriteFiles,
        || 1,
        || {
            port.publish_resolution(output_root, &policy, &requests, &resolutions, &plan)?;
            Ok(((), EffectOutput::None))
        },
    )?;
    effects.execute(
        "version-resolve-readback",
        EffectKind::ReadFiles,
        || 1,
        || {
            let output = port.verify_resolution(output_root, &policy, &requests, &resolutions, &plan)?;
            Ok(((), output))
        },
    )?;
    effects.finish()?;
    emit_resolution_result(&plan, output_root, json)?;
    if plan.blocked_receipt_identity_blake3.is_empty() {
        Ok(())
    } else {
        Err(RunError::Reported(FAILURE_EXIT_CODE))
    }
}

fn bind_policy_template_to_index(policy: &mut VersionSelectionPolicy, index: &VersionIndex) {
    if policy.accepted_cohort_identity_blake3.is_empty() {
        policy.accepted_cohort_identity_blake3 = index.cohort_identity_blake3.clone();
    }
    if policy.accepted_observation_set_identity_blake3.is_empty() {
        policy.accepted_observation_set_identity_blake3 = index.observation_set_identity_blake3.clone();
    }
    if policy.accepted_index_identity_blake3.is_empty() {
        policy.accepted_index_identity_blake3 = index.index_identity_blake3.clone();
    }
    assert!(!policy.accepted_cohort_identity_blake3.is_empty());
    assert!(!policy.accepted_index_identity_blake3.is_empty());
}

fn run_recheck(
    plan_path: &Path,
    template_manifest_path: &Path,
    nix_program: &Path,
    output_root: &Path,
    json: bool,
) -> Result<(), RunError> {
    run_recheck_with_port(&FsVersionPort, plan_path, template_manifest_path, nix_program, output_root, json)
}

fn run_recheck_with_port(
    port: &impl VersionPort,
    plan_path: &Path,
    template_manifest_path: &Path,
    nix_program: &Path,
    output_root: &Path,
    json: bool,
) -> Result<(), RunError> {
    let mut effects = VersionEffects::new(RECHECK_EFFECTS)?;
    let (plan, template) = effects.execute(
        "version-recheck-inputs",
        EffectKind::ReadFiles,
        || 1,
        || Ok((port.recheck_inputs(plan_path, template_manifest_path)?, EffectOutput::None)),
    )?;
    let recheck_set = effects.execute(
        "version-recheck-producer",
        EffectKind::RunProcess,
        || 1,
        || {
            let raw_rechecks = port.recheck_observations(&plan, nix_program)?;
            let set = seal_version_recheck_set(&plan, &VersionRecheckSet {
                schema: VERSION_RECHECK_SET_SCHEMA.into(),
                recheck_set_identity_blake3: String::new(),
                plan_identity_blake3: plan.plan_identity_blake3.clone(),
                rechecks: raw_rechecks,
            })
            .map_err(core_eval_error)?;
            Ok((set, EffectOutput::None))
        },
    )?;
    let manifests = build_version_group_manifests(&template, &plan, &recheck_set);
    effects.execute(
        "version-recheck-publish",
        EffectKind::WriteFiles,
        || 1,
        || {
            port.publish_recheck(output_root, template_manifest_path, &recheck_set, manifests.as_ref().ok())?;
            Ok(((), EffectOutput::None))
        },
    )?;
    effects.execute(
        "version-recheck-readback",
        EffectKind::ReadFiles,
        || 1,
        || {
            let output = port.verify_recheck(output_root, &recheck_set, manifests.as_ref().ok())?;
            Ok(((), output))
        },
    )?;
    effects.finish()?;
    match manifests {
        Ok(manifests) => emit_recheck_result(&recheck_set, &manifests, output_root, json),
        Err(error) => {
            emit_recheck_blocked(&recheck_set, output_root, json)?;
            let _ = error;
            Err(RunError::Reported(FAILURE_EXIT_CODE))
        }
    }
}

fn observe_rechecks(plan: &VersionProductionPlan, nix_program: &Path) -> Result<Vec<VersionGroupRecheck>, RunError> {
    let mut rechecks = Vec::with_capacity(plan.groups.len());
    for group in &plan.groups {
        let metadata_output = observe_group_metadata(nix_program, group)?;
        let (source_tree_blake3, group_reasons) = source_recheck_facts(metadata_output.as_ref())?;
        let mut entries = Vec::with_capacity(group.selectors.len());
        for selector in &group.selectors {
            entries.push(observe_recheck_entry(group, selector, nix_program, metadata_output.is_some())?);
        }
        rechecks.push(VersionGroupRecheck {
            schema: VERSION_RECHECK_SCHEMA.into(),
            recheck_identity_blake3: String::new(),
            group_identity_blake3: group.group_identity_blake3.clone(),
            source_reference: group.source_reference.clone(),
            revision: group.revision.clone(),
            nar_hash: group.nar_hash.clone(),
            source_tree_blake3,
            entries,
            reason_codes: group_reasons,
        });
    }
    assert_eq!(rechecks.len(), plan.groups.len());
    assert!(rechecks.iter().all(|item| !item.group_identity_blake3.is_empty()));
    Ok(rechecks)
}

fn observe_group_metadata(
    nix_program: &Path,
    group: &mantlepkgs_core::VersionRevisionGroup,
) -> Result<Option<NixFlakeMetadata>, RunError> {
    let revision = VersionCohortRevision {
        published_order: group.published_order,
        reference: group.source_reference.clone(),
        revision: group.revision.clone(),
        nar_hash: group.nar_hash.clone(),
    };
    let Some(mut metadata) = observe_metadata(nix_program, &revision)? else {
        return Ok(None);
    };
    let Some(path) = materialize_flake_source(nix_program, &group.source_reference)? else {
        return Ok(None);
    };
    metadata.path = path;
    assert_eq!(metadata.locked.rev, group.revision);
    assert!(Path::new(&metadata.path).is_absolute());
    Ok(Some(metadata))
}

fn materialize_flake_source(nix_program: &Path, reference: &str) -> Result<Option<String>, RunError> {
    let output = run_nix(
        nix_program,
        &[
            "--extra-experimental-features",
            NIX_EXPERIMENTAL_FEATURES,
            "flake",
            "archive",
            "--json",
            "--no-write-lock-file",
            reference,
        ],
        VERSION_INDEX_STDOUT_BYTES_MAX,
    )?;
    if !output.status.success() {
        return Ok(None);
    }
    let archive = serde_json::from_slice::<NixFlakeArchive>(&output.stdout)
        .map_err(|error| RunError::Eval(format!("decoding materialized Nixpkgs source: {error}")))?;
    let path = Path::new(&archive.path);
    if !path.is_absolute() || !path.is_dir() {
        return Ok(None);
    }
    assert!(path.is_absolute());
    assert!(path.is_dir());
    Ok(Some(archive.path))
}

fn source_recheck_facts(metadata: Option<&NixFlakeMetadata>) -> Result<(Option<String>, Vec<String>), RunError> {
    let Some(metadata) = metadata else {
        return Ok((None, vec!["source-metadata-mismatch".into()]));
    };
    let source_path = Path::new(&metadata.path);
    let digest = hash_nixpkgs_source_tree(source_path)?;
    assert!(!digest.is_empty());
    assert!(source_path.is_absolute());
    Ok((Some(digest), Vec::new()))
}

fn hash_nixpkgs_source_tree(root: &Path) -> Result<String, RunError> {
    let metadata = fs::symlink_metadata(root)
        .map_err(|error| RunError::Internal(format!("reading Nixpkgs source root {}: {error}", root.display())))?;
    if !metadata.is_dir() || metadata.file_type().is_symlink() {
        return Err(RunError::Eval("the materialized Nixpkgs source must be one non-symlink directory".into()));
    }
    let mut entries = Vec::new();
    let mut total_bytes = 0_u64;
    collect_source_entries(root, root, 0, &mut entries, &mut total_bytes)?;
    entries.sort();
    let mut hasher = blake3::Hasher::new();
    hasher.update(SOURCE_TREE_IDENTITY_DOMAIN);
    hasher.update(&[SOURCE_TREE_DOMAIN_SEPARATOR]);
    for entry in &entries {
        hash_source_entry(root, entry, &mut hasher)?;
    }
    assert!(u32::try_from(entries.len()).unwrap_or(u32::MAX) <= SOURCE_TREE_ENTRY_MAX);
    assert!(total_bytes <= SOURCE_TREE_BYTES_MAX);
    Ok(hasher.finalize().to_hex().to_string())
}

fn collect_source_entries(
    root: &Path,
    current: &Path,
    depth: u32,
    entries: &mut Vec<PathBuf>,
    total_bytes: &mut u64,
) -> Result<(), RunError> {
    if depth > SOURCE_TREE_DEPTH_MAX {
        return Err(RunError::Eval("the Nixpkgs source tree exceeds its depth limit".into()));
    }
    let mut children = fs::read_dir(current)
        .map_err(|error| {
            RunError::Internal(format!("reading Nixpkgs source directory {}: {error}", current.display()))
        })?
        .map(|entry| {
            entry
                .map(|value| value.path())
                .map_err(|error| RunError::Internal(format!("reading Nixpkgs source directory entry: {error}")))
        })
        .collect::<Result<Vec<_>, _>>()?;
    children.sort();
    for child in children {
        if u32::try_from(entries.len()).unwrap_or(u32::MAX) >= SOURCE_TREE_ENTRY_MAX {
            return Err(RunError::Eval("the Nixpkgs source tree exceeds its entry limit".into()));
        }
        let metadata = fs::symlink_metadata(&child).map_err(|error| {
            RunError::Internal(format!("reading Nixpkgs source entry {}: {error}", child.display()))
        })?;
        let payload_bytes = source_entry_payload_bytes(&child, &metadata)?;
        let path_bytes = u64::try_from(child.as_os_str().as_bytes().len())
            .map_err(|_| RunError::Eval("the Nixpkgs source path length overflowed".into()))?;
        let entry_bytes = payload_bytes
            .checked_add(path_bytes)
            .ok_or_else(|| RunError::Eval("the Nixpkgs source entry byte count overflowed".into()))?;
        *total_bytes = total_bytes
            .checked_add(entry_bytes)
            .ok_or_else(|| RunError::Eval("the Nixpkgs source byte count overflowed".into()))?;
        if *total_bytes > SOURCE_TREE_BYTES_MAX {
            return Err(RunError::Eval("the Nixpkgs source tree exceeds its byte limit".into()));
        }
        entries.push(child.clone());
        if metadata.is_dir() {
            collect_source_entries(root, &child, depth.saturating_add(1), entries, total_bytes)?;
        }
    }
    assert!(current.starts_with(root));
    assert!(depth <= SOURCE_TREE_DEPTH_MAX);
    Ok(())
}

fn source_entry_payload_bytes(path: &Path, metadata: &fs::Metadata) -> Result<u64, RunError> {
    if metadata.is_file() {
        return Ok(metadata.len());
    }
    if metadata.is_dir() {
        return Ok(0);
    }
    if metadata.file_type().is_symlink() {
        let target = fs::read_link(path).map_err(|error| {
            RunError::Internal(format!("reading Nixpkgs source symlink {}: {error}", path.display()))
        })?;
        return u64::try_from(target.as_os_str().as_bytes().len())
            .map_err(|_| RunError::Eval("the Nixpkgs symlink target length overflowed".into()));
    }
    Err(RunError::Eval(format!("the Nixpkgs source contains a special file: {}", path.display())))
}

fn hash_source_entry(root: &Path, path: &Path, hasher: &mut blake3::Hasher) -> Result<(), RunError> {
    let relative = path
        .strip_prefix(root)
        .map_err(|_| RunError::Internal("a Nixpkgs source entry escaped its root".into()))?;
    let metadata = fs::symlink_metadata(path)
        .map_err(|error| RunError::Internal(format!("reading Nixpkgs source entry {}: {error}", path.display())))?;
    hash_framed(hasher, b"path", relative.as_os_str().as_bytes())?;
    if metadata.is_file() {
        hash_framed(hasher, b"kind", SOURCE_TREE_FILE_TAG)?;
        let executable = if metadata.permissions().mode() & SOURCE_TREE_EXECUTABLE_BITS == 0 {
            b"0"
        } else {
            b"1"
        };
        hash_framed(hasher, b"executable", executable)?;
        hash_file_contents(path, hasher)?;
    } else if metadata.is_dir() {
        hash_framed(hasher, b"kind", SOURCE_TREE_DIRECTORY_TAG)?;
    } else if metadata.file_type().is_symlink() {
        hash_framed(hasher, b"kind", SOURCE_TREE_SYMLINK_TAG)?;
        let target = fs::read_link(path).map_err(|error| {
            RunError::Internal(format!("reading Nixpkgs source symlink {}: {error}", path.display()))
        })?;
        hash_framed(hasher, b"target", target.as_os_str().as_bytes())?;
    } else {
        return Err(RunError::Eval(format!("the Nixpkgs source contains a special file: {}", path.display())));
    }
    assert!(path.starts_with(root));
    assert!(!relative.as_os_str().is_empty());
    Ok(())
}

fn hash_file_contents(path: &Path, hasher: &mut blake3::Hasher) -> Result<(), RunError> {
    let mut file = fs::File::open(path)
        .map_err(|error| RunError::Internal(format!("opening Nixpkgs source file {}: {error}", path.display())))?;
    let size = file
        .metadata()
        .map_err(|error| {
            RunError::Internal(format!("reading Nixpkgs source file metadata {}: {error}", path.display()))
        })?
        .len();
    hasher.update(b"content");
    hasher.update(&size.to_le_bytes());
    let mut buffer = [0_u8; SOURCE_TREE_READ_BUFFER_BYTES];
    let mut observed = 0_u64;
    loop {
        let read = file
            .read(&mut buffer)
            .map_err(|error| RunError::Internal(format!("reading Nixpkgs source file {}: {error}", path.display())))?;
        if read == 0 {
            break;
        }
        observed = observed
            .checked_add(u64::try_from(read).map_err(|_| RunError::Internal("source read length overflow".into()))?)
            .ok_or_else(|| RunError::Eval("the Nixpkgs source file length overflowed".into()))?;
        hasher.update(&buffer[..read]);
    }
    if observed != size {
        return Err(RunError::Eval(format!("the Nixpkgs source file changed during hashing: {}", path.display())));
    }
    assert_eq!(observed, size);
    assert!(observed <= SOURCE_TREE_BYTES_MAX);
    Ok(())
}

fn hash_framed(hasher: &mut blake3::Hasher, label: &[u8], value: &[u8]) -> Result<(), RunError> {
    let length =
        u64::try_from(value.len()).map_err(|_| RunError::Internal("source hash field length overflow".into()))?;
    hasher.update(label);
    hasher.update(&[SOURCE_TREE_DOMAIN_SEPARATOR]);
    hasher.update(&length.to_le_bytes());
    hasher.update(value);
    assert!(!label.is_empty());
    assert_eq!(length, u64::try_from(value.len()).expect("validated source hash field length"));
    Ok(())
}

fn observe_recheck_entry(
    group: &mantlepkgs_core::VersionRevisionGroup,
    selector: &mantlepkgs_core::VersionGroupSelector,
    nix_program: &Path,
    metadata_ready: bool,
) -> Result<VersionRecheckEntry, RunError> {
    if !metadata_ready {
        return Ok(VersionRecheckEntry {
            receipt_identity_blake3: selector.receipt_identity_blake3.clone(),
            attribute: selector.attribute.clone(),
            expected_version: selector.reported_version.clone(),
            observed_version: None,
            status: VersionObservationStatus::Failed,
            reason_codes: vec!["source-metadata-mismatch".into()],
        });
    }
    let observed = observe_version(nix_program, &group.source_reference, &group.system, &selector.attribute)?;
    let mut status = observed.status;
    let mut reasons = observed.reason_codes;
    if status == VersionObservationStatus::Success && observed.version.as_ref() != Some(&selector.reported_version) {
        status = VersionObservationStatus::Failed;
        reasons.push("rechecked-version-mismatch".into());
    }
    let entry = VersionRecheckEntry {
        receipt_identity_blake3: selector.receipt_identity_blake3.clone(),
        attribute: selector.attribute.clone(),
        expected_version: selector.reported_version.clone(),
        observed_version: observed.version,
        status,
        reason_codes: reasons,
    };
    assert_eq!(entry.receipt_identity_blake3, selector.receipt_identity_blake3);
    assert_eq!(entry.expected_version, selector.reported_version);
    Ok(entry)
}

fn write_index_stage(
    stage: &Path,
    cohort: &VersionCohort,
    observations: &VersionObservationSet,
    index: &VersionIndex,
) -> Result<(), RunError> {
    write_json_new(&stage.join(VERSION_COHORT_FILE), cohort)?;
    write_json_new(&stage.join(VERSION_OBSERVATIONS_FILE), observations)?;
    let index_path = stage.join(&cohort.index_relative_path);
    create_parent_directories(&index_path)?;
    write_json_new(&index_path, index)?;
    sync_directory(stage)?;
    assert!(stage.join(VERSION_COHORT_FILE).is_file());
    assert!(index_path.is_file());
    Ok(())
}

fn write_resolution_stage(
    stage: &Path,
    policy: &VersionSelectionPolicy,
    requests: &VersionRequestSet,
    resolutions: &mantlepkgs_core::VersionResolutionSet,
    plan: &VersionProductionPlan,
) -> Result<(), RunError> {
    write_json_new(&stage.join(VERSION_POLICY_FILE), policy)?;
    write_json_new(&stage.join(VERSION_REQUESTS_FILE), requests)?;
    write_json_new(&stage.join(VERSION_RESOLUTIONS_FILE), resolutions)?;
    write_json_new(&stage.join(VERSION_PLAN_FILE), plan)?;
    let receipt_root = stage.join(VERSION_RECEIPTS_DIRECTORY);
    fs::create_dir(&receipt_root)
        .map_err(|error| RunError::Internal(format!("creating version receipt directory: {error}")))?;
    for receipt in &resolutions.receipts {
        write_json_new(&receipt_root.join(format!("{}.json", receipt.receipt_identity_blake3)), receipt)?;
    }
    sync_directory(&receipt_root)?;
    sync_directory(stage)?;
    assert_eq!(fs::read_dir(&receipt_root).map_err(io_error)?.count(), resolutions.receipts.len());
    assert!(stage.join(VERSION_PLAN_FILE).is_file());
    Ok(())
}

fn write_recheck_stage(
    stage: &Path,
    template_root: &Path,
    recheck_set: &VersionRecheckSet,
    manifests: Option<&Vec<(String, mantlepkgs_core::MantlepkgsManifest)>>,
) -> Result<(), RunError> {
    write_json_new(&stage.join(VERSION_RECHECKS_FILE), recheck_set)?;
    if let Some(manifests) = manifests {
        let manifest_root = stage.join(VERSION_MANIFESTS_DIRECTORY);
        fs::create_dir(&manifest_root)
            .map_err(|error| RunError::Internal(format!("creating version manifest directory: {error}")))?;
        for (group_identity, manifest) in manifests {
            write_manifest_group(&manifest_root, template_root, group_identity, manifest)?;
        }
        sync_directory(&manifest_root)?;
    }
    sync_directory(stage)?;
    assert!(stage.join(VERSION_RECHECKS_FILE).is_file());
    assert!(manifests.is_some() || !stage.join(VERSION_MANIFESTS_DIRECTORY).exists());
    Ok(())
}

fn write_manifest_group(
    manifest_root: &Path,
    template_root: &Path,
    group_identity: &str,
    manifest: &mantlepkgs_core::MantlepkgsManifest,
) -> Result<(), RunError> {
    let group_root = manifest_root.join(group_identity);
    fs::create_dir(&group_root)
        .map_err(|error| RunError::Internal(format!("creating version manifest group: {error}")))?;
    write_json_new(&group_root.join(VERSION_MANIFEST_JSON_FILE), manifest)?;
    write_bytes_new(&group_root.join(VERSION_MANIFEST_CONTRACT_FILE), VERSION_MANIFEST_CONTRACT.as_bytes())?;
    let nickel_manifest = render_manifest_nickel(manifest)?;
    write_bytes_new(&group_root.join(VERSION_MANIFEST_NICKEL_FILE), nickel_manifest.as_bytes())?;
    copy_manifest_policy_artifacts(template_root, &group_root, manifest)?;
    sync_directory(&group_root)?;
    assert!(group_root.join(VERSION_MANIFEST_JSON_FILE).is_file());
    assert!(group_root.join(VERSION_MANIFEST_NICKEL_FILE).is_file());
    Ok(())
}

fn copy_manifest_policy_artifacts(
    template_root: &Path,
    group_root: &Path,
    manifest: &MantlepkgsManifest,
) -> Result<(), RunError> {
    let artifacts = [
        &manifest.conversion_policy.translation_policy,
        &manifest.conversion_policy.execution_profile,
    ];
    for artifact in &artifacts {
        let source = template_root.join(&artifact.path);
        let metadata = fs::symlink_metadata(&source)
            .map_err(|error| RunError::Internal(format!("reading manifest policy {}: {error}", source.display())))?;
        if !metadata.is_file() || metadata.file_type().is_symlink() || metadata.len() > VERSION_INPUT_BYTES_MAX {
            return Err(RunError::Eval(format!(
                "manifest policy is not one bounded regular file: {}",
                source.display()
            )));
        }
        let bytes = fs::read(&source)
            .map_err(|error| RunError::Internal(format!("reading manifest policy {}: {error}", source.display())))?;
        let digest = blake3::hash(&bytes).to_hex().to_string();
        if digest != artifact.digest_blake3 {
            return Err(RunError::Eval(format!("manifest policy digest mismatch: {}", artifact.path)));
        }
        let destination = group_root.join(&artifact.path);
        if let Some(parent) = destination.parent() {
            fs::create_dir_all(parent)
                .map_err(|error| RunError::Internal(format!("creating manifest policy directory: {error}")))?;
        }
        write_or_confirm_policy(&destination, &bytes)?;
    }
    assert!(artifacts.iter().all(|artifact| group_root.join(&artifact.path).is_file()));
    assert!(template_root.is_dir());
    Ok(())
}

fn write_or_confirm_policy(destination: &Path, bytes: &[u8]) -> Result<(), RunError> {
    if destination.exists() {
        let existing = fs::read(destination)
            .map_err(|error| RunError::Internal(format!("reading generated manifest policy: {error}")))?;
        if existing != bytes {
            return Err(RunError::Eval("two manifest policies selected one path with different bytes".into()));
        }
        return Ok(());
    }
    write_bytes_new(destination, bytes)?;
    assert!(destination.is_file());
    assert!(u64::try_from(bytes.len()).unwrap_or(u64::MAX) <= VERSION_INPUT_BYTES_MAX);
    Ok(())
}

fn render_manifest_nickel(manifest: &MantlepkgsManifest) -> Result<String, RunError> {
    let selectors = manifest
        .selectors
        .iter()
        .map(render_selector_nickel)
        .collect::<Result<Vec<_>, _>>()?
        .join(",\n    ");
    Ok(format!(
        "let contracts = import \"contracts.ncl\" in\n{{\n  schema = {},\n  source = {{ reference = {}, revision = {}, lock_digest_blake3 = {} }},\n  systems = {},\n  selectors = [\n    {}\n  ],\n  conversion_policy = {{\n    mode = {},\n    target_store_prefix = {},\n    translation_policy = {{ path = {}, digest_blake3 = {} }},\n    execution_profile = {{ path = {}, digest_blake3 = {} }},\n  }},\n  source_policy = {{ mode = {}, optional_transports = {} }},\n  output = {{ generation_directory = {} }},\n  limits = {{\n    max_selectors = {},\n    max_graph_nodes = {},\n    max_graph_bytes = {},\n    max_source_requirements = {},\n    max_artifact_bytes = {},\n  }},\n}} | contracts.Manifest\n",
        nickel_string(&manifest.schema)?,
        nickel_string(&manifest.source.reference)?,
        nickel_string(&manifest.source.revision)?,
        nickel_string(&manifest.source.lock_digest_blake3)?,
        render_string_list_nickel(&manifest.systems)?,
        selectors,
        nickel_string(&manifest.conversion_policy.mode)?,
        nickel_string(&manifest.conversion_policy.target_store_prefix)?,
        nickel_string(&manifest.conversion_policy.translation_policy.path)?,
        nickel_string(&manifest.conversion_policy.translation_policy.digest_blake3)?,
        nickel_string(&manifest.conversion_policy.execution_profile.path)?,
        nickel_string(&manifest.conversion_policy.execution_profile.digest_blake3)?,
        nickel_string(&manifest.source_policy.mode)?,
        render_string_list_nickel(&manifest.source_policy.optional_transports)?,
        nickel_string(&manifest.output.generation_directory)?,
        manifest.limits.max_selectors,
        manifest.limits.max_graph_nodes,
        manifest.limits.max_graph_bytes,
        manifest.limits.max_source_requirements,
        manifest.limits.max_artifact_bytes,
    ))
}

fn render_selector_nickel(selector: &mantlepkgs_core::PackageSelector) -> Result<String, RunError> {
    Ok(format!(
        "{{ name = {}, attribute = {}, system = {}, aliases = {} }}",
        nickel_string(&selector.name)?,
        nickel_string(&selector.attribute)?,
        nickel_string(&selector.system)?,
        render_string_list_nickel(&selector.aliases)?,
    ))
}

fn render_string_list_nickel(values: &[String]) -> Result<String, RunError> {
    let items = values.iter().map(|value| nickel_string(value)).collect::<Result<Vec<_>, _>>()?;
    Ok(format!("[{}]", items.join(", ")))
}

fn nickel_string(value: &str) -> Result<String, RunError> {
    serde_json::to_string(value).map_err(|error| RunError::Internal(format!("encoding Nickel string: {error}")))
}

fn validate_nix_program(program: &Path) -> Result<(), RunError> {
    if !program.is_absolute() {
        return Err(RunError::Eval("--nix-program must be one exact absolute executable path".into()));
    }
    let metadata = fs::metadata(program)
        .map_err(|error| RunError::Internal(format!("reading Nix executable {}: {error}", program.display())))?;
    if !metadata.is_file() || metadata.len() > NIX_EXECUTABLE_BYTES_MAX {
        return Err(RunError::Eval("--nix-program must name one bounded regular file".into()));
    }
    if metadata.permissions().mode() & REQUIRED_OWNER_EXECUTE_BITS == 0 {
        return Err(RunError::Eval("--nix-program must be executable by its owner".into()));
    }
    let output = run_nix(program, &["--version"], VERSION_INDEX_STDOUT_BYTES_MAX)?;
    if !output.status.success() || output.stdout.is_empty() {
        return Err(RunError::Eval("--nix-program did not report a version".into()));
    }
    assert!(program.is_absolute());
    assert!(metadata.is_file());
    Ok(())
}

fn evaluate_nickel<T: DeserializeOwned>(path: &Path, kind: &str) -> Result<T, RunError> {
    let import_paths = path.parent().map(|parent| vec![OsString::from(parent)]).unwrap_or_default();
    let value = crunch_eval::evaluate_and_deserialize(path, &import_paths)
        .map_err(|error| RunError::Eval(format!("evaluating typed Mantlepkgs {kind} {}: {error}", path.display())))?;
    assert!(!kind.is_empty());
    assert!(!path.as_os_str().is_empty());
    Ok(value)
}

fn read_json_bounded<T: DeserializeOwned>(path: &Path) -> Result<T, RunError> {
    let metadata = fs::metadata(path)
        .map_err(|error| RunError::Internal(format!("reading version artifact {}: {error}", path.display())))?;
    if !metadata.is_file() || metadata.len() > VERSION_INPUT_BYTES_MAX {
        return Err(RunError::Eval(format!("version artifact is not one bounded regular file: {}", path.display())));
    }
    let bytes = fs::read(path)
        .map_err(|error| RunError::Internal(format!("reading version artifact {}: {error}", path.display())))?;
    let value = serde_json::from_slice(&bytes)
        .map_err(|error| RunError::Eval(format!("decoding version artifact {}: {error}", path.display())))?;
    assert_eq!(u64::try_from(bytes.len()).unwrap_or(u64::MAX), metadata.len());
    assert!(metadata.len() <= VERSION_INPUT_BYTES_MAX);
    Ok(value)
}

fn prepare_stage(output_root: &Path) -> Result<PathBuf, RunError> {
    if output_root.exists() {
        return Err(RunError::Eval(format!("version output root already exists: {}", output_root.display())));
    }
    let parent = output_root
        .parent()
        .ok_or_else(|| RunError::Eval("version output root must have a parent directory".into()))?;
    fs::create_dir_all(parent)
        .map_err(|error| RunError::Internal(format!("creating version output parent {}: {error}", parent.display())))?;
    let name = output_root
        .file_name()
        .ok_or_else(|| RunError::Eval("version output root must have a final component".into()))?
        .to_string_lossy();
    let stage = parent.join(format!(".{name}{STAGE_SEPARATOR}{}", std::process::id()));
    fs::create_dir(&stage)
        .map_err(|error| RunError::Internal(format!("creating version output stage {}: {error}", stage.display())))?;
    assert_ne!(stage, output_root);
    assert!(stage.is_dir());
    Ok(stage)
}

fn publish_stage(stage: &Path, output_root: &Path) -> Result<(), RunError> {
    sync_directory(stage)?;
    rename_path_no_replace(stage, output_root)
        .map_err(|error| RunError::Internal(format!("publishing version output {}: {error}", output_root.display())))?;
    let parent = output_root.parent().expect("validated output root has a parent");
    sync_directory(parent)?;
    assert!(output_root.is_dir());
    assert!(!stage.exists());
    Ok(())
}

fn cleanup_failed_stage(stage: &Path, failed: bool) {
    if failed && stage.exists() {
        let _ = fs::remove_dir_all(stage);
    }
    debug_assert!(!failed || !stage.exists());
    debug_assert!(failed || !stage.exists());
}

fn create_parent_directories(path: &Path) -> Result<(), RunError> {
    let parent = path.parent().ok_or_else(|| RunError::Eval("version artifact path must have a parent".into()))?;
    fs::create_dir_all(parent).map_err(|error| {
        RunError::Internal(format!("creating version artifact parent {}: {error}", parent.display()))
    })?;
    assert!(parent.is_dir());
    assert!(path.starts_with(parent));
    Ok(())
}

fn write_json_new<T: Serialize>(path: &Path, value: &T) -> Result<(), RunError> {
    let bytes = serde_json::to_vec_pretty(value)
        .map_err(|error| RunError::Internal(format!("serializing version artifact {}: {error}", path.display())))?;
    write_bytes_new(path, &bytes)
}

fn write_bytes_new(path: &Path, bytes: &[u8]) -> Result<(), RunError> {
    let mut file = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .map_err(|error| RunError::Internal(format!("creating version artifact {}: {error}", path.display())))?;
    file.write_all(bytes)
        .map_err(|error| RunError::Internal(format!("writing version artifact {}: {error}", path.display())))?;
    file.sync_all()
        .map_err(|error| RunError::Internal(format!("syncing version artifact {}: {error}", path.display())))?;
    assert!(path.is_file());
    assert_eq!(fs::metadata(path).map_err(io_error)?.len(), u64::try_from(bytes.len()).unwrap_or(u64::MAX));
    Ok(())
}

fn sync_directory(path: &Path) -> Result<(), RunError> {
    fs::File::open(path)
        .and_then(|directory| directory.sync_all())
        .map_err(|error| RunError::Internal(format!("syncing version directory {}: {error}", path.display())))?;
    assert!(path.is_dir());
    assert!(path.exists());
    Ok(())
}

fn emit_index_result(index: &VersionIndex, output_root: &Path, json: bool) -> Result<(), RunError> {
    if json {
        println!("{}", serde_json::to_string(index).map_err(json_error)?);
    } else {
        println!(
            "Mantlepkgs version index published: identity={} entries={} path={}",
            index.index_identity_blake3,
            index.entries.len(),
            output_root.display()
        );
    }
    Ok(())
}

fn emit_resolution_result(plan: &VersionProductionPlan, output_root: &Path, json: bool) -> Result<(), RunError> {
    if json {
        println!("{}", serde_json::to_string(plan).map_err(json_error)?);
    } else {
        println!(
            "Mantlepkgs version resolution published: identity={} groups={} blocked={} path={}",
            plan.plan_identity_blake3,
            plan.groups.len(),
            plan.blocked_receipt_identity_blake3.len(),
            output_root.display()
        );
    }
    Ok(())
}

fn emit_recheck_result(
    recheck_set: &VersionRecheckSet,
    manifests: &[(String, mantlepkgs_core::MantlepkgsManifest)],
    output_root: &Path,
    json: bool,
) -> Result<(), RunError> {
    if json {
        println!("{}", serde_json::to_string(recheck_set).map_err(json_error)?);
    } else {
        println!(
            "Mantlepkgs version recheck passed: identity={} manifests={} path={}",
            recheck_set.recheck_set_identity_blake3,
            manifests.len(),
            output_root.display()
        );
    }
    Ok(())
}

fn emit_recheck_blocked(recheck_set: &VersionRecheckSet, output_root: &Path, json: bool) -> Result<(), RunError> {
    if json {
        println!("{}", serde_json::to_string(recheck_set).map_err(json_error)?);
    } else {
        println!(
            "Mantlepkgs version recheck blocked: identity={} path={}",
            recheck_set.recheck_set_identity_blake3,
            output_root.display()
        );
    }
    Ok(())
}

fn json_error(error: serde_json::Error) -> RunError {
    RunError::Internal(format!("serializing version command output: {error}"))
}

fn io_error(error: std::io::Error) -> RunError {
    RunError::Internal(format!("reading version directory: {error}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    const REVISION: &str = "1111111111111111111111111111111111111111";
    const RESOLVED_REVISION: &str = "2222222222222222222222222222222222222222";
    const NAR_HASH: &str = "sha256-AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=";
    const TEST_OWNER_EXECUTABLE_MODE: u32 = 0o700;

    fn successful_output(bytes: &[u8]) -> Output {
        use std::os::unix::process::ExitStatusExt;
        Output {
            status: std::process::ExitStatus::from_raw(0),
            stdout: bytes.to_vec(),
            stderr: Vec::new(),
        }
    }

    #[test]
    fn metadata_parser_rejects_relative_source_path() {
        let bytes = format!(
            "{{\"path\":\"relative\",\"locked\":{{\"narHash\":\"{NAR_HASH}\",\"rev\":\"{REVISION}\",\"type\":\"github\"}}}}"
        );
        let error = parse_flake_metadata(bytes.as_bytes()).unwrap_err();
        assert!(matches!(error, RunError::Eval(_)));
    }

    #[test]
    fn version_output_preserves_success_unavailable_and_malformed_states() {
        let success = parse_version_output(&successful_output(br#"{"status":"success","version":"2.12.1"}"#)).unwrap();
        let unavailable =
            parse_version_output(&successful_output(br#"{"status":"version-unavailable","version":null}"#)).unwrap();
        let malformed =
            parse_version_output(&successful_output(br#"{"status":"malformed-version","version":null}"#)).unwrap();
        assert_eq!(success.status, VersionObservationStatus::Success);
        assert_eq!(success.version.as_deref(), Some("2.12.1"));
        assert_eq!(unavailable.status, VersionObservationStatus::Unavailable);
        assert_eq!(malformed.status, VersionObservationStatus::Failed);
    }

    #[test]
    fn version_output_rejects_unknown_and_incomplete_success() {
        let unknown =
            parse_version_output(&successful_output(br#"{"status":"nearby","version":"2.12.1"}"#)).unwrap_err();
        let incomplete =
            parse_version_output(&successful_output(br#"{"status":"success","version":null}"#)).unwrap_err();
        assert!(matches!(unknown, RunError::Eval(_)));
        assert!(matches!(incomplete, RunError::Eval(_)));
    }

    #[test]
    fn expression_uses_only_exact_reference_system_and_attribute_segments() {
        let reference = format!("github:NixOS/nixpkgs/{REVISION}");
        let expression = version_expression(&reference, "x86_64-linux", "python3.pkgs");
        assert!(expression.contains(&reference));
        assert!(expression.contains("legacyPackages.\"x86_64-linux\".\"python3\".\"pkgs\""));
        assert!(!expression.contains("builtins.parseDrvName"));
    }

    #[test]
    fn source_tree_identity_hashes_symlink_text_without_following_it() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().join("source");
        fs::create_dir(&root).unwrap();
        fs::write(root.join("file"), b"content\n").unwrap();
        std::os::unix::fs::symlink("../../../outside", root.join("link")).unwrap();
        let first = hash_nixpkgs_source_tree(&root).unwrap();
        fs::remove_file(root.join("link")).unwrap();
        std::os::unix::fs::symlink("../../../different", root.join("link")).unwrap();
        let second = hash_nixpkgs_source_tree(&root).unwrap();
        assert_ne!(first, second);
    }

    #[test]
    fn source_tree_identity_rejects_special_files() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().join("source");
        fs::create_dir(&root).unwrap();
        let _socket = std::os::unix::net::UnixListener::bind(root.join("socket")).unwrap();
        let error = hash_nixpkgs_source_tree(&root).unwrap_err();
        assert!(matches!(error, RunError::Eval(_)));
    }

    #[test]
    fn oversized_producer_capture_fails_closed() {
        let mut capture = tempfile::tempfile().unwrap();
        let oversized_bytes = VERSION_INDEX_STDOUT_BYTES_MAX.checked_add(1).unwrap();
        capture.set_len(u64::try_from(oversized_bytes).unwrap()).unwrap();
        let error =
            crate::mantlepkgs_cmd::read_capture(&mut capture, VERSION_INDEX_STDOUT_BYTES_MAX, "stdout").unwrap_err();
        assert!(matches!(error, RunError::Internal(_)));
    }

    #[test]
    fn atomic_stage_rejects_existing_output() {
        let temp = tempfile::tempdir().unwrap();
        let output = temp.path().join("output");
        fs::create_dir(&output).unwrap();
        let error = prepare_stage(&output).unwrap_err();
        assert!(matches!(error, RunError::Eval(_)));
        assert!(output.is_dir());
    }

    #[test]
    fn published_artifact_readback_rejects_modified_and_missing_bytes() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("published.json");
        fs::write(&path, br#"{"version":"wrong"}"#).unwrap();
        let expected = serde_json::json!({"version": "expected"});
        let mismatch = verify_published(&path, &expected).unwrap_err();
        assert!(matches!(mismatch, RunError::Eval(_)));
        fs::remove_file(&path).unwrap();
        let missing = verify_published(&path, &expected).unwrap_err();
        assert!(matches!(missing, RunError::Internal(_)));
    }

    #[test]
    fn negative_contract_fixtures_fail_closed() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("mantlepkgs/versions/fixtures");
        let fixtures = [
            "invalid-floating-cohort.ncl",
            "invalid-limit.ncl",
            "invalid-unknown-method.ncl",
            "invalid-unsafe-path.ncl",
            "invalid-wrong-system.ncl",
        ];
        for fixture in fixtures {
            let error = evaluate_nickel::<serde_json::Value>(&root.join(fixture), "negative contract fixture")
                .expect_err("negative fixture must fail");
            assert!(matches!(error, RunError::Eval(_)));
        }
    }

    #[test]
    fn typed_duplicate_fixture_reaches_core_and_fails_closed() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"));
        let fixture = root.join("mantlepkgs/versions/fixtures/invalid-duplicate-key.ncl");
        let cohort = evaluate_nickel::<VersionCohort>(&fixture, "duplicate fixture").unwrap();
        let error = seal_version_cohort(&cohort).unwrap_err();
        assert!(error.diagnostics.iter().any(|item| item.code == "duplicate-published-order"));
        assert!(error.diagnostics.iter().any(|item| item.code == "duplicate-version-revision"));
    }

    #[test]
    fn invalid_artifacts_fail_before_nix_execution() {
        let temp = tempfile::tempdir().unwrap();
        let (nix, signal) = signaling_nix(temp.path());
        let root = Path::new(env!("CARGO_MANIFEST_DIR"));
        let invalid_cohort = root.join("mantlepkgs/versions/fixtures/invalid-floating-cohort.ncl");
        let index_output = temp.path().join("invalid-index-output");
        let index_error = run_index(&invalid_cohort, &nix, &index_output, false).unwrap_err();
        assert!(matches!(index_error, RunError::Eval(_)));
        assert!(!signal.exists());
        assert!(!index_output.exists());

        let invalid_plan = temp.path().join("invalid-plan.json");
        fs::write(&invalid_plan, b"{}\n").unwrap();
        let template = root.join("mantlepkgs/live-cohort/manifest.ncl");
        let recheck_output = temp.path().join("invalid-recheck-output");
        let recheck_error = run_recheck(&invalid_plan, &template, &nix, &recheck_output, false).unwrap_err();
        assert!(matches!(recheck_error, RunError::Eval(_)));
        assert!(!signal.exists());
        assert!(!recheck_output.exists());
    }

    #[test]
    fn fake_nix_runs_index_resolution_recheck_and_manifest_emission() {
        let temp = tempfile::tempdir().unwrap();
        let source = temp.path().join("source");
        fs::create_dir(&source).unwrap();
        fs::write(source.join("README"), b"fixed source\n").unwrap();
        let nix = fake_nix(temp.path(), &source, true);
        let root = Path::new(env!("CARGO_MANIFEST_DIR"));
        let cohort_path = root.join("mantlepkgs/versions/fixtures/valid-cohort.ncl");
        let index_root = temp.path().join("index-output");
        run_index(&cohort_path, &nix, &index_root, false).unwrap();

        let cohort = read_json_bounded::<VersionCohort>(&index_root.join(VERSION_COHORT_FILE)).unwrap();
        let observations =
            read_json_bounded::<VersionObservationSet>(&index_root.join(VERSION_OBSERVATIONS_FILE)).unwrap();
        let index_path = index_root.join(&cohort.index_relative_path);
        let index = read_json_bounded::<VersionIndex>(&index_path).unwrap();
        assert_eq!(index.entries.len(), 1);
        assert_eq!(index.entries[0].revision, RESOLVED_REVISION);

        let policy = seal_version_selection_policy(&VersionSelectionPolicy {
            schema: mantlepkgs_core::VERSION_SELECTION_POLICY_SCHEMA.into(),
            policy_identity_blake3: String::new(),
            method: mantlepkgs_core::VERSION_SELECTION_METHOD_NEWEST.into(),
            system: cohort.system.clone(),
            accepted_cohort_identity_blake3: cohort.cohort_identity_blake3.clone(),
            accepted_observation_set_identity_blake3: observations.observation_set_identity_blake3,
            accepted_index_identity_blake3: index.index_identity_blake3,
            limits: cohort.limits.clone(),
        })
        .unwrap();
        let requests = seal_version_request_set(&policy, &VersionRequestSet {
            schema: mantlepkgs_core::VERSION_REQUEST_SET_SCHEMA.into(),
            request_set_identity_blake3: String::new(),
            policy_identity_blake3: policy.policy_identity_blake3.clone(),
            requests: vec![mantlepkgs_core::VersionRequest {
                request_identity_blake3: String::new(),
                system: cohort.system,
                attribute: "hello".into(),
                reported_version: "2.12.1".into(),
                public_selector: "hello@2.12.1".into(),
                aliases: vec!["hello-2.12.1".into()],
                unversioned_default: false,
            }],
        })
        .unwrap();
        let mut policy_template = policy;
        policy_template.policy_identity_blake3.clear();
        policy_template.accepted_cohort_identity_blake3.clear();
        policy_template.accepted_observation_set_identity_blake3.clear();
        policy_template.accepted_index_identity_blake3.clear();
        let mut request_template = requests;
        request_template.request_set_identity_blake3.clear();
        request_template.policy_identity_blake3.clear();
        let policy_path = write_nickel_import(temp.path(), "policy", &policy_template);
        let requests_path = write_nickel_import(temp.path(), "requests", &request_template);
        let resolve_root = temp.path().join("resolve-output");
        run_resolve(&index_path, &policy_path, &requests_path, &resolve_root, false).unwrap();
        let plan_path = resolve_root.join(VERSION_PLAN_FILE);
        let plan = read_json_bounded::<VersionProductionPlan>(&plan_path).unwrap();
        assert_eq!(plan.groups.len(), 1);
        assert!(plan.blocked_receipt_identity_blake3.is_empty());

        let template = root.join("mantlepkgs/live-cohort/manifest.ncl");
        let recheck_root = temp.path().join("recheck-output");
        run_recheck(&plan_path, &template, &nix, &recheck_root, false).unwrap();
        let generated_root = recheck_root.join(VERSION_MANIFESTS_DIRECTORY).join(&plan.groups[0].group_identity_blake3);
        let manifest = generated_root.join(VERSION_MANIFEST_NICKEL_FILE);
        assert!(manifest.is_file());
        let generated: MantlepkgsManifest = evaluate_nickel(&manifest, "generated manifest").unwrap();
        assert_eq!(generated.source.revision, RESOLVED_REVISION);
        assert!(generated_root.join(&generated.conversion_policy.translation_policy.path).is_file());
        assert!(generated_root.join(&generated.conversion_policy.execution_profile.path).is_file());
        assert!(recheck_root.join(VERSION_RECHECKS_FILE).is_file());
    }

    #[test]
    fn metadata_hash_mismatch_records_failures_without_success_entries() {
        let temp = tempfile::tempdir().unwrap();
        let source = temp.path().join("source");
        fs::create_dir(&source).unwrap();
        fs::write(source.join("README"), b"fixed source\n").unwrap();
        let nix = fake_nix(temp.path(), &source, false);
        let root = Path::new(env!("CARGO_MANIFEST_DIR"));
        let cohort_path = root.join("mantlepkgs/versions/fixtures/valid-cohort.ncl");
        let index_root = temp.path().join("index-output");
        run_index(&cohort_path, &nix, &index_root, false).unwrap();
        let index = read_json_bounded::<VersionIndex>(&index_root.join("indexes/x86_64-linux.json")).unwrap();
        assert!(index.entries.is_empty());
        assert_eq!(index.failed_observations, 2);
    }

    fn write_nickel_import<T: Serialize>(root: &Path, name: &str, value: &T) -> PathBuf {
        let json = root.join(format!("{name}.json"));
        let nickel = root.join(format!("{name}.ncl"));
        write_json_new(&json, value).unwrap();
        write_bytes_new(&nickel, format!("import \"{name}.json\"\n").as_bytes()).unwrap();
        assert!(json.is_file());
        assert!(nickel.is_file());
        nickel
    }

    fn signaling_nix(root: &Path) -> (PathBuf, PathBuf) {
        let nix = root.join("signaling-nix");
        let signal = root.join("nix-launched");
        let script = r#"#!/bin/sh
: > "${0%/*}/nix-launched"
printf '%s\n' 'nix (signaling fake) 1.0'
"#;
        fs::write(&nix, script).unwrap();
        let mut permissions = fs::metadata(&nix).unwrap().permissions();
        permissions.set_mode(TEST_OWNER_EXECUTABLE_MODE);
        fs::set_permissions(&nix, permissions).unwrap();
        assert!(nix.is_file());
        assert!(!signal.exists());
        (nix, signal)
    }

    fn fake_nix(root: &Path, source: &Path, matching_hashes: bool) -> PathBuf {
        let nix = root.join("fake-nix");
        let fallback_hash = "sha256-CCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCC=";
        let old_hash = if matching_hashes { NAR_HASH } else { fallback_hash };
        let new_hash = if matching_hashes {
            "sha256-BBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBB="
        } else {
            fallback_hash
        };
        let script = format!(
            r#"#!/bin/sh
case "$*" in
  *--version*) printf '%s\n' 'nix (fake) 1.0' ;;
  *metadata*1111111111111111111111111111111111111111*) printf '%s\n' '{{"path":"{}","locked":{{"narHash":"{}","rev":"1111111111111111111111111111111111111111","type":"github"}}}}' ;;
  *metadata*2222222222222222222222222222222222222222*) printf '%s\n' '{{"path":"{}","locked":{{"narHash":"{}","rev":"2222222222222222222222222222222222222222","type":"github"}}}}' ;;
  *archive*) printf '%s\n' '{{"inputs":{{}},"path":"{}"}}' ;;
  *eval*) printf '%s\n' '{{"status":"success","version":"2.12.1"}}' ;;
  *) exit 2 ;;
esac
"#,
            source.display(),
            old_hash,
            source.display(),
            new_hash,
            source.display(),
        );
        fs::write(&nix, script).unwrap();
        let mut permissions = fs::metadata(&nix).unwrap().permissions();
        permissions.set_mode(TEST_OWNER_EXECUTABLE_MODE);
        fs::set_permissions(&nix, permissions).unwrap();
        assert!(nix.is_file());
        assert_ne!(fs::metadata(&nix).unwrap().permissions().mode() & REQUIRED_OWNER_EXECUTE_BITS, 0);
        nix
    }
}
