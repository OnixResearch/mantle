let Artifact = {
  path | String,
  blake3 | String,
} in

let Validation = {
  command | String,
  verdict | String,
  evidence | String,
} in

let Finding = {
  scope | String,
  class | String,
  evidence | String,
} in

let AdmissionReceipt = {
  schema | String,
  implementation_commit | String,
  compatibility = {
    traditional_form | String,
    versioned_form | String,
    logical_store_prefixes | Array String,
    native_hash_domain | String,
    nix_compatibility_hash_domain | String,
    receipt_hash_domain | String,
  },
  limits = {
    candidate_bytes_max | Number,
    field_bytes_max | Number,
    collection_entries_max | Number,
    parent_edges_max | Number,
    dynamic_nodes_max | Number,
    dynamic_depth_max | Number,
    parser_collections_max | Number,
  },
  former_zero_fallback = {
    hdm_blake3 | String,
    drv_path | String,
    disposition | String,
  },
  artifacts | Array Artifact,
  validation | Array Validation,
  bounded_findings | Array Finding,
  claim_boundary | Array String,
} in

{
  schema = "mantle-native-dynamic-admission-receipt-v1",
  implementation_commit = "015813626bc6f2f875dc0c2682d5129cf9c3c150",
  compatibility = {
    traditional_form = "Derive(",
    versioned_form = "DrvWithVersion(\"xp-dyn-drv\",",
    logical_store_prefixes = ["configured-absolute-prefix"],
    native_hash_domain = "blake3",
    nix_compatibility_hash_domain = "nix-required-sha256",
    receipt_hash_domain = "blake3",
  },
  limits = {
    candidate_bytes_max = 4194304,
    field_bytes_max = 1048576,
    collection_entries_max = 16384,
    parent_edges_max = 4096,
    dynamic_nodes_max = 16384,
    dynamic_depth_max = 64,
    parser_collections_max = 65536,
  },
  former_zero_fallback = {
    hdm_blake3 = "3e0c7a40bf762ac25cd6197d36fe18f3b322a0d0d972c48a1fdad53565f3f906",
    drv_path = "/nix/store/rg33vjgs3ksf3d0g5c2gjqcf0np64dxz-hello.drv",
    disposition = "rejected-missing-parent-fact",
  },
  artifacts = [
    { path = "crates/crunch-build/src/dynamic.rs", blake3 = "e1d70c1bc32731a3f22ee44e86fe8b09954ab76c278804f9e103beaaa5a94ddf" },
    { path = "crates/crunch-build/src/registry.rs", blake3 = "2d49885eb8aa0de380f0a951ab7066e2451eb0005a65aa80c87d719a47762d00" },
    { path = "crates/crunch-build/src/worker.rs", blake3 = "583127472cfa728a095f7c3c367e0c84522a26d2df5985c51e153746780bacdf" },
    { path = "scripts/check-dynamic-admission-boundary.rs", blake3 = "4c720e1d1630435c1e5ebfb2290459b9ce568e6143942960bdf5ab7698586427" },
    { path = "adr/0002-dynamic-derivations.md", blake3 = "13b748e0f7182283351f93abbc1272dc492acd05e5f06333aaed9bfb829f2b55" },
    { path = "docs/dynamic-derivation-admission.md", blake3 = "98a95b64bc7ed211b2c5e3c56470fa55e45b33569d1c97b202856cce3e8c0f0b" },
  ],
  validation = [
    {
      command = "cargo test -p crunch-build --lib --tests",
      verdict = "pass-674-unit-plus-1-api",
      evidence = "implementation-validation.log",
    },
    {
      command = "cargo clippy -p crunch-build --lib --tests --no-deps -- -D warnings",
      verdict = "pass",
      evidence = "implementation-validation.log",
    },
    {
      command = "cargo -Zscript scripts/check-dynamic-admission-boundary.rs --self-test; cargo -Zscript scripts/check-dynamic-admission-boundary.rs",
      verdict = "pass-positive-negative-and-source-boundary",
      evidence = "implementation-validation.log",
    },
    {
      command = "cargo -Zscript scripts/check-nix-derivation-boundary.rs",
      verdict = "pass-no-native-dependency-leakage",
      evidence = "implementation-validation.log",
    },
    {
      command = "focused rustfmt and git diff --check",
      verdict = "pass",
      evidence = "implementation-validation.log",
    },
  ],
  bounded_findings = [
    {
      scope = "cargo test --workspace --lib --tests --no-fail-fast",
      class = "thirteen unrelated root and integration targets failed; crunch-build passed 674 tests",
      evidence = "workspace-tests.log",
    },
    {
      scope = "first-party strict Clippy",
      class = "unrelated mantlepkgs-core absurd-extreme-comparisons findings",
      evidence = "first-party-clippy.log",
    },
    {
      scope = "focused Tiger Style",
      class = "changed dynamic admission source has no finding; existing crunch-build files stop the package gate",
      evidence = "focused-tigerstyle.log",
    },
    {
      scope = "cargo fmt --all -- --check",
      class = "unrelated source_built_fixed_point_shell.rs drift",
      evidence = "fmt-all.log",
    },
    {
      scope = "nix build .#crunch -L",
      class = "release compilation reached the known seccomp listener test failure after 173 passes",
      evidence = "nix-package-check.log",
    },
  ],
  claim_boundary = [
    "recorded dynamic metadata passed bounded admission checks",
    "builder correctness is not proven",
    "source trust is not proven",
    "sandbox enforcement is not proven",
    "output correctness is not proven",
    "scheduler quality is not proven",
    "release eligibility is not proven",
  ],
} | AdmissionReceipt
