# Mantle representative Rust compatibility surface matrix.
#
# This is a bounded evidence inventory, not a full Cargo compatibility claim.
# Tests in tests/rust_compatibility_rail.rs consume these stable ids and docs
# cite this file when describing frontier movement.
# The unit_plan lane retains separate signed app42/provider-change and reviewed
# links/pkg-config cohorts; neither claims native C compilation inside build.rs
# or that Mantle's 925-unit/711-source workspace fits the 256-slice bound.

let LaneStatus = [| 'supported, 'blocked, 'out_of_scope |] in
let Lane = {
  status | LaneStatus,
  evidence_class | String,
  fixtures | Array String,
  blockers | Array String,
} in
let SurfaceShape = {
  id | String,
  title | String,
  offline_cargo | Lane,
  native_rust_plan | Lane,
  unit_plan | Lane,
  non_claims | Array String,
} in
let Surface = std.contract.from_validator (fun value =>
  let checked = value | SurfaceShape in
  std.deep_seq checked (
    if checked.unit_plan.status != 'supported
      || (std.array.length checked.unit_plan.fixtures > 0
        && std.array.all (std.string.is_match "[^[:space:]]") checked.unit_plan.fixtures)
    then 'Ok
    else 'Error { message = "supported unit_plan row %{checked.id} requires a named passing fixture" }
  )
) in
let SignedBuildScriptFixture = [
  "fixtures/cargo-build-script-units/Cargo.toml",
  ".cairn/changes/run-cargo-build-scripts-as-plan-units/evidence/t41-signed-app-2026-10-14.md",
  ".cairn/changes/run-cargo-build-scripts-as-plan-units/evidence/t42-selective-provider-2026-10-15.md",
] in

{
  schema_version | Number = 1,
  matrix_id | String = "representative-rust-compatibility-v1",
  matrix_path | String = "examples/rust_compatibility_surface_matrix.ncl",
  surfaces | Array Surface = [
    {
      id = "path-workspace-basic",
      title = "path workspace with binary package",
      offline_cargo = {
        status = 'out_of_scope,
        evidence_class = "unexercised-real-cargo-compilation",
        fixtures = [],
        blockers = [],
      },
      native_rust_plan = {
        status = 'supported,
        evidence_class = "cargo-free-bounded-topology",
        fixtures = ["native_path_only_surface_reports_matrix_binding_without_cargo_fallback"],
        blockers = [],
      },
      unit_plan = {
        status = 'supported,
        evidence_class = "cargo-unit-graph-dynamic-plan",
        fixtures = ["examples/cargo_unit_plan.ncl"],
        blockers = [],
      },
      non_claims = ["not-full-cargo-compatibility", "not-compiler-correctness"],
    },
    {
      id = "local-path-dependency",
      title = "local path library dependency",
      offline_cargo = {
        status = 'out_of_scope,
        evidence_class = "unexercised-real-cargo-compilation",
        fixtures = [],
        blockers = [],
      },
      native_rust_plan = {
        status = 'supported,
        evidence_class = "cargo-free-bounded-topology",
        fixtures = ["native_path_only_surface_reports_matrix_binding_without_cargo_fallback"],
        blockers = [],
      },
      unit_plan = {
        status = 'supported,
        evidence_class = "cargo-unit-graph-dynamic-plan",
        fixtures = ["examples/cargo_unit_plan.ncl"],
        blockers = [],
      },
      non_claims = ["not-full-cargo-feature-resolution", "not-full-cargo-compatibility"],
    },
    {
      id = "vendored-registry-source",
      title = "declared local vendored registry source",
      offline_cargo = {
        status = 'out_of_scope,
        evidence_class = "unexercised-real-cargo-compilation",
        fixtures = [],
        blockers = [],
      },
      native_rust_plan = {
        status = 'out_of_scope,
        evidence_class = "unexercised-no-cargo-oracle-surface",
        fixtures = [],
        blockers = [],
      },
      unit_plan = {
        status = 'out_of_scope,
        evidence_class = "unexercised-cargo-unit-surface",
        fixtures = [],
        blockers = [],
      },
      unit_plan = {
        status = 'out_of_scope,
        evidence_class = "unexercised-cargo-unit-surface",
        fixtures = [],
        blockers = [],
      },
      non_claims = ["not-network-or-ambient-cargo-source-resolution", "not-full-cargo-compatibility"],
    },
    {
      id = "proc-macro-host-artifact",
      title = "proc-macro host artifact",
      offline_cargo = {
        status = 'out_of_scope,
        evidence_class = "unexercised-real-cargo-compilation",
        fixtures = [],
        blockers = [],
      },
      native_rust_plan = {
        status = 'out_of_scope,
        evidence_class = "unexercised-no-cargo-oracle-surface",
        fixtures = [],
        blockers = [],
      },
      unit_plan = {
        status = 'supported,
        evidence_class = "signed-build-script-proc-macro-cohorts",
        fixtures = SignedBuildScriptFixture @ ["scripts/rust-unit-plan-build-script-replay/replay.sh"],
        blockers = [],
      },
      non_claims = ["not-general-cross-host-scheduling", "not-full-host-scheduling-or-execution", "not-full-cargo-compatibility"],
    },
    {
      id = "build-script-env-metadata",
      title = "executed build-script cfg, generated source and rustc-env metadata",
      offline_cargo = {
        status = 'out_of_scope,
        evidence_class = "unexercised-real-cargo-compilation",
        fixtures = [],
        blockers = [],
      },
      native_rust_plan = {
        status = 'out_of_scope,
        evidence_class = "unexercised-no-cargo-oracle-surface",
        fixtures = [],
        blockers = [],
      },
      unit_plan = {
        status = 'supported,
        evidence_class = "real-helper-script-84-and-signed-build-script-cohorts",
        fixtures = [
          "generated_source_cfg_and_env_flow_from_executed_script_to_compiler",
          ".cairn/changes/run-cargo-build-scripts-as-plan-units/evidence/t41-signed-app-2026-10-14.md",
          "scripts/rust-unit-plan-build-script-replay/replay.sh",
        ],
        blockers = [],
      },
      non_claims = ["not-arbitrary-build-script-directives", "not-full-host-scheduling-or-execution", "not-full-cargo-compatibility"],
    },
    {
      id = "feature-resolution-default",
      title = "bounded feature activation in the representative fixture",
      offline_cargo = {
        status = 'out_of_scope,
        evidence_class = "unexercised-real-cargo-compilation",
        fixtures = [],
        blockers = [],
      },
      native_rust_plan = {
        status = 'blocked,
        evidence_class = "blocked-unsupported-surface",
        fixtures = ["rust_plan_cli_no_cargo_oracle_blocks_unsupported_surface_without_cargo_fallback"],
        blockers = ["unsupported-feature-surface", "native-unit-graph-planning-blocked"],
      },
      unit_plan = {
        status = 'out_of_scope,
        evidence_class = "unexercised-cargo-unit-surface",
        fixtures = [],
        blockers = [],
      },
      unit_plan = {
        status = 'out_of_scope,
        evidence_class = "unexercised-cargo-unit-surface",
        fixtures = [],
        blockers = [],
      },
      non_claims = ["not-full-cargo-feature-resolution", "not-full-cargo-compatibility"],
    },
    {
      id = "target-specific-dependency",
      title = "target cfg dependency declaration",
      offline_cargo = {
        status = 'out_of_scope,
        evidence_class = "unexercised-real-cargo-compilation",
        fixtures = [],
        blockers = [],
      },
      native_rust_plan = {
        status = 'out_of_scope,
        evidence_class = "unexercised-no-cargo-oracle-surface",
        fixtures = [],
        blockers = [],
      },
      unit_plan = {
        status = 'out_of_scope,
        evidence_class = "unexercised-cargo-unit-surface",
        fixtures = [],
        blockers = [],
      },
      unit_plan = {
        status = 'out_of_scope,
        evidence_class = "unexercised-cargo-unit-surface",
        fixtures = [],
        blockers = [],
      },
      non_claims = ["not-full-cargo-feature-resolution", "not-full-cargo-compatibility"],
    },
    {
      id = "workspace-inheritance",
      title = "workspace package metadata inheritance",
      offline_cargo = {
        status = 'out_of_scope,
        evidence_class = "unexercised-real-cargo-compilation",
        fixtures = [],
        blockers = [],
      },
      native_rust_plan = {
        status = 'supported,
        evidence_class = "cargo-free-bounded-topology",
        fixtures = ["native_path_only_surface_reports_matrix_binding_without_cargo_fallback"],
        blockers = [],
      },
      unit_plan = {
        status = 'out_of_scope,
        evidence_class = "unexercised-cargo-unit-surface",
        fixtures = [],
        blockers = [],
      },
      non_claims = ["not-full-cargo-compatibility"],
    },
    {
      id = "multi-package-binary",
      title = "workspace sibling binary package",
      offline_cargo = {
        status = 'out_of_scope,
        evidence_class = "unexercised-real-cargo-compilation",
        fixtures = [],
        blockers = [],
      },
      native_rust_plan = {
        status = 'out_of_scope,
        evidence_class = "unexercised-no-cargo-oracle-surface",
        fixtures = [],
        blockers = [],
      },
      unit_plan = {
        status = 'supported,
        evidence_class = "cargo-unit-graph-dynamic-plan",
        fixtures = ["examples/cargo_unit_plan.ncl"],
        blockers = [],
      },
      unit_plan = {
        status = 'supported,
        evidence_class = "cargo-unit-graph-dynamic-plan",
        fixtures = ["examples/cargo_unit_plan.ncl"],
        blockers = [],
      },
      non_claims = ["not-general-rust-scheduler", "not-full-cargo-compatibility"],
    },
    {
      id = "vendored-git-source",
      title = "captured git source material",
      offline_cargo = {
        status = 'out_of_scope,
        evidence_class = "unexercised-real-cargo-compilation",
        fixtures = [],
        blockers = [],
      },
      native_rust_plan = {
        status = 'out_of_scope,
        evidence_class = "unexercised-no-cargo-oracle-surface",
        fixtures = [],
        blockers = [],
      },
      unit_plan = {
        status = 'blocked,
        evidence_class = "blocked-unsupported-surface",
        fixtures = ["proc_macro_host_unit_and_git_source_boundary"],
        blockers = ["unit-plan-source-unsupported"],
      },
      unit_plan = {
        status = 'blocked,
        evidence_class = "blocked-unsupported-surface",
        fixtures = ["proc_macro_host_unit_and_git_source_boundary"],
        blockers = ["unit-plan-source-unsupported"],
      },
      non_claims = ["not-network-or-ambient-cargo-source-resolution", "not-full-cargo-compatibility"],
    },
    {
      id = "native-link-metadata",
      title = "declared links/pkg-config/rustc-link metadata with a prebuilt native provider",
      offline_cargo = {
        status = 'out_of_scope,
        evidence_class = "unexercised-real-cargo-compilation",
        fixtures = [],
        blockers = [],
      },
      native_rust_plan = {
        status = 'out_of_scope,
        evidence_class = "unexercised-no-cargo-oracle-surface",
        fixtures = [],
        blockers = [],
      },
      unit_plan = {
        status = 'supported,
        evidence_class = "cargo-unit-graph-dynamic-plan",
        fixtures = [
          "scripts/rust-unit-plan-build-script-replay/replay.sh",
          "scripts/rust-unit-plan-build-script-replay/libbz2.ncl",
        ],
        blockers = [],
      },
      non_claims = ["not-native-c-in-build-script", "not-native-link-probing-correctness", "not-full-cargo-compatibility"],
    },
    {
      id = "declared-native-static-link",
      title = "declared prebuilt C-provider static linking through a links package",
      offline_cargo = {
        status = 'out_of_scope,
        evidence_class = "unexercised-real-cargo-compilation",
        fixtures = [],
        blockers = [],
      },
      native_rust_plan = {
        status = 'out_of_scope,
        evidence_class = "unexercised-declared-native-provider",
        fixtures = [],
        blockers = [],
      },
      unit_plan = {
        status = 'supported,
        evidence_class = "signed-app42-declared-static-provider-selective-rerun",
        fixtures = SignedBuildScriptFixture,
        blockers = [],
      },
      non_claims = ["not-pkg-config-execution", "not-native-c-in-build-script", "not-links-dep-metadata-consumption", "not-native-link-probing-correctness", "not-full-cargo-compatibility"],
    },
  ],
}
