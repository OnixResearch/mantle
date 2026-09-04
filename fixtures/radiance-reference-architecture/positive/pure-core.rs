struct StagePlan {
    source_projection: String,
    predecessor_role: String,
}

fn validate(plan: &StagePlan) -> bool {
    plan.source_projection == "." && !plan.predecessor_role.is_empty()
}
