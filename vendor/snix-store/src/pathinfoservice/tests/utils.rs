#[cfg(all(feature = "cloud", feature = "integration"))]
pub(crate) async fn make_bigtable_path_info_service() -> crate::pathinfoservice::BigtablePathInfoService {
    use crate::pathinfoservice::BigtablePathInfoService;
    use crate::pathinfoservice::bigtable::BigtableParameters;

    BigtablePathInfoService::connect("test".into(), BigtableParameters::default_for_tests())
        .await
        .unwrap()
}
