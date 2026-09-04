# Reviewable policy for the external Trellis fenced-attempt evidence seam.
let NonEmpty = std.contract.from_predicate (fun value =>
  std.is_string value && std.string.length value > 0
) in
let Blake3 = std.contract.from_predicate (fun value =>
  std.is_string value && std.string.is_match "^[0-9a-f]{64}$" value
) in
let Revision = std.contract.from_predicate (fun value =>
  std.is_string value && std.string.is_match "^[0-9a-f]{40}$" value
) in
let Positive = std.contract.from_predicate (fun value =>
  std.is_number value && value > 0
) in
let Source = {
  repository | std.contract.Equal "github.com/OnixResearch/trellis",
  revision | Revision,
  model_tree_oid | Revision,
  archive_blake3 | Blake3,
  archive_format | std.contract.Equal "git-archive-tar",
  scope | Array NonEmpty,
} in
let Oracle = {
  schema | std.contract.Equal "mantle-trellis-remote-admission-oracle-v1",
  case_count | Positive,
  record_bytes | Positive,
  matrix_blake3 | Blake3,
  unsupported_reasons | Array NonEmpty,
} in
let EvidenceProfile = {
  kamacite_revision | Revision,
  kamacite_profile | std.contract.Equal "kamacite.trellis-proof-evidence-profile.v1",
  valence_revision | Revision,
  valence_schema | std.contract.Equal "trellis.proof-evidence",
  producer_role | std.contract.Equal "formal-proof-candidate",
  validation_role | std.contract.Equal "property",
} in
let Profile = {
  schema | std.contract.Equal "mantle-trellis-remote-admission-profile-v1",
  source | Source,
  modeled_state_fields | Array NonEmpty,
  modeled_report_fields | Array NonEmpty,
  property_ids | Array NonEmpty,
  requirement_ids | Array NonEmpty,
  oracle | Oracle,
  evidence_profile | EvidenceProfile,
  non_claims | Array NonEmpty,
} in

let oracle_case_count = 6720 in
let oracle_record_bytes = 7 in
{
  schema = "mantle-trellis-remote-admission-profile-v1",
  source = {
    repository = "github.com/OnixResearch/trellis",
    revision = "8de4b24aa2d66cc2e6ec966d686df023492265d3",
    model_tree_oid = "91dace31060cf5195beb72dba71640d9091e67bf",
    archive_blake3 = "e13e9f71da4964ab4d4f04d9c29525b20f0e778997da5ada7b6caa0f1711f56c",
    archive_format = "git-archive-tar",
    scope = ["src/fenced_attempt"],
  },
  modeled_state_fields = [
    "job",
    "run",
    "epoch",
    "phase",
    "progress",
    "events",
    "result",
  ],
  modeled_report_fields = [
    "job",
    "run",
    "epoch",
    "event",
    "payload",
    "kind",
    "worker",
    "output",
    "result",
  ],
  property_ids = [
    "current_epoch_required",
    "reassignment_advances_epoch",
    "duplicate_event_is_noop",
    "event_conflict_rejected",
    "terminal_phase_closed",
    "completion_requires_linkage",
    "rejection_preserves_state",
    "exec_refines_spec",
  ],
  requirement_ids = [
    "r[fenced-attempt-admission.boundary]",
    "r[fenced-attempt-admission.refinement]",
    "r[fenced-attempt-admission.fence-safety]",
    "r[fenced-attempt-admission.event-idempotence]",
    "r[fenced-attempt-admission.terminal-closure]",
    "r[fenced-attempt-admission.completion-linkage]",
    "r[fenced-attempt-admission.rejection-preservation]",
    "r[fenced-attempt-admission.evidence]",
  ],
  oracle = {
    schema = "mantle-trellis-remote-admission-oracle-v1",
    case_count = oracle_case_count,
    record_bytes = oracle_record_bytes,
    matrix_blake3 = "bd1e98e26dbd43f44189941c1eac22bd63139d031c2ea328f0c2d5ce8582aa32",
    unsupported_reasons = [
      "authority-meaning-mismatch",
      "authority-order-mismatch",
      "history-order-mismatch",
      "phase-report-mismatch",
      "result-retention-mismatch",
    ],
  },
  evidence_profile = {
    kamacite_revision = "de710a092d351e829abfb288d46124e2db8e5b7f",
    kamacite_profile = "kamacite.trellis-proof-evidence-profile.v1",
    valence_revision = "27b8b2124e12b80718ded124274fec98bed7a581",
    valence_schema = "trellis.proof-evidence",
    producer_role = "formal-proof-candidate",
    validation_role = "property",
  },
  non_claims = [
    "not full implementation equivalence",
    "not persistence atomicity",
    "not transport reliability",
    "not cryptographic correctness",
    "not remote worker correctness",
    "not liveness or availability",
    "not whole-build correctness",
    "not release eligibility",
  ],
} | Profile
