//! Adapter: publish the local-route test sentinel marker.
//!
//! The activation rule lives in the application contract and the marker text
//! comes from that decision; this adapter only writes the marker.

use std::fs;
use std::path::Path;

use crate::RunError;

/// Publish the local-route marker with one trailing newline.
pub(crate) fn publish_local_route_marker(path: &Path, marker_text: &str) -> Result<(), RunError> {
    if path.as_os_str().is_empty() {
        return Err(RunError::Internal("local-route test sentinel path must not be empty".to_string()));
    }
    debug_assert!(!path.as_os_str().is_empty());
    debug_assert!(!marker_text.is_empty());
    let mut marker = marker_text.as_bytes().to_vec();
    marker.push(b'\n');
    fs::write(path, marker)
        .map_err(|error| RunError::Internal(format!("writing local-route test sentinel: {error}")))?;
    debug_assert!(path.is_file());
    debug_assert!(fs::read_to_string(path).map(|text| text.ends_with('\n')).unwrap_or(false));
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_marker_is_written_with_one_trailing_newline() {
        let root = tempfile::tempdir().expect("tempdir");
        let path = root.path().join("sentinel");
        publish_local_route_marker(&path, "local-route-entered").expect("publishes");
        assert_eq!(fs::read_to_string(&path).expect("read marker"), "local-route-entered\n");
    }

    #[test]
    fn an_existing_marker_is_replaced() {
        let root = tempfile::tempdir().expect("tempdir");
        let path = root.path().join("sentinel");
        fs::write(&path, "older marker content\n").expect("seed marker");
        publish_local_route_marker(&path, "local-route-entered").expect("publishes");
        assert_eq!(fs::read_to_string(&path).expect("read marker"), "local-route-entered\n");
    }

    #[test]
    fn a_missing_parent_directory_is_rejected() {
        let root = tempfile::tempdir().expect("tempdir");
        let path = root.path().join("absent").join("sentinel");
        let error = publish_local_route_marker(&path, "local-route-entered").expect_err("missing parent is rejected");
        assert!(format!("{error}").contains("writing local-route test sentinel"));
    }

    #[test]
    fn an_empty_path_is_rejected_before_io() {
        let error = publish_local_route_marker(Path::new(""), "marker").expect_err("empty path is rejected");
        assert!(format!("{error}").contains("must not be empty"));
    }
}
