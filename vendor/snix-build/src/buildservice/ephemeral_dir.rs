use std::io;
use std::path::Path;

const MAX_PREFIX_BYTES: usize = 128;

pub(crate) fn create_ephemeral_dir(parent_dir: &Path, prefix: &str) -> io::Result<tempfile::TempDir> {
    validate_prefix(prefix)?;
    std::fs::create_dir_all(parent_dir)?;
    tempfile::Builder::new().prefix(prefix).tempdir_in(parent_dir)
}

fn validate_prefix(prefix: &str) -> io::Result<()> {
    if prefix.is_empty() {
        return Err(io::Error::new(io::ErrorKind::InvalidInput, "ephemeral dir prefix must not be empty"));
    }
    if prefix.len() > MAX_PREFIX_BYTES {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("ephemeral dir prefix exceeds {MAX_PREFIX_BYTES} bytes"),
        ));
    }
    if prefix.contains(std::path::MAIN_SEPARATOR) {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("ephemeral dir prefix must not contain '{}': {prefix}", std::path::MAIN_SEPARATOR),
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_ephemeral_dir_creates_parent_and_child() {
        let temp = tempfile::tempdir().unwrap();
        let parent = temp.path().join("missing-parent");
        let child = create_ephemeral_dir(&parent, "build-").unwrap();

        assert!(parent.is_dir());
        assert!(child.path().is_dir());
        assert!(child.path().starts_with(&parent));
    }

    #[test]
    fn create_ephemeral_dir_removes_child_on_drop() {
        let temp = tempfile::tempdir().unwrap();
        let parent = temp.path().join("workdir");
        let child = create_ephemeral_dir(&parent, "build-").unwrap();
        let child_path = child.path().to_path_buf();

        assert!(child_path.exists());
        drop(child);
        assert!(!child_path.exists());
        assert!(parent.is_dir());
    }

    #[test]
    fn create_ephemeral_dir_rejects_empty_prefix() {
        let temp = tempfile::tempdir().unwrap();
        let err = create_ephemeral_dir(temp.path(), "").unwrap_err();

        assert_eq!(err.kind(), io::ErrorKind::InvalidInput);
        assert!(err.to_string().contains("must not be empty"));
    }
}
