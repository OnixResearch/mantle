let Contracts = import "contracts.ncl" in

{
  schema = "mantle-cc-cache-policy-v2",
  driver = {
    # No production inventory or proof-lane admission is wired yet.
    mode = "off",
    admitted_compiler_families = ["x86_64-linux-musl-*", "stage-tinycc", "stage-gcc"],
    driver_path = "",
    compiler_path = "",
    socket_path = "",
    receipt_path = "",
    driver_digest_blake3 = "",
    compiler_digest_blake3 = "",
    platform_digest = "",
    ambient_path_mutation = false,
    unclassified = "forward-exact-arguments",
    daemon_unavailable = "forward-exact-arguments",
  },
  identity = {
    hash_algorithm = "BLAKE3",
    tool = "executable-content-digest",
    source = "source-content-digest",
    platform = "explicit-platform-digest",
    arguments = "normalized-no-store-paths",
    roots = "whole-root-tree-digest",
    dependencies = "depfile-root-relative-label-and-content-digest",
    probe_script = "canonical-regular-script-content-digest",
    unknown_manifest = "miss-no-replay",
  },
  limits = {
    max_object_bytes = 4194304,
    max_frame_bytes = 8388608,
    max_roots = 128,
    max_dependencies = 8192,
    max_arguments = 4096,
  },
  daemon = {
    transport = "per-user-unix-socket",
    framing = "8-byte-length-prefixed-bounded-json",
    peer_authority = "same-user-uid",
    state = "private-rust-unit-cache",
    storage = "local-content-addressed-object-file",
    operations = ["manifest", "read", "publish"],
  },
  receipts = {
    dispositions = ["hit", "published", "compiler-only", "compiler-failure", "fallback", "rejected", "probe-failure-hit", "probe-failure-published", "probe-compiler-failure"],
    probe_read_dispositions = ["hit", "miss", "invalid", "unavailable"],
    proof_evidence = false,
  },
  probe = {
    # Available only after explicit on-mode daemon admission and --probe-script.
    invocation = "explicit-probe-script-classified-compiler",
    script_identity = "canonical-regular-script-content-digest",
    toolchain_identity = "compiler-executable-content-digest",
    dependency_identity = "complete-depfile-and-root-tree",
    platform_identity = "explicit-platform-digest",
    flags_identity = "normalized-semantic-arguments",
    environment = "LC_ALL=C-only",
    failure_replay = "complete-manifest-bounded-diagnostics-only",
    max_diagnostic_bytes = 262144,
  },
  proof = {
    strict_lane_cache = "off-or-independent-content-verification",
    transcript_cache_mode_required = true,
    non_claims = [
      "cache-hit-is-not-correctness-evidence",
      "content-hash-is-not-writer-trust",
      "cache-receipt-is-not-proof-evidence",
      "no-byte-parity-or-fixed-point-proof-yet",
      "bootstrap-inventory-admission-not-wired",
    ],
  },
} | Contracts.Policy
