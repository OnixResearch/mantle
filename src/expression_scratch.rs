//! Adapter: stage inline expression text as a temporary Nickel root.
//!
//! Inline roots are written once, handed to an evaluation path, and removed
//! when the staged value drops. The staged file is the only filesystem
//! authority this adapter holds.

use std::io::Write;
use std::path::Path;

use crate::RunError;

/// Suffix used for staged inline roots.
pub(crate) const INLINE_ROOT_SUFFIX: &str = ".ncl";

/// One staged inline expression root.
pub(crate) struct InlineRoot {
    file: tempfile::NamedTempFile,
}

impl InlineRoot {
    /// Stage one expression string as a temporary Nickel file.
    pub(crate) fn stage(expression: &str) -> Result<Self, RunError> {
        let mut file = tempfile::NamedTempFile::with_suffix(INLINE_ROOT_SUFFIX)
            .map_err(|error| RunError::Internal(format!("creating temp file: {error}")))?;
        file.write_all(expression.as_bytes())
            .map_err(|error| RunError::Internal(format!("writing temp file: {error}")))?;
        let root = Self { file };
        debug_assert!(root.path().to_string_lossy().ends_with(INLINE_ROOT_SUFFIX));
        debug_assert!(root.path().is_absolute());
        Ok(root)
    }

    /// Path of the staged root.
    pub(crate) fn path(&self) -> &Path {
        let path = self.file.path();
        debug_assert!(path.is_absolute());
        debug_assert!(path.to_string_lossy().ends_with(INLINE_ROOT_SUFFIX));
        path
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_staged_root_carries_the_expression_and_removes_itself() {
        let expression = "let x = 1 in x";
        let root = InlineRoot::stage(expression).expect("stages");
        let path = root.path().to_path_buf();
        assert!(path.is_absolute());
        assert!(path.to_string_lossy().ends_with(INLINE_ROOT_SUFFIX));
        assert_eq!(std::fs::read_to_string(&path).expect("read staged root"), expression);
        assert_eq!(std::fs::metadata(&path).expect("metadata").len(), u64::try_from(expression.len()).unwrap_or(0));
        drop(root);
        assert!(!path.exists(), "dropping the staged root removes its file");
    }

    #[test]
    fn staged_roots_are_distinct() {
        let first = InlineRoot::stage("1").expect("stages");
        let second = InlineRoot::stage("1").expect("stages");
        assert_ne!(first.path(), second.path());
        assert_eq!(std::fs::read_to_string(first.path()).expect("read first"), "1");
        assert_eq!(std::fs::read_to_string(second.path()).expect("read second"), "1");
    }

    #[test]
    fn an_empty_expression_stages_an_empty_root() {
        let root = InlineRoot::stage("").expect("stages");
        assert_eq!(std::fs::metadata(root.path()).expect("metadata").len(), 0);
        assert_eq!(std::fs::read_to_string(root.path()).expect("read staged root"), "");
    }
}
