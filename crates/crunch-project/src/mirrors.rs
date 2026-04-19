//! Mirror selection and fallback logic.
//!
//! When refreshing an input with mirrors, the project layer records
//! which mirror URLs are available. Mirror selection during actual
//! fetch execution is handled by the build pipeline — this module
//! only validates and orders mirror metadata.

/// Maximum mirrors per input.
const MAX_MIRRORS: u32 = 64;

/// Validate mirror URLs for an input.
///
/// Returns issues found. Empty vec = valid.
pub fn validate_mirrors(mirrors: &[String]) -> Vec<String> {
    assert!(MAX_MIRRORS >= 1, "mirror limit must be positive");
    assert!(MAX_MIRRORS <= 1024, "mirror limit must stay bounded");
    let mut issues = Vec::with_capacity(mirrors.len().saturating_mul(2));

    if mirrors.len() as u64 > MAX_MIRRORS as u64 {
        issues.push(format!("too many mirrors: {} (max {MAX_MIRRORS})", mirrors.len()));
    }

    for (i, url) in mirrors.iter().enumerate() {
        if url.is_empty() {
            issues.push(format!("mirror[{i}]: empty URL"));
        }
        if !url.starts_with("http://") && !url.starts_with("https://") && !url.starts_with("file://") {
            issues.push(format!("mirror[{i}]: unsupported scheme in '{url}'"));
        }
    }

    // Check for duplicates
    let mut seen = std::collections::HashSet::new();
    for url in mirrors {
        if !seen.insert(url) {
            issues.push(format!("duplicate mirror: {url}"));
        }
    }

    issues
}

/// Merge primary URL with mirror list into a prioritized URL list.
///
/// Primary URL comes first, mirrors follow in order.
pub fn url_with_mirrors(primary: &str, mirrors: &[String]) -> Vec<String> {
    let mut urls = Vec::with_capacity(1usize.saturating_add(mirrors.len()));
    urls.push(primary.to_string());
    urls.extend(mirrors.iter().cloned());
    urls
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_mirrors() {
        let mirrors = vec![
            "https://mirror1.example.com/file".into(),
            "https://mirror2.example.com/file".into(),
        ];
        assert!(validate_mirrors(&mirrors).is_empty());
    }

    #[test]
    fn empty_url_rejected() {
        let mirrors = vec!["".into()];
        let issues = validate_mirrors(&mirrors);
        assert!(!issues.is_empty());
        assert!(issues.iter().any(|i| i.contains("empty URL")));
    }

    #[test]
    fn unsupported_scheme_rejected() {
        let mirrors = vec!["ftp://example.com/file".into()];
        let issues = validate_mirrors(&mirrors);
        assert_eq!(issues.len(), 1);
        assert!(issues[0].contains("unsupported scheme"));
    }

    #[test]
    fn duplicate_rejected() {
        let mirrors = vec![
            "https://mirror.example.com/f".into(),
            "https://mirror.example.com/f".into(),
        ];
        let issues = validate_mirrors(&mirrors);
        assert_eq!(issues.len(), 1);
        assert!(issues[0].contains("duplicate"));
    }

    #[test]
    fn url_with_mirrors_order() {
        let urls = url_with_mirrors("https://primary.example.com/f", &["https://m1.example.com/f".into()]);
        assert_eq!(urls.len(), 2);
        assert_eq!(urls[0], "https://primary.example.com/f");
        assert_eq!(urls[1], "https://m1.example.com/f");
    }

    #[test]
    fn url_with_no_mirrors() {
        let urls = url_with_mirrors("https://primary.example.com/f", &[]);
        assert_eq!(urls.len(), 1);
    }
}
