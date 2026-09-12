//! Store-prefix resolution policy for the composition root.
//!
//! The root observes two CLI inputs and one declared default; this module turns
//! them into one resolved logical prefix, names which input decided it, records
//! an explicit prefix that `--nix-compat` precedence shadowed, and reports any
//! structural issue in a prefix without changing the resolved value.

use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;

/// Maximum admitted text for one store prefix.
pub const MAX_STORE_PREFIX_TEXT: usize = 4_096;

/// Logical store prefix the CLI declares when the operator omits the flag.
pub const DEFAULT_STORE_PREFIX: &str = "/mantle/store";

/// Logical store prefix selected by `--nix-compat`.
pub const NIX_COMPAT_STORE_PREFIX: &str = "/nix/store";

/// Structural issue observed in a store prefix.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum StorePrefixIssue {
    /// The prefix is empty.
    Empty,
    /// The prefix is not an absolute path.
    NotAbsolute,
    /// The prefix is only the root separator.
    RootOnly,
    /// The prefix ends with a separator.
    TrailingSeparator,
    /// The prefix repeats a separator.
    RepeatedSeparator,
    /// The prefix contains a relative component.
    RelativeComponent,
    /// The prefix contains whitespace.
    Whitespace,
    /// The prefix exceeds the admitted text bound.
    OverBound,
}

/// Which observed input decided the resolved prefix.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum StorePrefixSource {
    /// `--nix-compat` selected the Nix-compatible prefix.
    NixCompat,
    /// The operator declared a prefix other than the CLI default.
    Declared,
    /// The CLI default applied.
    Default,
}

/// The store-prefix inputs the root observes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StorePrefixRequest {
    /// Whether the operator requested Nix compatibility.
    pub is_nix_compat: bool,
    /// Prefix text from the CLI, which is the declared default when omitted.
    pub declared_prefix: String,
    /// The declared default the CLI applies when the operator omits the flag.
    pub default_prefix: String,
}

/// The resolved prefix plus the observations around it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StorePrefixDecision {
    /// Prefix that store paths are built from.
    pub resolved_prefix: String,
    /// Observed input that decided the prefix.
    pub source: StorePrefixSource,
    /// Explicit prefix that nix-compat precedence shadowed, when one was declared.
    pub shadowed_prefix: Option<String>,
    /// Structural issue in the resolved prefix, when one exists.
    pub issue: Option<StorePrefixIssue>,
}

/// Whether one prefix can safely take part in store-path construction.
pub fn is_store_prefix_admissible(prefix: &str) -> bool {
    let issue = observe_store_prefix_issue(prefix);
    debug_assert!(issue.is_none() || !prefix.is_empty() || issue == Some(StorePrefixIssue::Empty));
    debug_assert!(issue != Some(StorePrefixIssue::OverBound) || prefix.len() > MAX_STORE_PREFIX_TEXT);
    issue.is_none()
}

/// The first structural issue in one prefix, if any.
pub fn observe_store_prefix_issue(prefix: &str) -> Option<StorePrefixIssue> {
    let text = prefix;
    if text.is_empty() {
        return Some(StorePrefixIssue::Empty);
    }
    if text.len() > MAX_STORE_PREFIX_TEXT {
        return Some(StorePrefixIssue::OverBound);
    }
    if !text.starts_with('/') {
        return Some(StorePrefixIssue::NotAbsolute);
    }
    if text == "/" {
        return Some(StorePrefixIssue::RootOnly);
    }
    if text.ends_with('/') {
        return Some(StorePrefixIssue::TrailingSeparator);
    }
    if text.contains("//") {
        return Some(StorePrefixIssue::RepeatedSeparator);
    }
    if text.split('/').any(|component| component == "." || component == "..") {
        return Some(StorePrefixIssue::RelativeComponent);
    }
    if text.chars().any(char::is_whitespace) {
        return Some(StorePrefixIssue::Whitespace);
    }
    debug_assert!(text.starts_with('/') && !text.ends_with('/'));
    None
}

/// Resolve the logical store prefix from the observed CLI inputs.
///
/// Precedence is unchanged from the root: `--nix-compat` selects the
/// Nix-compatible prefix, otherwise the declared prefix applies. A declared
/// prefix that differs from the CLI default is recorded as shadowed when
/// nix-compat takes precedence, so the observation is not lost.
pub fn resolve_store_prefix_policy(request: &StorePrefixRequest) -> StorePrefixDecision {
    let is_declared_default = request.declared_prefix == request.default_prefix;
    let is_declared_visible = !is_declared_default && request.declared_prefix != NIX_COMPAT_STORE_PREFIX;
    let (resolved_prefix, source, shadowed_prefix) = if request.is_nix_compat {
        let shadowed = if is_declared_visible {
            Some(request.declared_prefix.clone())
        } else {
            None
        };
        (String::from(NIX_COMPAT_STORE_PREFIX), StorePrefixSource::NixCompat, shadowed)
    } else if is_declared_default {
        (request.default_prefix.clone(), StorePrefixSource::Default, None)
    } else {
        (request.declared_prefix.clone(), StorePrefixSource::Declared, None)
    };
    let issue = observe_store_prefix_issue(&resolved_prefix);
    let decision = StorePrefixDecision {
        resolved_prefix,
        source,
        shadowed_prefix,
        issue,
    };
    debug_assert!(!decision.resolved_prefix.is_empty());
    debug_assert!(decision.shadowed_prefix.is_none() || source == StorePrefixSource::NixCompat);
    decision
}

/// Every structural issue the checker can report, in canonical order.
pub fn store_prefix_issues() -> Vec<StorePrefixIssue> {
    let issues = vec![
        StorePrefixIssue::Empty,
        StorePrefixIssue::NotAbsolute,
        StorePrefixIssue::RootOnly,
        StorePrefixIssue::TrailingSeparator,
        StorePrefixIssue::RepeatedSeparator,
        StorePrefixIssue::RelativeComponent,
        StorePrefixIssue::Whitespace,
        StorePrefixIssue::OverBound,
    ];
    debug_assert_eq!(issues.len(), 8);
    debug_assert!(issues.iter().all(|issue| *issue <= StorePrefixIssue::OverBound));
    issues
}

#[cfg(test)]
mod tests {
    use super::*;

    fn request(is_nix_compat: bool, declared: &str) -> StorePrefixRequest {
        StorePrefixRequest {
            is_nix_compat,
            declared_prefix: String::from(declared),
            default_prefix: String::from(DEFAULT_STORE_PREFIX),
        }
    }

    #[test]
    fn nix_compat_selects_the_compatible_prefix_and_records_the_shadow() {
        let decision = resolve_store_prefix_policy(&request(true, "/custom/store"));
        assert_eq!(decision.resolved_prefix, NIX_COMPAT_STORE_PREFIX);
        assert_eq!(decision.source, StorePrefixSource::NixCompat);
        assert_eq!(decision.shadowed_prefix.as_deref(), Some("/custom/store"));
        assert!(decision.issue.is_none());
    }

    #[test]
    fn a_declared_prefix_applies_and_the_default_stays_default() {
        let declared = resolve_store_prefix_policy(&request(false, "/crunch/store"));
        assert_eq!(declared.resolved_prefix, "/crunch/store");
        assert_eq!(declared.source, StorePrefixSource::Declared);
        assert!(declared.shadowed_prefix.is_none());
        let default = resolve_store_prefix_policy(&request(false, DEFAULT_STORE_PREFIX));
        assert_eq!(default.source, StorePrefixSource::Default);
        assert_eq!(default.resolved_prefix, DEFAULT_STORE_PREFIX);
    }

    #[test]
    fn nix_compat_does_not_record_the_default_or_the_compatible_prefix_as_shadowed() {
        let default = resolve_store_prefix_policy(&request(true, DEFAULT_STORE_PREFIX));
        assert!(default.shadowed_prefix.is_none());
        let compatible = resolve_store_prefix_policy(&request(true, NIX_COMPAT_STORE_PREFIX));
        assert!(compatible.shadowed_prefix.is_none());
        assert_eq!(compatible.resolved_prefix, NIX_COMPAT_STORE_PREFIX);
    }

    #[test]
    fn malformed_prefixes_report_their_issue() {
        let cases: [(String, StorePrefixIssue); 7] = [
            (String::new(), StorePrefixIssue::Empty),
            (String::from("mantle/store"), StorePrefixIssue::NotAbsolute),
            (String::from("/"), StorePrefixIssue::RootOnly),
            (String::from("/mantle/store/"), StorePrefixIssue::TrailingSeparator),
            (String::from("/mantle//store"), StorePrefixIssue::RepeatedSeparator),
            (String::from("/mantle/../store"), StorePrefixIssue::RelativeComponent),
            (String::from("/mantle store"), StorePrefixIssue::Whitespace),
        ];
        for (prefix, expected) in cases {
            assert_eq!(observe_store_prefix_issue(&prefix), Some(expected), "prefix {prefix}");
            assert!(!is_store_prefix_admissible(&prefix));
        }
        let over_bound = String::from("/") + &"a".repeat(MAX_STORE_PREFIX_TEXT);
        assert_eq!(observe_store_prefix_issue(&over_bound), Some(StorePrefixIssue::OverBound));
    }

    #[test]
    fn well_formed_prefixes_are_admissible() {
        assert!(is_store_prefix_admissible(DEFAULT_STORE_PREFIX));
        assert!(is_store_prefix_admissible(NIX_COMPAT_STORE_PREFIX));
        assert!(is_store_prefix_admissible("/crunch/store"));
        assert_eq!(observe_store_prefix_issue("/crunch/store"), None);
        assert_eq!(store_prefix_issues().len(), 8);
    }

    #[test]
    fn a_malformed_declared_prefix_is_reported_without_changing_resolution() {
        let decision = resolve_store_prefix_policy(&request(false, "/mantle/store/"));
        assert_eq!(decision.resolved_prefix, "/mantle/store/");
        assert_eq!(decision.issue, Some(StorePrefixIssue::TrailingSeparator));
        assert_eq!(decision.source, StorePrefixSource::Declared);
    }
}
