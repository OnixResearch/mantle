use crunch_remote_core::attempt::RemoteAttemptId;
use crunch_remote_core::attempt::RemoteFenceGeneration;
use crunch_remote_core::attempt::RemoteJobId;
use crunch_remote_core::resource_policy::Controls;
use crunch_remote_core::resource_policy::Features;
use crunch_remote_core::resource_policy::MachineClass;
use crunch_remote_core::resource_policy::Observation;
use crunch_remote_core::resource_policy::OomCategory;
use crunch_remote_core::resource_policy::Outcome;
use crunch_remote_core::resource_policy::Quantities;
use crunch_remote_core::resource_policy::QuotaFacts;
use crunch_remote_core::resource_policy::Reason;
use crunch_remote_core::resource_policy::SelectionMode;
use crunch_remote_core::resource_policy::SelectionPolicy;

use super::*;

fn q(memory: u64) -> Quantities {
    Quantities {
        cpu_units: 2,
        memory_bytes: memory,
        scratch_bytes: 100,
    }
}
fn features() -> Features {
    Features {
        architecture: "x86_64".into(),
        platform: "linux-cgroup-v2".into(),
        trust_domain: "reviewed".into(),
        isolation: "sandbox".into(),
        kvm: true,
    }
}
fn controls() -> Controls {
    Controls {
        mode: SelectionMode::ProjectOptIn,
        project_opted_in: true,
        selection_enabled: true,
        oom_retry_enabled: true,
        quota_enforcement_enabled: true,
        sharing_enabled: true,
    }
}
fn policy() -> SelectionPolicy {
    SelectionPolicy {
        version: 1,
        min_samples: 1,
        max_age_s: 100,
        margin_basis_points: 0,
        max_measurement: q(1000),
        max_duration_ms: 1000,
        max_io_bytes: 1000,
    }
}
fn classes() -> Vec<MachineClass> {
    [("small", 100, 1), ("large", 200, 2)]
        .into_iter()
        .map(|(id, memory, order)| MachineClass {
            id: id.into(),
            features: features(),
            quantities: q(memory),
            order,
            reservation_units: u64::from(order) * 10,
            available: true,
            existing_admission_compatible: true,
        })
        .collect()
}
fn observation() -> Observation {
    Observation {
        schema: core::OBSERVATION_SCHEMA.into(),
        attempt_id: RemoteAttemptId::new("terminal").unwrap(),
        action_family: "family".into(),
        features: features(),
        policy_version: 1,
        machine_class: "large".into(),
        declared: q(100),
        selected: q(200),
        measured: q(150).into(),
        scratch_sampled_peak_bytes: None,
        queue_ms: Some(1),
        execution_ms: Some(10),
        terminal_unix_s: 100,
        cpu_ms: Some(10),
        io_bytes: Some(10),
        transfer_bytes: Some(10),
        executor_output_bytes: Some(10),
        process_peak_rss_bytes: None,
        oom: OomCategory::None,
        outcome: Outcome::Success,
        predecessor: None,
        collector_version: "collector-v1".into(),
        trusted_collector: true,
    }
}
fn select(classes: &[MachineClass], history: &[Observation], quota: u64) -> Result<core::Selection, Reason> {
    core::select(core::SelectionRequest {
        declared: q(100),
        features: &features(),
        family: "family",
        classes,
        history,
        now: 100,
        quota: QuotaFacts {
            project_remaining_units: quota,
            account_remaining_units: quota,
        },
        policy: policy(),
    })
}
#[test]
fn history_enlarges_without_weakening_requirements_and_order_is_stable() {
    let mut catalog = classes();
    let first = select(&catalog, &[observation()], 100).unwrap();
    assert_eq!(catalog[first.class_index].id, "large");
    catalog.reverse();
    let replay = select(&catalog, &[observation()], 100).unwrap();
    assert_eq!(catalog[replay.class_index].id, "large");
    for index in 0..5 {
        let mut broken = classes();
        match index {
            0 => broken[1].features.architecture = "aarch64".into(),
            1 => broken[1].features.platform = "other".into(),
            2 => broken[1].features.kvm = false,
            3 => broken[1].features.trust_domain = "untrusted".into(),
            _ => broken[1].features.isolation = "none".into(),
        }
        assert_eq!(select(&broken, &[observation()], 100), Err(Reason::NoClass));
    }
    let mut broken = classes();
    broken[1].existing_admission_compatible = false;
    assert_eq!(select(&broken, &[observation()], 100), Err(Reason::NoClass));
    assert_eq!(select(&classes(), &[observation()], 10), Err(Reason::QuotaDenied));
    assert_eq!(select(&[], &[], 100), Err(Reason::NoClass));
}
#[test]
fn unknown_untrusted_stale_incompatible_and_outlier_history_fall_back() {
    let missing = select(&classes(), &[], 100).unwrap();
    assert_eq!(missing.reason, Reason::HistoryMissing);
    assert_eq!(missing.requested, q(100));
    for (change, expected) in [
        (0, Reason::HistoryUntrusted),
        (1, Reason::HistoryStale),
        (2, Reason::HistoryIncompatible),
        (3, Reason::HistoryOutlier),
    ] {
        let mut o = observation();
        match change {
            0 => o.trusted_collector = false,
            1 => o.terminal_unix_s = 101,
            2 => o.action_family = "other".into(),
            _ => o.measured.memory_bytes = Some(1001),
        }
        let decision = select(&classes(), &[o], 100).unwrap();
        assert_eq!(decision.exclusions, vec![(0, expected)]);
        assert_eq!(decision.requested, q(100));
        assert_eq!(decision.reason, Reason::HistoryInsufficient);
    }
    let duplicate = select(&classes(), &[observation(), observation()], 100).unwrap();
    assert_eq!(duplicate.samples, 1);
    assert_eq!(duplicate.exclusions, vec![(1, Reason::HistoryDuplicate)]);
}
#[test]
fn observe_only_and_retry_rollback_are_independent() {
    let static_decision = select(&classes(), &[], 100);
    let historical = select(&classes(), &[observation()], 100);
    let mut c = controls();
    c.mode = SelectionMode::ObserveOnly;
    let report = core::rollout(c, static_decision.clone(), historical.clone());
    assert_eq!(report.active, static_decision);
    assert_eq!(report.historical_decision, historical);
    c.mode = SelectionMode::ProjectOptIn;
    c.oom_retry_enabled = false;
    assert!(c.selection_active());
    assert!(!c.retry_active());
    assert!(c.sharing_active());
    assert!(core::NON_CLAIMS.contains(&"no simulated service acceptance"));
}
struct EscalationFixture {
    job: RemoteJobId,
    attempt: RemoteAttemptId,
    next: RemoteAttemptId,
}
impl EscalationFixture {
    fn new() -> Self {
        Self {
            job: RemoteJobId::new("job").unwrap(),
            attempt: RemoteAttemptId::new("old").unwrap(),
            next: RemoteAttemptId::new("new").unwrap(),
        }
    }

    fn request<'a>(&'a self, catalog: &'a [MachineClass]) -> core::EscalationRequest<'a> {
        let fence = RemoteFenceGeneration::INITIAL;
        core::EscalationRequest {
            controls: controls(),
            context: core::RetryContext {
                job: &self.job,
                attempt: &self.attempt,
                fence,
                next_attempt: &self.next,
                next_fence: fence.advance().unwrap(),
                failed: true,
                retries: 0,
                cumulative_units: 10,
                elapsed_ms: 10,
                next_wall_budget_ms: 10,
            },
            evidence: core::OomEvidence {
                job: &self.job,
                attempt: &self.attempt,
                fence,
                platform: "linux-cgroup-v2",
                category: OomCategory::LinuxCgroupV2Kill,
                trusted_positive_platform_evidence: true,
            },
            prior: &catalog[0],
            classes: catalog,
            quota: QuotaFacts {
                project_remaining_units: 20,
                account_remaining_units: 20,
            },
            policy: core::EscalationPolicy {
                version: 1,
                max_retries: 1,
                largest_class_order: 2,
                max_cumulative_units: 30,
                max_wall_ms: 20,
            },
        }
    }
}

#[test]
fn oom_retry_needs_positive_current_fenced_evidence_and_all_limits() {
    let fixture = EscalationFixture::new();
    let catalog = classes();
    let request = || fixture.request(&catalog);
    let retry = core::plan_escalation(request()).unwrap();
    assert_eq!(retry.predecessor, fixture.attempt);
    assert_eq!(retry.next_attempt, fixture.next);
    assert_eq!(retry.next_fence, RemoteFenceGeneration::INITIAL.advance().unwrap());
    assert_eq!(retry.class_index, 1);
    // The context remains failed: None covers non-OOM failure, Unknown ambiguous termination.
    for category in [OomCategory::Unknown, OomCategory::None] {
        let mut r = request();
        r.evidence.category = category;
        assert_eq!(core::plan_escalation(r), Err(Reason::NotPositiveOom));
    }
    let mut r = request();
    r.evidence.trusted_positive_platform_evidence = false;
    assert_eq!(core::plan_escalation(r), Err(Reason::NotPositiveOom));
    let mut r = request();
    r.context.next_fence = RemoteFenceGeneration::INITIAL;
    assert_eq!(core::plan_escalation(r), Err(Reason::FenceInvalid));
    let mut r = request();
    r.context.retries = 1;
    assert_eq!(core::plan_escalation(r), Err(Reason::RetryLimit));
    let mut r = request();
    r.policy.max_cumulative_units = 29;
    assert_eq!(core::plan_escalation(r), Err(Reason::ChargeLimit));
    let mut r = request();
    r.policy.max_wall_ms = 19;
    assert_eq!(core::plan_escalation(r), Err(Reason::WallTimeLimit));
    let mut r = request();
    r.policy.largest_class_order = 1;
    assert_eq!(core::plan_escalation(r), Err(Reason::ClassLimit));
    let mut r = request();
    r.quota.project_remaining_units = 19;
    assert_eq!(core::plan_escalation(r), Err(Reason::QuotaDenied));
}

#[test]
fn oom_retry_rejects_nonfailed_attempt_and_unavailable_larger_class() {
    let fixture = EscalationFixture::new();
    let mut catalog = classes();
    let mut not_failed = fixture.request(&catalog);
    not_failed.context.failed = false;
    assert_eq!(core::plan_escalation(not_failed), Err(Reason::NotPositiveOom));
    catalog[1].available = false;
    assert_eq!(core::plan_escalation(fixture.request(&catalog)), Err(Reason::ClassLimit));
}
#[test]
fn sharing_never_skips_existing_signed_result_admission() {
    let checks = core::SharingChecks {
        request_and_action_match: true,
        trusted_producer_signature: true,
        policy_compatible: true,
        platform_compatible: true,
        verified_output_identities: true,
        current_cas_available: true,
    };
    let scope = || {
        Some(core::SharingScope {
            id: "reviewed-scope",
            producers: &["producer"],
            consumers: &["consumer"],
        })
    };
    let result = admit_authorized_result(controls(), "producer", "consumer", "signed-result", scope(), checks, || {
        Ok::<_, &str>(42)
    })
    .unwrap();
    assert_eq!(result.0, 42);
    assert_eq!(result.1.scope.as_deref(), Some("reviewed-scope"));
    assert!(matches!(
        admit_authorized_result(controls(), "producer", "consumer", "signed-result", scope(), checks, || Err::<(), _>(
            "bad signature"
        )),
        Err(SharingAdmissionError::ExistingAdmission("bad signature"))
    ));
    assert!(
        core::authorize_sharing(core::SharingRequest {
            controls: controls(),
            producer: "producer",
            consumer: "producer",
            result: "signed-result",
            scope: None,
            checks,
        })
        .is_ok()
    );
    assert_eq!(
        core::authorize_sharing(core::SharingRequest {
            controls: controls(),
            producer: "producer",
            consumer: "consumer",
            result: "signed-result",
            scope: None,
            checks,
        }),
        Err(Reason::SharingDenied)
    );
    for field in 0..6 {
        let mut broken = checks;
        let expected = match field {
            0 => {
                broken.request_and_action_match = false;
                Reason::IdentityDenied
            }
            1 => {
                broken.trusted_producer_signature = false;
                Reason::SignatureDenied
            }
            2 => {
                broken.policy_compatible = false;
                Reason::PolicyDenied
            }
            3 => {
                broken.platform_compatible = false;
                Reason::PlatformDenied
            }
            4 => {
                broken.verified_output_identities = false;
                Reason::OutputsDenied
            }
            _ => {
                broken.current_cas_available = false;
                Reason::CasUnavailable
            }
        };
        assert_eq!(
            core::authorize_sharing(core::SharingRequest {
                controls: controls(),
                producer: "producer",
                consumer: "consumer",
                result: "signed-result",
                scope: scope(),
                checks: broken,
            }),
            Err(expected)
        );
    }
}

#[test]
fn callable_adapter_rechecks_live_capacity_and_canonicalizes_replay() {
    use super::super::remote_resources::RemoteResourceRequirements;
    use super::super::remote_resources::RemoteResourceVector;
    use super::super::remote_resources::RemoteWorkerResourceInventory;
    let quantities = RemoteResourceVector {
        cpu_units: 2,
        memory_bytes: 200,
        scratch_bytes: 100,
        accelerators: vec![],
        named_tokens: vec![],
    };
    let inventory = RemoteWorkerResourceInventory {
        total: quantities.clone(),
    };
    let mut minima = quantities.clone();
    minima.memory_bytes = 100;
    let requirements = RemoteResourceRequirements {
        quantities: minima,
        semantic_accelerator_classes: vec![],
    };
    let run = |mut catalog: Vec<MachineClass>, reverse: bool| {
        if reverse {
            catalog.reverse();
        }
        let candidates = catalog
            .into_iter()
            .map(|class| CatalogCandidate {
                class,
                worker_endpoint_id: "worker",
                inventory: &inventory,
                live_leases: &[],
            })
            .collect();
        let mut observations = vec![observation()];
        evaluate_resource_policy(
            &requirements,
            &features(),
            "family",
            candidates,
            &mut observations,
            100,
            QuotaFacts {
                project_remaining_units: 100,
                account_remaining_units: 100,
            },
            policy(),
            controls(),
        )
        .unwrap()
    };
    let first = run(classes(), false);
    let replay = run(classes(), true);
    assert_eq!(first.replay_input_blake3, replay.replay_input_blake3);
    let selected = first.decision.active.unwrap();
    assert_eq!(first.classes[selected.class_index].id, "large");
    let mut oversized = classes();
    oversized[1].quantities.memory_bytes = 201;
    assert_eq!(run(oversized, false).decision.active, Err(Reason::NoClass));
}

proptest::proptest! {
    #[test]
    fn declared_minima_never_decrease_and_input_order_is_irrelevant(memory in 1_u64..=200) {
        let catalog = classes();
        let mut reversed = catalog.clone(); reversed.reverse();
        let evaluate = |classes: &[MachineClass]| core::select(core::SelectionRequest {
            declared: q(memory),
            features: &features(),
            family: "family",
            classes,
            history: &[],
            now: 100,
            quota: QuotaFacts { project_remaining_units: 100, account_remaining_units: 100 },
            policy: policy(),
        }).unwrap();
        let first = evaluate(&catalog);
        let second = evaluate(&reversed);
        proptest::prop_assert!(first.requested.fits(q(memory)));
        proptest::prop_assert_eq!(&catalog[first.class_index].id, &reversed[second.class_index].id);
        proptest::prop_assert!(first.exclusions.len() <= core::MAX_HISTORY);
    }
}

#[test]
fn missing_dimensions_and_sampled_scratch_never_become_resource_measurements() {
    let mut o = observation();
    o.measured.cpu_units = None;
    o.measured.scratch_bytes = None;
    o.scratch_sampled_peak_bytes = Some(100);
    o.queue_ms = None;
    o.io_bytes = None;
    o.transfer_bytes = None;
    let mut p = policy();
    p.margin_basis_points = 1000;
    let result = core::select(core::SelectionRequest {
        declared: q(100),
        features: &features(),
        family: "family",
        classes: &classes(),
        history: &[o.clone()],
        now: 100,
        quota: QuotaFacts {
            project_remaining_units: 100,
            account_remaining_units: 100,
        },
        policy: p,
    })
    .unwrap();
    assert_eq!(result.requested, Quantities {
        cpu_units: 2,
        memory_bytes: 165,
        scratch_bytes: 100
    });
    assert_eq!(result.reason, Reason::HistoryUsed);
    o.measured.memory_bytes = None;
    let missing = select(&classes(), &[o.clone()], 100).unwrap();
    assert_eq!(missing.requested, q(100));
    assert_eq!(missing.exclusions, vec![(0, Reason::HistoryMissing)]);
    o.schema = "mantle-resource-observation-v1".into();
    assert_eq!(select(&classes(), &[o], 100).unwrap().exclusions, vec![(0, Reason::HistoryIncompatible)]);
}

#[test]
fn sample_threshold_applies_independently_to_each_known_dimension() {
    let first = observation();
    let mut second = observation();
    second.attempt_id = RemoteAttemptId::new("second-terminal").unwrap();
    second.measured.cpu_units = None;
    second.measured.scratch_bytes = None;
    let mut p = policy();
    p.min_samples = 2;
    p.margin_basis_points = 1000;
    let result = core::select(core::SelectionRequest {
        declared: q(100),
        features: &features(),
        family: "family",
        classes: &classes(),
        history: &[first, second],
        now: 100,
        quota: QuotaFacts {
            project_remaining_units: 100,
            account_remaining_units: 100,
        },
        policy: p,
    })
    .unwrap();
    assert_eq!(result.requested.cpu_units, 2);
    assert_eq!(result.requested.scratch_bytes, 100);
    assert_eq!(result.requested.memory_bytes, 165);
}
