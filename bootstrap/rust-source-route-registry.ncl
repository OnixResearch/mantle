# Mechanism-level registry for Rust provider construction routes.
#
# A route being present is not evidence that it completed. Full-bootstrap
# eligibility additionally requires the independently validated binding receipt
# named by `completion_probe`.

let Route = {
  mechanism_id | String,
  provider_origin | String,
  compiler_host | String,
  source_policy | String,
  ambient_discovery | Bool,
  uses_prebuilt_rust | Bool,
  full_bootstrap_candidate | Bool,
  completion_probe | String,
  disqualifying_non_claim | String,
} in
let Registry = {
  schema | String,
  selected_candidate | String,
  routes | Array Route,
} in
{
  schema = "mantle-rust-source-route-registry-v1",
  selected_candidate = "musl-host-full-source",
  routes = [
    {
      mechanism_id = "musl-host-full-source",
      provider_origin = "materialized-from-authenticated-stage-sources",
      compiler_host = "admitted-full-source-native-provider",
      source_policy = "authenticated-offline-only",
      ambient_discovery = false,
      uses_prebuilt_rust = false,
      full_bootstrap_candidate = true,
      completion_probe = "validated mantle-full-source-rust-provider-binding-v1 receipt",
      disqualifying_non_claim = "not-complete-without-current-binding-receipt",
    },
    {
      mechanism_id = "source-built-gnu-host",
      provider_origin = "materialized-from-authenticated-stage-sources",
      compiler_host = "source-built-gnu-host-compatibility",
      source_policy = "authenticated-sources-with-host-compatibility",
      ambient_discovery = false,
      uses_prebuilt_rust = false,
      full_bootstrap_candidate = false,
      completion_probe = "host triple differs from admitted musl native provider",
      disqualifying_non_claim = "not-full-source-native-provider-bound",
    },
    {
      mechanism_id = "imported-provider",
      provider_origin = "imported-existing-provider",
      compiler_host = "unproven-by-import-operation",
      source_policy = "import-validation-only",
      ambient_discovery = false,
      uses_prebuilt_rust = false,
      full_bootstrap_candidate = false,
      completion_probe = "provider origin is import rather than current construction",
      disqualifying_non_claim = "import-does-not-prove-current-construction",
    },
    {
      mechanism_id = "ambient-host",
      provider_origin = "materialized-with-ambient-host-tools",
      compiler_host = "ambient-path-discovery",
      source_policy = "authenticated-rust-sources-with-ambient-tools",
      ambient_discovery = true,
      uses_prebuilt_rust = false,
      full_bootstrap_candidate = false,
      completion_probe = "any compiler or linker selected through ambient PATH",
      disqualifying_non_claim = "ambient-host-tool-discovery",
    },
    {
      mechanism_id = "fetched-rust-compatibility",
      provider_origin = "fetched-prebuilt-rust-toolchain",
      compiler_host = "prebuilt-rust",
      source_policy = "fixed-fetch-compatibility",
      ambient_discovery = false,
      uses_prebuilt_rust = true,
      full_bootstrap_candidate = false,
      completion_probe = "provider provenance records uses_prebuilt_rust",
      disqualifying_non_claim = "prebuilt-rust-compatibility-only",
    },
  ],
} | Registry
