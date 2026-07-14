# Typed source of truth for Mantle's diagnostic-only NASA SpaceWasm cohort.
let non_empty = std.contract.from_predicate (fun value =>
  std.is_string value && std.string.length value > 0
) in
let blake3 = std.contract.from_predicate (fun value =>
  std.is_string value && std.string.is_match "^[0-9a-f]{64}$" value
) in
let git_revision = std.contract.from_predicate (fun value =>
  std.is_string value && std.string.is_match "^[0-9a-f]{40}$" value
) in
let positive = std.contract.from_predicate (fun value =>
  std.is_number value && value > 0
) in
let check_status = std.contract.from_predicate (fun value =>
  std.array.elem value ["passed", "failed", "skipped", "unavailable", "unsupported"]
) in
let support_status = std.contract.from_predicate (fun value =>
  std.array.elem value ["supported", "unsupported", "planned", "unreviewed"]
) in
let target_role = std.contract.from_predicate (fun value =>
  std.array.elem value ["host-library", "wasm-library", "host-diagnostic-runner"]
) in
let fixture_class = std.contract.from_predicate (fun value =>
  std.array.elem value [
    "mvp-positive",
    "mvp-negative",
    "streaming-positive",
    "streaming-negative",
    "allocation-failure",
    "unsupported-feature",
    "trap",
    "out-of-fuel",
  ]
) in
let corpus_role = std.contract.from_predicate (fun value =>
  std.array.elem value ["upstream-spectest", "upstream-fuzz"]
) in
let ProfileUpdate = {
  previous_review_revision | git_revision,
  selected_revision | git_revision,
  reason | non_empty,
  replay_required | Bool,
  replay_evidence_id | non_empty,
} in
let SourceProfile = {
  repository | non_empty,
  revision | git_revision,
  archive_url | non_empty,
  archive_sha256_sri | non_empty,
  archive_blake3 | blake3,
  cargo_lock_blake3 | blake3,
  dependency_manifest_blake3 | blake3,
  dependency_package_count | positive,
  octet_support_projection_blake3 | blake3,
  license_members | Array non_empty,
  notice_members | Array non_empty,
} in
let ToolchainProfile = {
  rust_channel | non_empty,
  rust_version | non_empty,
  host_triple | non_empty,
  wasm_triple | non_empty,
} in
let TargetProfile = {
  role | target_role,
  triple | non_empty,
  pointer_width_bits | positive,
  features | Array non_empty,
} in
let SupportEntry = {
  feature | non_empty,
  status | support_status,
} in
let RunnerProfile = {
  runner_id | non_empty,
  max_input_bytes | positive,
  max_stream_chunks | positive,
  default_fuel | positive,
  max_code_pages | positive,
  max_control_frames | positive,
  max_stack_words | positive,
} in
let FixtureProfile = {
  fixture_id | non_empty,
  class | fixture_class,
  artifact_path | non_empty,
  descriptor_path | non_empty,
  artifact_blake3 | blake3,
  descriptor_blake3 | blake3,
  generator_input_blake3 | blake3,
  expected_status | check_status,
  expected_code | non_empty,
} in
let CorpusProfile = {
  corpus_id | non_empty,
  role | corpus_role,
  descriptor_path | non_empty,
  source_path | non_empty,
  provenance | non_empty,
  license_path | non_empty,
  descriptor_blake3 | blake3,
  expected_status | check_status,
} in
let CheckProfile = {
  check_id | non_empty,
  expected_status | check_status,
} in
let RetentionProfile = {
  source_archive | Bool,
  dependency_closure | Bool,
  toolchain | Bool,
  binaries | Bool,
  fixtures | Bool,
  reports | Bool,
  licenses_and_notices | Bool,
  non_claims | Bool,
} in
let ReferenceBounds = {
  max_profile_text_bytes | positive,
  max_collection_items | positive,
  max_bundle_members | positive,
  max_parent_edges | positive,
  max_bundle_member_bytes | positive,
  max_bundle_total_bytes | positive,
} in
let ReferenceProfile = {
  schema | std.contract.from_predicate (fun value => value == "mantle-spacewasm-reference-profile-v1"),
  profile_id | non_empty,
  profile_update | ProfileUpdate,
  source | SourceProfile,
  toolchain | ToolchainProfile,
  targets | Array TargetProfile,
  support_matrix | Array SupportEntry,
  requested_features | Array non_empty,
  runner | RunnerProfile,
  fixtures | Array FixtureProfile,
  corpora | Array CorpusProfile,
  checks | Array CheckProfile,
  retention | RetentionProfile,
  bounds | ReferenceBounds,
  non_claims | Array non_empty,
} in
let proposal_review_revision = "30cd6e9b91f84a39278edcb5d66514b773011ccc" in
let cohort_revision = "e24cf09355a90497148eb5029fdb8e3400bd63e3" in
let host_pointer_width = 64 in
let wasm_pointer_width = 32 in
let dependency_packages = 53 in
let profile_text_bytes_limit = 262144 in
let collection_items_limit = 128 in
let bundle_members_limit = 512 in
let parent_edges_limit = 2048 in
let bundle_member_bytes_limit = 536870912 in
let bundle_total_bytes_limit = 2147483648 in
let runner_input_bytes_limit = 1048576 in
let stream_chunks_limit = 4096 in
let runner_default_fuel = 1024 in
let code_pages_limit = 64 in
let control_frames_limit = 64 in
let stack_words_limit = 1024 in
let target = fun input_role input_triple input_pointer_width => {
  role = input_role,
  triple = input_triple,
  pointer_width_bits = input_pointer_width,
  features = ["default-features-disabled"],
} in
let support = fun input_feature input_status => { feature = input_feature, status = input_status } in
let fixture = fun input_id input_class input_artifact_path input_descriptor_path input_artifact_blake3 input_descriptor_blake3 input_generator_blake3 input_expected_code => {
  fixture_id = input_id,
  class = input_class,
  artifact_path = input_artifact_path,
  descriptor_path = input_descriptor_path,
  artifact_blake3 = input_artifact_blake3,
  descriptor_blake3 = input_descriptor_blake3,
  generator_input_blake3 = input_generator_blake3,
  expected_status = "passed",
  expected_code = input_expected_code,
} in
let check = fun input_id input_status => { check_id = input_id, expected_status = input_status } in
{
  schema = "mantle-spacewasm-reference-profile-v1",
  profile_id = "nasa-spacewasm-e24cf093-reference-v1",
  profile_update = {
    previous_review_revision = proposal_review_revision,
    selected_revision = cohort_revision,
    reason = "Update the proposal review commit to Octet's currently reviewed spacewasm-mvp cohort and replay every declared build, fixture, corpus, and bundle check.",
    replay_required = true,
    replay_evidence_id = "spacewasm-e24-complete-fixture-replay-v1",
  },
  source = {
    repository = "https://github.com/nasa/spacewasm",
    revision = cohort_revision,
    archive_url = "https://github.com/nasa/spacewasm/archive/e24cf09355a90497148eb5029fdb8e3400bd63e3.tar.gz",
    archive_sha256_sri = "sha256-iNgipIlAPOJU38oOqvqxXQF15FJ0ooN3rH5DzHiKOTU=",
    archive_blake3 = "0db57636fb83a8c47f0c55601236cab648dd5edfdd2567cae0619aa74b8c4da3",
    cargo_lock_blake3 = "4e6a07910125d4c921a62cec03d8bc9acb3bf341ebd1a11999df08d6c6c5c5f1",
    dependency_manifest_blake3 = "d9f51122ebe3fbac4b9e0f05795f64ce83cd41baad27e66cf9d42c52cb5b5d06",
    dependency_package_count = dependency_packages,
    octet_support_projection_blake3 = "7f3c40650c576bf9340d07df5371da28fad42ef4937d1d9ce9c2c8bc80a5b638",
    license_members = ["LICENSE", "licenses/wasmedge-spectest-MIT.txt"],
    notice_members = ["NOTICE"],
  },
  toolchain = {
    rust_channel = "stable",
    rust_version = "1.91.1",
    host_triple = "x86_64-unknown-linux-gnu",
    wasm_triple = "wasm32-unknown-unknown",
  },
  targets = [
    target "host-library" "x86_64-unknown-linux-gnu" host_pointer_width,
    target "wasm-library" "wasm32-unknown-unknown" wasm_pointer_width,
    target "host-diagnostic-runner" "x86_64-unknown-linux-gnu" host_pointer_width,
  ],
  support_matrix = [
    support "wasm1" "supported",
    support "mutable-globals" "supported",
    support "custom-page-sizes" "planned",
    support "bulk-memory" "planned",
    support "sign-extension-ops" "planned",
    support "non-trapping-float-to-int" "planned",
    support "multi-value" "unreviewed",
    support "multi-memory" "unreviewed",
    support "component-model" "unsupported",
    support "memory64" "unsupported",
    support "threads" "unsupported",
    support "wit" "unsupported",
    support "wasi" "unsupported",
  ],
  requested_features = ["wasm1", "mutable-globals"],
  runner = {
    runner_id = "mantle-spacewasm-diagnostic-runner-v1",
    max_input_bytes = runner_input_bytes_limit,
    max_stream_chunks = stream_chunks_limit,
    default_fuel = runner_default_fuel,
    max_code_pages = code_pages_limit,
    max_control_frames = control_frames_limit,
    max_stack_words = stack_words_limit,
  },
  fixtures = [
    fixture "mvp-positive" "mvp-positive" "fixtures/wasm/mvp-positive.wasm" "fixtures/descriptors/mvp-positive.json" "b2568cffcad3c442a90dd65ccb82d6b7e285bf71c2c9908ce539b6b515217bde" "6e2008fc3abaf41ec2af9c2d9e824829aa11d2d8b1b394cb9b40a8edb5b67af8" "95a86baa63e72211ccbe0d812937694e78a4b4d5e1fa616567d5c31335d80177" "finished",
    fixture "mvp-negative" "mvp-negative" "fixtures/wasm/mvp-negative.wasm" "fixtures/descriptors/mvp-negative.json" "a65dcf6433860fed3ae87310f5f01cf5d3abb75f5275112b7131f44e165ba6df" "bd8294c8c3bfbe78c0957d50c326ad25283729493234c7f3d3588e36efbb4485" "b2568cffcad3c442a90dd65ccb82d6b7e285bf71c2c9908ce539b6b515217bde" "malformed-magic",
    fixture "streaming-positive" "streaming-positive" "fixtures/wasm/streaming-positive.wasm" "fixtures/descriptors/streaming-positive.json" "b2568cffcad3c442a90dd65ccb82d6b7e285bf71c2c9908ce539b6b515217bde" "a1756e64451488a3b58c1055e3ac28e4154c5f60ef921966d3f88bef82f588ed" "b2568cffcad3c442a90dd65ccb82d6b7e285bf71c2c9908ce539b6b515217bde" "stream-finished",
    fixture "streaming-negative" "streaming-negative" "fixtures/wasm/streaming-negative.wasm" "fixtures/descriptors/streaming-negative.json" "94a979b596cf073331cf2954de8c15fb504749758c464d0201ae799a451e02dc" "fd9809a13bb0cdc2c61c45ab84bdcb65f3b0af9bf0a802a35bf37bfda2ecf966" "b2568cffcad3c442a90dd65ccb82d6b7e285bf71c2c9908ce539b6b515217bde" "stream-unexpected-eof",
    fixture "allocation-failure" "allocation-failure" "fixtures/wasm/allocation-failure.wasm" "fixtures/descriptors/allocation-failure.json" "b2568cffcad3c442a90dd65ccb82d6b7e285bf71c2c9908ce539b6b515217bde" "8af948702be742eafcac1e6ca8a4a234e6a80ad27e3918c1f6fa97221854eaa1" "95a86baa63e72211ccbe0d812937694e78a4b4d5e1fa616567d5c31335d80177" "allocation-failed-without-panic",
    fixture "unsupported-feature" "unsupported-feature" "fixtures/wasm/unsupported-bulk-memory.wasm" "fixtures/descriptors/unsupported-feature.json" "57a9e00325ec098e2ca8f7b7deb5b9659ccdcbce84d862ef60ea5f16e05e1624" "a350734b0166cce06244c1eaf9f3f4713ad02edbda2f7a63517e38f1d8f3283e" "be296033d6d2328e2ead458a66e2aefe76fee7c3a8b8dbca3315bcc0522fa91c" "unsupported-bulk-memory",
    fixture "trap" "trap" "fixtures/wasm/trap-unreachable.wasm" "fixtures/descriptors/trap.json" "8dadfe79ddcae5a3c738f1ad24e812a2beb9a95c88702b4bad896fa1c8332c0a" "30ca5786c9b5654017d979efc50100435ebaeaf2bfb588f06b838fe5d49836fd" "b28ed13daf132eaf5259e1499276bd46c3cf85add27402d89bff854701caf71d" "trap-unreachable",
    fixture "out-of-fuel" "out-of-fuel" "fixtures/wasm/out-of-fuel.wasm" "fixtures/descriptors/out-of-fuel.json" "3abd1aeeb22b89a284cc1a510d6f872a2a60f0bd61d43b640e1d0260d315eecb" "a3a1cfbcaf18ffab731bd2f6711982144ff4df03a70e331c7ea184788592310d" "129fa3bd5c3590311cffe329576b37344c0168c2943170c41e000a921d80a083" "out-of-fuel",
  ],
  corpora = [
    {
      corpus_id = "upstream-spectest",
      role = "upstream-spectest",
      descriptor_path = "corpora/spectest.json",
      source_path = "tests/core",
      provenance = "SpaceWasm source archive at the selected revision, curated from WasmEdge/wasmedge-spectest",
      license_path = "licenses/wasmedge-spectest-MIT.txt",
      descriptor_blake3 = "5138df4c6702c8bbccedffdf17a83e092658625ed8a9a5364d5422e454aa6ffc",
      expected_status = "passed",
    },
    {
      corpus_id = "upstream-fuzz",
      role = "upstream-fuzz",
      descriptor_path = "corpora/fuzz.json",
      source_path = "fuzz/fuzz_targets",
      provenance = "NASA SpaceWasm source archive at the selected revision",
      license_path = "LICENSE",
      descriptor_blake3 = "3fa7449e505a2fd685337a31a7b8be6b0c9afa89508ee4b31842c82a533644d8",
      expected_status = "skipped",
    },
  ],
  checks = [
    check "host-library-build" "passed",
    check "wasm-library-build" "passed",
    check "host-diagnostic-runner-build" "passed",
    check "upstream-unit-tests" "passed",
    check "upstream-spectest-address" "passed",
    check "fixture-replay" "passed",
    check "full-spectest-suite" "skipped",
    check "spacewasm-check" "unavailable",
    check "continuous-fuzzing" "unavailable",
    check "post-mvp-feature-build" "unsupported",
  ],
  retention = {
    source_archive = true,
    dependency_closure = true,
    toolchain = true,
    binaries = true,
    fixtures = true,
    reports = true,
    licenses_and_notices = true,
    non_claims = true,
  },
  bounds = {
    max_profile_text_bytes = profile_text_bytes_limit,
    max_collection_items = collection_items_limit,
    max_bundle_members = bundle_members_limit,
    max_parent_edges = parent_edges_limit,
    max_bundle_member_bytes = bundle_member_bytes_limit,
    max_bundle_total_bytes = bundle_total_bytes_limit,
  },
  non_claims = [
    "not-spacewasm-correctness",
    "not-memory-safety",
    "not-webassembly-conformance",
    "not-flight-qualification",
    "not-sandbox-effectiveness",
    "not-consumer-runtime-admission",
    "not-production-readiness",
    "not-release-eligibility",
  ],
} | ReferenceProfile
