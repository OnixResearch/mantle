# Admission policy, not evidence that a bootstrap proof has run.
let Exact = fun expected => std.contract.from_predicate (fun value => value == expected) in
let Bounded = fun maximum => std.contract.from_predicate (fun value =>
  std.is_number value && std.number.is_integer value && value > 0 && value <= maximum
) in
let Mode = std.contract.from_predicate (fun value =>
  std.is_string value &&
  (value == "off" || value == "local-read" || value == "local-read-write")
) in
let Driver = {
  mode | Mode,
  admitted_compiler_families | Exact ["x86_64-linux-musl-*", "stage-tinycc", "stage-gcc"],
  driver_path | String,
  compiler_path | String,
  socket_path | String,
  receipt_path | String,
  driver_digest_blake3 | String,
  compiler_digest_blake3 | String,
  platform_digest | String,
  ambient_path_mutation | Exact false,
  unclassified | Exact "forward-exact-arguments",
  daemon_unavailable | Exact "forward-exact-arguments",
} in
let AbsolutePath = fun value =>
  std.string.is_match "^/" value && !std.string.is_match "(^|/)\\.\\.(/|$)" value
in
let Digest = fun value => std.string.is_match "^[0-9a-f]{64}$" value in
let DriverAdmission = std.contract.from_predicate (fun value =>
  if value.mode == "off" then
    value.driver_path == "" && value.compiler_path == "" &&
    value.socket_path == "" && value.receipt_path == "" &&
    value.driver_digest_blake3 == "" &&
    value.compiler_digest_blake3 == "" && value.platform_digest == ""
  else
    AbsolutePath value.driver_path && AbsolutePath value.compiler_path &&
    AbsolutePath value.socket_path && AbsolutePath value.receipt_path &&
    Digest value.driver_digest_blake3 &&
    Digest value.compiler_digest_blake3 && Digest value.platform_digest
) in
let Identity = {
  hash_algorithm | Exact "BLAKE3",
  tool | Exact "executable-content-digest",
  source | Exact "source-content-digest",
  platform | Exact "explicit-platform-digest",
  arguments | Exact "normalized-no-store-paths",
  roots | Exact "whole-root-tree-digest",
  dependencies | Exact "depfile-root-relative-label-and-content-digest",
  probe_script | Exact "canonical-regular-script-content-digest",
  unknown_manifest | Exact "miss-no-replay",
} in
let Limits = {
  max_object_bytes | Bounded 4194304,
  max_frame_bytes | Bounded 8388608,
  max_roots | Bounded 128,
  max_dependencies | Bounded 8192,
  max_arguments | Bounded 4096,
} in
let Daemon = {
  transport | Exact "per-user-unix-socket",
  framing | Exact "8-byte-length-prefixed-bounded-json",
  peer_authority | Exact "same-user-uid",
  state | Exact "private-rust-unit-cache",
  storage | Exact "local-content-addressed-object-file",
  operations | Exact ["manifest", "read", "publish"],
} in
let Receipts = {
  dispositions | Exact ["hit", "published", "compiler-only", "compiler-failure", "fallback", "rejected", "probe-failure-hit", "probe-failure-published", "probe-compiler-failure"],
  probe_read_dispositions | Exact ["hit", "miss", "invalid", "unavailable"],
  proof_evidence | Exact false,
} in
let Probe = {
  invocation | Exact "explicit-probe-script-classified-compiler",
  script_identity | Exact "canonical-regular-script-content-digest",
  toolchain_identity | Exact "compiler-executable-content-digest",
  dependency_identity | Exact "complete-depfile-and-root-tree",
  platform_identity | Exact "explicit-platform-digest",
  flags_identity | Exact "normalized-semantic-arguments",
  environment | Exact "LC_ALL=C-only",
  failure_replay | Exact "complete-manifest-bounded-diagnostics-only",
  max_diagnostic_bytes | Bounded 262144,
} in
let Proof = {
  strict_lane_cache | Exact "off-or-independent-content-verification",
  transcript_cache_mode_required | Exact true,
  non_claims | Exact [
    "cache-hit-is-not-correctness-evidence",
    "content-hash-is-not-writer-trust",
    "cache-receipt-is-not-proof-evidence",
    "no-byte-parity-or-fixed-point-proof-yet",
    "bootstrap-inventory-admission-not-wired",
  ],
} in
{
  ModePolicy = Mode,
  DriverShape = Driver,
  DriverPolicy = DriverAdmission,
  IdentityPolicy = Identity,
  LimitsPolicy = Limits,
  DaemonPolicy = Daemon,
  ReceiptsPolicy = Receipts,
  ProbePolicy = Probe,
  ProofPolicy = Proof,
  Policy = {
    schema | Exact "mantle-cc-cache-policy-v2",
    driver | Driver | DriverAdmission,
    identity | Identity,
    limits | Limits,
    daemon | Daemon,
    receipts | Receipts,
    probe | Probe,
    proof | Proof,
  },
}
