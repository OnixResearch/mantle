trait OutputLookupPort {
    fn pathinfo(&self) -> std::sync::Arc<dyn snix_store::pathinfoservice::PathInfoService>;
}
