let OriginOctetFindings = 809 in
let CurrentOctetFindings = 803 in
let OriginTreeCopyFindings = 21 in
let CurrentTreeCopyFindings = 15 in
let CoreTestsPassed = 233 in
let ReleaseEvidenceTestsPassed = 44 in
let ReleaseTreeTestsPassed = 8 in
let FrontendTestsPassed = 7 in
let ReleaseCliTestsPassed = 149 in
let Receipt = {
  schema : String,
  change : String,
  completed_on : String,
  prerequisite : {
    repository : String,
    revision : String,
    establishment_archive : String,
  },
  dependency_admission : {
    cargo_source : String,
    nix_source : String,
    cargo_lock_blake3 : String,
    flake_lock_blake3 : String,
    sibling_path_fallback : Bool,
    github_fallback : Bool,
  },
  adapters : {
    release : Array String,
    frontend : Array String,
  },
  compatibility : {
    retained_identity_preimage : String,
    retained_path_order : String,
    retained_executable_semantics : String,
    shared_mechanism : Array String,
  },
  validation : {
    core_tests_passed : Number,
    release_evidence_tests_passed : Number,
    release_tree_tests_passed : Number,
    frontend_tests_passed : Number,
    release_cli_tests_passed : Number,
    core_clippy : String,
    octet : {
      verdict : String,
      acceptance_evidence : Bool,
      origin_findings : Number,
      current_findings : Number,
      origin_tree_copy_findings : Number,
      current_tree_copy_findings : Number,
      new_unbounded_growth_findings : Number,
    },
    nix_source_admission : String,
    nix_flake_evaluation : String,
    wasm_core_check : String,
  },
  rollback : {
    command : String,
    pre_adoption_revision : String,
  },
  non_claims : Array String,
} in
{
  schema = "mantle-bounded-tree-adoption-receipt-v1",
  change = "adopt-bounded-tree",
  completed_on = "2026-08-17",
  prerequisite = {
    repository = "rad:git:zqhtZvsteJhxCJE96dMAZSZ9y1PX",
    revision = "b0fd0103bc9eed2c1b6d852045959462d105d8f1",
    establishment_archive = "bounded-tree:cairn/archive/2026-08-15-establish-bounded-tree",
  },
  dependency_admission = {
    cargo_source = "https://seed.radicle.garden/zqhtZvsteJhxCJE96dMAZSZ9y1PX.git?rev=b0fd0103bc9eed2c1b6d852045959462d105d8f1",
    nix_source = "git+https://seed.radicle.garden/zqhtZvsteJhxCJE96dMAZSZ9y1PX.git?rev=b0fd0103bc9eed2c1b6d852045959462d105d8f1",
    cargo_lock_blake3 = "9639db5f3cd54d242b4d7db230bbc3e16e791fc1b2529fb3328ecab733584851",
    flake_lock_blake3 = "c5ae812e211c4bfbffc6f092df2b2948b16f6125a5009eed23312f4e383a3fcf",
    sibling_path_fallback = false,
    github_fallback = false,
  },
  adapters = {
    release = [
      "Mantle limits map to bounded-tree limits",
      "bounded-tree-cap owns observation, no-follow traversal, revalidation, and copy",
      "Mantle retains release DTOs, diagnostics, artifact identity, transport, and publication",
    ],
    frontend = [
      "one bounded-tree prepared observation feeds identity and copy",
      "source drift and external links fail before publication",
      "Mantle retains store paths, canonical tree preimage, and executable-bit meaning",
    ],
  },
  compatibility = {
    retained_identity_preimage = "mantle-tree-v1",
    retained_path_order = "UTF-8 relative-path string order",
    retained_executable_semantics = "regular files bind one normalized executable bit",
    shared_mechanism = [
      "tree-shape admission",
      "ordered member facts",
      "capability-relative observation",
      "source revalidation",
      "bounded copy execution",
    ],
  },
  validation = {
    core_tests_passed = CoreTestsPassed,
    release_evidence_tests_passed = ReleaseEvidenceTestsPassed,
    release_tree_tests_passed = ReleaseTreeTestsPassed,
    frontend_tests_passed = FrontendTestsPassed,
    release_cli_tests_passed = ReleaseCliTestsPassed,
    core_clippy = "passed with warnings denied",
    octet = {
      verdict = "warning-only; inherited repository posture improved but is not acceptance evidence",
      acceptance_evidence = false,
      origin_findings = OriginOctetFindings,
      current_findings = CurrentOctetFindings,
      origin_tree_copy_findings = OriginTreeCopyFindings,
      current_tree_copy_findings = CurrentTreeCopyFindings,
      new_unbounded_growth_findings = 0,
    },
    nix_source_admission = "passed",
    nix_flake_evaluation = "blocked by pre-existing unavailable SpaceWasm source and invalid cached derivation path",
    wasm_core_check = "blocked because the pinned standalone Rust sysroot lacks wasm32-unknown-unknown",
  },
  rollback = {
    command = "git revert <adopt-bounded-tree-completion-commit>",
    pre_adoption_revision = "20349655d5a890b063f6b9e9a488a65c313f0369",
  },
  non_claims = [
    "bounded-tree does not own Mantle artifact meaning",
    "bounded-tree does not own release publication or rollback authority",
    "bounded-tree does not prove source observation truth",
    "bounded-tree does not prove external storage durability",
    "Octet warning-only output is not clean acceptance evidence",
  ],
} | Receipt
