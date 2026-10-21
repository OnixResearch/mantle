//! CMake's imperative shell. This runner is an ordinary derivation builder;
//! only the existing store/sandbox pipeline grants input and network authority.
use std::collections::BTreeMap;
use std::fs;
use std::io::Read;
use std::io::Seek;
use std::io::SeekFrom;
use std::os::unix::fs::PermissionsExt;
use std::os::unix::fs::symlink;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;
use std::process::Stdio;

use crunch_cmake_core::Admission;
use crunch_cmake_core::Compiler;
use crunch_cmake_core::Member;
use crunch_cmake_core::Plan as WirePlan;
use crunch_cmake_core::Rejection;
use crunch_cmake_core::RunnerAdmission;
use crunch_cmake_core::RunnerRequest;
use crunch_cmake_core::StoreRootRequest;
use crunch_cmake_core::Tool;
use mantle_rust_plan_core::NativeInputDeclaration;
use serde::Serialize;

pub type Result<T, E = Box<dyn std::error::Error>> = std::result::Result<T, E>;
const WORK: &str = crunch_cmake_core::WORK_DIRECTORY;
const MAX_FILES: usize = 100_000;
const MAX_ARTIFACT_BYTES: u64 = 256 * 1024 * 1024;
const MAX_TREE_BYTES: u64 = 2 * 1024 * 1024 * 1024;

fn invalid(message: impl Into<String>) -> Box<dyn std::error::Error> {
    std::io::Error::other(message.into()).into()
}
struct Plan(Admission);
impl std::ops::Deref for Plan {
    type Target = WirePlan;
    fn deref(&self) -> &WirePlan {
        &self.0.plan
    }
}
impl Serialize for Plan {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error> {
        self.0.plan.serialize(serializer)
    }
}
impl Plan {
    fn tool_list(&self) -> impl Iterator<Item = &Tool> {
        let t = &self.tools;
        [&t.cmake, &t.ninja, &t.cc, &t.cxx, &t.ar, &t.ranlib, &t.linker]
            .into_iter()
            .chain(&self.host_tools)
            .chain(&self.emulator)
    }
    fn tool(&self, tool: &Tool) -> std::result::Result<&str, Rejection> {
        self.tool_list()
            .zip(&self.0.tool_paths)
            .find(|(candidate, _)| *candidate == tool)
            .map(|(_, path)| path.as_str())
            .ok_or_else(|| Rejection {
                code: "cmake-role-undeclared".into(),
                detail: tool.role.clone(),
            })
    }
    fn member(&self, member: &Member) -> std::result::Result<&str, Rejection> {
        self.c_includes
            .iter()
            .chain(&self.cxx_includes)
            .chain(self.toolchain_file.iter())
            .zip(&self.0.member_paths)
            .find(|(candidate, _)| *candidate == member)
            .map(|(_, path)| path.as_str())
            .ok_or_else(|| Rejection {
                code: "cmake-role-undeclared".into(),
                detail: member.role.clone(),
            })
    }
}
fn checked(plan: WirePlan) -> Result<Plan> {
    crunch_cmake_core::admit(plan).map(Plan).map_err(|e| invalid(e.to_string()))
}
fn executable(plan: &Plan, tool: &Tool) -> Result<PathBuf> {
    let path = PathBuf::from(plan.tool(tool).map_err(|e| invalid(e.to_string()))?);
    bound_member(plan, &path)?;
    let meta = fs::metadata(&path)?;
    if !meta.is_file() || meta.permissions().mode() & 0o111 == 0 {
        return Err(invalid(format!("cmake-tool-not-executable: {}", path.display())));
    }
    Ok(path)
}
/// Symlinks are allowed only if their resolution remains in the admitted closure.
fn bound_member(plan: &Plan, path: &Path) -> Result<()> {
    let resolved = fs::canonicalize(path)?;
    if !plan.roles.values().any(|root| resolved.starts_with(root)) {
        return Err(invalid(format!("cmake-input-escape: {}", path.display())));
    }
    debug_assert!(resolved.is_absolute());
    debug_assert!(path.is_absolute());
    Ok(())
}
fn environment(plan: &Plan) -> Result<BTreeMap<String, String>> {
    let mut dirs = vec![format!("{WORK}/wrappers")];
    // CMake/Ninja themselves are addressed absolutely; only explicit build-host
    // generators enter PATH. Target-installed bin/ is never added.
    for tool in &plan.host_tools {
        let path = executable(plan, tool)?;
        dirs.push(path.parent().ok_or_else(|| invalid("tool has no parent"))?.display().to_string());
    }
    let mut env = BTreeMap::from([
        ("PATH".into(), dirs.join(":")),
        ("HOME".into(), format!("{WORK}/home")),
        ("XDG_CONFIG_HOME".into(), format!("{WORK}/home/config")),
        ("XDG_CACHE_HOME".into(), format!("{WORK}/home/cache")),
        ("TMPDIR".into(), format!("{WORK}/tmp")),
        ("LC_ALL".into(), "C".into()),
        ("TZ".into(), "UTC".into()),
        ("SOURCE_DATE_EPOCH".into(), plan.source_date_epoch.to_string()),
        ("ZERO_AR_DATE".into(), "1".into()),
        ("MANTLE_CMAKE_PLAN".into(), serde_json::to_string(plan)?),
    ]);
    // Some declared tools use a build-host runtime. No ambient LD_LIBRARY_PATH,
    // CPATH, CFLAGS, CMAKE_PREFIX_PATH, pkg-config registry or user config survives.
    env.insert("PKG_CONFIG_PATH".into(), String::new());
    env.insert(
        "PKG_CONFIG_LIBDIR".into(),
        plan.dependencies
            .iter()
            .map(|role| format!("{}/lib/pkgconfig", plan.roles[role]))
            .collect::<Vec<_>>()
            .join(":"),
    );
    env.insert("PKG_CONFIG_SYSROOT_DIR".into(), plan.roles[&plan.sysroot].clone());
    Ok(env)
}
fn command(program: &Path, env: &BTreeMap<String, String>) -> Command {
    let mut command = Command::new(program);
    command.env_clear().envs(env).current_dir(WORK).stdin(Stdio::null());
    command
}
fn run(command: &mut Command, phase: &str) -> Result<()> {
    let status = command.status()?;
    if !status.success() {
        return Err(invalid(format!(
            "cmake-{phase}-failed: {status}; dependencies must be separately admitted; network acquisition is forbidden"
        )));
    }
    Ok(())
}
fn versions(plan: &Plan, env: &BTreeMap<String, String>) -> Result<BTreeMap<String, String>> {
    let mut observed = BTreeMap::new();
    for tool in plan.tool_list() {
        let path = executable(plan, tool)?;
        let output = command(&path, env).arg("--version").output()?;
        let text = String::from_utf8(output.stdout)?;
        let line = text.lines().next().unwrap_or("");
        if !output.status.success() || !line.contains(&tool.version) {
            return Err(invalid(format!(
                "cmake-incompatible-tool: {} expected {} observed {line}",
                path.display(),
                tool.version
            )));
        }
        observed.insert(path.display().to_string(), line.to_string());
    }
    for tool in [&plan.tools.cc, &plan.tools.cxx] {
        let path = executable(plan, tool)?;
        let mut probe = command(&path, env);
        if plan.compiler == Compiler::Clang {
            probe.arg(format!("--target={}", plan.target.triple));
        }
        let output = probe.arg("-dumpmachine").output()?;
        let triple = String::from_utf8(output.stdout)?;
        if !output.status.success() || triple.trim() != plan.target.triple {
            return Err(invalid(format!(
                "cmake-incompatible-toolchain: expected {} observed {}",
                plan.target.triple,
                triple.trim()
            )));
        }
    }
    Ok(observed)
}
/// Offline CMake-language guard; callers separately prepend their declared
/// ExternalProject rejection module and use the ordinary network-denied sandbox.
pub const OFFLINE_GUARD: &str = r#"
# This is a compatibility guard; the ordinary sandbox owns network isolation.
include_guard(GLOBAL)
function(file)
  if(ARGV0 STREQUAL "DOWNLOAD" OR ARGV0 STREQUAL "UPLOAD")
    message(FATAL_ERROR "cmake-undeclared-download: acquire and admit dependencies separately")
  endif()
  if(ARGV0 STREQUAL "LOCK")
    message(FATAL_ERROR "cmake-file-lock-unsupported: wrapper cannot preserve caller-scope lock lifetime")
  endif()
  # Preserve empty arguments, semicolons, quotes and newlines when forwarding.
  set(_mantle_call "_file(")
  math(EXPR _mantle_last "${ARGC} - 1")
  foreach(_mantle_i RANGE 0 ${_mantle_last})
    set(_mantle_arg "${ARGV${_mantle_i}}")
    set(_mantle_eq "=")
    string(FIND "${_mantle_arg}" "]${_mantle_eq}]" _mantle_found)
    while(NOT _mantle_found EQUAL -1)
      string(APPEND _mantle_eq "=")
      string(FIND "${_mantle_arg}" "]${_mantle_eq}]" _mantle_found)
    endwhile()
    string(APPEND _mantle_call " [${_mantle_eq}[${_mantle_arg}]${_mantle_eq}]")
  endforeach()
  string(APPEND _mantle_call ")")
  cmake_language(EVAL CODE "${_mantle_call}")
  # file's output variables must retain builtin caller-scope semantics.
  get_cmake_property(_mantle_variables VARIABLES)
  foreach(_mantle_var IN LISTS _mantle_variables)
    if(NOT _mantle_var MATCHES "^(_mantle_|ARGV|ARGC|ARGN|CMAKE_CURRENT_FUNCTION)")
      set("${_mantle_var}" "${${_mantle_var}}" PARENT_SCOPE)
    endif()
  endforeach()
endfunction()
"#;
/// Lower to a generic derivation; no scheduler/tool-specific dispatch is needed.
#[derive(Serialize)]
pub struct Derivation {
    pub name: String,
    pub builder: String,
    pub args: Vec<String>,
    pub system: String,
    pub outputs: Vec<String>,
    pub addressing_mode: String,
    pub inputs: Vec<String>,
    pub env: BTreeMap<String, String>,
}
pub fn lower(plan: WirePlan, runner: Tool) -> Result<Derivation> {
    let RunnerAdmission { plan, builder } =
        crunch_cmake_core::admit_runner(RunnerRequest { plan, runner }).map_err(|e| invalid(e.to_string()))?;
    let json = serde_json::to_string(&plan)?;
    Ok(Derivation {
        name: plan.name,
        builder,
        args: vec!["run".into()],
        system: plan.build_platform,
        outputs: vec!["out".into()],
        addressing_mode: "input-addressed".into(),
        inputs: plan.roles.into_values().collect(),
        env: BTreeMap::from([("MANTLE_CMAKE_PLAN".into(), json)]),
    })
}
pub fn native_input(
    output_root: &str,
    role: &str,
    root_env: &str,
    store_prefix: &str,
) -> Result<NativeInputDeclaration> {
    if !crunch_cmake_core::store_root(StoreRootRequest {
        value: output_root.into(),
        prefix: store_prefix.into(),
    }) || !crunch_cmake_core::identifier(role.into())
    {
        return Err(invalid("cmake-native-export-invalid"));
    }
    let declaration = NativeInputDeclaration {
        inputs: BTreeMap::from([(role.into(), output_root.into())]),
        env: BTreeMap::from([(root_env.into(), format!("{{{{role:{role}}}}}"))]),
        path: Vec::new(),
    };
    mantle_rust_plan_core::resolve_native_inputs(&declaration).map_err(|e| invalid(e.to_string()))?;
    Ok(declaration)
}
pub fn build(plan: WirePlan, out: &Path) -> Result<()> {
    let lowering = crunch_cmake_core::lower(crunch_cmake_core::LowerRequest {
        plan,
        output: out.display().to_string(),
    })
    .map_err(|e| invalid(e.to_string()))?;
    let lowered = lowering.lowered;
    let admitted = Plan(lowering.admission);
    let plan = &admitted;
    fs::create_dir(WORK)?; // A build never silently reuses an ambient CMake cache.
    for dir in ["home/config", "home/cache", "tmp", "wrappers", "modules", "stage"] {
        fs::create_dir_all(Path::new(WORK).join(dir))?;
    }
    let source = Path::new(&lowered.source);
    bound_member(plan, source)?;
    let cmakelists = source.join("CMakeLists.txt");
    bound_member(plan, &cmakelists)?;
    for member in plan.c_includes.iter().chain(&plan.cxx_includes).chain(plan.toolchain_file.iter()) {
        bound_member(plan, Path::new(plan.member(member).map_err(|e| invalid(e.to_string()))?))?;
    }
    let runner = std::env::current_exe()?;
    for name in ["mantle-cc", "mantle-cxx"] {
        symlink(&runner, Path::new(WORK).join("wrappers").join(name))?;
    }
    symlink(executable(plan, &plan.tools.linker)?, Path::new(WORK).join("wrappers/ld"))?;
    fs::write(Path::new(WORK).join("toolchain.cmake"), lowered.toolchain_cmake)?;
    fs::write(Path::new(WORK).join("guard.cmake"), OFFLINE_GUARD)?;
    fs::write(
        Path::new(WORK).join("modules/ExternalProject.cmake"),
        "message(FATAL_ERROR \"cmake-undeclared-download: ExternalProject is unsupported; admit a separate derivation\")\n",
    )?;
    let env = environment(plan)?;
    let versions = versions(plan, &env)?;
    let cmake = executable(plan, &plan.tools.cmake)?;
    run(command(&cmake, &env).args(lowered.configure_args), "configure")?;
    run(command(&cmake, &env).args(lowered.build_args), "build")?;
    let stage = Path::new(WORK).join("stage");
    run(command(&cmake, &env).env("DESTDIR", &stage).args(lowered.install_args), "install")?;
    let installed = stage.join(out.strip_prefix("/")?);
    validate_stage(&stage, &installed)?;
    fs::create_dir(out)?;
    let mut budget = TreeBudget { files: 0, bytes: 0 };
    copy_installed(&installed, out, out, lowered.expected_elf_machine, &mut budget)?;
    for member in &plan.outputs {
        let path = out.join(member);
        if !fs::metadata(&path)?.is_file() {
            return Err(invalid(format!("cmake-output-missing: {member}")));
        }
    }
    write_provenance(plan, out, versions)?;
    Ok(())
}
fn validate_stage(stage: &Path, installed: &Path) -> Result<()> {
    let mut current = stage.to_path_buf();
    for component in installed.strip_prefix(stage)?.components() {
        let std::path::Component::Normal(expected) = component else {
            return Err(invalid("cmake-install-prefix-escape"));
        };
        for entry in fs::read_dir(&current)? {
            if entry?.file_name().as_os_str() != expected {
                return Err(invalid("cmake-install-prefix-escape"));
            }
        }
        current.push(expected);
        let meta = fs::symlink_metadata(&current)?;
        if !meta.is_dir() || meta.file_type().is_symlink() {
            return Err(invalid("cmake-install-prefix-escape"));
        }
    }
    debug_assert!(current == installed);
    debug_assert!(current.starts_with(stage));
    Ok(())
}

#[cfg(test)]
mod stage_tests {
    use super::validate_stage;
    #[test]
    fn absolute_install_destinations_outside_prefix_are_not_silently_discarded() {
        let stage = tempfile::tempdir().unwrap();
        let installed = stage.path().join("mantle/store/output");
        std::fs::create_dir_all(&installed).unwrap();
        assert!(validate_stage(stage.path(), &installed).is_ok());
        std::fs::create_dir_all(stage.path().join("etc")).unwrap();
        assert!(
            validate_stage(stage.path(), &installed)
                .unwrap_err()
                .to_string()
                .contains("cmake-install-prefix-escape")
        );
    }
}

struct TreeBudget {
    files: usize,
    bytes: u64,
}
fn copy_installed(source: &Path, dest: &Path, root: &Path, machine: u16, budget: &mut TreeBudget) -> Result<()> {
    let mut entries = fs::read_dir(source)?.take(MAX_FILES + 1).collect::<std::io::Result<Vec<_>>>()?;
    entries.sort_by_key(std::fs::DirEntry::file_name);
    for entry in entries {
        budget.files += 1;
        if budget.files > MAX_FILES {
            return Err(invalid("cmake-install-limit"));
        }
        let name = entry.file_name();
        if name.to_str().is_none_or(|s| !crunch_cmake_core::identifier(s.into())) {
            return Err(invalid("cmake-installed-name-invalid"));
        }
        let src = entry.path();
        let dst = dest.join(name);
        let relative = dst.strip_prefix(root)?;
        if relative.components().count() > 64 || relative.as_os_str().len() > 1024 {
            return Err(invalid("cmake-install-limit"));
        }
        let meta = fs::symlink_metadata(&src)?;
        if meta.file_type().is_symlink() {
            let target = fs::read_link(&src)?;
            if target.is_absolute() {
                return Err(invalid("cmake-install-symlink-escape"));
            }
            let canonical = fs::canonicalize(&src)?;
            // Resolve against the complete stage (not only a nested directory).
            let stage_root = Path::new(WORK).join("stage").join(root.strip_prefix("/")?);
            if !canonical.starts_with(stage_root) {
                return Err(invalid("cmake-install-symlink-escape"));
            }
            symlink(target, dst)?;
        } else if meta.is_dir() {
            fs::create_dir(&dst)?;
            copy_installed(&src, &dst, root, machine, budget)?;
        } else if meta.is_file() {
            budget.bytes = budget.bytes.checked_add(meta.len()).ok_or_else(|| invalid("cmake-install-limit"))?;
            if meta.len() > MAX_ARTIFACT_BYTES || budget.bytes > MAX_TREE_BYTES {
                return Err(invalid("cmake-install-limit"));
            }
            inspect_architecture(&src, machine)?;
            fs::copy(&src, &dst)?;
            fs::set_permissions(
                &dst,
                fs::Permissions::from_mode(if meta.permissions().mode() & 0o111 != 0 {
                    0o755
                } else {
                    0o644
                }),
            )?;
        } else {
            return Err(invalid("cmake-install-special-file"));
        }
    }
    Ok(())
}
/// Inspect ELF files and every ELF member of ordinary ar archives; never execute.
pub fn inspect_architecture(path: &Path, machine: u16) -> Result<()> {
    let mut file = fs::File::open(path)?;
    let length = file.metadata()?.len();
    if length < 8 {
        return Ok(());
    }
    let mut magic = [0_u8; 8];
    file.read_exact(&mut magic)?;
    if &magic == b"!<thin>\n" {
        return Err(invalid("cmake-thin-archive-unsupported"));
    }
    if &magic == b"!<arch>\n" {
        let mut offset = 8_u64;
        while offset < length {
            if offset + 60 > length {
                return Err(invalid("cmake-archive-invalid"));
            }
            file.seek(SeekFrom::Start(offset))?;
            let mut header = [0_u8; 60];
            file.read_exact(&mut header)?;
            if &header[58..60] != b"`\n" || header.starts_with(b"#1/") {
                return Err(invalid("cmake-archive-format-unsupported"));
            }
            let size: u64 = std::str::from_utf8(&header[48..58])?.trim().parse()?;
            let start = offset + 60;
            if start.checked_add(size).is_none_or(|end| end > length) {
                return Err(invalid("cmake-archive-invalid"));
            }
            if size >= 20 {
                inspect_elf(&mut file, machine)?;
            }
            offset = start
                .checked_add(size)
                .and_then(|n| n.checked_add(size % 2))
                .ok_or_else(|| invalid("cmake-archive-invalid"))?;
        }
    } else if length >= 20 {
        file.seek(SeekFrom::Start(0))?;
        inspect_elf(&mut file, machine)?;
    }
    Ok(())
}
fn inspect_elf(file: &mut fs::File, machine: u16) -> Result<()> {
    let mut header = [0_u8; 20];
    file.read_exact(&mut header)?;
    if &header[..4] != b"\x7fELF" {
        return Ok(());
    }
    if header[4] != 2 || header[5] != 1 || u16::from_le_bytes([header[18], header[19]]) != machine {
        return Err(invalid("cmake-incompatible-artifact: expected little-endian ELF64 target architecture"));
    }
    debug_assert!(header[4] == 2);
    debug_assert!(header[5] == 1);
    Ok(())
}
#[derive(Serialize)]
struct Artifact {
    path: String,
    blake3: String,
    size_bytes: u64,
    executable: bool,
    symlink_target: Option<String>,
}
fn artifacts(root: &Path, dir: &Path, result: &mut Vec<Artifact>) -> Result<()> {
    let mut entries = fs::read_dir(dir)?.collect::<std::io::Result<Vec<_>>>()?;
    entries.sort_by_key(std::fs::DirEntry::file_name);
    for entry in entries {
        let path = entry.path();
        let meta = fs::symlink_metadata(&path)?;
        if meta.is_dir() {
            artifacts(root, &path, result)?;
        } else if meta.is_file() {
            let mut hasher = blake3::Hasher::new();
            let mut file = fs::File::open(&path)?;
            let mut buffer = [0_u8; 65536];
            loop {
                let n = file.read(&mut buffer)?;
                if n == 0 {
                    break;
                }
                hasher.update(&buffer[..n]);
            }
            result.push(Artifact {
                path: path.strip_prefix(root)?.display().to_string(),
                blake3: hasher.finalize().to_hex().to_string(),
                size_bytes: meta.len(),
                executable: meta.permissions().mode() & 0o111 != 0,
                symlink_target: None,
            });
        } else if meta.file_type().is_symlink() {
            let target = fs::read_link(&path)?
                .into_os_string()
                .into_string()
                .map_err(|_| invalid("cmake-symlink-non-utf8"))?;
            result.push(Artifact {
                path: path.strip_prefix(root)?.display().to_string(),
                blake3: blake3::hash(target.as_bytes()).to_hex().to_string(),
                size_bytes: target.len() as u64,
                executable: false,
                symlink_target: Some(target),
            });
        }
    }
    Ok(())
}
fn write_provenance(plan: &Plan, out: &Path, versions: BTreeMap<String, String>) -> Result<()> {
    let mut installed = Vec::new();
    artifacts(out, out, &mut installed)?;
    let identity = blake3::hash(&serde_json::to_vec(plan)?).to_hex().to_string();
    let dir = out.join("share/mantle");
    fs::create_dir_all(&dir)?;
    let path = dir.join("cmake-provenance.json");
    if path.exists() {
        return Err(invalid("cmake-provenance-collision"));
    }
    fs::write(
        path,
        serde_json::to_vec_pretty(&serde_json::json!({
            "schema": "mantle-cmake-provenance-v1", "plan_blake3": identity, "plan": plan,
            "observed_tools": versions, "artifacts": installed,
            "non_claims": ["not-compiler-correctness", "not-universal-cmake-compatibility", "not-global-reproducibility"],
        }))?,
    )?;
    Ok(())
}
/// Compiler interposition fixes target/sysroot/includes/linker/normalization and
/// removes execute bits from linked target artifacts. Explicit emulators read
/// ELF directly; direct target execution, including native architecture, fails.
pub fn compile(plan: WirePlan, cxx: bool, args: &[std::ffi::OsString]) -> Result<i32> {
    let admitted = checked(plan)?;
    let plan = &admitted;
    let tool = if cxx { &plan.tools.cxx } else { &plan.tools.cc };
    let mut compiler = command(&executable(plan, tool)?, &environment(plan)?);
    // Relative paths in Ninja rules must retain the caller's build directory.
    compiler.current_dir(std::env::current_dir()?);
    if plan.compiler == Compiler::Clang {
        compiler.arg(format!("--target={}", plan.target.triple));
    }
    compiler.arg(format!("--sysroot={}", plan.roles[&plan.sysroot])).arg("-nostdinc");
    if cxx {
        compiler.arg("-nostdinc++");
    }
    for member in (if cxx { plan.cxx_includes.iter() } else { [].iter() }).chain(&plan.c_includes) {
        let path = plan.member(member).map_err(|e| invalid(e.to_string()))?;
        bound_member(plan, Path::new(path))?;
        compiler.arg("-isystem").arg(path);
    }
    compiler.arg(format!("-B{WORK}/wrappers/"));
    if plan.compiler == Compiler::Clang {
        compiler.arg(format!("-fuse-ld={}", plan.tool(&plan.tools.linker).map_err(|e| invalid(e.to_string()))?));
    }
    compiler
        .arg(format!("-ffile-prefix-map={}=./source", plan.roles[&plan.source]))
        .arg(format!("-ffile-prefix-map={WORK}=./build"));
    compiler.args(args);
    let status = compiler.status()?;
    if status.success() {
        let output = args
            .windows(2)
            .find(|pair| pair[0] == "-o")
            .map(|pair| PathBuf::from(&pair[1]))
            .unwrap_or_else(|| PathBuf::from("a.out"));
        if output.is_file() {
            let meta = fs::metadata(&output)?;
            fs::set_permissions(&output, fs::Permissions::from_mode(meta.permissions().mode() & !0o111))?;
        }
    }
    Ok(status.code().unwrap_or(1))
}
