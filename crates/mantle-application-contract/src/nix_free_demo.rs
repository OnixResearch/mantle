//! Typed Nix-free demo bundle generation: effect plan, port, and observation
//! classification.
//!
//! Generation reads the inputs the operator declared, admits or rejects the
//! request, and writes one bundle. The plan names the read and the write before
//! the port is called. A rejected request stops after the read, so it writes
//! nothing and no observation is recorded for the write it never ran. Once the
//! write has run, the generation classifies as complete only when the facts
//! show that the bundle copies exactly the transcripts that were read into an
//! output that was observed free, and when the port reports every planned file
//! written.

use alloc::string::String;
use alloc::vec::Vec;

use crate::envelope::ApplicationOutcome;
use crate::envelope::CapabilityError;
use crate::envelope::EffectId;
use crate::envelope::EffectKind;
use crate::envelope::EffectMeasure;
use crate::envelope::EffectOutput;
use crate::envelope::EffectPlan;
use crate::envelope::EffectSpec;
use crate::envelope::ExpectedOutput;
use crate::envelope::Observation;
use crate::envelope::ObservationStatus;
use crate::envelope::classify_observations;
use crate::envelope::plan_effects;
use crate::family::CommandFamily;

/// Effect identity of the declared-input read.
pub const NIX_FREE_DEMO_INPUT_READ_EFFECT: &str = "read-files";

/// Effect identity of the bundle write.
pub const NIX_FREE_DEMO_BUNDLE_WRITE_EFFECT: &str = "write-files";

/// Documents every bundle holds besides its transcripts: summary, manifest,
/// validation, and README.
pub const NIX_FREE_DEMO_BUNDLE_DOCUMENT_COUNT: u32 = 4;

/// Diagnostic code for a read that stopped at an input it could not read.
pub const NIX_FREE_DEMO_INPUT_UNAVAILABLE_CODE: &str = "nix-free-demo-input-unavailable";

/// Diagnostic code for a write whose inputs were not observed as admitted.
pub const NIX_FREE_DEMO_INPUT_UNADMITTED_CODE: &str = "nix-free-demo-input-unadmitted";

/// Diagnostic code for a write that reported fewer files than planned.
pub const NIX_FREE_DEMO_BUNDLE_INCOMPLETE_CODE: &str = "nix-free-demo-bundle-incomplete";

/// Observed state of the requested output directory.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NixFreeDemoOutputState {
    /// Nothing exists at the output path.
    Absent,
    /// The output path is an empty directory.
    Empty,
    /// The output path holds at least one entry.
    Occupied,
    /// The output path exists but could not be listed.
    Unreadable { detail: String },
}

impl NixFreeDemoOutputState {
    /// Whether a new bundle may be written at the output path.
    pub fn is_free(&self) -> bool {
        let is_free = matches!(self, Self::Absent | Self::Empty);
        debug_assert!(is_free || matches!(self, Self::Occupied | Self::Unreadable { .. }));
        debug_assert!(!is_free || !matches!(self, Self::Occupied));
        is_free
    }
}

/// One observed read of a declared transcript.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NixFreeDemoTranscriptRead {
    /// The transcript bytes as read.
    Read(Vec<u8>),
    /// The transcript could not be read.
    Unavailable { detail: String },
}

/// Facts the input read recorded, as far as it ran.
///
/// The default is the state before the read: nothing observed yet.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct NixFreeDemoInputFacts {
    /// Whether the output directory was observed absent or empty.
    pub output_free: bool,
    /// Declared transcripts read successfully.
    pub transcripts_read: u32,
    /// Whether the read stopped at an input it could not read: an output
    /// directory it could not list, or a transcript it could not read.
    pub input_unavailable: bool,
}

/// One declared transcript the bundle write copies.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NixFreeDemoTranscriptCopy {
    /// Position of the transcript in the operator's declaration order.
    pub transcript_index: u32,
    /// Bundle-relative destination.
    pub bundle_path: String,
}

/// One admitted bundle, borrowed from the operation that rendered it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NixFreeDemoBundleWrite<'a> {
    /// Declared transcripts to copy, in declaration order.
    pub transcripts: &'a [NixFreeDemoTranscriptCopy],
    /// Serialized machine summary.
    pub summary_json: &'a [u8],
    /// Serialized bundle manifest.
    pub manifest_json: &'a [u8],
    /// Serialized validation.
    pub validation_json: &'a [u8],
    /// Rendered operator README.
    pub readme: &'a str,
}

/// Facts one finished bundle write reports.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NixFreeDemoBundleFacts {
    /// Transcripts copied into the bundle.
    pub transcripts_copied: u32,
    /// Documents written into the bundle.
    pub documents_written: u32,
}

/// Port: the generate request's declared inputs and its one output directory.
///
/// An adapter holds only what the operator declared. It can observe and write
/// the named output directory, and read the declared transcripts by declaration
/// index. It cannot reach anything else.
pub trait NixFreeDemoBundlePort {
    /// Observe whether the output directory can receive a new bundle.
    fn probe_output(&mut self) -> NixFreeDemoOutputState;

    /// Read one declared transcript.
    fn read_transcript(&mut self, transcript_index: u32) -> NixFreeDemoTranscriptRead;

    /// Write one admitted bundle.
    fn write_bundle(&mut self, bundle: &NixFreeDemoBundleWrite<'_>) -> Result<NixFreeDemoBundleFacts, CapabilityError>;
}

/// The bounded plan for one generation: the input read, then the bundle write.
pub fn nix_free_demo_generate_effect_plan() -> Result<EffectPlan, CapabilityError> {
    plan_effects(CommandFamily::Planning, &[
        EffectSpec {
            effect_id: NIX_FREE_DEMO_INPUT_READ_EFFECT,
            kind: EffectKind::ReadFiles,
            limit: EffectMeasure::Calls(1),
            expected_output: ExpectedOutput::None,
        },
        EffectSpec {
            effect_id: NIX_FREE_DEMO_BUNDLE_WRITE_EFFECT,
            kind: EffectKind::WriteFiles,
            limit: EffectMeasure::Calls(1),
            expected_output: ExpectedOutput::None,
        },
    ])
    .map_err(|error| CapabilityError::new(error.code(), "the Nix-free demo generate plan was rejected"))
}

/// Effect identity of the one summary read behind `validate` and `readme`.
pub const NIX_FREE_DEMO_SUMMARY_READ_EFFECT: &str = "read-files";

/// Port: read the one summary the operator named, and nothing else.
pub trait NixFreeDemoSummaryPort {
    /// Read the summary text.
    fn read_summary(&mut self) -> Result<String, CapabilityError>;
}

/// The bounded plan for one summary consumer: a single summary read.
pub fn nix_free_demo_summary_effect_plan() -> Result<EffectPlan, CapabilityError> {
    plan_effects(CommandFamily::Planning, &[EffectSpec {
        effect_id: NIX_FREE_DEMO_SUMMARY_READ_EFFECT,
        kind: EffectKind::ReadFiles,
        limit: EffectMeasure::Calls(1),
        expected_output: ExpectedOutput::None,
    }])
    .map_err(|error| CapabilityError::new(error.code(), "the Nix-free demo summary plan was rejected"))
}

/// Classify one summary read the port executed.
///
/// `read` is that call's result, so the observation is never synthesized. A
/// read the port refused fails the operation before anything is reported;
/// whether the summary is claimable is a later domain decision, not part of
/// this observation.
pub fn classify_nix_free_demo_summary_read(
    plan: &EffectPlan,
    read: &Result<String, CapabilityError>,
) -> ApplicationOutcome {
    let read_code = read.as_ref().err().map(|error| error.code.clone());
    let observations = [observation(
        NIX_FREE_DEMO_SUMMARY_READ_EFFECT,
        EffectKind::ReadFiles,
        read_code,
    )];
    let outcome = classify_observations(plan, &observations);
    debug_assert!(outcome != ApplicationOutcome::Completed || read.is_ok());
    debug_assert!(!matches!(outcome, ApplicationOutcome::Blocked { .. }));
    outcome
}

/// Classify one generation that admission rejected after its input read.
///
/// The write never ran, so it is recorded as skipped rather than observed. The
/// read fails only when it stopped at an input it could not read. A request
/// rejected over what the read observed, or over its own arguments, keeps a
/// succeeded read. A rejected generation is never complete.
pub fn classify_nix_free_demo_rejection(plan: &EffectPlan, inputs: &NixFreeDemoInputFacts) -> ApplicationOutcome {
    let read_code = inputs.input_unavailable.then(|| String::from(NIX_FREE_DEMO_INPUT_UNAVAILABLE_CODE));
    let observations = [
        observation(NIX_FREE_DEMO_INPUT_READ_EFFECT, EffectKind::ReadFiles, read_code),
        Observation {
            effect_id: EffectId(String::from(NIX_FREE_DEMO_BUNDLE_WRITE_EFFECT)),
            kind: EffectKind::WriteFiles,
            status: ObservationStatus::Skipped,
            output: EffectOutput::None,
            usage: EffectMeasure::Calls(0),
            diagnostics_code: None,
        },
    ];
    let outcome = classify_observations(plan, &observations);
    debug_assert!(outcome != ApplicationOutcome::Completed);
    debug_assert!(!matches!(outcome, ApplicationOutcome::Blocked { .. }));
    outcome
}

/// Classify one generation whose bundle write has run.
///
/// Only an operation whose port executed the write may call this: `written`
/// is that call's result, so the write observation is never synthesized. The
/// read succeeds only when it read every input, observed the output free, and
/// read every transcript the bundle copies. The write succeeds only when the
/// port reports every planned transcript and document written. Anything else
/// fails closed, so a partial bundle never reports as generated.
pub fn classify_nix_free_demo_generate(
    plan: &EffectPlan,
    inputs: &NixFreeDemoInputFacts,
    bundle: &NixFreeDemoBundleWrite<'_>,
    written: &Result<NixFreeDemoBundleFacts, CapabilityError>,
) -> ApplicationOutcome {
    let planned_transcripts = u32::try_from(bundle.transcripts.len()).ok();
    let is_input_admitted =
        !inputs.input_unavailable && inputs.output_free && planned_transcripts == Some(inputs.transcripts_read);
    let write_code = match written {
        Ok(facts) => {
            let is_complete = planned_transcripts == Some(facts.transcripts_copied)
                && facts.documents_written == NIX_FREE_DEMO_BUNDLE_DOCUMENT_COUNT;
            (!is_complete).then(|| String::from(NIX_FREE_DEMO_BUNDLE_INCOMPLETE_CODE))
        }
        Err(error) => Some(error.code.clone()),
    };
    let input_code = (!is_input_admitted).then(|| String::from(NIX_FREE_DEMO_INPUT_UNADMITTED_CODE));
    let observations = [
        observation(NIX_FREE_DEMO_INPUT_READ_EFFECT, EffectKind::ReadFiles, input_code),
        observation(NIX_FREE_DEMO_BUNDLE_WRITE_EFFECT, EffectKind::WriteFiles, write_code),
    ];
    let outcome = classify_observations(plan, &observations);
    debug_assert!(outcome != ApplicationOutcome::Completed || (is_input_admitted && written.is_ok()));
    debug_assert!(!matches!(outcome, ApplicationOutcome::Blocked { .. }));
    outcome
}

/// One observation of an executed port call.
fn observation(effect: &str, kind: EffectKind, diagnostics_code: Option<String>) -> Observation {
    debug_assert!(!effect.is_empty());
    let status = if diagnostics_code.is_some() {
        ObservationStatus::Failed
    } else {
        ObservationStatus::Succeeded
    };
    debug_assert!(diagnostics_code.as_ref().is_none_or(|code| !code.is_empty()));
    Observation {
        effect_id: EffectId(String::from(effect)),
        kind,
        status,
        output: EffectOutput::None,
        usage: EffectMeasure::Calls(1),
        diagnostics_code,
    }
}

#[cfg(test)]
mod tests {
    use alloc::vec;

    use super::*;

    const README: &str = "readme";

    fn plan() -> EffectPlan {
        nix_free_demo_generate_effect_plan().expect("the generate plan is admitted")
    }

    fn copies() -> Vec<NixFreeDemoTranscriptCopy> {
        vec![
            NixFreeDemoTranscriptCopy {
                transcript_index: 0,
                bundle_path: String::from("transcripts/stage1.log"),
            },
            NixFreeDemoTranscriptCopy {
                transcript_index: 1,
                bundle_path: String::from("transcripts/stage2.log"),
            },
        ]
    }

    fn bundle(transcripts: &[NixFreeDemoTranscriptCopy]) -> NixFreeDemoBundleWrite<'_> {
        NixFreeDemoBundleWrite {
            transcripts,
            summary_json: b"{}",
            manifest_json: b"{}",
            validation_json: b"{}",
            readme: README,
        }
    }

    fn admitted(transcripts_read: u32) -> NixFreeDemoInputFacts {
        NixFreeDemoInputFacts {
            output_free: true,
            transcripts_read,
            input_unavailable: false,
        }
    }

    fn written(transcripts_copied: u32, documents_written: u32) -> Result<NixFreeDemoBundleFacts, CapabilityError> {
        Ok(NixFreeDemoBundleFacts {
            transcripts_copied,
            documents_written,
        })
    }

    #[test]
    fn a_complete_write_of_admitted_inputs_completes() {
        let copies = copies();
        let outcome = classify_nix_free_demo_generate(&plan(), &admitted(2), &bundle(&copies), &written(2, 4));
        assert_eq!(outcome, ApplicationOutcome::Completed);
    }

    #[test]
    fn a_failed_write_fails_the_generation() {
        let copies = copies();
        let failure = Err(CapabilityError::new("nix-free-demo-bundle-write-failed", "creating out: denied"));
        let outcome = classify_nix_free_demo_generate(&plan(), &admitted(2), &bundle(&copies), &failure);
        assert_eq!(outcome, ApplicationOutcome::Failed { failed_effect_count: 1 });
    }

    #[test]
    fn a_write_reporting_fewer_files_than_planned_fails_closed() {
        let copies = copies();
        let short_transcripts =
            classify_nix_free_demo_generate(&plan(), &admitted(2), &bundle(&copies), &written(1, 4));
        let short_documents = classify_nix_free_demo_generate(&plan(), &admitted(2), &bundle(&copies), &written(2, 3));
        assert_eq!(short_transcripts, ApplicationOutcome::Failed { failed_effect_count: 1 });
        assert_eq!(short_documents, ApplicationOutcome::Failed { failed_effect_count: 1 });
    }

    #[test]
    fn a_bundle_not_backed_by_admitted_inputs_fails_closed() {
        let copies = copies();
        let unread = classify_nix_free_demo_generate(&plan(), &admitted(1), &bundle(&copies), &written(2, 4));
        let occupied = NixFreeDemoInputFacts {
            output_free: false,
            ..admitted(2)
        };
        let not_free = classify_nix_free_demo_generate(&plan(), &occupied, &bundle(&copies), &written(2, 4));
        let unavailable = NixFreeDemoInputFacts {
            input_unavailable: true,
            ..admitted(2)
        };
        let not_read = classify_nix_free_demo_generate(&plan(), &unavailable, &bundle(&copies), &written(2, 4));
        assert_eq!(unread, ApplicationOutcome::Failed { failed_effect_count: 1 });
        assert_eq!(not_free, ApplicationOutcome::Failed { failed_effect_count: 1 });
        assert_eq!(not_read, ApplicationOutcome::Failed { failed_effect_count: 1 });
    }

    #[test]
    fn a_rejection_records_the_write_as_skipped_and_never_completes() {
        let occupied = NixFreeDemoInputFacts {
            output_free: false,
            ..NixFreeDemoInputFacts::default()
        };
        let rejected_after_clean_read = classify_nix_free_demo_rejection(&plan(), &occupied);
        let unavailable = NixFreeDemoInputFacts {
            input_unavailable: true,
            ..admitted(0)
        };
        let rejected_after_failed_read = classify_nix_free_demo_rejection(&plan(), &unavailable);
        assert_eq!(rejected_after_clean_read, ApplicationOutcome::Failed { failed_effect_count: 1 });
        assert_eq!(rejected_after_failed_read, ApplicationOutcome::Failed { failed_effect_count: 2 });
    }

    #[test]
    fn only_an_absent_or_empty_output_is_free() {
        assert!(NixFreeDemoOutputState::Absent.is_free());
        assert!(NixFreeDemoOutputState::Empty.is_free());
        assert!(!NixFreeDemoOutputState::Occupied.is_free());
        assert!(
            !NixFreeDemoOutputState::Unreadable {
                detail: String::from("denied"),
            }
            .is_free()
        );
    }
}
