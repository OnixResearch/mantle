use std::path::Path;

use crate::early_native_row_receipt::RowExpectation;
use crate::early_native_row_receipt::SourcePolicyExpectation;
use crate::early_native_row_receipt_shell::validate_row;

const FINAL_NATIVE_ROW_RECEIPT_SCHEMA: &str = "mantle-final-native-row-receipt-v1";
const FINAL_NATIVE_ARTIFACT_EVIDENCE_SCHEMA: &str = "mantle-final-native-artifact-evidence-v1";
const FINAL_NATIVE_ACCEPTANCE_EVIDENCE_SCHEMA: &str = "mantle-final-native-row-acceptance-v1";

const GCC47_RECEIPT_PATH: &str = "bootstrap/evidence/final-native-gcc47-row-v1.json";
const GCC47_ARTIFACT_PATH: &str = "bootstrap/evidence/final-native-gcc47-artifact.json";
const GCC47_ACCEPTANCE_PATH: &str = "bootstrap/evidence/final-native-gcc47-acceptance-v1.json";
const GCC47_SUCCESS_MARKER: &str =
    "GCC 4.7.4 final-native row C/C++11 runtime, relocation, and rejection matrix passed";
const GCC47_SOURCES: &[&str] = &[
    "bootstrap/gcc-4.7.ncl",
    "bootstrap/autoconf-2.64-gcc.ncl",
    "bootstrap/bison-3.4.1-gcc.ncl",
    "bootstrap/flex-2.6.4-gcc.ncl",
    "bootstrap/gmp-6.2.1.ncl",
    "bootstrap/mpfr-4.1.0.ncl",
    "bootstrap/mpc-1.2.1.ncl",
];
const GCC47_PREDECESSORS: &[&str] = &[
    "gcc40", "binutils", "musl", "gmp", "mpfr", "mpc", "autoconf", "bison", "flex",
];
const GCC47_PREDECESSOR_EVIDENCE: &[&str] = &[
    "bootstrap/evidence/final-native-predecessors/gcc40.json",
    "bootstrap/evidence/final-native-predecessors/early-binutils.json",
    "bootstrap/evidence/final-native-predecessors/musl-pass5.json",
    "bootstrap/evidence/final-native-predecessors/gmp.json",
    "bootstrap/evidence/final-native-predecessors/mpfr.json",
    "bootstrap/evidence/final-native-predecessors/mpc.json",
    "bootstrap/evidence/final-native-predecessors/autoconf.json",
    "bootstrap/evidence/final-native-predecessors/bison.json",
    "bootstrap/evidence/final-native-predecessors/flex.json",
];
const GCC47_GENERATED: &[&str] = &[
    "configure-set-autoconf-2.64",
    "config-header-set-autoheader-2.64",
    "intl-plural-bison-3.4.1",
    "gengtype-scanner-flex-2.6.4",
    "config-sub-automake-1.15.1",
];
const GCC47_POSITIVE: &[&str] = &[
    "c-static-runtime",
    "cc1-and-cc1plus-present",
    "cxx11-exception-rtti-allocation-runtime",
    "libgcc-libstdcxx-libsupcxx-present",
    "relocated-c-runtime",
];
const GCC47_REJECTION: &[&str] = &[
    "malformed-c-rejected",
    "malformed-cxx-rejected",
    "ambient-path-scan-clean",
    "host-compiler-scan-clean",
    "release-generated-substitution-scan-clean",
    "wrong-predecessor-scan-clean",
];
const GCC47_POLICY_REQUIRED: &[&str] = &[
    "final-native-gcc47-row.txt",
    "accepted malformed C",
    "accepted malformed C++",
    "gcc47-regenerated-files.txt",
];
const GCC47_POLICY_FORBIDDEN: &[&str] = &[
    "$STAGE0/bin:$PATH",
    "command -v ",
    "/usr/bin/",
    "/home/",
    ".pi/cairn-drain/",
    "NON_ADMISSION",
];
const GCC47_SOURCE_POLICIES: &[SourcePolicyExpectation] = &[SourcePolicyExpectation {
    path: "bootstrap/gcc-4.7.ncl",
    required: GCC47_POLICY_REQUIRED,
    forbidden: GCC47_POLICY_FORBIDDEN,
}];

const GCC10_RECEIPT_PATH: &str = "bootstrap/evidence/final-native-gcc10-row-v1.json";
const GCC10_ARTIFACT_PATH: &str = "bootstrap/evidence/final-native-gcc10-artifact.json";
const GCC10_ACCEPTANCE_PATH: &str = "bootstrap/evidence/final-native-gcc10-acceptance-v1.json";
const GCC10_SUCCESS_MARKER: &str =
    "GCC 10.5.0 final-native row C/C++17 runtime, ELF, relocation, and rejection matrix passed";
const GCC10_SOURCES: &[&str] = &["bootstrap/gcc-10.ncl", "bootstrap/gcc-10.5.0-regenerated-source.ncl"];
const GCC10_PREDECESSORS: &[&str] = &["gcc47", "binutils", "musl", "regenerated-source", "gmp", "mpfr", "mpc"];
const GCC10_PREDECESSOR_EVIDENCE: &[&str] = &[
    "bootstrap/evidence/final-native-predecessors/gcc47.json",
    "bootstrap/evidence/final-native-predecessors/early-binutils.json",
    "bootstrap/evidence/final-native-predecessors/musl-pass5.json",
    "bootstrap/evidence/final-native-predecessors/gcc10-regenerated-source.json",
    "bootstrap/evidence/final-native-predecessors/gmp.json",
    "bootstrap/evidence/final-native-predecessors/mpfr.json",
    "bootstrap/evidence/final-native-predecessors/mpc.json",
];
const GCC10_GENERATED: &[&str] = &[
    "configure-and-header-set-autoconf-2.69",
    "intl-plural-bison-3.4.1",
    "gengtype-scanner-flex-2.6.4",
];
const GCC10_POSITIVE: &[&str] = &[
    "c-static-runtime",
    "cc1-and-cc1plus-present",
    "cxx17-exception-rtti-allocation-runtime",
    "libgcc-libstdcxx-libsupcxx-present",
    "regenerated-source-manifest-present",
    "relocated-cxx-runtime",
];
const GCC10_REJECTION: &[&str] = &[
    "malformed-c-rejected",
    "malformed-cxx-rejected",
    "ambient-path-scan-clean",
    "host-compiler-scan-clean",
    "release-generated-substitution-scan-clean",
    "wrong-predecessor-scan-clean",
];
const GCC10_POLICY_REQUIRED: &[&str] = &[
    "final-native-gcc10-row.txt",
    "accepted malformed C",
    "accepted malformed C++",
    "gcc-10.5.0-files.txt",
];
const GCC10_POLICY_FORBIDDEN: &[&str] = &[
    "$STAGE0/bin:$PATH",
    "command -v ",
    "/usr/bin/",
    "/home/",
    ".pi/cairn-drain/",
    "NON_ADMISSION",
];
const GCC10_SOURCE_POLICIES: &[SourcePolicyExpectation] = &[SourcePolicyExpectation {
    path: "bootstrap/gcc-10.ncl",
    required: GCC10_POLICY_REQUIRED,
    forbidden: GCC10_POLICY_FORBIDDEN,
}];

const FINAL_RECEIPT_PATH: &str = "bootstrap/evidence/final-native-musl-binutils-row-v1.json";
const FINAL_ARTIFACT_PATH: &str = "bootstrap/evidence/final-native-binutils-artifact.json";
const FINAL_ACCEPTANCE_PATH: &str = "bootstrap/evidence/final-native-musl-binutils-acceptance-v1.json";
const FINAL_SUCCESS_MARKER: &str =
    "binutils 2.41 final-native assembler/linker/archive/object runtime, relocation, and rejection matrix passed";
const FINAL_SOURCES: &[&str] = &[
    "bootstrap/gcc-10-final.ncl",
    "bootstrap/gcc-10.5.0-regenerated-source.ncl",
    "bootstrap/musl-full.ncl",
    "bootstrap/binutils-full.ncl",
    "bootstrap/binutils-2.41-regeneration-ready-source.ncl",
];
const FINAL_PREDECESSORS: &[&str] = &["gcc10", "gcc10-final-cycle", "musl-final"];
const FINAL_PREDECESSOR_EVIDENCE: &[&str] = &[
    "bootstrap/evidence/final-native-predecessors/gcc10.json",
    "bootstrap/evidence/final-native-predecessors/gcc10-final.json",
    "bootstrap/evidence/final-native-predecessors/musl-final.json",
];
const FINAL_GENERATED: &[&str] = &[
    "gcc10-final-regenerated-source-set",
    "binutils-configure-set-autoconf-2.69",
    "binutils-parser-scanner-set-bison-flex",
];
const FINAL_POSITIVE: &[&str] = &[
    "gcc10-static-cxx17-runtime",
    "gcc10-dynamic-cxx17-runtime",
    "gcc10-shared-runtime-members",
    "gcc10-relocated-runtime",
    "musl-header-crt-static-shared-interpreter-set",
    "musl-static-runtime",
    "musl-dynamic-runtime",
    "musl-copied-tree-runtime",
    "binutils-tool-set",
    "binutils-runtime-and-relocation",
    "binutils-copied-tree-runtime",
];
const FINAL_REJECTION: &[&str] = &[
    "malformed-cxx-rejected",
    "absolute-shared-runtime-dependency-rejected",
    "malformed-c-rejected",
    "malformed-assembly-rejected",
    "malformed-object-rejected",
    "malformed-archive-rejected",
    "undefined-symbol-link-rejected",
    "ambient-path-scan-clean",
    "state-pinned-input-scan-clean",
    "missing-runtime-member-scan-clean",
];
const GCC10_FINAL_POLICY_REQUIRED: &[&str] = &[
    "final-native-gcc10-cycle.txt",
    "accepted malformed C++",
    "absolute libc dependency",
];
const MUSL_POLICY_REQUIRED: &[&str] = &[
    "final-native-musl-row.txt",
    "accepted malformed C",
    "ld-musl-x86_64.so.1",
    "RELOCATED_MUSL=\"$WORK/relocated-musl\"",
];
const BINUTILS_POLICY_REQUIRED: &[&str] = &[
    "final-native-binutils-row.txt",
    "expect_bounded_rejection \"malformed assembly\"",
    "expect_bounded_rejection \"malformed object inspection\"",
    "expect_bounded_rejection \"malformed archive\"",
    "expect_bounded_rejection \"undefined-symbol link\"",
    "binutils-2.41-removed-generated-files.txt",
];
const FINAL_POLICY_FORBIDDEN: &[&str] = &[
    "$STAGE0/bin:$PATH",
    "command -v ",
    "/usr/bin/",
    "/home/",
    ".pi/cairn-drain/",
    "NON_ADMISSION",
];
const FINAL_SOURCE_POLICIES: &[SourcePolicyExpectation] = &[
    SourcePolicyExpectation {
        path: "bootstrap/gcc-10-final.ncl",
        required: GCC10_FINAL_POLICY_REQUIRED,
        forbidden: FINAL_POLICY_FORBIDDEN,
    },
    SourcePolicyExpectation {
        path: "bootstrap/musl-full.ncl",
        required: MUSL_POLICY_REQUIRED,
        forbidden: FINAL_POLICY_FORBIDDEN,
    },
    SourcePolicyExpectation {
        path: "bootstrap/binutils-full.ncl",
        required: BINUTILS_POLICY_REQUIRED,
        forbidden: FINAL_POLICY_FORBIDDEN,
    },
];

const REQUIRED_NON_CLAIMS: &[&str] = &[
    "general compiler, libc, or binutils correctness",
    "provider or seed admission",
    "whole-bootstrap correctness",
];

pub(crate) fn validate_gcc47_row(project_root: &Path) -> Result<(), String> {
    validate_row(project_root, GCC47_RECEIPT_PATH, &gcc47_expectation())
}

pub(crate) fn validate_gcc10_row(project_root: &Path) -> Result<(), String> {
    validate_row(project_root, GCC10_RECEIPT_PATH, &gcc10_expectation())
}

pub(crate) fn validate_final_musl_binutils_row(project_root: &Path) -> Result<(), String> {
    validate_row(project_root, FINAL_RECEIPT_PATH, &final_expectation())
}

fn gcc47_expectation() -> RowExpectation {
    RowExpectation {
        schema: FINAL_NATIVE_ROW_RECEIPT_SCHEMA,
        row_id: "gcc.4.7",
        derivation: "bootstrap/gcc-4.7.ncl",
        artifact_evidence_path: GCC47_ARTIFACT_PATH,
        artifact_evidence_schema: FINAL_NATIVE_ARTIFACT_EVIDENCE_SCHEMA,
        acceptance_evidence_path: GCC47_ACCEPTANCE_PATH,
        acceptance_evidence_schema: FINAL_NATIVE_ACCEPTANCE_EVIDENCE_SCHEMA,
        success_marker: GCC47_SUCCESS_MARKER,
        source_paths: GCC47_SOURCES,
        source_policies: GCC47_SOURCE_POLICIES,
        predecessor_roles: GCC47_PREDECESSORS,
        predecessor_evidence_paths: GCC47_PREDECESSOR_EVIDENCE,
        generated_artifacts: GCC47_GENERATED,
        positive_matrix: GCC47_POSITIVE,
        rejection_matrix: GCC47_REJECTION,
        required_non_claims: REQUIRED_NON_CLAIMS,
    }
}

fn gcc10_expectation() -> RowExpectation {
    RowExpectation {
        schema: FINAL_NATIVE_ROW_RECEIPT_SCHEMA,
        row_id: "gcc.10",
        derivation: "bootstrap/gcc-10.ncl",
        artifact_evidence_path: GCC10_ARTIFACT_PATH,
        artifact_evidence_schema: FINAL_NATIVE_ARTIFACT_EVIDENCE_SCHEMA,
        acceptance_evidence_path: GCC10_ACCEPTANCE_PATH,
        acceptance_evidence_schema: FINAL_NATIVE_ACCEPTANCE_EVIDENCE_SCHEMA,
        success_marker: GCC10_SUCCESS_MARKER,
        source_paths: GCC10_SOURCES,
        source_policies: GCC10_SOURCE_POLICIES,
        predecessor_roles: GCC10_PREDECESSORS,
        predecessor_evidence_paths: GCC10_PREDECESSOR_EVIDENCE,
        generated_artifacts: GCC10_GENERATED,
        positive_matrix: GCC10_POSITIVE,
        rejection_matrix: GCC10_REJECTION,
        required_non_claims: REQUIRED_NON_CLAIMS,
    }
}

fn final_expectation() -> RowExpectation {
    RowExpectation {
        schema: FINAL_NATIVE_ROW_RECEIPT_SCHEMA,
        row_id: "full-musl-binutils",
        derivation: "bootstrap/binutils-full.ncl",
        artifact_evidence_path: FINAL_ARTIFACT_PATH,
        artifact_evidence_schema: FINAL_NATIVE_ARTIFACT_EVIDENCE_SCHEMA,
        acceptance_evidence_path: FINAL_ACCEPTANCE_PATH,
        acceptance_evidence_schema: FINAL_NATIVE_ACCEPTANCE_EVIDENCE_SCHEMA,
        success_marker: FINAL_SUCCESS_MARKER,
        source_paths: FINAL_SOURCES,
        source_policies: FINAL_SOURCE_POLICIES,
        predecessor_roles: FINAL_PREDECESSORS,
        predecessor_evidence_paths: FINAL_PREDECESSOR_EVIDENCE,
        generated_artifacts: FINAL_GENERATED,
        positive_matrix: FINAL_POSITIVE,
        rejection_matrix: FINAL_REJECTION,
        required_non_claims: REQUIRED_NON_CLAIMS,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn final_row_contracts_are_non_interchangeable() {
        let gcc47 = gcc47_expectation();
        let gcc10 = gcc10_expectation();
        let final_row = final_expectation();
        assert_ne!(gcc47.row_id, gcc10.row_id);
        assert_ne!(gcc10.row_id, final_row.row_id);
        assert_ne!(gcc47.artifact_evidence_path, final_row.artifact_evidence_path);
    }

    #[test]
    fn final_row_contracts_require_positive_and_negative_matrices() {
        for expectation in [gcc47_expectation(), gcc10_expectation(), final_expectation()] {
            assert!(!expectation.positive_matrix.is_empty());
            assert!(!expectation.rejection_matrix.is_empty());
        }
    }
}
