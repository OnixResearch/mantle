use std::collections::BTreeMap;

#[derive(Debug, Default)]
pub struct InMemoryRustPlanCacheAdapter {
    pub artifacts: BTreeMap<String, Vec<mantle_rust_plan_core::ObservedArtifact>>,
    pub publications: Vec<mantle_rust_plan::CachePublishRequest>,
}

impl mantle_rust_plan::RustCachePort for InMemoryRustPlanCacheAdapter {
    fn lookup(
        &mut self,
        request: mantle_rust_plan::CacheLookupRequest,
    ) -> Result<mantle_rust_plan::CacheLookupObservation, mantle_rust_plan::RustPlanPortError> {
        let (status, artifacts) = match self.artifacts.get(&request.effect.effect_id) {
            Some(artifacts) => (mantle_rust_plan_core::CacheObservationStatus::Hit, artifacts.clone()),
            None => (mantle_rust_plan_core::CacheObservationStatus::Miss, Vec::new()),
        };
        debug_assert!(!request.effect.effect_id.is_empty());
        debug_assert!(artifacts.len() <= request.effect.limits.output_identities_max as usize);
        Ok(mantle_rust_plan::CacheLookupObservation {
            observation: mantle_rust_plan_core::CacheObservation {
                effect_id: request.effect.effect_id,
                status,
                artifacts,
                failure_code: None,
            },
        })
    }

    fn publish(
        &mut self,
        request: mantle_rust_plan::CachePublishRequest,
    ) -> Result<(), mantle_rust_plan::RustPlanPortError> {
        if request.effect.effect_id != request.outcome.effect_id {
            return Err(mantle_rust_plan::RustPlanPortError::new(
                mantle_rust_plan::RustPlanCapability::RustCache,
                "cache-publication-effect-mismatch",
                false,
            ));
        }
        self.artifacts.insert(request.effect.effect_id.clone(), request.outcome.artifacts.clone());
        self.publications.push(request);
        debug_assert!(!self.publications.is_empty());
        debug_assert_eq!(self.artifacts.len(), self.publications.len());
        Ok(())
    }
}
