use std::io;
use std::path::Component;
use std::path::Path;

use cap_std::ambient_authority;
use cap_std::fs::Dir;

const MAX_RELEASE_RELATIVE_PATH_BYTES: usize = 4096;
const MAX_RELEASE_PATH_COMPONENTS: usize = 128;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ReleaseRootKind {
    ReleaseEvidence,
    WitnessRebuild,
    Bootstrap,
    BuildArtifact,
    Store,
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
        let dir = Dir::open_ambient_dir(root_path, ambient_authority())?;
        Ok(Self { kind, dir })
    }

    pub(crate) fn kind(&self) -> ReleaseRootKind {
        self.kind
    }

    pub(crate) fn read(&self, path: &ValidatedReleasePath) -> io::Result<Vec<u8>> {
        self.dir.read(path.as_str())
    }

    pub(crate) fn write(&self, path: &ValidatedReleasePath, bytes: &[u8]) -> io::Result<()> {
        self.dir.write(path.as_str(), bytes)
    }
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

fn validate_relative_release_path(path: &str) -> Result<(), ReleasePathError> {
    if path.is_empty() {
        return Err(ReleasePathError::EmptyPath);
    }
    if path.len() > MAX_RELEASE_RELATIVE_PATH_BYTES {
        return Err(ReleasePathError::TooLong);
    }
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
        let cases: [(&str, ReleasePathRequest, ReleasePathError); 4] = [
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
