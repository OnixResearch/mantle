//! Pool executor factory: per-endpoint dispatches plus the placement selector.
//!
//! The pipeline hands over its opened store; this module clones the owned
//! transfer capability into each endpoint executor and builds the selector from
//! the admitted pool and the caller's scheduling preference order.

use std::path::PathBuf;
use std::sync::Arc;

use crunch_build::distributed::PoolDispatch;
use crunch_build::distributed::PoolEndpointService;
use crunch_build::distributed::PoolGoalExecutor;
use crunch_build::distributed::PoolGoalSelector;
use crunch_build::distributed::PoolSelectionError;
use crunch_build::distributed::PreparedPoolGoal;
use crunch_build::distributed::RemoteFailureDebugPolicy;
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
        })
    }
}

impl PoolExecutorFactory for RemotePoolExecutorFactory {
    fn create(&self, store: &crunch_store::StoreHandle) -> Result<Arc<PoolDispatch>, String> {
        let transfer = store.owned_transfer_objects();
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
        });
        PoolDispatch::new(services, selector).map(Arc::new).map_err(|error| error.to_string())
    }
}

/// Selects an endpoint from resolved goal facts through the pool placement core.
struct PoolPolicySelector {
    admitted: AdmittedPool,
    workers: Vec<WorkerFacts>,
    preferences: Vec<PlacementPreference>,
}

impl PoolGoalSelector for PoolPolicySelector {
    fn select(&self, goal: &PreparedPoolGoal, endpoint_ids: &[String]) -> Result<usize, PoolSelectionError> {
        let requirement = GoalRequirement {
            system: goal.system.clone(),
            sandbox_mode: goal.required_sandbox_mode.clone(),
            network_mode: goal.required_network_mode.clone(),
            required_features: Vec::new(),
            required_accelerator_classes: Vec::new(),
            requested: Capacity {
                cpu_units: 0,
                memory_bytes: 0,
                scratch_bytes: 0,
            },
            requires_named_resources: false,
        };
        match plan_goal_placement(&self.admitted, &requirement, &self.workers, self.preferences.iter().copied()) {
            GoalPlacement::Selected { endpoint_name, .. } => {
                endpoint_ids.iter().position(|id| id == &endpoint_name).ok_or(PoolSelectionError::NoFit)
            }
            GoalPlacement::Unplaceable { .. } => Err(PoolSelectionError::NoFit),
        }
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
        })
        .expect("fixture goal")
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
        };
        let endpoints = vec![String::from("worker-a"), String::from("worker-b")];
        assert_eq!(selector.select(&goal("x86_64-linux"), &endpoints), Ok(0));
        assert_eq!(selector.select(&goal("riscv64-linux"), &endpoints), Err(PoolSelectionError::NoFit));
        assert!(RemotePoolExecutorFactory::new(admitted, vec![plan("worker-a", "x86_64-linux")], Vec::new(),).is_err());
    }
}
