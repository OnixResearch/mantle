// machine-artifact-public: nix-free-demo.reports
use std::fs;
use std::path::Path;
use std::path::PathBuf;

use clap::Subcommand;
use mantle_application_contract::ApplicationOutcome;
use mantle_application_contract::CapabilityError;
use mantle_application_contract::EffectPlan;
use mantle_application_contract::NIX_FREE_DEMO_BUNDLE_DOCUMENT_COUNT;
use mantle_application_contract::NixFreeDemoBundleFacts;
use mantle_application_contract::NixFreeDemoBundlePort;
use mantle_application_contract::NixFreeDemoBundleWrite;
use mantle_application_contract::NixFreeDemoInputFacts;
use mantle_application_contract::NixFreeDemoOutputState;
use mantle_application_contract::NixFreeDemoSummaryPort;
use mantle_application_contract::NixFreeDemoTranscriptCopy;
use mantle_application_contract::NixFreeDemoTranscriptRead;
use mantle_application_contract::classify_nix_free_demo_generate;
use mantle_application_contract::classify_nix_free_demo_rejection;
use mantle_application_contract::classify_nix_free_demo_summary_read;
use mantle_application_contract::nix_free_demo_generate_effect_plan;
use mantle_application_contract::nix_free_demo_summary_effect_plan;
use serde::Serialize;

use crate::errors::RunError;
use crate::nix_free_demo_bundle::NixFreeDemoDiagnostic;
use crate::nix_free_demo_bundle::NixFreeDemoEvidenceRef;
use crate::nix_free_demo_bundle::NixFreeDemoGeneratedBundle;
use crate::nix_free_demo_bundle::NixFreeDemoGuardEvidence;
use crate::nix_free_demo_bundle::NixFreeDemoMachineSummary;
use crate::nix_free_demo_bundle::NixFreeDemoManifestInput;
use crate::nix_free_demo_bundle::NixFreeDemoNamedDigest;
use crate::nix_free_demo_bundle::NixFreeDemoValidation;
use crate::nix_free_demo_bundle::build_nix_free_demo_bundle;
use crate::nix_free_demo_bundle::is_blake3_hex;
use crate::nix_free_demo_bundle::render_nix_free_demo_readme;
use crate::nix_free_demo_bundle::validate_nix_free_demo_bundle;

const CLI_REPORT_SCHEMA: &str = "mantle-nix-free-demo-cli-v1";
const GENERATE_REPORT_SCHEMA: &str = "mantle-nix-free-demo-generate-cli-v1";
const UNKNOWN_PROFILE: &str = "unknown";
const MALFORMED_VERDICT: &str = "malformed";
const MALFORMED_SUMMARY_CODE: &str = "malformed-summary";
const NON_CLAIMABLE_EXIT_CODE: u8 = 1;
const GENERATE_FAILURE_EXIT_CODE: u8 = 1;
const JSON_SERIALIZATION_CONTEXT: &str = "serializing Nix-free demo CLI report";
const SUMMARY_FILE_NAME: &str = "summary.json";
const README_FILE_NAME: &str = "README.md";
const MANIFEST_FILE_NAME: &str = "manifest.json";
const VALIDATION_FILE_NAME: &str = "validation.json";
const TRANSCRIPTS_DIR: &str = "transcripts";
const DIGEST_SEPARATOR: char = ':';
const DIGEST_PART_COUNT: usize = 2;
const GUARD_PART_COUNT: usize = 3;

const _: () = assert!(DIGEST_PART_COUNT > 0);

// CLI variants retain complete clap payloads to preserve flag and help compatibility.
#[allow(clippy::large_enum_variant)]
#[derive(Subcommand, Debug, Clone)]
pub(crate) enum NixFreeDemoAction {
    /// Validate a Nix-free demo bundle machine summary JSON file
    Validate {
        /// Path to `mantle-nix-free-demo-summary-v1` JSON
        summary: PathBuf,
    },

    /// Render the operator README from a Nix-free demo bundle machine summary JSON file
    Readme {
        /// Path to `mantle-nix-free-demo-summary-v1` JSON
        summary: PathBuf,
    },

    /// Generate a self-contained Nix-free demo bundle from explicit evidence inputs
    Generate {
        /// Output directory to create or reuse if empty
        #[arg(long)]
        out: PathBuf,

        /// Proof status, for example `success` or `blocked`
        #[arg(long = "proof-status")]
        proof_status: String,

        /// Source-root identity recorded in the generated summary
        #[arg(long = "source-root-identity")]
        source_root_identity: String,

        /// BLAKE3 digest of the toolchain policy
        #[arg(long = "toolchain-policy-digest-blake3")]
        toolchain_policy_digest_blake3: String,

        /// Optional stage1 Mantle binary BLAKE3 digest
        #[arg(long = "stage1-binary-blake3")]
        stage1_binary_blake3: Option<String>,

        /// Optional stage2 Mantle binary BLAKE3 digest
        #[arg(long = "stage2-binary-blake3")]
        stage2_binary_blake3: Option<String>,

        /// Guard evidence as `guard:status:diagnostic`; repeat for each guard
        #[arg(long = "guard")]
        guards: Vec<String>,

        /// Transcript file to copy into the bundle; repeat for each transcript
        #[arg(long = "transcript")]
        transcripts: Vec<PathBuf>,

        /// Receipt digest as `name:blake3`; repeat for each receipt
        #[arg(long = "receipt-digest")]
        receipt_digests: Vec<String>,

        /// Artifact digest as `name:blake3`; repeat for each artifact
        #[arg(long = "artifact-digest")]
        artifact_digests: Vec<String>,

        /// Replay hint to include in the summary; repeat for each hint
        #[arg(long = "replay-hint")]
        replay_hints: Vec<String>,

        /// Explicit non-claim text; repeat for each non-claim
        #[arg(long = "non-claim")]
        non_claims: Vec<String>,

        /// Mark the generated evidence as synthetic or fixture-derived
        #[arg(long)]
        synthetic: bool,
    },
}

#[derive(Debug, Serialize, PartialEq, Eq)]
struct NixFreeDemoCliReport {
    schema: String,
    profile: String,
    verdict: String,
    demo_claimable: bool,
    diagnostics: Vec<NixFreeDemoDiagnostic>,
}

#[derive(Debug, Serialize, PartialEq, Eq)]
struct NixFreeDemoGenerateReport {
    schema: String,
    generated: bool,
    bundle_dir: String,
    demo_claimable: bool,
    diagnostics: Vec<NixFreeDemoDiagnostic>,
    files: Vec<String>,
}

#[derive(Debug)]
struct GenerateOptions<'a> {
    out: &'a Path,
    proof_status: String,
    stage1_binary_blake3: Option<String>,
    stage2_binary_blake3: Option<String>,
    source_root_identity: String,
    toolchain_policy_digest_blake3: String,
    guards: Vec<String>,
    transcripts: Vec<PathBuf>,
    receipt_digests: Vec<String>,
    artifact_digests: Vec<String>,
    replay_hints: Vec<String>,
    non_claims: Vec<String>,
    synthetic: bool,
}

#[derive(Debug)]
struct PreparedGeneratedBundle {
    bundle: NixFreeDemoGeneratedBundle,
    transcript_copies: Vec<NixFreeDemoTranscriptCopy>,
    files: Vec<String>,
}

/// Transcript evidence admitted from the declared transcripts in declaration order.
#[derive(Debug)]
struct AdmittedTranscripts {
    refs: Vec<NixFreeDemoEvidenceRef>,
    copies: Vec<NixFreeDemoTranscriptCopy>,
}

/// Bundle documents serialized before the write effect runs.
#[derive(Debug)]
struct BundleDocuments {
    summary: Vec<u8>,
    manifest: Vec<u8>,
    validation: Vec<u8>,
}

/// Diagnostic code of a bundle write the filesystem refused.
const BUNDLE_WRITE_FAILED_CODE: &str = "nix-free-demo-bundle-write-failed";

/// Filesystem adapter scoped to one generate request's declared paths.
///
/// It reaches only the named output directory and the declared transcripts.
struct GenerateBundleFiles<'a> {
    out: &'a Path,
    transcripts: &'a [PathBuf],
}

pub(crate) fn cmd_nix_free_demo(action: NixFreeDemoAction, json: bool) -> Result<(), RunError> {
    match action {
        NixFreeDemoAction::Validate { summary } => run_validate(&summary, json),
        NixFreeDemoAction::Readme { summary } => run_readme(&summary, json),
        NixFreeDemoAction::Generate {
            out,
            proof_status,
            source_root_identity,
            toolchain_policy_digest_blake3,
            stage1_binary_blake3,
            stage2_binary_blake3,
            guards,
            transcripts,
            receipt_digests,
            artifact_digests,
            replay_hints,
            non_claims,
            synthetic,
        } => run_generate(
            GenerateOptions {
                out: &out,
                proof_status,
                stage1_binary_blake3,
                stage2_binary_blake3,
                source_root_identity,
                toolchain_policy_digest_blake3,
                guards,
                transcripts,
                receipt_digests,
                artifact_digests,
                replay_hints,
                non_claims,
                synthetic,
            },
            json,
        ),
    }
}

fn run_validate(summary_path: &Path, json: bool) -> Result<(), RunError> {
    let contents = read_summary(summary_path)?;
    let (summary, validation) = match parse_and_validate_summary(&contents) {
        Ok(pair) => pair,
        Err(report) => return emit_report(report, json),
    };
    let command_output = report_for_summary(&summary, &validation);
    emit_report(command_output, json)
}

fn run_readme(summary_path: &Path, _json: bool) -> Result<(), RunError> {
    let contents = read_summary(summary_path)?;
    let (summary, validation) = match parse_and_validate_summary(&contents) {
        Ok(pair) => pair,
        Err(report) => return emit_report(report, false),
    };
    let readme = render_nix_free_demo_readme(&summary, &validation);
    println!("{readme}");
    if validation.demo_claimable {
        Ok(())
    } else {
        Err(RunError::Reported(NON_CLAIMABLE_EXIT_CODE))
    }
}

/// Plan the generation, then run its read and write through the bundle port.
///
/// The plan exists before the port is first called. The read records what it
/// observed as it runs. A rejected request stops before the write: its read is
/// classified with the write skipped, then it reports as it always did. An
/// admitted bundle is classified from the executed write before anything is
/// reported.
fn run_generate(options: GenerateOptions<'_>, json: bool) -> Result<(), RunError> {
    let plan = nix_free_demo_generate_effect_plan().map_err(capability_failure)?;
    let mut port = GenerateBundleFiles {
        out: options.out,
        transcripts: &options.transcripts,
    };
    let mut inputs = NixFreeDemoInputFacts::default();
    let prepared = match prepare_generate_bundle(&options, &mut port, &mut inputs) {
        Ok(prepared) => prepared,
        Err(diagnostic) => {
            return emit_rejected_generation(&plan, &inputs, rejected_generate_report(options.out, diagnostic), json);
        }
    };
    debug_assert!(inputs.output_free && !inputs.input_unavailable);
    debug_assert_eq!(prepared.transcript_copies.len(), options.transcripts.len());
    let documents = render_bundle_documents(&prepared.bundle)?;
    let bundle = NixFreeDemoBundleWrite {
        transcripts: &prepared.transcript_copies,
        summary_json: &documents.summary,
        manifest_json: &documents.manifest,
        validation_json: &documents.validation,
        readme: &prepared.bundle.readme,
    };
    let written = port.write_bundle(&bundle);
    match classify_nix_free_demo_generate(&plan, &inputs, &bundle, &written) {
        ApplicationOutcome::Completed => emit_generate_report(generated_report(options.out, &prepared), json),
        ApplicationOutcome::Failed { .. } => Err(generate_failure(&inputs, &bundle, written)),
        other => Err(RunError::Internal(format!("Nix-free demo generate observations were inconsistent: {other:?}"))),
    }
}

/// Report a rejected generation once its observations classify it as incomplete.
fn emit_rejected_generation(
    plan: &EffectPlan,
    inputs: &NixFreeDemoInputFacts,
    report: NixFreeDemoGenerateReport,
    json: bool,
) -> Result<(), RunError> {
    debug_assert!(!report.generated);
    debug_assert!(report.files.is_empty());
    match classify_nix_free_demo_rejection(plan, inputs) {
        ApplicationOutcome::Failed { .. } => emit_generate_report(report, json),
        other => Err(RunError::Internal(format!(
            "Nix-free demo generate rejection observations were inconsistent: {other:?}"
        ))),
    }
}

/// The exact error a failed generation reports; a refused write keeps its own message.
fn generate_failure(
    inputs: &NixFreeDemoInputFacts,
    bundle: &NixFreeDemoBundleWrite<'_>,
    written: Result<NixFreeDemoBundleFacts, CapabilityError>,
) -> RunError {
    match written {
        Err(error) => RunError::Internal(error.detail),
        Ok(facts) => RunError::Internal(format!(
            "Nix-free demo bundle write did not match its admitted inputs: read {} and copied {} of {} planned \
             transcript(s), wrote {} of {NIX_FREE_DEMO_BUNDLE_DOCUMENT_COUNT} document(s)",
            inputs.transcripts_read,
            facts.transcripts_copied,
            bundle.transcripts.len(),
            facts.documents_written,
        )),
    }
}

fn capability_failure(error: CapabilityError) -> RunError {
    RunError::Internal(format!("{}: {}", error.code, error.detail))
}

/// Plan the summary read, run it through the summary port, and classify it.
///
/// `validate` and `readme` report only after this returns: a read the port
/// refused fails with its own message before anything is printed.
fn read_summary(summary_path: &Path) -> Result<String, RunError> {
    let plan = nix_free_demo_summary_effect_plan().map_err(capability_failure)?;
    let read = SummaryFile { path: summary_path }.read_summary();
    match (classify_nix_free_demo_summary_read(&plan, &read), read) {
        (ApplicationOutcome::Completed, Ok(contents)) => Ok(contents),
        (ApplicationOutcome::Failed { .. }, Err(error)) => Err(RunError::Internal(error.detail)),
        (other, _) => {
            Err(RunError::Internal(format!("Nix-free demo summary observations were inconsistent: {other:?}")))
        }
    }
}

/// Diagnostic code of a summary read the filesystem refused.
const SUMMARY_READ_FAILED_CODE: &str = "nix-free-demo-summary-read-failed";

/// Filesystem adapter scoped to the one summary path the operator named.
struct SummaryFile<'a> {
    path: &'a Path,
}

impl NixFreeDemoSummaryPort for SummaryFile<'_> {
    fn read_summary(&mut self) -> Result<String, CapabilityError> {
        debug_assert!(!SUMMARY_READ_FAILED_CODE.is_empty());
        fs::read_to_string(self.path).map_err(|error| CapabilityError {
            code: SUMMARY_READ_FAILED_CODE.to_string(),
            detail: format!("reading Nix-free demo summary {}: {error}", self.path.display()),
        })
    }
}

fn parse_and_validate_summary(
    contents: &str,
) -> Result<(NixFreeDemoMachineSummary, NixFreeDemoValidation), NixFreeDemoCliReport> {
    match serde_json::from_str::<NixFreeDemoMachineSummary>(contents) {
        Ok(summary) => {
            let validation = validate_nix_free_demo_bundle(&summary);
            Ok((summary, validation))
        }
        Err(error) => Err(malformed_report(&error.to_string())),
    }
}

/// Read the declared inputs through the port and admit the request.
///
/// `inputs` records what the read observed as it runs, so a rejection can be
/// classified from the read that actually happened.
fn prepare_generate_bundle(
    options: &GenerateOptions<'_>,
    port: &mut impl NixFreeDemoBundlePort,
    inputs: &mut NixFreeDemoInputFacts,
) -> Result<PreparedGeneratedBundle, NixFreeDemoDiagnostic> {
    debug_assert!(!SUMMARY_FILE_NAME.is_empty());
    debug_assert!(!MANIFEST_FILE_NAME.is_empty());
    let output = port.probe_output();
    inputs.output_free = output.is_free();
    inputs.input_unavailable = matches!(output, NixFreeDemoOutputState::Unreadable { .. });
    admit_output(&output)?;
    let transcripts = transcript_evidence(&options.transcripts, port, inputs)?;
    let input = NixFreeDemoManifestInput {
        proof_status: options.proof_status.clone(),
        stage1_binary_blake3: options.stage1_binary_blake3.clone(),
        stage2_binary_blake3: options.stage2_binary_blake3.clone(),
        source_root_identity: options.source_root_identity.clone(),
        toolchain_policy_digest_blake3: options.toolchain_policy_digest_blake3.clone(),
        command_owned_wrappers: Vec::new(),
        guards: parse_guards(&options.guards)?,
        replay_hints: options.replay_hints.clone(),
        non_claims: options.non_claims.clone(),
        transcripts: transcripts.refs,
        receipt_digests: parse_named_digests(&options.receipt_digests, "receipt-digest")?,
        artifact_digests: parse_named_digests(&options.artifact_digests, "artifact-digest")?,
        synthetic: options.synthetic,
    };
    let bundle = build_nix_free_demo_bundle(input)?;
    let files = generated_files(&bundle);
    Ok(PreparedGeneratedBundle {
        bundle,
        transcript_copies: transcripts.copies,
        files,
    })
}

/// Admit the observed output directory: only an absent or empty one may receive a bundle.
fn admit_output(output: &NixFreeDemoOutputState) -> Result<(), NixFreeDemoDiagnostic> {
    match output {
        NixFreeDemoOutputState::Absent | NixFreeDemoOutputState::Empty => Ok(()),
        NixFreeDemoOutputState::Occupied => Err(diagnostic("output-conflict", "output directory is not empty")),
        NixFreeDemoOutputState::Unreadable { detail } => {
            Err(diagnostic("output-conflict", format!("read output dir: {detail}")))
        }
    }
}

/// Read the declared transcripts in declaration order and admit their evidence.
///
/// The first transcript that cannot be read or has no UTF-8 file name rejects
/// the request, and no later transcript is read. Every read is recorded in
/// `inputs` as it happens.
fn transcript_evidence(
    paths: &[PathBuf],
    port: &mut impl NixFreeDemoBundlePort,
    inputs: &mut NixFreeDemoInputFacts,
) -> Result<AdmittedTranscripts, NixFreeDemoDiagnostic> {
    debug_assert!(!TRANSCRIPTS_DIR.is_empty());
    let mut refs = Vec::with_capacity(paths.len());
    let mut copies = Vec::with_capacity(paths.len());
    for (source, transcript_index) in paths.iter().zip(0_u32..) {
        let bytes = match port.read_transcript(transcript_index) {
            NixFreeDemoTranscriptRead::Read(bytes) => bytes,
            NixFreeDemoTranscriptRead::Unavailable { detail } => {
                inputs.input_unavailable = true;
                return Err(diagnostic("missing-transcript", format!("{}: {detail}", source.display())));
            }
        };
        inputs.transcripts_read = inputs.transcripts_read.saturating_add(1);
        let file_name = source
            .file_name()
            .and_then(|name| name.to_str())
            .ok_or_else(|| diagnostic("missing-transcript", "transcript path must have a UTF-8 file name"))?;
        let bundle_path = format!("{TRANSCRIPTS_DIR}/{file_name}");
        refs.push(NixFreeDemoEvidenceRef {
            name: file_name.to_string(),
            digest_blake3: blake3::hash(&bytes).to_hex().to_string(),
            bundle_path: bundle_path.clone(),
        });
        copies.push(NixFreeDemoTranscriptCopy {
            transcript_index,
            bundle_path,
        });
    }
    debug_assert_eq!(refs.len(), paths.len());
    debug_assert_eq!(copies.len(), paths.len());
    Ok(AdmittedTranscripts { refs, copies })
}

fn parse_named_digests(values: &[String], code: &str) -> Result<Vec<NixFreeDemoNamedDigest>, NixFreeDemoDiagnostic> {
    values.iter().map(|value| parse_named_digest(value, code)).collect::<Result<Vec<_>, _>>()
}

fn parse_named_digest(value: &str, code: impl AsRef<str>) -> Result<NixFreeDemoNamedDigest, NixFreeDemoDiagnostic> {
    let code = code.as_ref();
    let parts = value.splitn(DIGEST_PART_COUNT, DIGEST_SEPARATOR).collect::<Vec<_>>();
    if parts.len() != DIGEST_PART_COUNT || parts[0].trim().is_empty() || !is_blake3_hex(parts[1]) {
        return Err(diagnostic(code, "expected name:blake3"));
    }
    Ok(NixFreeDemoNamedDigest {
        name: parts[0].to_string(),
        digest_blake3: parts[1].to_string(),
    })
}

fn parse_guards(values: &[String]) -> Result<Vec<NixFreeDemoGuardEvidence>, NixFreeDemoDiagnostic> {
    values.iter().map(|value| parse_guard(value)).collect::<Result<Vec<_>, _>>()
}

fn parse_guard(value: &str) -> Result<NixFreeDemoGuardEvidence, NixFreeDemoDiagnostic> {
    let parts = value.splitn(GUARD_PART_COUNT, DIGEST_SEPARATOR).collect::<Vec<_>>();
    if parts.len() != GUARD_PART_COUNT || parts[0].trim().is_empty() || parts[1].trim().is_empty() {
        return Err(diagnostic("guard-evidence", "expected guard:status:diagnostic"));
    }
    Ok(NixFreeDemoGuardEvidence {
        guard: parts[0].to_string(),
        status: parts[1].to_string(),
        diagnostic: parts[2].to_string(),
    })
}

/// Serialize the bundle documents before the write effect, so a write never starts half-rendered.
fn render_bundle_documents(bundle: &NixFreeDemoGeneratedBundle) -> Result<BundleDocuments, RunError> {
    let documents = BundleDocuments {
        summary: json_document(&bundle.summary)?,
        manifest: json_document(&bundle.manifest)?,
        validation: json_document(&bundle.validation)?,
    };
    debug_assert!(!documents.summary.is_empty() && !documents.manifest.is_empty());
    debug_assert!(!documents.validation.is_empty());
    Ok(documents)
}

fn json_document(value: &impl Serialize) -> Result<Vec<u8>, RunError> {
    serde_json::to_vec_pretty(value)
        .map_err(|error| RunError::Internal(format!("{JSON_SERIALIZATION_CONTEXT}: {error}")))
}

impl GenerateBundleFiles<'_> {
    /// The declared transcript at `transcript_index`, if the operator declared one there.
    fn transcript(&self, transcript_index: u32) -> Option<&Path> {
        let index = usize::try_from(transcript_index).ok()?;
        self.transcripts.get(index).map(PathBuf::as_path)
    }
}

impl NixFreeDemoBundlePort for GenerateBundleFiles<'_> {
    fn probe_output(&mut self) -> NixFreeDemoOutputState {
        if !self.out.exists() {
            return NixFreeDemoOutputState::Absent;
        }
        match fs::read_dir(self.out) {
            Ok(mut entries) => {
                if entries.next().is_some() {
                    NixFreeDemoOutputState::Occupied
                } else {
                    NixFreeDemoOutputState::Empty
                }
            }
            Err(error) => NixFreeDemoOutputState::Unreadable {
                detail: error.to_string(),
            },
        }
    }

    fn read_transcript(&mut self, transcript_index: u32) -> NixFreeDemoTranscriptRead {
        let Some(source) = self.transcript(transcript_index) else {
            return NixFreeDemoTranscriptRead::Unavailable {
                detail: format!("transcript {transcript_index} was not declared"),
            };
        };
        match fs::read(source) {
            Ok(bytes) => NixFreeDemoTranscriptRead::Read(bytes),
            Err(error) => NixFreeDemoTranscriptRead::Unavailable {
                detail: error.to_string(),
            },
        }
    }

    fn write_bundle(&mut self, bundle: &NixFreeDemoBundleWrite<'_>) -> Result<NixFreeDemoBundleFacts, CapabilityError> {
        let out = self.out;
        fs::create_dir_all(out).map_err(|error| write_refused(format!("creating {}: {error}", out.display())))?;
        fs::create_dir_all(out.join(TRANSCRIPTS_DIR))
            .map_err(|error| write_refused(format!("creating transcript dir: {error}")))?;
        let mut facts = NixFreeDemoBundleFacts {
            transcripts_copied: 0,
            documents_written: 0,
        };
        for copy in bundle.transcripts {
            let source = self
                .transcript(copy.transcript_index)
                .ok_or_else(|| write_refused(format!("transcript {} was not declared", copy.transcript_index)))?;
            fs::copy(source, out.join(&copy.bundle_path))
                .map_err(|error| write_refused(format!("copying transcript {}: {error}", source.display())))?;
            facts.transcripts_copied = facts.transcripts_copied.saturating_add(1);
        }
        for (file_name, bytes) in [
            (SUMMARY_FILE_NAME, bundle.summary_json),
            (MANIFEST_FILE_NAME, bundle.manifest_json),
            (VALIDATION_FILE_NAME, bundle.validation_json),
        ] {
            let path = out.join(file_name);
            fs::write(&path, bytes).map_err(|error| write_refused(format!("writing {}: {error}", path.display())))?;
            facts.documents_written = facts.documents_written.saturating_add(1);
        }
        fs::write(out.join(README_FILE_NAME), bundle.readme)
            .map_err(|error| write_refused(format!("writing generated README: {error}")))?;
        facts.documents_written = facts.documents_written.saturating_add(1);
        debug_assert_eq!(facts.documents_written, NIX_FREE_DEMO_BUNDLE_DOCUMENT_COUNT);
        debug_assert!(usize::try_from(facts.transcripts_copied).is_ok_and(|copied| copied == bundle.transcripts.len()));
        Ok(facts)
    }
}

fn write_refused(detail: String) -> CapabilityError {
    debug_assert!(!detail.is_empty());
    debug_assert!(!BUNDLE_WRITE_FAILED_CODE.is_empty());
    CapabilityError {
        code: BUNDLE_WRITE_FAILED_CODE.to_string(),
        detail,
    }
}

fn generated_files(bundle: &NixFreeDemoGeneratedBundle) -> Vec<String> {
    let mut files = vec![
        SUMMARY_FILE_NAME.to_string(),
        README_FILE_NAME.to_string(),
        MANIFEST_FILE_NAME.to_string(),
        VALIDATION_FILE_NAME.to_string(),
    ];
    files.extend(bundle.manifest.transcripts.iter().map(|transcript| transcript.bundle_path.clone()));
    files.sort();
    files
}

fn report_for_summary(summary: &NixFreeDemoMachineSummary, validation: &NixFreeDemoValidation) -> NixFreeDemoCliReport {
    NixFreeDemoCliReport {
        schema: CLI_REPORT_SCHEMA.to_string(),
        profile: validation.profile.clone(),
        verdict: summary.fixed_point_verdict.clone(),
        demo_claimable: validation.demo_claimable,
        diagnostics: validation.diagnostics.clone(),
    }
}

fn malformed_report(message: &str) -> NixFreeDemoCliReport {
    NixFreeDemoCliReport {
        schema: CLI_REPORT_SCHEMA.to_string(),
        profile: UNKNOWN_PROFILE.to_string(),
        verdict: MALFORMED_VERDICT.to_string(),
        demo_claimable: false,
        diagnostics: vec![NixFreeDemoDiagnostic {
            code: MALFORMED_SUMMARY_CODE.to_string(),
            message: message.to_string(),
        }],
    }
}

fn generated_report(out: &Path, prepared: &PreparedGeneratedBundle) -> NixFreeDemoGenerateReport {
    NixFreeDemoGenerateReport {
        schema: GENERATE_REPORT_SCHEMA.to_string(),
        generated: true,
        bundle_dir: out.display().to_string(),
        demo_claimable: prepared.bundle.validation.demo_claimable,
        diagnostics: prepared.bundle.validation.diagnostics.clone(),
        files: prepared.files.clone(),
    }
}

fn rejected_generate_report(out: &Path, diagnostic: NixFreeDemoDiagnostic) -> NixFreeDemoGenerateReport {
    NixFreeDemoGenerateReport {
        schema: GENERATE_REPORT_SCHEMA.to_string(),
        generated: false,
        bundle_dir: out.display().to_string(),
        demo_claimable: false,
        diagnostics: vec![diagnostic],
        files: Vec::new(),
    }
}

fn emit_report(report: NixFreeDemoCliReport, json: bool) -> Result<(), RunError> {
    if json {
        println!("{}", serialize_report(&report)?);
    } else {
        println!("{}", human_report(&report));
    }
    if report.demo_claimable {
        Ok(())
    } else {
        Err(RunError::Reported(NON_CLAIMABLE_EXIT_CODE))
    }
}

fn emit_generate_report(report: NixFreeDemoGenerateReport, json: bool) -> Result<(), RunError> {
    if json {
        println!("{}", serialize_generate_report(&report)?);
    } else {
        println!("{}", human_generate_report(&report));
    }
    if report.generated {
        Ok(())
    } else {
        Err(RunError::Reported(GENERATE_FAILURE_EXIT_CODE))
    }
}

fn serialize_report(report: &NixFreeDemoCliReport) -> Result<String, RunError> {
    serde_json::to_string_pretty(report)
        .map_err(|error| RunError::Internal(format!("{JSON_SERIALIZATION_CONTEXT}: {error}")))
}

fn serialize_generate_report(report: &NixFreeDemoGenerateReport) -> Result<String, RunError> {
    serde_json::to_string_pretty(report)
        .map_err(|error| RunError::Internal(format!("{JSON_SERIALIZATION_CONTEXT}: {error}")))
}

fn human_report(report: &NixFreeDemoCliReport) -> String {
    let mut lines = Vec::new();
    if report.demo_claimable {
        lines.push("Nix-free demo bundle: claimable".to_string());
    } else {
        lines.push("Nix-free demo bundle: not claimable".to_string());
    }
    lines.push(format!("profile: {}", report.profile));
    lines.push(format!("fixed-point verdict: {}", report.verdict));
    append_diagnostics(&mut lines, &report.diagnostics);
    lines.join("\n")
}

fn human_generate_report(report: &NixFreeDemoGenerateReport) -> String {
    let mut lines = Vec::new();
    if report.generated {
        lines.push(format!("Nix-free demo bundle generated: {}", report.bundle_dir));
    } else {
        lines.push(format!("Nix-free demo bundle generation failed: {}", report.bundle_dir));
    }
    lines.push(format!("demo claimable: {}", report.demo_claimable));
    append_diagnostics(&mut lines, &report.diagnostics);
    if !report.files.is_empty() {
        lines.push("files:".to_string());
        for file in &report.files {
            lines.push(format!("- {file}"));
        }
    }
    lines.join("\n")
}

fn append_diagnostics(lines: &mut Vec<String>, diagnostics: &[NixFreeDemoDiagnostic]) {
    if diagnostics.is_empty() {
        lines.push("diagnostics: none".to_string());
    } else {
        lines.push("diagnostics:".to_string());
        for diagnostic in diagnostics {
            lines.push(format!("- {}: {}", diagnostic.code, diagnostic.message));
        }
    }
}

fn diagnostic(code: &str, message: impl AsRef<str>) -> NixFreeDemoDiagnostic {
    NixFreeDemoDiagnostic {
        code: code.to_string(),
        message: message.as_ref().to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const DIGEST_A: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
    const DIGEST_B: &str = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
    const TRANSCRIPT_NAME: &str = "proof.log";

    #[test]
    fn report_for_summary_preserves_claimability_and_verdict() {
        let summary = claimable_summary();
        let validation = validate_nix_free_demo_bundle(&summary);

        let report = report_for_summary(&summary, &validation);
        let human = human_report(&report);

        assert_eq!(report.schema, CLI_REPORT_SCHEMA);
        assert_eq!(report.profile, "source-root-cargo-free-fixed-point");
        assert_eq!(report.verdict, "success");
        assert!(report.demo_claimable);
        assert!(report.diagnostics.is_empty());
        assert!(human.contains("Nix-free demo bundle: claimable"));
        assert!(human.contains("diagnostics: none"));
    }

    #[test]
    fn malformed_summary_report_fails_closed_with_stable_code() {
        let report = parse_and_validate_summary("not-json").unwrap_err();
        let encoded = serialize_report(&report).unwrap();

        assert_eq!(report.schema, CLI_REPORT_SCHEMA);
        assert_eq!(report.profile, UNKNOWN_PROFILE);
        assert_eq!(report.verdict, MALFORMED_VERDICT);
        assert!(!report.demo_claimable);
        assert_eq!(report.diagnostics.len(), 1);
        assert_eq!(report.diagnostics[0].code, MALFORMED_SUMMARY_CODE);
        assert!(encoded.contains(MALFORMED_SUMMARY_CODE));
    }

    #[test]
    fn generate_report_succeeds_for_claimable_bundle() {
        let input = claimable_generate_input();
        let bundle = build_nix_free_demo_bundle(input).unwrap();
        let prepared = PreparedGeneratedBundle {
            files: generated_files(&bundle),
            transcript_copies: Vec::new(),
            bundle,
        };

        let report = generated_report(Path::new("/tmp/demo"), &prepared);
        let human = human_generate_report(&report);

        assert!(report.generated);
        assert!(report.demo_claimable);
        assert!(report.files.contains(&SUMMARY_FILE_NAME.to_string()));
        assert!(human.contains("Nix-free demo bundle generated"));
    }

    #[test]
    fn parse_named_digest_rejects_malformed_digest() {
        let err = parse_named_digest("receipt:not-a-digest", "receipt-digest").unwrap_err();

        assert_eq!(err.code, "receipt-digest");
        assert!(err.message.contains("name:blake3"));
    }

    fn claimable_summary() -> NixFreeDemoMachineSummary {
        NixFreeDemoMachineSummary {
            schema: "mantle-nix-free-demo-summary-v1".to_string(),
            profile: "source-root-cargo-free-fixed-point".to_string(),
            fixed_point_verdict: "success".to_string(),
            stage1_binary_blake3: Some(DIGEST_A.to_string()),
            stage2_binary_blake3: Some(DIGEST_A.to_string()),
            source_root_identity: "source-root-v1".to_string(),
            toolchain_policy_digest_blake3: DIGEST_B.to_string(),
            command_owned_wrappers: Vec::new(),
            guards: guard_evidence(),
            replay_hints: vec!["copy bundle and rerun validator".to_string()],
            non_claims: vec!["not compiler correctness".to_string()],
        }
    }

    fn claimable_generate_input() -> NixFreeDemoManifestInput {
        NixFreeDemoManifestInput {
            proof_status: "success".to_string(),
            stage1_binary_blake3: Some(DIGEST_A.to_string()),
            stage2_binary_blake3: Some(DIGEST_A.to_string()),
            source_root_identity: "source-root-v1".to_string(),
            toolchain_policy_digest_blake3: DIGEST_B.to_string(),
            command_owned_wrappers: Vec::new(),
            guards: guard_evidence(),
            replay_hints: vec!["copy bundle and rerun validator".to_string()],
            non_claims: vec!["not compiler correctness".to_string()],
            transcripts: vec![NixFreeDemoEvidenceRef {
                name: TRANSCRIPT_NAME.to_string(),
                digest_blake3: DIGEST_A.to_string(),
                bundle_path: format!("{TRANSCRIPTS_DIR}/{TRANSCRIPT_NAME}"),
            }],
            receipt_digests: vec![NixFreeDemoNamedDigest {
                name: "receipt".to_string(),
                digest_blake3: DIGEST_A.to_string(),
            }],
            artifact_digests: vec![NixFreeDemoNamedDigest {
                name: "stage2-mantle".to_string(),
                digest_blake3: DIGEST_A.to_string(),
            }],
            synthetic: false,
        }
    }

    fn guard_evidence() -> Vec<NixFreeDemoGuardEvidence> {
        ["cargo", "nix", "rustup", "ambient-wrapper"]
            .iter()
            .map(|guard| NixFreeDemoGuardEvidence {
                guard: (*guard).to_string(),
                status: "denied".to_string(),
                diagnostic: format!("{guard} denied by fixture"),
            })
            .collect()
    }
}
