use std::collections::BTreeSet;
use std::fs::File;
use std::fs::OpenOptions;
use std::io::Read;
use std::io::Write;
use std::path::Component;
use std::path::Path;
use std::path::PathBuf;

use crunch_composition_core::user_profile::GENERATION_SCHEMA;
use crunch_composition_core::user_profile::Generation;
use fs2::FileExt;
use serde::Deserialize;
use serde::Serialize;

pub(super) const JSON_BYTES_MAX: u64 = 8_388_608;
pub(super) const GENERATIONS_MAX: usize = 4096;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub(super) struct Binding {
    pub schema: String,
    pub state_dir: PathBuf,
    pub output_dir: PathBuf,
    pub store_prefix: String,
    pub platform: String,
}

pub(super) fn lock_profile(profile: &Path) -> Result<File, String> {
    require_directory(profile, true)?;
    let mut options = OpenOptions::new();
    options.read(true).write(true).create(true).truncate(false);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.custom_flags(libc::O_NOFOLLOW).mode(0o600);
    }
    let file = options.open(profile.join("profile.lock")).map_err(|error| error.to_string())?;
    file.try_lock_exclusive()
        .map_err(|error| format!("another profile writer or inspector is active: {error}"))?;
    assert!(profile.is_absolute());
    assert!(file.metadata().map_err(|error| error.to_string())?.is_file());
    Ok(file)
}

/// Refuse symlink traversal and non-normalized targets, including ancestors.
pub(super) fn require_directory(path: &Path, create: bool) -> Result<(), String> {
    if !path.is_absolute() {
        return Err("profile/store directories must be explicit absolute paths".into());
    }
    let mut current = PathBuf::new();
    for component in path.components() {
        match component {
            Component::RootDir | Component::Normal(_) => current.push(component.as_os_str()),
            _ => return Err("profile/store directory contains a non-normal path component".into()),
        }
        match std::fs::symlink_metadata(&current) {
            Ok(metadata) if metadata.is_dir() => {}
            Ok(_) => return Err(format!("profile/store ancestor is not a real directory: {}", current.display())),
            Err(error) if create && error.kind() == std::io::ErrorKind::NotFound => {
                let mut builder = std::fs::DirBuilder::new();
                #[cfg(unix)]
                {
                    use std::os::unix::fs::DirBuilderExt;
                    builder.mode(0o700);
                }
                match builder.create(&current) {
                    Ok(()) => {}
                    Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {
                        if !std::fs::symlink_metadata(&current).map_err(|error| error.to_string())?.is_dir() {
                            return Err("concurrently created profile ancestor is not a directory".into());
                        }
                    }
                    Err(error) => return Err(error.to_string()),
                }
            }
            Err(error) => return Err(error.to_string()),
        }
    }
    assert!(path.is_absolute());
    assert!(path.is_dir());
    Ok(())
}

pub(super) fn bind(profile: &Path, expected: &Binding, apply: bool) -> Result<(), String> {
    let path = profile.join("binding.json");
    match read_optional::<Binding>(&path)? {
        Some(binding) if binding != *expected => {
            return Err("profile target is bound to a different store or platform".into());
        }
        Some(_) => {}
        None => {
            require_unbound_empty(profile)?;
            if apply {
                publish_new_json(&path, expected)?;
            }
        }
    }
    require_directory(&profile.join("generations"), true)?;
    assert_eq!(expected.schema, "mantle-user-profile-binding-v1");
    assert!(!expected.platform.is_empty());
    Ok(())
}

fn require_unbound_empty(profile: &Path) -> Result<(), String> {
    for entry in std::fs::read_dir(profile).map_err(|error| error.to_string())? {
        let entry = entry.map_err(|error| error.to_string())?;
        if entry.file_name() == "profile.lock" {
            continue;
        }
        if entry.file_name() == "generations" {
            require_directory(&entry.path(), false)?;
            if std::fs::read_dir(entry.path()).map_err(|error| error.to_string())?.next().is_none() {
                continue;
            }
        }
        return Err("unbound profile target is not empty; refusing to adopt existing content".into());
    }
    assert!(profile.is_absolute());
    assert!(!profile.join("binding.json").exists());
    Ok(())
}

pub(super) fn read_optional<T: for<'de> Deserialize<'de>>(path: &Path) -> Result<Option<T>, String> {
    let metadata = match std::fs::symlink_metadata(path) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(error.to_string()),
    };
    if !metadata.is_file() || metadata.len() > JSON_BYTES_MAX {
        return Err(format!("invalid profile metadata file: {}", path.display()));
    }
    let mut bytes = Vec::new();
    File::open(path)
        .map_err(|error| error.to_string())?
        .take(JSON_BYTES_MAX.saturating_add(1))
        .read_to_end(&mut bytes)
        .map_err(|error| error.to_string())?;
    if bytes.len() as u64 > JSON_BYTES_MAX {
        return Err("profile metadata exceeds byte bound".into());
    }
    let value = serde_json::from_slice(&bytes)
        .map_err(|error| format!("invalid profile metadata {}: {error}", path.display()))?;
    assert!(metadata.is_file());
    assert!(bytes.len() as u64 <= JSON_BYTES_MAX);
    Ok(Some(value))
}

pub(super) fn read_generation(profile: &Path, number: u64) -> Result<Generation, String> {
    let path = generation_path(profile, number);
    require_directory(&path, false)?;
    let generation: Generation =
        read_optional(&path.join("manifest.json"))?.ok_or("profile generation manifest missing")?;
    if generation.schema != GENERATION_SCHEMA || generation.number != number || number == 0 {
        return Err("profile generation identity/schema mismatch".into());
    }
    generation.manifest.validate().map_err(|error| error.to_string())?;
    if generation.links.len() > crunch_composition_core::user_profile::ENTRIES_MAX {
        return Err("profile generation link inventory exceeds bound".into());
    }
    assert_eq!(generation.number, number);
    assert_eq!(generation.schema, GENERATION_SCHEMA);
    Ok(generation)
}

pub(super) fn active(profile: &Path) -> Result<Option<u64>, String> {
    let path = profile.join("current");
    match std::fs::symlink_metadata(&path) {
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(error.to_string()),
        Ok(metadata) if !metadata.file_type().is_symlink() => return Err("profile current is not a symlink".into()),
        Ok(_) => {}
    }
    let target = std::fs::read_link(&path).map_err(|error| error.to_string())?;
    let text = target.to_str().ok_or("profile current target is not UTF-8")?;
    let number = text
        .strip_prefix("generations/")
        .ok_or("profile current has an invalid target")?
        .parse::<u64>()
        .map_err(|error| error.to_string())?;
    if number == 0 || text != format!("generations/{number}") {
        return Err("profile current target is not canonical".into());
    }
    read_generation(profile, number)?;
    assert!(number > 0);
    assert_eq!(target, PathBuf::from(format!("generations/{number}")));
    Ok(Some(number))
}

pub(super) fn generations(profile: &Path) -> Result<Vec<u64>, String> {
    let mut result = Vec::new();
    for entry in std::fs::read_dir(profile.join("generations")).map_err(|error| error.to_string())? {
        let entry = entry.map_err(|error| error.to_string())?;
        if result.len() >= GENERATIONS_MAX {
            return Err("profile retained generations exceed bound".into());
        }
        let name = entry.file_name().into_string().map_err(|_| "profile generation is not UTF-8")?;
        let number = name.parse::<u64>().map_err(|_| "unexpected entry in profile generations")?;
        if number == 0 || name != number.to_string() {
            return Err("noncanonical profile generation number".into());
        }
        read_generation(profile, number)?;
        result.push(number);
    }
    result.sort_unstable();
    assert!(result.len() <= GENERATIONS_MAX);
    assert!(result.iter().all(|number| *number > 0));
    Ok(result)
}

pub(super) fn next_number(profile: &Path, retained: &[u64]) -> Result<u64, String> {
    let allocated = read_optional::<u64>(&profile.join("sequence.json"))?.unwrap_or(0);
    let greatest = retained.last().copied().unwrap_or(0).max(allocated);
    assert!(greatest >= allocated);
    assert!(greatest >= retained.last().copied().unwrap_or(0));
    greatest.checked_add(1).ok_or_else(|| "profile generation counter exhausted".into())
}

pub(super) fn allocate(profile: &Path, number: u64) -> Result<(), String> {
    replace_json(&profile.join("sequence.json"), &number)?;
    assert!(number > 0);
    assert_eq!(read_optional::<u64>(&profile.join("sequence.json"))?, Some(number));
    Ok(())
}

pub(super) fn stage(profile: &Path, generation: &Generation) -> Result<tempfile::TempDir, String> {
    let stage = tempfile::Builder::new()
        .prefix(&format!(".stage-{}-", generation.number))
        .tempdir_in(profile)
        .map_err(|error| error.to_string())?;
    publish_new_json(&stage.path().join("manifest.json"), generation)?;
    let mut directories = BTreeSet::new();
    for (relative, target) in &generation.links {
        let destination = stage.path().join(relative);
        let parent = destination.parent().ok_or("profile link has no parent")?;
        std::fs::create_dir_all(parent).map_err(|error| error.to_string())?;
        let mut directory = parent;
        while directory != stage.path() {
            directories.insert(directory.to_path_buf());
            directory = directory.parent().ok_or("profile directory escaped staging")?;
        }
        symlink(Path::new(target), &destination)?;
    }
    for directory in directories.iter().rev() {
        sync_dir(directory)?;
    }
    sync_dir(stage.path())?;
    assert!(stage.path().join("manifest.json").is_file());
    assert!(generation.number > 0);
    Ok(stage)
}

pub(super) fn publish(profile: &Path, stage: tempfile::TempDir, generation: u64) -> Result<(), String> {
    let destination = generation_path(profile, generation);
    if std::fs::symlink_metadata(&destination).is_ok() {
        return Err("profile generation already exists".into());
    }
    std::fs::rename(stage.path(), &destination).map_err(|error| error.to_string())?;
    sync_dir(&profile.join("generations"))?;
    sync_dir(profile)?;
    assert!(destination.is_dir());
    assert!(generation > 0);
    Ok(())
}

pub(super) fn switch(profile: &Path, generation: u64) -> Result<(), String> {
    let stage = tempfile::Builder::new().prefix(".switch-").tempdir_in(profile).map_err(|error| error.to_string())?;
    let temporary = stage.path().join("current");
    symlink(&PathBuf::from(format!("generations/{generation}")), &temporary)?;
    let previous = active(profile)?;
    let backup = stage.path().join("previous");
    if let Some(number) = previous {
        symlink(&PathBuf::from(format!("generations/{number}")), &backup)?;
    }
    sync_dir(stage.path())?;
    std::fs::rename(&temporary, profile.join("current")).map_err(|error| error.to_string())?;
    if let Err(error) = sync_dir(profile) {
        let restore = match previous {
            Some(_) => std::fs::rename(&backup, profile.join("current")),
            None => std::fs::remove_file(profile.join("current")),
        }
        .map_err(|restore| restore.to_string())
        .and_then(|()| sync_dir(profile));
        return match restore {
            Ok(()) => Err(format!("profile pointer sync failed; previous active state restored: {error}")),
            Err(restore) => Err(format!(
                "profile pointer sync and recovery failed; commit durability uncertain; inspect current before retry: {error}; {restore}"
            )),
        };
    }
    assert!(profile.is_absolute());
    assert!(generation > 0);
    Ok(())
}

pub(super) fn verify_forest(profile: &Path, generation: &Generation) -> Result<(), String> {
    let root = generation_path(profile, generation.number);
    let mut pending = vec![root.clone()];
    let mut found = std::collections::BTreeMap::new();
    let mut count = 0_usize;
    while let Some(directory) = pending.pop() {
        for entry in std::fs::read_dir(directory).map_err(|error| error.to_string())? {
            let entry = entry.map_err(|error| error.to_string())?;
            count = count.saturating_add(1);
            if count > crunch_composition_core::user_profile::ENTRIES_MAX {
                return Err("profile forest exceeds entry bound".into());
            }
            let path = entry.path();
            if path == root.join("manifest.json") {
                continue;
            }
            let metadata = std::fs::symlink_metadata(&path).map_err(|error| error.to_string())?;
            if metadata.is_dir() {
                pending.push(path);
                continue;
            }
            if !metadata.file_type().is_symlink() {
                return Err("profile forest contains an unexpected regular file".into());
            }
            let relative = path
                .strip_prefix(&root)
                .map_err(|error| error.to_string())?
                .to_str()
                .ok_or("non-UTF8 profile leaf")?
                .to_string();
            let target = std::fs::read_link(&path)
                .map_err(|error| error.to_string())?
                .into_os_string()
                .into_string()
                .map_err(|_| "non-UTF8 profile link")?;
            if found.len() >= crunch_composition_core::user_profile::ENTRIES_MAX {
                return Err("profile forest exceeds entry bound".into());
            }
            found.insert(relative, target);
        }
    }
    if found != generation.links {
        return Err("profile generation symlink forest was tampered".into());
    }
    assert!(pending.is_empty());
    assert_eq!(found, generation.links);
    Ok(())
}

pub(super) fn tombstone(profile: &Path, number: u64) -> Result<Generation, String> {
    let removed = profile.join(format!(".removed-{number}"));
    let source = generation_path(profile, number);
    if std::fs::symlink_metadata(&removed).is_ok() {
        if std::fs::symlink_metadata(&source).is_ok() {
            return Err("ambiguous generation and removal tombstone".into());
        }
        require_directory(&removed, false)?;
    } else if std::fs::symlink_metadata(&source).is_ok() {
        let generation = read_generation(profile, number)?;
        std::fs::rename(&source, &removed).map_err(|error| error.to_string())?;
        sync_dir(&profile.join("generations"))?;
        sync_dir(profile)?;
        return Ok(generation);
    } else {
        let generation: Generation = read_optional(&profile.join(format!(".pending-{number}.json")))?
            .ok_or("retained or pending generation not found")?;
        if generation.number != number || generation.schema != GENERATION_SCHEMA {
            return Err("invalid pending generation".into());
        }
        generation.manifest.validate().map_err(|error| error.to_string())?;
        let temporary = tempfile::Builder::new()
            .prefix(".removing-")
            .tempdir_in(profile)
            .map_err(|error| error.to_string())?;
        publish_new_json(&temporary.path().join("manifest.json"), &generation)?;
        std::fs::rename(temporary.path(), &removed).map_err(|error| error.to_string())?;
        sync_dir(profile)?;
        return Ok(generation);
    }
    let generation: Generation =
        read_optional(&removed.join("manifest.json"))?.ok_or("removal tombstone manifest missing")?;
    if generation.number != number || generation.schema != GENERATION_SCHEMA {
        return Err("invalid generation tombstone".into());
    }
    generation.manifest.validate().map_err(|error| error.to_string())?;
    assert_eq!(generation.number, number);
    assert!(number > 0);
    Ok(generation)
}

pub(super) fn generation_path(profile: &Path, generation: u64) -> PathBuf {
    profile.join("generations").join(generation.to_string())
}

pub(super) fn record_pending(profile: &Path, generation: &Generation) -> Result<(), String> {
    publish_new_json(&profile.join(format!(".pending-{}.json", generation.number)), generation)
}

pub(super) fn finish_pending(profile: &Path, number: u64) -> Result<(), String> {
    let path = profile.join(format!(".pending-{number}.json"));
    match std::fs::remove_file(&path) {
        Ok(()) => sync_dir(profile),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(error.to_string()),
    }
}

pub(super) fn pending_generations(profile: &Path) -> Result<Vec<u64>, String> {
    let mut pending = Vec::new();
    for entry in std::fs::read_dir(profile).map_err(|error| error.to_string())? {
        let entry = entry.map_err(|error| error.to_string())?;
        let name = entry.file_name();
        let Some(text) = name.to_str() else {
            continue;
        };
        let Some(suffix) = text.strip_prefix(".pending-") else {
            continue;
        };
        let Some(number) = suffix.strip_suffix(".json") else {
            continue;
        };
        let number = number.parse::<u64>().map_err(|error| error.to_string())?;
        if number == 0 || text != format!(".pending-{number}.json") {
            return Err("invalid pending generation name".into());
        }
        if pending.len() >= GENERATIONS_MAX {
            return Err("too many pending profile generations".into());
        }
        pending.push(number);
    }
    pending.sort_unstable();
    assert!(pending.len() <= GENERATIONS_MAX);
    assert!(pending.iter().all(|number| *number > 0));
    Ok(pending)
}

fn publish_new_json<T: Serialize>(path: &Path, value: &T) -> Result<(), String> {
    let parent = path.parent().ok_or("profile JSON has no parent")?;
    let mut temporary = tempfile::NamedTempFile::new_in(parent).map_err(|error| error.to_string())?;
    let bytes = serde_json::to_vec_pretty(value).map_err(|error| error.to_string())?;
    if bytes.len() as u64 > JSON_BYTES_MAX {
        return Err("profile JSON exceeds byte bound".into());
    }
    temporary.write_all(&bytes).map_err(|error| error.to_string())?;
    temporary.as_file().sync_all().map_err(|error| error.to_string())?;
    temporary.persist_noclobber(path).map_err(|error| error.to_string())?;
    sync_dir(parent)?;
    assert!(path.is_file());
    assert!(bytes.len() as u64 <= JSON_BYTES_MAX);
    Ok(())
}

fn replace_json<T: Serialize>(path: &Path, value: &T) -> Result<(), String> {
    let parent = path.parent().ok_or("profile JSON has no parent")?;
    let mut temporary = tempfile::NamedTempFile::new_in(parent).map_err(|error| error.to_string())?;
    let bytes = serde_json::to_vec(value).map_err(|error| error.to_string())?;
    temporary.write_all(&bytes).map_err(|error| error.to_string())?;
    temporary.as_file().sync_all().map_err(|error| error.to_string())?;
    temporary.persist(path).map_err(|error| error.to_string())?;
    sync_dir(parent)?;
    assert!(path.is_file());
    assert!(!bytes.is_empty());
    Ok(())
}

fn sync_dir(path: &Path) -> Result<(), String> {
    File::open(path)
        .and_then(|directory| directory.sync_all())
        .map_err(|error| format!("syncing {}: {error}", path.display()))
}

#[cfg(unix)]
fn symlink(target: &Path, destination: &Path) -> Result<(), String> {
    std::os::unix::fs::symlink(target, destination).map_err(|error| error.to_string())
}

#[cfg(not(unix))]
fn symlink(_target: &Path, _destination: &Path) -> Result<(), String> {
    Err("user profiles require Unix symlink publication".into())
}
