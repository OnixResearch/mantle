# Mantle representative Rust compatibility surface matrix.
#
# This is a bounded evidence inventory, not a full Cargo compatibility claim.
# Tests in tests/rust_compatibility_rail.rs consume these stable ids and docs
# cite this file when describing frontier movement.
# The unit_plan lane is an opt-in signed two-package fixture, not a claim that
# Mantle's measured 925-unit/711-source workspace fits the 256-slice bound.

let LaneStatus = [| 'supported, 'blocked, 'out_of_scope |] in
let Lane = {
  status | LaneStatus,
  evidence_class | String,
  fixtures | Array String,
  blockers | Array String,
} in
let Surface = {
  id | String,
  title | String,
  offline_cargo | Lane,
  native_rust_plan | Lane,
  unit_plan | Lane,
  non_claims | Array String,
} in

{
  schema_version | Number = 1,
  matrix_id | String = "representative-rust-compatibility-v1",
  matrix_path | String = "examples/rust_compatibility_surface_matrix.ncl",
  surfaces | Array Surface = [
    {
      id = "path-workspace-basic",
      title = "path workspace with binary package",
      offline_cargo = {
        status = 'supported,
        evidence_class = "cargo-inside-mantle-sandbox",
        fixtures = ["representative_offline_cargo_rail_runs_sandbox_smoke_without_network_inputs"],
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
        status = 'supported,
        evidence_class = "cargo-inside-mantle-sandbox",
        fixtures = ["representative_fixture_contains_required_practical_rust_surfaces"],
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
        status = 'supported,
        evidence_class = "cargo-inside-mantle-sandbox",
        fixtures = ["representative_fixture_contains_required_practical_rust_surfaces"],
        blockers = [],
      },
      native_rust_plan = {
        status = 'blocked,
        evidence_class = "blocked-unsupported-surface",
        fixtures = ["representative_negative_fixtures_report_stable_blockers"],
        blockers = ["missing-vendored-registry-source", "native-registry-source-planning-blocked"],
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
        status = 'supported,
        evidence_class = "cargo-inside-mantle-sandbox",
        fixtures = ["representative_fixture_contains_required_practical_rust_surfaces"],
        blockers = [],
      },
      native_rust_plan = {
        status = 'blocked,
        evidence_class = "blocked-unsupported-surface",
        fixtures = ["representative_negative_fixtures_report_stable_blockers"],
        blockers = ["missing-proc-macro-host-artifact"],
      },
      unit_plan = {
        status = 'out_of_scope,
        evidence_class = "unexercised-cargo-unit-surface",
        fixtures = [],
        blockers = [],
      },
      non_claims = ["not-full-host-scheduling-or-execution", "not-full-cargo-compatibility"],
    },
    {
      id = "build-script-env-metadata",
      title = "simple build-script rustc-env metadata",
      offline_cargo = {
        status = 'supported,
        evidence_class = "cargo-inside-mantle-sandbox",
        fixtures = ["representative_fixture_contains_required_practical_rust_surfaces"],
        blockers = [],
      },
      native_rust_plan = {
        status = 'blocked,
        evidence_class = "blocked-unsupported-surface",
        fixtures = ["representative_negative_fixtures_report_stable_blockers"],
        blockers = ["malformed-build-script-metadata", "native-host-unit-graph-planning-blocked"],
      },
      unit_plan = {
        status = 'blocked,
        evidence_class = "blocked-unsupported-surface",
        fixtures = ["unsupported_execution_modes_and_unbound_paths_fail_closed"],
        blockers = ["unit-plan-build-script-run-unsupported"],
      },
      non_claims = ["not-full-host-scheduling-or-execution", "not-full-cargo-compatibility"],
    },
    {
      id = "feature-resolution-default",
      title = "bounded feature activation in the representative fixture",
      offline_cargo = {
        status = 'supported,
        evidence_class = "cargo-inside-mantle-sandbox",
        fixtures = ["representative_fixture_contains_required_practical_rust_surfaces"],
        blockers = [],
      },
      native_rust_plan = {
        status = 'blocked,
        evidence_class = "blocked-unsupported-surface",
        fixtures = ["representative_negative_fixtures_report_stable_blockers"],
        blockers = ["missing-feature-surface", "native-unit-graph-planning-blocked"],
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
        status = 'supported,
        evidence_class = "cargo-inside-mantle-sandbox",
        fixtures = ["representative_fixture_contains_required_practical_rust_surfaces"],
        blockers = [],
      },
      native_rust_plan = {
        status = 'blocked,
        evidence_class = "blocked-unsupported-surface",
        fixtures = ["representative_negative_fixtures_report_stable_blockers"],
        blockers = ["missing-target-cfg-surface", "native-package-target-planning-blocked"],
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
        status = 'supported,
        evidence_class = "cargo-inside-mantle-sandbox",
        fixtures = ["representative_fixture_contains_required_practical_rust_surfaces"],
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
        status = 'supported,
        evidence_class = "cargo-inside-mantle-sandbox",
        fixtures = ["representative_fixture_contains_required_practical_rust_surfaces"],
        blockers = [],
      },
      native_rust_plan = {
        status = 'blocked,
        evidence_class = "blocked-unsupported-surface",
        fixtures = ["representative_negative_fixtures_report_stable_blockers"],
        blockers = ["missing-multi-package-binary-surface"],
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
        status = 'blocked,
        evidence_class = "blocked-unsupported-surface",
        fixtures = ["representative_negative_fixtures_report_stable_blockers"],
        blockers = ["unsupported-vendored-git-source"],
      },
      native_rust_plan = {
        status = 'blocked,
        evidence_class = "blocked-unsupported-surface",
        fixtures = ["representative_negative_fixtures_report_stable_blockers"],
        blockers = ["unsupported-vendored-git-source", "native-git-source-planning-blocked"],
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
      title = "links/pkg-config/rustc-link/native C metadata",
      offline_cargo = {
        status = 'blocked,
        evidence_class = "blocked-unsupported-surface",
        fixtures = ["representative_negative_fixtures_report_stable_blockers"],
        blockers = ["unsupported-native-link-metadata", "unsupported-rustc-link-metadata", "unsupported-pkg-config", "unsupported-native-c-compile"],
      },
      native_rust_plan = {
        status = 'blocked,
        evidence_class = "blocked-unsupported-surface",
        fixtures = ["representative_negative_fixtures_report_stable_blockers"],
        blockers = ["unsupported-native-link-metadata", "unsupported-rustc-link-metadata", "unsupported-pkg-config", "unsupported-native-c-compile"],
      },
      unit_plan = {
        status = 'out_of_scope,
        evidence_class = "unexercised-cargo-unit-surface",
        fixtures = [],
        blockers = [],
      },
      non_claims = ["not-native-link-probing-correctness", "not-full-cargo-compatibility"],
    },
  ],
}
