#[derive(Debug, Clone)]
pub struct SuppliedWorkspaceFactsAdapter {
    pub facts: mantle_rust_plan_core::StructuralWorkspaceFacts,
}

impl mantle_rust_plan::WorkspaceFactsPort for SuppliedWorkspaceFactsAdapter {
    fn load_workspace(
        &mut self,
        request: mantle_rust_plan::WorkspaceFactsRequest,
    ) -> Result<mantle_rust_plan::WorkspaceFactsObservation, mantle_rust_plan::RustPlanPortError> {
        if request.workspace_hint.is_empty() {
            return Err(mantle_rust_plan::RustPlanPortError::new(
                mantle_rust_plan::RustPlanCapability::WorkspaceFacts,
                "workspace-hint-empty",
                false,
            ));
        }
        debug_assert!(!self.facts.workspace_identity.is_empty());
        debug_assert!(!self.facts.schema.is_empty());
        Ok(mantle_rust_plan::WorkspaceFactsObservation {
            facts: self.facts.clone(),
        })
    }
}
