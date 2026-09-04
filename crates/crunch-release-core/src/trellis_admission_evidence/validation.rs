use alloc::string::String;
use alloc::vec::Vec;

use super::TrellisAdmissionProjectionCounts;
use super::contract::BLAKE3_HEX_LENGTH;
use super::contract::REVISION_HEX_LENGTH;

pub(super) struct LabeledValue<'a> {
    pub value: &'a str,
    pub field: &'static str,
}

pub(super) struct ExpectedValue<'a> {
    pub actual: &'a str,
    pub expected: &'static str,
    pub field: &'static str,
}

pub(super) fn projection_count_total(counts: TrellisAdmissionProjectionCounts) -> Option<u32> {
    counts
        .supported
        .checked_add(counts.authority_meaning_mismatch)?
        .checked_add(counts.authority_order_mismatch)?
        .checked_add(counts.history_order_mismatch)?
        .checked_add(counts.phase_report_mismatch)?
        .checked_add(counts.result_retention_mismatch)
}

pub(super) fn relative_path(input: LabeledValue<'_>, diagnostics: &mut Vec<String>) {
    let is_invalid = input.value.is_empty()
        || input.value.starts_with('/')
        || input.value.split('/').any(|segment| segment.is_empty() || segment == "." || segment == "..");
    if is_invalid {
        diagnostics.push(alloc::format!("{} is not a safe relative path", input.field));
    }
}

pub(super) fn blake3(input: LabeledValue<'_>, diagnostics: &mut Vec<String>) {
    let is_valid = input.value.len() == BLAKE3_HEX_LENGTH
        && input.value.bytes().all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte));
    if !is_valid {
        diagnostics.push(alloc::format!("{} is not lowercase BLAKE3 hex", input.field));
    }
}

pub(super) fn revision(input: LabeledValue<'_>, diagnostics: &mut Vec<String>) {
    let is_valid = input.value.len() == REVISION_HEX_LENGTH
        && input.value.bytes().all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte));
    if !is_valid {
        diagnostics.push(alloc::format!("{} is not lowercase revision hex", input.field));
    }
}

pub(super) fn equal(input: ExpectedValue<'_>, diagnostics: &mut Vec<String>) {
    if input.actual != input.expected {
        diagnostics.push(alloc::format!("{} mismatch", input.field));
    }
}
