//! Rendering for a materialized source-root provider.

use std::path::Path;

use crate::RunError;

/// Facts reported after one provider materialization.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct SourceRootProviderReport<'a> {
    /// Final store path that received the provider tree.
    pub(crate) final_output: &'a Path,
    /// Seed file written for the provider.
    pub(crate) output_path: &'a Path,
    pub(crate) manifest_digest: &'a str,
    pub(crate) output_digest: &'a str,
    pub(crate) expected_output_role_count: u32,
    pub(crate) dependency_trace_url_count: u32,
}

/// Render the provider report as the operator sees it.
pub(crate) fn render_source_root_provider_report(report: &SourceRootProviderReport<'_>) -> String {
    debug_assert!(!report.manifest_digest.is_empty());
    debug_assert!(!report.output_digest.is_empty());
    let lines = [
        format!("Materialized source-root provider {}", report.final_output.display()),
        format!("  manifest_digest: {}", report.manifest_digest),
        format!("  output_digest: {}", report.output_digest),
        format!("  expected_output_roles: {}", report.expected_output_role_count),
        format!("  dependency_trace_urls: {}", report.dependency_trace_url_count),
        format!("Wrote {}", report.output_path.display()),
    ];
    debug_assert_eq!(lines.len(), 6);
    debug_assert!(lines.iter().all(|line| !line.is_empty()));
    lines.join("\n")
}

/// Write the provider report to standard error.
pub(crate) fn emit_source_root_provider_report(report: &SourceRootProviderReport<'_>) -> Result<(), RunError> {
    let rendered = render_source_root_provider_report(report);
    eprintln!("{rendered}");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn report<'a>(final_output: &'a Path, output_path: &'a Path) -> SourceRootProviderReport<'a> {
        SourceRootProviderReport {
            final_output,
            output_path,
            manifest_digest: "manifest-digest",
            output_digest: "output-digest",
            expected_output_role_count: 3,
            dependency_trace_url_count: 2,
        }
    }

    #[test]
    fn the_report_lists_every_field_with_its_label() {
        let final_output = Path::new("/store/aaa-provider");
        let output_path = Path::new("/tmp/seed.ncl");
        let rendered = render_source_root_provider_report(&report(final_output, output_path));
        let lines: Vec<&str> = rendered.lines().collect();
        assert_eq!(lines.len(), 6);
        assert_eq!(lines[0], "Materialized source-root provider /store/aaa-provider");
        assert_eq!(lines[1], "  manifest_digest: manifest-digest");
        assert_eq!(lines[2], "  output_digest: output-digest");
        assert_eq!(lines[3], "  expected_output_roles: 3");
        assert_eq!(lines[4], "  dependency_trace_urls: 2");
        assert_eq!(lines[5], "Wrote /tmp/seed.ncl");
    }

    #[test]
    fn the_report_tracks_the_paths_it_is_given() {
        let rendered =
            render_source_root_provider_report(&report(Path::new("/store/bbb-provider"), Path::new("/tmp/other.ncl")));
        assert!(rendered.contains("/store/bbb-provider"));
        assert!(rendered.contains("/tmp/other.ncl"));
        assert!(!rendered.contains("/store/aaa-provider"));
    }
}
