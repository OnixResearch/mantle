//! Pool executor factory: per-endpoint dispatches plus the placement selector.
//!
//! The pipeline hands over its opened store; this module clones the owned
//! transfer capability into each endpoint executor and builds the selector from
//! the admitted pool and the caller's scheduling preference order.

use std::path::Path;
use std::path::PathBuf;
use std::sync::Arc;

use crunch_build::distributed::PoolDispatch;
use crunch_build::distributed::PoolEndpointService;
use crunch_build::distributed::PoolGoalExecutor;
use crunch_build::distributed::PoolGoalSelector;
use crunch_build::distributed::PoolSelectionError;
use crunch_build::distributed::PreparedPoolGoal;
use crunch_build::distributed::RemoteFailureDebugPolicy;
use crunch_build::distributed::RemoteResourceRequirements;
use crunch_build::distributed::RemoteTransferPolicy;
use crunch_build::distributed::RemoteWorkerResourceInventory;
use crunch_pipeline::PoolExecutorFactory;
use crunch_remote_core::pool::AdmittedPool;
use crunch_remote_core::pool::DeclaredCapabilities;
use crunch_remote_core::pool::GoalPlacement;
use crunch_remote_core::pool::GoalRequirement;
use crunch_remote_core::pool::WorkerFacts;
use crunch_remote_core::pool::plan_goal_placement;
use crunch_remote_core::resource::Capacity;
use crunch_remote_core::resource::NamedResourceFit;
use crunch_remote_core::resource::PlacementPreference;

use crate::remote_build::RemoteClientBuildOptions;
use crate::remote_build::RemoteCoordinatorState;
use crate::remote_build::pool_goal_executor::PoolEndpointDispatch;
use crate::remote_build::pool_goal_executor::RemotePoolGoalExecutor;

/// One endpoint's plan before the store capability is available.
#[derive(Clone)]
pub struct PoolEndpointPlan {
    pub endpoint_name: String,
    pub options: RemoteClientBuildOptions,
    pub worker_generation: u64,
    pub worker_concurrency: u32,
    pub resource_inventory: Option<RemoteWorkerResourceInventory>,
    pub failure_debug: RemoteFailureDebugPolicy,
    pub transfer_policy: RemoteTransferPolicy,
    pub state_dir: PathBuf,
    pub capabilities: DeclaredCapabilities,
}

/// Factory that builds a `PoolDispatch` over the pipeline's store.
pub struct RemotePoolExecutorFactory {
    admitted: AdmittedPool,
    endpoints: Vec<PoolEndpointPlan>,
    preferences: Vec<PlacementPreference>,
    local_fallback: bool,
    resource_requirements: Option<RemoteResourceRequirements>,
}

impl std::fmt::Debug for RemotePoolExecutorFactory {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("RemotePoolExecutorFactory")
            .field("endpoints", &self.admitted.endpoint_names())
            .finish_non_exhaustive()
    }
}

impl RemotePoolExecutorFactory {
    /// Admit pre-dispatch local fallback for this factory.
    #[must_use]
    pub fn with_local_fallback(mut self, allowed: bool) -> Self {
        self.local_fallback = allowed;
        self
    }

    /// Attach the operator-declared quantified requirements for every goal.
    #[must_use]
    pub fn with_resource_requirements(mut self, requirements: Option<RemoteResourceRequirements>) -> Self {
        self.resource_requirements = requirements;
        self
    }

    pub fn new(
        admitted: AdmittedPool,
        endpoints: Vec<PoolEndpointPlan>,
        preferences: Vec<PlacementPreference>,
    ) -> Result<Self, String> {
        if endpoints.len() != admitted.endpoint_names().len() {
            return Err("remote-pool-factory-endpoint-count-mismatch".to_string());
        }
        for endpoint in &endpoints {
            if !admitted.endpoint_names().iter().any(|name| name == &endpoint.endpoint_name) {
                return Err("remote-pool-factory-endpoint-not-admitted".to_string());
            }
        }
        debug_assert!(!endpoints.is_empty());
        debug_assert!(preferences.len() <= 3);
        Ok(Self {
            admitted,
            endpoints,
            preferences,
            local_fallback: false,
            resource_requirements: None,
        })
    }
}

impl PoolExecutorFactory for RemotePoolExecutorFactory {
    fn create(&self, store: &crunch_store::StoreHandle) -> Result<Arc<PoolDispatch>, String> {
        let transfer = store.owned_transfer_objects();
        let observations = Arc::new(std::sync::Mutex::new(Vec::new()));
        let mut services = Vec::with_capacity(self.endpoints.len());
        let mut workers = Vec::with_capacity(self.endpoints.len());
        for endpoint in &self.endpoints {
            let dispatch = PoolEndpointDispatch {
                options: endpoint.options.clone(),
                worker_generation: endpoint.worker_generation,
                worker_concurrency: endpoint.worker_concurrency,
                resource_inventory: endpoint.resource_inventory.clone(),
                failure_debug: endpoint.failure_debug.clone(),
                transfer_policy: endpoint.transfer_policy,
                state_dir: endpoint.state_dir.clone(),
                transfer: Some(transfer.clone()),
                observations: Some(Arc::clone(&observations)),
            };
            let executor: Arc<dyn PoolGoalExecutor> = Arc::new(RemotePoolGoalExecutor::new(dispatch));
            services.push(PoolEndpointService::new(endpoint.endpoint_name.clone(), executor));
            workers.push(WorkerFacts {
                endpoint_name: endpoint.endpoint_name.clone(),
                capabilities: endpoint.capabilities.clone(),
                declared_accelerator_classes: Vec::new(),
                worker_generation: endpoint.worker_generation,
                remaining: Capacity {
                    cpu_units: 0,
                    memory_bytes: 0,
                    scratch_bytes: 0,
                },
                named: NamedResourceFit {
                    is_empty: true,
                    is_constrained: false,
                },
                resource_fit: 0,
                content_locality: 0,
                transfer_cost: 0,
            });
        }
        let selector: Arc<dyn PoolGoalSelector> = Arc::new(PoolPolicySelector {
            admitted: self.admitted.clone(),
            workers,
            preferences: self.preferences.clone(),
            inventories: self.endpoints.iter().map(|endpoint| endpoint.resource_inventory.clone()).collect(),
            state_dirs: self.endpoints.iter().map(|endpoint| endpoint.state_dir.clone()).collect(),
        });
        PoolDispatch::new(services, selector)
            .map(|dispatch| dispatch.with_session_limit(self.admitted.session().max_concurrency))
            .map(|dispatch| dispatch.with_local_fallback(self.local_fallback))
            .map(|dispatch| dispatch.with_resource_requirements(self.resource_requirements.clone()))
            .map(|dispatch| dispatch.with_observation_sink(Arc::clone(&observations)))
            .map(Arc::new)
            .map_err(|error| error.to_string())
    }
}

/// Selects an endpoint from resolved goal facts through the pool placement core.
struct PoolPolicySelector {
    admitted: AdmittedPool,
    workers: Vec<WorkerFacts>,
    preferences: Vec<PlacementPreference>,
    inventories: Vec<Option<RemoteWorkerResourceInventory>>,
    state_dirs: Vec<PathBuf>,
}

impl PoolPolicySelector {
    /// Rebuild worker facts with the capacity currently remaining after live
    /// leases. An endpoint without a declared inventory keeps unknown (zero)
    /// capacity, so a quantified goal cannot be placed by inference.
    fn worker_facts_with_current_capacity(&self) -> Result<Vec<WorkerFacts>, PoolSelectionError> {
        debug_assert_eq!(self.workers.len(), self.inventories.len());
        debug_assert_eq!(self.workers.len(), self.state_dirs.len());
        let mut facts = self.workers.clone();
        let mut states: std::collections::BTreeMap<&Path, RemoteCoordinatorState> = std::collections::BTreeMap::new();
        for (index, inventory) in self.inventories.iter().enumerate() {
            let Some(inventory) = inventory else {
                continue;
            };
            let state_dir = self.state_dirs[index].as_path();
            if !states.contains_key(state_dir) {
                let state = crate::remote_build::load_coordinator_state(state_dir)
                    .map_err(|_| PoolSelectionError::CapacitySnapshotInvalid)?;
                states.insert(state_dir, state);
            }
            let state = states.get(state_dir).ok_or(PoolSelectionError::CapacitySnapshotInvalid)?;
            let leases = state.resource_leases.values().cloned().collect::<Vec<_>>();
            let remaining = crunch_build::distributed::remote_resource_remaining_capacity(
                &facts[index].endpoint_name,
                inventory,
                &leases,
            )
            .map_err(|_| PoolSelectionError::CapacitySnapshotInvalid)?;
            facts[index].remaining = Capacity {
                cpu_units: remaining.cpu_units,
                memory_bytes: remaining.memory_bytes,
                scratch_bytes: remaining.scratch_bytes,
            };
        }
        Ok(facts)
    }
}

impl PoolGoalSelector for PoolPolicySelector {
    fn select(&self, goal: &PreparedPoolGoal, endpoint_ids: &[String]) -> Result<usize, PoolSelectionError> {
        let requires_named_resources = goal.resource_requirements.as_ref().is_some_and(|requirements| {
            !requirements.quantities.accelerators.is_empty() || !requirements.quantities.named_tokens.is_empty()
        });
        let requirement = GoalRequirement {
            system: goal.system.clone(),
            sandbox_mode: goal.required_sandbox_mode.clone(),
            network_mode: goal.required_network_mode.clone(),
            required_features: goal.required_features.clone(),
            required_accelerator_classes: goal.semantic_accelerator_classes.clone(),
            requested: goal.requested,
            requires_named_resources,
        };
        let workers = self.worker_facts_with_current_capacity()?;
        match plan_goal_placement(&self.admitted, &requirement, &workers, self.preferences.iter().copied()) {
            GoalPlacement::Selected { endpoint_name, .. } => {
                endpoint_ids.iter().position(|id| id == &endpoint_name).ok_or(PoolSelectionError::NoFit)
            }
            GoalPlacement::Unplaceable { .. } => Err(PoolSelectionError::NoFit),
        }
    }
}

/// Inputs for constructing the factory from operator configuration.
pub struct PoolFactoryInput<'a> {
    pub farm_path: &'a Path,
    pub pool_id: &'a str,
    pub bindings: &'a crate::remote_build::endpoint_config::RemoteEndpointBindingConfig,
    pub ticket_fds: &'a [String],
    pub store_prefix: &'a str,
    pub state_dir: &'a Path,
    pub now_unix_s: u64,
    pub build_time_limit_secs: u64,
    pub preferences: Vec<PlacementPreference>,
    pub resource_requirements: Option<RemoteResourceRequirements>,
}

/// Validate a pool selection without credentials, tickets, or dispatch.
///
/// Plan mode uses this: the evaluated pool must exist, every selected endpoint
/// must resolve its current binding, and each transport program must match its
/// declared BLAKE3 identity. Ticket descriptors and executor construction are
/// deliberately skipped.
pub fn plan_pool_selection(
    farm_path: &Path,
    pool_id: &str,
    bindings: &crate::remote_build::endpoint_config::RemoteEndpointBindingConfig,
) -> Result<(), crate::errors::RunError> {
    let farm = crate::remote_farm_config::load_remote_build_farm_config(farm_path)
        .map_err(|reason| crate::errors::RunError::Internal(format!("remote-pool:config-invalid:{reason}")))?;
    let pool = farm
        .pools
        .iter()
        .find(|pool| pool.pool_id == pool_id)
        .ok_or_else(|| crate::errors::RunError::Internal("remote-pool:pool-unknown".to_string()))?;
    crate::remote_build::pool_config::project_pool_selection(&farm, pool_id, bindings)?;
    crate::remote_build::pool_config::verify_transport_programs(pool)?;
    Ok(())
}

/// Project the configured pool, admit it against current bindings and caller
/// descriptors, and build the per-endpoint plans the factory needs.
pub fn plan_pool_factory(input: PoolFactoryInput<'_>) -> Result<RemotePoolExecutorFactory, crate::errors::RunError> {
    let farm = crate::remote_farm_config::load_remote_build_farm_config(input.farm_path)
        .map_err(|reason| crate::errors::RunError::Internal(format!("remote-pool:config-invalid:{reason}")))?;
    let pool = farm
        .pools
        .iter()
        .find(|pool| pool.pool_id == input.pool_id)
        .ok_or_else(|| crate::errors::RunError::Internal("remote-pool:pool-unknown".to_string()))?;
    let selection = crate::remote_build::pool_config::project_pool_selection(&farm, input.pool_id, input.bindings)?;
    let parsed = crate::remote_build::pool_config::parse_pool_ticket_fds(input.ticket_fds)?;
    crate::remote_build::pool_config::verify_owned_descriptors(&parsed)?;
    let mut fds = std::collections::BTreeMap::new();
    for entry in &parsed {
        fds.insert(entry.endpoint_name.clone(), entry.fd);
    }
    let credentials = pool
        .endpoints
        .iter()
        .map(|endpoint| crunch_remote_core::pool::CallerCredential {
            endpoint_name: endpoint.endpoint_id.clone(),
            reference: endpoint.credential_reference.clone(),
        })
        .collect::<Vec<_>>();
    let records = crate::remote_build::endpoint::EndpointBindingStore::open(&input.bindings.endpoint_state_directory)?
        .records()?;
    let table = crunch_remote_core::endpoint::EndpointBindingTable::from_records(records)
        .map_err(|_| crate::errors::RunError::Internal("remote-pool:binding-table-invalid".to_string()))?;
    let admitted = crunch_remote_core::pool::admits_pool(&selection, &table, &credentials, input.now_unix_s)
        .map_err(|error| crate::errors::RunError::Internal(format!("remote-pool:{error:?}")))?;
    crate::remote_build::pool_config::verify_transport_programs(pool)?;
    let mut plans = Vec::with_capacity(pool.endpoints.len());
    for endpoint in &pool.endpoints {
        let fd = *fds
            .get(&endpoint.endpoint_id)
            .ok_or_else(|| crate::errors::RunError::Internal("remote-pool:pool-credential-missing".to_string()))?;
        let ticket = crate::remote_build::read_remote_ticket_credential_from_owned_fd(fd)
            .map_err(crate::errors::RunError::Internal)?;
        let binding = input.bindings.build_endpoint(&endpoint.endpoint_id).ok_or_else(|| {
            crate::errors::RunError::Internal("remote-pool:pool-endpoint-binding-missing".to_string())
        })?;
        let options = RemoteClientBuildOptions {
            store_prefix: input.store_prefix.to_string(),
            endpoint_binding: Some(binding.endpoint_binding.clone()),
            ticket,
            builder: crate::remote_build::RemoteStdioBuilderCommand {
                endpoint_id: endpoint.endpoint_id.clone(),
                program: PathBuf::from(&endpoint.transport_program),
                args: endpoint.transport_args.clone(),
            },
            trusted_output_keys: endpoint.trusted_output_keys.iter().map(|key| key.public_key_name.clone()).collect(),
            now_unix_s: input.now_unix_s,
            build_time_limit_secs: input.build_time_limit_secs,
            client_endpoint: None,
            transfer_capabilities: crate::remote_build::RemoteTransferCapabilities::full_only().with_streaming(),
        };
        plans.push(PoolEndpointPlan {
            endpoint_name: endpoint.endpoint_id.clone(),
            options,
            worker_generation: endpoint.profile.worker_generation,
            worker_concurrency: endpoint.profile.max_concurrency,
            resource_inventory: endpoint.profile.resource_inventory.clone(),
            failure_debug: RemoteFailureDebugPolicy::default(),
            transfer_policy: RemoteTransferPolicy::default(),
            state_dir: input.state_dir.to_path_buf(),
            capabilities: DeclaredCapabilities {
                system: endpoint.profile.system.clone(),
                sandbox_mode: sandbox_mode_name(endpoint.profile.sandbox_mode).to_string(),
                network_mode: network_mode_name(endpoint.profile.network_mode).to_string(),
                features: endpoint.profile.features.clone(),
            },
        });
    }
    RemotePoolExecutorFactory::new(admitted, plans, input.preferences)
        .map(|factory| {
            factory.with_local_fallback(matches!(
                pool.fallback_policy,
                crate::remote_farm_config::RemoteFallbackPolicy::Always
            ))
        })
        .map(|factory| factory.with_resource_requirements(input.resource_requirements.clone()))
        .map_err(crate::errors::RunError::Internal)
}

fn sandbox_mode_name(mode: crate::remote_farm_config::RemoteSandboxMode) -> &'static str {
    match mode {
        crate::remote_farm_config::RemoteSandboxMode::Practical => "practical",
        crate::remote_farm_config::RemoteSandboxMode::Strict => "strict",
    }
}

fn network_mode_name(mode: crate::remote_farm_config::RemoteNetworkMode) -> &'static str {
    match mode {
        crate::remote_farm_config::RemoteNetworkMode::None => "none",
        crate::remote_farm_config::RemoteNetworkMode::Local => "local",
        crate::remote_farm_config::RemoteNetworkMode::Full => "full",
    }
}

#[cfg(test)]
mod tests {
    use crunch_remote_core::endpoint::EndpointBinding;
    use crunch_remote_core::endpoint::EndpointBindingSpec;
    use crunch_remote_core::endpoint::EndpointKind;
    use crunch_remote_core::pool::PoolEndpoint;
    use crunch_remote_core::pool::PoolSelection;
    use crunch_remote_core::pool::PoolSessionPolicy;

    use super::*;

    fn admitted(names: &[&str]) -> AdmittedPool {
        let endpoints = names
            .iter()
            .map(|name| PoolEndpoint {
                presented_binding: EndpointBinding {
                    spec: EndpointBindingSpec {
                        kind: EndpointKind::BuilderTicket,
                        name: name.to_string(),
                        scope: "native-build".to_string(),
                        trust_requirement: "signed".to_string(),
                        store_prefix: "/mantle/store".to_string(),
                        expires_unix_s: 10_000,
                    },
                    generation: 1,
                },
                transport: "stdio".to_string(),
                capabilities: DeclaredCapabilities {
                    system: "x86_64-linux".to_string(),
                    sandbox_mode: "practical".to_string(),
                    network_mode: "none".to_string(),
                    features: Vec::new(),
                },
                trusted_output_keys: vec![crunch_remote_core::pool::TrustedOutputKey {
                    name: format!("{name}-key"),
                    material_base64: format!("material-{name}"),
                }],
                credential_reference: format!("bearer-{name}"),
                declares_resource_inventory: false,
            })
            .collect();
        let table = crunch_remote_core::endpoint::EndpointBindingTable::from_records(
            names
                .iter()
                .map(|name| crunch_remote_core::endpoint::EndpointBindingRecord {
                    binding: EndpointBinding {
                        spec: EndpointBindingSpec {
                            kind: EndpointKind::BuilderTicket,
                            name: name.to_string(),
                            scope: "native-build".to_string(),
                            trust_requirement: "signed".to_string(),
                            store_prefix: "/mantle/store".to_string(),
                            expires_unix_s: 10_000,
                        },
                        generation: 1,
                    },
                    revoked: false,
                })
                .collect(),
        )
        .expect("fixture table");
        crunch_remote_core::pool::plans_pool(
            &PoolSelection {
                pool_id: "pool".to_string(),
                endpoints,
                session: PoolSessionPolicy {
                    max_concurrency: 1,
                    job_count_max: 10,
                    session_bytes_max: 1024,
                },
            },
            &table,
            1_000,
        )
        .expect("fixture pool admits")
    }

    fn plan(name: &str, system: &str) -> PoolEndpointPlan {
        PoolEndpointPlan {
            endpoint_name: name.to_string(),
            options: RemoteClientBuildOptions {
                store_prefix: "/mantle/store".to_string(),
                endpoint_binding: None,
                ticket: crate::remote_build::RemoteTicketCredential {
                    ticket_id: "ticket".to_string(),
                    secret: "secret".to_string(),
                },
                builder: crate::remote_build::RemoteStdioBuilderCommand {
                    endpoint_id: name.to_string(),
                    program: PathBuf::from("/bin/mantle"),
                    args: Vec::new(),
                },
                trusted_output_keys: vec![format!("{name}-key")],
                now_unix_s: 1,
                build_time_limit_secs: 60,
                client_endpoint: None,
                transfer_capabilities: crate::remote_build::RemoteTransferCapabilities::full_only(),
            },
            worker_generation: 1,
            worker_concurrency: 1,
            resource_inventory: None,
            failure_debug: RemoteFailureDebugPolicy::default(),
            transfer_policy: RemoteTransferPolicy::default(),
            state_dir: PathBuf::from("/tmp/remote-pool-factory-test"),
            capabilities: DeclaredCapabilities {
                system: system.to_string(),
                sandbox_mode: "practical".to_string(),
                network_mode: "none".to_string(),
                features: Vec::new(),
            },
        }
    }

    fn goal(system: &str) -> PreparedPoolGoal {
        PreparedPoolGoal::new(crunch_build::distributed::PreparedPoolGoalFacts {
            request: snix_build::buildservice::BuildRequest::default(),
            action_ref: "action:fixture".to_string(),
            drv_path: "/mantle/store/00000000000000000000000000000000-fixture.drv".to_string(),
            derivation_json: "{}".to_string(),
            closure: Vec::new(),
            system: system.to_string(),
            required_features: Vec::new(),
            required_sandbox_mode: "practical".to_string(),
            required_network_mode: "none".to_string(),
            semantic_accelerator_classes: Vec::new(),
            requested: Capacity {
                cpu_units: 0,
                memory_bytes: 0,
                scratch_bytes: 0,
            },
            resource_requirements: None,
        })
        .expect("fixture goal")
    }

    fn goal_with_capacity(system: &str, cpu_units: u32) -> PreparedPoolGoal {
        PreparedPoolGoal::new(crunch_build::distributed::PreparedPoolGoalFacts {
            request: snix_build::buildservice::BuildRequest::default(),
            action_ref: "action:capacity-fixture".to_string(),
            drv_path: "/mantle/store/00000000000000000000000000000000-capacity.drv".to_string(),
            derivation_json: "{}".to_string(),
            closure: Vec::new(),
            system: system.to_string(),
            required_features: Vec::new(),
            required_sandbox_mode: "practical".to_string(),
            required_network_mode: "none".to_string(),
            semantic_accelerator_classes: Vec::new(),
            requested: Capacity {
                cpu_units,
                memory_bytes: 0,
                scratch_bytes: 0,
            },
            resource_requirements: None,
        })
        .expect("capacity fixture goal")
    }

    fn capacity_worker() -> WorkerFacts {
        WorkerFacts {
            endpoint_name: "worker-a".to_string(),
            capabilities: DeclaredCapabilities {
                system: "x86_64-linux".to_string(),
                sandbox_mode: "practical".to_string(),
                network_mode: "none".to_string(),
                features: Vec::new(),
            },
            declared_accelerator_classes: Vec::new(),
            worker_generation: 1,
            remaining: Capacity {
                cpu_units: 0,
                memory_bytes: 0,
                scratch_bytes: 0,
            },
            named: NamedResourceFit {
                is_empty: true,
                is_constrained: false,
            },
            resource_fit: 0,
            content_locality: 0,
            transfer_cost: 0,
        }
    }

    #[test]
    fn selector_uses_declared_goal_capacity_against_declared_inventory() {
        let admitted = admitted(&["worker-a"]);
        let root = tempfile::tempdir().expect("selector capacity tempdir");
        let inventory = RemoteWorkerResourceInventory {
            total: crunch_build::distributed::RemoteResourceVector {
                cpu_units: 1,
                memory_bytes: 1_024,
                scratch_bytes: 1_024,
                accelerators: Vec::new(),
                named_tokens: Vec::new(),
            },
        };
        let selector = PoolPolicySelector {
            admitted,
            workers: vec![capacity_worker()],
            preferences: Vec::new(),
            inventories: vec![Some(inventory)],
            state_dirs: vec![root.path().to_path_buf()],
        };
        let endpoints = vec!["worker-a".to_string()];
        assert_eq!(selector.select(&goal_with_capacity("x86_64-linux", 1), &endpoints), Ok(0));
        assert_eq!(selector.select(&goal_with_capacity("x86_64-linux", 2), &endpoints), Err(PoolSelectionError::NoFit));
        let unknown = PoolPolicySelector {
            inventories: vec![None],
            ..selector
        };
        assert_eq!(
            unknown.select(&goal_with_capacity("x86_64-linux", 1), &endpoints),
            Err(PoolSelectionError::NoFit),
            "an endpoint without a declared inventory cannot prove quantified capacity"
        );
    }

    #[test]
    fn selector_uses_pool_placement_and_rejects_mismatched_capabilities() {
        let admitted = admitted(&["worker-a", "worker-b"]);
        let _factory = RemotePoolExecutorFactory::new(
            admitted.clone(),
            vec![plan("worker-a", "x86_64-linux"), plan("worker-b", "aarch64-linux")],
            vec![PlacementPreference::ResourceFit],
        )
        .expect("factory");
        let selector = PoolPolicySelector {
            admitted: admitted.clone(),
            workers: vec![
                WorkerFacts {
                    endpoint_name: "worker-a".to_string(),
                    capabilities: DeclaredCapabilities {
                        system: "x86_64-linux".to_string(),
                        sandbox_mode: "practical".to_string(),
                        network_mode: "none".to_string(),
                        features: Vec::new(),
                    },
                    declared_accelerator_classes: Vec::new(),
                    worker_generation: 1,
                    remaining: Capacity {
                        cpu_units: 0,
                        memory_bytes: 0,
                        scratch_bytes: 0,
                    },
                    named: NamedResourceFit {
                        is_empty: true,
                        is_constrained: false,
                    },
                    resource_fit: 0,
                    content_locality: 0,
                    transfer_cost: 0,
                },
                WorkerFacts {
                    endpoint_name: "worker-b".to_string(),
                    capabilities: DeclaredCapabilities {
                        system: "aarch64-linux".to_string(),
                        sandbox_mode: "practical".to_string(),
                        network_mode: "none".to_string(),
                        features: Vec::new(),
                    },
                    declared_accelerator_classes: Vec::new(),
                    worker_generation: 1,
                    remaining: Capacity {
                        cpu_units: 0,
                        memory_bytes: 0,
                        scratch_bytes: 0,
                    },
                    named: NamedResourceFit {
                        is_empty: true,
                        is_constrained: false,
                    },
                    resource_fit: 0,
                    content_locality: 0,
                    transfer_cost: 0,
                },
            ],
            preferences: vec![PlacementPreference::ResourceFit],
            inventories: vec![None, None],
            state_dirs: vec![PathBuf::from("/tmp/remote-pool-factory-test"); 2],
        };
        let endpoints = vec![String::from("worker-a"), String::from("worker-b")];
        assert_eq!(selector.select(&goal("x86_64-linux"), &endpoints), Ok(0));
        assert_eq!(selector.select(&goal("riscv64-linux"), &endpoints), Err(PoolSelectionError::NoFit));
        assert!(RemotePoolExecutorFactory::new(admitted, vec![plan("worker-a", "x86_64-linux")], Vec::new(),).is_err());
    }
}
