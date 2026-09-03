use std::collections::BTreeMap;

#[derive(Debug, Default)]
pub struct ObservedUnitExecutionAdapter {
    pub observations: BTreeMap<String, mantle_rust_plan_core::RustUnitObservation>,
}

impl mantle_rust_plan::UnitExecutionPort for ObservedUnitExecutionAdapter {
    fn execute_unit(
        &mut self,
        request: mantle_rust_plan::UnitExecutionRequest,
    ) -> Result<mantle_rust_plan::UnitExecutionObservation, mantle_rust_plan::RustPlanPortError> {
        let observation = self.observations.remove(&request.effect.effect_id).ok_or_else(|| {
            mantle_rust_plan::RustPlanPortError::new(
                mantle_rust_plan::RustPlanCapability::UnitExecution,
                "unit-observation-missing",
                false,
            )
        })?;
        debug_assert_eq!(observation.effect_id, request.effect.effect_id);
        debug_assert!(!observation.stdout_blake3.is_empty());
        Ok(mantle_rust_plan::UnitExecutionObservation { observation })
    }
}
