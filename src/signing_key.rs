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

    // 3. Auto-generate.
    let config_dir = config_dir_or(state_dir);
    let default_path = default_signing_key_path(state_dir);
    let (keypair, line): (signing::KeyPair, String) = signing::generate_keypair();
    std::fs::create_dir_all(&config_dir)
        .map_err(|e| RunError::Internal(format!("creating config dir {}: {e}", config_dir.display())))?;

    // Write with 0600 permissions.
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .mode(0o600)
            .open(&default_path)
            .and_then(|mut f| {
                use std::io::Write;
                f.write_all(line.as_bytes())?;
                f.write_all(b"\n")?;
                Ok(())
            })
            .map_err(|e| RunError::Internal(format!("writing signing key {}: {e}", default_path.display())))?;
    }
    #[cfg(not(unix))]
    {
        std::fs::write(&default_path, format!("{line}\n"))
            .map_err(|e| RunError::Internal(format!("writing signing key {}: {e}", default_path.display())))?;
    }

    debug_assert!(!line.is_empty());
    debug_assert!(!keypair.verifying_key.name().is_empty());
    if emit_human {
        eprintln!("Generated signing key: {} ({})", keypair.verifying_key.name(), default_path.display());
    }
    Ok(keypair)
}

fn load_signing_keypair_from_path(path: &Path) -> Result<signing::KeyPair, RunError> {
    let contents = std::fs::read_to_string(path)
        .map_err(|e| RunError::Internal(format!("reading signing key {}: {e}", path.display())))?;
    signing::load_keypair(&contents)
        .map_err(|e| RunError::Internal(format!("parsing signing key {}: {e}", path.display())))
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
