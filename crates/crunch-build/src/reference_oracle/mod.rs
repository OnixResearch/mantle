//! Bounded reference evaluator, differential `proptest` suite, mutation checks,
//! and `#40` Comment 1 Nickel composition fixtures.

mod evaluator;
mod harness;
mod nickel_fixtures;
mod task_language;

use std::collections::BTreeMap;
use std::collections::BTreeSet;

use nix_compat::store_path::StorePath;
use proptest::prelude::*;
use proptest::test_runner::Config as ProptestConfig;
use proptest::test_runner::RngAlgorithm;
use proptest::test_runner::RngSeed;
use snix_build::buildservice::BuildService as _;
use snix_castore::Node;

use self::evaluator::ReferenceResult;
use self::evaluator::evaluate_reference;
use self::harness::InstrumentedEnv;
use self::harness::lower_task_graph;
use self::harness::run_mutation_check_a_coarse_recipe_key;
use self::harness::run_mutation_check_b_dropped_dependency_wait;
use self::harness::run_mutation_check_c_skipped_admission_condition;
use self::harness::verify_scenario_1_with_converter;
use self::harness::verify_scenario_2_diamond_and_duplicate_roots;
use self::harness::verify_scenario_3_ca_early_cutoff;
use self::harness::verify_scenario_4_dynamic_plan;
use self::harness::verify_scenario_5_missing_and_incomplete_objects;
use self::harness::verify_scenario_6_tampered_and_conflicting;
use self::harness::verify_scenario_7_watch_retraction_and_cancellation;
use self::harness::verify_scenario_8_cycles_and_bounds;
use self::harness::verify_static_graph_differential;
use self::task_language::GraphBounds;
use self::task_language::GraphFault;
use self::task_language::OutputRule;
use self::task_language::ReferenceTarget;
use self::task_language::RootRequest;
use self::task_language::SourceId;
use self::task_language::SourceSpec;
use self::task_language::TaskAddressingMode;
use self::task_language::TaskDep;
use self::task_language::TaskGraph;
use self::task_language::TaskId;
use self::task_language::TaskNode;
use self::task_language::arb_ca_chain_graph;
use self::task_language::arb_dynamic_task_graph;
use self::task_language::arb_static_task_graph;
use crate::ExecutionProfile;
use crate::HermeticityMode;
use crate::Worker;
use crate::derivation_to_build_request;

fn run_async<F: std::future::Future>(fut: F) -> F::Output {
    let rt = tokio::runtime::Builder::new_current_thread().enable_all().build().expect("tokio rt");
    rt.block_on(fut)
}

#[test]
fn scenario_1_same_name_builder_system_different_recipe_both_orders() {
    run_async(async {
        verify_scenario_1_with_converter(false, crunch_glue::convert).await.expect("order A->B");
        verify_scenario_1_with_converter(true, crunch_glue::convert).await.expect("order B->A");
    });
}

#[test]
fn scenario_2_diamond_and_duplicate_root_requests() {
    run_async(async {
        verify_scenario_2_diamond_and_duplicate_roots().await.expect("diamond + duplicate roots");
    });
}

#[test]
fn scenario_3_ca_early_cutoff_and_negative_control() {
    let build_pair = |same_realized_ca: bool| {
        let mk = |salt: u64, realized: Vec<u8>| {
            let mut p = TaskGraph::simple_task(0, "ca-p", salt, TaskAddressingMode::ContentAddressed);
            p.output_rules.get_mut("out").expect("out").ca_content_override = Some(realized);
            let mut c = TaskGraph::simple_task(1, "ca-c", 20, TaskAddressingMode::ContentAddressed);
            c.deps.push(TaskDep::AllOutputs { target: TaskId(0) });
            let mut g = TaskGraph::simple_task(2, "ia-g", 30, TaskAddressingMode::InputAddressed);
            g.deps.push(TaskDep::AllOutputs { target: TaskId(1) });
            TaskGraph {
                store_dir: "/nix/store".to_string(),
                sources: Vec::new(),
                tasks: vec![p, c, g],
                roots: vec![RootRequest {
                    task: TaskId(2),
                    selected_outputs: None,
                    arrival_batch: 0,
                }],
                fault: GraphFault::None,
            }
        };
        let v1 = mk(10, b"ca-stable".to_vec());
        let v2 = mk(
            99,
            if same_realized_ca {
                b"ca-stable".to_vec()
            } else {
                b"ca-changed".to_vec()
            },
        );
        (v1, v2)
    };

    run_async(async {
        let (v1, v2_same) = build_pair(true);
        verify_scenario_3_ca_early_cutoff(&v1, &v2_same, true).await.expect("CA early cutoff reuse");

        let (v1b, v2_diff) = build_pair(false);
        verify_scenario_3_ca_early_cutoff(&v1b, &v2_diff, false).await.expect("CA changed output rebuild");
    });
}

#[test]
fn scenario_4_dynamic_plan_fresh_vs_cached_and_wait_invariant() {
    let mut runner = proptest::test_runner::TestRunner::new(ProptestConfig {
        cases: 4,
        failure_persistence: None,
        rng_algorithm: RngAlgorithm::ChaCha,
        rng_seed: RngSeed::Fixed(0x4d41_4e54_4c45_0004),
        ..ProptestConfig::default()
    });
    runner
        .run(&arb_dynamic_task_graph(), |graph| {
            run_async(verify_scenario_4_dynamic_plan(&graph)).map_err(TestCaseError::fail)
        })
        .expect("dynamic plan fresh vs cached");
}

#[test]
fn scenario_5_missing_intermediate_or_incomplete_output_objects() {
    run_async(async {
        verify_scenario_5_missing_and_incomplete_objects()
            .await
            .expect("missing/incomplete output reconstruction");
    });
}

#[test]
fn scenario_6_tampered_and_conflicting_candidates_all_orders() {
    run_async(async {
        verify_scenario_6_tampered_and_conflicting(false, false).await.expect("normal candidate order");
        verify_scenario_6_tampered_and_conflicting(false, true).await.expect("reversed candidate order");
    });
}

#[test]
fn scenario_7_watch_retraction_and_late_completion_discard() {
    run_async(async {
        verify_scenario_7_watch_retraction_and_cancellation()
            .await
            .expect("watch retraction discards late completion");
    });
}

#[test]
fn scenario_8_invalid_cycles_bounds_and_exhaustion_rejection() {
    run_async(async {
        verify_scenario_8_cycles_and_bounds().await.expect("cycles/bounds/exhaustion rejection");
    });

    // Also verify ExceedTaskBound, ExceedDepthBound, and Source reference target in evaluate_reference.
    let bounds = GraphBounds::default();
    let mut bound_graph = TaskGraph {
        store_dir: "/nix/store".to_string(),
        sources: vec![self::task_language::SourceSpec {
            id: self::task_language::SourceId(0),
            name: "src-0".to_string(),
            bytes: b"src-bytes".to_vec(),
        }],
        tasks: vec![{
            let mut t = TaskGraph::simple_task(0, "src-user", 1, TaskAddressingMode::InputAddressed);
            t.source_inputs.push(self::task_language::SourceId(0));
            t.output_rules
                .get_mut("out")
                .expect("out")
                .reference_targets
                .push(ReferenceTarget::Source(self::task_language::SourceId(0)));
            t
        }],
        roots: vec![RootRequest {
            task: TaskId(0),
            selected_outputs: None,
            arrival_batch: 0,
        }],
        fault: GraphFault::None,
    };
    assert!(matches!(evaluate_reference(&bound_graph, &bounds), ReferenceResult::Evaluated(_)));
    bound_graph.fault = GraphFault::ExceedTaskBound;
    assert!(matches!(evaluate_reference(&bound_graph, &bounds), ReferenceResult::Rejected(_)));
    bound_graph.fault = GraphFault::ExceedDepthBound;
    assert!(matches!(evaluate_reference(&bound_graph, &bounds), ReferenceResult::Rejected(_)));
}

#[test]
fn selected_output_demand_and_late_reopening_with_separate_metrics() {
    run_async(async {
        let mut multi_rules = BTreeMap::new();
        multi_rules.insert("out".to_string(), OutputRule {
            salt: 11,
            ..OutputRule::default()
        });
        multi_rules.insert("dev".to_string(), OutputRule {
            salt: 22,
            ..OutputRule::default()
        });
        multi_rules.insert("lib".to_string(), OutputRule {
            salt: 33,
            reference_targets: vec![ReferenceTarget::SiblingOutput("dev".to_string())],
            ..OutputRule::default()
        });
        let multi = TaskNode {
            id: TaskId(0),
            name: "sdk-multi-out".to_string(),
            builder: "/bin/sh".to_string(),
            system: "x86_64-linux".to_string(),
            salt: 11,
            extra_env: BTreeMap::new(),
            outputs: vec!["out".to_string(), "dev".to_string(), "lib".to_string()],
            addressing_mode: TaskAddressingMode::InputAddressed,
            source_inputs: Vec::new(),
            deps: Vec::new(),
            dynamic_plan_outputs: Vec::new(),
            output_rules: multi_rules,
            dynamic_plan_rule: None,
        };
        let graph = TaskGraph {
            store_dir: "/nix/store".to_string(),
            sources: Vec::new(),
            tasks: vec![multi],
            roots: vec![RootRequest {
                task: TaskId(0),
                selected_outputs: Some(BTreeSet::from(["out".to_string()])),
                arrival_batch: 0,
            }],
            fault: GraphFault::None,
        };

        let env = InstrumentedEnv::new(&graph.store_dir);
        let mut lowered = lower_task_graph(&graph, &BTreeMap::new()).expect("lower");
        let drv_sp = lowered.root_drv_paths[0].clone();
        let entry = lowered
            .registry
            .get_by_drv_path(&drv_sp.to_absolute_path_with_prefix(&graph.store_dir))
            .expect("entry");
        let out_sp = entry.derivation.outputs["out"].path.clone().expect("out path");
        let dev_sp = entry.derivation.outputs["dev"].path.clone().expect("dev path");

        // Build all outputs cold first, then erase `dev` from PIS and local AR index
        // so `check_cache` exercises `#35` demand-projected lookup without probing `dev`!
        let mut b1 = env.make_builder(false);
        let mut w1 = Worker::new(1);
        w1.want(&drv_sp, &lowered.registry, true).expect("want all");
        let _ = w1.run(&mut b1, &mut lowered.registry).await.expect("cold build");

        let calls = env.build_service.recorded_calls();
        assert_eq!(calls.len(), 1);
        assert!(!calls[0].command_args.is_empty());
        assert_eq!(calls[0].outputs.len(), 3);
        assert!(calls[0].input_components.is_empty());
        assert!(!calls[0].was_cancellable);
        assert!(!calls[0].cancelled_during_run);

        let _ = std::fs::remove_dir_all(env.state_dir.join("action-results"));
        let _ = env.pis.remove_for_test(&dev_sp);
        env.reset_metrics();

        // Demanding only `out` succeeds from cache with 0 executor calls and 0 probes of `dev`!
        let mut b2 = env.make_builder(false);
        let mut w2 = Worker::new(1);
        w2.want_outputs(&drv_sp, &lowered.registry, true, Some(BTreeSet::from(["out".to_string()])))
            .expect("want out");
        let res2 = w2.run(&mut b2, &mut lowered.registry).await.expect("projected hit");
        assert!(res2.outcomes[0].cached);
        assert_eq!(env.metrics().executor_calls, 0);
        assert!(env.pis.get_count(&out_sp) >= 1);
        assert_eq!(env.pis.get_count(&dev_sp), 0, "unselected unreferenced `dev` must not be probed");

        // Late reopening within a single streaming invocation: demand `out` first, then
        // arrive with full-output EvalMessage requiring `dev` -> must execute once for `dev`.
        let mut b3 = env.make_builder(false);
        let mut w3 = Worker::new(1);
        w3.want_outputs(&drv_sp, &lowered.registry, true, Some(BTreeSet::from(["out".to_string()])))
            .expect("want out first");
        let (tx, mut rx) = tokio::sync::mpsc::channel(2);
        let drv_sp_late = drv_sp.clone();
        let sender = tokio::spawn(async move {
            tokio::time::sleep(std::time::Duration::from_millis(15)).await;
            let _ = tx
                .send(crate::EvalMessage {
                    label: "sdk-multi-late".to_string(),
                    drv_path: drv_sp_late,
                    new_entries: Vec::new(),
                })
                .await;
        });
        let res3 = w3.run_streaming(&mut b3, &mut lowered.registry, &mut rx).await.expect("reopened streaming run");
        sender.await.expect("sender task");
        assert!(res3.outcomes[0].outputs.contains_key("dev"));
        assert_eq!(env.metrics().executor_calls, 1);
    });
}

#[test]
fn mutation_check_a_catches_coarse_recipe_key() {
    run_async(async {
        run_mutation_check_a_coarse_recipe_key().await.expect("mutation check (a) must pass");
    });
}

#[test]
fn mutation_check_b_catches_dropped_dependency_wait() {
    run_async(async {
        run_mutation_check_b_dropped_dependency_wait().await.expect("mutation check (b) must pass");
    });
}

#[test]
fn mutation_check_c_catches_skipped_admission_condition() {
    run_async(async {
        run_mutation_check_c_skipped_admission_condition().await.expect("mutation check (c) must pass");
    });
}

#[test]
fn scenario_8_closure_bound_exhaustion_returns_exhausted() {
    let tight_bounds = GraphBounds {
        max_tasks: 2,
        max_edges: 2,
        max_depth: 2,
        max_dynamic_units: 0,
        max_eval_steps: 512,
    };
    let outputs: Vec<String> = (0..70).map(|i| format!("o{i}")).collect();
    let mut t0_rules = BTreeMap::new();
    for (idx, out_name) in outputs.iter().enumerate() {
        t0_rules.insert(out_name.clone(), OutputRule {
            salt: idx as u64,
            ..OutputRule::default()
        });
    }
    let t0 = TaskNode {
        id: TaskId(0),
        name: "wide-outputs".to_string(),
        builder: "/bin/sh".to_string(),
        system: "x86_64-linux".to_string(),
        salt: 1,
        extra_env: BTreeMap::new(),
        outputs,
        addressing_mode: TaskAddressingMode::InputAddressed,
        source_inputs: Vec::new(),
        deps: Vec::new(),
        dynamic_plan_outputs: Vec::new(),
        output_rules: t0_rules,
        dynamic_plan_rule: None,
    };
    let mut t1 = TaskGraph::simple_task(1, "consumer", 2, TaskAddressingMode::InputAddressed);
    t1.deps.push(TaskDep::AllOutputs { target: TaskId(0) });
    let graph = TaskGraph {
        store_dir: "/nix/store".to_string(),
        sources: Vec::new(),
        tasks: vec![t0, t1],
        roots: vec![RootRequest {
            task: TaskId(1),
            selected_outputs: None,
            arrival_batch: 0,
        }],
        fault: GraphFault::None,
    };
    assert!(matches!(evaluate_reference(&graph, &tight_bounds), ReferenceResult::Exhausted(_)));
}

#[test]
fn regression_shared_name_reviewer_ia_ca_both_orders() {
    let build_graph = |target_idx: u32| {
        let t0 = TaskGraph::simple_task(0, "shared-name", 10, TaskAddressingMode::InputAddressed);
        let t1 = TaskGraph::simple_task(1, "shared-name", 2, TaskAddressingMode::ContentAddressed);
        let mut t2 = TaskGraph::simple_task(2, "consumer", 3, TaskAddressingMode::InputAddressed);
        t2.deps = if target_idx == 1 {
            vec![TaskDep::AllOutputs { target: TaskId(1) }, TaskDep::AllOutputs {
                target: TaskId(0),
            }]
        } else {
            vec![TaskDep::AllOutputs { target: TaskId(0) }, TaskDep::AllOutputs {
                target: TaskId(1),
            }]
        };
        t2.output_rules.get_mut("out").expect("out").reference_targets.push(ReferenceTarget::DepOutput {
            dep_index: 0,
            output: "out".to_string(),
        });
        TaskGraph {
            store_dir: "/nix/store".to_string(),
            sources: Vec::new(),
            tasks: vec![t0, t1, t2],
            roots: vec![RootRequest {
                task: TaskId(2),
                selected_outputs: None,
                arrival_batch: 0,
            }],
            fault: GraphFault::None,
        }
    };
    run_async(async {
        verify_static_graph_differential(&build_graph(1), &GraphBounds::default(), false)
            .await
            .expect("IA+CA shared-name targeting CA TaskId(1)");
        verify_static_graph_differential(&build_graph(0), &GraphBounds::default(), false)
            .await
            .expect("IA+CA shared-name targeting IA TaskId(0)");
    });
}

#[test]
fn regression_shared_name_challenger_four_node_counterexample() {
    let mut t0_rules = BTreeMap::new();
    t0_rules.insert("out".to_string(), OutputRule {
        salt: 1239,
        reference_targets: vec![ReferenceTarget::SiblingOutput("dev".to_string())],
        ..OutputRule::default()
    });
    t0_rules.insert("dev".to_string(), OutputRule {
        salt: 1240,
        ..OutputRule::default()
    });
    let t0 = TaskNode {
        id: TaskId(0),
        name: "task-0".to_string(),
        builder: "/bin/sh".to_string(),
        system: "x86_64-linux".to_string(),
        salt: 1239,
        extra_env: BTreeMap::new(),
        outputs: vec!["out".to_string(), "dev".to_string()],
        addressing_mode: TaskAddressingMode::InputAddressed,
        source_inputs: Vec::new(),
        deps: Vec::new(),
        dynamic_plan_outputs: Vec::new(),
        output_rules: t0_rules,
        dynamic_plan_rule: None,
    };

    let mut t1 = TaskGraph::simple_task(1, "shared-name", 1, TaskAddressingMode::InputAddressed);
    t1.deps.push(TaskDep::AllOutputs { target: TaskId(0) });
    t1.output_rules.get_mut("out").expect("out").reference_targets.push(ReferenceTarget::DepOutput {
        dep_index: 0,
        output: "out".to_string(),
    });

    let t2 = TaskGraph::simple_task(2, "shared-name", 9676, TaskAddressingMode::InputAddressed);

    let mut t3 = TaskGraph::simple_task(3, "task-3", 7079, TaskAddressingMode::InputAddressed);
    t3.deps = vec![TaskDep::AllOutputs { target: TaskId(2) }, TaskDep::AllOutputs {
        target: TaskId(1),
    }];
    t3.output_rules.get_mut("out").expect("out").reference_targets.push(ReferenceTarget::DepOutput {
        dep_index: 0,
        output: "out".to_string(),
    });

    let graph = TaskGraph {
        store_dir: "/nix/store".to_string(),
        sources: Vec::new(),
        tasks: vec![t0, t1, t2, t3],
        roots: vec![RootRequest {
            task: TaskId(3),
            selected_outputs: None,
            arrival_batch: 0,
        }],
        fault: GraphFault::None,
    };
    run_async(verify_static_graph_differential(&graph, &GraphBounds::default(), false))
        .expect("4-node shared-name diamond DepOutput reference must resolve to TaskId(2)");
}

#[test]
fn regression_shared_name_ia_ia_and_identical_content_ia_ca() {
    // 1. IA + IA with identical content_prefix via ca_content_override.
    let mut ia0 = TaskGraph::simple_task(0, "shared-name", 11, TaskAddressingMode::InputAddressed);
    ia0.output_rules.get_mut("out").expect("out").ca_content_override = Some(b"identical-ia-bytes".to_vec());
    let mut ia1 = TaskGraph::simple_task(1, "shared-name", 22, TaskAddressingMode::InputAddressed);
    ia1.output_rules.get_mut("out").expect("out").ca_content_override = Some(b"identical-ia-bytes".to_vec());
    let mut ia_consumer = TaskGraph::simple_task(2, "ia-ia-consumer", 33, TaskAddressingMode::InputAddressed);
    ia_consumer.deps = vec![TaskDep::AllOutputs { target: TaskId(0) }, TaskDep::AllOutputs {
        target: TaskId(1),
    }];
    ia_consumer.output_rules.get_mut("out").expect("out").reference_targets = vec![
        ReferenceTarget::DepOutput {
            dep_index: 0,
            output: "out".to_string(),
        },
        ReferenceTarget::DepOutput {
            dep_index: 1,
            output: "out".to_string(),
        },
    ];
    let ia_ia_graph = TaskGraph {
        store_dir: "/nix/store".to_string(),
        sources: Vec::new(),
        tasks: vec![ia0, ia1, ia_consumer],
        roots: vec![RootRequest {
            task: TaskId(2),
            selected_outputs: None,
            arrival_batch: 0,
        }],
        fault: GraphFault::None,
    };

    // 2. IA + CA with identical OutputRule.salt and empty deps -> identical content_prefix!
    let build_identical_ia_ca = |ca_first: bool| {
        let ia = TaskGraph::simple_task(0, "shared-name", 42, TaskAddressingMode::InputAddressed);
        let ca = TaskGraph::simple_task(1, "shared-name", 42, TaskAddressingMode::ContentAddressed);
        let mut consumer = TaskGraph::simple_task(2, "ia-ca-identical", 99, TaskAddressingMode::InputAddressed);
        consumer.deps = if ca_first {
            vec![TaskDep::AllOutputs { target: TaskId(1) }, TaskDep::AllOutputs {
                target: TaskId(0),
            }]
        } else {
            vec![TaskDep::AllOutputs { target: TaskId(0) }, TaskDep::AllOutputs {
                target: TaskId(1),
            }]
        };
        consumer.output_rules.get_mut("out").expect("out").reference_targets = vec![
            ReferenceTarget::DepOutput {
                dep_index: 0,
                output: "out".to_string(),
            },
            ReferenceTarget::DepOutput {
                dep_index: 1,
                output: "out".to_string(),
            },
        ];
        TaskGraph {
            store_dir: "/nix/store".to_string(),
            sources: Vec::new(),
            tasks: vec![ia, ca, consumer],
            roots: vec![RootRequest {
                task: TaskId(2),
                selected_outputs: None,
                arrival_batch: 0,
            }],
            fault: GraphFault::None,
        }
    };

    run_async(async {
        verify_static_graph_differential(&ia_ia_graph, &GraphBounds::default(), false)
            .await
            .expect("IA+IA identical content must disambiguate");
        verify_static_graph_differential(&build_identical_ia_ca(true), &GraphBounds::default(), false)
            .await
            .expect("IA+CA identical content (CA first) must disambiguate");
        verify_static_graph_differential(&build_identical_ia_ca(false), &GraphBounds::default(), false)
            .await
            .expect("IA+CA identical content (IA first) must disambiguate");
    });
}

#[test]
fn regression_shared_name_ca_ca_distinct_and_identical_content_diff_refs() {
    // 1. CA + CA with distinct salts.
    let ca_a = TaskGraph::simple_task(0, "shared-name", 101, TaskAddressingMode::ContentAddressed);
    let ca_b = TaskGraph::simple_task(1, "shared-name", 202, TaskAddressingMode::ContentAddressed);
    let mut ca_distinct_consumer = TaskGraph::simple_task(2, "ca-ca-distinct", 303, TaskAddressingMode::InputAddressed);
    ca_distinct_consumer.deps = vec![TaskDep::AllOutputs { target: TaskId(1) }, TaskDep::AllOutputs {
        target: TaskId(0),
    }];
    ca_distinct_consumer.output_rules.get_mut("out").expect("out").reference_targets = vec![
        ReferenceTarget::DepOutput {
            dep_index: 0,
            output: "out".to_string(),
        },
        ReferenceTarget::DepOutput {
            dep_index: 1,
            output: "out".to_string(),
        },
    ];
    let ca_distinct_graph = TaskGraph {
        store_dir: "/nix/store".to_string(),
        sources: Vec::new(),
        tasks: vec![ca_a, ca_b, ca_distinct_consumer],
        roots: vec![RootRequest {
            task: TaskId(2),
            selected_outputs: None,
            arrival_batch: 0,
        }],
        fault: GraphFault::None,
    };

    // 2. CA + CA with identical content_prefix (`ca_content_override`) and distinct `Source` refs!
    let build_ca_diff_refs = |second_first: bool| {
        let mut t0 = TaskGraph::simple_task(0, "shared-name", 1, TaskAddressingMode::ContentAddressed);
        t0.source_inputs.push(SourceId(0));
        {
            let rule = t0.output_rules.get_mut("out").expect("out");
            rule.ca_content_override = Some(b"identical-ca-prefix".to_vec());
            rule.reference_targets.push(ReferenceTarget::Source(SourceId(0)));
        }
        let mut t1 = TaskGraph::simple_task(1, "shared-name", 2, TaskAddressingMode::ContentAddressed);
        t1.source_inputs.push(SourceId(1));
        {
            let rule = t1.output_rules.get_mut("out").expect("out");
            rule.ca_content_override = Some(b"identical-ca-prefix".to_vec());
            rule.reference_targets.push(ReferenceTarget::Source(SourceId(1)));
        }
        let mut consumer = TaskGraph::simple_task(2, "ca-diff-refs-consumer", 3, TaskAddressingMode::InputAddressed);
        consumer.deps = if second_first {
            vec![TaskDep::AllOutputs { target: TaskId(1) }, TaskDep::AllOutputs {
                target: TaskId(0),
            }]
        } else {
            vec![TaskDep::AllOutputs { target: TaskId(0) }, TaskDep::AllOutputs {
                target: TaskId(1),
            }]
        };
        consumer.output_rules.get_mut("out").expect("out").reference_targets = vec![
            ReferenceTarget::DepOutput {
                dep_index: 0,
                output: "out".to_string(),
            },
            ReferenceTarget::DepOutput {
                dep_index: 1,
                output: "out".to_string(),
            },
        ];
        TaskGraph {
            store_dir: "/nix/store".to_string(),
            sources: vec![
                SourceSpec {
                    id: SourceId(0),
                    name: "ref-src-a".to_string(),
                    bytes: b"payload-a".to_vec(),
                },
                SourceSpec {
                    id: SourceId(1),
                    name: "ref-src-b".to_string(),
                    bytes: b"payload-b".to_vec(),
                },
            ],
            tasks: vec![t0, t1, consumer],
            roots: vec![RootRequest {
                task: TaskId(2),
                selected_outputs: None,
                arrival_batch: 0,
            }],
            fault: GraphFault::None,
        }
    };

    run_async(async {
        verify_static_graph_differential(&ca_distinct_graph, &GraphBounds::default(), false)
            .await
            .expect("CA+CA distinct salts must disambiguate");
        verify_static_graph_differential(&build_ca_diff_refs(false), &GraphBounds::default(), false)
            .await
            .expect("CA+CA identical content with diff refs (t0 first) must disambiguate");
        verify_static_graph_differential(&build_ca_diff_refs(true), &GraphBounds::default(), false)
            .await
            .expect("CA+CA identical content with diff refs (t1 first) must disambiguate");
    });
}

#[test]
fn regression_shared_source_names_disambiguated() {
    let mut consumer = TaskNode {
        id: TaskId(0),
        name: "multi-src-consumer".to_string(),
        builder: "/bin/sh".to_string(),
        system: "x86_64-linux".to_string(),
        salt: 77,
        extra_env: BTreeMap::new(),
        outputs: vec!["out".to_string(), "dev".to_string()],
        addressing_mode: TaskAddressingMode::InputAddressed,
        source_inputs: vec![SourceId(0), SourceId(1)],
        deps: Vec::new(),
        dynamic_plan_outputs: Vec::new(),
        output_rules: BTreeMap::new(),
        dynamic_plan_rule: None,
    };
    consumer.output_rules.insert("out".to_string(), OutputRule {
        salt: 77,
        reference_targets: vec![ReferenceTarget::Source(SourceId(0))],
        ..OutputRule::default()
    });
    consumer.output_rules.insert("dev".to_string(), OutputRule {
        salt: 78,
        reference_targets: vec![ReferenceTarget::Source(SourceId(1))],
        ..OutputRule::default()
    });
    let graph = TaskGraph {
        store_dir: "/nix/store".to_string(),
        sources: vec![
            SourceSpec {
                id: SourceId(0),
                name: "shared-src".to_string(),
                bytes: b"alpha-src-bytes".to_vec(),
            },
            SourceSpec {
                id: SourceId(1),
                name: "shared-src".to_string(),
                bytes: b"beta-src-bytes".to_vec(),
            },
        ],
        tasks: vec![consumer],
        roots: vec![RootRequest {
            task: TaskId(0),
            selected_outputs: None,
            arrival_batch: 0,
        }],
        fault: GraphFault::None,
    };
    run_async(verify_static_graph_differential(&graph, &GraphBounds::default(), false))
        .expect("multi-source with colliding logical names must disambiguate");
}

#[test]
fn regression_shared_name_missing_input_detection_negative_control() {
    async fn assert_missing_detected(graph: &TaskGraph, consumer_idx: u32, is_multi_source: bool) {
        let env = InstrumentedEnv::new(&graph.store_dir);
        let sources = env.seed_sources(graph).await.expect("seed sources");
        let mut lowered = lower_task_graph(graph, &sources).expect("lower");
        let mut builder = env.make_builder(false);
        let mut worker = Worker::new(1);
        worker.want(&lowered.root_drv_paths[0], &lowered.registry, true).expect("want");
        let res = worker.run(&mut builder, &mut lowered.registry).await.expect("worker.run");
        assert!(res.failed.is_empty(), "setup run failed: {:?}", res.failed);

        let mut full_inputs: BTreeMap<StorePath<String>, Node> = BTreeMap::new();
        for sp in sources.values() {
            let pi = env.pis.get_cached_for_test(sp).expect("source PathInfo");
            full_inputs.insert(sp.clone(), pi.node);
        }
        let mut colliding: Vec<StorePath<String>> = if is_multi_source {
            sources.values().cloned().collect()
        } else {
            Vec::new()
        };
        for task_idx in 0..consumer_idx {
            let drv = &lowered.task_drv_paths[&task_idx];
            let out_sp = lowered
                .registry
                .get_output_path(&drv.to_absolute_path_with_prefix(&graph.store_dir), "out")
                .expect("dep output path");
            let pi = env.pis.get_cached_for_test(&out_sp).expect("dep PathInfo");
            full_inputs.insert(out_sp.clone(), pi.node);
            if !is_multi_source {
                colliding.push(out_sp);
            }
        }
        assert_eq!(colliding.len(), 2, "expected 2 colliding store paths");
        assert_ne!(colliding[0], colliding[1], "colliding paths must be distinct");

        let consumer_drv = &lowered.task_drv_paths[&consumer_idx];
        let consumer_entry = lowered
            .registry
            .get_by_drv_path(&consumer_drv.to_absolute_path_with_prefix(&graph.store_dir))
            .expect("consumer entry");
        let profile = ExecutionProfile::native_compatibility();

        let ok_req = derivation_to_build_request(
            &consumer_entry.derivation,
            &full_inputs,
            &graph.store_dir,
            &profile,
            HermeticityMode::Practical,
            1,
        )
        .expect("build full request")
        .build_request;
        env.build_service.do_build(ok_req).await.expect("full request succeeds");

        for missing_sp in &colliding {
            let mut dropped = full_inputs.clone();
            dropped.remove(missing_sp);
            let bad_req = derivation_to_build_request(
                &consumer_entry.derivation,
                &dropped,
                &graph.store_dir,
                &profile,
                HermeticityMode::Practical,
                1,
            )
            .expect("build dropped request")
            .build_request;
            let err = env
                .build_service
                .do_build(bad_req)
                .await
                .expect_err("dropping one colliding input must fail with NotFound");
            assert_eq!(
                err.kind(),
                std::io::ErrorKind::NotFound,
                "expected NotFound when dropping {missing_sp}, got {err:?}"
            );
        }
    }

    run_async(async {
        // 1. IA + IA
        let ia_ia = TaskGraph {
            store_dir: "/nix/store".to_string(),
            sources: Vec::new(),
            tasks: vec![
                TaskGraph::simple_task(0, "shared-name", 11, TaskAddressingMode::InputAddressed),
                TaskGraph::simple_task(1, "shared-name", 22, TaskAddressingMode::InputAddressed),
                {
                    let mut c = TaskGraph::simple_task(2, "consumer", 33, TaskAddressingMode::InputAddressed);
                    c.deps = vec![TaskDep::AllOutputs { target: TaskId(0) }, TaskDep::AllOutputs {
                        target: TaskId(1),
                    }];
                    c
                },
            ],
            roots: vec![RootRequest {
                task: TaskId(2),
                selected_outputs: None,
                arrival_batch: 0,
            }],
            fault: GraphFault::None,
        };
        assert_missing_detected(&ia_ia, 2, false).await;

        // 2. IA + CA identical content
        let ia_ca = TaskGraph {
            store_dir: "/nix/store".to_string(),
            sources: Vec::new(),
            tasks: vec![
                TaskGraph::simple_task(0, "shared-name", 42, TaskAddressingMode::InputAddressed),
                TaskGraph::simple_task(1, "shared-name", 42, TaskAddressingMode::ContentAddressed),
                {
                    let mut c = TaskGraph::simple_task(2, "consumer", 99, TaskAddressingMode::InputAddressed);
                    c.deps = vec![TaskDep::AllOutputs { target: TaskId(0) }, TaskDep::AllOutputs {
                        target: TaskId(1),
                    }];
                    c
                },
            ],
            roots: vec![RootRequest {
                task: TaskId(2),
                selected_outputs: None,
                arrival_batch: 0,
            }],
            fault: GraphFault::None,
        };
        assert_missing_detected(&ia_ca, 2, false).await;

        // 3. CA + CA distinct
        let ca_ca = TaskGraph {
            store_dir: "/nix/store".to_string(),
            sources: Vec::new(),
            tasks: vec![
                TaskGraph::simple_task(0, "shared-name", 101, TaskAddressingMode::ContentAddressed),
                TaskGraph::simple_task(1, "shared-name", 202, TaskAddressingMode::ContentAddressed),
                {
                    let mut c = TaskGraph::simple_task(2, "consumer", 303, TaskAddressingMode::InputAddressed);
                    c.deps = vec![TaskDep::AllOutputs { target: TaskId(0) }, TaskDep::AllOutputs {
                        target: TaskId(1),
                    }];
                    c
                },
            ],
            roots: vec![RootRequest {
                task: TaskId(2),
                selected_outputs: None,
                arrival_batch: 0,
            }],
            fault: GraphFault::None,
        };
        assert_missing_detected(&ca_ca, 2, false).await;

        // 4. Multi-source
        let multi_src = TaskGraph {
            store_dir: "/nix/store".to_string(),
            sources: vec![
                SourceSpec {
                    id: SourceId(0),
                    name: "shared-src".to_string(),
                    bytes: b"alpha".to_vec(),
                },
                SourceSpec {
                    id: SourceId(1),
                    name: "shared-src".to_string(),
                    bytes: b"beta".to_vec(),
                },
            ],
            tasks: vec![{
                let mut c = TaskGraph::simple_task(0, "src-consumer", 5, TaskAddressingMode::InputAddressed);
                c.source_inputs = vec![SourceId(0), SourceId(1)];
                c
            }],
            roots: vec![RootRequest {
                task: TaskId(0),
                selected_outputs: None,
                arrival_batch: 0,
            }],
            fault: GraphFault::None,
        };
        assert_missing_detected(&multi_src, 0, true).await;
    });
}

proptest! {
    #![proptest_config(ProptestConfig {
        cases: 64,
        rng_algorithm: RngAlgorithm::ChaCha,
        rng_seed: RngSeed::Fixed(0x4d41_4e54_4c45_0001),
        ..ProptestConfig::default()
    })]

    #[test]
    fn proptest_reference_evaluator_never_exhausts_within_bounds(graph in arb_static_task_graph()) {
        let bounds = GraphBounds::default();
        match evaluate_reference(&graph, &bounds) {
            ReferenceResult::Evaluated(_) | ReferenceResult::Rejected(_) => {}
            ReferenceResult::Exhausted(ex) => {
                prop_assert!(false, "reference evaluation exhausted budget: {:?}", ex);
            }
        }
    }
}

proptest! {
    #![proptest_config(ProptestConfig {
        cases: 32,
        rng_algorithm: RngAlgorithm::ChaCha,
        rng_seed: RngSeed::Fixed(0x4d41_4e54_4c45_0002),
        ..ProptestConfig::default()
    })]

    #[test]
    fn proptest_stage1_static_graphs_match_reference_oracle(
        graph in arb_static_task_graph(),
        reverse_roots in any::<bool>(),
    ) {
        let bounds = GraphBounds::default();
        let res = run_async(verify_static_graph_differential(&graph, &bounds, reverse_roots));
        prop_assert!(res.is_ok(), "stage 1 differential mismatch: {:?}", res.err());
    }
}

proptest! {
    #![proptest_config(ProptestConfig {
        cases: 16,
        rng_algorithm: RngAlgorithm::ChaCha,
        rng_seed: RngSeed::Fixed(0x4d41_4e54_4c45_0003),
        ..ProptestConfig::default()
    })]

    #[test]
    fn proptest_stage2_dynamic_plans_match_reference_and_wait_invariant(
        graph in arb_dynamic_task_graph(),
    ) {
        let res = run_async(verify_scenario_4_dynamic_plan(&graph));
        prop_assert!(res.is_ok(), "stage 2 dynamic plan mismatch: {:?}", res.err());
    }

    #[test]
    fn proptest_stage2_ca_early_cutoff_and_negative_control(
        (v1, v2, same_realized) in arb_ca_chain_graph(),
    ) {
        let res = run_async(verify_scenario_3_ca_early_cutoff(&v1, &v2, same_realized));
        prop_assert!(res.is_ok(), "stage 2 CA chain mismatch: {:?}", res.err());
    }
}
