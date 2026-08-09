use alloc::string::String;

use crate::CANCELLED_EXIT_CODE;
use crate::EVALUATION_NON_SUCCESS_EXIT_CODE;
use crate::INTERNAL_EXIT_CODE;
use crate::PIPELINE_NON_SUCCESS_EXIT_CODE;
use crate::STREAM_SCHEMA;
use crate::SUCCESS_EXIT_CODE;
use crate::types::OutcomeError;
use crate::types::RootOutcome;
use crate::types::RootSet;
use crate::types::RunDisposition;
use crate::types::RunSummary;
use crate::types::SelectedRoot;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RunStartValue {
    schema: String,
    run_id: String,
    evaluator_cohort: String,
    source_blake3: String,
    selector: String,
    selected_root_count: u32,
}

impl RunStartValue {
    pub fn from_root_set(root_set: &RootSet) -> Result<Self, OutcomeError> {
        let context = root_set.context();
        Ok(Self {
            schema: STREAM_SCHEMA.into(),
            run_id: root_set.run_id()?,
            evaluator_cohort: context.evaluator_cohort(),
            source_blake3: context.source_blake3(),
            selector: context.selector(),
            selected_root_count: root_set.root_count(),
        })
    }

    pub fn schema(&self) -> String {
        self.schema.clone()
    }

    pub fn run_id(&self) -> String {
        self.run_id.clone()
    }

    pub fn evaluator_cohort(&self) -> String {
        self.evaluator_cohort.clone()
    }

    pub fn source_blake3(&self) -> String {
        self.source_blake3.clone()
    }

    pub fn selector(&self) -> String {
        self.selector.clone()
    }

    pub fn selected_root_count(&self) -> u32 {
        self.selected_root_count
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StreamRecordValue {
    RunStart(RunStartValue),
    RootDiscovered(SelectedRoot),
    RootTerminal(RootOutcome),
    RunSummary(RunSummary),
}

impl StreamRecordValue {
    pub fn run_start(root_set: &RootSet) -> Result<Self, OutcomeError> {
        Ok(Self::RunStart(RunStartValue::from_root_set(root_set)?))
    }

    pub fn root_discovered(root: SelectedRoot) -> Self {
        Self::RootDiscovered(root)
    }

    pub fn root_terminal(outcome: RootOutcome) -> Self {
        Self::RootTerminal(outcome)
    }

    pub fn run_summary(summary: RunSummary) -> Self {
        Self::RunSummary(summary)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProcessMode {
    Evaluation,
    Pipeline,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProcessOutcome {
    Completed(RunDisposition),
    InternalFailure,
}

pub fn process_status(outcome: ProcessOutcome, mode: ProcessMode) -> u8 {
    match (outcome, mode) {
        (ProcessOutcome::InternalFailure, _) => INTERNAL_EXIT_CODE,
        (ProcessOutcome::Completed(RunDisposition::Success), _) => SUCCESS_EXIT_CODE,
        (ProcessOutcome::Completed(RunDisposition::Cancelled), _) => CANCELLED_EXIT_CODE,
        (ProcessOutcome::Completed(RunDisposition::Partial | RunDisposition::Failed), ProcessMode::Evaluation) => {
            EVALUATION_NON_SUCCESS_EXIT_CODE
        }
        (ProcessOutcome::Completed(RunDisposition::Partial | RunDisposition::Failed), ProcessMode::Pipeline) => {
            PIPELINE_NON_SUCCESS_EXIT_CODE
        }
    }
}
