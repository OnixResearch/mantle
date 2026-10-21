//! Pure sequencing contract. A prepared replay is not execution evidence.
use alloc::vec;
use alloc::vec::Vec;

use serde::Deserialize;
use serde::Serialize;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Operation {
    Inspect,
    Verify,
    Export,
    Replay,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Stage {
    ValidateCompatibility,
    VerifyPayloads,
    AdmitNativePlan,
    VerifyReceiptChain,
    ImportSignedStoreArchive,
    ImportSources,
    ImportReceipts,
    ReplayExportedGraph,
    PublishBundle,
}

/// The shell must finish the complete verification prefix before any admission.
pub fn stages(operation: Operation) -> Vec<Stage> {
    let mut stages = vec![Stage::ValidateCompatibility];
    if operation == Operation::Inspect {
        return stages;
    }
    stages.extend([Stage::VerifyPayloads, Stage::AdmitNativePlan, Stage::VerifyReceiptChain]);
    if operation == Operation::Export {
        stages.push(Stage::PublishBundle);
    }
    if operation == Operation::Replay {
        stages.extend([
            Stage::ImportSignedStoreArchive,
            Stage::ImportSources,
            Stage::ImportReceipts,
            Stage::ReplayExportedGraph,
        ]);
    }
    stages
}
