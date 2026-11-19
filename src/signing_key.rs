use std::path::Path;
use std::path::PathBuf;

use crunch_build::signing;

use crate::errors::RunError;

/// Load a signing keypair from the given path or the default config location,
/// but never generate a new key.
pub fn load_existing_signing_keypair(
    explicit_path: Option<&Path>,
    state_dir: &Path,
) -> Result<(signing::KeyPair, PathBuf), RunError> {
    if let Some(path) = explicit_path {
        let keypair = load_signing_keypair_from_path(path)?;
        return Ok((keypair, path.to_path_buf()));
    }

    let default_path = default_signing_key_path(state_dir);
    if default_path.exists() {
        let keypair = load_signing_keypair_from_path(&default_path)?;
        return Ok((keypair, default_path));
    }

    Err(RunError::Internal(format!("no signing key found at {}", default_path.display())))
}

/// Load a signing keypair from the given path, the default config location,
/// or generate one automatically.
pub fn load_or_generate_signing_keypair(
    explicit_path: Option<&Path>,
    state_dir: &Path,
    emit_human: bool,
) -> Result<signing::KeyPair, RunError> {
    match load_existing_signing_keypair(explicit_path, state_dir) {
        Ok((keypair, _path)) => return Ok(keypair),
        Err(err) => {
            if explicit_path.is_some() {
                return Err(err);
            }
        }
    }

    // 3. Auto-generate. Publish through an atomic no-replace temp-file
    // persistence so a concurrent caller never observes a partial key and
    // exactly one generated key wins.
    let config_dir = config_dir_or(state_dir);
    let default_path = default_signing_key_path(state_dir);
    let (keypair, line): (signing::KeyPair, String) = signing::generate_keypair();
    let line = zeroize::Zeroizing::new(line);
    std::fs::create_dir_all(&config_dir)
        .map_err(|e| RunError::Internal(format!("creating config dir {}: {e}", config_dir.display())))?;

    let mut temp = tempfile::Builder::new()
        .prefix(".signing-key.")
        .tempfile_in(&config_dir)
        .map_err(|e| RunError::Internal(format!("creating temporary signing key in {}: {e}", config_dir.display())))?;
    {
        use std::io::Write;
        temp.write_all(line.as_bytes())
            .and_then(|()| temp.write_all(b"\n"))
            .and_then(|()| temp.as_file().sync_all())
            .map_err(|e| RunError::Internal(format!("writing signing key {}: {e}", default_path.display())))?;
    }
    match temp.persist_noclobber(&default_path) {
        Ok(_) => {}
        Err(error) if error.error.kind() == std::io::ErrorKind::AlreadyExists => {
            // Another caller won the first-run race. Load the complete key it
            // published; do not overwrite it or return our losing key.
            return load_signing_keypair_from_path(&default_path);
        }
        Err(error) => {
            return Err(RunError::Internal(format!(
                "publishing signing key {}: {}",
                default_path.display(),
                error.error
            )));
        }
    }

    debug_assert!(!line.is_empty());
    debug_assert!(!keypair.verifying_key.name().is_empty());
    if emit_human {
        eprintln!("Generated signing key: {} ({})", keypair.verifying_key.name(), default_path.display());
    }
    Ok(keypair)
}

fn load_signing_keypair_from_path(path: &Path) -> Result<signing::KeyPair, RunError> {
    let bytes = read_private_key_file(path, "signing key", SIGNING_KEY_FILE_MAX_BYTES)?;
    let contents = std::str::from_utf8(&bytes)
        .map_err(|_| RunError::Internal(format!("signing key {} is not UTF-8", path.display())))?;
    signing::load_keypair(contents)
        .map_err(|e| RunError::Internal(format!("parsing signing key {}: {e}", path.display())))
}

/// Largest admitted signing-key file.
pub(crate) const SIGNING_KEY_FILE_MAX_BYTES: u64 = 4_096;

/// Permission bits a private key file must not carry.
const PRIVATE_KEY_FORBIDDEN_MODE_BITS: u32 = 0o077;

/// Read one bounded, private regular file without following symlinks.
///
/// The returned buffer is zeroizing. Errors name only the file role and the
/// failure class: key material must never appear in diagnostics.
pub(crate) fn read_private_key_file(
    path: &Path,
    label: &str,
    max_bytes: u64,
) -> Result<zeroize::Zeroizing<Vec<u8>>, RunError> {
    assert!(!label.is_empty(), "private key label must not be empty");
    assert!(max_bytes > 0, "private key size bound must be positive");

    let metadata = std::fs::symlink_metadata(path)
        .map_err(|error| RunError::Internal(format!("reading {label} metadata: {error}")))?;
    if !metadata.file_type().is_file() || metadata.file_type().is_symlink() || metadata.len() > max_bytes {
        return Err(RunError::Internal(format!("{label} must be a bounded regular file")));
    }
    #[cfg(unix)]
    let mode = {
        use std::os::unix::fs::PermissionsExt;
        metadata.permissions().mode()
    };
    #[cfg(unix)]
    if mode & PRIVATE_KEY_FORBIDDEN_MODE_BITS != 0 {
        return Err(RunError::Internal(format!("{label} permissions are not private")));
    }
    let mut file = {
        use std::os::unix::fs::OpenOptionsExt;
        std::fs::OpenOptions::new()
            .read(true)
            .custom_flags(libc::O_NOFOLLOW)
            .open(path)
            .map_err(|error| RunError::Internal(format!("opening {label}: {error}")))?
    };
    let opened_metadata = file.metadata().map_err(|error| RunError::Internal(format!("checking {label}: {error}")))?;
    let same_file = {
        use std::os::unix::fs::MetadataExt;
        metadata.dev() == opened_metadata.dev()
            && metadata.ino() == opened_metadata.ino()
            && metadata.len() == opened_metadata.len()
    };
    if !same_file {
        return Err(RunError::Internal(format!("{label} changed before reading")));
    }
    let capacity =
        usize::try_from(opened_metadata.len()).map_err(|_| RunError::Internal(format!("{label} is too large")))?;
    let mut bytes = zeroize::Zeroizing::new(Vec::with_capacity(capacity));
    std::io::Read::read_to_end(&mut file, &mut bytes)
        .map_err(|error| RunError::Internal(format!("reading {label}: {error}")))?;
    if bytes.len() != capacity {
        return Err(RunError::Internal(format!("{label} size changed while reading")));
    }
    Ok(bytes)
}

pub(crate) fn default_signing_key_path(state_dir: &Path) -> PathBuf {
    config_dir_or(state_dir).join("signing-key")
}

pub fn load_configured_trusted_public_keys(
    explicit_keys: Option<&[nix_compat::narinfo::VerifyingKey]>,
    state_dir: &Path,
) -> Result<Option<Vec<nix_compat::narinfo::VerifyingKey>>, RunError> {
    if let Some(keys) = explicit_keys {
        return Ok(Some(keys.to_vec()));
    }

    let config_dir = config_dir_or(state_dir);
    let default_path = config_dir.join("trusted-public-keys");
    if !default_path.exists() {
        return Ok(None);
    }

    let contents = std::fs::read_to_string(&default_path)
        .map_err(|e| RunError::Internal(format!("reading trusted public keys {}: {e}", default_path.display())))?;

    let trusted_key_count_max = contents.split(',').count();
    let mut parsed = Vec::with_capacity(trusted_key_count_max);
    for raw_line in contents.lines() {
        let line = raw_line.trim();
        if line.is_empty() {
            continue;
        }
        if line.starts_with('#') {
            continue;
        }
        for key_str in line.split(',') {
            let trimmed = key_str.trim();
            if trimmed.is_empty() {
                continue;
            }
            parsed.push(nix_compat::narinfo::VerifyingKey::parse(trimmed).map_err(|e| {
                RunError::Internal(format!(
                    "invalid trusted public key '{}' in {}: {e}",
                    trimmed,
                    default_path.display()
                ))
            })?);
        }
    }

    if parsed.is_empty() {
        return Ok(None);
    }
    debug_assert!(!parsed.is_empty());
    debug_assert!(parsed.iter().all(|key| !key.name().is_empty()));
    Ok(Some(parsed))
}

pub(crate) fn config_dir_or(state_dir: &Path) -> PathBuf {
    std::env::var("CRUNCH_CONFIG_DIR").map(PathBuf::from).unwrap_or_else(|_| state_dir.to_path_buf())
}

#[cfg(test)]
mod tests {
    use std::os::unix::fs::PermissionsExt;

    use super::*;

    const PRIVATE_KEY_MODE: u32 = 0o600;
    const LOOSE_KEY_MODE: u32 = 0o644;

    fn write_key(path: &Path, contents: &str, mode: u32) {
        std::fs::write(path, contents).expect("write key");
        std::fs::set_permissions(path, std::fs::Permissions::from_mode(mode)).expect("set mode");
    }

    fn generated_line() -> (signing::KeyPair, String) {
        signing::generate_keypair()
    }

    fn keypair_error(result: Result<signing::KeyPair, String>) -> String {
        match result {
            Ok(_) => panic!("malformed keypair unexpectedly parsed"),
            Err(error) => error,
        }
    }

    fn load_error(result: Result<signing::KeyPair, RunError>) -> RunError {
        match result {
            Ok(_) => panic!("invalid key file unexpectedly loaded"),
            Err(error) => error,
        }
    }

    #[test]
    fn valid_private_key_file_loads() {
        let root = tempfile::tempdir().expect("tempdir");
        let path = root.path().join("signing-key");
        let (generated, line) = generated_line();
        write_key(&path, &line, PRIVATE_KEY_MODE);

        let loaded = load_signing_keypair_from_path(&path).expect("valid key loads");
        assert_eq!(loaded.verifying_key.name(), generated.verifying_key.name());
        assert_eq!(loaded.verifying_key, generated.verifying_key);
    }

    #[test]
    fn malformed_base64_error_is_redacted() {
        let (_, line) = generated_line();
        let (name, encoded) = line.trim().split_once(':').expect("generated key has a separator");
        let mut chars = encoded.chars().collect::<Vec<_>>();
        let index = chars.len() / 2;
        chars[index] = '!';
        let malformed = format!("{name}:{}", chars.into_iter().collect::<String>());

        let error = keypair_error(signing::load_keypair(&malformed));
        assert!(error.contains("base64"), "error class is retained: {error}");
        assert!(!error.contains(encoded), "encoded key material must not appear in the error: {error}");
        assert!(!error.contains(&malformed), "the key file text must not appear in the error: {error}");
    }

    #[test]
    fn public_half_mismatch_is_rejected() {
        let (_, line) = generated_line();
        let (name, encoded) = line.trim().split_once(':').expect("generated key has a separator");
        let mut chars = encoded.chars().collect::<Vec<_>>();
        let index = chars.len() / 2 + 10;
        chars[index] = if chars[index] == 'A' { 'B' } else { 'A' };
        let mismatched = format!("{name}:{}", chars.into_iter().collect::<String>());

        let error = keypair_error(signing::load_keypair(&mismatched));
        assert!(error.contains("does not match"), "mismatch is classified: {error}");
        assert!(!error.contains(encoded), "encoded key material must not appear in the error: {error}");
    }

    #[test]
    fn loose_mode_is_rejected_before_reading() {
        let root = tempfile::tempdir().expect("tempdir");
        let path = root.path().join("signing-key");
        let (_, line) = generated_line();
        write_key(&path, &line, LOOSE_KEY_MODE);

        let error = load_error(load_signing_keypair_from_path(&path));
        assert!(format!("{error}").contains("permissions are not private"));
    }

    #[test]
    fn concurrent_first_run_publishes_one_complete_private_key() {
        use std::os::unix::fs::PermissionsExt;

        let root = tempfile::tempdir().expect("tempdir");
        let state = root.path().join("state");
        std::fs::create_dir_all(&state).expect("state dir");

        let loaded = std::thread::scope(|scope| {
            let handles: Vec<_> =
                (0..8).map(|_| scope.spawn(|| load_or_generate_signing_keypair(None, &state, false))).collect();
            handles
                .into_iter()
                .map(|handle| handle.join().expect("thread").expect("key loads"))
                .collect::<Vec<_>>()
        });

        let expected = loaded[0].verifying_key.to_string();
        for keypair in &loaded {
            assert_eq!(keypair.verifying_key.to_string(), expected, "concurrent callers must share one key");
        }
        let path = default_signing_key_path(&state);
        let mode = std::fs::metadata(&path).expect("published key").permissions().mode() & 0o777;
        assert_eq!(mode, 0o600, "published key must remain private");
        let contents = std::fs::read_to_string(&path).expect("read published key");
        assert_eq!(contents.lines().count(), 1, "published key must be complete");
        assert!(!contents.trim().is_empty(), "published key must not be empty");
    }

    #[test]
    fn symlink_and_oversized_keys_are_rejected() {
        let root = tempfile::tempdir().expect("tempdir");
        let target = root.path().join("target-key");
        let (_, line) = generated_line();
        write_key(&target, &line, PRIVATE_KEY_MODE);
        let link = root.path().join("linked-key");
        std::os::unix::fs::symlink(&target, &link).expect("symlink");
        let error = load_error(load_signing_keypair_from_path(&link));
        assert!(format!("{error}").contains("bounded regular file"));

        let oversized = root.path().join("oversized-key");
        let capacity = usize::try_from(SIGNING_KEY_FILE_MAX_BYTES).expect("bound fits usize");
        write_key(&oversized, &"a".repeat(capacity.saturating_add(1)), PRIVATE_KEY_MODE);
        let error = load_error(load_signing_keypair_from_path(&oversized));
        assert!(format!("{error}").contains("bounded regular file"));
    }
}
