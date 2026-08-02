# Checked cache-imported Nixpkgs tool-use workflow.
let Producer = {
  source_ref | String,
  revision | String,
  attribute | String,
  root_derivation | String,
  selected_output | String,
} in
let Consumer = {
  derivation | String,
  builder | String,
  source_input | String,
  expected_provenance | String,
} in
let Workflow = {
  schema | String,
  id | String,
  producer | Producer,
  consumer | Consumer,
  phases | Array String,
  positive_evidence | Array String,
  negative_cases | Array String,
  non_claims | Array String,
} in

{
  schema = "mantle-example-workflow-v1",
  id = "nixpkgs-tool-use",
  producer = {
    source_ref = "github:NixOS/nixpkgs/dfd9566f82a6e1d55c30f861879186440614696e",
    revision = "dfd9566f82a6e1d55c30f861879186440614696e",
    attribute = "mksh",
    root_derivation = "/nix/store/ibzx4zxyq58c1d6cv41505y152c9incd-mksh-59c.drv",
    selected_output = "/nix/store/baa9yl7sazygz4k4ma2v343n4aadjhlj-mksh-59c",
  },
  consumer = {
    derivation = "consumer.ncl",
    builder = "/nix/store/baa9yl7sazygz4k4ma2v343n4aadjhlj-mksh-59c/bin/mksh",
    source_input = "/nix/store/baa9yl7sazygz4k4ma2v343n4aadjhlj-mksh-59c",
    expected_provenance = "cache-imported",
  },
  phases = [
    "producer-export",
    "frontend-free-plan",
    "trusted-cache-import",
    "strict-mantle-build",
  ],
  positive_evidence = [
    "foreign-realization-receipt-status-complete",
    "foreign-realization-route-cache-only-preserve-v1",
    "consumer-build-built-total-one",
    "consumer-output-builder-equals-imported-mksh",
    "host-store-builder-path-absent",
    "nix-command-absent-from-consumer-path",
  ],
  negative_cases = [
    "untrusted-cache-signature",
    "changed-nar-content",
    "missing-imported-pathinfo",
    "missing-runtime-closure-member",
    "ambient-builder-fallback",
  ],
  non_claims = [
    "not-mantle-built-mksh",
    "not-nix-evaluator-parity",
    "not-package-correctness",
    "not-reproducibility",
    "not-release-eligibility",
  ],
} | Workflow
