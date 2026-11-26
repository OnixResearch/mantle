//! Production differential test harness, instrumentation, scenario verifiers, and mutation checks
//! (`#40`).

use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::Mutex;

use crunch_action_result_core::DiscoveredActionResultCandidate;
use crunch_action_result_core::StrongReuseRequest;
use crunch_action_result_core::plan_strong_reuse;
use crunch_glue::ConversionCache;
use crunch_glue::CrunchDerivation;
use crunch_glue::Input as GlueInput;
use crunch_glue::OutputRef as GlueOutputRef;
use crunch_glue::PlanOutputRef as GluePlanOutputRef;
use crunch_store::ActionResultStoreSet;
use crunch_store::LocalActionResultStore;
use crunch_store::StoreBackend;
use crunch_store::StoreHandle;
use crunch_store::StoreHandleServices;
use nix_compat::derivation::Derivation;
use nix_compat::nixhash::CAHash;
use nix_compat::nixhash::NixHash;
use nix_compat::store_path::StorePath;
use sha2::Digest as _;
use snix_castore::Node;
use snix_castore::blobservice::BlobService;
use snix_castore::blobservice::MemoryBlobService;
use snix_castore::directoryservice::DirectoryService;
use snix_store::path_info::PathInfo;
use snix_store::pathinfoservice::PathInfoService;
use tokio::io::AsyncWriteExt as _;
use tokio::sync::mpsc;
use tokio::sync::watch;

use super::evaluator::ReferenceEvaluation;
use super::evaluator::ReferenceRejection;
use super::evaluator::ReferenceResult;
use super::evaluator::evaluate_reference;
use super::task_language::DynamicPlanRule;
use super::task_language::GraphBounds;
use super::task_language::GraphFault;
use super::task_language::OutputRule;
use super::task_language::ReferenceTarget;
use super::task_language::RootRequest;
use super::task_language::TaskAddressingMode;
use super::task_language::TaskDep;
use super::task_language::TaskGraph;
use super::task_language::TaskId;
use super::task_language::TaskNode;
use crate::Builder;
use crate::DerivationRegistry;
use crate::EvalMessage;
use crate::GoalRegistry;
use crate::GoalState;
use crate::SchedulingPolicy;
use crate::Worker;
use crate::WorkerResult;
use crate::action_result::action_ref_for_derivation;
use crate::dynamic_plan::MANTLE_PLAN_V1_SCHEMA;
use crate::populate_registry;
use crate::signing;
use crate::test_support::ActionResultStoreCounts;
use crate::test_support::BlobServiceCounts;
use crate::test_support::CountingActionResultStore;
use crate::test_support::CountingBlobService;
use crate::test_support::CountingDirectoryService;
use crate::test_support::CountingPathInfoService;
use crate::test_support::DeterministicFakeBuildService;
use crate::test_support::DirectoryServiceCounts;
use crate::test_support::FakeBuildProgram;
use crate::test_support::FakeInputMatcher;
use crate::test_support::FakeOutputProgram;
use crate::test_support::FakeRefTagMatcher;
use crate::test_support::MANTLE_REF_PROG_ARG_PREFIX;
use crate::test_support::WatchBarrierGate;
use crate::test_support::compute_fake_exec_output_prefix;
use crate::test_support::corrupt_signature_bytes;
use crate::test_support::strip_refscan_suffix;
use crate::test_support::test_keypair;
use crate::test_support::test_trusted_keys;
use crate::test_support::tmp_ds;
use crate::worker::WatchWorkerEvent;
use crate::worker::WatchWorkerUpdate;

pub(crate) type GlueConverterFn =
    fn(&CrunchDerivation, &mut ConversionCache) -> Result<(StorePath<String>, Derivation), crunch_glue::Error>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ExecutionMetrics {
    pub executor_calls: u32,
    pub pathinfo_gets: u32,
    pub pathinfo_puts: u32,
    pub pathinfo_lists: u32,
    pub action_result: ActionResultStoreCounts,
    pub blobs: BlobServiceCounts,
    pub dirs: DirectoryServiceCounts,
}

pub(crate) struct InstrumentedEnv {
    _temp_dir: tempfile::TempDir,
    pub store_dir: String,
    pub output_dir: PathBuf,
    pub state_dir: PathBuf,
    pub blobs: CountingBlobService,
    pub dirs: CountingDirectoryService,
    pub pis: CountingPathInfoService,
    pub remote_pis: Option<CountingPathInfoService>,
    pub action_result_store: CountingActionResultStore,
    pub build_service: DeterministicFakeBuildService,
}

impl InstrumentedEnv {
    pub fn new(store_dir: &str) -> Self {
        Self::with_options(store_dir, None, false)
    }

    pub fn with_options(store_dir: &str, watch_gate: Option<WatchBarrierGate>, with_remote_pis: bool) -> Self {
        let temp_dir = tempfile::tempdir().expect("create isolated proptest tempdir");
        let output_dir = temp_dir.path().join("store");
        let state_dir = temp_dir.path().join("state");
        std::fs::create_dir_all(&output_dir).expect("create store output dir");
        std::fs::create_dir_all(&state_dir).expect("create state dir");

        let raw_blobs = MemoryBlobService::default();
        let blobs = CountingBlobService::new(Arc::new(raw_blobs));
        let dirs = CountingDirectoryService::new(Arc::new(tmp_ds()));
        let pis = CountingPathInfoService::default();
        let remote_pis = with_remote_pis.then(CountingPathInfoService::default);
        let local_ar = Arc::new(LocalActionResultStore::new(&state_dir));
        let action_result_store = CountingActionResultStore::new(local_ar);

        let mut build_service = DeterministicFakeBuildService::new(Arc::new(blobs.clone()));
        if let Some(gate) = watch_gate {
            build_service = build_service.with_watch_gate(gate);
        }

        Self {
            _temp_dir: temp_dir,
            store_dir: store_dir.to_string(),
            output_dir,
            state_dir,
            blobs,
            dirs,
            pis,
            remote_pis,
            action_result_store,
            build_service,
        }
    }

    pub fn make_builder(&self, trust_unsigned: bool) -> Builder<DeterministicFakeBuildService> {
        let mut handle = StoreHandle::from_services_with_store_dir(
            StoreBackend::Snix,
            StoreHandleServices {
                blob_service: Arc::new(self.blobs.clone()) as Arc<dyn BlobService>,
                directory_service: Arc::new(self.dirs.clone()) as Arc<dyn DirectoryService>,
                pathinfo_service: Arc::new(self.pis.clone()) as Arc<dyn PathInfoService>,
                remote_pathinfo: self.remote_pis.as_ref().map(|r| Arc::new(r.clone()) as Arc<dyn PathInfoService>),
                state_dir: self.state_dir.clone(),
                output_dir_str: self.output_dir.to_string_lossy().into_owned(),
                publishers: Vec::new(),
            },
            self.store_dir.clone(),
        )
        .expect("build StoreHandle from instrumented services");

        let mut stores = ActionResultStoreSet::new(true);
        stores.add_local(Box::new(self.action_result_store.clone()));
        handle.replace_action_result_stores(stores);

        Builder::from_store_parts(
            handle.into_builder_store_parts(),
            self.build_service.clone(),
            test_keypair(),
            test_trusted_keys(),
            trust_unsigned,
            false,
        )
    }

    pub fn reset_metrics(&self) {
        self.build_service.clear_calls();
        self.pis.reset_counts();
        if let Some(remote) = &self.remote_pis {
            remote.reset_counts();
        }
        self.action_result_store.reset_counts();
        self.blobs.reset_counts();
        self.dirs.reset_counts();
    }

    pub fn metrics(&self) -> ExecutionMetrics {
        ExecutionMetrics {
            executor_calls: self.build_service.call_count(),
            pathinfo_gets: self.pis.total_get_count(),
            pathinfo_puts: self.pis.put_count(),
            pathinfo_lists: self.pis.list_count(),
            action_result: self.action_result_store.counts(),
            blobs: self.blobs.counts(),
            dirs: self.dirs.counts(),
        }
    }

    pub async fn seed_sources(&self, graph: &TaskGraph) -> Result<BTreeMap<u32, StorePath<String>>, String> {
        let mut seeded = BTreeMap::new();
        let keypair = test_keypair();
        for src in &graph.sources {
            let mut writer = BlobService::open_write(&self.blobs).await;
            writer.write_all(&src.bytes).await.map_err(|e| e.to_string())?;
            let digest = writer.close().await.map_err(|e| e.to_string())?;
            let size = u64::try_from(src.bytes.len()).map_err(|e| e.to_string())?;
            let node = Node::File {
                digest,
                size,
                executable: false,
            };
            let sha256: [u8; 32] = sha2::Sha256::digest(&src.bytes).into();
            let sp: StorePath<String> = nix_compat::store_path::build_ca_path_with_store_dir(
                &src.name,
                &CAHash::Flat(NixHash::Sha256(sha256)),
                std::iter::empty::<String>(),
                false,
                &self.store_dir,
            )
            .map_err(|e| e.to_string())?;
            let disk_path = self.output_dir.join(sp.to_string());
            std::fs::write(&disk_path, &src.bytes).map_err(|e| e.to_string())?;

            // Compute NAR digest via BuildStore for exact PathInfo consistency.
            let tmp_handle = StoreHandle::from_services_with_store_dir(
                StoreBackend::Snix,
                StoreHandleServices {
                    blob_service: Arc::new(self.blobs.clone()) as Arc<dyn BlobService>,
                    directory_service: Arc::new(self.dirs.clone()) as Arc<dyn DirectoryService>,
                    pathinfo_service: Arc::new(self.pis.clone()) as Arc<dyn PathInfoService>,
                    remote_pathinfo: None,
                    state_dir: self.state_dir.clone(),
                    output_dir_str: self.output_dir.to_string_lossy().into_owned(),
                    publishers: Vec::new(),
                },
                self.store_dir.clone(),
            )
            .map_err(|e| e.to_string())?;
            let (nar_size, nar_sha256) = tmp_handle
                .into_builder_store_parts()
                .build_store
                .calculate_nar(&node)
                .await
                .map_err(|e| e.to_string())?;
            let mut pi = PathInfo {
                store_path: sp.clone(),
                node,
                references: Vec::new(),
                nar_size,
                nar_sha256,
                signatures: Vec::new(),
                deriver: None,
                ca: Some(CAHash::Flat(NixHash::Sha256(sha256))),
            };
            let _ = signing::sign_pathinfo_with_store_dir(&mut pi, &keypair.signing_key, &self.store_dir);
            self.pis.put(pi).await.map_err(|e| e.to_string())?;
            seeded.insert(src.id.0, sp);
        }
        Ok(seeded)
    }
}

pub(crate) struct LoweredGraph {
    pub registry: DerivationRegistry,
    pub task_drv_paths: BTreeMap<u32, StorePath<String>>,
    pub root_drv_paths: Vec<StorePath<String>>,
    pub source_paths: BTreeMap<u32, StorePath<String>>,
}

pub(crate) fn lower_task_graph(
    graph: &TaskGraph,
    source_paths: &BTreeMap<u32, StorePath<String>>,
) -> Result<LoweredGraph, crunch_glue::Error> {
    lower_task_graph_with_converter(graph, source_paths, crunch_glue::convert)
}

pub(crate) fn lower_task_graph_with_converter(
    graph: &TaskGraph,
    source_paths: &BTreeMap<u32, StorePath<String>>,
    convert_fn: GlueConverterFn,
) -> Result<LoweredGraph, crunch_glue::Error> {
    if matches!(graph.fault, GraphFault::DependencyCycle { .. }) {
        // Construct an explicit cyclic CrunchDerivation pair (self-nesting same visiting identity).
        let cyclic = CrunchDerivation {
            name: "cycle-node".to_string(),
            builder: "/bin/sh".to_string(),
            system: "x86_64-linux".to_string(),
            args: vec!["-c".to_string(), "cycle".to_string()],
            outputs: vec!["out".to_string()],
            dynamic_plan_outputs: Vec::new(),
            env: HashMap::new(),
            inputs: vec![GlueInput::Derivation(Box::new(CrunchDerivation {
                name: "cycle-node".to_string(),
                builder: "/bin/sh".to_string(),
                system: "x86_64-linux".to_string(),
                args: vec!["-c".to_string(), "cycle-inner".to_string()],
                outputs: vec!["out".to_string()],
                dynamic_plan_outputs: Vec::new(),
                env: HashMap::new(),
                inputs: Vec::new(),
                fixed_output: None,
                addressing_mode: "input-addressed".to_string(),
                provenance: None,
            }))],
            fixed_output: None,
            addressing_mode: "input-addressed".to_string(),
            provenance: None,
        };
        let mut cache = ConversionCache::new(&graph.store_dir);
        convert_fn(&cyclic, &mut cache)?;
    }

    let mut cache = ConversionCache::new(&graph.store_dir);
    let mut prefix_ctx = LoweringPrefixContext {
        registry: DerivationRegistry::new(&graph.store_dir),
        task_drv_paths: BTreeMap::new(),
        task_output_prefixes: BTreeMap::new(),
        dynamic_unit_prefixes: BTreeMap::new(),
        dynamic_path_tasks: BTreeSet::new(),
        non_ca_outputs: Vec::new(),
    };
    for src in &graph.sources {
        if let Some(sp) = source_paths.get(&src.id.0) {
            let content_digest = *blake3::hash(&src.bytes).as_bytes();
            prefix_ctx.non_ca_outputs.push((src.name.clone(), content_digest, sp.to_string()));
        }
    }

    let mut crunch_drvs: Vec<CrunchDerivation> = Vec::with_capacity(graph.tasks.len());
    for (idx, task) in graph.tasks.iter().enumerate() {
        compute_task_output_prefixes_for_lowering(task, graph, &mut prefix_ctx);

        let mut inputs: Vec<GlueInput> = Vec::new();
        let mut expected_inputs: Vec<FakeInputMatcher> = Vec::new();
        let mut plan_binding_args: Vec<String> = Vec::new();

        for sid in &task.source_inputs {
            let sp = source_paths.get(&sid.0).ok_or_else(|| crunch_glue::Error::MissingField {
                field: format!("source {}", sid.0),
            })?;
            inputs.push(GlueInput::Source(sp.to_absolute_path_with_prefix(&graph.store_dir)));
            expected_inputs.push(FakeInputMatcher::ExactComponent(sp.to_string()));
        }

        for dep in &task.deps {
            match dep {
                TaskDep::AllOutputs { target } => {
                    let target_drv = crunch_drvs
                        .get(target.0 as usize)
                        .ok_or_else(|| crunch_glue::Error::MissingField {
                            field: format!("dep task {}", target.0),
                        })?
                        .clone();
                    for out_name in &target_drv.outputs {
                        expected_inputs.push(resolve_task_output_matcher(
                            target.0,
                            out_name,
                            graph,
                            source_paths,
                            &prefix_ctx,
                        ));
                    }
                    inputs.push(GlueInput::Derivation(Box::new(target_drv)));
                }
                TaskDep::SelectedOutput { target, output } => {
                    let target_drv = crunch_drvs
                        .get(target.0 as usize)
                        .ok_or_else(|| crunch_glue::Error::MissingField {
                            field: format!("dep task {}", target.0),
                        })?
                        .clone();
                    expected_inputs.push(resolve_task_output_matcher(
                        target.0,
                        output,
                        graph,
                        source_paths,
                        &prefix_ctx,
                    ));
                    inputs.push(GlueInput::OutputSelection(Box::new(GlueOutputRef {
                        drv: target_drv,
                        output: output.clone(),
                    })));
                }
                TaskDep::PlanOutput {
                    binding_name,
                    producer,
                    plan_output,
                    root_unit,
                    unit_output,
                } => {
                    let prod_drv = crunch_drvs
                        .get(producer.0 as usize)
                        .ok_or_else(|| crunch_glue::Error::MissingField {
                            field: format!("producer task {}", producer.0),
                        })?
                        .clone();
                    expected_inputs.push(resolve_task_output_matcher(
                        producer.0,
                        plan_output,
                        graph,
                        source_paths,
                        &prefix_ctx,
                    ));
                    plan_binding_args
                        .push(format!("--plan-binding-{binding_name}={{{{mantle-plan-output:{binding_name}}}}}"));
                    inputs.push(GlueInput::PlanOutput(Box::new(GluePlanOutputRef {
                        name: binding_name.clone(),
                        producer: prod_drv,
                        plan_output: plan_output.clone(),
                        root: root_unit.clone(),
                        unit_output: unit_output.clone(),
                    })));
                }
            }
        }

        let program = build_fake_program(task, graph, expected_inputs, source_paths, &prefix_ctx)?;
        let prog_json = serde_json::to_string(&program).map_err(|e| crunch_glue::Error::MissingField {
            field: format!("prog json: {e}"),
        })?;
        let mut args = vec![format!("{MANTLE_REF_PROG_ARG_PREFIX}{prog_json}")];
        args.extend(plan_binding_args);

        let mut env: HashMap<String, String> = task.extra_env.clone().into_iter().collect();
        env.insert("MANTLE_REF_SALT".to_string(), task.salt.to_string());

        let addressing_mode = match task.addressing_mode {
            TaskAddressingMode::InputAddressed => "input-addressed".to_string(),
            TaskAddressingMode::ContentAddressed => "content-addressed".to_string(),
        };

        let crunch_drv = CrunchDerivation {
            name: task.name.clone(),
            builder: task.builder.clone(),
            system: task.system.clone(),
            args,
            outputs: task.outputs.clone(),
            dynamic_plan_outputs: task.dynamic_plan_outputs.clone(),
            env,
            inputs,
            fixed_output: None,
            addressing_mode,
            provenance: None,
        };
        let (drv_path, _) = convert_fn(&crunch_drv, &mut cache)?;
        populate_registry(&mut prefix_ctx.registry, cache.drain_pending());
        let task_idx = u32::try_from(idx).unwrap_or(u32::MAX);
        let has_dynamic_output_path = task.addressing_mode == TaskAddressingMode::ContentAddressed
            || task.deps.iter().any(|dep| match dep {
                TaskDep::AllOutputs { target } | TaskDep::SelectedOutput { target, .. } => {
                    prefix_ctx.dynamic_path_tasks.contains(&target.0)
                }
                TaskDep::PlanOutput { .. } => true,
            });
        if has_dynamic_output_path {
            prefix_ctx.dynamic_path_tasks.insert(task_idx);
        } else {
            let drv_abs = drv_path.to_absolute_path_with_prefix(&graph.store_dir);
            for out_name in &task.outputs {
                if let Some(sp) = prefix_ctx.registry.get_output_path(&drv_abs, out_name)
                    && let Some(prefix) = prefix_ctx.task_output_prefixes.get(&(task.id.0, out_name.clone()))
                {
                    let content_digest = *blake3::hash(prefix).as_bytes();
                    prefix_ctx.non_ca_outputs.push((
                        logical_output_name(&task.name, out_name),
                        content_digest,
                        sp.to_string(),
                    ));
                }
            }
        }
        prefix_ctx.task_drv_paths.insert(task_idx, drv_path);
        crunch_drvs.push(crunch_drv);
    }

    if let GraphFault::UnknownSelectedOutput { task, output } = &graph.fault {
        let target = crunch_drvs[task.0 as usize].clone();
        let bad_consumer = CrunchDerivation {
            name: "bad-select".to_string(),
            builder: "/bin/sh".to_string(),
            system: "x86_64-linux".to_string(),
            args: vec!["-c".to_string(), "echo bad".to_string()],
            outputs: vec!["out".to_string()],
            dynamic_plan_outputs: Vec::new(),
            env: HashMap::new(),
            inputs: vec![GlueInput::OutputSelection(Box::new(GlueOutputRef {
                drv: target,
                output: output.clone(),
            }))],
            fixed_output: None,
            addressing_mode: "input-addressed".to_string(),
            provenance: None,
        };
        let mut fault_cache = ConversionCache::new(&graph.store_dir);
        convert_fn(&bad_consumer, &mut fault_cache)?;
    }

    let mut root_drv_paths = Vec::with_capacity(graph.roots.len());
    for root in &graph.roots {
        let drv_path = prefix_ctx.task_drv_paths[&root.task.0].clone();
        root_drv_paths.push(drv_path);
    }

    Ok(LoweredGraph {
        registry: prefix_ctx.registry,
        task_drv_paths: prefix_ctx.task_drv_paths,
        root_drv_paths,
        source_paths: source_paths.clone(),
    })
}

struct LoweringPrefixContext {
    registry: DerivationRegistry,
    task_drv_paths: BTreeMap<u32, StorePath<String>>,
    task_output_prefixes: BTreeMap<(u32, String), Vec<u8>>,
    dynamic_unit_prefixes: BTreeMap<(u32, String, String), Vec<u8>>,
    dynamic_path_tasks: BTreeSet<u32>,
    non_ca_outputs: Vec<(String, [u8; 32], String)>,
}

fn logical_output_name(task_name: &str, out_name: &str) -> String {
    if out_name == "out" {
        task_name.to_string()
    } else {
        format!("{task_name}-{out_name}")
    }
}

fn sort_lowering_inputs(inputs: &mut [(String, Vec<u8>)]) {
    inputs.sort_by(|left, right| {
        let left_hash = *blake3::hash(&left.1).as_bytes();
        let right_hash = *blake3::hash(&right.1).as_bytes();
        left.0.cmp(&right.0).then(left_hash.cmp(&right_hash))
    });
}

fn compute_task_output_prefixes_for_lowering(task: &TaskNode, graph: &TaskGraph, ctx: &mut LoweringPrefixContext) {
    let mut closure_queue: std::collections::VecDeque<(Option<u32>, Option<String>, Option<u32>)> =
        std::collections::VecDeque::new();
    let mut seen_closure: BTreeSet<(Option<u32>, Option<String>, Option<u32>)> = BTreeSet::new();
    let mut mounted_inputs: Vec<(String, Vec<u8>)> = Vec::new();

    for sid in &task.source_inputs {
        closure_queue.push_back((None, None, Some(sid.0)));
    }
    for dep in &task.deps {
        match dep {
            TaskDep::AllOutputs { target } => {
                if let Some(dep_task) = graph.tasks.get(target.0 as usize) {
                    for out_name in &dep_task.outputs {
                        closure_queue.push_back((Some(target.0), Some(out_name.clone()), None));
                    }
                }
            }
            TaskDep::SelectedOutput { target, output } => {
                closure_queue.push_back((Some(target.0), Some(output.clone()), None));
            }
            TaskDep::PlanOutput {
                producer,
                root_unit,
                unit_output,
                ..
            } => {
                if let Some(dyn_bytes) =
                    ctx.dynamic_unit_prefixes.get(&(producer.0, root_unit.clone(), unit_output.clone()))
                {
                    let unit_name = graph
                        .tasks
                        .get(producer.0 as usize)
                        .and_then(|prod_task| prod_task.dynamic_plan_rule.as_ref())
                        .and_then(|rule| {
                            rule.branches
                                .iter()
                                .flat_map(|branch| branch.units.iter())
                                .find(|unit| unit.unit_id == *root_unit)
                        })
                        .map_or_else(|| root_unit.clone(), |unit| unit.name.clone());
                    mounted_inputs.push((unit_name, dyn_bytes.clone()));
                }
            }
        }
    }

    while let Some(item) = closure_queue.pop_front() {
        if !seen_closure.insert(item.clone()) {
            continue;
        }
        match item {
            (None, None, Some(sid)) => {
                if let Some(src) = graph.sources.iter().find(|s| s.id.0 == sid) {
                    mounted_inputs.push((src.name.clone(), src.bytes.clone()));
                }
            }
            (Some(t_idx), Some(out_name), None) => {
                if let Some(dep_task) = graph.tasks.get(t_idx as usize)
                    && let Some(prefix) = ctx.task_output_prefixes.get(&(t_idx, out_name.clone()))
                {
                    mounted_inputs.push((logical_output_name(&dep_task.name, &out_name), prefix.clone()));
                    let rule = dep_task.output_rules.get(&out_name).cloned().unwrap_or(OutputRule {
                        salt: dep_task.salt,
                        ..OutputRule::default()
                    });
                    for target_ref in &rule.reference_targets {
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

    sort_lowering_inputs(&mut mounted_inputs);
    let ignore_plan_inputs = task.deps.iter().any(|d| matches!(d, TaskDep::PlanOutput { .. }));
    let hash_inputs: Vec<(&str, &[u8])> = mounted_inputs
        .iter()
        .filter(|(logical, _)| !(ignore_plan_inputs && logical.ends_with("-plan")))
        .map(|(logical, bytes)| (logical.as_str(), bytes.as_slice()))
        .collect();

    let mut plan_branch_bytes: Option<Vec<u8>> = None;
    if let Some(plan_rule) = &task.dynamic_plan_rule
        && !plan_rule.branches.is_empty()
        && let Ok(serialized_branches) = serialize_dynamic_branches(plan_rule, &graph.store_dir, &task.name)
    {
        let mut hasher = blake3::Hasher::new();
        for (_, bytes) in &hash_inputs {
            hasher.update(&(bytes.len() as u64).to_le_bytes());
            hasher.update(bytes);
        }
        let digest = hasher.finalize();
        let branch_idx = usize::from(digest.as_bytes()[0]) % plan_rule.branches.len();
        plan_branch_bytes = serialized_branches.get(branch_idx).cloned();
        let branch = &plan_rule.branches[branch_idx];
        if !branch.force_invalid_plan {
            let mut evaluated_units: BTreeMap<(String, String), Vec<u8>> = BTreeMap::new();
            for unit in &branch.units {
                let mut unit_inputs: Vec<(String, Vec<u8>)> = Vec::new();
                for sid in &unit.source_ids {
                    if let Some(src) = graph.sources.iter().find(|s| s.id == *sid) {
                        unit_inputs.push((src.name.clone(), src.bytes.clone()));
                    }
                }
                for (dep_uid, dep_out) in &unit.dep_unit_outputs {
                    if let Some(dep_unit) = branch.units.iter().find(|u| u.unit_id == *dep_uid)
                        && let Some(dep_bytes) = evaluated_units.get(&(dep_uid.clone(), dep_out.clone()))
                    {
                        unit_inputs.push((dep_unit.name.clone(), dep_bytes.clone()));
                    }
                }
                sort_lowering_inputs(&mut unit_inputs);
                let unit_input_refs: Vec<(&str, &[u8])> =
                    unit_inputs.iter().map(|(name, bytes)| (name.as_str(), bytes.as_slice())).collect();
                for out_name in &unit.outputs {
                    let payload = compute_fake_exec_output_prefix(&unit.name, out_name, unit.salt, &unit_input_refs);
                    evaluated_units.insert((unit.unit_id.clone(), out_name.clone()), payload.clone());
                    if branch.roots.contains(&unit.unit_id) {
                        ctx.dynamic_unit_prefixes.insert((task.id.0, unit.unit_id.clone(), out_name.clone()), payload);
                    }
                }
            }
        }
    }

    for out_name in &task.outputs {
        let rule = task.output_rules.get(out_name).cloned().unwrap_or(OutputRule {
            salt: task.salt,
            ..OutputRule::default()
        });
        let prefix = if task.dynamic_plan_rule.as_ref().is_some_and(|pr| pr.plan_output == *out_name) {
            plan_branch_bytes.clone().unwrap_or_default()
        } else if let Some(override_bytes) = &rule.ca_content_override {
            override_bytes.clone()
        } else {
            compute_fake_exec_output_prefix(&task.name, out_name, rule.salt, &hash_inputs)
        };
        ctx.task_output_prefixes.insert((task.id.0, out_name.clone()), prefix);
    }
}

fn build_expected_ref_matchers(
    target_idx: u32,
    out_name: &str,
    graph: &TaskGraph,
    source_paths: &BTreeMap<u32, StorePath<String>>,
    ctx: &LoweringPrefixContext,
) -> Vec<FakeRefTagMatcher> {
    let Some(dep_task) = graph.tasks.get(target_idx as usize) else {
        return Vec::new();
    };
    let rule = dep_task.output_rules.get(out_name).cloned().unwrap_or(OutputRule {
        salt: dep_task.salt,
        ..OutputRule::default()
    });
    let mut expected_refs = Vec::new();
    if rule.embed_self_output_ref {
        expected_refs.push(FakeRefTagMatcher::SelfOutput);
    }
    for target in &rule.reference_targets {
        if matches!(target, ReferenceTarget::SiblingOutput(_)) {
            expected_refs.push(FakeRefTagMatcher::SiblingOutput);
        }
    }
    for target in &rule.reference_targets {
        match target {
            ReferenceTarget::SiblingOutput(_) => {}
            ReferenceTarget::DepOutput { dep_index, output } => {
                if let Some(sub_dep) = dep_task.deps.get(*dep_index as usize) {
                    let sub_idx = sub_dep.target_task().0;
                    if let Some(sub_task) = graph.tasks.get(sub_idx as usize) {
                        if !ctx.dynamic_path_tasks.contains(&sub_idx)
                            && let Some(sub_drv) = ctx.task_drv_paths.get(&sub_idx)
                            && let Some(sub_sp) = ctx
                                .registry
                                .get_output_path(&sub_drv.to_absolute_path_with_prefix(&graph.store_dir), output)
                        {
                            expected_refs.push(FakeRefTagMatcher::InputComponent(sub_sp.to_string()));
                        } else {
                            let sub_logical = logical_output_name(&sub_task.name, output);
                            let sub_digest = ctx
                                .task_output_prefixes
                                .get(&(sub_idx, output.clone()))
                                .map_or([0u8; 32], |bytes| *blake3::hash(bytes).as_bytes());
                            expected_refs.push(FakeRefTagMatcher::InputContent {
                                logical_name: sub_logical,
                                content_digest: sub_digest,
                            });
                        }
                    }
                }
            }
            ReferenceTarget::Source(sub_sid) => {
                if let Some(sp) = source_paths.get(&sub_sid.0) {
                    expected_refs.push(FakeRefTagMatcher::InputComponent(sp.to_string()));
                }
            }
        }
    }
    expected_refs
}

fn resolve_task_output_matcher(
    target_idx: u32,
    out_name: &str,
    graph: &TaskGraph,
    source_paths: &BTreeMap<u32, StorePath<String>>,
    ctx: &LoweringPrefixContext,
) -> FakeInputMatcher {
    let Some(target_task) = graph.tasks.get(target_idx as usize) else {
        return FakeInputMatcher::NameOrComponent(out_name.to_string());
    };
    let logical_name = logical_output_name(&target_task.name, out_name);
    if !ctx.dynamic_path_tasks.contains(&target_idx) {
        if let Some(drv_sp) = ctx.task_drv_paths.get(&target_idx)
            && let Some(out_sp) =
                ctx.registry.get_output_path(&drv_sp.to_absolute_path_with_prefix(&graph.store_dir), out_name)
        {
            FakeInputMatcher::ExactComponent(out_sp.to_string())
        } else {
            FakeInputMatcher::NameOrComponent(logical_name)
        }
    } else {
        let Some(prefix) = ctx.task_output_prefixes.get(&(target_idx, out_name.to_string())) else {
            return FakeInputMatcher::NameOrComponent(logical_name);
        };
        let content_digest = *blake3::hash(prefix).as_bytes();
        let exclude_components: Vec<String> = ctx
            .non_ca_outputs
            .iter()
            .filter(|(name, digest, _)| name == &logical_name && digest == &content_digest)
            .map(|(_, _, comp)| comp.clone())
            .collect();
        let expected_refs = build_expected_ref_matchers(target_idx, out_name, graph, source_paths, ctx);
        FakeInputMatcher::ContentDigest {
            logical_name,
            content_digest,
            exclude_components,
            expected_refs,
        }
    }
}

fn build_fake_program(
    task: &TaskNode,
    graph: &TaskGraph,
    expected_inputs: Vec<FakeInputMatcher>,
    source_paths: &BTreeMap<u32, StorePath<String>>,
    ctx: &LoweringPrefixContext,
) -> Result<FakeBuildProgram, crunch_glue::Error> {
    let mut output_programs = BTreeMap::new();
    for out_name in &task.outputs {
        let rule = task.output_rules.get(out_name).cloned().unwrap_or(OutputRule {
            salt: task.salt,
            ..OutputRule::default()
        });
        let mut ref_sibling_outputs = Vec::new();
        let mut ref_inputs = Vec::new();
        for target in &rule.reference_targets {
            match target {
                ReferenceTarget::SiblingOutput(sib) => ref_sibling_outputs.push(sib.clone()),
                ReferenceTarget::DepOutput { dep_index, output } => {
                    if let Some(dep) = task.deps.get(*dep_index as usize) {
                        ref_inputs.push(resolve_task_output_matcher(
                            dep.target_task().0,
                            output,
                            graph,
                            source_paths,
                            ctx,
                        ));
                    }
                }
                ReferenceTarget::Source(sid) => {
                    if let Some(sp) = source_paths.get(&sid.0) {
                        ref_inputs.push(FakeInputMatcher::ExactComponent(sp.to_string()));
                    } else if let Some(src) = graph.sources.iter().find(|s| s.id == *sid) {
                        ref_inputs.push(FakeInputMatcher::NameOrComponent(src.name.clone()));
                    }
                }
            }
        }
        let dynamic_plan_branches = if let Some(plan_rule) = &task.dynamic_plan_rule
            && plan_rule.plan_output == *out_name
        {
            Some(serialize_dynamic_branches(plan_rule, &graph.store_dir, &task.name)?)
        } else {
            None
        };

        output_programs.insert(out_name.clone(), FakeOutputProgram {
            salt: rule.salt,
            ca_content_override: rule.ca_content_override,
            embed_self_output_ref: rule.embed_self_output_ref,
            ref_sibling_outputs,
            ref_input_names: Vec::new(),
            ref_inputs,
            dynamic_plan_branches,
        });
    }
    let ignore_plan_inputs = task.deps.iter().any(|d| matches!(d, TaskDep::PlanOutput { .. }));
    Ok(FakeBuildProgram {
        task_name: task.name.clone(),
        expected_input_names: Vec::new(),
        expected_inputs,
        ignore_plan_inputs,
        output_programs,
        fail_message: None,
        omit_outputs: Vec::new(),
    })
}

fn serialize_dynamic_branches(
    rule: &DynamicPlanRule,
    store_dir: &str,
    producer_name: &str,
) -> Result<Vec<Vec<u8>>, crunch_glue::Error> {
    let builder_path = format!("{store_dir}/00000000000000000000000000000000-builder/bin/sh");
    let mut serialized = Vec::with_capacity(rule.branches.len());
    for branch in &rule.branches {
        if branch.force_invalid_plan {
            serialized.push(b"{not-valid-mantle-plan".to_vec());
            continue;
        }
        let mut units_json = Vec::with_capacity(branch.units.len());
        for unit in &branch.units {
            let mut unit_out_programs = BTreeMap::new();
            for out_name in &unit.outputs {
                unit_out_programs.insert(out_name.clone(), FakeOutputProgram {
                    salt: unit.salt,
                    ca_content_override: None,
                    embed_self_output_ref: false,
                    ref_sibling_outputs: Vec::new(),
                    ref_input_names: Vec::new(),
                    ref_inputs: Vec::new(),
                    dynamic_plan_branches: None,
                });
            }
            let dep_names: Vec<String> = unit
                .dep_unit_outputs
                .iter()
                .filter_map(|(uid, _)| branch.units.iter().find(|u| u.unit_id == *uid).map(|u| u.name.clone()))
                .collect();
            let unit_prog = FakeBuildProgram {
                task_name: unit.name.clone(),
                expected_input_names: dep_names,
                expected_inputs: Vec::new(),
                ignore_plan_inputs: false,
                output_programs: unit_out_programs,
                fail_message: None,
                omit_outputs: Vec::new(),
            };
            let unit_prog_json = serde_json::to_string(&unit_prog).map_err(|e| crunch_glue::Error::MissingField {
                field: format!("unit prog json: {e}"),
            })?;
            let unit_inputs: Vec<serde_json::Value> = unit
                .dep_unit_outputs
                .iter()
                .map(|(uid, out)| {
                    serde_json::json!({
                        "kind": "unit_output",
                        "unit": uid,
                        "output": out,
                    })
                })
                .collect();
            let addr = match unit.addressing_mode {
                TaskAddressingMode::InputAddressed => "input-addressed",
                TaskAddressingMode::ContentAddressed => "content-addressed",
            };
            units_json.push(serde_json::json!({
                "id": unit.unit_id,
                "derivation": {
                    "name": unit.name,
                    "builder": builder_path,
                    "system": "x86_64-linux",
                    "args": [format!("{MANTLE_REF_PROG_ARG_PREFIX}{unit_prog_json}")],
                    "outputs": unit.outputs,
                    "env": { "MANTLE_REF_SALT": unit.salt.to_string() },
                    "inputs": unit_inputs,
                    "fixed_output": null,
                    "addressing_mode": addr,
                    "sandbox": "native",
                    "dynamic_plan_outputs": []
                },
                "requested_outputs": unit.outputs,
                "policy": {
                    "sandbox": "inherit",
                    "substitutions": "inherit",
                    "store_prefix": "inherit",
                    "host_paths": "none"
                }
            }));
        }
        let doc = serde_json::json!({
            "schema": MANTLE_PLAN_V1_SCHEMA,
            "producer": { "logical_name": producer_name, "goal_hint": null },
            "sources": [],
            "units": units_json,
            "roots": branch.roots,
            "provenance": {}
        });
        let bytes = serde_json::to_vec(&doc).map_err(|e| crunch_glue::Error::MissingField {
            field: format!("plan json: {e}"),
        })?;
        serialized.push(bytes);
    }
    Ok(serialized)
}

pub(crate) async fn verify_static_graph_differential(
    graph: &TaskGraph,
    bounds: &GraphBounds,
    root_order_reverse: bool,
) -> Result<(), String> {
    let eval = match evaluate_reference(graph, bounds) {
        ReferenceResult::Evaluated(eval) => eval,
        ReferenceResult::Rejected(rej) => {
            // Confirm lowering or Worker rejects cleanly too.
            let env = InstrumentedEnv::new(&graph.store_dir);
            let sources = env.seed_sources(graph).await?;
            match lower_task_graph(graph, &sources) {
                Err(_) => return Ok(()),
                Ok(mut lowered) => {
                    let mut worker = Worker::new(2);
                    for (idx, root) in graph.roots.iter().enumerate() {
                        if worker
                            .want_outputs(
                                &lowered.root_drv_paths[idx],
                                &lowered.registry,
                                true,
                                root.selected_outputs.clone(),
                            )
                            .is_err()
                        {
                            return Ok(());
                        }
                    }
                    let mut builder = env.make_builder(false);
                    let res = worker.run(&mut builder, &mut lowered.registry).await;
                    return if res.is_err() {
                        Ok(())
                    } else {
                        Err(format!("reference rejected graph with {rej:?} but production succeeded"))
                    };
                }
            }
        }
        ReferenceResult::Exhausted(exh) => {
            return Err(format!(
                "reference evaluation exhausted step budget ({} / {}) — exhaustion must never pass",
                exh.steps_executed, exh.max_eval_steps
            ));
        }
    };

    let env = InstrumentedEnv::new(&graph.store_dir);
    let sources = env.seed_sources(graph).await?;
    let mut lowered = lower_task_graph(graph, &sources).map_err(|e| format!("lower failed: {e}"))?;
    let mut builder = env.make_builder(false);
    let mut worker = Worker::new(2);

    let mut ordered_roots: Vec<(usize, &RootRequest)> = graph.roots.iter().enumerate().collect();
    if root_order_reverse {
        ordered_roots.reverse();
    } else {
        ordered_roots.sort_by_key(|(idx, req)| (req.arrival_batch, *idx));
    }

    for (idx, req) in &ordered_roots {
        worker
            .want_outputs(&lowered.root_drv_paths[*idx], &lowered.registry, true, req.selected_outputs.clone())
            .map_err(|e| format!("worker want_outputs failed: {e}"))?;
    }

    let worker_res =
        tokio::time::timeout(std::time::Duration::from_secs(10), worker.run(&mut builder, &mut lowered.registry))
            .await
            .map_err(|_| "worker.run timed out".to_string())?
            .map_err(|e| format!("worker.run failed: {e}"))?;

    if !worker_res.failed.is_empty() {
        return Err(format!("unexpected failed roots: {:?}", worker_res.failed));
    }

    // Verify no duplicate root outcomes for same drv_path.
    let mut seen_outcome_drvs = BTreeSet::new();
    for outcome in &worker_res.outcomes {
        if !seen_outcome_drvs.insert(outcome.drv_path.to_absolute_path()) {
            return Err(format!("duplicate BuildOutcome for {}", outcome.drv_path));
        }
    }

    verify_outcomes_against_eval(graph, &eval, &lowered, &worker_res, &env).await?;

    let mut expected_demanded_by_drv: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    for (task_idx, demanded) in &eval.task_demanded_outputs {
        let drv_key = lowered.task_drv_paths[task_idx].to_absolute_path();
        expected_demanded_by_drv.entry(drv_key).or_default().extend(demanded.iter().cloned());
    }
    let cold_all_by_drv: BTreeMap<String, &crate::BuildOutcome> =
        worker_res.all_outcomes.iter().map(|o| (o.drv_path.to_absolute_path(), o)).collect();
    for (drv_key, expected_outputs) in &expected_demanded_by_drv {
        let outcome = cold_all_by_drv
            .get(drv_key)
            .ok_or_else(|| format!("missing cold all_outcomes entry for demanded drv {drv_key}"))?;
        for out_name in expected_outputs {
            if !outcome.outputs.contains_key(out_name) {
                return Err(format!("cold outcome {drv_key} missing demanded output '{out_name}'"));
            }
        }
    }
    for drv_key in cold_all_by_drv.keys() {
        if !expected_demanded_by_drv.contains_key(drv_key) {
            return Err(format!("cold run executed/admitted undemanded drv {drv_key}"));
        }
    }
    for drv_path in lowered.task_drv_paths.values() {
        let drv_key = drv_path.to_absolute_path();
        if worker.demanded_outputs_for(drv_path) != expected_demanded_by_drv.get(&drv_key) {
            return Err(format!(
                "Worker::goal_demands mismatch for {drv_key}: actual {:?} != expected {:?}",
                worker.demanded_outputs_for(drv_path),
                expected_demanded_by_drv.get(&drv_key),
            ));
        }
    }

    let cold_metrics = env.metrics();
    if cold_metrics.executor_calls as usize > eval.unique_task_recipes.len() {
        return Err(format!(
            "executor calls ({}) exceeded unique tasks ({})",
            cold_metrics.executor_calls,
            eval.unique_task_recipes.len()
        ));
    }

    // Second warm pass: verify #41 preflight / #35 demand-projected / CA reuse yields 0 executor calls!
    env.reset_metrics();
    let mut warm_builder = env.make_builder(false);
    let mut warm_worker = Worker::new(2);
    for (idx, req) in &ordered_roots {
        warm_worker
            .want_outputs(&lowered.root_drv_paths[*idx], &lowered.registry, true, req.selected_outputs.clone())
            .map_err(|e| format!("warm want_outputs failed: {e}"))?;
    }
    let warm_res = tokio::time::timeout(
        std::time::Duration::from_secs(10),
        warm_worker.run(&mut warm_builder, &mut lowered.registry),
    )
    .await
    .map_err(|_| "warm worker.run timed out".to_string())?
    .map_err(|e| format!("warm worker.run failed: {e}"))?;

    if !warm_res.failed.is_empty() {
        return Err(format!("warm run failed: {:?}", warm_res.failed));
    }
    let warm_metrics = env.metrics();
    if warm_metrics.executor_calls != 0 {
        return Err(format!("warm run re-executed {} tasks instead of 0", warm_metrics.executor_calls));
    }
    for outcome in &warm_res.outcomes {
        if !outcome.cached {
            return Err(format!("warm outcome for {} had cached=false", outcome.drv_path));
        }
    }
    verify_outcomes_against_eval(graph, &eval, &lowered, &warm_res, &env).await?;
    for outcome in &warm_res.all_outcomes {
        let drv_key = outcome.drv_path.to_absolute_path();
        let expected_outputs = expected_demanded_by_drv
            .get(&drv_key)
            .ok_or_else(|| format!("warm run returned outcome for undemanded drv {drv_key}"))?;
        for out_name in expected_outputs {
            if !outcome.outputs.contains_key(out_name) {
                return Err(format!("warm outcome {drv_key} missing demanded output '{out_name}'"));
            }
        }
    }

    // For #41 preflight-eligible roots that are not shared as a root with their own build-only leaf,
    // verify zero PathInfo reads happened for non-referenced build-only ancestors when root hits
    // preflight.
    if graph.roots.len() == 1
        && eval.preflight_eligible_roots.contains(&graph.roots[0].task.0)
        && graph.tasks[graph.roots[0].task.0 as usize]
            .output_rules
            .values()
            .all(|r| r.reference_targets.is_empty())
    {
        for (task_idx, drv_path) in &lowered.task_drv_paths {
            if *task_idx == graph.roots[0].task.0 {
                continue;
            }
            if let Some(entry) =
                lowered.registry.get_by_drv_path(&drv_path.to_absolute_path_with_prefix(&graph.store_dir))
            {
                for (out_name, out) in &entry.derivation.outputs {
                    if let Some(sp) = &out.path
                        && env.pis.get_count(sp) != 0
                    {
                        return Err(format!(
                            "#41 preflight probed build-only ancestor {}:{out_name} ({} times)",
                            entry.drv_path.name(),
                            env.pis.get_count(sp)
                        ));
                    }
                }
            }
        }
    }

    Ok(())
}

async fn verify_outcomes_against_eval(
    graph: &TaskGraph,
    eval: &ReferenceEvaluation,
    lowered: &LoweredGraph,
    worker_res: &WorkerResult,
    env: &InstrumentedEnv,
) -> Result<(), String> {
    let outcome_by_drv: BTreeMap<String, &crate::BuildOutcome> =
        worker_res.outcomes.iter().map(|o| (o.drv_path.to_absolute_path(), o)).collect();

    for (root_idx, root) in graph.roots.iter().enumerate() {
        let root_drv = &lowered.root_drv_paths[root_idx];
        let outcome = outcome_by_drv
            .get(&root_drv.to_absolute_path())
            .ok_or_else(|| format!("missing BuildOutcome for root {root_drv}"))?;
        let task = &graph.tasks[root.task.0 as usize];
        let wanted: BTreeSet<String> = match &root.selected_outputs {
            Some(sel) => sel.clone(),
            None => task.outputs.iter().cloned().collect(),
        };
        for out_name in &wanted {
            let pi = outcome
                .outputs
                .get(out_name)
                .ok_or_else(|| format!("root {root_drv} missing demanded output '{out_name}'"))?;
            let Node::File { digest, .. } = &pi.node else {
                return Err(format!("expected Node::File for {root_drv}:{out_name}"));
            };
            let actual_bytes = env
                .blobs
                .read_all_bytes_uncounted(digest)
                .await
                .map_err(|e| format!("reading blob: {e}"))?
                .ok_or_else(|| format!("missing blob for {root_drv}:{out_name}"))?;
            let actual_prefix = strip_refscan_suffix(&actual_bytes);
            let expected = eval
                .root_outputs
                .get(&(u32::try_from(root_idx).unwrap_or(u32::MAX), out_name.clone()))
                .ok_or_else(|| format!("missing eval output for root {root_idx}:{out_name}"))?;
            if actual_prefix != expected.content_prefix.as_slice() {
                return Err(format!(
                    "output content mismatch for task '{}' output '{out_name}': actual {:?} != expected {:?}",
                    task.name,
                    String::from_utf8_lossy(actual_prefix),
                    String::from_utf8_lossy(&expected.content_prefix)
                ));
            }
            for target in &expected.expected_ref_targets {
                match target {
                    ReferenceTarget::SiblingOutput(sib) => {
                        let sib_sp = outcome.outputs.get(sib).map(|s| s.store_path.clone()).or_else(|| {
                            lowered
                                .registry
                                .get_output_path(&root_drv.to_absolute_path_with_prefix(&graph.store_dir), sib)
                        });
                        if let Some(sp) = sib_sp
                            && !pi.references.contains(&sp)
                        {
                            return Err(format!("missing sibling reference {sp} in PathInfo.references"));
                        }
                    }
                    ReferenceTarget::DepOutput { dep_index, output } => {
                        if let Some(dep) = task.deps.get(*dep_index as usize)
                            && let Some(dep_drv) = lowered.task_drv_paths.get(&dep.target_task().0)
                            && let Some(dep_sp) = lowered
                                .registry
                                .get_output_path(&dep_drv.to_absolute_path_with_prefix(&graph.store_dir), output)
                            && !pi.references.contains(&dep_sp)
                        {
                            return Err(format!("missing dep reference {dep_sp} in PathInfo.references"));
                        }
                    }
                    ReferenceTarget::Source(sid) => {
                        if let Some(src_sp) = lowered.source_paths.get(&sid.0)
                            && !pi.references.contains(src_sp)
                        {
                            return Err(format!("missing source reference {src_sp} in PathInfo.references"));
                        }
                    }
                }
            }
        }
    }
    Ok(())
}

// ── Scenario 1: Same name/builder/system, different recipe (#38) ────────────────

pub(crate) async fn verify_scenario_1_with_converter(
    reverse_order: bool,
    convert_fn: GlueConverterFn,
) -> Result<(), String> {
    let mut node_a = TaskGraph::simple_task(0, "colliding-pkg", 101, TaskAddressingMode::InputAddressed);
    node_a.output_rules.get_mut("out").expect("out").salt = 101;
    let mut node_b = TaskGraph::simple_task(1, "colliding-pkg", 202, TaskAddressingMode::InputAddressed);
    node_b.output_rules.get_mut("out").expect("out").salt = 202;

    let graph = TaskGraph {
        store_dir: "/nix/store".to_string(),
        sources: Vec::new(),
        tasks: vec![node_a, node_b],
        roots: vec![
            RootRequest {
                task: TaskId(0),
                selected_outputs: None,
                arrival_batch: if reverse_order { 1 } else { 0 },
            },
            RootRequest {
                task: TaskId(1),
                selected_outputs: None,
                arrival_batch: if reverse_order { 0 } else { 1 },
            },
        ],
        fault: GraphFault::None,
    };

    let eval = match evaluate_reference(&graph, &GraphBounds::default()) {
        ReferenceResult::Evaluated(e) => e,
        other => return Err(format!("unexpected eval result: {other:?}")),
    };
    if eval.unique_task_recipes.len() != 2 {
        return Err("reference evaluator collapsed distinct recipes".to_string());
    }

    let env = InstrumentedEnv::new(&graph.store_dir);
    let sources = env.seed_sources(&graph).await?;
    let mut lowered =
        lower_task_graph_with_converter(&graph, &sources, convert_fn).map_err(|e| format!("lower failed: {e}"))?;

    let drv_a = &lowered.task_drv_paths[&0];
    let drv_b = &lowered.task_drv_paths[&1];
    if drv_a == drv_b {
        return Err(format!("distinct recipes collapsed to identical drv_path {drv_a}"));
    }

    let mut builder = env.make_builder(false);
    let mut worker = Worker::new(2);
    if reverse_order {
        worker.want(drv_b, &lowered.registry, true).map_err(|e| e.to_string())?;
        worker.want(drv_a, &lowered.registry, true).map_err(|e| e.to_string())?;
    } else {
        worker.want(drv_a, &lowered.registry, true).map_err(|e| e.to_string())?;
        worker.want(drv_b, &lowered.registry, true).map_err(|e| e.to_string())?;
    }

    let res = worker.run(&mut builder, &mut lowered.registry).await.map_err(|e| e.to_string())?;
    if !res.failed.is_empty() || res.outcomes.len() != 2 {
        return Err(format!("expected 2 distinct outcomes, got {} (failed {:?})", res.outcomes.len(), res.failed));
    }
    if env.metrics().executor_calls != 2 {
        return Err(format!("expected 2 executor calls, got {}", env.metrics().executor_calls));
    }
    verify_outcomes_against_eval(&graph, &eval, &lowered, &res, &env).await
}

// ── Scenario 2: Diamond graph & duplicate root requests ─────────────────────────

pub(crate) async fn verify_scenario_2_diamond_and_duplicate_roots() -> Result<(), String> {
    let base = TaskGraph::simple_task(0, "diamond-base", 11, TaskAddressingMode::InputAddressed);
    let mut left = TaskGraph::simple_task(1, "diamond-left", 22, TaskAddressingMode::InputAddressed);
    left.deps.push(TaskDep::AllOutputs { target: TaskId(0) });
    let mut right = TaskGraph::simple_task(2, "diamond-right", 33, TaskAddressingMode::InputAddressed);
    right.deps.push(TaskDep::AllOutputs { target: TaskId(0) });
    let mut top = TaskGraph::simple_task(3, "diamond-top", 44, TaskAddressingMode::InputAddressed);
    top.deps.push(TaskDep::AllOutputs { target: TaskId(1) });
    top.deps.push(TaskDep::AllOutputs { target: TaskId(2) });

    let graph = TaskGraph {
        store_dir: "/nix/store".to_string(),
        sources: Vec::new(),
        tasks: vec![base, left, right, top],
        roots: vec![
            RootRequest {
                task: TaskId(3),
                selected_outputs: None,
                arrival_batch: 0,
            },
            RootRequest {
                task: TaskId(3),
                selected_outputs: None,
                arrival_batch: 1,
            },
            RootRequest {
                task: TaskId(0),
                selected_outputs: None,
                arrival_batch: 2,
            },
        ],
        fault: GraphFault::None,
    };

    let eval = match evaluate_reference(&graph, &GraphBounds::default()) {
        ReferenceResult::Evaluated(e) => e,
        other => return Err(format!("unexpected eval result: {other:?}")),
    };
    let env = InstrumentedEnv::new(&graph.store_dir);
    let sources = env.seed_sources(&graph).await?;
    let mut lowered = lower_task_graph(&graph, &sources).map_err(|e| e.to_string())?;
    let mut builder = env.make_builder(false);
    let mut worker = Worker::new(2);

    // Duplicate root wants + late streaming arrival of shared base after top starts.
    let top_sp = lowered.task_drv_paths[&3].clone();
    let base_sp = lowered.task_drv_paths[&0].clone();
    let (tx, mut rx) = mpsc::channel(4);
    tx.send(EvalMessage {
        label: "top-1".to_string(),
        drv_path: top_sp.clone(),
        new_entries: Vec::new(),
    })
    .await
    .map_err(|e| e.to_string())?;
    tx.send(EvalMessage {
        label: "top-dup".to_string(),
        drv_path: top_sp.clone(),
        new_entries: Vec::new(),
    })
    .await
    .map_err(|e| e.to_string())?;
    tx.send(EvalMessage {
        label: "base-dup".to_string(),
        drv_path: base_sp.clone(),
        new_entries: Vec::new(),
    })
    .await
    .map_err(|e| e.to_string())?;
    drop(tx);

    let res = worker
        .run_streaming(&mut builder, &mut lowered.registry, &mut rx)
        .await
        .map_err(|e| e.to_string())?;
    if !res.failed.is_empty() {
        return Err(format!("diamond failed: {:?}", res.failed));
    }
    let metrics = env.metrics();
    if metrics.executor_calls != 4 {
        return Err(format!("expected 4 unique executor calls, got {}", metrics.executor_calls));
    }
    let base_calls = env
        .build_service
        .recorded_calls()
        .iter()
        .filter(|c| c.task_name.as_deref() == Some("diamond-base"))
        .count();
    if base_calls != 1 {
        return Err(format!("diamond-base executed {base_calls} times instead of 1"));
    }
    let unique_roots: BTreeSet<String> = res.outcomes.iter().map(|o| o.drv_path.to_absolute_path()).collect();
    if unique_roots.len() != res.outcomes.len() {
        return Err("duplicate root outcomes returned".to_string());
    }
    verify_outcomes_against_eval(&graph, &eval, &lowered, &res, &env).await
}

// ── Scenario 3: Changed CA producer, unchanged realized output -> early cutoff ──

pub(crate) async fn verify_scenario_3_ca_early_cutoff(
    v1: &TaskGraph,
    v2: &TaskGraph,
    same_realized_ca: bool,
) -> Result<(), String> {
    let eval1 = match evaluate_reference(v1, &GraphBounds::default()) {
        ReferenceResult::Evaluated(e) => e,
        other => return Err(format!("v1 eval: {other:?}")),
    };
    let eval2 = match evaluate_reference(v2, &GraphBounds::default()) {
        ReferenceResult::Evaluated(e) => e,
        other => return Err(format!("v2 eval: {other:?}")),
    };

    let env = InstrumentedEnv::new(&v1.store_dir);
    let sources1 = env.seed_sources(v1).await?;
    let mut lowered1 = lower_task_graph(v1, &sources1).map_err(|e| e.to_string())?;
    let mut builder1 = env.make_builder(false);
    let mut worker1 = Worker::new(1);
    worker1.want(&lowered1.root_drv_paths[0], &lowered1.registry, true).map_err(|e| e.to_string())?;
    let res1 = worker1.run(&mut builder1, &mut lowered1.registry).await.map_err(|e| e.to_string())?;
    if !res1.failed.is_empty() || env.metrics().executor_calls != 3 {
        return Err(format!(
            "gen1 expected 3 executor calls and 0 failures, got {} calls, failed={:?}",
            env.metrics().executor_calls,
            res1.failed
        ));
    }
    verify_outcomes_against_eval(v1, &eval1, &lowered1, &res1, &env).await?;

    // Gen 2 on the same warm store: producer recipe changed, realized bytes either equal or changed.
    env.reset_metrics();
    let sources2 = env.seed_sources(v2).await?;
    let mut lowered2 = lower_task_graph(v2, &sources2).map_err(|e| e.to_string())?;
    let mut builder2 = env.make_builder(false);
    let mut worker2 = Worker::new(1);
    worker2.want(&lowered2.root_drv_paths[0], &lowered2.registry, true).map_err(|e| e.to_string())?;
    let res2 = worker2.run(&mut builder2, &mut lowered2.registry).await.map_err(|e| e.to_string())?;
    if !res2.failed.is_empty() {
        return Err(format!("gen2 failed: {:?}", res2.failed));
    }

    let calls2 = env.metrics().executor_calls;
    if same_realized_ca {
        if calls2 != 1 {
            return Err(format!("CA early cutoff failed: expected 1 executor call (producer only), got {calls2}"));
        }
        let child_drv = &lowered2.task_drv_paths[&1];
        let grand_drv = &lowered2.task_drv_paths[&2];
        let child_outcome =
            res2.all_outcomes.iter().find(|o| &o.drv_path == child_drv).ok_or("missing child outcome")?;
        let grand_outcome =
            res2.outcomes.iter().find(|o| &o.drv_path == grand_drv).ok_or("missing grandchild outcome")?;
        if !child_outcome.cached || !grand_outcome.cached {
            return Err("expected child and grandchild cached=true on CA early cutoff".to_string());
        }
    } else if calls2 != 3 {
        return Err(format!(
            "CA negative control failed: expected 3 executor calls when CA output changed, got {calls2}"
        ));
    }

    verify_outcomes_against_eval(v2, &eval2, &lowered2, &res2, &env).await
}

// ── Scenario 4: Fresh vs cached dynamic-plan producer & consumer wait ───────────

pub(crate) async fn verify_scenario_4_dynamic_plan(graph: &TaskGraph) -> Result<(), String> {
    let eval = match evaluate_reference(graph, &GraphBounds::default()) {
        ReferenceResult::Evaluated(e) => e,
        ReferenceResult::Exhausted(ex) => return Err(format!("exhausted: {ex:?}")),
        ReferenceResult::Rejected(rej) => return Err(format!("rejected: {rej:?}")),
    };

    let env = InstrumentedEnv::new(&graph.store_dir);
    let sources = env.seed_sources(graph).await?;

    // Fresh cold run with live GoalRegistry observer verifying consumer never leaves Waiting
    // before the generated dynamic root is Done.
    let mut lowered_fresh = lower_task_graph(graph, &sources).map_err(|e| e.to_string())?;
    let consumer_sp = lowered_fresh.root_drv_paths[0].clone();
    let consumer_key = consumer_sp.to_absolute_path();
    let producer_idx = graph
        .tasks
        .iter()
        .find(|t| !t.dynamic_plan_outputs.is_empty())
        .map(|t| t.id.0)
        .ok_or("missing dynamic producer")?;
    let producer_key = lowered_fresh.task_drv_paths[&producer_idx].to_absolute_path();

    let (tx, mut rx) = mpsc::channel(2);
    tx.send(EvalMessage {
        label: "dyn-consumer".to_string(),
        drv_path: consumer_sp.clone(),
        new_entries: Vec::new(),
    })
    .await
    .map_err(|e| e.to_string())?;
    drop(tx);

    let premature_release_violation = Arc::new(Mutex::new(None::<String>));
    let violation_clone = premature_release_violation.clone();
    let consumer_key_obs = consumer_key.clone();
    let producer_key_obs = producer_key.clone();
    let mut observer = move |reg: &GoalRegistry| {
        let Some(consumer_goal) = reg.get(&consumer_key_obs) else {
            return;
        };
        let producer_done = reg.get(&producer_key_obs).is_some_and(|g| g.state == GoalState::Done);
        let any_dyn_non_terminal = reg.iter().any(|(k, g)| {
            k != consumer_key_obs && k != producer_key_obs && !matches!(g.state, GoalState::Done | GoalState::Failed)
        });
        if (!producer_done || any_dyn_non_terminal)
            && matches!(consumer_goal.state, GoalState::Ready | GoalState::Building | GoalState::Done)
        {
            *violation_clone.lock().unwrap() = Some(format!(
                "consumer released prematurely into {:?} while producer_done={producer_done}, dyn_non_terminal={any_dyn_non_terminal}",
                consumer_goal.state
            ));
        }
    };

    let mut builder_fresh = env.make_builder(false);
    let mut worker_fresh = Worker::new(1);
    let fresh_res = worker_fresh
        .run_streaming_observed(&mut builder_fresh, &mut lowered_fresh.registry, &mut rx, Some(&mut observer))
        .await
        .map_err(|e| format!("fresh dynamic worker failed: {e}"))?;
    if let Some(msg) = premature_release_violation.lock().unwrap().take() {
        return Err(msg);
    }
    if !fresh_res.failed.is_empty() {
        return Err(format!("fresh dynamic failed: {:?}", fresh_res.failed));
    }
    verify_outcomes_against_eval(graph, &eval, &lowered_fresh, &fresh_res, &env).await?;

    // Second run: producer is cached, while dynamic units may also hit or rebuild cleanly.
    // Even with a fresh Worker + DerivationRegistry, cached producer admission MUST register the exact
    // same dynamic plan digest and bindings as the fresh run.
    env.reset_metrics();
    let mut lowered_cached = lower_task_graph(graph, &sources).map_err(|e| e.to_string())?;
    let mut builder_cached = env.make_builder(false);
    let mut worker_cached = Worker::new(1);
    worker_cached
        .want(&lowered_cached.root_drv_paths[0], &lowered_cached.registry, true)
        .map_err(|e| e.to_string())?;
    let cached_res = worker_cached
        .run(&mut builder_cached, &mut lowered_cached.registry)
        .await
        .map_err(|e| format!("cached dynamic worker failed: {e}"))?;
    if !cached_res.failed.is_empty() {
        return Err(format!("cached dynamic failed: {:?}", cached_res.failed));
    }

    if fresh_res.native_dynamic_plans.len() != cached_res.native_dynamic_plans.len() {
        return Err("fresh vs cached dynamic plan count diverged".to_string());
    }
    for (a, b) in fresh_res.native_dynamic_plans.iter().zip(&cached_res.native_dynamic_plans) {
        if a.canonical_plan_digest != b.canonical_plan_digest || a.accepted_unit_ids != b.accepted_unit_ids {
            return Err("fresh vs cached dynamic plan digest or admitted units diverged".to_string());
        }
    }
    if fresh_res.plan_output_bindings != cached_res.plan_output_bindings {
        return Err("fresh vs cached plan_output_bindings diverged".to_string());
    }
    verify_outcomes_against_eval(graph, &eval, &lowered_cached, &cached_res, &env).await
}

// ── Scenario 5: Missing intermediate or incomplete output objects ───────────────

pub(crate) async fn verify_scenario_5_missing_and_incomplete_objects() -> Result<(), String> {
    let mut leaf = TaskGraph::simple_task(0, "mid-leaf", 51, TaskAddressingMode::InputAddressed);
    leaf.output_rules.get_mut("out").expect("out").salt = 51;
    let mut root = TaskGraph::simple_task(1, "mid-root", 52, TaskAddressingMode::InputAddressed);
    root.deps.push(TaskDep::AllOutputs { target: TaskId(0) });
    root.output_rules.get_mut("out").expect("out").reference_targets.push(ReferenceTarget::DepOutput {
        dep_index: 0,
        output: "out".to_string(),
    });

    let graph = TaskGraph {
        store_dir: "/nix/store".to_string(),
        sources: Vec::new(),
        tasks: vec![leaf, root],
        roots: vec![RootRequest {
            task: TaskId(1),
            selected_outputs: None,
            arrival_batch: 0,
        }],
        fault: GraphFault::None,
    };
    let eval = match evaluate_reference(&graph, &GraphBounds::default()) {
        ReferenceResult::Evaluated(e) => e,
        other => return Err(format!("eval: {other:?}")),
    };

    let env = InstrumentedEnv::new(&graph.store_dir);
    let sources = env.seed_sources(&graph).await?;
    let mut lowered = lower_task_graph(&graph, &sources).map_err(|e| e.to_string())?;

    // Cold build populates both leaf and root.
    let mut builder = env.make_builder(false);
    let mut worker = Worker::new(1);
    worker.want(&lowered.root_drv_paths[0], &lowered.registry, true).map_err(|e| e.to_string())?;
    let res = worker.run(&mut builder, &mut lowered.registry).await.map_err(|e| e.to_string())?;
    let root_pi = res.outcomes[0].outputs["out"].clone();
    let Node::File {
        digest: root_digest, ..
    } = &root_pi.node
    else {
        return Err("expected Node::File".to_string());
    };

    // Subcase (a): Hide root's blob in CountingBlobService -> check_cache must NOT admit incomplete
    // hit!
    env.reset_metrics();
    env.blobs.hide_digest(root_digest);
    // Also remove root's on-disk export so local store doesn't treat disk export as complete without
    // blob.
    let disk_root = env.output_dir.join(root_pi.store_path.to_string());
    let _ = std::fs::remove_file(&disk_root);

    let mut builder_rebuild = env.make_builder(false);
    let mut worker_rebuild = Worker::new(1);
    worker_rebuild
        .want(&lowered.root_drv_paths[0], &lowered.registry, true)
        .map_err(|e| e.to_string())?;
    let res_rebuild =
        worker_rebuild.run(&mut builder_rebuild, &mut lowered.registry).await.map_err(|e| e.to_string())?;
    if res_rebuild.outcomes[0].cached {
        return Err("incomplete blob must NEVER produce cached=true".to_string());
    }
    if env.metrics().executor_calls == 0 {
        return Err("incomplete blob must trigger reconstruction (executor_calls > 0)".to_string());
    }
    env.blobs.unhide_all();
    verify_outcomes_against_eval(&graph, &eval, &lowered, &res_rebuild, &env).await?;

    // Subcase (b): Remove leaf PathInfo (a runtime closure member of root) -> preflight must reject
    // incomplete runtime closure and rebuild leaf (and root if needed), never returning false hit!
    let leaf_drv = &lowered.task_drv_paths[&0];
    let leaf_out_sp = lowered
        .registry
        .get_output_path(&leaf_drv.to_absolute_path_with_prefix(&graph.store_dir), "out")
        .ok_or("missing leaf output path")?;
    let _ = env.pis.remove_for_test(&leaf_out_sp);
    let _ = std::fs::remove_file(env.output_dir.join(leaf_out_sp.to_string()));
    env.reset_metrics();

    let mut builder_closure = env.make_builder(false);
    let mut worker_closure = Worker::new(1);
    worker_closure
        .want(&lowered.root_drv_paths[0], &lowered.registry, true)
        .map_err(|e| e.to_string())?;
    let res_closure =
        worker_closure.run(&mut builder_closure, &mut lowered.registry).await.map_err(|e| e.to_string())?;
    if !res_closure.failed.is_empty() {
        return Err(format!("closure reconstruction failed: {:?}", res_closure.failed));
    }
    if env.metrics().executor_calls == 0 {
        return Err("missing runtime closure member must trigger reconstruction".to_string());
    }
    verify_outcomes_against_eval(&graph, &eval, &lowered, &res_closure, &env).await
}

// ── Scenario 6: Tampered, mismatched, or conflicting result candidates ──────────

pub(crate) async fn verify_scenario_6_tampered_and_conflicting(
    trust_unsigned: bool,
    reverse_candidates: bool,
) -> Result<(), String> {
    let graph = TaskGraph {
        store_dir: "/nix/store".to_string(),
        sources: Vec::new(),
        tasks: vec![TaskGraph::simple_task(
            0,
            "trust-target",
            61,
            TaskAddressingMode::InputAddressed,
        )],
        roots: vec![RootRequest {
            task: TaskId(0),
            selected_outputs: None,
            arrival_batch: 0,
        }],
        fault: GraphFault::None,
    };
    let eval = match evaluate_reference(&graph, &GraphBounds::default()) {
        ReferenceResult::Evaluated(e) => e,
        other => return Err(format!("eval: {other:?}")),
    };

    // Subcase A: Tampered PathInfo signature with no ActionResult reuse -> must NOT be trusted when
    // trust_unsigned=false.
    let env = InstrumentedEnv::new(&graph.store_dir);
    let sources = env.seed_sources(&graph).await?;
    let mut lowered = lower_task_graph(&graph, &sources).map_err(|e| e.to_string())?;
    let drv_sp = lowered.root_drv_paths[0].clone();
    let entry = lowered
        .registry
        .get_by_drv_path(&drv_sp.to_absolute_path_with_prefix(&graph.store_dir))
        .ok_or("missing registry entry")?;
    let out_sp = entry.derivation.outputs["out"].path.clone().ok_or("missing output path")?;

    // Write a tampered PathInfo into PIS with forged payload & corrupt signature.
    let mut writer = BlobService::open_write(&env.blobs).await;
    let forged_bytes = b"forged-untrusted-payload";
    writer.write_all(forged_bytes).await.map_err(|e| e.to_string())?;
    let forged_digest = writer.close().await.map_err(|e| e.to_string())?;
    let mut tampered_pi = PathInfo {
        store_path: out_sp.clone(),
        node: Node::File {
            digest: forged_digest,
            size: u64::try_from(forged_bytes.len()).unwrap_or(0),
            executable: false,
        },
        references: Vec::new(),
        nar_size: 24,
        nar_sha256: [9u8; 32],
        signatures: Vec::new(),
        deriver: None,
        ca: None,
    };
    let _ = signing::sign_pathinfo_with_store_dir(&mut tampered_pi, &test_keypair().signing_key, &graph.store_dir);
    corrupt_signature_bytes(&mut tampered_pi);
    env.pis.insert_for_test(tampered_pi);

    let mut builder = env.make_builder(trust_unsigned);
    let mut worker = Worker::new(1);
    worker.want(&drv_sp, &lowered.registry, true).map_err(|e| e.to_string())?;
    let res = worker.run(&mut builder, &mut lowered.registry).await.map_err(|e| e.to_string())?;
    if !res.failed.is_empty() {
        return Err(format!("tampered fallback failed: {:?}", res.failed));
    }
    if res.outcomes[0].cached {
        return Err("tampered PathInfo signature was falsely admitted as cached=true".to_string());
    }
    verify_outcomes_against_eval(&graph, &eval, &lowered, &res, &env).await?;

    // Subcase B: Conflicting valid signed ActionResultRecords in either candidate order -> must fail
    // closed!
    let env2 = InstrumentedEnv::new(&graph.store_dir);
    let sources2 = env2.seed_sources(&graph).await?;
    let mut lowered2 = lower_task_graph(&graph, &sources2).map_err(|e| e.to_string())?;
    let drv_sp2 = lowered2.root_drv_paths[0].clone();
    let entry2 = lowered2
        .registry
        .get_by_drv_path(&drv_sp2.to_absolute_path_with_prefix(&graph.store_dir))
        .ok_or("missing registry entry 2")?;
    let action_ref = action_ref_for_derivation(&entry2.derivation, &graph.store_dir);
    let out_sp2 = entry2.derivation.outputs["out"].path.clone().ok_or("missing output path 2")?;

    // Build two distinct valid PathInfo + SignedActionResultRecord candidates for the same action_ref.
    let mut w1 = BlobService::open_write(&env2.blobs).await;
    w1.write_all(b"candidate-one").await.map_err(|e| e.to_string())?;
    let d1 = w1.close().await.map_err(|e| e.to_string())?;
    let mut w2 = BlobService::open_write(&env2.blobs).await;
    w2.write_all(b"candidate-two-different").await.map_err(|e| e.to_string())?;
    let d2 = w2.close().await.map_err(|e| e.to_string())?;

    let pi1 = signed_test_path_info(&env2, &out_sp2, d1, 13, [1u8; 32]).await;
    let pi2 = signed_test_path_info(&env2, &out_sp2, d2, 23, [2u8; 32]).await;
    let seq_pis = Arc::new(crate::test_support::SequencePathInfoService::new());
    if reverse_candidates {
        seq_pis.push(pi2.clone());
        seq_pis.push(pi1.clone());
    } else {
        seq_pis.push(pi1.clone());
        seq_pis.push(pi2.clone());
    }

    let rec1 = crate::action_result::signed_record_for_outputs_with_action_ref(
        &entry2.derivation,
        &BTreeMap::from([("out".to_string(), pi1)]),
        crate::action_result::SignedRecordContext {
            store_dir: &graph.store_dir,
            hermeticity_mode: crate::HermeticityMode::Practical,
            keypair: &test_keypair(),
            action_ref: action_ref.clone(),
        },
    )
    .map_err(|e| e.to_string())?;
    let rec2 = crate::action_result::signed_record_for_outputs_with_action_ref(
        &entry2.derivation,
        &BTreeMap::from([("out".to_string(), pi2)]),
        crate::action_result::SignedRecordContext {
            store_dir: &graph.store_dir,
            hermeticity_mode: crate::HermeticityMode::Practical,
            keypair: &test_keypair(),
            action_ref: action_ref.clone(),
        },
    )
    .map_err(|e| e.to_string())?;

    env2.action_result_store.inject_record(&action_ref, rec1);
    env2.action_result_store.inject_record(&action_ref, rec2);
    env2.action_result_store.set_reverse_records(reverse_candidates);

    let mut handle2 = StoreHandle::from_services_with_store_dir(
        StoreBackend::Snix,
        StoreHandleServices {
            blob_service: Arc::new(env2.blobs.clone()) as Arc<dyn BlobService>,
            directory_service: Arc::new(env2.dirs.clone()) as Arc<dyn DirectoryService>,
            pathinfo_service: seq_pis as Arc<dyn PathInfoService>,
            remote_pathinfo: None,
            state_dir: env2.state_dir.clone(),
            output_dir_str: env2.output_dir.to_string_lossy().into_owned(),
            publishers: Vec::new(),
        },
        env2.store_dir.clone(),
    )
    .map_err(|e| e.to_string())?;
    let mut stores2 = ActionResultStoreSet::new(true);
    stores2.add_local(Box::new(env2.action_result_store.clone()));
    handle2.replace_action_result_stores(stores2);
    let mut builder2 = Builder::from_store_parts(
        handle2.into_builder_store_parts(),
        env2.build_service.clone(),
        test_keypair(),
        test_trusted_keys(),
        trust_unsigned,
        false,
    );

    let mut worker2 = Worker::new(1);
    worker2.want(&drv_sp2, &lowered2.registry, true).map_err(|e| e.to_string())?;
    let conflict_res = worker2.run(&mut builder2, &mut lowered2.registry).await.map_err(|e| e.to_string())?;
    if conflict_res.failed.len() != 1 || !conflict_res.outcomes.is_empty() {
        return Err(format!(
            "conflicting action results must fail closed: outcomes={}, failed={:?}",
            conflict_res.outcomes.len(),
            conflict_res.failed
        ));
    }
    if env2.metrics().executor_calls != 0 {
        return Err("conflicting IA action results should fail closed before executor dispatch".to_string());
    }
    Ok(())
}

async fn signed_test_path_info(
    env: &InstrumentedEnv,
    store_path: &StorePath<String>,
    digest: snix_castore::B3Digest,
    size: u64,
    nar_sha256: [u8; 32],
) -> PathInfo {
    let mut pi = PathInfo {
        store_path: store_path.clone(),
        node: Node::File {
            digest,
            size,
            executable: false,
        },
        references: Vec::new(),
        nar_size: size,
        nar_sha256,
        signatures: Vec::new(),
        deriver: None,
        ca: None,
    };
    let _ = signing::sign_pathinfo_with_store_dir(&mut pi, &test_keypair().signing_key, &env.store_dir);
    pi
}

// ── Scenario 7: Watch retraction & cancellation with late completion ───────────

pub(crate) async fn verify_scenario_7_watch_retraction_and_cancellation() -> Result<(), String> {
    let g1 = TaskGraph {
        store_dir: "/nix/store".to_string(),
        sources: Vec::new(),
        tasks: vec![TaskGraph::simple_task(
            0,
            "watch-gen1-slow",
            701,
            TaskAddressingMode::InputAddressed,
        )],
        roots: vec![RootRequest {
            task: TaskId(0),
            selected_outputs: None,
            arrival_batch: 0,
        }],
        fault: GraphFault::None,
    };
    let g2 = TaskGraph {
        store_dir: "/nix/store".to_string(),
        sources: Vec::new(),
        tasks: vec![TaskGraph::simple_task(
            0,
            "watch-gen2-fast",
            702,
            TaskAddressingMode::InputAddressed,
        )],
        roots: vec![RootRequest {
            task: TaskId(0),
            selected_outputs: None,
            arrival_batch: 0,
        }],
        fault: GraphFault::None,
    };
    let eval2 = match evaluate_reference(&g2, &GraphBounds::default()) {
        ReferenceResult::Evaluated(e) => e,
        other => return Err(format!("eval2: {other:?}")),
    };

    let (started_tx, mut started_rx) = mpsc::unbounded_channel();
    let (cancelled_tx, mut cancelled_rx) = mpsc::unbounded_channel();
    let (release_tx, release_rx) = watch::channel(false);

    let gate = WatchBarrierGate {
        match_substring: "watch-gen1-slow".to_string(),
        started: started_tx,
        cancellation_observed: cancelled_tx,
        release_teardown: release_rx,
    };
    let env = InstrumentedEnv::with_options(&g1.store_dir, Some(gate), false);
    let lowered1 = lower_task_graph(&g1, &BTreeMap::new()).map_err(|e| e.to_string())?;
    let lowered2 = lower_task_graph(&g2, &BTreeMap::new()).map_err(|e| e.to_string())?;

    let gen1_drv = lowered1.root_drv_paths[0].clone();
    let gen1_out_sp = lowered1
        .registry
        .get_output_path(&gen1_drv.to_absolute_path_with_prefix(&g1.store_dir), "out")
        .ok_or("missing gen1 out path")?;
    let gen2_drv = lowered2.root_drv_paths[0].clone();

    let mut shared_registry = DerivationRegistry::new(&g1.store_dir);
    for sp in lowered1.task_drv_paths.values() {
        if let Some(e) = lowered1.registry.get_by_drv_path(&sp.to_absolute_path_with_prefix(&g1.store_dir)) {
            shared_registry.insert(
                e.drv_path.clone(),
                e.hash_derivation_modulo,
                (*e.derivation).clone(),
                e.content_addressed,
                e.provenance_claims.clone(),
            );
        }
    }

    let mut builder = env.make_builder(false);
    let mut worker = Worker::new(1);
    let (updates_tx, mut updates_rx) = mpsc::channel(4);
    let (events_tx, mut events_rx) = mpsc::channel(16);

    let gen1_drv_ctrl = gen1_drv.clone();
    let gen2_drv_ctrl = gen2_drv.clone();
    let gen2_entries: Vec<_> = lowered2
        .task_drv_paths
        .values()
        .filter_map(|sp| {
            lowered2.registry.get_by_drv_path(&sp.to_absolute_path_with_prefix(&g2.store_dir)).map(|e| {
                (
                    e.drv_path.clone(),
                    e.hash_derivation_modulo,
                    (*e.derivation).clone(),
                    e.content_addressed,
                    e.dynamic_plan_outputs.clone(),
                    e.provenance_claims.clone(),
                )
            })
        })
        .collect();

    let controller = async move {
        updates_tx
            .send(WatchWorkerUpdate {
                generation: 1,
                roots: vec![EvalMessage {
                    label: "watch-gen1-slow".to_string(),
                    drv_path: gen1_drv_ctrl,
                    new_entries: Vec::new(),
                }],
            })
            .await
            .map_err(|e| e.to_string())?;

        match tokio::time::timeout(std::time::Duration::from_secs(5), events_rx.recv())
            .await
            .map_err(|_| "timeout waiting gen1 CandidateReady".to_string())?
        {
            Some(WatchWorkerEvent::CandidateReady { generation: 1, commit }) => {
                commit.send(()).map_err(|_| "commit gen1 failed".to_string())?;
            }
            other => return Err(format!("expected gen1 CandidateReady, got {other:?}")),
        }

        tokio::time::timeout(std::time::Duration::from_secs(5), started_rx.recv())
            .await
            .map_err(|_| "timeout waiting gen1 start".to_string())?
            .ok_or("started channel closed")?;

        // Send generation 2 retracting gen1 while gen1 is in flight!
        updates_tx
            .send(WatchWorkerUpdate {
                generation: 2,
                roots: vec![EvalMessage {
                    label: "watch-gen2-fast".to_string(),
                    drv_path: gen2_drv_ctrl.clone(),
                    new_entries: gen2_entries,
                }],
            })
            .await
            .map_err(|e| e.to_string())?;

        tokio::time::timeout(std::time::Duration::from_secs(5), cancelled_rx.recv())
            .await
            .map_err(|_| "timeout waiting gen1 cancellation".to_string())?
            .ok_or("cancelled channel closed")?;

        // Release gen1 build returning Ok(BuildResult) AFTER cancellation!
        release_tx.send(true).map_err(|e| e.to_string())?;

        let mut saw_gen2_settled = None;
        for _ in 0..8 {
            match tokio::time::timeout(std::time::Duration::from_secs(5), events_rx.recv())
                .await
                .map_err(|_| "timeout waiting gen2 events".to_string())?
            {
                Some(WatchWorkerEvent::PendingCancellation { .. }) => {}
                Some(WatchWorkerEvent::CandidateReady { generation: 2, commit }) => {
                    commit.send(()).map_err(|_| "commit gen2 failed".to_string())?;
                }
                Some(WatchWorkerEvent::Settled {
                    generation: 2,
                    outcomes,
                    failed,
                    ..
                }) => {
                    saw_gen2_settled = Some((outcomes, failed));
                    break;
                }
                other => return Err(format!("unexpected watch event: {other:?}")),
            }
        }
        drop(updates_tx);
        saw_gen2_settled.ok_or_else(|| "never saw Settled for generation 2".to_string())
    };

    let (watch_res, ctrl_res) =
        tokio::join!(worker.run_watch(&mut builder, &mut shared_registry, &mut updates_rx, &events_tx), controller);
    watch_res.map_err(|e| format!("run_watch error: {e}"))?;
    let (outcomes2, failed2) = ctrl_res?;
    if !failed2.is_empty() || outcomes2.len() != 1 || outcomes2[0].drv_path != gen2_drv {
        return Err(format!("gen2 settled mismatch: outcomes={outcomes2:?}, failed={failed2:?}"));
    }

    // Crucial fence invariant: retracted gen1 late Ok(BuildResult) must NEVER write PathInfo!
    if env.pis.get_cached_for_test(&gen1_out_sp).is_some() {
        return Err("retracted gen1 late completion leaked PathInfo into store".to_string());
    }

    let synth_res = WorkerResult {
        outcomes: outcomes2,
        all_outcomes: Vec::new(),
        failed: failed2,
        native_dynamic_plans: Vec::new(),
        plan_output_bindings: Vec::new(),
        priority_decisions: Vec::new(),
        waiter_notifications: 0,
    };
    verify_outcomes_against_eval(&g2, &eval2, &lowered2, &synth_res, &env).await
}

// ── Scenario 8: Invalid cycles or resource bounds -> deterministic rejection ────

pub(crate) async fn verify_scenario_8_cycles_and_bounds() -> Result<(), String> {
    let bounds = GraphBounds::default();

    // 1. Cycle fault
    let cycle_graph = TaskGraph {
        store_dir: "/nix/store".to_string(),
        sources: Vec::new(),
        tasks: vec![TaskGraph::simple_task(
            0,
            "cycle-node",
            1,
            TaskAddressingMode::InputAddressed,
        )],
        roots: vec![RootRequest {
            task: TaskId(0),
            selected_outputs: None,
            arrival_batch: 0,
        }],
        fault: GraphFault::DependencyCycle {
            from: TaskId(0),
            to: TaskId(0),
        },
    };
    match evaluate_reference(&cycle_graph, &bounds) {
        ReferenceResult::Rejected(ReferenceRejection::CycleDetected { .. }) => {}
        other => return Err(format!("expected CycleDetected, got {other:?}")),
    }
    let cycle_err = lower_task_graph(&cycle_graph, &BTreeMap::new())
        .err()
        .ok_or("expected CircularDependency from crunch_glue::convert")?;
    if !matches!(cycle_err, crunch_glue::Error::CircularDependency(_)) {
        return Err(format!("expected CircularDependency, got {cycle_err:?}"));
    }

    // 2. Unknown output selection fault
    let bad_out_graph = TaskGraph {
        store_dir: "/nix/store".to_string(),
        sources: Vec::new(),
        tasks: vec![TaskGraph::simple_task(
            0,
            "valid-node",
            2,
            TaskAddressingMode::InputAddressed,
        )],
        roots: vec![RootRequest {
            task: TaskId(0),
            selected_outputs: None,
            arrival_batch: 0,
        }],
        fault: GraphFault::UnknownSelectedOutput {
            task: TaskId(0),
            output: "missing-out".to_string(),
        },
    };
    match evaluate_reference(&bad_out_graph, &bounds) {
        ReferenceResult::Rejected(ReferenceRejection::UnknownOutputSelection { .. }) => {}
        other => return Err(format!("expected UnknownOutputSelection, got {other:?}")),
    }
    let out_err = lower_task_graph(&bad_out_graph, &BTreeMap::new()).err().ok_or("expected InvalidOutputSelection")?;
    if !matches!(out_err, crunch_glue::Error::InvalidOutputSelection { .. }) {
        return Err(format!("expected InvalidOutputSelection, got {out_err:?}"));
    }

    // 3. Step-budget exhaustion classification check (never passes!)
    let exhaust_bounds = GraphBounds {
        max_eval_steps: 1,
        ..GraphBounds::default()
    };
    let chain = TaskGraph {
        store_dir: "/nix/store".to_string(),
        sources: Vec::new(),
        tasks: vec![
            TaskGraph::simple_task(0, "step-0", 1, TaskAddressingMode::InputAddressed),
            {
                let mut t = TaskGraph::simple_task(1, "step-1", 2, TaskAddressingMode::InputAddressed);
                t.deps.push(TaskDep::AllOutputs { target: TaskId(0) });
                t
            },
        ],
        roots: vec![RootRequest {
            task: TaskId(1),
            selected_outputs: None,
            arrival_batch: 0,
        }],
        fault: GraphFault::None,
    };
    if !matches!(evaluate_reference(&chain, &exhaust_bounds), ReferenceResult::Exhausted(_)) {
        return Err("expected Exhausted when max_eval_steps=1".to_string());
    }

    // 4. Worker resource bound rejections (`max_jobs == 0` and `max_jobs > 64`)
    if Worker::with_scheduling_policy(0, SchedulingPolicy::default()).is_ok() {
        return Err("expected Worker::with_scheduling_policy(0) to reject".to_string());
    }
    if Worker::with_scheduling_policy(65, SchedulingPolicy::default()).is_ok() {
        return Err("expected Worker::with_scheduling_policy(65) to reject".to_string());
    }
    Ok(())
}

// ── 3 Explicit Mutation Checks (Acceptance Criterion 4) ─────────────────────────

pub(crate) async fn run_mutation_check_a_coarse_recipe_key() -> Result<(), String> {
    fn coarse_recipe_converter(
        drv: &CrunchDerivation,
        cache: &mut ConversionCache,
    ) -> Result<(StorePath<String>, Derivation), crunch_glue::Error> {
        // Deliberately buggy #38-style memo key: hashes only (name, builder, system).
        let coarse_key = *blake3::hash(format!("{}:{}:{}", drv.name, drv.builder, drv.system).as_bytes()).as_bytes();
        if let Some(existing) = cache.get_completed_recipe(&coarse_key) {
            return Ok((existing.drv_path, existing.derivation));
        }
        let (sp, nix_drv) = crunch_glue::convert(drv, cache)?;
        cache.insert_completed_recipe(coarse_key, sp.clone(), nix_drv.clone(), drv.dynamic_plan_outputs.clone());
        Ok((sp, nix_drv))
    }

    // Production converter must pass in both orders.
    verify_scenario_1_with_converter(false, crunch_glue::convert).await?;
    verify_scenario_1_with_converter(true, crunch_glue::convert).await?;

    // Mutated coarse-key converter MUST be caught by the exact same differential check.
    let err = verify_scenario_1_with_converter(false, coarse_recipe_converter)
        .await
        .err()
        .ok_or("mutation check (a) FAILED: differential oracle did not catch coarse recipe key")?;
    if !err.contains("collapsed") {
        return Err(format!("mutation check (a) failed with unexpected error: {err}"));
    }
    Ok(())
}

pub(crate) async fn run_mutation_check_b_dropped_dependency_wait() -> Result<(), String> {
    let mut base = TaskGraph::simple_task(0, "wait-dep", 801, TaskAddressingMode::InputAddressed);
    base.output_rules.get_mut("out").expect("out").salt = 801;
    let mut consumer = TaskGraph::simple_task(1, "wait-consumer", 802, TaskAddressingMode::InputAddressed);
    consumer.deps.push(TaskDep::AllOutputs { target: TaskId(0) });

    let graph = TaskGraph {
        store_dir: "/nix/store".to_string(),
        sources: Vec::new(),
        tasks: vec![base, consumer],
        roots: vec![RootRequest {
            task: TaskId(1),
            selected_outputs: None,
            arrival_batch: 0,
        }],
        fault: GraphFault::None,
    };

    // Production Worker::run passes.
    verify_static_graph_differential(&graph, &GraphBounds::default(), false).await?;

    // Mutated wait-dropping driver MUST be caught!
    let env = InstrumentedEnv::new(&graph.store_dir);
    let sources = env.seed_sources(&graph).await?;
    let mut lowered = lower_task_graph(&graph, &sources).map_err(|e| e.to_string())?;
    let mut builder = env.make_builder(false);
    let mutated_res = crate::worker::tests::run_with_dropped_wait_mutation(
        &lowered.root_drv_paths[0],
        &mut builder,
        &mut lowered.registry,
    )
    .await
    .map_err(|e| e.to_string())?;

    if mutated_res.failed.is_empty() && mutated_res.outcomes.len() == 1 {
        return Err("mutation check (b) FAILED: dropped dependency wait succeeded without detection".to_string());
    }
    Ok(())
}

pub(crate) async fn run_mutation_check_c_skipped_admission_condition() -> Result<(), String> {
    // 1. Production trust_unsigned=false rejects tampered PathInfo signature (scenario 6 passes).
    verify_scenario_6_tampered_and_conflicting(false, false).await?;

    // Mutated trust_unsigned=true skips PathInfo signature verification -> differential check MUST
    // catch it!
    let err = verify_scenario_6_tampered_and_conflicting(true, false)
        .await
        .err()
        .ok_or("mutation check (c.1) FAILED: trust_unsigned=true was not caught")?;
    if !err.contains("tampered PathInfo signature was falsely admitted") {
        return Err(format!("mutation check (c.1) unexpected error: {err}"));
    }

    // 2. Also prove skipping plan_strong_reuse conflict check on two valid conflicting records is
    //    caught.
    let keypair = test_keypair();
    let trusted_keys = test_trusted_keys();
    let store_dir = "/nix/store";
    let graph = TaskGraph {
        store_dir: store_dir.to_string(),
        sources: Vec::new(),
        tasks: vec![TaskGraph::simple_task(
            0,
            "conflict-mut",
            901,
            TaskAddressingMode::InputAddressed,
        )],
        roots: vec![RootRequest {
            task: TaskId(0),
            selected_outputs: None,
            arrival_batch: 0,
        }],
        fault: GraphFault::None,
    };
    let lowered = lower_task_graph(&graph, &BTreeMap::new()).map_err(|e| e.to_string())?;
    let entry = lowered
        .registry
        .get_by_drv_path(&lowered.root_drv_paths[0].to_absolute_path_with_prefix(store_dir))
        .ok_or("missing entry")?;
    let out_sp = entry.derivation.outputs["out"].path.clone().ok_or("missing out path")?;
    let action_ref = action_ref_for_derivation(&entry.derivation, store_dir);
    let policy_refs = crate::action_result::policy_refs_for_derivation(
        &entry.derivation,
        store_dir,
        crate::HermeticityMode::Practical,
    )
    .map_err(|e| e.to_string())?;
    let policy = crate::action_result::trust_policy_for_action(&policy_refs, &trusted_keys);

    let make_cand = |bytes: &[u8], nar: [u8; 32]| -> Result<DiscoveredActionResultCandidate, String> {
        let mut pi = PathInfo {
            store_path: out_sp.clone(),
            node: Node::File {
                digest: blake3::hash(bytes).as_bytes().into(),
                size: u64::try_from(bytes.len()).unwrap_or(0),
                executable: false,
            },
            references: Vec::new(),
            nar_size: u64::try_from(bytes.len()).unwrap_or(0),
            nar_sha256: nar,
            signatures: Vec::new(),
            deriver: None,
            ca: None,
        };
        let _ = signing::sign_pathinfo_with_store_dir(&mut pi, &keypair.signing_key, store_dir);
        let outputs_map = BTreeMap::from([("out".to_string(), pi)]);
        let signed = crate::action_result::signed_record_for_outputs_with_action_ref(
            &entry.derivation,
            &outputs_map,
            crate::action_result::SignedRecordContext {
                store_dir,
                hermeticity_mode: crate::HermeticityMode::Practical,
                keypair: &keypair,
                action_ref: action_ref.clone(),
            },
        )
        .map_err(|e| e.to_string())?;
        let facts = crate::action_result::candidate_admission_facts_with_action_ref(
            "local-action-results".to_string(),
            "local".to_string(),
            &signed,
            &entry.derivation,
            Some(&outputs_map),
            store_dir,
            crate::HermeticityMode::Practical,
            &trusted_keys,
            &action_ref,
        );
        Ok(DiscoveredActionResultCandidate {
            signed_record: signed,
            facts,
        })
    };

    let c1 = make_cand(b"cand-1", [1u8; 32])?;
    let c2 = make_cand(b"cand-2-diff", [2u8; 32])?;

    for (label, candidates) in [
        ("c1-then-c2", vec![c1.clone(), c2.clone()]),
        ("c2-then-c1", vec![c2.clone(), c1.clone()]),
    ] {
        let req = StrongReuseRequest {
            action_ref: action_ref.clone(),
            output_names: vec!["out".to_string()],
            policy: policy.clone(),
        };
        // Production plan_strong_reuse detects conflict in both candidate orders.
        let plan = plan_strong_reuse(req, candidates).map_err(|e| e.code().to_string())?;
        if plan.conflict_class.as_deref() != Some(crunch_action_result_core::CONFLICTING_ACTION_RESULTS) {
            return Err(format!("production plan_strong_reuse failed to detect conflict ({label})"));
        }
        if plan.selected_result_ref.is_some() {
            return Err(format!(
                "mutation check (c.2) ({label}) selected a conflicting candidate: {:?}",
                plan.selected_result_ref
            ));
        }
        if plan.candidate_decisions.len() != 2 || !plan.candidate_decisions.iter().all(|d| d.admitted) {
            return Err(format!(
                "expected both candidates admitted before conflict rejection ({label}): {:?}",
                plan.candidate_decisions
            ));
        }
    }
    Ok(())
}
