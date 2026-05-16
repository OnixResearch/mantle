use alloc::collections::BTreeSet;
use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;

const MAX_MIRRORS: usize = 32;

const _: () = assert!(MAX_MIRRORS >= 1, "mirror limit must be positive");
const _: () = assert!(MAX_MIRRORS <= 1024, "mirror limit must stay bounded");

pub fn validate_mirrors(mirrors: Vec<String>) -> Vec<String> {
    let mut issues = Vec::new();

    if mirrors.len() as u64 > MAX_MIRRORS as u64 {
        issues.push(format!("too many mirrors: {} (max {MAX_MIRRORS})", mirrors.len()));
    }

    for (index, url) in mirrors.iter().enumerate() {
        if url.is_empty() {
            issues.push(format!("mirror[{index}]: empty URL"));
        }
        if !url.starts_with("http://") && !url.starts_with("https://") && !url.starts_with("file://") {
            issues.push(format!("mirror[{index}]: unsupported scheme in '{url}'"));
        }
    }

    let mut seen = BTreeSet::new();
    for url in mirrors {
        if !seen.insert(url.clone()) {
            issues.push(format!("duplicate mirror: {url}"));
        }
    }

    issues
}

pub fn url_with_mirrors(primary: String, mirrors: Vec<String>) -> Vec<String> {
    let mut urls = Vec::with_capacity(1usize.saturating_add(mirrors.len()));
    urls.push(primary);
    urls.extend(mirrors);
    urls
}

#[cfg(test)]
mod tests {
    use alloc::vec;

    use super::*;

    #[test]
    fn valid_mirrors() {
        let mirrors = vec![
            "https://mirror1.example.com/file".into(),
            "https://mirror2.example.com/file".into(),
        ];
        assert!(validate_mirrors(mirrors).is_empty());
    }

    #[test]
    fn empty_url_rejected() {
        let mirrors = vec!["".into()];
        let issues = validate_mirrors(mirrors);
        assert!(!issues.is_empty());
        assert!(issues.iter().any(|issue| issue.contains("empty URL")));
    }

    #[test]
    fn unsupported_scheme_rejected() {
        let mirrors = vec!["ftp://example.com/file".into()];
        let issues = validate_mirrors(mirrors);
        assert_eq!(issues.len(), 1);
        assert!(issues[0].contains("unsupported scheme"));
    }

    #[test]
    fn duplicate_rejected() {
        let mirrors = vec![
            "https://mirror.example.com/f".into(),
            "https://mirror.example.com/f".into(),
        ];
        let issues = validate_mirrors(mirrors);
        assert_eq!(issues.len(), 1);
        assert!(issues[0].contains("duplicate"));
    }

    #[test]
    fn url_with_mirrors_order() {
        let urls = url_with_mirrors("https://primary.example.com/f".into(), vec!["https://m1.example.com/f".into()]);
        assert_eq!(urls.len(), 2);
        assert_eq!(urls[0], "https://primary.example.com/f");
        assert_eq!(urls[1], "https://m1.example.com/f");
    }

    #[test]
    fn url_with_no_mirrors() {
        let urls = url_with_mirrors("https://primary.example.com/f".into(), vec![]);
        assert_eq!(urls.len(), 1);
    }
}
