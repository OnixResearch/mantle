//! Linux atomic no-replace rename shell.
//!
//! Rust's `libc` crate does not expose the `renameat2` function on every libc
//! target even when the Linux kernel syscall is available. Keep the unsafe ABI
//! boundary in one place and preserve `RENAME_NOREPLACE` semantics for every
//! publication caller.

#[cfg(target_os = "linux")]
use std::ffi::CStr;
#[cfg(target_os = "linux")]
use std::ffi::CString;
#[cfg(target_os = "linux")]
use std::fs::File;
#[cfg(target_os = "linux")]
use std::os::fd::AsRawFd;
#[cfg(target_os = "linux")]
use std::os::fd::RawFd;
#[cfg(target_os = "linux")]
use std::os::unix::ffi::OsStrExt;
use std::path::Path;

#[cfg(target_os = "linux")]
pub(crate) fn rename_no_replace(
    source_directory_fd: RawFd,
    source: &CStr,
    destination_directory_fd: RawFd,
    destination: &CStr,
) -> std::io::Result<()> {
    assert!(!source.to_bytes().is_empty(), "rename source must not be empty");
    assert!(!destination.to_bytes().is_empty(), "rename destination must not be empty");

    // SAFETY: both path pointers come from live CStr values. The directory file
    // descriptors are supplied by the caller and remain live for the call.
    // Linux RENAME_NOREPLACE makes publication one race-free no-clobber step.
    let result = unsafe {
        libc::syscall(
            libc::SYS_renameat2,
            source_directory_fd,
            source.as_ptr(),
            destination_directory_fd,
            destination.as_ptr(),
            libc::RENAME_NOREPLACE,
        )
    };
    if result == 0 {
        return Ok(());
    }
    Err(std::io::Error::last_os_error())
}

#[cfg(target_os = "linux")]
pub(crate) fn rename_path_no_replace(source: &Path, destination: &Path) -> std::io::Result<()> {
    assert!(!source.as_os_str().is_empty(), "rename source path must not be empty");
    assert!(!destination.as_os_str().is_empty(), "rename destination path must not be empty");
    let source_parent = source
        .parent()
        .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::InvalidInput, "rename source has no parent"))?;
    let destination_parent = destination
        .parent()
        .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::InvalidInput, "rename destination has no parent"))?;
    let source_name = source
        .file_name()
        .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::InvalidInput, "rename source has no file name"))?;
    let destination_name = destination
        .file_name()
        .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::InvalidInput, "rename destination has no file name"))?;
    let source_directory = File::open(source_parent)?;
    let destination_directory = File::open(destination_parent)?;
    let source_name = CString::new(source_name.as_bytes())
        .map_err(|_| std::io::Error::new(std::io::ErrorKind::InvalidInput, "rename source contains NUL"))?;
    let destination_name = CString::new(destination_name.as_bytes())
        .map_err(|_| std::io::Error::new(std::io::ErrorKind::InvalidInput, "rename destination contains NUL"))?;
    rename_no_replace(source_directory.as_raw_fd(), &source_name, destination_directory.as_raw_fd(), &destination_name)
}

#[cfg(not(target_os = "linux"))]
pub(crate) fn rename_path_no_replace(_source: &Path, _destination: &Path) -> std::io::Result<()> {
    Err(std::io::Error::new(
        std::io::ErrorKind::Unsupported,
        "atomic no-replace path publication requires Linux renameat2",
    ))
}

#[cfg(all(test, target_os = "linux"))]
mod tests {
    use std::ffi::CString;
    use std::fs;
    use std::fs::File;
    use std::os::fd::AsRawFd;

    const SOURCE_NAME: &str = "source";
    const DESTINATION_NAME: &str = "destination";
    const SOURCE_CONTENT: &str = "source-content";
    const DESTINATION_CONTENT: &str = "destination-content";

    #[test]
    fn rename_no_replace_moves_source_atomically_when_destination_is_absent() {
        let root = tempfile::tempdir().unwrap();
        let parent = File::open(root.path()).unwrap();
        let source_path = root.path().join(SOURCE_NAME);
        let destination_path = root.path().join(DESTINATION_NAME);
        fs::write(&source_path, SOURCE_CONTENT).unwrap();
        assert!(source_path.is_file());
        assert!(!destination_path.exists());

        super::rename_no_replace(
            parent.as_raw_fd(),
            &CString::new(SOURCE_NAME).unwrap(),
            parent.as_raw_fd(),
            &CString::new(DESTINATION_NAME).unwrap(),
        )
        .unwrap();

        assert!(!source_path.exists());
        assert_eq!(fs::read_to_string(destination_path).unwrap(), SOURCE_CONTENT);
    }

    #[test]
    fn rename_no_replace_preserves_both_files_when_destination_exists() {
        let root = tempfile::tempdir().unwrap();
        let parent = File::open(root.path()).unwrap();
        let source_path = root.path().join(SOURCE_NAME);
        let destination_path = root.path().join(DESTINATION_NAME);
        fs::write(&source_path, SOURCE_CONTENT).unwrap();
        fs::write(&destination_path, DESTINATION_CONTENT).unwrap();
        assert!(source_path.is_file());
        assert!(destination_path.is_file());

        let error = super::rename_no_replace(
            parent.as_raw_fd(),
            &CString::new(SOURCE_NAME).unwrap(),
            parent.as_raw_fd(),
            &CString::new(DESTINATION_NAME).unwrap(),
        )
        .unwrap_err();

        assert_eq!(error.kind(), std::io::ErrorKind::AlreadyExists);
        assert_eq!(fs::read_to_string(source_path).unwrap(), SOURCE_CONTENT);
        assert_eq!(fs::read_to_string(destination_path).unwrap(), DESTINATION_CONTENT);
    }
}
