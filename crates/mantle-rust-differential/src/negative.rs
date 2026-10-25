//! Real ordinary compiler observations plus explicitly labelled adversarial policy inputs.
//! Passing this campaign means the negative cases stayed nonpassing, not baseline acceptance.
use std::process::Command;
use std::time::Instant;

use mantle_rust_differential_core::Coverage;
use mantle_rust_differential_core::Observation;
use mantle_rust_differential_core::Status;
use mantle_rust_differential_core::Verdict;
use mantle_rust_differential_core::{self as core};
use serde::Deserialize;
use serde::Serialize;
use serde_json::Value;
use serde_json::json;

use crate::Result;
use crate::err;
use crate::fixtures;
use crate::process;
use crate::runner::Options;
use crate::runner::{self};

const CASE_IDS: [&str; 17] = [
    "alleged-broken-source-accepted",
    "wrong-rejection-diagnostic",
    "duplicated-diagnostic",
    "reordered-diagnostics",
    "stale-success-output-on-rejection",
    "incomplete-capture",
    "missed-edit",
    "wrong-postimage",
    "stream-bound",
    "deadline",
    "lost-cell",
    "lost-test",
    "lost-unit",
    "denominator-rewrite",
    "baseline-rewrite",
    "increased-required-skip",
    "fallback-as-original-coverage",
];
#[derive(Debug, Serialize, Deserialize)]
pub struct NegativeCase {
    pub id: String,
    pub evidence: Value,
    pub rejected: bool,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct NegativeReport {
    pub schema: String,
    pub options: Options,
    pub cases: Vec<NegativeCase>,
    pub planned_cases: Vec<String>,
    pub all_negatives_observed: bool,
    pub ordinary_population: core::CoverageSummary,
    pub policy: String,
}
fn case(cases: &mut Vec<NegativeCase>, id: &str, evidence: Value, rejected: bool) {
    cases.push(NegativeCase {
        id: id.into(),
        evidence,
        rejected,
    });
}
fn comparison(
    cases: &mut Vec<NegativeCase>,
    id: &str,
    left: &Observation,
    right: &Observation,
    expected: &core::Expected,
) {
    let verdict = core::compare(left, right, expected);
    case(
        cases,
        id,
        json!({"left":left,"right":right,"expectation":expected,"verdict":verdict,"provenance":"adversarial mutation of retained real compiler observation"}),
        matches!(verdict, Verdict::Mismatch | Verdict::HarnessError),
    );
}

pub fn execute(mut options: Options) -> Result<NegativeReport> {
    if options.baseline.is_some() {
        return Err("negative-campaign-freezes-its-own-unrun-population".into());
    }
    if unsafe { libc::prctl(libc::PR_SET_CHILD_SUBREAPER, 1, 0, 0, 0) } != 0 {
        return Err("subreaper-unavailable".into());
    }
    crunch_rustc_wrapper::observed_process::private_dir(&options.root)?;
    let total = Instant::now();
    runner::pin(&mut options)?;
    let fixtures = ["chain", "wide", "proc-macro", "generated"]
        .into_iter()
        .map(fixtures::generate)
        .collect::<Result<Vec<_>>>()?;
    runner::save(&options.root.join("fixture-recipe.json"), &fixtures)?;
    let probes = runner::probe_tools(&options, &fixtures[0], total)?;
    let manifest = runner::freeze(&options, &fixtures, &probes)?;
    runner::save(&options.root.join("manifest.json"), &manifest)?;
    let fixture = &fixtures[0];
    let cold = &fixture.steps[0];
    let broken = fixture.steps.iter().find(|s| s.id == "broken").ok_or("broken-fixture-missing")?;
    let cell = manifest.cells.iter().find(|c| c.id == "chain-build").ok_or("chain-build-missing")?;
    let mut ordered = broken.clone();
    ordered.id = "two-ordered-errors".into();
    let source = String::from_utf8(ordered.snapshot["lib0/src/lib.rs"].clone()).map_err(err)?;
    let needle = "fn body()->i64 { missing_value }";
    if source.matches(needle).count() != 1 {
        return Err("ordered-error-preimage".into());
    }
    ordered.snapshot.insert(
        "lib0/src/lib.rs".into(),
        source.replacen(needle, "fn body()->i64 { missing_value + another_missing_value }", 1).into_bytes(),
    );
    ordered.digest = fixtures::identity(&ordered.snapshot);
    ordered.edits.clear();
    ordered.expected.diagnostic_codes.push("E0425".into());
    ordered
        .expected
        .diagnostic_messages
        .push("cannot find value `another_missing_value` in this scope".into());
    runner::save(&options.root.join("negative-inputs.json"), &ordered)?;
    runner::save(&options.root.join("planned-cases.json"), &CASE_IDS)?;
    let (state, _) = runner::prepare_state(&options, &options.root.join("successful-clean"), &cold.snapshot, total)?;
    let success = runner::attempt(
        &options,
        &state,
        cell,
        cold,
        "compile",
        total,
        None,
        crunch_rustc_wrapper::observed_process::STREAM_BYTES,
    )?;
    core::validate_oracle(&success.observation, &cold.expected).map_err(err)?;
    let (rejection_state, _) =
        runner::prepare_state(&options, &options.root.join("rejected-clean"), &broken.snapshot, total)?;
    let rejection = runner::attempt(
        &options,
        &rejection_state,
        cell,
        broken,
        "compile",
        total,
        None,
        crunch_rustc_wrapper::observed_process::STREAM_BYTES,
    )?;
    core::validate_oracle(&rejection.observation, &broken.expected).map_err(err)?;
    let (ordered_state, _) =
        runner::prepare_state(&options, &options.root.join("ordered-clean"), &ordered.snapshot, total)?;
    let ordered_attempt = runner::attempt(
        &options,
        &ordered_state,
        cell,
        &ordered,
        "compile",
        total,
        None,
        crunch_rustc_wrapper::observed_process::STREAM_BYTES,
    )?;
    core::validate_oracle(&ordered_attempt.observation, &ordered.expected).map_err(err)?;
    let mut cases = vec![];
    comparison(
        &mut cases,
        "alleged-broken-source-accepted",
        &success.observation,
        &success.observation,
        &broken.expected,
    );
    let mut wrong_oracle = broken.expected.clone();
    wrong_oracle.diagnostic_codes = vec!["E0308".into()];
    wrong_oracle.diagnostic_messages = vec!["mismatched types".into()];
    comparison(
        &mut cases,
        "wrong-rejection-diagnostic",
        &rejection.observation,
        &rejection.observation,
        &wrong_oracle,
    );
    let mut duplicate = rejection.observation.clone();
    let records = &mut duplicate
        .units
        .iter_mut()
        .find(|u| !u.records.is_empty())
        .ok_or("real-rejection-record-missing")?
        .records;
    records.push(records[0].clone());
    comparison(&mut cases, "duplicated-diagnostic", &rejection.observation, &duplicate, &broken.expected);
    let mut reordered = ordered_attempt.observation.clone();
    let changed = reordered.units.iter_mut().filter(|u| u.records.len() > 1).any(|u| {
        let before = u.records.clone();
        u.records.reverse();
        u.records != before
    });
    let verdict = core::compare(&ordered_attempt.observation, &reordered, &ordered.expected);
    case(
        &mut cases,
        "reordered-diagnostics",
        json!({"original":ordered_attempt.observation,"observation":reordered,"expectation":ordered.expected,"verdict":verdict,"distinct_real_records_available":changed}),
        changed && verdict == Verdict::Mismatch,
    );
    let mut stale = rejection.observation.clone();
    stale.accepted_outputs = success.observation.accepted_outputs.clone();
    comparison(&mut cases, "stale-success-output-on-rejection", &stale, &stale, &broken.expected);
    let mut incomplete = rejection.observation.clone();
    incomplete.complete = false;
    comparison(&mut cases, "incomplete-capture", &incomplete, &incomplete, &broken.expected);
    let missed = process::verify(&state.source, &fixture.steps[3].snapshot);
    case(
        &mut cases,
        "missed-edit",
        json!({"source":cold.digest,"declared_postimage":fixture.steps[3].digest,"result":missed}),
        missed.is_err(),
    );
    let mut wrong = fixture.steps[3].clone();
    wrong.edits[0].after = wrong.edits[0].before.clone();
    let postimage = process::apply(&state.source, &cold.snapshot, &wrong);
    case(&mut cases, "wrong-postimage", json!({"edit":wrong,"result":postimage}), postimage.is_err());
    // Real child executions exercise a zero capture ceiling and an expired deadline.
    // These are infrastructure failures, never compiler-rejection or ordinary coverage.
    for (id, deadline, bound) in [
        ("stream-bound", crunch_rustc_wrapper::observed_process::PROBE_MS, 0),
        ("deadline", 0, crunch_rustc_wrapper::observed_process::STREAM_BYTES),
    ] {
        let root = options.root.join(id);
        crunch_rustc_wrapper::observed_process::private_dir(&root)?;
        let mut command = Command::new(&options.rustc);
        command
            .arg("--version")
            .current_dir(&rejection_state.source)
            .env_clear()
            .envs(runner::environment(&options, &rejection_state));
        let capture = crunch_rustc_wrapper::observed_process::supervise(
            command,
            crunch_rustc_wrapper::observed_process::Supervision {
                root: &root,
                scratch: &options.root,
                timeout_ms: deadline,
                total_start: total,
                injection: None,
                stream_bound: bound,
            },
        )?;
        let mut infrastructure = rejection.observation.clone();
        infrastructure.status = if id == "deadline" {
            Status::Timeout
        } else {
            Status::Infrastructure
        };
        infrastructure.complete = capture.complete;
        infrastructure.teardown_complete = capture.teardown_complete;
        let verdict = core::compare(&infrastructure, &infrastructure, &broken.expected);
        let observed = capture.failure.as_deref() == Some(id)
            && !capture.complete
            && capture.teardown_complete
            && verdict == Verdict::HarnessError;
        case(&mut cases, id, json!({"capture":capture,"observation":infrastructure,"verdict":verdict}), observed);
    }
    // Only ordinary cold/broken compiles contribute to this population. The separately
    // frozen ordered-error input is a negative, not an extra passing ordinary row.
    // Other planned members remain unrun; mutations never manufacture a denominator.
    let mut coverage: Vec<_> = manifest
        .population
        .iter()
        .map(|m| Coverage {
            key: m.key.clone(),
            attempted: false,
            executed: false,
            reused: false,
            fallback: None,
            unsupported: (!m.eligible && m.selected).then(|| "unregistered-capability".into()),
            failed: false,
            unrun: m.selected,
        })
        .collect();
    for key in [
        "chain-build/a/cold",
        "oracle/chain-build/a/cold",
        "chain-build/a/broken",
        "oracle/chain-build/a/broken",
    ] {
        let row = coverage.iter_mut().find(|c| c.key == key).ok_or("negative-coverage-key")?;
        row.attempted = true;
        row.executed = true;
        row.unrun = false;
    }
    let ordinary_population = core::reconcile(&manifest, &manifest, &coverage).map_err(err)?;
    for role in ["cell", "test", "unit"] {
        let mut lost = coverage.clone();
        let index = manifest.population.iter().position(|m| m.role == role).ok_or("negative-role-missing")?;
        lost.remove(index);
        let result = core::reconcile(&manifest, &manifest, &lost);
        case(
            &mut cases,
            &format!("lost-{role}"),
            json!({"result":result,"removed":coverage[index].key}),
            result == Err(core::Error::Missing),
        );
    }
    let mut rewritten = manifest.clone();
    rewritten.denominators[0].members.pop();
    let result = core::reconcile(&manifest, &rewritten, &coverage);
    case(&mut cases, "denominator-rewrite", json!({"result":result,"candidate":rewritten}), result.is_err());
    let mut rewritten = manifest.clone();
    rewritten.identities.iter_mut().find(|i| i.role == "source").ok_or("source-identity")?.digest =
        core::Digest::from_slice(b"attempted-report-time-baseline-rewrite");
    let result = core::reconcile(&manifest, &rewritten, &coverage);
    case(
        &mut cases,
        "baseline-rewrite",
        json!({"result":result,"candidate":rewritten}),
        result == Err(core::Error::BaselineRewrite),
    );
    let mut skipped = coverage.clone();
    let row = skipped.iter_mut().find(|c| c.key == "chain-build/a/cold").ok_or("cold-coverage")?;
    row.executed = false;
    row.unrun = true;
    let result = core::reconcile(&manifest, &manifest, &skipped);
    case(
        &mut cases,
        "increased-required-skip",
        json!({"result":result}),
        result.as_ref().is_ok_and(|r| r.deficits.contains(&"chain-build/a/cold".into())),
    );
    let mut fallback = coverage.clone();
    fallback.iter_mut().find(|c| c.key == "chain-build/a/cold").ok_or("cold-coverage")?.fallback =
        Some("clean-reference-restart".into());
    let result = core::reconcile(&manifest, &manifest, &fallback);
    case(
        &mut cases,
        "fallback-as-original-coverage",
        json!({"result":result}),
        result.as_ref().is_ok_and(|r| r.deficits.contains(&"chain-build/a/cold".into())),
    );
    let all_negatives_observed = cases.iter().map(|c| c.id.as_str()).eq(CASE_IDS) && cases.iter().all(|c| c.rejected);
    let report=NegativeReport{schema:"mantle-rust-differential-negatives-v1".into(),options,cases,planned_cases:CASE_IDS.iter().map(|id|(*id).into()).collect(),all_negatives_observed,ordinary_population,policy:"Only explicit real clean compiles and bounded child probes executed; derived adversarial observations are labelled policy injections, not compiler outputs. The ordinary matrix remains unrun/nonpassing here. Crash/cancel require the baseline command's separate actual fault campaign.".into()};
    runner::save(&report.options.root.join("negatives.json"), &report)?;
    Ok(report)
}
