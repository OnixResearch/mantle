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
const MAX_ARTIFACT_BYTES: u64 = 64 * 1024 * 1024;
const MAX_RUNTIME_LIBRARY_BYTES: u64 = 256 * 1024 * 1024;
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
fn versions(plan: &Plan, paths: &Paths) -> Result<BTreeMap<String, String>> {
    let mut versions = BTreeMap::new();
    for (value, path) in [
        (&plan.clang, &paths.clang),
        (&plan.host_compiler, &paths.host_compiler),
        (&plan.inspector, &paths.inspector),
    ] {
        let path = tool(plan, path)?;
        let output = command(&path, plan).arg("--version").output()?;
        let text = String::from_utf8(output.stdout)?;
        if !output.status.success()
            || !text.split_whitespace().any(|part| {
                part.strip_prefix(value.version.as_str())
                    .is_some_and(|suffix| suffix.is_empty() || suffix.starts_with("git"))
            })
        {
            return Err(invalid(format!("gpu-incompatible-tool: {} expected {}", path.display(), value.version)));
        }
        versions.insert(path.display().to_string(), text.trim().to_string());
    }
    let output = command(&tool(plan, &paths.host_compiler)?, plan)
        .arg(format!("--target={}", plan.host_target))
        .arg("-dumpmachine")
        .output()?;
    if !output.status.success() || String::from_utf8(output.stdout)?.trim() != plan.host_target {
        return Err(invalid("gpu-host-compiler-target-mismatch"));
    }
    Ok(versions)
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
fn inputs(plan: &Plan, paths: &Paths) -> Result<BTreeMap<String, String>> {
    let mut digests = BTreeMap::new();
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
    for path in files {
        digests.insert(path.display().to_string(), hash_file(&path, MAX_ARTIFACT_BYTES)?);
    }
    for value in &paths.runtime_libraries {
        let path = member(plan, value, false)?;
        digests.insert(path.display().to_string(), hash_file(&path, MAX_RUNTIME_LIBRARY_BYTES)?);
    }
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
    Ok(digests)
}
fn hash_file(path: &Path, max_bytes: u64) -> Result<String> {
    use std::io::Read;
    let mut file = fs::File::open(path)?;
    if file.metadata()?.len() > max_bytes {
        return Err(invalid("gpu-file-size-limit"));
    }
    let mut hash = blake3::Hasher::new();
    let mut buffer = [0_u8; 65536];
    loop {
        let count = file.read(&mut buffer)?;
        if count == 0 {
            break;
        }
        hash.update(&buffer[..count]);
    }
    Ok(hash.finalize().to_hex().to_string())
}
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
    args.extend([
        member(plan, &paths.kernel, false)?.display().to_string(),
        "-o".into(),
        output.display().to_string(),
    ]);
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
fn region(data: &[u8], offset: u64, length: u64) -> Result<&[u8]> {
    let end = offset.checked_add(length).ok_or_else(|| invalid("gpu-elf-overflow"))?;
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
        region(self.sections, (index as u64).checked_mul(64).ok_or_else(|| invalid("gpu-elf-overflow"))?, 64)
    }
    fn contents(&self, section: &[u8]) -> Result<&'a [u8]> {
        region(self.data, u64_at(section, 24)?, u64_at(section, 32)?)
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
    UInt(u64),
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
            0x00..=0x7f => return Ok(Metadata::UInt(u64::from(tag))),
            0xcc..=0xcf => return Ok(Metadata::UInt(self.number(1 << (tag - 0xcc))?)),
            0xd0..=0xd3 => {
                let count = 1 << (tag - 0xd0);
                let bytes = self.take(count)?;
                return Ok(if bytes[0] & 0x80 == 0 {
                    Metadata::UInt(bytes.iter().fold(0, |value, byte| (value << 8) | u64::from(*byte)))
                } else {
                    Metadata::Other
                });
            }
            0xa0..=0xbf => (0, u64::from(tag & 31)),
            0xd9..=0xdb => (0, self.number(1 << (tag - 0xd9))?),
            0x90..=0x9f => (1, u64::from(tag & 15)),
            0xdc..=0xdd => (1, self.number(2 << (tag - 0xdc))?),
            0x80..=0x8f => (2, u64::from(tag & 15)),
            0xde..=0xdf => (2, self.number(2 << (tag - 0xde))?),
            0xc4..=0xc6 => (3, self.number(1 << (tag - 0xc4))?),
            0xc7..=0xc9 => (3, self.number(1 << (tag - 0xc7))? + 1),
            0xd4..=0xd8 => (3, (1 << (tag - 0xd4)) + 1),
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
                if count > 65536 - self.nodes {
                    return Err(invalid("gpu-metadata-limit"));
                }
                let mut values = Vec::with_capacity(count);
                for _ in 0..count {
                    values.push(self.value(depth + 1)?);
                }
                Ok(Metadata::Array(values))
            }
            2 => {
                if count > (65536 - self.nodes) / 2 {
                    return Err(invalid("gpu-metadata-limit"));
                }
                let mut values = BTreeMap::new();
                for _ in 0..count {
                    let Metadata::String(key) = self.value(depth + 1)? else {
                        return Err(invalid("gpu-metadata-map-key"));
                    };
                    if values.insert(key, self.value(depth + 1)?).is_some() {
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
            Self::UInt(value) => Ok(*value),
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
// Validate known optional fields too. Vendor extensions remain permitted, but
// cannot obscure ill-typed standard fields or duplicate argument-map keys.
fn kernel_metadata_schema(kernel: &Metadata<'_>) -> Result<()> {
    for (key, value) in kernel.map()? {
        match *key {
            ".language" => metadata_enum(value, &["OpenCL C", "OpenCL C++", "HCC", "HIP", "OpenMP", "Assembler"])?,
            ".kind" => metadata_enum(value, &["normal", "init", "fini"])?,
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
            }
            ".uses_dynamic_stack" if !matches!(value, Metadata::Boolean) => {
                return Err(invalid("gpu-metadata-boolean"));
            }
            ".language_version" | ".reqd_workgroup_size" | ".workgroup_size_hint" | ".cluster_dims" => {
                let array = value.array()?;
                let size = if *key == ".language_version" { 2 } else { 3 };
                if array.len() != size {
                    return Err(invalid("gpu-metadata-array-size"));
                }
                for item in array {
                    item.uint()?;
                }
            }
            ".args" => {
                let kernarg = kernel.field(".kernarg_segment_size")?.uint()?;
                for argument in value.array()? {
                    let size = argument.field(".size")?.uint()?;
                    let offset = argument.field(".offset")?.uint()?;
                    metadata_enum(argument.field(".value_kind")?, &[
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
                    ])?;
                    if offset.checked_add(size).is_none_or(|end| end > kernarg) {
                        return Err(invalid("gpu-metadata-argument-extent"));
                    }
                    for (key, value) in argument.map()? {
                        match *key {
                            ".address_space" => {
                                metadata_enum(value, &["private", "global", "constant", "local", "generic", "region"])?
                            }
                            ".access" | ".actual_access" => {
                                metadata_enum(value, &["read_only", "write_only", "read_write"])?
                            }
                            ".name" | ".type_name" | ".value_kind" => match value {
                                // Empty argument names are permitted by the ABI.
                                Metadata::String(text) if !text.contains('\0') => {}
                                _ => return Err(invalid("gpu-metadata-argument-string")),
                            },
                            ".pointee_align" => {
                                value.uint()?;
                            }
                            ".is_const" | ".is_restrict" | ".is_volatile" | ".is_pipe" => match value {
                                Metadata::Boolean => {}
                                _ => return Err(invalid("gpu-metadata-boolean")),
                            },
                            _ => {}
                        }
                    }
                }
            }
            _ => {}
        }
    }
    Ok(())
}
fn metadata<'a>(elf: &Elf<'a>) -> Result<Metadata<'a>> {
    let mut found = None;
    for section in elf.sections.chunks_exact(64) {
        if u32_at(section, 4)? != 7 {
            continue;
        }
        let notes = elf.contents(section)?;
        let mut offset = 0;
        while offset < notes.len() {
            let header = region(notes, offset as u64, 12)?;
            let names = u64::from(u32_at(header, 0)?);
            let size = u64::from(u32_at(header, 4)?);
            let name = region(notes, offset as u64 + 12, names)?;
            let start = offset as u64 + 12 + ((names + 3) & !3);
            let value = region(notes, start, size)?;
            offset = usize::try_from(start + ((size + 3) & !3))?;
            if offset > notes.len() {
                return Err(invalid("gpu-elf-note-padding"));
            }
            if name == b"AMDGPU\0" && u32_at(header, 8)? == 32 {
                if found.is_some() || size > 4 * 1024 * 1024 {
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
            }
        }
    }
    found.ok_or_else(|| invalid("gpu-metadata-missing"))
}
fn kernel_descriptor(
    elf: &Elf<'_>,
    code: Symbol<'_>,
    descriptor: Symbol<'_>,
    symtab: usize,
    symbols: &[u8],
) -> Result<()> {
    let text = elf.section(code.section()?)?;
    let rodata = elf.section(descriptor.section()?)?;
    // STV_PROTECTED is emitted for an otherwise default-visible entry, allowing
    // the static relocation; hidden/internal kernels retain their visibility.
    if code.raw[4] != 0x12
        || !(1..=3).contains(&code.raw[5])
        || descriptor.raw[4] != 0x11
        || descriptor.raw[5] > 3
        || (descriptor.raw[5] != code.raw[5] && !(descriptor.raw[5] == 0 && code.raw[5] == 3))
        || u32_at(text, 4)? != 1
        || u64_at(text, 8)? != 6
        || u32_at(rodata, 4)? != 1
        || u64_at(rodata, 8)? != 2
        || u64_at(text, 48)? < 256
        || code.value()? % 256 != 0
        || code.size()? == 0
        || code.size()? % 4 != 0
        || u64_at(rodata, 48)? < 64
        || descriptor.value()? % 64 != 0
        || descriptor.size()? != 64
    {
        return Err(invalid("gpu-kernel-symbol-or-section"));
    }
    region(elf.contents(text)?, code.value()?, code.size()?)?;
    let value = descriptor.value()?;
    let bytes = region(elf.contents(rodata)?, value, 64)?;
    if bytes[12..16].iter().chain(&bytes[24..48]).chain(&bytes[58..64]).any(|b| *b != 0)
        || u64_at(bytes, 16)? != 0
        || u32_at(bytes, 48)? & 0xfb50_0c00 != 0
        || u32_at(bytes, 52)? & 0x80ff_e040 != 0
        || u16_at(bytes, 56)? & !0x087f != 0
        || (u32_at(bytes, 52)? >> 11) & 3 == 3
    {
        return Err(invalid("gpu-kernel-descriptor"));
    }
    let properties = u16_at(bytes, 56)?;
    let sgprs = [4, 2, 2, 2, 2, 2, 1]
        .iter()
        .enumerate()
        .filter(|(bit, _)| properties & (1 << bit) != 0)
        .map(|(_, count)| count)
        .sum::<u32>();
    if sgprs > 16 || (u32_at(bytes, 52)? >> 1) & 31 != sgprs {
        return Err(invalid("gpu-kernel-user-sgprs"));
    }
    let mut entry_relocation = false;
    for section in elf.sections.chunks_exact(64) {
        if u32_at(section, 4)? != 4 || usize::try_from(u32_at(section, 44)?)? != descriptor.section()? {
            continue;
        }
        if usize::try_from(u32_at(section, 40)?)? != symtab {
            return Err(invalid("gpu-kernel-relocation-link"));
        }
        for relocation in elf.contents(section)?.chunks_exact(24) {
            let offset = u64_at(relocation, 0)?;
            // Consider a full eight-byte window even for unknown relocation
            // kinds: no second relocation may alter any descriptor byte.
            if offset >= value + 64 || offset.saturating_add(8) <= value {
                continue;
            }
            let info = u64_at(relocation, 8)?;
            let index = usize::try_from(info >> 32)?;
            let target = Symbol {
                raw: region(symbols, (index as u64) * 24, 24)?,
                index,
            };
            let addend = i64::from_le_bytes(relocation[16..24].try_into()?);
            if entry_relocation
                || offset != value + 16
                || info as u32 != 5
                || target.section()? != code.section()?
                || (target.index != code.index && target.raw[4] & 15 != 3)
                || i128::from(target.value()?) + i128::from(addend) != i128::from(code.value()?) + 16
            {
                return Err(invalid("gpu-kernel-entry-relocation"));
            }
            entry_relocation = true;
        }
    }
    if !entry_relocation {
        return Err(invalid("gpu-kernel-entry-relocation-missing"));
    }
    Ok(())
}
/// Admit only gfx900 HSA ABI4 relocatable kernels with descriptor, relocation,
/// executable extent and unambiguous AMDHSA metadata agreement.
pub fn inspect(data: &[u8], expected: &[String]) -> Result<Inspection> {
    if data.len() < 64
        || data.len() as u64 > MAX_ARTIFACT_BYTES
        || &data[..9] != b"\x7fELF\x02\x01\x01\x40\x02"
        || u32_at(data, 20)? != 1
        || u16_at(data, 52)? != 64
        || u64_at(data, 24)? != 0
        || u64_at(data, 32)? != 0
        || u16_at(data, 56)? != 0
    {
        return Err(invalid("gpu-elf-format-or-abi"));
    }
    let kind = u16_at(data, 16)?;
    let machine = u16_at(data, 18)?;
    let flags = u32_at(data, 48)?;
    if kind != 1 || machine != 224 || flags & 0xff != 0x2c || flags & !0x3ff != 0 {
        return Err(invalid("gpu-elf-architecture-or-kind"));
    }
    let table = u64_at(data, 40)?;
    let count = u16_at(data, 60)?;
    if u16_at(data, 58)? != 64 || count == 0 || count > 4096 || table < 64 || table % 8 != 0 {
        return Err(invalid("gpu-elf-section-limit"));
    }
    let elf = Elf {
        data,
        sections: region(data, table, u64::from(count) * 64)?,
    };
    if elf.section(0)?.iter().any(|b| *b != 0) {
        return Err(invalid("gpu-elf-null-section"));
    }
    let mut ranges = vec![(0, 64), (table, table + u64::from(count) * 64)];
    let mut symtab = None;
    for (index, section) in elf.sections.chunks_exact(64).enumerate().skip(1) {
        let section_type = u32_at(section, 4)?;
        let offset = u64_at(section, 24)?;
        let size = u64_at(section, 32)?;
        let align = u64_at(section, 48)?;
        if u64_at(section, 16)? != 0
            || (align != 0 && (!align.is_power_of_two() || offset % align != 0))
            || u64_at(section, 8)? & 0x800 != 0
            || section_type == 9
        {
            return Err(invalid("gpu-elf-section"));
        }
        if section_type != 8 {
            elf.contents(section)?;
            if size != 0 {
                ranges.push((offset, offset + size));
            }
        }
        if section_type == 2 && symtab.replace(index).is_some() {
            return Err(invalid("gpu-elf-ambiguous-symbol-table"));
        }
    }
    let names_index = usize::from(u16_at(data, 62)?);
    if names_index != 0 {
        let names = elf.section(names_index)?;
        if u32_at(names, 4)? != 3 {
            return Err(invalid("gpu-elf-section-names"));
        }
        let names = elf.contents(names)?;
        for section in elf.sections.chunks_exact(64) {
            elf_string(names, u32_at(section, 0)?)?;
        }
    } else if elf.sections.chunks_exact(64).any(|section| section[..4] != [0; 4]) {
        return Err(invalid("gpu-elf-section-names"));
    }
    ranges.sort_unstable();
    if ranges.windows(2).any(|pair| pair[0].1 > pair[1].0) {
        return Err(invalid("gpu-elf-overlapping-sections"));
    }
    let symtab = symtab.ok_or_else(|| invalid("gpu-elf-symbol-table-missing"))?;
    let section = elf.section(symtab)?;
    let entries = elf.contents(section)?;
    let strings = elf.section(usize::try_from(u32_at(section, 40)?)?)?;
    if u32_at(strings, 4)? != 3
        || u64_at(section, 56)? != 24
        || entries.len() % 24 != 0
        || entries.len() < 24
        || entries[..24].iter().any(|b| *b != 0)
        || usize::try_from(u32_at(section, 44)?)? > entries.len() / 24
    {
        return Err(invalid("gpu-elf-symbol-table"));
    }
    let strings = elf.contents(strings)?;
    if strings.first() != Some(&0) || strings.last() != Some(&0) {
        return Err(invalid("gpu-elf-string-table"));
    }
    let first_global = usize::try_from(u32_at(section, 44)?)?;
    let mut symbols = BTreeMap::new();
    for (index, raw) in entries.chunks_exact(24).enumerate() {
        let name = elf_string(strings, u32_at(raw, 0)?)?;
        let section_index = usize::from(u16_at(raw, 6)?);
        if (raw[4] >> 4 == 0) != (index < first_global)
            || (section_index >= usize::from(count) && section_index < 0xff00)
        {
            return Err(invalid("gpu-elf-symbol-index-or-binding"));
        }
        if !name.is_empty() && symbols.insert(name, Symbol { raw, index }).is_some() {
            return Err(invalid("gpu-elf-ambiguous-symbol"));
        }
    }
    for section in elf.sections.chunks_exact(64) {
        if u32_at(section, 4)? == 4 {
            let target = usize::try_from(u32_at(section, 44)?)?;
            if target == 0
                || target >= usize::from(count)
                || usize::try_from(u32_at(section, 40)?)? != symtab
                || u64_at(section, 56)? != 24
                || u64_at(section, 32)? % 24 != 0
            {
                return Err(invalid("gpu-elf-relocation-table"));
            }
            if u32_at(elf.section(target)?, 4)? == 7 {
                return Err(invalid("gpu-metadata-relocation"));
            }
            for relocation in elf.contents(section)?.chunks_exact(24) {
                let info = u64_at(relocation, 8)?;
                let width = match info as u32 {
                    0 => 0,
                    1 | 2 | 4 | 6..=11 => 4,
                    3 | 5 | 13 => 8,
                    14 => 2,
                    _ => return Err(invalid("gpu-elf-relocation-type")),
                };
                let end = u64_at(relocation, 0)?.checked_add(width).ok_or_else(|| invalid("gpu-elf-relocation"))?;
                if info >> 32 >= (entries.len() / 24) as u64 || end > u64_at(elf.section(target)?, 32)? {
                    return Err(invalid("gpu-elf-relocation"));
                }
            }
        }
    }
    let metadata = metadata(&elf)?;
    let version = metadata.field("amdhsa.version")?.array()?;
    let target = metadata.field("amdhsa.target")?.string()?;
    let target_valid = match flags & 0x300 {
        0x200 => target == "amdgcn-amd-amdhsa--gfx900:xnack-",
        0x300 => target == "amdgcn-amd-amdhsa--gfx900:xnack+",
        _ => target == "amdgcn-amd-amdhsa--gfx900",
    };
    if version.len() != 2 || version[0].uint()? != 1 || version[1].uint()? != 1 || !target_valid {
        return Err(invalid("gpu-metadata-version-or-target"));
    }
    if let Some(printf) = metadata.map()?.get("amdhsa.printf") {
        for format in printf.array()? {
            format.string()?;
        }
    }
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
        kernel_descriptor(&elf, code, descriptor, symtab, entries)?;
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
        if !align.is_power_of_two()
            || kernel.field(".wavefront_size")?.uint()? != 64
            || sgprs > 112
            || vgprs > 256
            || !(1..=1024).contains(&workgroup)
        {
            return Err(invalid("gpu-kernel-metadata-resources"));
        }
    }
    if expected.is_empty() || expected.iter().any(|name| !kernels.contains(name.as_str())) {
        return Err(invalid("gpu-entry-point-missing"));
    }
    Ok(Inspection {
        elf_type: kind,
        machine,
        flags,
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
    if fs::metadata(&object)?.len() > MAX_ARTIFACT_BYTES {
        return Err(invalid("gpu-artifact-size-limit"));
    }
    let inspection = inspect(&fs::read(&object)?, &plan.entry_points)?;
    let report = out.join("share/mantle/gpu-llvm-inspection.txt");
    let file = fs::File::create(report)?;
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
