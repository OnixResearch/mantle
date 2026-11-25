//! Independent bounded reference evaluator for `TaskGraph` (`#40`).
//!
//! Intentionally shares ZERO code with `crunch_glue::convert`, `ConversionCache`,
//! `DerivationRegistry`, `Worker`, `Builder`, `crunch_action_result_core`, or
//! `nix_compat::derivation`. All loops are iterative and explicitly bounded.

use std::collections::BTreeMap;
use std::collections::BTreeSet;

use super::task_language::GraphBounds;
use super::task_language::GraphFault;
use super::task_language::OutputRule;
use super::task_language::ReferenceTarget;
use super::task_language::TaskAddressingMode;
use super::task_language::TaskDep;
use super::task_language::TaskGraph;
use super::task_language::TaskId;
use super::task_language::TaskNode;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ReferenceResult {
    Evaluated(ReferenceEvaluation),
    Rejected(ReferenceRejection),
    Exhausted(ReferenceExhaustion),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ReferenceRejection {
    CycleDetected { cycle: Vec<TaskId> },
    TaskBoundExceeded { count: u32, max: u32 },
    EdgeBoundExceeded { count: u32, max: u32 },
    DepthBoundExceeded { depth: u32, max: u32 },
    DynamicUnitBoundExceeded { count: u32, max: u32 },
    UnknownOutputSelection { task: TaskId, output: String },
    EmptyOutputSelection { task: TaskId },
    InvalidPlanBinding { task: TaskId, reason: String },
    InvalidTaskDefinition { task: TaskId, reason: String },
    InvalidDynamicPlan { task: TaskId, reason: String },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ReferenceExhaustion {
    pub steps_executed: u32,
    pub max_eval_steps: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct EvaluatedOutput {
    pub content_prefix: Vec<u8>,
    pub content_digest: [u8; 32],
    pub expected_ref_targets: BTreeSet<ReferenceTarget>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ReferenceEvaluation {
    pub root_outputs: BTreeMap<(u32, String), EvaluatedOutput>,
    pub task_outputs: BTreeMap<(u32, String), EvaluatedOutput>,
    pub task_demanded_outputs: BTreeMap<u32, BTreeSet<String>>,
    pub preflight_eligible_roots: BTreeSet<u32>,
    pub dynamic_selected_branch: BTreeMap<u32, usize>,
    pub dynamic_root_outputs: BTreeMap<(u32, String, String), Vec<u8>>,
    pub unique_task_recipes: BTreeSet<[u8; 32]>,
    pub topo_order: Vec<u32>,
    pub steps_executed: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum VisitColor {
    Unvisited,
    Visiting,
    Visited,
}

struct DfsFrame {
    task_idx: u32,
    next_dep: u32,
}

pub(crate) fn evaluate_reference(graph: &TaskGraph, bounds: &GraphBounds) -> ReferenceResult {
    let mut steps: u32 = 0;

    if let Err(rej) = validate_graph_shape(graph, bounds) {
        return ReferenceResult::Rejected(rej);
    }

    let (topo_order, depths) = match compute_topo_and_depths(graph, bounds, &mut steps) {
        Ok(res) => res,
        Err(Err(exhaust)) => return ReferenceResult::Exhausted(exhaust),
        Err(Ok(rej)) => return ReferenceResult::Rejected(rej),
    };

    for (idx, depth) in depths.iter().enumerate() {
        if *depth > bounds.max_depth {
            let _ = idx;
            return ReferenceResult::Rejected(ReferenceRejection::DepthBoundExceeded {
                depth: *depth,
                max: bounds.max_depth,
            });
        }
    }

    if let Err(rej) = validate_selections_and_bindings(graph) {
        return ReferenceResult::Rejected(rej);
    }

    let task_demanded_outputs = match propagate_demanded_outputs(graph, &topo_order, bounds, &mut steps) {
        Ok(demands) => demands,
        Err(exhaust) => return ReferenceResult::Exhausted(exhaust),
    };

    let preflight_eligible_roots = compute_preflight_eligible_roots(graph);

    let mut source_map: BTreeMap<u32, (&str, &[u8])> = BTreeMap::new();
    for src in &graph.sources {
        source_map.insert(src.id.0, (src.name.as_str(), src.bytes.as_slice()));
    }

    let mut task_outputs: BTreeMap<(u32, String), EvaluatedOutput> = BTreeMap::new();
    let mut dynamic_selected_branch: BTreeMap<u32, usize> = BTreeMap::new();
    let mut dynamic_root_outputs: BTreeMap<(u32, String, String), Vec<u8>> = BTreeMap::new();
    let mut unique_task_recipes: BTreeSet<[u8; 32]> = BTreeSet::new();

    for &task_idx in &topo_order {
        if steps >= bounds.max_eval_steps {
            return ReferenceResult::Exhausted(ReferenceExhaustion {
                steps_executed: steps,
                max_eval_steps: bounds.max_eval_steps,
            });
        }
        steps = steps.saturating_add(1);

        let task = &graph.tasks[task_idx as usize];
        unique_task_recipes.insert(independent_recipe_digest(task, graph));

        // Gather mounted input bytes AND their transitive runtime closure in deterministic
        // (logical_name, blake3) order, matching `Builder::collect_sandbox_inputs`.
        let mut closure_queue: std::collections::VecDeque<(Option<u32>, Option<String>, Option<u32>)> =
            std::collections::VecDeque::new();
        let mut seen_closure: BTreeSet<(Option<u32>, Option<String>, Option<u32>)> = BTreeSet::new();
        let mut mounted_inputs: Vec<(String, Vec<u8>)> = Vec::new();

        for src_id in &task.source_inputs {
            if !source_map.contains_key(&src_id.0) {
                return ReferenceResult::Rejected(ReferenceRejection::InvalidTaskDefinition {
                    task: task.id,
                    reason: format!("unknown source id {}", src_id.0),
                });
            }
            closure_queue.push_back((None, None, Some(src_id.0)));
        }
        for dep in &task.deps {
            match dep {
                TaskDep::AllOutputs { target } => {
                    let dep_task = &graph.tasks[target.0 as usize];
                    for out_name in &dep_task.outputs {
                        if !task_outputs.contains_key(&(target.0, out_name.clone())) {
                            return ReferenceResult::Rejected(ReferenceRejection::InvalidTaskDefinition {
                                task: task.id,
                                reason: format!("missing evaluated output {}:{out_name}", target.0),
                            });
                        }
                        closure_queue.push_back((Some(target.0), Some(out_name.clone()), None));
                    }
                }
                TaskDep::SelectedOutput { target, output } => {
                    if !task_outputs.contains_key(&(target.0, output.clone())) {
                        return ReferenceResult::Rejected(ReferenceRejection::UnknownOutputSelection {
                            task: *target,
                            output: output.clone(),
                        });
                    }
                    closure_queue.push_back((Some(target.0), Some(output.clone()), None));
                }
                TaskDep::PlanOutput {
                    producer,
                    root_unit,
                    unit_output,
                    ..
                } => {
                    let Some(dyn_bytes) =
                        dynamic_root_outputs.get(&(producer.0, root_unit.clone(), unit_output.clone()))
                    else {
                        return ReferenceResult::Rejected(ReferenceRejection::InvalidPlanBinding {
                            task: task.id,
                            reason: format!("dynamic root {root_unit}:{unit_output} not produced by {}", producer.0),
                        });
                    };
                    let prod_task = &graph.tasks[producer.0 as usize];
                    let unit_name = prod_task
                        .dynamic_plan_rule
                        .as_ref()
                        .and_then(|rule| dynamic_selected_branch.get(&producer.0).map(|&b| (rule, b)))
                        .and_then(|(rule, b)| rule.branches.get(b))
                        .and_then(|branch| branch.units.iter().find(|u| u.unit_id == *root_unit))
                        .map(|u| u.name.clone())
                        .unwrap_or_else(|| root_unit.clone());
                    mounted_inputs.push((unit_name, dyn_bytes.clone()));
                }
            }
        }

        let closure_bound = bounds.max_tasks.saturating_mul(16).saturating_add(32);
        for _ in 0..closure_bound {
            let Some(item) = closure_queue.pop_front() else {
                break;
            };
            if !seen_closure.insert(item.clone()) {
                continue;
            }
            match item {
                (None, None, Some(sid)) => {
                    if let Some((src_name, src_bytes)) = source_map.get(&sid) {
                        mounted_inputs.push(((*src_name).to_string(), (*src_bytes).to_vec()));
                    }
                }
                (Some(t_idx), Some(out_name), None) => {
                    let dep_task = &graph.tasks[t_idx as usize];
                    if let Some(eval_out) = task_outputs.get(&(t_idx, out_name.clone())) {
                        let logical_name = if out_name == "out" {
                            dep_task.name.clone()
                        } else {
                            format!("{}-{out_name}", dep_task.name)
                        };
                        mounted_inputs.push((logical_name, eval_out.content_prefix.clone()));
                        for target_ref in &eval_out.expected_ref_targets {
                            match target_ref {
                                ReferenceTarget::SiblingOutput(sib) => {
                                    closure_queue.push_back((Some(t_idx), Some(sib.clone()), None));
                                }
                                ReferenceTarget::DepOutput { dep_index, output } => {
                                    if let Some(sub_dep) = dep_task.deps.get(*dep_index as usize) {
                                        closure_queue.push_back((
                                            Some(sub_dep.target_task().0),
                                            Some(output.clone()),
                                            None,
                                        ));
                                    }
                                }
                                ReferenceTarget::Source(sub_sid) => {
                                    closure_queue.push_back((None, None, Some(sub_sid.0)));
                                }
                            }
                        }
                    }
                }
                _ => {}
            }
        }
        if !closure_queue.is_empty() {
            return ReferenceResult::Exhausted(ReferenceExhaustion {
                steps_executed: steps,
                max_eval_steps: bounds.max_eval_steps,
            });
        }

        sort_mounted_inputs(&mut mounted_inputs);

        // If this task declares a dynamic plan rule, evaluate its selected branch.
        if let Some(plan_rule) = &task.dynamic_plan_rule {
            if plan_rule.branches.is_empty() {
                return ReferenceResult::Rejected(ReferenceRejection::InvalidDynamicPlan {
                    task: task.id,
                    reason: "dynamic plan has zero branches".to_string(),
                });
            }
            let mut sel_hasher = blake3::Hasher::new();
            for (_, bytes) in &mounted_inputs {
                sel_hasher.update(&(bytes.len() as u64).to_le_bytes());
                sel_hasher.update(bytes);
            }
            let sel_digest = sel_hasher.finalize();
            let branch_idx = usize::from(sel_digest.as_bytes()[0]) % plan_rule.branches.len();
            let branch = &plan_rule.branches[branch_idx];
            if branch.force_invalid_plan {
                return ReferenceResult::Rejected(ReferenceRejection::InvalidDynamicPlan {
                    task: task.id,
                    reason: "branch forced invalid dynamic plan".to_string(),
                });
            }
            let unit_count = u32::try_from(branch.units.len()).unwrap_or(u32::MAX);
            if unit_count > bounds.max_dynamic_units {
                return ReferenceResult::Rejected(ReferenceRejection::DynamicUnitBoundExceeded {
                    count: unit_count,
                    max: bounds.max_dynamic_units,
                });
            }
            dynamic_selected_branch.insert(task_idx, branch_idx);

            let mut unit_map: BTreeMap<String, &super::task_language::DynamicUnitSpec> = BTreeMap::new();
            for unit in &branch.units {
                unit_map.insert(unit.unit_id.clone(), unit);
            }
            let mut evaluated_units: BTreeMap<(String, String), Vec<u8>> = BTreeMap::new();
            for unit in &branch.units {
                if steps >= bounds.max_eval_steps {
                    return ReferenceResult::Exhausted(ReferenceExhaustion {
                        steps_executed: steps,
                        max_eval_steps: bounds.max_eval_steps,
                    });
                }
                steps = steps.saturating_add(1);
                let mut unit_inputs: Vec<(String, Vec<u8>)> = Vec::new();
                for sid in &unit.source_ids {
                    let Some((src_name, src_bytes)) = source_map.get(&sid.0) else {
                        return ReferenceResult::Rejected(ReferenceRejection::InvalidDynamicPlan {
                            task: task.id,
                            reason: format!("dynamic unit references missing source {}", sid.0),
                        });
                    };
                    unit_inputs.push(((*src_name).to_string(), (*src_bytes).to_vec()));
                }
                for (dep_uid, dep_out) in &unit.dep_unit_outputs {
                    let Some(dep_unit) = unit_map.get(dep_uid) else {
                        return ReferenceResult::Rejected(ReferenceRejection::InvalidDynamicPlan {
                            task: task.id,
                            reason: format!("dynamic unit references unknown unit {dep_uid}"),
                        });
                    };
                    let Some(dep_bytes) = evaluated_units.get(&(dep_uid.clone(), dep_out.clone())) else {
                        return ReferenceResult::Rejected(ReferenceRejection::InvalidDynamicPlan {
                            task: task.id,
                            reason: format!("dynamic unit dependency {dep_uid}:{dep_out} not topologically prior"),
                        });
                    };
                    unit_inputs.push((dep_unit.name.clone(), dep_bytes.clone()));
                }
                sort_mounted_inputs(&mut unit_inputs);
                for out_name in &unit.outputs {
                    let payload = compute_output_payload(&unit.name, out_name, unit.salt, &unit_inputs, None);
                    evaluated_units.insert((unit.unit_id.clone(), out_name.clone()), payload.clone());
                    if branch.roots.contains(&unit.unit_id) {
                        dynamic_root_outputs.insert((task_idx, unit.unit_id.clone(), out_name.clone()), payload);
                    }
                }
            }
        }

        for out_name in &task.outputs {
            let rule = task.output_rules.get(out_name).cloned().unwrap_or(OutputRule {
                salt: task.salt,
                ..OutputRule::default()
            });
            let content_prefix = compute_output_payload(
                &task.name,
                out_name,
                rule.salt,
                &mounted_inputs,
                rule.ca_content_override.as_deref(),
            );
            let content_digest = *blake3::hash(&content_prefix).as_bytes();
            let expected_ref_targets: BTreeSet<ReferenceTarget> = rule.reference_targets.into_iter().collect();
            task_outputs.insert((task_idx, out_name.clone()), EvaluatedOutput {
                content_prefix,
                content_digest,
                expected_ref_targets,
            });
        }
    }

    let mut root_outputs = BTreeMap::new();
    for (root_idx, root) in graph.roots.iter().enumerate() {
        let task = &graph.tasks[root.task.0 as usize];
        let wanted: BTreeSet<String> = match &root.selected_outputs {
            Some(sel) => sel.clone(),
            None => task.outputs.iter().cloned().collect(),
        };
        for out_name in wanted {
            if let Some(eval_out) = task_outputs.get(&(root.task.0, out_name.clone())) {
                root_outputs.insert((u32::try_from(root_idx).unwrap_or(u32::MAX), out_name), eval_out.clone());
            }
        }
    }

    ReferenceResult::Evaluated(ReferenceEvaluation {
        root_outputs,
        task_outputs,
        task_demanded_outputs,
        preflight_eligible_roots,
        dynamic_selected_branch,
        dynamic_root_outputs,
        unique_task_recipes,
        topo_order,
        steps_executed: steps,
    })
}

fn validate_graph_shape(graph: &TaskGraph, bounds: &GraphBounds) -> Result<(), ReferenceRejection> {
    if matches!(graph.fault, GraphFault::ExceedTaskBound) {
        return Err(ReferenceRejection::TaskBoundExceeded {
            count: bounds.max_tasks.saturating_add(1),
            max: bounds.max_tasks,
        });
    }
    if matches!(graph.fault, GraphFault::ExceedDepthBound) {
        return Err(ReferenceRejection::DepthBoundExceeded {
            depth: bounds.max_depth.saturating_add(1),
            max: bounds.max_depth,
        });
    }
    let task_count = u32::try_from(graph.tasks.len()).unwrap_or(u32::MAX);
    if task_count > bounds.max_tasks {
        return Err(ReferenceRejection::TaskBoundExceeded {
            count: task_count,
            max: bounds.max_tasks,
        });
    }
    let mut total_edges: u32 = 0;
    for (idx, task) in graph.tasks.iter().enumerate() {
        if task.id.0 != u32::try_from(idx).unwrap_or(u32::MAX) {
            return Err(ReferenceRejection::InvalidTaskDefinition {
                task: task.id,
                reason: "task id must match array index".to_string(),
            });
        }
        if task.name.is_empty() || task.builder.is_empty() || task.outputs.is_empty() {
            return Err(ReferenceRejection::InvalidTaskDefinition {
                task: task.id,
                reason: "empty name, builder, or outputs".to_string(),
            });
        }
        let dep_len = u32::try_from(task.deps.len()).unwrap_or(u32::MAX);
        total_edges = total_edges.saturating_add(dep_len);
        if total_edges > bounds.max_edges {
            return Err(ReferenceRejection::EdgeBoundExceeded {
                count: total_edges,
                max: bounds.max_edges,
            });
        }
        for dep in &task.deps {
            if dep.target_task().0 >= task_count {
                return Err(ReferenceRejection::InvalidTaskDefinition {
                    task: task.id,
                    reason: format!("unknown dependency task {}", dep.target_task().0),
                });
            }
        }
    }
    Ok(())
}

fn compute_topo_and_depths(
    graph: &TaskGraph,
    bounds: &GraphBounds,
    steps: &mut u32,
) -> Result<(Vec<u32>, Vec<u32>), Result<ReferenceRejection, ReferenceExhaustion>> {
    let n = graph.tasks.len();
    let mut colors = vec![VisitColor::Unvisited; n];
    let mut depths = vec![0_u32; n];
    let mut topo_order = Vec::with_capacity(n);

    // Build adjacency list including injected cycle fault if any.
    let mut adj: Vec<Vec<u32>> = vec![Vec::new(); n];
    for (idx, task) in graph.tasks.iter().enumerate() {
        for dep in &task.deps {
            adj[idx].push(dep.target_task().0);
        }
    }
    if let GraphFault::DependencyCycle { from, to } = graph.fault
        && (from.0 as usize) < n
        && (to.0 as usize) < n
    {
        adj[from.0 as usize].push(to.0);
    }

    let max_dfs_iters = bounds.max_tasks.saturating_mul(4).saturating_add(bounds.max_edges).saturating_add(16);
    for start in 0..n {
        if colors[start] != VisitColor::Unvisited {
            continue;
        }
        let mut stack = vec![DfsFrame {
            task_idx: u32::try_from(start).unwrap_or(u32::MAX),
            next_dep: 0,
        }];
        colors[start] = VisitColor::Visiting;

        for _ in 0..max_dfs_iters {
            let Some(top) = stack.last_mut() else {
                break;
            };
            if *steps >= bounds.max_eval_steps {
                return Err(Err(ReferenceExhaustion {
                    steps_executed: *steps,
                    max_eval_steps: bounds.max_eval_steps,
                }));
            }
            *steps = steps.saturating_add(1);

            let current = top.task_idx;
            let dep_pos = top.next_dep as usize;
            if dep_pos < adj[current as usize].len() {
                let dep_idx = adj[current as usize][dep_pos];
                top.next_dep = top.next_dep.saturating_add(1);
                match colors[dep_idx as usize] {
                    VisitColor::Visiting => {
                        let cycle =
                            stack.iter().map(|f| TaskId(f.task_idx)).chain(std::iter::once(TaskId(dep_idx))).collect();
                        return Err(Ok(ReferenceRejection::CycleDetected { cycle }));
                    }
                    VisitColor::Unvisited => {
                        colors[dep_idx as usize] = VisitColor::Visiting;
                        stack.push(DfsFrame {
                            task_idx: dep_idx,
                            next_dep: 0,
                        });
                    }
                    VisitColor::Visited => {}
                }
            } else {
                let mut max_child_depth = 0_u32;
                for &dep_idx in &adj[current as usize] {
                    max_child_depth = max_child_depth.max(depths[dep_idx as usize].saturating_add(1));
                }
                depths[current as usize] = max_child_depth;
                colors[current as usize] = VisitColor::Visited;
                topo_order.push(current);
                stack.pop();
            }
        }
        if !stack.is_empty() {
            return Err(Err(ReferenceExhaustion {
                steps_executed: *steps,
                max_eval_steps: bounds.max_eval_steps,
            }));
        }
    }

    Ok((topo_order, depths))
}

fn validate_selections_and_bindings(graph: &TaskGraph) -> Result<(), ReferenceRejection> {
    if let GraphFault::UnknownSelectedOutput { task, output } = &graph.fault {
        return Err(ReferenceRejection::UnknownOutputSelection {
            task: *task,
            output: output.clone(),
        });
    }
    let task_count = u32::try_from(graph.tasks.len()).unwrap_or(u32::MAX);
    for task in &graph.tasks {
        for plan_out in &task.dynamic_plan_outputs {
            if !task.outputs.contains(plan_out) {
                return Err(ReferenceRejection::InvalidPlanBinding {
                    task: task.id,
                    reason: format!("undeclared dynamic plan output '{plan_out}'"),
                });
            }
        }
        for dep in &task.deps {
            match dep {
                TaskDep::AllOutputs { .. } => {}
                TaskDep::SelectedOutput { target, output } => {
                    let target_task = &graph.tasks[target.0 as usize];
                    if !target_task.outputs.contains(output) {
                        return Err(ReferenceRejection::UnknownOutputSelection {
                            task: *target,
                            output: output.clone(),
                        });
                    }
                }
                TaskDep::PlanOutput {
                    producer, plan_output, ..
                } => {
                    let prod_task = &graph.tasks[producer.0 as usize];
                    if !prod_task.dynamic_plan_outputs.contains(plan_output) {
                        return Err(ReferenceRejection::InvalidPlanBinding {
                            task: task.id,
                            reason: format!(
                                "producer {} does not declare dynamic plan output '{plan_output}'",
                                producer.0
                            ),
                        });
                    }
                }
            }
        }
    }
    for root in &graph.roots {
        if root.task.0 >= task_count {
            return Err(ReferenceRejection::InvalidTaskDefinition {
                task: root.task,
                reason: "root references unknown task".to_string(),
            });
        }
        let target_task = &graph.tasks[root.task.0 as usize];
        if let Some(selected) = &root.selected_outputs {
            if selected.is_empty() {
                return Err(ReferenceRejection::EmptyOutputSelection { task: root.task });
            }
            for output in selected {
                if !target_task.outputs.contains(output) {
                    return Err(ReferenceRejection::UnknownOutputSelection {
                        task: root.task,
                        output: output.clone(),
                    });
                }
            }
        }
    }
    Ok(())
}

fn propagate_demanded_outputs(
    graph: &TaskGraph,
    topo_order: &[u32],
    bounds: &GraphBounds,
    steps: &mut u32,
) -> Result<BTreeMap<u32, BTreeSet<String>>, ReferenceExhaustion> {
    let mut demands: BTreeMap<u32, BTreeSet<String>> = BTreeMap::new();
    for root in &graph.roots {
        let task = &graph.tasks[root.task.0 as usize];
        let entry = demands.entry(root.task.0).or_default();
        match &root.selected_outputs {
            Some(selected) => entry.extend(selected.iter().cloned()),
            None => entry.extend(task.outputs.iter().cloned()),
        }
    }
    for &task_idx in topo_order.iter().rev() {
        if *steps >= bounds.max_eval_steps {
            return Err(ReferenceExhaustion {
                steps_executed: *steps,
                max_eval_steps: bounds.max_eval_steps,
            });
        }
        *steps = steps.saturating_add(1);
        if !demands.contains_key(&task_idx) {
            continue;
        }
        let task = &graph.tasks[task_idx as usize];
        for dep in &task.deps {
            match dep {
                TaskDep::AllOutputs { target } => {
                    let target_task = &graph.tasks[target.0 as usize];
                    demands.entry(target.0).or_default().extend(target_task.outputs.iter().cloned());
                }
                TaskDep::SelectedOutput { target, output } => {
                    demands.entry(target.0).or_default().insert(output.clone());
                }
                TaskDep::PlanOutput {
                    producer, plan_output, ..
                } => {
                    demands.entry(producer.0).or_default().insert(plan_output.clone());
                }
            }
        }
    }
    Ok(demands)
}

fn compute_preflight_eligible_roots(graph: &TaskGraph) -> BTreeSet<u32> {
    let mut eligible = BTreeSet::new();
    for root in &graph.roots {
        let root_task = &graph.tasks[root.task.0 as usize];
        if root_task.deps.is_empty() {
            continue;
        }
        if is_subtree_preflight_eligible(graph, root.task.0) {
            eligible.insert(root.task.0);
        }
    }
    eligible
}

fn is_subtree_preflight_eligible(graph: &TaskGraph, root_idx: u32) -> bool {
    let mut visited = BTreeSet::new();
    let mut queue = std::collections::VecDeque::from([root_idx]);
    let max_steps = graph.tasks.len().saturating_mul(2).saturating_add(4);
    for _ in 0..max_steps {
        let Some(curr) = queue.pop_front() else {
            return true;
        };
        if !visited.insert(curr) {
            continue;
        }
        let task = &graph.tasks[curr as usize];
        if task.addressing_mode != TaskAddressingMode::InputAddressed {
            return false;
        }
        if !task.dynamic_plan_outputs.is_empty() {
            return false;
        }
        for dep in &task.deps {
            match dep {
                TaskDep::AllOutputs { target } | TaskDep::SelectedOutput { target, .. } => {
                    queue.push_back(target.0);
                }
                TaskDep::PlanOutput { .. } => return false,
            }
        }
    }
    queue.is_empty()
}

fn sort_mounted_inputs(inputs: &mut [(String, Vec<u8>)]) {
    inputs.sort_by(|left, right| {
        let left_hash = *blake3::hash(&left.1).as_bytes();
        let right_hash = *blake3::hash(&right.1).as_bytes();
        left.0.cmp(&right.0).then(left_hash.cmp(&right_hash))
    });
}

fn compute_output_payload(
    task_name: &str,
    output_name: &str,
    salt: u64,
    sorted_inputs: &[(String, Vec<u8>)],
    ca_override: Option<&[u8]>,
) -> Vec<u8> {
    if let Some(override_bytes) = ca_override {
        return override_bytes.to_vec();
    }
    let mut hasher = blake3::Hasher::new();
    hasher.update(b"mantle.ref.fake-exec.v1");
    hasher.update(&salt.to_le_bytes());
    hasher.update(output_name.as_bytes());
    for (name, bytes) in sorted_inputs {
        hasher.update(&(name.len() as u64).to_le_bytes());
        hasher.update(name.as_bytes());
        hasher.update(&(bytes.len() as u64).to_le_bytes());
        hasher.update(bytes);
    }
    let digest = hasher.finalize();
    format!("out:{task_name}:{output_name}:{}", digest.to_hex()).into_bytes()
}

fn independent_recipe_digest(task: &TaskNode, graph: &TaskGraph) -> [u8; 32] {
    let mut hasher = blake3::Hasher::new();
    hasher.update(b"mantle.ref.recipe.v1");
    hasher.update(graph.store_dir.as_bytes());
    hasher.update(task.name.as_bytes());
    hasher.update(task.builder.as_bytes());
    hasher.update(task.system.as_bytes());
    hasher.update(&task.salt.to_le_bytes());
    let mode_tag = match task.addressing_mode {
        TaskAddressingMode::InputAddressed => 0_u8,
        TaskAddressingMode::ContentAddressed => 1_u8,
    };
    hasher.update(&[mode_tag]);
    for out in &task.outputs {
        hasher.update(out.as_bytes());
        hasher.update(&[0]);
    }
    for (k, v) in &task.extra_env {
        hasher.update(k.as_bytes());
        hasher.update(v.as_bytes());
    }
    for src in &task.source_inputs {
        hasher.update(&src.0.to_le_bytes());
    }
    for dep in &task.deps {
        match dep {
            TaskDep::AllOutputs { target } => {
                hasher.update(&[1]);
                hasher.update(&target.0.to_le_bytes());
            }
            TaskDep::SelectedOutput { target, output } => {
                hasher.update(&[2]);
                hasher.update(&target.0.to_le_bytes());
                hasher.update(output.as_bytes());
            }
            TaskDep::PlanOutput {
                binding_name,
                producer,
                plan_output,
                root_unit,
                unit_output,
            } => {
                hasher.update(&[3]);
                hasher.update(binding_name.as_bytes());
                hasher.update(&producer.0.to_le_bytes());
                hasher.update(plan_output.as_bytes());
                hasher.update(root_unit.as_bytes());
                hasher.update(unit_output.as_bytes());
            }
        }
    }
    *hasher.finalize().as_bytes()
}
