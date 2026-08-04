use std::ffi::OsString;
use std::io;
use std::io::Write;
use std::path::Component;
use std::path::Path;
use std::path::PathBuf;

use cap_fs_ext::DirExt;
use cap_fs_ext::FollowSymlinks;
use cap_fs_ext::OpenOptionsFollowExt;
use cap_std::ambient_authority;
use cap_std::fs::Dir;
use cap_std::fs::File;
use cap_std::fs::OpenOptions;
#[cfg(unix)]
use cap_std::fs::OpenOptionsExt;

const MAX_RELEASE_RELATIVE_PATH_BYTES: usize = 4_096;
const MAX_RELEASE_PATH_COMPONENTS: usize = 128;
#[cfg(unix)]
const PRIVATE_STAGING_FILE_MODE: u32 = 0o600;

const _: () = {
    assert!(MAX_RELEASE_RELATIVE_PATH_BYTES > 0);
    assert!(MAX_RELEASE_PATH_COMPONENTS > 0);
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ReleaseRootKind {
    ReleaseEvidence,
    ContentBoundRequirementEvidence,
    ReleaseTreeSource,
    ReleaseTreeDestination,
    WitnessRebuild,
    Bootstrap,
    BuildArtifact,
    Store,
    RemoteFailureCapture,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ValidatedReleasePath(String);

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ReleasePathError {
    EmptyPath,
    AbsolutePath,
    ParentTraversal,
    PrefixOrRootComponent,
    TooLong,
    TooManyComponents,
    MissingRootAuthority,
    WrongRootAuthority {
        expected: ReleaseRootKind,
        actual: ReleaseRootKind,
    },
}

#[derive(Debug)]
pub(crate) struct ReleaseCapabilityRoot {
    kind: ReleaseRootKind,
    dir: Dir,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ReleasePathRequest {
    pub required_root: ReleaseRootKind,
    pub available_root: Option<ReleaseRootKind>,
    pub relative_path: String,
}

impl ValidatedReleasePath {
    pub(crate) fn new(path: &str) -> Result<Self, ReleasePathError> {
        validate_relative_release_path(path)?;
        Ok(Self(path.to_string()))
    }

    pub(crate) fn as_str(&self) -> &str {
        &self.0
    }
}

impl ReleaseCapabilityRoot {
    pub(crate) fn open_ambient(kind: ReleaseRootKind, root_path: &Path) -> io::Result<Self> {
        Self::open_ambient_nofollow(kind, root_path)
    }

    pub(crate) fn open_ambient_nofollow(kind: ReleaseRootKind, root_path: &Path) -> io::Result<Self> {
        let dir = walk_ambient_directory_nofollow(root_path, false)?;
        Ok(Self { kind, dir })
    }

    pub(crate) fn create_ambient_dir_all_nofollow(kind: ReleaseRootKind, root_path: &Path) -> io::Result<Self> {
        let dir = walk_ambient_directory_nofollow(root_path, true)?;
        Ok(Self { kind, dir })
    }

    pub(crate) fn from_open_dir(kind: ReleaseRootKind, dir: Dir) -> Self {
        Self { kind, dir }
    }

    pub(crate) fn kind(&self) -> ReleaseRootKind {
        self.kind
    }

    pub(crate) fn dir(&self) -> &Dir {
        &self.dir
    }

    pub(crate) fn is_empty(&self) -> io::Result<bool> {
        Ok(self.dir.entries()?.next().is_none())
    }

    pub(crate) fn read(&self, path: &ValidatedReleasePath) -> io::Result<Vec<u8>> {
        self.dir.read(path.as_str())
    }

    pub(crate) fn open_file_read_nofollow(&self, path: &ValidatedReleasePath) -> io::Result<File> {
        let mut options = OpenOptions::new();
        options.read(true);
        options.follow(FollowSymlinks::No);
        #[cfg(unix)]
        options.custom_flags(libc::O_CLOEXEC | libc::O_NONBLOCK);
        debug_assert!(!path.as_str().is_empty());
        debug_assert!(!Path::new(path.as_str()).is_absolute());
        self.dir.open_with(path.as_str(), &options)
    }

    pub(crate) fn write(&self, path: &ValidatedReleasePath, bytes: &[u8]) -> io::Result<()> {
        self.dir.write(path.as_str(), bytes)
    }

    pub(crate) fn create_dir_all_relative_nofollow(
        &self,
        kind: ReleaseRootKind,
        path: &ValidatedReleasePath,
    ) -> io::Result<Self> {
        let dir = walk_relative_directory_nofollow(&self.dir, path, true)?;
        Ok(Self::from_open_dir(kind, dir))
    }

    pub(crate) fn open_new_file_nofollow(&self, path: &ValidatedReleasePath) -> io::Result<File> {
        let (parent, name) = open_or_create_relative_parent(&self.dir, path)?;
        let mut options = OpenOptions::new();
        options.write(true).create_new(true);
        options.follow(FollowSymlinks::No);
        #[cfg(unix)]
        options.mode(PRIVATE_STAGING_FILE_MODE);
        parent.open_with(name, &options)
    }

    pub(crate) fn write_new_file_nofollow(&self, path: &ValidatedReleasePath, bytes: &[u8]) -> io::Result<()> {
        debug_assert!(!path.as_str().is_empty());
        debug_assert!(!Path::new(path.as_str()).is_absolute());
        let mut file = self.open_new_file_nofollow(path)?;
        file.write_all(bytes)?;
        file.flush()?;
        Ok(())
    }

    pub(crate) fn create_symlink_nofollow(&self, path: &ValidatedReleasePath, target: &str) -> io::Result<()> {
        let (parent, name) = open_or_create_relative_parent(&self.dir, path)?;
        match parent.symlink_metadata(&name) {
            Err(error) if error.kind() == io::ErrorKind::NotFound => {}
            Err(error) => return Err(error),
            Ok(_) => {
                return Err(io::Error::new(
                    io::ErrorKind::AlreadyExists,
                    format!("release symlink destination already exists: {}", path.as_str()),
                ));
            }
        }
        DirExt::symlink(&parent, target, &name)?;
        debug_assert!(!path.as_str().is_empty());
        debug_assert!(!target.is_empty());
        Ok(())
    }

    pub(crate) fn remove_file_nofollow(&self, path: &ValidatedReleasePath) -> io::Result<()> {
        let (parent, name) = open_existing_relative_parent(&self.dir, path)?;
        parent.remove_file(name)
    }
}

fn walk_relative_directory_nofollow(root: &Dir, path: &ValidatedReleasePath, create_missing: bool) -> io::Result<Dir> {
    let mut current = root.try_clone()?;
    let components = validated_normal_components(path)?;
    for component in components {
        current = open_or_create_child_directory(&current, &component, create_missing)?;
    }
    Ok(current)
}

fn open_or_create_relative_parent(root: &Dir, path: &ValidatedReleasePath) -> io::Result<(Dir, OsString)> {
    let mut components = validated_normal_components(path)?;
    let name = components
        .pop()
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "release path is empty"))?;
    let mut parent = root.try_clone()?;
    for component in components {
        parent = open_or_create_child_directory(&parent, &component, true)?;
    }
    Ok((parent, name))
}

fn open_existing_relative_parent(root: &Dir, path: &ValidatedReleasePath) -> io::Result<(Dir, OsString)> {
    debug_assert!(!path.as_str().is_empty());
    debug_assert!(!Path::new(path.as_str()).is_absolute());
    let mut components = validated_normal_components(path)?;
    let name = components
        .pop()
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "release path is empty"))?;
    let mut parent = root.try_clone()?;
    for component in components {
        parent = open_or_create_child_directory(&parent, &component, false)?;
    }
    Ok((parent, name))
}

fn validated_normal_components(path: &ValidatedReleasePath) -> io::Result<Vec<OsString>> {
    debug_assert!(!path.as_str().is_empty());
    debug_assert!(!Path::new(path.as_str()).is_absolute());
    let components = Path::new(path.as_str())
        .components()
        .filter_map(|component| match component {
            Component::Normal(name) => Some(Ok(name.to_os_string())),
            Component::CurDir => None,
            _ => Some(Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("release path is not normalized: {}", path.as_str()),
            ))),
        })
        .collect::<io::Result<Vec<_>>>()?;
    if components.is_empty() {
        return Err(io::Error::new(io::ErrorKind::InvalidInput, "release path has no normal components"));
    }
    Ok(components)
}

// r[impl mantle.release_provenance.cap_std_boundary.root_wrappers]
// r[impl mantle.release_provenance.cap_std_boundary.conversion]
pub(crate) fn authorize_release_path(request: &ReleasePathRequest) -> Result<ValidatedReleasePath, ReleasePathError> {
    let actual = request.available_root.ok_or(ReleasePathError::MissingRootAuthority)?;
    if actual != request.required_root {
        return Err(ReleasePathError::WrongRootAuthority {
            expected: request.required_root,
            actual,
        });
    }
    ValidatedReleasePath::new(&request.relative_path)
}

fn walk_ambient_directory_nofollow(root_path: &Path, create_missing: bool) -> io::Result<Dir> {
    let (anchor, components) = absolute_anchor_and_components(root_path)?;
    let mut current = Dir::open_ambient_dir(&anchor, ambient_authority())?;
    for component in components {
        current = open_or_create_child_directory(&current, &component, create_missing)?;
    }
    Ok(current)
}

fn absolute_anchor_and_components(path: &Path) -> io::Result<(PathBuf, Vec<OsString>)> {
    let absolute = std::path::absolute(path)?;
    let component_count = absolute.components().count();
    let mut anchor = PathBuf::new();
    let mut components = Vec::with_capacity(component_count);
    for component in absolute.components() {
        match component {
            Component::Prefix(prefix) => anchor.push(prefix.as_os_str()),
            Component::RootDir => anchor.push(component.as_os_str()),
            Component::Normal(name) => components.push(name.to_os_string()),
            Component::CurDir => {}
            Component::ParentDir => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    format!("capability root path retains parent traversal: {}", absolute.display()),
                ));
            }
        }
    }
    if anchor.as_os_str().is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("capability root path has no absolute anchor: {}", absolute.display()),
        ));
    }
    debug_assert!(absolute.is_absolute());
    debug_assert!(components.len() <= component_count);
    Ok((anchor, components))
}

fn open_or_create_child_directory(parent: &Dir, name: &OsString, create_missing: bool) -> io::Result<Dir> {
    debug_assert!(!name.is_empty());
    debug_assert_eq!(Path::new(name).components().count(), 1);
    match parent.symlink_metadata(name) {
        Ok(metadata) => validate_real_directory(&metadata, name)?,
        Err(error) if error.kind() == io::ErrorKind::NotFound && create_missing => match parent.create_dir(name) {
            Ok(()) => {}
            Err(create_error) if create_error.kind() == io::ErrorKind::AlreadyExists => {}
            Err(create_error) => return Err(create_error),
        },
        Err(error) => return Err(error),
    }
    parent.open_dir_nofollow(name)
}

fn validate_real_directory(metadata: &cap_std::fs::Metadata, name: &OsString) -> io::Result<()> {
    if metadata.file_type().is_symlink() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("capability root path component is a symlink: {}", Path::new(name).display()),
        ));
    }
    if !metadata.is_dir() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("capability root path component is not a directory: {}", Path::new(name).display()),
        ));
    }
    Ok(())
}

fn validate_relative_release_path(path: &str) -> Result<(), ReleasePathError> {
    if path.is_empty() {
        return Err(ReleasePathError::EmptyPath);
    }
    if path.len() > MAX_RELEASE_RELATIVE_PATH_BYTES {
        return Err(ReleasePathError::TooLong);
    }
    debug_assert!(!path.is_empty());
    debug_assert!(path.len() <= MAX_RELEASE_RELATIVE_PATH_BYTES);
    let mut component_count = 0_usize;
    for component in Path::new(path).components() {
        component_count = component_count.saturating_add(1);
        if component_count > MAX_RELEASE_PATH_COMPONENTS {
            return Err(ReleasePathError::TooManyComponents);
        }
        match component {
            Component::Normal(_) => {}
            Component::CurDir => {}
            Component::ParentDir => return Err(ReleasePathError::ParentTraversal),
            Component::RootDir => return Err(ReleasePathError::AbsolutePath),
            Component::Prefix(_) => return Err(ReleasePathError::PrefixOrRootComponent),
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    const PATH_NEGATIVE_CASES_COUNT: usize = 4;

    #[test]
    fn accepts_relative_paths_under_declared_roots() {
        let request = ReleasePathRequest {
            required_root: ReleaseRootKind::ReleaseEvidence,
            available_root: Some(ReleaseRootKind::ReleaseEvidence),
            relative_path: "evidence/manifest.json".to_string(),
        };

        let path = authorize_release_path(&request).expect("relative path should authorize");

        assert_eq!(path.as_str(), "evidence/manifest.json");
    }

    #[test]
    fn rejects_path_and_authority_negative_fixtures() {
        let cases: [(&str, ReleasePathRequest, ReleasePathError); PATH_NEGATIVE_CASES_COUNT] = [
            (
                "parent traversal",
                request("../secret", Some(ReleaseRootKind::ReleaseEvidence)),
                ReleasePathError::ParentTraversal,
            ),
            (
                "absolute path",
                request("/tmp/secret", Some(ReleaseRootKind::ReleaseEvidence)),
                ReleasePathError::AbsolutePath,
            ),
            ("missing authority", request("manifest.json", None), ReleasePathError::MissingRootAuthority),
            (
                "wrong authority",
                request("manifest.json", Some(ReleaseRootKind::Store)),
                ReleasePathError::WrongRootAuthority {
                    expected: ReleaseRootKind::ReleaseEvidence,
                    actual: ReleaseRootKind::Store,
                },
            ),
        ];

        for (name, candidate, expected) in cases {
            let err = authorize_release_path(&candidate).expect_err(name);
            assert_eq!(err, expected, "{name}");
        }
    }

    #[test]
    #[cfg(unix)]
    fn cap_std_root_rejects_symlink_escape_reads() {
        let temp = tempfile::tempdir().expect("tempdir");
        let root_path = temp.path().join("root");
        let outside_path = temp.path().join("outside");
        std::fs::create_dir(&root_path).expect("root dir");
        std::fs::create_dir(&outside_path).expect("outside dir");
        std::fs::write(outside_path.join("secret.txt"), b"secret").expect("outside secret");
        std::os::unix::fs::symlink(&outside_path, root_path.join("link")).expect("symlink");
        let root = ReleaseCapabilityRoot::open_ambient(ReleaseRootKind::ReleaseEvidence, &root_path).expect("cap root");
        let path = ValidatedReleasePath::new("link/secret.txt").expect("relative path");

        let read = root.read(&path);

        assert!(read.is_err(), "capability read should reject symlink escape");
    }

    #[test]
    #[cfg(unix)]
    fn nofollow_root_open_rejects_symlink_root_and_parent() {
        let temp = tempfile::tempdir().expect("tempdir");
        let real_root = temp.path().join("real-root");
        let root_link = temp.path().join("root-link");
        let parent_link = temp.path().join("parent-link");
        std::fs::create_dir(&real_root).expect("real root");
        std::os::unix::fs::symlink(&real_root, &root_link).expect("root symlink");
        std::os::unix::fs::symlink(temp.path(), &parent_link).expect("parent symlink");

        let root_error = ReleaseCapabilityRoot::open_ambient_nofollow(ReleaseRootKind::ReleaseEvidence, &root_link);
        let parent_error = ReleaseCapabilityRoot::create_ambient_dir_all_nofollow(
            ReleaseRootKind::ReleaseEvidence,
            &parent_link.join("child"),
        );

        assert!(root_error.is_err(), "symlink root must be rejected");
        assert!(parent_error.is_err(), "symlink parent must be rejected");
    }

    #[test]
    fn nofollow_root_creation_builds_and_opens_real_directory_chain() {
        let temp = tempfile::tempdir().expect("tempdir");
        let root_path = temp.path().join("nested/root");
        let root =
            ReleaseCapabilityRoot::create_ambient_dir_all_nofollow(ReleaseRootKind::ReleaseTreeDestination, &root_path)
                .expect("cap root");

        assert_eq!(root.kind(), ReleaseRootKind::ReleaseTreeDestination);
        assert!(root.is_empty().expect("empty root"));
        assert!(std::fs::symlink_metadata(root_path).expect("root metadata").is_dir());
    }

    #[test]
    fn cap_std_root_reads_and_writes_relative_files() {
        let temp = tempfile::tempdir().expect("tempdir");
        let root = ReleaseCapabilityRoot::open_ambient(ReleaseRootKind::BuildArtifact, temp.path()).expect("cap root");
        let path = ValidatedReleasePath::new("artifact.bin").expect("relative path");

        root.write(&path, b"artifact").expect("write relative file");
        let bytes = root.read(&path).expect("read relative file");

        assert_eq!(root.kind(), ReleaseRootKind::BuildArtifact);
        assert_eq!(bytes, b"artifact");
    }

    fn request(path: &str, available_root: Option<ReleaseRootKind>) -> ReleasePathRequest {
        ReleasePathRequest {
            required_root: ReleaseRootKind::ReleaseEvidence,
            available_root,
            relative_path: path.to_string(),
        }
    }
}
