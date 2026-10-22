//! Bounded offline replay of the production selector; input facts confer no authority.
use std::io::Read as _;
use std::io::Write as _;

use crunch_remote_core::resource_policy::Controls;
use crunch_remote_core::resource_policy::Features;
use crunch_remote_core::resource_policy::MachineClass;
use crunch_remote_core::resource_policy::Observation;
use crunch_remote_core::resource_policy::Quantities;
use crunch_remote_core::resource_policy::QuotaFacts;
use crunch_remote_core::resource_policy::SelectionMode;
use crunch_remote_core::resource_policy::SelectionPolicy;
use crunch_remote_core::resource_policy::{self as policy};
use serde::Deserialize;

const MAX_REPLAY_BYTES: u64 = 1_048_576;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Replay {
    declared: Quantities,
    features: Features,
    action_family: String,
    classes: Vec<MachineClass>,
    history: Vec<Observation>,
    now_unix_s: u64,
    project_remaining_units: u64,
    account_remaining_units: u64,
    policy: SelectionPolicy,
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let mut bytes = Vec::new();
    std::io::stdin().lock().take(MAX_REPLAY_BYTES + 1).read_to_end(&mut bytes)?;
    if bytes.len() as u64 > MAX_REPLAY_BYTES {
        return Err("resource replay input exceeds one MiB".into());
    }
    let input: Replay = serde_json::from_slice(&bytes)?;
    let quota = QuotaFacts {
        project_remaining_units: input.project_remaining_units,
        account_remaining_units: input.account_remaining_units,
    };
    let static_decision = policy::select(policy::SelectionRequest {
        declared: input.declared,
        features: &input.features,
        family: &input.action_family,
        classes: &input.classes,
        history: &[],
        now: input.now_unix_s,
        quota,
        policy: input.policy,
    });
    let historical_decision = policy::select(policy::SelectionRequest {
        declared: input.declared,
        features: &input.features,
        family: &input.action_family,
        classes: &input.classes,
        history: &input.history,
        now: input.now_unix_s,
        quota,
        policy: input.policy,
    });
    let decision = policy::rollout(
        Controls {
            mode: SelectionMode::ObserveOnly,
            project_opted_in: false,
            selection_enabled: false,
            oom_retry_enabled: false,
            quota_enforcement_enabled: false,
            sharing_enabled: false,
        },
        static_decision,
        historical_decision,
    );
    let stdout = std::io::stdout();
    let mut output = stdout.lock();
    serde_json::to_writer(&mut output, &decision)?;
    output.write_all(b"\n")?;
    Ok(())
}

fn main() -> std::process::ExitCode {
    match run() {
        Ok(()) => std::process::ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("resource-policy-replay: {error}");
            std::process::ExitCode::from(2)
        }
    }
}
