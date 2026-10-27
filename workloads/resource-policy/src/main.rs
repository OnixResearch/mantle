// SPDX-License-Identifier: AGPL-3.0-or-later
#![feature(register_tool)]
#![register_tool(tigerstyle)]
//! External Rust guest: real production policy and filesystem accounting shell.
mod activity;
mod crash;
mod decisions;
// Deliberately compile the production shell, not a copied accounting model.
#[path = "../../../crates/crunch-build/src/distributed/resource_quota_store.rs"]
#[allow(dead_code)]
mod quota_store;

use std::os::unix::fs::DirBuilderExt;
use std::path::Path;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;

use chaoscontrol_sdk::prelude::*;
use crunch_remote_core::resource_policy::Reason;
use crunch_remote_core::resource_quota::*;
use decisions::*;
use quota_store::ResourceQuotaStore;
use serde_json::json;

const WORKLOAD: &str = "mantle-resource-policy";
static FAILED: AtomicBool = AtomicBool::new(false);
macro_rules! safety {
    ($condition:expr, $message:literal, $details:expr) => {{
        let valid = $condition;
        if !valid {
            FAILED.store(true, Ordering::Relaxed);
        }
        cc_assert_always_category!(WORKLOAD, "resource-invariant", valid, $message, &$details);
    }};
}
fn error(e: impl std::fmt::Debug) -> String {
    format!("{e:?}")
}
fn limits() -> Limits {
    Limits {
        project_units: 10,
        account_units: 10,
    }
}
fn details(case: u32) -> serde_json::Value {
    json!({"case":case,"adoption_track":"external-harness"})
}
fn retry_scenario(control: &str) {
    let mut plan = retry(0).ok();
    if control == "broken-retry-link" {
        if let Some(plan) = &mut plan {
            plan.predecessor = plan.next_attempt.clone();
        }
    }
    safety!(
        plan.as_ref().is_some_and(linked),
        "resource retry has fresh fence and exact predecessor",
        details(0)
    );
    for (index, expected) in RETRY_REASONS.iter().enumerate() {
        let actual = retry(index as u32 + 1);
        safety!(
            actual == Err(*expected),
            "resource retry rejects ambiguous stale and exhausted evidence",
            details(index as u32 + 1)
        );
    }
    cc_assert_reachable_category!(
        WORKLOAD,
        "outcome",
        "resource positive and denied retry outcomes observed",
        &details(0)
    );
}
fn ledger_scenario(root: &Path, control: &str) -> Result<(), String> {
    let r = reservation("usage");
    let store = ResourceQuotaStore::provision(root).map_err(error)?;
    store.reserve(r.clone(), limits()).map_err(error)?;
    store.reconcile(&r, UsageEvent::WorkerLost).map_err(error)?;
    let pending = store.snapshot().map_err(error)?.summary(&r).map_err(error)?;
    safety!(
        pending.project_reserved == 10 && pending.account_reserved == 10,
        "resource worker loss retains project and account reservation",
        details(0)
    );
    let denied = store.reserve(reservation("extra"), limits());
    safety!(denied == Err(QuotaError::Denied), "resource pending usage cannot mint quota", details(0));
    let mut stale = r.clone();
    stale.fence = stale.fence.advance().unwrap();
    safety!(
        store.reconcile(&stale, complete()) == Err(QuotaError::Conflict),
        "resource delayed stale fence cannot reconcile current reservation",
        details(0)
    );
    let settled = store.reconcile(&r, complete()).map_err(error)?;
    let revision = store.snapshot().map_err(error)?.revision;
    let duplicate = store.reconcile(&r, complete()).map_err(error)?;
    let reopened = ResourceQuotaStore::open(root).map_err(error)?;
    let snapshot = reopened.snapshot().map_err(error)?;
    let mut summary = snapshot.summary(&r).map_err(error)?;
    if control == "double-charge" {
        summary.project_charged += 3;
    }
    safety!(
        duplicate == settled
            && snapshot.revision == revision
            && summary.project_charged == 3
            && summary.account_charged == 3
            && summary.project_reserved == 0
            && summary.account_reserved == 0,
        "resource duplicate completion charges once across reopen",
        details(0)
    );
    safety!(
        store.reconcile(&r, UsageEvent::WorkerLost) == Err(QuotaError::Conflict),
        "resource delayed worker loss cannot overwrite settlement",
        details(0)
    );
    let mut malformed = reservation("malformed");
    malformed.schedule.cpu_ms_per_unit = 0;
    safety!(
        store.reserve(malformed, limits()) == Err(QuotaError::Invalid),
        "resource malformed schedule rejected before reservation",
        details(0)
    );
    cc_assert_sometimes_category!(
        WORKLOAD,
        "progress",
        summary.account_charged == 3 && summary.account_reserved == 0,
        "resource delayed trusted completion settles pending accounting",
        &details(0)
    );
    Ok(())
}
fn crash_scenario(root: &Path, boundary: u32, control: &str) -> Result<(), String> {
    let store = ResourceQuotaStore::provision(root).map_err(error)?;
    let r = reservation("crash");
    store.reserve(r.clone(), limits()).map_err(error)?;
    crash::interrupt(root, boundary)?;
    let reopened = ResourceQuotaStore::open(root).map_err(error)?;
    let mut before = reopened.snapshot().map_err(error)?.summary(&r).map_err(error)?;
    if control == "lost-reservation" {
        before.project_reserved = 0;
        before.project_charged = 0;
    }
    let reserved = before.project_reserved == 10
        && before.project_charged == 0
        && before.account_reserved == 10
        && before.account_charged == 0;
    let committed = before.project_reserved == 0
        && before.project_charged == 3
        && before.account_reserved == 0
        && before.account_charged == 3;
    safety!(
        (reserved || committed) && (boundary != 0 || reserved) && (boundary != 1 || committed),
        "resource killed accounting leaves complete old or new transaction",
        details(boundary)
    );
    reopened.reconcile(&r, complete()).map_err(error)?;
    reopened.reconcile(&r, complete()).map_err(error)?;
    let recovered = ResourceQuotaStore::open(root)
        .map_err(error)?
        .snapshot()
        .map_err(error)?
        .summary(&r)
        .map_err(error)?;
    safety!(
        recovered.project_charged == 3
            && recovered.account_charged == 3
            && recovered.project_reserved == 0
            && recovered.account_reserved == 0,
        "resource interrupted accounting recovers exactly one charge",
        details(boundary)
    );
    cc_assert_reachable_category!(
        WORKLOAD,
        "outcome",
        "resource actual accounting child killed and reopened",
        &details(boundary)
    );
    Ok(())
}
fn unavailable_scenario(root: &Path, control: &str) -> Result<(), String> {
    std::fs::DirBuilder::new().mode(0o700).create(root).map_err(error)?;
    safety!(ResourceQuotaStore::open(root).is_err(), "resource missing ledger cannot admit usage", details(0));
    let store = ResourceQuotaStore::provision(root).map_err(error)?;
    let r = reservation("unavailable");
    store.reserve(r.clone(), limits()).map_err(error)?;
    let path = root.join("resource-quota.json");
    let bytes = std::fs::read(&path).map_err(error)?;
    for (case, broken) in [(1, b"{".as_slice()), (2, b"{}".as_slice())] {
        std::fs::write(&path, broken).map_err(error)?;
        safety!(
            store.reserve(reservation("extra"), limits()).is_err() && store.reconcile(&r, complete()).is_err(),
            "resource malformed or missing accounting fields fail closed",
            details(case)
        );
    }
    let file = std::fs::File::create(&path).map_err(error)?;
    file.set_len(quota_store::MAX_LEDGER_BYTES + 1).map_err(error)?;
    safety!(ResourceQuotaStore::open(root).is_err(), "resource oversized ledger fails closed", details(3));
    std::fs::remove_file(&path).map_err(error)?;
    safety!(
        ResourceQuotaStore::provision(root).is_err(),
        "resource lost ledger sentinel prevents silent reprovision",
        details(4)
    );
    std::fs::write(&path, bytes).map_err(error)?;
    let cas = root.join("cas-object");
    std::fs::write(&cas, b"bounded-cas-object").map_err(error)?;
    safety!(
        sharing(std::fs::File::open(&cas).is_ok(), true).is_ok(),
        "resource authorized available result policy accepts scoped reuse",
        details(0)
    );
    std::fs::remove_file(&cas).map_err(error)?;
    let mut reuse = sharing(std::fs::File::open(&cas).is_ok(), true);
    if control == "cas-bypass" {
        reuse = sharing(true, true);
    }
    safety!(reuse == Err(Reason::CasUnavailable), "resource absent CAS denies result reuse", details(0));
    safety!(
        sharing(true, false) == Err(Reason::SharingDenied),
        "resource CAS presence alone grants no cross project authority",
        details(0)
    );
    Ok(())
}
fn run(control: &str) -> Result<(), String> {
    let root = tempfile::tempdir().map_err(error)?;
    retry_scenario(control);
    ledger_scenario(&root.path().join("ledger"), control)?;
    // Cover every crash boundary locally and in every VM; guided choice varies
    // their order, rather than permitting a passing campaign with no faults.
    let offset = random_choice(3) as u32;
    for index in 0..3 {
        let boundary = (index + offset) % 3;
        crash_scenario(&root.path().join(format!("crash-{boundary}")), boundary, control)?;
    }
    unavailable_scenario(&root.path().join("unavailable"), control)?;
    Ok(())
}
fn main() {
    // Linux forwards unrecognized kernel arguments to PID 1. They are boot
    // inputs, not the local negative-control CLI; parse CLI args only outside
    // the init role so a real boot cannot exit before SDK initialization.
    let args: Vec<_> = if std::process::id() == 1 {
        Vec::new()
    } else {
        std::env::args().skip(1).collect()
    };
    let control = match args.as_slice() {
        [] => "none",
        [flag, value]
            if flag == "--negative-control"
                && ["broken-retry-link", "double-charge", "lost-reservation", "cas-bypass"]
                    .contains(&value.as_str()) =>
        {
            value
        }
        _ => {
            eprintln!(
                "usage: mantle-resource-policy-chaos [--negative-control broken-retry-link|double-charge|lost-reservation|cas-bypass]"
            );
            std::process::exit(2);
        }
    };
    let local = std::env::var_os("CHAOSCONTROL_SDK_LOCAL_OUTPUT").is_some();
    if !local {
        guest_init();
    }
    std::fs::create_dir_all("/tmp").expect("guest writable temporary directory");
    let harness = WorkloadHarness::new(WORKLOAD);
    harness.init();
    harness.setup_complete(json!({"adoption_track":"external-harness", "production_shell":"resource_quota_store", "platform_oom_collector_exercised":false}));
    let result = harness.scenario("production resource and quota faults", || {
        let activity = activity::GuestActivity::start()?;
        let result = run(control);
        let stopped = activity.stop();
        result.and(stopped)
    });
    if result.is_err() {
        FAILED.store(true, Ordering::Relaxed);
    }
    cc_assert_sometimes_category!(
        WORKLOAD,
        "progress",
        result.is_ok(),
        "resource fault workload completes bounded operations",
        &json!({"adoption_track":"external-harness", "error":result.err().map(|s| s.chars().take(192).collect::<String>())})
    );
    send_event(
        "workload_done",
        &json!({"workload":WORKLOAD,"adoption_track":"external-harness","failed":FAILED.load(Ordering::Relaxed)}),
    );
    if local {
        std::process::exit(i32::from(FAILED.load(Ordering::Relaxed)));
    }
    // PID 1 must remain alive; returning would panic the guest kernel.
    loop {
        unsafe {
            libc::pause();
        }
    }
}
