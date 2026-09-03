#[derive(Debug, Clone)]
pub struct SuppliedBuildPlanningObservations {
    pub facts: crunch_build_planning_core::BuildPlanningFacts,
    pub policy: crunch_build_planning_core::BuildPlanningPolicy,
    pub parallelism: crunch_build_planning_core::ParallelismFacts,
}

impl SuppliedBuildPlanningObservations {
    pub fn plan(
        self,
    ) -> Result<crunch_build_planning_core::BuildPlanningDecision, crunch_build_planning_core::BuildPlanningError> {
        let request = crunch_build_planning_core::BuildPlanningRequest {
            facts: self.facts,
            policy: self.policy,
            parallelism: self.parallelism,
        };
        crunch_build_planning_core::plan_build(request)
    }
}
