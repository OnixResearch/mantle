# Representative offline two-package Cargo unit plan. Its declared /nix/store
# inputs require Mantle's normal signed SourceAdmission before a build: no
# direct NAR injection and no Cargo compilation fallback for blocked units.
# Full Mantle workspace (925 units, 711 package sources, max direct 65,
# 948146 compact Cargo graph JSON bytes) exceeds the 256-slice plan limit.
let cargo_unit_plan = import "../lib/cargo_unit_plan.ncl" in
let mantle = import "../lib/lib.ncl" in
let plan_producer = cargo_unit_plan {
  goal_name = "cargo-unit-plan-representative",
  producer = "/nix/store/x1j5wypgp65wf4gsdllq7gjqa0jbmdhv-mantle-cargo-unit-producer",
  config = "/nix/store/zqv8v8dgf38c81yws4b7szs29hmxfq9z-producer-config.json",
  workspace = "/nix/store/2cc8iz424blki3kgz09ajdndy0n6pjlc-baseline-workspace",
  planner = "/nix/store/6380jpkn2v95iqh12gdpal2dv0yiscll-mantle",
  toolchain = "/nix/store/1hz41jg6cc3py3szsn6cxw3m5p42r4fj-rust-default-1.96.0-nightly-2026-04-08",
  linker = "/nix/store/hb2bs5fg5wkm04x565737qd5nh2hy5nk-gcc-wrapper-15.2.0",
  helper = "/nix/store/hmq5j94ssxwf4sxga8gys22wslsncgkk-mantle-cargo-unit-helper",
  vendor_sources = [],
} in
{
  producer = plan_producer,
  app = {
    name = "cargo-unit-plan-app-check",
    builder = "/nix/store/7r1xv5ij16r1s1v2mijhhijgwbz4ysi8-app-consumer",
    addressing_mode = 'input-addressed,
    sandbox = 'native,
    inputs = [
      "/nix/store/7r1xv5ij16r1s1v2mijhhijgwbz4ysi8-app-consumer",
      {
        name = "app",
        producer = plan_producer,
        plan_output = "plan",
        root = "u.a1b9ca38f6231fb003fd22ff87350acddc895585f48116b9eaa7162a3800722b",
        unit_output = "out",
      },
    ],
    env = { MANTLE_UNIT_APP = "{{mantle-plan-output:app}}" },
  } | mantle.Derivation,
}
