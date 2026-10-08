use super::*;

const DIGEST_A: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const DIGEST_B: &str = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
const STAGE_BASIC: StageRequirements = StageRequirements {
    require_action_reconciliation: false,
    require_v2_receipt: false,
};
const STAGE_LOCAL_ACTIONS: StageRequirements = StageRequirements {
    require_action_reconciliation: true,
    require_v2_receipt: false,
};
const STAGE_FINAL: StageRequirements = StageRequirements {
    require_action_reconciliation: true,
    require_v2_receipt: true,
};

fn service<'a>(id: &'a str, depends_on: &'a [&'a str], policy: Option<RestartPolicy>) -> Component<'a> {
    Component {
        id,
        kind: ComponentKind::Service,
        depends_on,
        user_states: &[],
        restart_policy: policy,
        stage_requirements: None,
    }
}

fn stage<'a>(id: &'a str, depends_on: &'a [&'a str], requirements: StageRequirements) -> Component<'a> {
    Component {
        id,
        kind: ComponentKind::ProofStage,
        depends_on,
        user_states: &[],
        restart_policy: None,
        stage_requirements: Some(requirements),
    }
}

fn observation<'a>(id: &'a str, states: &'a [&'a str], acknowledged: bool) -> Observation<'a> {
    Observation {
        id,
        generation: 1,
        states,
        exit: None,
        request_acknowledged: acknowledged,
        proof_completion: None,
    }
}

fn snapshot<'a>(components: &'a [Component<'a>], observations: &'a [Observation<'a>]) -> Snapshot<'a> {
    Snapshot {
        schema: READINESS_SCHEMA,
        components,
        observations,
    }
}

fn row<'a>(report: &'a ReadinessReport<'a>, id: &str) -> &'a ReadinessRow<'a> {
    report.components.iter().find(|item| item.id == id).expect("declared row")
}

fn error_code(input: &Snapshot<'_>) -> ErrorCode {
    evaluate(input).expect_err("input must fail admission").code
}

#[test]
fn dependent_waits_for_real_request_ack_not_started_socket_or_local_ready_flag() {
    let declarations = [
        service("daemon", &[], Some(RestartPolicy::Always)),
        service("dependent", &["daemon"], Some(RestartPolicy::Never)),
    ];
    let started = [observation("daemon", &["started"], false)];
    let report = evaluate(&snapshot(&declarations, &started)).unwrap();
    assert_eq!(row(&report, "daemon").states, ["started"]);
    assert!(!row(&report, "daemon").ready);
    assert_eq!(row(&report, "dependent").blocked_by, ["daemon"]);
    assert_eq!(may_start(&report, "dependent"), Some(false));

    let premature = [observation("daemon", &["started", "ready"], false)];
    assert_eq!(error_code(&snapshot(&declarations, &premature)), ErrorCode::MissingRequestAcknowledgement);
    let acknowledged = [observation("daemon", &["started", "ready"], true)];
    let report = evaluate(&snapshot(&declarations, &acknowledged)).unwrap();
    assert_eq!(row(&report, "daemon").states, ["started", "ready"]);
    assert!(row(&report, "daemon").ready);
    assert_eq!(may_start(&report, "dependent"), Some(true));
    assert!(row(&report, "dependent").blocked_by.is_empty());
}

#[test]
fn dependent_retracts_ready_after_predecessor_loses_readiness() {
    let declarations = [
        service("a", &[], Some(RestartPolicy::Always)),
        service("b", &["a"], Some(RestartPolicy::Never)),
    ];
    let running = [
        observation("a", &["started", "ready"], true),
        observation("b", &["started", "ready"], true),
    ];
    let report = evaluate(&snapshot(&declarations, &running)).unwrap();
    assert!(row(&report, "b").ready);

    let retracted = [
        observation("a", &["started"], true),
        observation("b", &["started", "ready"], true),
    ];
    let report = evaluate(&snapshot(&declarations, &retracted)).unwrap();
    assert_eq!(row(&report, "b").states, ["started"]);
    assert!(!row(&report, "b").ready);
    assert_eq!(row(&report, "b").blocked_by, ["a"]);
    assert_eq!(may_start(&report, "b"), Some(false));
}

#[test]
fn dependency_graph_rejects_missing_cycle_and_duplicate_edges() {
    let missing = [service("b", &["absent"], Some(RestartPolicy::Always))];
    assert_eq!(error_code(&snapshot(&missing, &[])), ErrorCode::UnknownDependency);
    let cycle = [
        service("a", &["b"], Some(RestartPolicy::Always)),
        service("b", &["a"], Some(RestartPolicy::Always)),
    ];
    assert_eq!(error_code(&snapshot(&cycle, &[])), ErrorCode::CyclicDependency);
    let duplicate = [
        service("a", &[], Some(RestartPolicy::Always)),
        service("b", &["a", "a"], Some(RestartPolicy::Always)),
    ];
    assert_eq!(error_code(&snapshot(&duplicate, &[])), ErrorCode::DuplicateDependency);
    let self_cycle = [service("self", &["self"], Some(RestartPolicy::Always))];
    assert_eq!(error_code(&snapshot(&self_cycle, &[])), ErrorCode::CyclicDependency);
}

#[test]
fn admission_rejects_missing_policy_unknown_state_and_wrong_version() {
    let missing = [service("cache", &[], None)];
    assert_eq!(error_code(&snapshot(&missing, &[])), ErrorCode::MissingPolicy);
    assert!(RestartPolicy::parse("on-success").is_none());
    assert_eq!(RestartPolicy::parse("on-error"), Some(RestartPolicy::OnError));
    let declarations = [service("cache", &[], Some(RestartPolicy::OnError))];
    let unknown = [observation("cache", &["started", "healthy"], false)];
    assert_eq!(error_code(&snapshot(&declarations, &unknown)), ErrorCode::UnknownState);
    let wrong_schema = Snapshot {
        schema: "mantle-service-readiness-v2",
        components: &declarations,
        observations: &[],
    };
    assert_eq!(error_code(&wrong_schema), ErrorCode::UnsupportedSchema);
}

#[test]
fn admission_rejects_duplicate_id_state_observation_and_reserved_user_state() {
    let duplicate = [
        service("a", &[], Some(RestartPolicy::Always)),
        service("a", &[], Some(RestartPolicy::Never)),
    ];
    assert_eq!(error_code(&snapshot(&duplicate, &[])), ErrorCode::DuplicateComponent);
    let declarations = [service("a", &[], Some(RestartPolicy::Always))];
    let duplicate_state = [observation("a", &["started", "started"], false)];
    assert_eq!(error_code(&snapshot(&declarations, &duplicate_state)), ErrorCode::DuplicateState);
    let duplicate_observation = [
        observation("a", &["started"], false),
        observation("a", &["started"], false),
    ];
    assert_eq!(error_code(&snapshot(&declarations, &duplicate_observation)), ErrorCode::DuplicateObservation);
    let invalid_user_state = [Component {
        user_states: &["ready"],
        ..service("a", &[], Some(RestartPolicy::Always))
    }];
    assert_eq!(error_code(&snapshot(&invalid_user_state, &[])), ErrorCode::InvalidUserState);
}

#[test]
fn early_exits_report_failed_and_closed_restart_matrix_matches_exit_class() {
    let expectations = [
        (RestartPolicy::Always, RestartAction::Component, RestartAction::Component),
        (RestartPolicy::OnError, RestartAction::None, RestartAction::Component),
        (RestartPolicy::All, RestartAction::None, RestartAction::Group),
        (RestartPolicy::Never, RestartAction::None, RestartAction::None),
    ];
    for (policy, normal_action, abnormal_action) in expectations {
        let declaration = [service("cache", &[], Some(policy))];
        for (exit, expected_action) in [(ExitKind::Normal, normal_action), (ExitKind::Abnormal, abnormal_action)] {
            let exited = [Observation {
                exit: Some(exit),
                ..observation("cache", &[], false)
            }];
            let report = evaluate(&snapshot(&declaration, &exited)).unwrap();
            let current = row(&report, "cache");
            assert_eq!(current.states, ["failed"], "early {exit:?} under {policy:?}");
            assert_eq!(current.restart_action, Some(expected_action));
            assert!(!current.ready);
            assert_eq!(current.restart_policy, Some(policy));
        }
    }
}

#[test]
fn acknowledged_normal_exit_completes_and_abnormal_exit_never_becomes_complete() {
    let declaration = [service("cache", &[], Some(RestartPolicy::Never))];
    let normal = [Observation {
        exit: Some(ExitKind::Normal),
        ..observation("cache", &[], true)
    }];
    let report = evaluate(&snapshot(&declaration, &normal)).unwrap();
    assert_eq!(row(&report, "cache").states, ["complete"]);
    assert_eq!(row(&report, "cache").restart_action, Some(RestartAction::None));
    let abnormal = [Observation {
        exit: Some(ExitKind::Abnormal),
        ..observation("cache", &[], true)
    }];
    let report = evaluate(&snapshot(&declaration, &abnormal)).unwrap();
    assert_eq!(row(&report, "cache").states, ["failed"]);
    let contradictory = [Observation {
        states: &["complete"],
        exit: Some(ExitKind::Abnormal),
        ..observation("cache", &[], true)
    }];
    assert_eq!(error_code(&snapshot(&declaration, &contradictory)), ErrorCode::InvalidStateCombination);
}

#[test]
fn proof_stages_report_direct_blockers_without_inventing_completion() {
    let declarations = [
        stage("stagex-transition", &[], STAGE_LOCAL_ACTIONS),
        stage("stagex-provider-publication", &["stagex-transition"], STAGE_LOCAL_ACTIONS),
        stage("full-source-native-provider", &["stagex-provider-publication"], STAGE_LOCAL_ACTIONS),
        stage("full-source-rust-provider", &["full-source-native-provider"], STAGE_LOCAL_ACTIONS),
        stage("mantle-stage1", &["full-source-native-provider", "full-source-rust-provider"], STAGE_LOCAL_ACTIONS),
        stage(
            "mantle-stage2",
            &[
                "full-source-native-provider",
                "full-source-rust-provider",
                "mantle-stage1",
            ],
            STAGE_FINAL,
        ),
    ];
    let report = evaluate(&snapshot(&declarations, &[])).unwrap();
    assert_eq!(row(&report, "stagex-provider-publication").blocked_by, ["stagex-transition"]);
    assert_eq!(row(&report, "mantle-stage1").blocked_by, [
        "full-source-native-provider",
        "full-source-rust-provider"
    ]);
    assert_eq!(row(&report, "mantle-stage2").blocked_by, [
        "full-source-native-provider",
        "full-source-rust-provider",
        "mantle-stage1"
    ]);
    assert_eq!(row(&report, "mantle-stage2").generation, None);
    assert!(row(&report, "mantle-stage2").states.is_empty());
    assert!(!row(&report, "mantle-stage2").ready);
    assert_eq!(may_start(&report, "mantle-stage2"), Some(false));
}

#[test]
fn a_new_proof_stage_cannot_start_before_an_authoritative_predecessor() {
    let declarations = [
        stage("first", &[], STAGE_LOCAL_ACTIONS),
        stage("second", &["first"], STAGE_LOCAL_ACTIONS),
    ];
    let empty = evaluate(&snapshot(&declarations, &[])).unwrap();
    let premature = [observation("second", &["started"], false)];
    let blocked = advance(&empty, &snapshot(&declarations, &premature)).unwrap_err();
    assert_eq!(blocked.code, ErrorCode::BlockedStart);
    assert_eq!(blocked.related, Some("first"));

    let first_started = [observation("first", &["started"], false)];
    let running = advance(&empty, &snapshot(&declarations, &first_started)).unwrap();
    let first_failed = [observation("first", &["failed"], false)];
    let failed = advance(&running, &snapshot(&declarations, &first_failed)).unwrap();
    assert_eq!(row(&failed, "second").blocked_by, ["first"]);
    assert_eq!(advance(&failed, &snapshot(&declarations, &premature)).unwrap_err().code, ErrorCode::BlockedStart);

    let first_complete = [Observation {
        proof_completion: Some(ProofCompletion {
            stage_evidence_digest_blake3: DIGEST_A,
            output_digest_blake3: DIGEST_B,
            execution_verified: true,
            action_reconciled: true,
            v2_receipt_verified: false,
        }),
        ..observation("first", &["complete"], false)
    }];
    let accepted = advance(&running, &snapshot(&declarations, &first_complete)).unwrap();
    let admitted = [first_complete[0], observation("second", &["started"], false)];
    let dependent = advance(&accepted, &snapshot(&declarations, &admitted)).unwrap();
    assert_eq!(row(&dependent, "second").states, ["started"]);
}

#[test]
fn proof_stage_requires_admitted_execution_digests_and_stage_specific_authority() {
    let declaration = [stage("stage", &[], STAGE_FINAL)];
    let missing = [observation("stage", &["complete"], false)];
    assert_eq!(error_code(&snapshot(&declaration, &missing)), ErrorCode::InvalidProofCompletion);
    let complete = ProofCompletion {
        stage_evidence_digest_blake3: DIGEST_A,
        output_digest_blake3: DIGEST_B,
        execution_verified: true,
        action_reconciled: true,
        v2_receipt_verified: true,
    };
    let bad_action = [Observation {
        proof_completion: Some(ProofCompletion {
            action_reconciled: false,
            ..complete
        }),
        ..missing[0]
    }];
    assert_eq!(error_code(&snapshot(&declaration, &bad_action)), ErrorCode::InvalidProofCompletion);
    let bad_receipt = [Observation {
        proof_completion: Some(ProofCompletion {
            v2_receipt_verified: false,
            ..complete
        }),
        ..missing[0]
    }];
    assert_eq!(error_code(&snapshot(&declaration, &bad_receipt)), ErrorCode::InvalidProofCompletion);
    let invalid_digest = [Observation {
        proof_completion: Some(ProofCompletion {
            output_digest_blake3: "bad",
            ..complete
        }),
        ..missing[0]
    }];
    assert_eq!(error_code(&snapshot(&declaration, &invalid_digest)), ErrorCode::InvalidProofDigest);
    let admitted = [Observation {
        proof_completion: Some(complete),
        ..missing[0]
    }];
    let report = evaluate(&snapshot(&declaration, &admitted)).unwrap();
    assert_eq!(row(&report, "stage").states, ["complete"]);
    assert!(!row(&report, "stage").ready);
    assert_eq!(row(&report, "stage").restart_policy, None);
}

#[test]
fn even_claimed_proof_completion_is_rejected_if_predecessor_is_blocked() {
    let declarations = [stage("a", &[], STAGE_BASIC), stage("b", &["a"], STAGE_BASIC)];
    let claimed = [Observation {
        proof_completion: Some(ProofCompletion {
            stage_evidence_digest_blake3: DIGEST_A,
            output_digest_blake3: DIGEST_B,
            execution_verified: true,
            action_reconciled: false,
            v2_receipt_verified: false,
        }),
        ..observation("b", &["complete"], false)
    }];
    let err = evaluate(&snapshot(&declarations, &claimed)).unwrap_err();
    assert_eq!(err.code, ErrorCode::BlockedCompletion);
    assert_eq!(err.component, Some("b"));
    assert_eq!(err.related, Some("a"));
}

#[test]
fn restart_generation_cannot_inherit_a_predecessors_acknowledgement() {
    let declarations = [
        service("daemon", &[], Some(RestartPolicy::Always)),
        service("worker", &["daemon"], Some(RestartPolicy::Never)),
    ];
    let before = [Observation {
        generation: 1,
        ..observation("daemon", &["started", "ready"], true)
    }];
    let report = evaluate(&snapshot(&declarations, &before)).unwrap();
    assert!(row(&report, "daemon").ready);
    assert_eq!(may_start(&report, "worker"), Some(true));

    let exiting = [Observation {
        generation: 1,
        exit: Some(ExitKind::Abnormal),
        ..observation("daemon", &[], true)
    }];
    let report = evaluate(&snapshot(&declarations, &exiting)).unwrap();
    assert_eq!(row(&report, "daemon").states, ["failed"]);
    assert_eq!(row(&report, "daemon").restart_action, Some(RestartAction::Component));
    assert_eq!(row(&report, "worker").blocked_by, ["daemon"]);

    let restarted = [Observation {
        generation: 2,
        ..observation("daemon", &["started"], false)
    }];
    let report = evaluate(&snapshot(&declarations, &restarted)).unwrap();
    assert_eq!(row(&report, "daemon").generation, Some(2));
    assert!(!row(&report, "daemon").ready);
    assert_eq!(may_start(&report, "worker"), Some(false));
    let premature = [Observation {
        generation: 2,
        ..observation("daemon", &["started", "ready"], false)
    }];
    assert_eq!(error_code(&snapshot(&declarations, &premature)), ErrorCode::MissingRequestAcknowledgement);
}

#[test]
fn dependency_evaluation_is_order_independent_and_reports_direct_blockers() {
    let reversed = [
        service("z", &["a", "m"], Some(RestartPolicy::Never)),
        service("m", &[], Some(RestartPolicy::Always)),
        service("a", &[], Some(RestartPolicy::Always)),
    ];
    let observed = [
        observation("m", &["started"], false),
        observation("a", &["started"], false),
    ];
    let report = evaluate(&snapshot(&reversed, &observed)).unwrap();
    assert_eq!(report.components.iter().map(|entry| entry.id).collect::<std::vec::Vec<_>>(), ["a", "m", "z"]);
    assert_eq!(row(&report, "z").blocked_by, ["a", "m"]);
    assert_eq!(may_start(&report, "z"), Some(false));
    assert_eq!(may_start(&report, "unknown"), None);
}

#[test]
fn proof_terminal_state_cannot_coexist_with_custom_state_or_unverified_execution() {
    let declaration = [Component {
        user_states: &["running"],
        ..stage("stage", &[], STAGE_BASIC)
    }];
    let mixed = [observation("stage", &["failed", "running"], false)];
    assert_eq!(error_code(&snapshot(&declaration, &mixed)), ErrorCode::InvalidStateCombination);
    let invalid = [Observation {
        proof_completion: Some(ProofCompletion {
            stage_evidence_digest_blake3: DIGEST_A,
            output_digest_blake3: DIGEST_B,
            execution_verified: false,
            action_reconciled: false,
            v2_receipt_verified: false,
        }),
        ..observation("stage", &["complete"], false)
    }];
    assert_eq!(error_code(&snapshot(&declaration, &invalid)), ErrorCode::InvalidProofCompletion);
}

#[test]
fn declared_user_state_is_preserved_but_cannot_satisfy_a_dependency() {
    let declarations = [
        Component {
            user_states: &["warming"],
            ..service("cache", &[], Some(RestartPolicy::OnError))
        },
        service("consumer", &["cache"], Some(RestartPolicy::Never)),
    ];
    let warm = [observation("cache", &["started", "warming"], false)];
    let report = evaluate(&snapshot(&declarations, &warm)).unwrap();
    assert_eq!(row(&report, "cache").states, ["started", "warming"]);
    assert_eq!(row(&report, "consumer").blocked_by, ["cache"]);
    assert_eq!(may_start(&report, "consumer"), Some(false));
    let acknowledged = [observation("cache", &["started", "warming", "ready"], true)];
    let report = evaluate(&snapshot(&declarations, &acknowledged)).unwrap();
    assert_eq!(may_start(&report, "consumer"), Some(true));
}

#[test]
fn excessive_component_count_is_rejected_before_duplicate_identity() {
    let components = std::vec![service("same", &[], Some(RestartPolicy::Always)); MAX_COMPONENTS as usize + 1];
    let input = snapshot(&components, &[]);
    assert_eq!(error_code(&input), ErrorCode::TooManyComponents);
}

#[test]
fn transition_waits_for_a_real_ack_then_allows_the_dependent_to_start() {
    let declarations = [
        service("daemon", &[], Some(RestartPolicy::Always)),
        service("worker", &["daemon"], Some(RestartPolicy::Never)),
    ];
    let empty = evaluate(&snapshot(&declarations, &[])).unwrap();
    let premature = [observation("daemon", &["started", "ready"], true)];
    assert_eq!(advance(&empty, &snapshot(&declarations, &premature)).unwrap_err().code, ErrorCode::UnobservedStart);

    let daemon_started = [observation("daemon", &["started"], false)];
    let started = advance(&empty, &snapshot(&declarations, &daemon_started)).unwrap();
    let worker_too_early = [
        observation("daemon", &["started"], false),
        observation("worker", &["started"], false),
    ];
    let blocked = advance(&started, &snapshot(&declarations, &worker_too_early)).unwrap_err();
    assert_eq!(blocked.code, ErrorCode::BlockedStart);
    assert_eq!(blocked.related, Some("daemon"));

    let daemon_ack = [observation("daemon", &["started", "ready"], true)];
    let ready = advance(&started, &snapshot(&declarations, &daemon_ack)).unwrap();
    let worker_started = [
        observation("daemon", &["started", "ready"], true),
        observation("worker", &["started"], false),
    ];
    let running = advance(&ready, &snapshot(&declarations, &worker_started)).unwrap();
    assert_eq!(row(&running, "worker").states, ["started"]);
    assert!(!row(&running, "worker").ready);
    let worker_ack = [
        observation("daemon", &["started", "ready"], true),
        observation("worker", &["started", "ready"], true),
    ];
    let operating = advance(&running, &snapshot(&declarations, &worker_ack)).unwrap();
    assert!(row(&operating, "worker").ready);
}

#[test]
fn transition_requires_terminal_observation_and_fresh_started_generation() {
    let declarations = [service("daemon", &[], Some(RestartPolicy::Always))];
    let empty = evaluate(&snapshot(&declarations, &[])).unwrap();
    let started_input = [observation("daemon", &["started"], false)];
    let started = advance(&empty, &snapshot(&declarations, &started_input)).unwrap();
    assert_eq!(advance(&started, &snapshot(&declarations, &[])).unwrap_err().code, ErrorCode::UnobservedExit);
    let jumped = [Observation {
        generation: 2,
        ..observation("daemon", &["started"], false)
    }];
    assert_eq!(advance(&started, &snapshot(&declarations, &jumped)).unwrap_err().code, ErrorCode::UnobservedExit);
    let exited_input = [Observation {
        exit: Some(ExitKind::Abnormal),
        ..observation("daemon", &[], false)
    }];
    let failed = advance(&started, &snapshot(&declarations, &exited_input)).unwrap();
    assert_eq!(row(&failed, "daemon").states, ["failed"]);
    assert_eq!(advance(&failed, &snapshot(&declarations, &[])).unwrap_err().code, ErrorCode::UnobservedExit);
    let revived = [observation("daemon", &["started"], false)];
    assert_eq!(advance(&failed, &snapshot(&declarations, &revived)).unwrap_err().code, ErrorCode::RevivedTerminal);
    let stale = [Observation {
        generation: 0,
        ..observation("daemon", &[], false)
    }];
    assert_eq!(advance(&failed, &snapshot(&declarations, &stale)).unwrap_err().code, ErrorCode::InvalidGeneration);
    let next = [Observation {
        generation: 2,
        ..observation("daemon", &["started"], false)
    }];
    let restarted = advance(&failed, &snapshot(&declarations, &next)).unwrap();
    assert_eq!(row(&restarted, "daemon").generation, Some(2));
    assert!(!row(&restarted, "daemon").ready);
    assert_eq!(
        advance(&restarted, &snapshot(&declarations, &started_input)).unwrap_err().code,
        ErrorCode::StaleGeneration
    );

    let never = [service("daemon", &[], Some(RestartPolicy::Never))];
    let initial = evaluate(&snapshot(&never, &[])).unwrap();
    let running = advance(&initial, &snapshot(&never, &started_input)).unwrap();
    let terminal = advance(&running, &snapshot(&never, &exited_input)).unwrap();
    assert_eq!(advance(&terminal, &snapshot(&never, &next)).unwrap_err().code, ErrorCode::RestartDenied);
}

#[test]
fn a_terminal_exit_class_cannot_be_rewritten_to_change_restart_decision() {
    let declarations = [service("daemon", &[], Some(RestartPolicy::OnError))];
    let empty = evaluate(&snapshot(&declarations, &[])).unwrap();
    let started_input = [observation("daemon", &["started"], false)];
    let started = advance(&empty, &snapshot(&declarations, &started_input)).unwrap();
    let normal = [Observation {
        exit: Some(ExitKind::Normal),
        ..observation("daemon", &[], false)
    }];
    let terminal = advance(&started, &snapshot(&declarations, &normal)).unwrap();
    assert_eq!(row(&terminal, "daemon").states, ["failed"]);
    assert_eq!(row(&terminal, "daemon").restart_action, Some(RestartAction::None));
    let abnormal = [Observation {
        exit: Some(ExitKind::Abnormal),
        ..observation("daemon", &[], false)
    }];
    assert_eq!(
        advance(&terminal, &snapshot(&declarations, &abnormal)).unwrap_err().code,
        ErrorCode::RevivedTerminal
    );
}

#[test]
fn transition_does_not_accept_an_evidence_labeled_previous_report() {
    let declarations = [service("daemon", &[], Some(RestartPolicy::Always))];
    let mut previous = evaluate(&snapshot(&declarations, &[])).unwrap();
    previous.evidence_eligible = true;
    let started = [observation("daemon", &["started"], false)];
    assert_eq!(
        advance(&previous, &snapshot(&declarations, &started)).unwrap_err().code,
        ErrorCode::InvalidPreviousReport
    );
}
