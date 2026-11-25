//! Bounded finite acyclic deterministic task language for `#40` differential testing.
//!
//! This module is intentionally independent of `crunch_glue::convert`, `ConversionCache`,
//! `DerivationRegistry`, `Worker`, and `Builder`. It defines a small AST (`TaskGraph`)
//! that can be evaluated directly by `evaluator::evaluate_reference` AND lowered into
//! production `CrunchDerivation`s by `harness::lower_task_graph`.

use std::collections::BTreeMap;
use std::collections::BTreeSet;

use proptest::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct TaskId(pub u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct SourceId(pub u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct GraphBounds {
    pub max_tasks: u32,
    pub max_edges: u32,
    pub max_depth: u32,
    pub max_dynamic_units: u32,
    pub max_eval_steps: u32,
}

impl Default for GraphBounds {
    fn default() -> Self {
        Self {
            max_tasks: 16,
            max_edges: 32,
            max_depth: 8,
            max_dynamic_units: 8,
            max_eval_steps: 512,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SourceSpec {
    pub id: SourceId,
    pub name: String,
    pub bytes: Vec<u8>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum TaskAddressingMode {
    InputAddressed,
    ContentAddressed,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum TaskDep {
    AllOutputs {
        target: TaskId,
    },
    SelectedOutput {
        target: TaskId,
        output: String,
    },
    PlanOutput {
        binding_name: String,
        producer: TaskId,
        plan_output: String,
        root_unit: String,
        unit_output: String,
    },
}

impl TaskDep {
    pub fn target_task(&self) -> TaskId {
        match self {
            Self::AllOutputs { target } | Self::SelectedOutput { target, .. } => *target,
            Self::PlanOutput { producer, .. } => *producer,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum ReferenceTarget {
    SiblingOutput(String),
    DepOutput { dep_index: u32, output: String },
    Source(SourceId),
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct OutputRule {
    pub salt: u64,
    pub ca_content_override: Option<Vec<u8>>,
    pub embed_self_output_ref: bool,
    pub reference_targets: Vec<ReferenceTarget>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct DynamicUnitSpec {
    pub unit_id: String,
    pub name: String,
    pub salt: u64,
    pub outputs: Vec<String>,
    pub addressing_mode: TaskAddressingMode,
    pub dep_unit_outputs: Vec<(String, String)>,
    pub source_ids: Vec<SourceId>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct DynamicPlanBranch {
    pub units: Vec<DynamicUnitSpec>,
    pub roots: Vec<String>,
    pub force_invalid_plan: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct DynamicPlanRule {
    pub plan_output: String,
    pub branches: Vec<DynamicPlanBranch>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct TaskNode {
    pub id: TaskId,
    pub name: String,
    pub builder: String,
    pub system: String,
    pub salt: u64,
    pub extra_env: BTreeMap<String, String>,
    pub outputs: Vec<String>,
    pub addressing_mode: TaskAddressingMode,
    pub source_inputs: Vec<SourceId>,
    pub deps: Vec<TaskDep>,
    pub dynamic_plan_outputs: Vec<String>,
    pub output_rules: BTreeMap<String, OutputRule>,
    pub dynamic_plan_rule: Option<DynamicPlanRule>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct RootRequest {
    pub task: TaskId,
    pub selected_outputs: Option<BTreeSet<String>>,
    pub arrival_batch: u8,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum GraphFault {
    None,
    DependencyCycle { from: TaskId, to: TaskId },
    UnknownSelectedOutput { task: TaskId, output: String },
    ExceedTaskBound,
    ExceedDepthBound,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct TaskGraph {
    pub store_dir: String,
    pub sources: Vec<SourceSpec>,
    pub tasks: Vec<TaskNode>,
    pub roots: Vec<RootRequest>,
    pub fault: GraphFault,
}

impl TaskGraph {
    pub fn simple_task(id: u32, name: &str, salt: u64, addressing_mode: TaskAddressingMode) -> TaskNode {
        let mut output_rules = BTreeMap::new();
        output_rules.insert("out".to_string(), OutputRule {
            salt,
            ca_content_override: None,
            embed_self_output_ref: false,
            reference_targets: Vec::new(),
        });
        TaskNode {
            id: TaskId(id),
            name: name.to_string(),
            builder: "/bin/sh".to_string(),
            system: "x86_64-linux".to_string(),
            salt,
            extra_env: BTreeMap::new(),
            outputs: vec!["out".to_string()],
            addressing_mode,
            source_inputs: Vec::new(),
            deps: Vec::new(),
            dynamic_plan_outputs: Vec::new(),
            output_rules,
            dynamic_plan_rule: None,
        }
    }
}

fn candidate_output_names(count: u8) -> Vec<String> {
    match count {
        0 => vec!["out".to_string()],
        1 => vec!["out".to_string(), "dev".to_string()],
        _ => vec!["out".to_string(), "dev".to_string(), "lib".to_string()],
    }
}

pub(crate) fn arb_static_task_graph() -> impl Strategy<Value = TaskGraph> {
    (
        1_u32..=6,
        0_u32..=2,
        proptest::collection::vec((any::<bool>(), any::<u8>(), any::<u64>(), any::<u8>()), 6),
        proptest::collection::vec(proptest::collection::vec((0_u32..16, any::<bool>(), 0_u8..3), 0..=3), 6),
        proptest::collection::vec((0_u32..16, any::<bool>(), 0_u8..7, 0_u8..3), 1..=3),
    )
        .prop_map(|(task_count, source_count, node_specs, edge_specs, root_specs)| {
            let mut sources = Vec::with_capacity(source_count as usize);
            for sid in 0..source_count {
                sources.push(SourceSpec {
                    id: SourceId(sid),
                    name: format!("src-{sid}"),
                    bytes: format!("source-payload-{sid}").into_bytes(),
                });
            }

            let mut tasks: Vec<TaskNode> = Vec::with_capacity(task_count as usize);
            let mut has_shared_ancestor: Vec<bool> = Vec::with_capacity(task_count as usize);
            for idx in 0..task_count {
                let (is_ca, out_kind, salt_raw, collide_hint) = node_specs[idx as usize];
                let outputs = candidate_output_names(out_kind % 3);
                let addressing_mode = if is_ca {
                    TaskAddressingMode::ContentAddressed
                } else {
                    TaskAddressingMode::InputAddressed
                };
                let salt = (salt_raw % 10_000).saturating_add(u64::from(idx));

                let mut deps = Vec::new();
                let mut seen_targets = BTreeSet::new();
                if idx > 0 {
                    for (raw_target, use_selected, raw_out_idx) in &edge_specs[idx as usize] {
                        let target_idx = *raw_target % idx;
                        if !seen_targets.insert(target_idx) {
                            continue;
                        }
                        let target_outputs = &tasks[target_idx as usize].outputs;
                        if *use_selected {
                            let selected = target_outputs[usize::from(*raw_out_idx) % target_outputs.len()].clone();
                            deps.push(TaskDep::SelectedOutput {
                                target: TaskId(target_idx),
                                output: selected,
                            });
                        } else {
                            deps.push(TaskDep::AllOutputs {
                                target: TaskId(target_idx),
                            });
                        }
                    }
                }

                let dep_has_shared = deps.iter().any(|d| has_shared_ancestor[d.target_task().0 as usize]);
                // Periodically emit colliding (name, builder, system) across independent/sibling
                // branches (never ancestor->descendant, which crunch_glue visiting_identity rejects as a cycle).
                let is_shared_name = idx > 0 && collide_hint % 4 == 0 && !dep_has_shared;
                let name = if is_shared_name {
                    "shared-name".to_string()
                } else {
                    format!("task-{idx}")
                };
                has_shared_ancestor.push(is_shared_name || dep_has_shared);

                let mut source_inputs = Vec::new();
                if source_count > 0 && idx % 2 == 0 {
                    source_inputs.push(SourceId(idx % source_count));
                }

                let mut output_rules = BTreeMap::new();
                for (out_idx, out_name) in outputs.iter().enumerate() {
                    let mut reference_targets = Vec::new();
                    if !is_ca && out_idx == 0 && outputs.len() > 1 && salt_raw % 3 == 0 {
                        reference_targets.push(ReferenceTarget::SiblingOutput(outputs[1].clone()));
                    }
                    if out_idx == 0 && !deps.is_empty() && salt_raw % 2 == 0 {
                        let dep_out = match &deps[0] {
                            TaskDep::SelectedOutput { output, .. } => output.clone(),
                            _ => tasks[deps[0].target_task().0 as usize].outputs[0].clone(),
                        };
                        reference_targets.push(ReferenceTarget::DepOutput {
                            dep_index: 0,
                            output: dep_out,
                        });
                    }
                    output_rules.insert(out_name.clone(), OutputRule {
                        salt: salt.saturating_add(out_idx as u64),
                        ca_content_override: None,
                        embed_self_output_ref: false,
                        reference_targets,
                    });
                }

                tasks.push(TaskNode {
                    id: TaskId(idx),
                    name,
                    builder: "/bin/sh".to_string(),
                    system: "x86_64-linux".to_string(),
                    salt,
                    extra_env: BTreeMap::new(),
                    outputs,
                    addressing_mode,
                    source_inputs,
                    deps,
                    dynamic_plan_outputs: Vec::new(),
                    output_rules,
                    dynamic_plan_rule: None,
                });
            }

            let mut roots = Vec::with_capacity(root_specs.len());
            for (raw_task, select_subset, mask, batch) in root_specs {
                let task_idx = raw_task % task_count;
                let task_outputs = &tasks[task_idx as usize].outputs;
                let selected_outputs = if select_subset {
                    let mut chosen = BTreeSet::new();
                    for (bit, out_name) in task_outputs.iter().enumerate() {
                        if (mask & (1 << bit)) != 0 {
                            chosen.insert(out_name.clone());
                        }
                    }
                    if chosen.is_empty() {
                        chosen.insert(task_outputs[0].clone());
                    }
                    Some(chosen)
                } else {
                    None
                };
                roots.push(RootRequest {
                    task: TaskId(task_idx),
                    selected_outputs,
                    arrival_batch: batch,
                });
            }

            TaskGraph {
                store_dir: "/nix/store".to_string(),
                sources,
                tasks,
                roots,
                fault: GraphFault::None,
            }
        })
}

pub(crate) fn arb_dynamic_task_graph() -> impl Strategy<Value = TaskGraph> {
    (1_u64..1000, 1_u64..1000, any::<bool>(), any::<bool>()).prop_map(
        |(producer_salt, consumer_salt, consumer_ca, include_leaf_dep)| {
            let mut tasks = Vec::new();
            let mut producer_deps = Vec::new();
            if include_leaf_dep {
                tasks.push(TaskGraph::simple_task(
                    0,
                    "plan-input",
                    producer_salt + 1,
                    TaskAddressingMode::InputAddressed,
                ));
                producer_deps.push(TaskDep::AllOutputs { target: TaskId(0) });
            }
            let producer_id = tasks.len() as u32;
            let mut producer_rules = BTreeMap::new();
            producer_rules.insert("out".to_string(), OutputRule {
                salt: producer_salt,
                ..OutputRule::default()
            });
            producer_rules.insert("plan".to_string(), OutputRule {
                salt: producer_salt,
                ..OutputRule::default()
            });

            let branch_a = DynamicPlanBranch {
                units: vec![DynamicUnitSpec {
                    unit_id: "unit.alpha".to_string(),
                    name: "dyn-alpha".to_string(),
                    salt: producer_salt + 10,
                    outputs: vec!["out".to_string()],
                    addressing_mode: TaskAddressingMode::ContentAddressed,
                    dep_unit_outputs: Vec::new(),
                    source_ids: Vec::new(),
                }],
                roots: vec!["unit.alpha".to_string()],
                force_invalid_plan: false,
            };
            let branch_b = DynamicPlanBranch {
                units: vec![
                    DynamicUnitSpec {
                        unit_id: "unit.leaf".to_string(),
                        name: "dyn-leaf".to_string(),
                        salt: producer_salt + 20,
                        outputs: vec!["out".to_string()],
                        addressing_mode: TaskAddressingMode::InputAddressed,
                        dep_unit_outputs: Vec::new(),
                        source_ids: Vec::new(),
                    },
                    DynamicUnitSpec {
                        unit_id: "unit.alpha".to_string(),
                        name: "dyn-beta".to_string(),
                        salt: producer_salt + 30,
                        outputs: vec!["out".to_string()],
                        addressing_mode: TaskAddressingMode::ContentAddressed,
                        dep_unit_outputs: vec![("unit.leaf".to_string(), "out".to_string())],
                        source_ids: Vec::new(),
                    },
                ],
                roots: vec!["unit.alpha".to_string()],
                force_invalid_plan: false,
            };

            tasks.push(TaskNode {
                id: TaskId(producer_id),
                name: "dyn-producer".to_string(),
                builder: "/bin/sh".to_string(),
                system: "x86_64-linux".to_string(),
                salt: producer_salt,
                extra_env: BTreeMap::new(),
                outputs: vec!["out".to_string(), "plan".to_string()],
                addressing_mode: TaskAddressingMode::InputAddressed,
                source_inputs: Vec::new(),
                deps: producer_deps,
                dynamic_plan_outputs: vec!["plan".to_string()],
                output_rules: producer_rules,
                dynamic_plan_rule: Some(DynamicPlanRule {
                    plan_output: "plan".to_string(),
                    branches: vec![branch_a, branch_b],
                }),
            });

            let consumer_id = tasks.len() as u32;
            let mut consumer_rules = BTreeMap::new();
            consumer_rules.insert("out".to_string(), OutputRule {
                salt: consumer_salt,
                ..OutputRule::default()
            });
            tasks.push(TaskNode {
                id: TaskId(consumer_id),
                name: "dyn-consumer".to_string(),
                builder: "/bin/sh".to_string(),
                system: "x86_64-linux".to_string(),
                salt: consumer_salt,
                extra_env: BTreeMap::new(),
                outputs: vec!["out".to_string()],
                addressing_mode: if consumer_ca {
                    TaskAddressingMode::ContentAddressed
                } else {
                    TaskAddressingMode::InputAddressed
                },
                source_inputs: Vec::new(),
                deps: vec![TaskDep::PlanOutput {
                    binding_name: "dyn_root".to_string(),
                    producer: TaskId(producer_id),
                    plan_output: "plan".to_string(),
                    root_unit: "unit.alpha".to_string(),
                    unit_output: "out".to_string(),
                }],
                dynamic_plan_outputs: Vec::new(),
                output_rules: consumer_rules,
                dynamic_plan_rule: None,
            });

            TaskGraph {
                store_dir: "/nix/store".to_string(),
                sources: Vec::new(),
                tasks,
                roots: vec![RootRequest {
                    task: TaskId(consumer_id),
                    selected_outputs: None,
                    arrival_batch: 0,
                }],
                fault: GraphFault::None,
            }
        },
    )
}

pub(crate) fn arb_ca_chain_graph() -> impl Strategy<Value = (TaskGraph, TaskGraph, bool)> {
    (1_u64..1000, 1_u64..1000, 1_u64..1000, any::<bool>()).prop_map(|(p_salt, c_salt, g_salt, same_realized_ca)| {
        let build_chain = |producer_salt: u64, realized_bytes: Vec<u8>| {
            let mut p = TaskGraph::simple_task(0, "ca-producer", producer_salt, TaskAddressingMode::ContentAddressed);
            p.output_rules.get_mut("out").expect("out rule").ca_content_override = Some(realized_bytes);

            let mut c = TaskGraph::simple_task(1, "ca-child", c_salt, TaskAddressingMode::ContentAddressed);
            c.deps.push(TaskDep::AllOutputs { target: TaskId(0) });

            let mut g = TaskGraph::simple_task(2, "ia-grandchild", g_salt, TaskAddressingMode::InputAddressed);
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

        let v1 = build_chain(p_salt, format!("ca-realized-v1-{p_salt}").into_bytes());
        let v2_bytes = if same_realized_ca {
            format!("ca-realized-v1-{p_salt}").into_bytes()
        } else {
            format!("ca-realized-v2-{p_salt}").into_bytes()
        };
        let v2 = build_chain(p_salt + 10_000, v2_bytes);
        (v1, v2, same_realized_ca)
    })
}
