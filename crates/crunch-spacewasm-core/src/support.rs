use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec::Vec;

use serde::Deserialize;
use serde::Serialize;

use crate::Diagnostic;
use crate::ReferenceProfile;
use crate::SupportEntry;
use crate::SupportStatus;
use crate::diagnostic::ErrorDiagnostic;
use crate::diagnostic::MAX_DIAGNOSTIC_COUNT;
use crate::diagnostic::error;
use crate::diagnostic::has_errors;
use crate::diagnostic::ordered;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SupportComparison {
    pub matches_profile: bool,
    pub diagnostics: Vec<Diagnostic>,
}

pub fn compare_support_matrix(profile: ReferenceProfile, observed: Vec<SupportEntry>) -> SupportComparison {
    let reserved_diagnostic_count = profile
        .support_matrix
        .len()
        .saturating_add(observed.len())
        .saturating_add(profile.requested_features.len())
        .min(MAX_DIAGNOSTIC_COUNT);
    let mut diagnostics = Vec::with_capacity(reserved_diagnostic_count);
    let expected = support_map(&profile.support_matrix, "profile", &mut diagnostics);
    let actual = support_map(&observed, "observed", &mut diagnostics);
    for (feature, expected_status) in &expected {
        match actual.get(feature) {
            Some(actual_status) if actual_status == expected_status => {}
            Some(_) => diagnostics.push(error(ErrorDiagnostic {
                code: "support-status-mismatch",
                subject: feature,
                message: "observed support status differs from the reviewed profile",
            })),
            None => diagnostics.push(error(ErrorDiagnostic {
                code: "missing-support-observation",
                subject: feature,
                message: "observed support matrix omits a reviewed feature",
            })),
        }
    }
    for feature in actual.keys() {
        if !expected.contains_key(feature) {
            diagnostics.push(error(ErrorDiagnostic {
                code: "unreviewed-support-observation",
                subject: feature,
                message: "observed support matrix introduces an unreviewed feature",
            }));
        }
    }
    for requested in &profile.requested_features {
        if actual.get(requested) != Some(&SupportStatus::Supported) {
            diagnostics.push(error(ErrorDiagnostic {
                code: "unsupported-feature-requested",
                subject: requested,
                message: "requested feature is not observed as supported",
            }));
        }
    }
    let diagnostics = ordered(diagnostics);
    let is_profile_match = !has_errors(&diagnostics);
    debug_assert_eq!(
        is_profile_match,
        diagnostics.iter().all(|item| item.severity != crate::DiagnosticSeverity::Error)
    );
    debug_assert!(diagnostics.iter().all(|item| !item.subject.is_empty()));
    SupportComparison {
        matches_profile: is_profile_match,
        diagnostics,
    }
}

fn support_map(
    entries: &[SupportEntry],
    subject: &str,
    diagnostics: &mut Vec<Diagnostic>,
) -> BTreeMap<String, SupportStatus> {
    let mut map = BTreeMap::new();
    for entry in entries {
        if entry.feature.is_empty() || map.insert(entry.feature.clone(), entry.status).is_some() {
            diagnostics.push(error(ErrorDiagnostic {
                code: "duplicate-support-entry",
                subject: subject,
                message: "support feature names must be unique and non-empty",
            }));
        }
    }
    debug_assert!(map.len() <= entries.len());
    debug_assert!(entries.is_empty() || !map.is_empty() || !diagnostics.is_empty());
    map
}
