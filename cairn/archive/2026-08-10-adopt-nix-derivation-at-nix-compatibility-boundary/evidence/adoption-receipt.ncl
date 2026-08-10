let Artifact = {
  path | String,
  blake3 | String,
} in

let Validation = {
  command | String,
  verdict | String,
  evidence | String,
} in

let Failure = {
  scope | String,
  class | String,
  evidence | String,
} in

let AdoptionReceipt = {
  schema | String,
  implementation_commit | String,
  dependency = {
    package | String,
    version | String,
    crate_sha256 | String,
    license | String,
    repository | String,
    upstream_commit | String,
  },
  compatibility = {
    nix_versions | Array String,
    logical_store_prefix | String,
    native_hash_domain | String,
    receipt_hash_domain | String,
  },
  artifacts | Array Artifact,
  validation | Array Validation,
  bounded_failures | Array Failure,
  claim_boundary | Array String,
} in

{
  schema = "mantle-nix-derivation-adoption-receipt-v1",
  implementation_commit = "7483db622d97c32788cecd7ad48f909d2f874e78",
  dependency = {
    package = "nix-derivation",
    version = "0.1.0",
    crate_sha256 = "a5d03dfde06a8ce7e0e007f4795ab74c546e5c7d6c6375a08ceb20677ec8a074",
    license = "Apache-2.0",
    repository = "https://github.com/cachix/nix-derivation",
    upstream_commit = "2cfc0f90ed83ea3cc983e5c305f89494a6df073e",
  },
  compatibility = {
    nix_versions = ["2.34"],
    logical_store_prefix = "/nix/store",
    native_hash_domain = "sha256",
    receipt_hash_domain = "blake3",
  },
  artifacts = [
    { path = "Cargo.toml", blake3 = "a7de99c1c70d40cf07896806660a0104d39af3f8d46c8f8ea71baa9ca415a56c" },
    { path = "Cargo.lock", blake3 = "3b11996f74a0f765c3333728b5eaef789885b7df5398f06ba16610de058485e1" },
    { path = "src/nix_derivation_adapter.rs", blake3 = "fa0a39c8c10ee59e3e5367ae0fed1b5face074e04779d437e47cb2424181e50f" },
    { path = "scripts/check-nix-derivation-boundary.rs", blake3 = "b766dd90b900c6f9e07b07d99d1fa2af9a2838afe8ed72c0fed9be2819051c4c" },
    { path = "config/nix-derivation-admission.ncl", blake3 = "9538985594d04ae68608f09a4ea4e7d9e3a654ebd42668a7fbc55ddaa0f85fd3" },
    { path = "tests/fixtures/foreign-import/nixpkgs-hello-root.drv", blake3 = "140b911a35816a8904a0a5d29ec5f72fe7d2eb63b3d404a67a92a2224facdb3d" },
    { path = "tests/fixtures/foreign-import/nixpkgs-hello-source.drv", blake3 = "ab10367baaee0e3f71297c6bfcd3f349d95bffc3f314d1fa1228bcef099013e6" },
  ],
  validation = [
    {
      command = "focused adapter, foreign import, producer, upstream, guard, rustfmt, and diff checks",
      verdict = "pass",
      evidence = "final-focused-validation.log",
    },
    {
      command = "crate archive and detached upstream source parity",
      verdict = "pass",
      evidence = "source-admission-validation.log",
    },
    {
      command = "offline locked Cargo metadata through vendor-deps",
      verdict = "pass",
      evidence = "vendor-offline-validation.log",
    },
    {
      command = "focused first-party clippy with the existing large-enum allowance",
      verdict = "pass",
      evidence = "focused-clippy.log",
    },
  ],
  bounded_failures = [
    {
      scope = "cargo test --workspace",
      class = "five unrelated root-binary failures after 2292 passes",
      evidence = "workspace-tests.log",
    },
    {
      scope = "cargo clippy --workspace --all-targets -- -D warnings",
      class = "vendored fuse-backend-rs warnings",
      evidence = "workspace-clippy.log",
    },
    {
      scope = "cargo fmt --all -- --check",
      class = "unrelated source_built_fixed_point_shell.rs drift",
      evidence = "fmt-all.log",
    },
    {
      scope = "nix build .#crunch -L",
      class = "known seccomp sandbox test failure after release compilation and 173 test passes",
      evidence = "nix-package-check.log",
    },
  ],
  claim_boundary = [
    "selected Nix derivation metadata passed the recorded compatibility checks",
    "builder safety is not proven",
    "sandboxing is not proven",
    "output correctness is not proven",
    "reproducibility is not proven",
    "release eligibility is not proven",
  ],
} | AdoptionReceipt
