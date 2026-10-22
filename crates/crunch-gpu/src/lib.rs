//! HIP compilation is an ordinary admitted derivation, never a scheduler backend.
use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;
use std::process::Stdio;

use crunch_gpu_core::AdmittedPlan;
use crunch_gpu_core::DEVICE_LIBRARIES;
use crunch_gpu_core::Paths;
use crunch_gpu_core::Plan;
use crunch_gpu_core::validate;
use serde::Serialize;

pub type Result<T, E = Box<dyn std::error::Error>> = std::result::Result<T, E>;
const WORK: &str = "/build/mantle-gpu";
/// 64 MiB: the admitted ceiling for one compiled kernel object.
const MAX_ARTIFACT_BYTES: u64 = 67_108_864;
/// 256 MiB: the admitted ceiling for one runtime library input.
const MAX_RUNTIME_LIBRARY_BYTES: u64 = 268_435_456;
/// 4 MiB: the admitted ceiling for one AMDHSA metadata note.
const MAX_METADATA_BYTES: u64 = 4_194_304;
/// The relocation type occupies the low 32 bits of an ELF `r_info` word.
const RELOCATION_TYPE_MASK: u64 = 0xffff_ffff;
/// The source languages an admitted kernel may declare.
const LANGUAGES: &[&str] = &["OpenCL C", "OpenCL C++", "HCC", "HIP", "OpenMP", "Assembler"];
/// The kernel kinds the ABI defines.
const KINDS: &[&str] = &["normal", "init", "fini"];
/// The address spaces an argument may declare.
const ADDRESS_SPACES: &[&str] = &["private", "global", "constant", "local", "generic", "region"];
/// The access modes an argument may declare.
const ACCESS_MODES: &[&str] = &["read_only", "write_only", "read_write"];
/// The value kinds an argument may declare.
const VALUE_KINDS: &[&str] = &[
    "by_value",
    "global_buffer",
    "dynamic_shared_pointer",
    "sampler",
    "image",
    "pipe",
    "queue",
    "hidden_block_count_x",
    "hidden_block_count_y",
    "hidden_block_count_z",
    "hidden_group_size_x",
    "hidden_group_size_y",
    "hidden_group_size_z",
    "hidden_remainder_x",
    "hidden_remainder_y",
    "hidden_remainder_z",
    "hidden_global_offset_x",
    "hidden_global_offset_y",
    "hidden_global_offset_z",
    "hidden_grid_dims",
    "hidden_none",
    "hidden_printf_buffer",
    "hidden_hostcall_buffer",
    "hidden_heap_v1",
    "hidden_default_queue",
    "hidden_completion_action",
    "hidden_multigrid_sync_arg",
    "hidden_dynamic_lds_size",
    "hidden_private_base",
    "hidden_shared_base",
    "hidden_queue_ptr",
];

fn invalid(message: impl Into<String>) -> Box<dyn std::error::Error> {
    std::io::Error::other(message.into()).into()
}

/// The ordinary pipeline authenticates these roots. This check only prevents a
/// declared member's symlink from escaping that already-admitted closure.
fn bound(plan: &Plan, path: &Path) -> Result<()> {
    let actual =
        fs::canonicalize(path).map_err(|error| invalid(format!("gpu-input-missing: {}: {error}", path.display())))?;
    if !plan.roles.values().any(|root| actual.starts_with(root)) {
        return Err(invalid(format!("gpu-input-escape: {}", path.display())));
    }
    debug_assert!(path.is_absolute());
    debug_assert!(actual.is_absolute());
    Ok(())
}
fn member(plan: &Plan, value: &str, directory: bool) -> Result<PathBuf> {
    let path = PathBuf::from(value);
    bound(plan, &path)?;
    let meta = fs::metadata(&path)?;
    if (directory && !meta.is_dir()) || (!directory && !meta.is_file()) {
        return Err(invalid(format!("gpu-input-kind: {}", path.display())));
    }
    Ok(path)
}
fn tool(plan: &Plan, value: &str) -> Result<PathBuf> {
    let path = member(plan, value, false)?;
    if fs::metadata(&path)?.permissions().mode() & 0o111 == 0 {
        return Err(invalid("gpu-tool-not-executable"));
    }
    Ok(path)
}
fn command(program: &Path, plan: &Plan) -> Command {
    let mut command = Command::new(program);
    command
        .env_clear()
        .envs([
            ("PATH", "/no-ambient-tools".to_string()),
            ("HOME", format!("{WORK}/home")),
            ("TMPDIR", format!("{WORK}/tmp")),
            ("LC_ALL", "C".into()),
            ("TZ", "UTC".into()),
            ("SOURCE_DATE_EPOCH", plan.source_date_epoch.to_string()),
        ])
        .current_dir(WORK)
        .stdin(Stdio::null());
    command
}
fn run(command: &mut Command, phase: &str) -> Result<()> {
    let status = command.status()?;
    if !status.success() {
        return Err(invalid(format!("gpu-{phase}-failed: {status}")));
    }
    Ok(())
}
/// The admitted toolchain, and the exact version each tool reports.
fn versions(plan: &Plan, paths: &Paths) -> Result<BTreeMap<String, String>> {
    let mut recorded = Vec::with_capacity(3);
    for (spec, path) in [
        (&plan.clang, &paths.clang),
        (&plan.host_compiler, &paths.host_compiler),
        (&plan.inspector, &paths.inspector),
    ] {
        let path = tool(plan, path)?;
        let output = command(&path, plan).arg("--version").output()?;
        let text = String::from_utf8(output.stdout)?;
        let is_expected = output.status.success()
            && text.split_whitespace().any(|part| {
                part.strip_prefix(spec.version.as_str())
                    .is_some_and(|suffix| suffix.is_empty() || suffix.starts_with("git"))
            });
        if !is_expected {
            return Err(invalid(format!("gpu-incompatible-tool: {} expected {}", path.display(), spec.version)));
        }
        debug_assert!(!text.trim().is_empty());
        recorded.push((path.display().to_string(), text.trim().to_string()));
    }
    debug_assert_eq!(recorded.len(), 3);
    let output = command(&tool(plan, &paths.host_compiler)?, plan)
        .arg(format!("--target={}", plan.host_target))
        .arg("-dumpmachine")
        .output()?;
    if !output.status.success() || String::from_utf8(output.stdout)?.trim() != plan.host_target {
        return Err(invalid("gpu-host-compiler-target-mismatch"));
    }
    Ok(recorded.into_iter().collect())
}
/// No network or unsigned-input escape hatch exists here. These inputs are
/// consumed through the same generic signed PathInfo admission as native CMake.
pub fn lower(plan: Plan) -> Result<crunch_cmake::Derivation> {
    let AdmittedPlan { plan, paths } = validate(plan).map_err(|e| invalid(e.to_string()))?;
    let json = serde_json::to_string(&plan)?;
    Ok(crunch_cmake::Derivation {
        name: plan.name,
        builder: paths.runner,
        args: vec!["run".into()],
        system: plan.build_platform,
        outputs: vec!["out".into()],
        addressing_mode: "input-addressed".into(),
        inputs: plan.roles.into_values().collect(),
        env: BTreeMap::from([("MANTLE_GPU_PLAN".into(), json)]),
    })
}
/// The digest of every declared input, keyed by its absolute path.
fn inputs(plan: &Plan, paths: &Paths) -> Result<BTreeMap<String, String>> {
    let mut files = vec![member(plan, &paths.kernel, false)?];
    files.extend(paths.generated.iter().map(|m| member(plan, m, false)).collect::<Result<Vec<_>>>()?);
    let libraries = member(plan, &paths.device_libraries, true)?;
    for name in DEVICE_LIBRARIES {
        let path = libraries.join(name);
        bound(plan, &path).map_err(|e| invalid(format!("gpu-device-library-missing-or-escaping: {name}: {e}")))?;
        if !fs::metadata(&path)?.is_file() {
            return Err(invalid(format!("gpu-device-library-missing: {name}")));
        }
        files.push(path);
    }
    debug_assert!(!files.is_empty());
    let file_count = files.len();
    let library_count = paths.runtime_libraries.len();
    let mut digests = Vec::with_capacity(file_count.saturating_add(library_count));
    for path in files {
        digests.push((path.display().to_string(), hash_file(&path, MAX_ARTIFACT_BYTES)?));
    }
    for value in &paths.runtime_libraries {
        let path = member(plan, value, false)?;
        digests.push((path.display().to_string(), hash_file(&path, MAX_RUNTIME_LIBRARY_BYTES)?));
    }
    debug_assert_eq!(digests.len(), file_count.saturating_add(library_count));
    for header in &paths.headers {
        member(plan, header, true)?;
    }
    member(plan, &paths.resource_dir, true)?;
    for role in [&paths.sysroot, &paths.toolkit] {
        let path = Path::new(role);
        bound(plan, path)?;
        if !path.is_dir() {
            return Err(invalid("gpu-root-not-directory"));
        }
    }
    Ok(digests.into_iter().collect())
}
/// The blake3 digest of one file, bounded by the admitted byte ceiling.
fn hash_file(path: &Path, max_bytes: u64) -> Result<String> {
    use std::io::Read;
    let mut file = fs::File::open(path)?;
    let length_bytes = file.metadata()?.len();
    if length_bytes > max_bytes {
        return Err(invalid("gpu-file-size-limit"));
    }
    debug_assert!(length_bytes <= max_bytes);
    let mut hash = blake3::Hasher::new();
    let mut buffer = [0_u8; 65536];
    let mut remaining = length_bytes;
    while remaining > 0 {
        let count = file.read(&mut buffer)?;
        if count == 0 {
            break;
        }
        debug_assert!(count <= buffer.len());
        hash.update(&buffer[..count]);
        remaining = remaining.saturating_sub(u64::try_from(count)?);
    }
    Ok(hash.finalize().to_hex().to_string())
}
/// The exact HIP compilation invocation, and the arguments it ran with.
fn compile(plan: &Plan, paths: &Paths, output: &Path) -> Result<Vec<String>> {
    let mut args = vec![
        "-x".into(),
        "hip".into(),
        "--offload-device-only".into(),
        "-c".into(),
        "-nogpuinc".into(),
        "-nostdinc".into(),
        "-nostdinc++".into(),
        "-mcode-object-version=4".into(),
        "-fno-gpu-flush-denormals-to-zero".into(),
        "-fno-unsafe-math-optimizations".into(),
        "-fno-finite-math-only".into(),
        "-fhip-fp32-correctly-rounded-divide-sqrt".into(),
        format!("--target={}", plan.host_target),
        format!("--offload-arch={}", plan.architecture),
        format!("--sysroot={}", paths.sysroot),
        format!("--rocm-path={}", paths.toolkit),
        format!("--rocm-device-lib-path={}", member(plan, &paths.device_libraries, true)?.display()),
        format!("-resource-dir={}", member(plan, &paths.resource_dir, true)?.display()),
        format!("-O{}", plan.optimization),
        "-Werror=date-time".into(),
        "-fno-ident".into(),
    ];
    for (role, root) in &plan.roles {
        args.push(format!("-ffile-prefix-map={root}=/inputs/{role}"));
    }
    for header in &paths.headers {
        args.extend(["-I".into(), member(plan, header, true)?.display().to_string()]);
    }
    for generated in &paths.generated {
        args.extend(["-include".into(), member(plan, generated, false)?.display().to_string()]);
    }
    for (name, value) in &plan.defines {
        args.push(format!("-D{name}={value}"));
    }
    let kernel = member(plan, &paths.kernel, false)?.display().to_string();
    let output_path = output.display().to_string();
    args.extend([kernel, "-o".into(), output_path.clone()]);
    debug_assert!(args.iter().any(|arg| arg == "-o"));
    debug_assert_eq!(args.last(), Some(&output_path));
    run(command(&tool(plan, &paths.clang)?, plan).args(&args), "compile")?;
    Ok(args)
}

#[derive(Debug, PartialEq, Eq, Serialize)]
pub struct Inspection {
    pub elf_type: u16,
    pub machine: u16,
    pub flags: u32,
    pub architecture: String,
    pub entry_points: Vec<String>,
}
fn u16_at(data: &[u8], at: usize) -> Result<u16> {
    Ok(u16::from_le_bytes(region(data, u64::try_from(at)?, 2)?.try_into()?))
}
fn u32_at(data: &[u8], at: usize) -> Result<u32> {
    Ok(u32::from_le_bytes(region(data, u64::try_from(at)?, 4)?.try_into()?))
}
fn u64_at(data: &[u8], at: usize) -> Result<u64> {
    Ok(u64::from_le_bytes(region(data, u64::try_from(at)?, 8)?.try_into()?))
}
/// The `length` bytes at `offset`, without a narrowing or wrapping access.
fn region(data: &[u8], offset: u64, length: usize) -> Result<&[u8]> {
    let end = offset.checked_add(u64::try_from(length)?).ok_or_else(|| invalid("gpu-elf-overflow"))?;
    data.get(usize::try_from(offset)?..usize::try_from(end)?)
        .ok_or_else(|| invalid("gpu-elf-truncated"))
}
// ABI authority: https://github.com/ROCm/llvm-project/tree/f58b06dce1f9c15707c5f808fd002e18c2accf7e
// (rocm-7.2.3), llvm/docs/AMDGPUUsage.rst
// (ELF Header, Symbols, Relocation Records, Code Object V4 Metadata, Kernel
// Descriptor), llvm/include/llvm/Support/AMDHSAKernelDescriptor.h, and
// llvm/lib/Target/AMDGPU/MCTargetDesc/AMDGPUTargetStreamer.cpp:
// EmitAmdhsaKernelDescriptor. In particular ET_REL descriptors contain REL64
// relocations, not already-linked code addresses. llvm-readobj is only a report.
struct Elf<'a> {
    data: &'a [u8],
    sections: &'a [u8],
}
impl<'a> Elf<'a> {
    fn section(&self, index: usize) -> Result<&'a [u8]> {
        let start = (index as u64).checked_mul(64).ok_or_else(|| invalid("gpu-elf-overflow"))?;
        region(self.sections, start, 64)
    }
    fn contents(&self, section: &[u8]) -> Result<&'a [u8]> {
        region(self.data, u64_at(section, 24)?, usize::try_from(u64_at(section, 32)?)?)
    }
}
#[derive(Clone, Copy)]
struct Symbol<'a> {
    raw: &'a [u8],
    index: usize,
}
impl Symbol<'_> {
    fn section(self) -> Result<usize> {
        Ok(usize::from(u16_at(self.raw, 6)?))
    }
    fn value(self) -> Result<u64> {
        u64_at(self.raw, 8)
    }
    fn size(self) -> Result<u64> {
        u64_at(self.raw, 16)
    }
}
fn elf_string(strings: &[u8], offset: u32) -> Result<&str> {
    let name = strings.get(usize::try_from(offset)?..).ok_or_else(|| invalid("gpu-elf-symbol-name"))?;
    let end = name.iter().position(|b| *b == 0).ok_or_else(|| invalid("gpu-elf-symbol-name"))?;
    Ok(std::str::from_utf8(&name[..end])?)
}

// Borrow strings and opaque payloads; bound both recursion and total nodes before
// allocating containers. All map keys are strings in AMDHSA metadata. Rejecting
// duplicates before insertion avoids different consumers seeing different kernels.
enum Metadata<'a> {
    Uint(u64),
    String(&'a str),
    Array(Vec<Metadata<'a>>),
    Map(BTreeMap<&'a str, Metadata<'a>>),
    Boolean,
    Other,
}
struct MetadataReader<'a> {
    remaining: &'a [u8],
    nodes: usize,
}
impl<'a> MetadataReader<'a> {
    fn take(&mut self, count: usize) -> Result<&'a [u8]> {
        let result = self.remaining.get(..count).ok_or_else(|| invalid("gpu-metadata-truncated"))?;
        self.remaining = &self.remaining[count..];
        Ok(result)
    }
    fn number(&mut self, count: usize) -> Result<u64> {
        Ok(self.take(count)?.iter().fold(0, |value, byte| (value << 8) | u64::from(*byte)))
    }
    fn value(&mut self, depth: usize) -> Result<Metadata<'a>> {
        if depth > 32 || self.nodes == 65536 {
            return Err(invalid("gpu-metadata-limit"));
        }
        self.nodes += 1;
        let tag = self.take(1)?[0];
        let (kind, count) = match tag {
            0x00..=0x7f => return Ok(Metadata::Uint(u64::from(tag))),
            0xcc..=0xcf => {
                let width = match tag {
                    0xcc => 1,
                    0xcd => 2,
                    0xce => 4,
                    _ => 8,
                };
                return Ok(Metadata::Uint(self.number(width)?));
            }
            0xd0..=0xd3 => {
                let width = match tag {
                    0xd0 => 1,
                    0xd1 => 2,
                    0xd2 => 4,
                    _ => 8,
                };
                let bytes = self.take(width)?;
                return Ok(if bytes[0] & 0x80 == 0 {
                    Metadata::Uint(bytes.iter().fold(0, |value, byte| (value << 8) | u64::from(*byte)))
                } else {
                    Metadata::Other
                });
            }
            0xa0..=0xbf => (0, u64::from(tag & 31)),
            0xd9..=0xdb => {
                let width = match tag {
                    0xd9 => 1,
                    0xda => 2,
                    _ => 4,
                };
                (0, self.number(width)?)
            }
            0x90..=0x9f => (1, u64::from(tag & 15)),
            0xdc..=0xdd => {
                let width = match tag {
                    0xdc => 2,
                    _ => 4,
                };
                (1, self.number(width)?)
            }
            0x80..=0x8f => (2, u64::from(tag & 15)),
            0xde..=0xdf => {
                let width = match tag {
                    0xde => 2,
                    _ => 4,
                };
                (2, self.number(width)?)
            }
            0xc4..=0xc6 => {
                let width = match tag {
                    0xc4 => 1,
                    0xc5 => 2,
                    _ => 4,
                };
                (3, self.number(width)?)
            }
            0xc7..=0xc9 => {
                let width = match tag {
                    0xc7 => 1,
                    0xc8 => 2,
                    _ => 4,
                };
                let count = self.number(width)?.checked_add(1).ok_or_else(|| invalid("gpu-metadata-limit"))?;
                (3, count)
            }
            0xd4..=0xd8 => {
                let count = match tag {
                    0xd4 => 2,
                    0xd5 => 4,
                    0xd6 => 8,
                    0xd7 => 16,
                    _ => 32,
                };
                (3, count)
            }
            0xca => (3, 4),
            0xcb => (3, 8),
            0xc2 | 0xc3 => return Ok(Metadata::Boolean),
            0xc0 | 0xe0..=0xff => return Ok(Metadata::Other),
            _ => return Err(invalid("gpu-metadata-tag")),
        };
        let count = usize::try_from(count)?;
        if count > self.remaining.len() {
            return Err(invalid("gpu-metadata-truncated"));
        }
        match kind {
            0 => Ok(Metadata::String(std::str::from_utf8(self.take(count)?)?)),
            1 => {
                if count > 65536_usize.saturating_sub(self.nodes) {
                    return Err(invalid("gpu-metadata-limit"));
                }
                let mut values = Vec::with_capacity(count);
                for _ in 0..count {
                    values.push(self.value(depth.saturating_add(1))?);
                }
                Ok(Metadata::Array(values))
            }
            2 => {
                if count > 65536_usize.saturating_sub(self.nodes) / 2 {
                    return Err(invalid("gpu-metadata-limit"));
                }
                let mut values = BTreeMap::new();
                for _ in 0..count {
                    let Metadata::String(key) = self.value(depth.saturating_add(1))? else {
                        return Err(invalid("gpu-metadata-map-key"));
                    };
                    if values.insert(key, self.value(depth.saturating_add(1))?).is_some() {
                        return Err(invalid("gpu-metadata-duplicate-key"));
                    }
                }
                Ok(Metadata::Map(values))
            }
            _ => {
                self.take(count)?;
                Ok(Metadata::Other)
            }
        }
    }
}
impl<'a> Metadata<'a> {
    fn map(&self) -> Result<&BTreeMap<&'a str, Metadata<'a>>> {
        match self {
            Self::Map(value) => Ok(value),
            _ => Err(invalid("gpu-metadata-map")),
        }
    }
    fn array(&self) -> Result<&[Metadata<'a>]> {
        match self {
            Self::Array(value) => Ok(value),
            _ => Err(invalid("gpu-metadata-array")),
        }
    }
    fn string(&self) -> Result<&'a str> {
        match self {
            Self::String(value) if !value.is_empty() && !value.contains('\0') => Ok(value),
            _ => Err(invalid("gpu-metadata-string")),
        }
    }
    fn uint(&self) -> Result<u64> {
        match self {
            Self::Uint(value) => Ok(*value),
            _ => Err(invalid("gpu-metadata-integer")),
        }
    }
    fn field(&self, key: &str) -> Result<&Metadata<'a>> {
        self.map()?.get(key).ok_or_else(|| invalid(format!("gpu-metadata-missing: {key}")))
    }
}
fn metadata_enum(value: &Metadata<'_>, allowed: &[&str]) -> Result<()> {
    if !allowed.contains(&value.string()?) {
        return Err(invalid("gpu-metadata-enum"));
    }
    Ok(())
}
/// Validate known optional fields too. Vendor extensions remain permitted, but
/// cannot obscure ill-typed standard fields or duplicate argument-map keys.
fn kernel_metadata_schema(kernel: &Metadata<'_>) -> Result<()> {
    for (key, value) in kernel.map()? {
        kernel_field(kernel, key, value)?;
    }
    Ok(())
}
/// One known kernel metadata field, validated against the ABI.
fn kernel_field(kernel: &Metadata<'_>, key: &str, value: &Metadata<'_>) -> Result<()> {
    match key {
        ".language" => metadata_enum(value, LANGUAGES)?,
        ".kind" => metadata_enum(value, KINDS)?,
        ".vec_type_hint" | ".device_enqueue_symbol" => {
            value.string()?;
        }
        ".sgpr_spill_count"
        | ".vgpr_spill_count"
        | ".agpr_count"
        | ".uniform_work_group_size"
        | ".workgroup_processor_mode"
        | ".max_num_work_groups_x"
        | ".max_num_work_groups_y"
        | ".max_num_work_groups_z" => {
            value.uint()?;
            debug_assert!(matches!(value, Metadata::Uint(_)));
        }
        ".uses_dynamic_stack" => {
            if !matches!(value, Metadata::Boolean) {
                return Err(invalid("gpu-metadata-boolean"));
            }
            debug_assert!(matches!(value, Metadata::Boolean));
        }
        ".language_version" | ".reqd_workgroup_size" | ".workgroup_size_hint" | ".cluster_dims" => {
            dimension_array(key, value)?;
        }
        ".args" => {
            kernel_arguments(kernel, value)?;
        }
        _ => {}
    }
    Ok(())
}
/// A fixed-length dimension array of unsigned integers.
fn dimension_array(key: &str, value: &Metadata<'_>) -> Result<()> {
    let array = value.array()?;
    let expected = if key == ".language_version" { 2 } else { 3 };
    if array.len() != expected {
        return Err(invalid("gpu-metadata-array-size"));
    }
    for item in array {
        item.uint()?;
    }
    debug_assert!(array.iter().all(|item| matches!(item, Metadata::Uint(_))));
    Ok(())
}
/// Every kernel argument is typed and lies inside the kernarg segment.
fn kernel_arguments(kernel: &Metadata<'_>, value: &Metadata<'_>) -> Result<()> {
    let kernarg = kernel.field(".kernarg_segment_size")?.uint()?;
    for argument in value.array()? {
        let size_bytes = argument.field(".size")?.uint()?;
        let offset_bytes = argument.field(".offset")?.uint()?;
        let kind = argument.field(".value_kind")?;
        metadata_enum(kind, VALUE_KINDS)?;
        debug_assert!(matches!(kind, Metadata::String(_)));
        let extent = offset_bytes.checked_add(size_bytes);
        if extent.is_none_or(|end| end > kernarg) {
            return Err(invalid("gpu-metadata-argument-extent"));
        }
        debug_assert!(extent.is_some_and(|end| end <= kernarg));
        argument_fields(argument)?;
    }
    Ok(())
}
/// The optional per-argument metadata fields.
fn argument_fields(argument: &Metadata<'_>) -> Result<()> {
    for (key, value) in argument.map()? {
        match *key {
            ".address_space" => metadata_enum(value, ADDRESS_SPACES)?,
            ".access" | ".actual_access" => metadata_enum(value, ACCESS_MODES)?,
            ".name" | ".type_name" | ".value_kind" => match value {
                // Empty argument names are permitted by the ABI.
                Metadata::String(text) if !text.contains('\0') => {
                    debug_assert!(matches!(value, Metadata::String(_)));
                }
                _ => return Err(invalid("gpu-metadata-argument-string")),
            },
            ".pointee_align" => {
                value.uint()?;
            }
            ".is_const" | ".is_restrict" | ".is_volatile" | ".is_pipe" => match value {
                Metadata::Boolean => {
                    debug_assert!(matches!(value, Metadata::Boolean));
                }
                _ => return Err(invalid("gpu-metadata-boolean")),
            },
            _ => {}
        }
    }
    Ok(())
}
/// The single AMDHSA metadata note of an admitted object.
fn metadata<'a>(elf: &Elf<'a>) -> Result<Metadata<'a>> {
    let mut found = None;
    for section in elf.sections.chunks_exact(64) {
        if u32_at(section, 4)? != 7 {
            continue;
        }
        let notes = elf.contents(section)?;
        let mut offset_bytes = 0_usize;
        while offset_bytes < notes.len() {
            let header = region(notes, u64::try_from(offset_bytes)?, 12)?;
            let names_bytes = usize::try_from(u32_at(header, 0)?)?;
            let size_bytes = usize::try_from(u32_at(header, 4)?)?;
            let name_start = offset_bytes.checked_add(12).ok_or_else(|| invalid("gpu-elf-overflow"))?;
            let name = region(notes, u64::try_from(name_start)?, names_bytes)?;
            let value_start = name_start
                .checked_add(names_bytes.checked_add(3).ok_or_else(|| invalid("gpu-elf-overflow"))? & !3)
                .ok_or_else(|| invalid("gpu-elf-overflow"))?;
            let value = region(notes, u64::try_from(value_start)?, size_bytes)?;
            offset_bytes = value_start
                .checked_add(size_bytes.checked_add(3).ok_or_else(|| invalid("gpu-elf-overflow"))? & !3)
                .ok_or_else(|| invalid("gpu-elf-overflow"))?;
            if offset_bytes > notes.len() {
                return Err(invalid("gpu-elf-note-padding"));
            }
            debug_assert!(offset_bytes <= notes.len());
            if name == b"AMDGPU\0" && u32_at(header, 8)? == 32 {
                if found.is_some() || u64::try_from(size_bytes)? > MAX_METADATA_BYTES {
                    return Err(invalid("gpu-metadata-ambiguous-or-limit"));
                }
                let mut reader = MetadataReader {
                    remaining: value,
                    nodes: 0,
                };
                found = Some(reader.value(0)?);
                if !reader.remaining.is_empty() {
                    return Err(invalid("gpu-metadata-trailing-data"));
                }
                debug_assert!(reader.nodes <= 65536);
            }
        }
    }
    found.ok_or_else(|| invalid("gpu-metadata-missing"))
}
/// The kernel descriptor, its section and the one entry relocation it needs.
fn kernel_descriptor(
    elf: &Elf<'_>,
    code: Symbol<'_>,
    descriptor: Symbol<'_>,
    symtab: usize,
    symbols: &[u8],
) -> Result<()> {
    let text = elf.section(code.section()?)?;
    let rodata = elf.section(descriptor.section()?)?;
    if !kernel_symbols_admit(code, descriptor, text, rodata)? {
        return Err(invalid("gpu-kernel-symbol-or-section"));
    }
    let code_value = code.value()?;
    let code_size_bytes = code.size()?;
    region(elf.contents(text)?, code_value, usize::try_from(code_size_bytes)?)?;
    let bytes = region(elf.contents(rodata)?, descriptor.value()?, 64)?;
    debug_assert_eq!(bytes.len(), 64);
    if !descriptor_bytes_admit(bytes)? {
        return Err(invalid("gpu-kernel-descriptor"));
    }
    let properties = u16_at(bytes, 56)?;
    let sgprs = [4, 2, 2, 2, 2, 2, 1]
        .iter()
        .enumerate()
        .filter(|(bit, _)| properties & (1 << bit) != 0)
        .map(|(_, count)| count)
        .sum::<u32>();
    let is_user_sgprs_bounded = sgprs <= 16;
    let is_user_sgprs_declared = (u32_at(bytes, 52)? >> 1) & 31 == sgprs;
    if [is_user_sgprs_bounded, is_user_sgprs_declared].iter().any(|fact| !*fact) {
        return Err(invalid("gpu-kernel-user-sgprs"));
    }
    debug_assert!(sgprs <= 16);
    entry_relocation(elf, code, descriptor, symtab, symbols)
}
/// Every symbol and section fact the AMDGPU ABI requires before the kernel
/// descriptor bytes are trusted.
fn kernel_symbols_admit(code: Symbol<'_>, descriptor: Symbol<'_>, text: &[u8], rodata: &[u8]) -> Result<bool> {
    debug_assert!(text.len() >= 64);
    debug_assert!(rodata.len() >= 64);
    let code_value = code.value()?;
    let code_size_bytes = code.size()?;
    let descriptor_value = descriptor.value()?;
    let descriptor_size_bytes = descriptor.size()?;
    // STV_PROTECTED is emitted for an otherwise default-visible entry, allowing
    // the static relocation; hidden/internal kernels retain their visibility.
    let is_code_protected = code.raw[4] == 0x12;
    let is_code_visibility = (1..=3).contains(&code.raw[5]);
    let is_descriptor_binding = descriptor.raw[4] == 0x11;
    let is_descriptor_visibility = descriptor.raw[5] <= 3;
    let is_visibility_agrees = descriptor.raw[5] == code.raw[5] || (descriptor.raw[5] == 0 && code.raw[5] == 3);
    let is_text_type = u32_at(text, 4)? == 1;
    let is_text_flags = u64_at(text, 8)? == 6;
    let is_rodata_type = u32_at(rodata, 4)? == 1;
    let is_rodata_flags = u64_at(rodata, 8)? == 2;
    let is_text_extent = u64_at(text, 48)? >= 256;
    let is_code_aligned = code_value % 256 == 0;
    let is_code_extent = code_size_bytes != 0;
    let is_code_word_aligned = code_size_bytes % 4 == 0;
    let is_rodata_extent = u64_at(rodata, 48)? >= 64;
    let is_descriptor_aligned = descriptor_value % 64 == 0;
    let is_descriptor_extent = descriptor_size_bytes == 64;
    Ok([
        is_code_protected,
        is_code_visibility,
        is_descriptor_binding,
        is_descriptor_visibility,
        is_visibility_agrees,
        is_text_type,
        is_text_flags,
        is_rodata_type,
        is_rodata_flags,
        is_text_extent,
        is_code_aligned,
        is_code_extent,
        is_code_word_aligned,
        is_rodata_extent,
        is_descriptor_aligned,
        is_descriptor_extent,
    ]
    .iter()
    .all(|fact| *fact))
}
/// The kernel descriptor bytes that must be zero, clear or absent.
fn descriptor_bytes_admit(bytes: &[u8]) -> Result<bool> {
    debug_assert_eq!(bytes.len(), 64);
    let is_reserved_zero = bytes[12..16].iter().chain(&bytes[24..48]).chain(&bytes[58..64]).all(|byte| *byte == 0);
    let is_kernel_address_zero = u64_at(bytes, 16)? == 0;
    let is_group_segment_clear = u32_at(bytes, 48)? & 0xfb50_0c00 == 0;
    let is_private_segment_clear = u32_at(bytes, 52)? & 0x80ff_e040 == 0;
    let is_properties_clear = u16_at(bytes, 56)? & !0x087f == 0;
    let is_dispatch_clear = (u32_at(bytes, 52)? >> 11) & 3 != 3;
    Ok([
        is_reserved_zero,
        is_kernel_address_zero,
        is_group_segment_clear,
        is_private_segment_clear,
        is_properties_clear,
        is_dispatch_clear,
    ]
    .iter()
    .all(|fact| *fact))
}
/// The one entry relocation that points at the kernel descriptor.
fn entry_relocation(
    elf: &Elf<'_>,
    code: Symbol<'_>,
    descriptor: Symbol<'_>,
    symtab: usize,
    symbols: &[u8],
) -> Result<()> {
    let descriptor_value = descriptor.value()?;
    debug_assert!(descriptor_value % 64 == 0);
    debug_assert!(symbols.len().is_multiple_of(24));
    let mut is_entry_relocation = false;
    for section in elf.sections.chunks_exact(64) {
        if u32_at(section, 4)? != 4 || usize::try_from(u32_at(section, 44)?)? != descriptor.section()? {
            continue;
        }
        if usize::try_from(u32_at(section, 40)?)? != symtab {
            return Err(invalid("gpu-kernel-relocation-link"));
        }
        for relocation in elf.contents(section)?.chunks_exact(24) {
            let relocation_offset_bytes = u64_at(relocation, 0)?;
            // Consider a full eight-byte window even for unknown relocation
            // kinds: no second relocation may alter any descriptor byte.
            let is_outside_descriptor = relocation_offset_bytes >= descriptor_value.saturating_add(64)
                || relocation_offset_bytes.saturating_add(8) <= descriptor_value;
            if is_outside_descriptor {
                continue;
            }
            let info = u64_at(relocation, 8)?;
            let index = usize::try_from(info >> 32)?;
            let target = Symbol {
                raw: region(symbols, (index as u64).checked_mul(24).ok_or_else(|| invalid("gpu-elf-overflow"))?, 24)?,
                index,
            };
            let addend = i64::from_le_bytes(relocation[16..24].try_into()?);
            let is_first_entry = !is_entry_relocation;
            let is_entry_position = relocation_offset_bytes == descriptor_value.saturating_add(16);
            let is_entry_type = u32::try_from(info & RELOCATION_TYPE_MASK)? == 5;
            let is_entry_section = target.section()? == code.section()?;
            let is_entry_target = target.index == code.index || target.raw[4] & 15 == 3;
            let is_entry_value = i128::from(target.value()?) + i128::from(addend) == i128::from(code.value()?) + 16;
            if [
                is_first_entry,
                is_entry_position,
                is_entry_type,
                is_entry_section,
                is_entry_target,
                is_entry_value,
            ]
            .iter()
            .any(|fact| !*fact)
            {
                return Err(invalid("gpu-kernel-entry-relocation"));
            }
            is_entry_relocation = true;
        }
    }
    if !is_entry_relocation {
        return Err(invalid("gpu-kernel-entry-relocation-missing"));
    }
    Ok(())
}
/// The ELF header facts an admitted gfx900 relocatable object must declare.
struct ElfHeader {
    kind: u16,
    machine: u16,
    flags: u32,
    table: u64,
    count: u16,
    names_index: usize,
}
/// The ELF header, and the architecture and section table it declares.
fn elf_header(data: &[u8]) -> Result<ElfHeader> {
    if data.len() < 64 {
        return Err(invalid("gpu-elf-format-or-abi"));
    }
    debug_assert!(data.len() >= 64);
    let is_ident_valid = &data[..9] == b"\x7fELF\x02\x01\x01\x40\x02";
    let is_object_type = u32_at(data, 20)? == 1;
    let is_elf_class = u16_at(data, 52)? == 64;
    let is_no_entry = u64_at(data, 24)? == 0;
    let is_no_program_header = u64_at(data, 32)? == 0;
    let is_no_section_header = u16_at(data, 56)? == 0;
    let is_within_ceiling = data.len() as u64 <= MAX_ARTIFACT_BYTES;
    if [
        is_ident_valid,
        is_object_type,
        is_elf_class,
        is_no_entry,
        is_no_program_header,
        is_no_section_header,
        is_within_ceiling,
    ]
    .iter()
    .any(|fact| !*fact)
    {
        return Err(invalid("gpu-elf-format-or-abi"));
    }
    let kind = u16_at(data, 16)?;
    let machine = u16_at(data, 18)?;
    let flags = u32_at(data, 48)?;
    let is_relocatable = kind == 1;
    let is_amdgpu = machine == 224;
    let is_abi_flags = flags & 0xff == 0x2c;
    let is_reserved_flags_clear = flags & !0x3ff == 0;
    if [is_relocatable, is_amdgpu, is_abi_flags, is_reserved_flags_clear].iter().any(|fact| !*fact) {
        return Err(invalid("gpu-elf-architecture-or-kind"));
    }
    let table = u64_at(data, 40)?;
    let count = u16_at(data, 60)?;
    let is_entry_stride = u16_at(data, 58)? == 64;
    let is_count_bounded = count > 0 && count <= 4096;
    let is_table_aligned = table >= 64 && table % 8 == 0;
    if [is_entry_stride, is_count_bounded, is_table_aligned].iter().any(|fact| !*fact) {
        return Err(invalid("gpu-elf-section-limit"));
    }
    debug_assert!(table >= 64);
    Ok(ElfHeader {
        kind,
        machine,
        flags,
        table,
        count,
        names_index: usize::from(u16_at(data, 62)?),
    })
}
/// The section content ranges, and the single symbol table index.
struct SectionTable {
    ranges: Vec<(u64, u64)>,
    symtab: Option<usize>,
}

/// The admitted section table, and the symbol table it may declare.
fn section_table(elf: &Elf<'_>, count: u16) -> Result<SectionTable> {
    debug_assert!(elf.sections.len().is_multiple_of(64));
    let mut ranges = Vec::with_capacity(usize::from(count));
    let mut symtab = None;
    for (index, section) in elf.sections.chunks_exact(64).enumerate().skip(1) {
        let section_type = u32_at(section, 4)?;
        let section_offset_bytes = u64_at(section, 24)?;
        let section_size_bytes = u64_at(section, 32)?;
        let align = u64_at(section, 48)?;
        let is_flags_clear = u64_at(section, 16)? == 0;
        let is_aligned = align == 0 || (align.is_power_of_two() && section_offset_bytes % align == 0);
        let is_compression_clear = u64_at(section, 8)? & 0x800 == 0;
        let is_known_section_type = section_type != 9;
        if [is_flags_clear, is_aligned, is_compression_clear, is_known_section_type].iter().any(|fact| !*fact) {
            return Err(invalid("gpu-elf-section"));
        }
        if section_type != 8 {
            elf.contents(section)?;
            if section_size_bytes != 0 {
                ranges.push((section_offset_bytes, section_offset_bytes.saturating_add(section_size_bytes)));
            }
        }
        if section_type == 2 && symtab.replace(index).is_some() {
            return Err(invalid("gpu-elf-ambiguous-symbol-table"));
        }
    }
    debug_assert!(ranges.len() <= usize::from(count));
    Ok(SectionTable { ranges, symtab })
}
/// The section name table, when the image declares one.
fn section_names(elf: &Elf<'_>, names_index: usize) -> Result<()> {
    if names_index != 0 {
        let names = elf.section(names_index)?;
        if u32_at(names, 4)? != 3 {
            return Err(invalid("gpu-elf-section-names"));
        }
        let names = elf.contents(names)?;
        for section in elf.sections.chunks_exact(64) {
            elf_string(names, u32_at(section, 0)?)?;
        }
        return Ok(());
    }
    if elf.sections.chunks_exact(64).any(|section| section[..4] != [0; 4]) {
        return Err(invalid("gpu-elf-section-names"));
    }
    Ok(())
}
/// The symbol table entries keyed by name, and the number of entries.
fn symbol_table<'a>(elf: &Elf<'a>, symtab: usize, count: u16) -> Result<(BTreeMap<&'a str, Symbol<'a>>, usize)> {
    let section = elf.section(symtab)?;
    let entries = elf.contents(section)?;
    let strings = elf.section(usize::try_from(u32_at(section, 40)?)?)?;
    let is_string_section = u32_at(strings, 4)? == 3;
    let is_entry_stride = u64_at(section, 56)? == 24;
    let is_entry_aligned = entries.len().is_multiple_of(24);
    let is_entry_present = entries.len() >= 24;
    if [is_string_section, is_entry_stride, is_entry_aligned, is_entry_present].iter().any(|fact| !*fact) {
        return Err(invalid("gpu-elf-symbol-table"));
    }
    debug_assert!(entries.len().is_multiple_of(24));
    let first_global = usize::try_from(u32_at(section, 44)?)?;
    let is_null_entry = entries[..24].iter().all(|byte| *byte == 0);
    let is_first_global_bounded = first_global <= entries.len() / 24;
    if [is_null_entry, is_first_global_bounded].iter().any(|fact| !*fact) {
        return Err(invalid("gpu-elf-symbol-table"));
    }
    let strings = elf.contents(strings)?;
    if strings.first() != Some(&0) || strings.last() != Some(&0) {
        return Err(invalid("gpu-elf-string-table"));
    }
    let mut names: BTreeSet<&'a str> = BTreeSet::new();
    let mut found: Vec<(&'a str, Symbol<'a>)> = Vec::with_capacity(entries.len() / 24);
    for (index, raw) in entries.chunks_exact(24).enumerate() {
        let name = elf_string(strings, u32_at(raw, 0)?)?;
        let section_index = usize::from(u16_at(raw, 6)?);
        let is_local_binding = raw[4] >> 4 == 0;
        let is_binding_ordered = is_local_binding == (index < first_global);
        let is_section_known = section_index < usize::from(count) || section_index >= 0xff00;
        if [is_binding_ordered, is_section_known].iter().any(|fact| !*fact) {
            return Err(invalid("gpu-elf-symbol-index-or-binding"));
        }
        if !name.is_empty() {
            if !names.insert(name) {
                return Err(invalid("gpu-elf-ambiguous-symbol"));
            }
            found.push((name, Symbol { raw, index }));
        }
    }
    debug_assert!(found.len() <= entries.len() / 24);
    debug_assert_eq!(names.len(), found.len());
    let symbols: BTreeMap<&'a str, Symbol<'a>> = found.into_iter().collect();
    Ok((symbols, entries.len() / 24))
}
/// Every relocation section must target a known section and a known symbol.
fn relocation_sections(elf: &Elf<'_>, symtab: usize, count: u16, symbol_count: usize) -> Result<()> {
    debug_assert!(symbol_count > 0);
    for section in elf.sections.chunks_exact(64) {
        if u32_at(section, 4)? != 4 {
            continue;
        }
        let target = usize::try_from(u32_at(section, 44)?)?;
        let is_target_known = target > 0 && target < usize::from(count);
        let is_symbol_table = usize::try_from(u32_at(section, 40)?)? == symtab;
        let is_entry_stride = u64_at(section, 56)? == 24;
        let is_entry_aligned = u64_at(section, 32)?.is_multiple_of(24);
        if [is_target_known, is_symbol_table, is_entry_stride, is_entry_aligned].iter().any(|fact| !*fact) {
            return Err(invalid("gpu-elf-relocation-table"));
        }
        if u32_at(elf.section(target)?, 4)? == 7 {
            return Err(invalid("gpu-metadata-relocation"));
        }
        for relocation in elf.contents(section)?.chunks_exact(24) {
            let info = u64_at(relocation, 8)?;
            let width = match u32::try_from(info & RELOCATION_TYPE_MASK)? {
                0 => 0,
                1 | 2 | 4 | 6..=11 => 4,
                3 | 5 | 13 => 8,
                14 => 2,
                _ => return Err(invalid("gpu-elf-relocation-type")),
            };
            let end = u64_at(relocation, 0)?.checked_add(width).ok_or_else(|| invalid("gpu-elf-relocation"))?;
            let is_symbol_known = (info >> 32) < symbol_count as u64;
            let is_within_target = end <= u64_at(elf.section(target)?, 32)?;
            if [is_symbol_known, is_within_target].iter().any(|fact| !*fact) {
                return Err(invalid("gpu-elf-relocation"));
            }
        }
    }
    Ok(())
}
/// The AMDHSA metadata version and target triple must agree with the flags.
fn metadata_version(metadata: &Metadata<'_>, flags: u32) -> Result<()> {
    let version = metadata.field("amdhsa.version")?.array()?;
    let target = metadata.field("amdhsa.target")?.string()?;
    if version.len() != 2 {
        return Err(invalid("gpu-metadata-version-or-target"));
    }
    let is_version_major = version[0].uint()? == 1;
    let is_version_minor = version[1].uint()? == 1;
    let is_target_valid = match flags & 0x300 {
        0x200 => target == "amdgcn-amd-amdhsa--gfx900:xnack-",
        0x300 => target == "amdgcn-amd-amdhsa--gfx900:xnack+",
        _ => target == "amdgcn-amd-amdhsa--gfx900",
    };
    if [is_version_major, is_version_minor, is_target_valid].iter().any(|fact| !*fact) {
        return Err(invalid("gpu-metadata-version-or-target"));
    }
    Ok(())
}
/// Every declared kernel must agree with the symbol table and its descriptor.
fn admitted_kernels<'a>(
    elf: &Elf<'a>,
    metadata: &Metadata<'a>,
    symbols: &BTreeMap<&'a str, Symbol<'a>>,
    symtab: usize,
) -> Result<BTreeSet<&'a str>> {
    let entries = elf.contents(elf.section(symtab)?)?;
    debug_assert!(entries.len().is_multiple_of(24));
    let mut kernels = BTreeSet::new();
    let mut names = BTreeSet::new();
    for kernel in metadata.field("amdhsa.kernels")?.array()? {
        let name = kernel.field(".name")?.string()?;
        let descriptor_name = kernel.field(".symbol")?.string()?;
        let entry = descriptor_name.strip_suffix(".kd").ok_or_else(|| invalid("gpu-kernel-metadata-symbol"))?;
        if !kernels.insert(entry) || !names.insert(name) {
            return Err(invalid("gpu-kernel-metadata-ambiguous"));
        }
        let code = *symbols.get(entry).ok_or_else(|| invalid("gpu-entry-point-missing"))?;
        let descriptor = *symbols.get(descriptor_name).ok_or_else(|| invalid("gpu-kernel-descriptor-missing"))?;
        kernel_descriptor(elf, code, descriptor, symtab, entries)?;
        let bytes = region(elf.contents(elf.section(descriptor.section()?)?)?, descriptor.value()?, 64)?;
        kernel_metadata_schema(kernel)?;
        for (key, offset) in [
            (".group_segment_fixed_size", 0),
            (".private_segment_fixed_size", 4),
            (".kernarg_segment_size", 8),
        ] {
            if kernel.field(key)?.uint()? != u64::from(u32_at(bytes, offset)?) {
                return Err(invalid("gpu-kernel-metadata-size"));
            }
        }
        let align = kernel.field(".kernarg_segment_align")?.uint()?;
        let sgprs = kernel.field(".sgpr_count")?.uint()?;
        let vgprs = kernel.field(".vgpr_count")?.uint()?;
        let workgroup = kernel.field(".max_flat_workgroup_size")?.uint()?;
        let is_align_power_of_two = align.is_power_of_two();
        let is_wavefront_width = kernel.field(".wavefront_size")?.uint()? == 64;
        let is_sgprs_bounded = sgprs <= 112;
        let is_vgprs_bounded = vgprs <= 256;
        let is_workgroup_bounded = (1..=1024).contains(&workgroup);
        if [
            is_align_power_of_two,
            is_wavefront_width,
            is_sgprs_bounded,
            is_vgprs_bounded,
            is_workgroup_bounded,
        ]
        .iter()
        .any(|fact| !*fact)
        {
            return Err(invalid("gpu-kernel-metadata-resources"));
        }
    }
    debug_assert_eq!(names.len(), kernels.len());
    Ok(kernels)
}
/// Admit only gfx900 HSA ABI4 relocatable kernels with descriptor, relocation,
/// executable extent and unambiguous AMDHSA metadata agreement.
pub fn inspect(data: &[u8], expected: &[String]) -> Result<Inspection> {
    let header = elf_header(data)?;
    let section_bytes = u64::from(header.count).checked_mul(64).ok_or_else(|| invalid("gpu-elf-overflow"))?;
    let elf = Elf {
        data,
        sections: region(data, header.table, usize::try_from(section_bytes)?)?,
    };
    if elf.section(0)?.iter().any(|byte| *byte != 0) {
        return Err(invalid("gpu-elf-null-section"));
    }
    let sections = section_table(&elf, header.count)?;
    let mut ranges = vec![
        (0_u64, 64_u64),
        (header.table, header.table.saturating_add(section_bytes)),
    ];
    ranges.extend(sections.ranges);
    section_names(&elf, header.names_index)?;
    ranges.sort_unstable();
    if ranges.windows(2).any(|pair| pair[0].1 > pair[1].0) {
        return Err(invalid("gpu-elf-overlapping-sections"));
    }
    let symtab = sections.symtab.ok_or_else(|| invalid("gpu-elf-symbol-table-missing"))?;
    let (symbols, symbol_count) = symbol_table(&elf, symtab, header.count)?;
    relocation_sections(&elf, symtab, header.count, symbol_count)?;
    let amdhsa = metadata(&elf)?;
    metadata_version(&amdhsa, header.flags)?;
    if let Some(printf) = amdhsa.map()?.get("amdhsa.printf") {
        for format in printf.array()? {
            format.string()?;
        }
    }
    let kernels = admitted_kernels(&elf, &amdhsa, &symbols, symtab)?;
    if expected.is_empty() || expected.iter().any(|name| !kernels.contains(name.as_str())) {
        return Err(invalid("gpu-entry-point-missing"));
    }
    debug_assert!(header.count > 0);
    debug_assert!(symbols.len() <= symbol_count);
    Ok(Inspection {
        elf_type: header.kind,
        machine: header.machine,
        flags: header.flags,
        architecture: "gfx900".into(),
        entry_points: expected.to_vec(),
    })
}
#[derive(Serialize)]
struct Provenance<'a> {
    schema: &'static str,
    plan: &'a Plan,
    plan_blake3: String,
    input_files_blake3: BTreeMap<String, String>,
    tool_versions: BTreeMap<String, String>,
    command_args: Vec<String>,
    inspection: Inspection,
    artifact_blake3: String,
    compilation_only: bool,
}
pub fn build(plan: Plan, out: &Path) -> Result<()> {
    let AdmittedPlan { plan, paths } = validate(plan).map_err(|e| invalid(e.to_string()))?;
    let plan = &plan;
    if !crunch_cmake_core::store_root(crunch_cmake_core::StoreRootRequest {
        value: out.display().to_string(),
        prefix: plan.store_prefix.clone(),
    }) {
        return Err(invalid("gpu-output-path-invalid"));
    }
    fs::create_dir(WORK)?;
    for dir in ["home", "tmp"] {
        fs::create_dir(Path::new(WORK).join(dir))?;
    }
    let input_files_blake3 = inputs(plan, &paths)?;
    let tool_versions = versions(plan, &paths)?;
    fs::create_dir(out)?;
    fs::create_dir(out.join("lib"))?;
    fs::create_dir_all(out.join("share/mantle"))?;
    let object = out.join("lib/kernel.o");
    let command_args = compile(plan, &paths, &object)?;
    debug_assert!(!command_args.is_empty());
    if fs::metadata(&object)?.len() > MAX_ARTIFACT_BYTES {
        return Err(invalid("gpu-artifact-size-limit"));
    }
    let inspection = inspect(&fs::read(&object)?, &plan.entry_points)?;
    debug_assert_eq!(inspection.entry_points.len(), plan.entry_points.len());
    let inspection_file = out.join("share/mantle/gpu-llvm-inspection.txt");
    let file = fs::File::create(inspection_file)?;
    run(
        command(&tool(plan, &paths.inspector)?, plan)
            .args(["--file-headers", "--symbols", "--notes"])
            .arg(&object)
            .stdout(Stdio::from(file)),
        "inspect",
    )?;
    let canonical = serde_json::to_vec(plan)?;
    let provenance = Provenance {
        schema: "mantle-gpu-provenance-v1",
        plan,
        plan_blake3: blake3::hash(&canonical).to_hex().to_string(),
        input_files_blake3,
        tool_versions,
        command_args,
        inspection,
        artifact_blake3: hash_file(&object, MAX_ARTIFACT_BYTES)?,
        compilation_only: true,
    };
    fs::write(out.join("share/mantle/gpu-provenance.json"), serde_json::to_vec_pretty(&provenance)?)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn runtime_hash_accepts_large_libraries_without_relaxing_object_limits() {
        let file = tempfile::NamedTempFile::new().unwrap();
        file.as_file().set_len(MAX_ARTIFACT_BYTES + 1).unwrap();
        let mut expected = blake3::Hasher::new();
        let zeros = [0_u8; 65536];
        for _ in 0..MAX_ARTIFACT_BYTES / 65536 {
            expected.update(&zeros);
        }
        expected.update(&[0]);
        assert_eq!(
            hash_file(file.path(), MAX_RUNTIME_LIBRARY_BYTES).unwrap(),
            expected.finalize().to_hex().to_string()
        );
        assert!(hash_file(file.path(), MAX_ARTIFACT_BYTES).is_err());
        file.as_file().set_len(MAX_RUNTIME_LIBRARY_BYTES + 1).unwrap();
        assert!(hash_file(file.path(), MAX_RUNTIME_LIBRARY_BYTES).is_err());
    }
}
