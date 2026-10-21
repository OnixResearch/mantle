//! Actual command observations; no original executable instrumentation or guessed
//! stdio offsets. The wire records match the source-owned controller protocol.
use std::fs::File;
use std::fs::Metadata;
use std::fs::OpenOptions;
use std::fs::{self};
use std::io::Seek;
use std::io::SeekFrom;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;
use std::sync::Arc;

use super::CommandOperands;
use super::create;
use super::error;
use super::rooted_directory;
use crate::errors::RunError;
use crate::source_built_recipe::RECIPE_BYTES_MAX;
use crate::source_built_recipe::RecipeCommandState;
use crate::source_built_recipe::RecipeDescriptor;
use crate::source_built_recipe::RecipeDescriptorAccess;
use crate::source_built_recipe::RecipeDescriptorTarget;
use crate::source_built_recipe::RecipeInvocation;
use crate::source_built_recipe::RecipeStep;
use crate::source_built_recipe::RecipeWord;
use crate::source_built_recipe::WordPart;
use crate::source_built_recipe::relative;
use crate::source_built_recipe::source_member;

const ENTRY_COUNT_MAX: u32 = 262_144;
const TIMES_MAGIC: &str = "MANTLE-WORKSPACE-TIMES-1";
const DESCRIPTORS_MAGIC: &str = "MANTLE-COMMAND-DESCRIPTORS-2";

const DESCRIPTOR_COUNT: usize = 64;
struct DescriptorFiles {
    slots: [Option<File>; DESCRIPTOR_COUNT],
    closed: File,
    snapshots: [bool; DESCRIPTOR_COUNT],
}
#[derive(Clone)]
pub(super) struct Descriptors(Arc<DescriptorFiles>);
#[derive(Clone, Copy)]
pub(super) struct DescriptorInjection {
    pub source_fd: i32,
    pub target_fd: u32,
    pub close_on_exec: bool,
}
impl Descriptors {
    pub(super) fn retain(input: &File, output: &File, error_output: &File) -> Result<Self, RunError> {
        let mut slots = std::array::from_fn(|_| None);
        for (slot, file) in slots.iter_mut().zip([input, output, error_output]) {
            *slot = Some(file.try_clone().map_err(|failure| error(failure.to_string()))?);
        }
        Self::from_slots(slots, [false; DESCRIPTOR_COUNT])
    }
    fn from_slots(
        slots: [Option<File>; DESCRIPTOR_COUNT],
        snapshots: [bool; DESCRIPTOR_COUNT],
    ) -> Result<Self, RunError> {
        let closed = File::open("/dev/null").map_err(|failure| error(failure.to_string()))?;
        Ok(Self(Arc::new(DescriptorFiles {
            slots,
            closed,
            snapshots,
        })))
    }
    pub(super) fn prepare(
        invocation: &RecipeInvocation,
        output_dir: &Path,
        operands: &CommandOperands<'_>,
    ) -> Result<Self, RunError> {
        let table = invocation.descriptors.as_ref().ok_or_else(|| error("missing original descriptor table"))?;
        if table.slots.len() != DESCRIPTOR_COUNT {
            return Err(error("original descriptor table must have exactly 64 slots"));
        }
        let mut slots: [Option<File>; DESCRIPTOR_COUNT] = std::array::from_fn(|_| None);
        let mut snapshots = [false; DESCRIPTOR_COUNT];
        for (index, entry) in table.slots.iter().enumerate() {
            let Some(entry) = entry else {
                continue;
            };
            let alias = usize::from(entry.alias);
            if alias > index {
                return Err(error("original descriptor alias is not backward or self"));
            }
            snapshots[index] = alias == index && matches!(entry.target, RecipeDescriptorTarget::File { .. });
            let file = if alias == index {
                match &entry.target {
                    RecipeDescriptorTarget::File { path } => open_descriptor(
                        &descriptor_path(path, entry.access, invocation, output_dir, operands)?,
                        entry,
                        false,
                    )?,
                    RecipeDescriptorTarget::NullDevice => open_descriptor(Path::new("/dev/null"), entry, true)?,
                }
            } else {
                if table.slots[alias].as_ref() != Some(entry) {
                    return Err(error("original descriptor alias changes OFD attributes"));
                }
                slots[alias]
                    .as_ref()
                    .ok_or_else(|| error("original descriptor aliases a closed slot"))?
                    .try_clone()
                    .map_err(|failure| error(failure.to_string()))?
            };
            slots[index] = Some(file);
        }
        Self::from_slots(slots, snapshots)
    }
    #[cfg(unix)]
    pub(super) fn injections(&self) -> [DescriptorInjection; DESCRIPTOR_COUNT] {
        use std::os::fd::AsRawFd;
        std::array::from_fn(|index| DescriptorInjection {
            source_fd: self.0.slots[index].as_ref().unwrap_or(&self.0.closed).as_raw_fd(),
            target_fd: index as u32,
            close_on_exec: self.0.slots[index].is_none(),
        })
    }
}
fn descriptor_path(
    word: &RecipeWord,
    access: RecipeDescriptorAccess,
    invocation: &RecipeInvocation,
    output_dir: &Path,
    operands: &CommandOperands<'_>,
) -> Result<PathBuf, RunError> {
    let writing = access != RecipeDescriptorAccess::Read;
    let (root, member) = match word.parts.as_slice() {
        [WordPart::Input { name, path }] => {
            if writing {
                return Err(error("original descriptor cannot write a readonly input"));
            }
            let binding = operands.binding(&operands.step.inputs[name]);
            operands.authenticate_produced(name, binding.kind, "original descriptor input")?;
            (binding.path.clone(), path.as_str())
        }
        [WordPart::Runtime { mount, path }] => {
            if writing {
                return Err(error("original descriptor cannot write a readonly runtime"));
            }
            let crate::source_built_recipe::RecipeOperation::Exec { runtime_mounts, .. } = &operands.step.operation
            else {
                return Err(error("original descriptor runtime outside command"));
            };
            let name = &runtime_mounts[mount];
            let binding = operands.binding(&operands.step.inputs[name]);
            operands.authenticate_produced(name, binding.kind, "original descriptor runtime")?;
            (binding.path.clone(), path.as_str())
        }
        [WordPart::Output { name, path }] => (output_dir.join(name), path.as_str()),
        [WordPart::Workspace { mount, path }] => {
            let mapping = invocation
                .workspace_mounts
                .get(mount)
                .ok_or_else(|| error("original descriptor unknown workspace"))?;
            (rooted_directory(&output_dir.join(&mapping.output), &mapping.path)?, path.as_str())
        }
        _ => return Err(error("original descriptor path requires one typed file operand")),
    };
    source_member(&root, member)
}
#[cfg(target_os = "linux")]
fn open_descriptor(path: &Path, entry: &RecipeDescriptor, null_device: bool) -> Result<File, RunError> {
    use std::os::fd::AsRawFd;
    use std::os::unix::fs::MetadataExt;
    use std::os::unix::fs::OpenOptionsExt;
    if entry.flags & !((libc::O_APPEND | libc::O_NONBLOCK) as u32) != 0 || entry.offset > i64::MAX as u64 {
        return Err(error("unsupported original descriptor flags or offset"));
    }
    // Lookup protections are private: O_NOFOLLOW itself is observable through
    // F_GETFL and must not be added to the original delivered description.
    let guard = OpenOptions::new()
        .read(true)
        .custom_flags(libc::O_PATH | libc::O_NOFOLLOW)
        .open(path)
        .map_err(|failure| error(failure.to_string()))?;
    let before = guard.metadata().map_err(|failure| error(failure.to_string()))?;
    if null_device {
        if !is_null_device(&before) || entry.offset != 0 {
            return Err(error("original null descriptor is not the exact zero-offset null device"));
        }
    } else if !before.is_file() {
        return Err(error("original descriptor snapshot is not a regular file"));
    }
    let mut options = OpenOptions::new();
    options
        .read(entry.access != RecipeDescriptorAccess::Write)
        .write(entry.access != RecipeDescriptorAccess::Read);
    let mut file = options
        .custom_flags(entry.flags as i32)
        .open(format!("/proc/self/fd/{}", guard.as_raw_fd()))
        .map_err(|failure| error(failure.to_string()))?;
    let after = file.metadata().map_err(|failure| error(failure.to_string()))?;
    if (null_device && !is_null_device(&after))
        || (!null_device && !after.is_file())
        || before.dev() != after.dev()
        || before.ino() != after.ino()
    {
        return Err(error("original descriptor snapshot changed during open"));
    }
    file.seek(SeekFrom::Start(entry.offset)).map_err(|failure| error(failure.to_string()))?;
    Ok(file)
}
#[cfg(not(target_os = "linux"))]
fn open_descriptor(_path: &Path, _entry: &RecipeDescriptor, _null_device: bool) -> Result<File, RunError> {
    Err(error("original descriptor transport requires Linux"))
}
#[cfg(target_os = "linux")]
fn is_null_device(metadata: &Metadata) -> bool {
    use std::os::unix::fs::FileTypeExt;
    use std::os::unix::fs::MetadataExt;
    metadata.file_type().is_char_device() && libc::major(metadata.rdev()) == 1 && libc::minor(metadata.rdev()) == 3
}
#[cfg(not(target_os = "linux"))]
fn is_null_device(_metadata: &Metadata) -> bool {
    false
}

fn reserve(remaining: &mut u64, bytes: u64) -> Result<(), RunError> {
    debug_assert!(*remaining <= RECIPE_BYTES_MAX);
    debug_assert!(bytes <= RECIPE_BYTES_MAX);
    let length_bytes = u64::from(bytes.checked_ilog10().unwrap_or(0)) + 1;
    *remaining = remaining
        .checked_sub(bytes + length_bytes + 2)
        .ok_or_else(|| error("command observation exceeds its declared output bound"))?;
    Ok(())
}
fn record(file: &mut File, value: &str, remaining: &mut u64) -> Result<(), RunError> {
    let bytes = value.len() as u64;
    reserve(remaining, bytes)?;
    write!(file, "{bytes}:{value},").map_err(|failure| error(failure.to_string()))
}
fn number(file: &mut File, value: i64, remaining: &mut u64) -> Result<(), RunError> {
    let bytes = value.unsigned_abs().checked_ilog10().unwrap_or(0) + 1 + u32::from(value < 0);
    reserve(remaining, u64::from(bytes))?;
    write!(file, "{bytes}:{value},").map_err(|failure| error(failure.to_string()))
}
fn take<'a>(input: &mut &'a [u8]) -> Result<&'a str, RunError> {
    let bytes = *input;
    let delimiter =
        bytes.iter().position(|byte| *byte == b':').ok_or_else(|| error("command state record delimiter"))?;
    let digits = &bytes[..delimiter];
    if digits.is_empty() || (digits.len() > 1 && digits[0] == b'0') || !digits.iter().all(u8::is_ascii_digit) {
        return Err(error("noncanonical command state record length"));
    }
    let length = std::str::from_utf8(digits)
        .map_err(|failure| error(failure.to_string()))?
        .parse::<u32>()
        .map_err(|failure| error(failure.to_string()))?;
    if u64::from(length) > RECIPE_BYTES_MAX {
        return Err(error("command state record bound"));
    }
    let (value, tail) = bytes[delimiter + 1..]
        .split_at_checked(length as usize)
        .ok_or_else(|| error("truncated command state record"))?;
    if tail.first() != Some(&b',') || value.contains(&0) {
        return Err(error("invalid command state record bytes"));
    }
    *input = &tail[1..];
    std::str::from_utf8(value).map_err(|failure| error(failure.to_string()))
}
fn integer(input: &mut &[u8]) -> Result<i64, RunError> {
    let value = take(input)?;
    let digits = value.strip_prefix('-').unwrap_or(value);
    if digits.is_empty()
        || !digits.bytes().all(|byte| byte.is_ascii_digit())
        || (digits.starts_with('0') && digits.len() != 1)
        || value == "-0"
    {
        return Err(error("noncanonical command state integer"));
    }
    value.parse().map_err(|failure| error(format!("command state integer: {failure}")))
}
fn walk(
    root: &Path,
    node: &mut PathBuf,
    count: &mut u32,
    visit: &mut impl FnMut(&Path, &str, &Metadata) -> Result<(), RunError>,
) -> Result<(), RunError> {
    debug_assert!(node.starts_with(root));
    debug_assert!(*count <= ENTRY_COUNT_MAX);
    *count = count
        .checked_add(1)
        .filter(|value| *value <= ENTRY_COUNT_MAX)
        .ok_or_else(|| error("command workspace node bound"))?;
    let metadata = fs::symlink_metadata(&node).map_err(|failure| error(failure.to_string()))?;
    if !metadata.is_file() && !metadata.is_dir() && !metadata.is_symlink() {
        return Err(error("unsupported command workspace member kind"));
    }
    if metadata.is_dir() {
        let mut names = Vec::new();
        for entry in fs::read_dir(&node).map_err(|failure| error(failure.to_string()))? {
            if names.len() as u64 >= u64::from(ENTRY_COUNT_MAX - *count) {
                return Err(error("command workspace directory node bound"));
            }
            names.push(entry.map_err(|failure| error(failure.to_string()))?.file_name());
        }
        names.sort_unstable();
        for name in names {
            node.push(name);
            walk(root, node, count, visit)?;
            node.pop();
        }
    }
    let member = node
        .strip_prefix(root)
        .map_err(|failure| error(failure.to_string()))?
        .to_str()
        .ok_or_else(|| error("command workspace member is not UTF-8"))?;
    relative(member, true)?;
    visit(node, member, &metadata)
}
#[cfg(unix)]
fn set_mtime(path: &Path, seconds: i64, nanoseconds: i64) -> Result<(), RunError> {
    use std::os::unix::ffi::OsStrExt;
    if !(0..1_000_000_000).contains(&nanoseconds) {
        return Err(error("invalid command workspace nanoseconds"));
    }
    let path = std::ffi::CString::new(path.as_os_str().as_bytes()).map_err(|failure| error(failure.to_string()))?;
    let times = [
        libc::timespec {
            tv_sec: 0,
            tv_nsec: libc::UTIME_OMIT,
        },
        libc::timespec {
            tv_sec: seconds,
            tv_nsec: nanoseconds,
        },
    ];
    // walk has lstat-checked every ancestor. The leaf may itself be a symlink;
    // never open it or change the target's timestamps.
    if unsafe { libc::utimensat(libc::AT_FDCWD, path.as_ptr(), times.as_ptr(), libc::AT_SYMLINK_NOFOLLOW) } != 0 {
        return Err(error(format!("restoring command workspace mtime: {}", std::io::Error::last_os_error())));
    }
    Ok(())
}
#[cfg(not(unix))]
fn set_mtime(_path: &Path, _seconds: i64, _nanoseconds: i64) -> Result<(), RunError> {
    Err(error("original command workspace times require Unix"))
}
fn restore(root: &Path, bytes: &[u8]) -> Result<(), RunError> {
    let mut input = bytes;
    if take(&mut input)? != TIMES_MAGIC {
        return Err(error("command workspace time manifest version"));
    }
    walk(root, &mut root.to_path_buf(), &mut 0, &mut |path, member, _| {
        if take(&mut input)? != member {
            return Err(error("command workspace time inventory differs from actual seed"));
        }
        let seconds = integer(&mut input)?;
        let nanoseconds = integer(&mut input)?;
        set_mtime(path, seconds, nanoseconds)
    })?;
    if !input.is_empty() {
        return Err(error("command workspace time manifest trailing records"));
    }
    Ok(())
}
pub(super) fn prepare(
    config: &RecipeCommandState,
    output_dir: &Path,
    operands: &CommandOperands<'_>,
) -> Result<(), RunError> {
    let root = rooted_directory(&output_dir.join(&config.workspace_output), "")?;
    if let Some(input) = &config.prior_times_input {
        restore(&root, &operands.bytes(input, RECIPE_BYTES_MAX, "original command workspace times")?)?;
    }
    Ok(())
}
#[cfg(unix)]
fn capture_times(root: &Path, file: &mut File, remaining: &mut u64) -> Result<(), RunError> {
    use std::os::unix::fs::MetadataExt;
    record(file, TIMES_MAGIC, remaining)?;
    walk(root, &mut root.to_path_buf(), &mut 0, &mut |_, member, metadata| {
        record(file, member, remaining)?;
        number(file, metadata.mtime(), remaining)?;
        number(file, metadata.mtime_nsec(), remaining)
    })
}
#[cfg(not(unix))]
fn capture_times(_root: &Path, _file: &mut File, _remaining: &mut u64) -> Result<(), RunError> {
    Err(error("original command workspace times require Unix"))
}
#[cfg(unix)]
fn observed_descriptor(file: Option<&File>) -> Result<(i64, i64), RunError> {
    use std::os::fd::AsRawFd;
    let Some(mut file) = file else {
        return Ok((-1, -1));
    };
    let position = file
        .stream_position()
        .map_err(|failure| error(format!("observing actual command OFD position: {failure}")))?;
    let position =
        i64::try_from(position).map_err(|_| error("actual command descriptor offset exceeds source long"))?;
    let flags = unsafe { libc::fcntl(file.as_raw_fd(), libc::F_GETFL) };
    if flags < 0 {
        return Err(error(format!("observing actual command OFD flags: {}", std::io::Error::last_os_error())));
    }
    #[cfg(target_os = "linux")]
    if flags & (libc::O_ASYNC | libc::O_DIRECT | libc::O_NOATIME) != 0 {
        return Err(error("actual command selected unsupported shared descriptor flags"));
    }
    Ok((position, i64::from(flags & (libc::O_APPEND | libc::O_NONBLOCK))))
}
#[cfg(not(unix))]
fn observed_descriptor(_file: Option<&File>) -> Result<(i64, i64), RunError> {
    Err(error("original descriptor observations require Unix"))
}
#[cfg(target_os = "linux")]
fn capture_descriptor_files(output: &Path, bytes_max: u64, descriptors: &Descriptors) -> Result<(), RunError> {
    use std::io::Read;
    use std::os::fd::AsRawFd;
    let root = rooted_directory(output, "")?;
    let mut remaining = bytes_max;
    for (index, snapshot) in descriptors.0.snapshots.iter().enumerate() {
        if !snapshot {
            continue;
        }
        let retained = descriptors.0.slots[index]
            .as_ref()
            .ok_or_else(|| error("missing retained regular descriptor snapshot"))?;
        // Opening this owned descriptor through proc creates a fresh read OFD;
        // it neither seeks the original nor grants read access to the child.
        let source = File::open(format!("/proc/self/fd/{}", retained.as_raw_fd()))
            .map_err(|failure| error(failure.to_string()))?;
        let metadata = source.metadata().map_err(|failure| error(failure.to_string()))?;
        if !metadata.is_file() || metadata.len() > remaining {
            return Err(error("actual descriptor snapshots exceed their declared tree bound"));
        }
        let temporary = root.join(format!(".{index}.observation"));
        let mut target = create(&temporary)?;
        let copied = std::io::copy(
            &mut source.take(remaining.checked_add(1).ok_or_else(|| error("descriptor snapshot bound overflow"))?),
            &mut target,
        )
        .map_err(|failure| error(failure.to_string()))?;
        remaining = remaining
            .checked_sub(copied)
            .ok_or_else(|| error("actual descriptor snapshot grew beyond its declared tree bound"))?;
        target.sync_all().map_err(|failure| error(failure.to_string()))?;
        // The target may itself back a retained FD; never truncate that inode
        // while it is being read. Publish only the completed actual snapshot.
        fs::rename(temporary, root.join(index.to_string())).map_err(|failure| error(failure.to_string()))?;
    }
    Ok(())
}
#[cfg(not(target_os = "linux"))]
fn capture_descriptor_files(_output: &Path, _bytes_max: u64, _descriptors: &Descriptors) -> Result<(), RunError> {
    Err(error("actual descriptor backing snapshots require Linux"))
}
pub(super) fn capture(
    config: &RecipeCommandState,
    step: &RecipeStep,
    output_dir: &Path,
    descriptors: &Descriptors,
) -> Result<(), RunError> {
    let root = rooted_directory(&output_dir.join(&config.workspace_output), "")?;
    let mut descriptor_file = create(&output_dir.join(&config.descriptors_output))?;
    let mut remaining = step.outputs[&config.descriptors_output].bytes_max.min(RECIPE_BYTES_MAX);
    record(&mut descriptor_file, DESCRIPTORS_MAGIC, &mut remaining)?;
    for (index, file) in descriptors.0.slots.iter().enumerate() {
        let (position, flags) = observed_descriptor(file.as_ref())?;
        number(&mut descriptor_file, index as i64, &mut remaining)?;
        number(&mut descriptor_file, position, &mut remaining)?;
        number(&mut descriptor_file, flags, &mut remaining)?;
    }
    descriptor_file.sync_all().map_err(|failure| error(failure.to_string()))?;
    let mut times = create(&output_dir.join(&config.times_output))?;
    let mut remaining = step.outputs[&config.times_output].bytes_max.min(RECIPE_BYTES_MAX);
    capture_times(&root, &mut times, &mut remaining)?;
    times.sync_all().map_err(|failure| error(failure.to_string()))?;
    if let Some(name) = &config.descriptor_files_output {
        capture_descriptor_files(&output_dir.join(name), step.outputs[name].bytes_max, descriptors)?;
    }
    Ok(())
}

#[cfg(all(test, target_os = "linux"))]
mod tests {
    use std::io::Read;
    use std::io::Write;
    use std::os::fd::AsRawFd;
    use std::os::unix::fs::MetadataExt;
    use std::os::unix::fs::symlink;

    use super::*;

    fn manifest(fields: &[&str]) -> Vec<u8> {
        fields.iter().map(|field| format!("{}:{field},", field.len())).collect::<String>().into_bytes()
    }

    #[test]
    fn workspace_times_restore_exact_postorder_without_following_symlinks() {
        let temporary = tempfile::tempdir().unwrap();
        let workspace = temporary.path().join("workspace");
        fs::create_dir_all(workspace.join("dir")).unwrap();
        fs::write(workspace.join("dir/member"), b"original source\n").unwrap();
        let outside = temporary.path().join("outside");
        fs::write(&outside, b"not workspace authority\n").unwrap();
        symlink("../outside", workspace.join("link")).unwrap();
        set_mtime(&outside, 13, 17).unwrap();
        let expected = [
            ("dir/member", 123, 987_654_321),
            ("dir", 124, 42),
            ("link", 125, 31),
            ("", 126, 29),
        ];
        for (member, seconds, nanoseconds) in expected {
            set_mtime(&workspace.join(member), seconds, nanoseconds).unwrap();
        }
        let path = temporary.path().join("times");
        capture_times(&workspace, &mut File::create(&path).unwrap(), &mut 4096).unwrap();
        let bytes = fs::read(path).unwrap();
        let mut records = bytes.as_slice();
        assert_eq!(take(&mut records).unwrap(), TIMES_MAGIC);
        for (member, seconds, nanoseconds) in expected {
            assert_eq!(take(&mut records).unwrap(), member);
            assert_eq!(integer(&mut records).unwrap(), seconds);
            assert_eq!(integer(&mut records).unwrap(), nanoseconds);
            set_mtime(&workspace.join(member), 200, 0).unwrap();
        }
        assert!(records.is_empty());
        restore(&workspace, &bytes).unwrap();
        for (member, seconds, nanoseconds) in expected {
            let actual = fs::symlink_metadata(workspace.join(member)).unwrap();
            assert_eq!((actual.mtime(), actual.mtime_nsec()), (seconds, nanoseconds));
        }
        let outside = fs::metadata(outside).unwrap();
        assert_eq!((outside.mtime(), outside.mtime_nsec()), (13, 17));
    }

    #[test]
    fn workspace_times_reject_changed_inventory_noncanonical_and_out_of_range_records() {
        let temporary = tempfile::tempdir().unwrap();
        let workspace = temporary.path().join("workspace");
        fs::create_dir(&workspace).unwrap();
        let outside = temporary.path().join("outside");
        fs::write(&outside, b"not workspace authority\n").unwrap();
        set_mtime(&outside, 13, 17).unwrap();
        let valid = manifest(&[TIMES_MAGIC, "", "-1", "999999999"]);
        restore(&workspace, &valid).unwrap();
        let actual = fs::metadata(&workspace).unwrap();
        assert_eq!((actual.mtime(), actual.mtime_nsec()), (-1, 999_999_999));
        for fields in [
            ["wrong-version", "", "123", "0"],
            [TIMES_MAGIC, "../outside", "123", "0"],
            [TIMES_MAGIC, "", "00", "0"],
            [TIMES_MAGIC, "", "-0", "0"],
            [TIMES_MAGIC, "", "123", "1000000000"],
            [TIMES_MAGIC, "", "123", "-1"],
        ] {
            assert!(restore(&workspace, &manifest(&fields)).is_err(), "{fields:?}");
        }
        let mut trailing = valid.clone();
        trailing.extend_from_slice(b"0:,");
        assert!(restore(&workspace, &trailing).is_err());
        assert!(restore(&workspace, &valid[..valid.len() - 1]).is_err());
        let outside = fs::metadata(outside).unwrap();
        assert_eq!((outside.mtime(), outside.mtime_nsec()), (13, 17));
    }

    fn descriptor(target: RecipeDescriptorTarget) -> RecipeDescriptor {
        RecipeDescriptor {
            target,
            access: RecipeDescriptorAccess::ReadWrite,
            flags: 0,
            offset: 0,
            alias: 0,
        }
    }

    #[test]
    fn descriptor_observation_uses_actual_shared_offsets_and_append_flags() {
        let temporary = tempfile::tempdir().unwrap();
        let path = temporary.path().join("stream");
        fs::write(&path, b"abcdef").unwrap();
        let target = RecipeDescriptorTarget::File {
            path: RecipeWord {
                parts: vec![WordPart::Output {
                    name: "streams".into(),
                    path: "0".into(),
                }],
            },
        };
        let mut entry = descriptor(target);
        entry.offset = 1;
        let mut original = open_descriptor(&path, &entry, false).unwrap();
        let mut alias = original.try_clone().unwrap();
        let flags = unsafe { libc::fcntl(alias.as_raw_fd(), libc::F_GETFL) };
        assert!(flags >= 0);
        assert_eq!(flags & (libc::O_NOFOLLOW | libc::O_PATH), 0);
        assert_eq!(flags & libc::O_ACCMODE, libc::O_RDWR);
        alias.write_all(b"X").unwrap();
        assert_eq!(observed_descriptor(Some(&original)).unwrap(), (2, 0));
        original.seek(SeekFrom::Start(1)).unwrap();
        assert_eq!(unsafe { libc::fcntl(alias.as_raw_fd(), libc::F_SETFL, libc::O_APPEND | libc::O_NONBLOCK) }, 0);
        alias.write_all(b"!").unwrap();
        let expected = (7, i64::from(libc::O_APPEND | libc::O_NONBLOCK));
        assert_eq!(observed_descriptor(Some(&original)).unwrap(), expected);
        assert_eq!(observed_descriptor(Some(&alias)).unwrap(), expected);
        assert_eq!(fs::read(&path).unwrap(), b"aXcdef!");
        assert_eq!(observed_descriptor(None).unwrap(), (-1, -1));
        entry.flags = libc::O_TRUNC as u32;
        assert!(open_descriptor(&path, &entry, false).is_err());
        assert_eq!(fs::read(&path).unwrap(), b"aXcdef!");
    }

    #[test]
    fn null_descriptor_preserves_device_semantics_and_rejects_regular_substitution() {
        let entry = descriptor(RecipeDescriptorTarget::NullDevice);
        let mut null = open_descriptor(Path::new("/dev/null"), &entry, true).unwrap();
        null.write_all(b"discarded, never a regular backing snapshot").unwrap();
        assert_eq!(null.read(&mut [0; 1]).unwrap(), 0);
        assert_eq!(observed_descriptor(Some(&null)).unwrap(), (0, 0));
        assert!(is_null_device(&null.metadata().unwrap()));
        let temporary = tempfile::tempdir().unwrap();
        let regular = temporary.path().join("not-null");
        fs::write(&regular, b"must remain untouched").unwrap();
        assert!(open_descriptor(&regular, &entry, true).is_err());
        let mut bad_offset = entry;
        bad_offset.offset = 1;
        assert!(open_descriptor(Path::new("/dev/null"), &bad_offset, true).is_err());
        assert_eq!(fs::read(regular).unwrap(), b"must remain untouched");
    }

    #[test]
    fn descriptor_snapshot_keeps_workspace_inode_and_actual_original_offset() {
        let temporary = tempfile::tempdir().unwrap();
        let workspace = temporary.path().join("workspace");
        let snapshots = temporary.path().join("descriptor-files");
        fs::create_dir(&workspace).unwrap();
        fs::create_dir(&snapshots).unwrap();
        let path = workspace.join("config.log");
        fs::write(&path, b"prior").unwrap();
        let mut entry = descriptor(RecipeDescriptorTarget::File {
            path: RecipeWord {
                parts: vec![WordPart::Output {
                    name: "workspace".into(),
                    path: "config.log".into(),
                }],
            },
        });
        entry.offset = 1;
        let mut file = open_descriptor(&path, &entry, false).unwrap();
        file.write_all(b"X").unwrap();
        let original = file.metadata().unwrap();
        let mut slots = std::array::from_fn(|_| None);
        slots[63] = Some(file.try_clone().unwrap());
        slots[5] = Some(file);
        let mut primary = [false; DESCRIPTOR_COUNT];
        primary[5] = true;
        let retained = Descriptors::from_slots(slots, primary).unwrap();
        capture_descriptor_files(&snapshots, 5, &retained).unwrap();
        assert_eq!(fs::read(snapshots.join("5")).unwrap(), b"pXior");
        assert!(!snapshots.join("63").exists());
        assert_eq!(observed_descriptor(retained.0.slots[63].as_ref()).unwrap(), (2, 0));
        let actual = fs::metadata(&path).unwrap();
        assert_eq!((actual.dev(), actual.ino()), (original.dev(), original.ino()));
        assert_eq!(fs::read(&path).unwrap(), b"pXior");
        assert!(capture_descriptor_files(&snapshots, 4, &retained).is_err());
        assert_eq!(fs::read(snapshots.join("5")).unwrap(), b"pXior");
        assert_eq!(observed_descriptor(retained.0.slots[5].as_ref()).unwrap(), (2, 0));
    }
}
