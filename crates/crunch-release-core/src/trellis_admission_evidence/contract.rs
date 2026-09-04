pub const TRELLIS_ADMISSION_EVIDENCE_SCHEMA: &str = "mantle-trellis-remote-admission-evidence-v1";
pub const TRELLIS_ADMISSION_REVISION: &str = "8de4b24aa2d66cc2e6ec966d686df023492265d3";
pub const TRELLIS_ADMISSION_SOURCE_BLAKE3: &str = "e13e9f71da4964ab4d4f04d9c29525b20f0e778997da5ada7b6caa0f1711f56c";
pub const TRELLIS_ADMISSION_MODEL_TREE_OID: &str = "91dace31060cf5195beb72dba71640d9091e67bf";
pub const TRELLIS_ADMISSION_ORACLE_CASES: u32 = 6_720;
pub const TRELLIS_ADMISSION_CLAIM: &str =
    "the named abstract fenced-attempt safety properties agree for the recorded supported projection cases";
pub const TRELLIS_ADMISSION_RUNTIME_AUTHORITY: &str = "unchanged-plan_remote_attempt_report";
pub const TRELLIS_ADMISSION_NON_CLAIMS: &[&str] = &[
    "not full implementation equivalence",
    "not persistence atomicity",
    "not transport reliability",
    "not cryptographic correctness",
    "not remote worker correctness",
    "not liveness or availability",
    "not whole-build correctness",
    "not release eligibility",
];

pub(super) const TRELLIS_REPOSITORY: &str = "github.com/OnixResearch/trellis";
pub(super) const TRELLIS_SOURCE_ARCHIVE_BYTES: u64 = 61_440;
pub(super) const KAMACITE_REVISION: &str = "de710a092d351e829abfb288d46124e2db8e5b7f";
pub(super) const KAMACITE_PROFILE: &str = "kamacite.trellis-proof-evidence-profile.v1";
pub(super) const VALENCE_REVISION: &str = "27b8b2124e12b80718ded124274fec98bed7a581";
pub(super) const VALENCE_SCHEMA: &str = "trellis.proof-evidence";
pub(super) const EXPECTED_PROPERTY_COUNT: usize = 8;
pub(super) const EXPECTED_REQUIREMENT_COUNT: usize = 8;
pub(super) const ASSUMPTION_COUNT_MAX: usize = 16;
pub(super) const BLAKE3_HEX_LENGTH: usize = 64;
pub(super) const REVISION_HEX_LENGTH: usize = 40;
pub(super) const DIAGNOSTIC_COUNT_MAX: usize = 64;
