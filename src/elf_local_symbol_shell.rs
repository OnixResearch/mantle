use std::fs;
use std::io::Write as _;
use std::path::Path;

const CANONICAL_TEMP_EXTENSION: &str = "mantle-canonical.tmp";

pub(crate) fn canonicalize_elf_file(path: &Path, file_bytes_max: u64) -> Result<u32, String> {
    if file_bytes_max == 0 {
        return Err("ELF canonicalization byte limit must be positive".to_string());
    }
    let metadata =
        fs::symlink_metadata(path).map_err(|error| format!("reading ELF canonicalization input metadata: {error}"))?;
    if !metadata.is_file()
        || metadata.file_type().is_symlink()
        || metadata.len() == 0
        || metadata.len() > file_bytes_max
    {
        return Err(format!("ELF canonicalization input is invalid: {}", path.display()));
    }
    let input_bytes =
        fs::read(path).map_err(|error| format!("reading ELF canonicalization input {}: {error}", path.display()))?;
    let canonical = crate::elf_local_symbol_core::canonicalize_local_elf_symbol_names(&input_bytes)
        .map_err(|error| format!("canonicalizing ELF input {}: {error}", path.display()))?;
    if canonical.bytes == input_bytes {
        assert_eq!(canonical.bytes.len(), input_bytes.len());
        debug_assert!(path.is_file());
        return Ok(canonical.rewrite_count);
    }
    publish_canonical_bytes(path, &metadata, &canonical.bytes)?;
    assert!(u64::try_from(canonical.bytes.len()).is_ok_and(|bytes_len| bytes_len >= metadata.len()));
    debug_assert!(path.is_file());
    Ok(canonical.rewrite_count)
}

fn publish_canonical_bytes(path: &Path, metadata: &fs::Metadata, bytes: &[u8]) -> Result<(), String> {
    let staged = path.with_extension(CANONICAL_TEMP_EXTENSION);
    let write_result = (|| -> Result<(), String> {
        let mut file = fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&staged)
            .map_err(|error| format!("creating canonical ELF output: {error}"))?;
        file.write_all(bytes).map_err(|error| format!("writing canonical ELF output: {error}"))?;
        file.set_permissions(metadata.permissions())
            .map_err(|error| format!("setting canonical ELF output mode: {error}"))?;
        file.sync_all().map_err(|error| format!("syncing canonical ELF output: {error}"))?;
        fs::rename(&staged, path).map_err(|error| format!("publishing canonical ELF output: {error}"))?;
        Ok(())
    })();
    if write_result.is_err() && staged.exists() {
        let _ = fs::remove_file(&staged);
    }
    write_result?;
    assert!(!bytes.is_empty());
    debug_assert!(!staged.exists());
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::os::unix::fs::PermissionsExt as _;

    use super::*;

    const MODE_PERMISSION_BITS: u32 = 0o777;
    const TEST_FILE_MODE: u32 = 0o640;
    const TEST_FILE_BYTES_MAX: u64 = 64;

    #[test]
    fn publication_replaces_bytes_and_preserves_mode() {
        let temporary = tempfile::tempdir().unwrap();
        let path = temporary.path().join("member.o");
        fs::write(&path, b"old").unwrap();
        fs::set_permissions(&path, fs::Permissions::from_mode(TEST_FILE_MODE)).unwrap();
        let metadata = fs::symlink_metadata(&path).unwrap();

        publish_canonical_bytes(&path, &metadata, b"canonical").unwrap();

        assert_eq!(fs::read(&path).unwrap(), b"canonical");
        assert_eq!(fs::symlink_metadata(&path).unwrap().permissions().mode() & MODE_PERMISSION_BITS, TEST_FILE_MODE);
    }

    #[test]
    fn canonicalization_rejects_zero_limit_before_file_access() {
        let error = canonicalize_elf_file(Path::new("/missing/mantle-object.o"), 0).unwrap_err();

        assert!(error.contains("limit must be positive"));
        assert!(!error.contains("metadata"));
    }

    #[test]
    fn canonicalization_rejects_non_regular_input() {
        let temporary = tempfile::tempdir().unwrap();
        let error = canonicalize_elf_file(temporary.path(), TEST_FILE_BYTES_MAX).unwrap_err();

        assert!(error.contains("input is invalid"));
        assert!(temporary.path().is_dir());
    }
}
