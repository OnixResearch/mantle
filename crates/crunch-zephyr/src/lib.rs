//! Offline Zephyr derivation builder. Store admission and sandbox authority remain generic.
use std::collections::BTreeMap;
use std::fs;
use std::io::Read;
use std::io::Write;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;
use std::process::Stdio;

use crunch_cmake::Derivation;
use crunch_cmake_core::StoreRootRequest;
use crunch_cmake_core::cmake_literal;
use crunch_cmake_core::store_root;
use crunch_zephyr_core::ARTIFACTS;
use crunch_zephyr_core::Admission;
use crunch_zephyr_core::Member;
use crunch_zephyr_core::Plan as WirePlan;
use crunch_zephyr_core::Rejection;
use crunch_zephyr_core::ResolvedWest;
use crunch_zephyr_core::RunnerAdmission;
use crunch_zephyr_core::RunnerRequest;
use crunch_zephyr_core::Tool;
use serde::Serialize;

pub type Result<T, E = Box<dyn std::error::Error>> = std::result::Result<T, E>;
const WORK: &str = "/build/mantle-zephyr";
const MAX_FILES: u64 = 400_000;
const MAX_BYTES: u64 = 8_589_934_592;
const MAX_DEPTH: u32 = 64;
/// Chunk size of the bounded read loops.
const CHUNK_BYTES: usize = 64 * 1024;
/// Read iterations over one tree file: the byte bound over the chunk size, plus the empty read.
const MAX_TREE_CHUNKS: u64 = 131_073;
/// Largest published artifact.
const MAX_ARTIFACT_BYTES: u64 = 67_108_864;
/// Read iterations over one artifact: the byte bound over the chunk size, plus the empty read.
const MAX_ARTIFACT_CHUNKS: u64 = 1_025;
/// Working-set size of the explicit traversal stacks.
const FRAME_CAPACITY: usize = 65;
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
    fn root(&self, role: &str) -> std::result::Result<&str, Rejection> {
        self.roles.get(role).map(|input| input.root.as_str()).ok_or_else(|| Rejection {
            code: "zephyr-missing-role".into(),
            detail: role.into(),
        })
    }
    fn tools(&self) -> impl Iterator<Item = &Tool> {
        [
            &self.tools.cmake,
            &self.tools.ninja,
            &self.tools.python,
            &self.tools.dtc,
        ]
        .into_iter()
        .chain(self.tools.host.iter())
    }
    fn tool(&self, tool: &Tool) -> std::result::Result<&str, Rejection> {
        self.tools()
            .zip(&self.0.tool_paths)
            .find(|(candidate, _)| *candidate == tool)
            .map(|(_, path)| path.as_str())
            .ok_or_else(|| Rejection {
                code: "zephyr-missing-role".into(),
                detail: tool.role.clone(),
            })
    }
    fn member(&self, member: &Member) -> std::result::Result<&str, Rejection> {
        self.kconfig
            .iter()
            .chain(&self.overlays)
            .chain(core::iter::once(&self.resolved_west))
            .zip(&self.0.member_paths)
            .find(|(candidate, _)| *candidate == member)
            .map(|(_, path)| path.as_str())
            .ok_or_else(|| Rejection {
                code: "zephyr-missing-role".into(),
                detail: member.role.clone(),
            })
    }
}
fn checked(plan: WirePlan) -> Result<Plan> {
    crunch_zephyr_core::admit(plan).map(Plan).map_err(|e| invalid(e.to_string()))
}

/// A CMake cache key, kept distinct from its value so the two cannot be swapped.
struct SettingKey<'a>(&'a str);

/// Length-delimited, sorted tree encoding; ignores mtimes, uid/gid and writable bits.
/// Symlink target bytes and executable bits are content, not their resolved host path.
pub fn tree_blake3(root: &Path) -> Result<String> {
    if !fs::symlink_metadata(root)?.is_dir() {
        return Err(invalid("zephyr-input-not-directory"));
    }
    let mut state = TreeState {
        hasher: blake3::Hasher::new(),
        files: 0,
        bytes: 0,
    };
    state.hasher.update(b"mantle-zephyr-tree-v1\0");
    hash_directory(root, root, 0, &mut state)?;
    Ok(state.hasher.finalize().to_hex().to_string())
}
struct TreeState {
    hasher: blake3::Hasher,
    files: u64,
    bytes: u64,
}
fn frame(hasher: &mut blake3::Hasher, bytes: &[u8]) {
    hasher.update(&(bytes.len() as u64).to_le_bytes());
    hasher.update(bytes);
}
/// One directory of the explicit depth first tree traversal.
struct TreeFrame {
    entries: Vec<fs::DirEntry>,
    index: usize,
    depth: u32,
}
impl TreeFrame {
    fn new(dir: &Path, depth: u32) -> Result<Self> {
        if depth > MAX_DEPTH {
            return Err(invalid("zephyr-tree-depth-limit"));
        }
        let mut entries = fs::read_dir(dir)?.collect::<std::io::Result<Vec<_>>>()?;
        entries.sort_by_key(|entry| entry.file_name());
        Ok(Self {
            entries,
            index: 0,
            depth,
        })
    }
    fn next_path(&mut self) -> Option<PathBuf> {
        let entry = self.entries.get(self.index)?;
        self.index = self.index.saturating_add(1);
        Some(entry.path())
    }
}
fn hash_directory(root: &Path, dir: &Path, depth: u32, state: &mut TreeState) -> Result<()> {
    let mut pending: Vec<TreeFrame> = Vec::with_capacity(FRAME_CAPACITY);
    let directory = TreeFrame::new(dir, depth)?;
    check_frame_capacity(&directory, state)?;
    pending.push(directory);
    while !pending.is_empty() {
        let next = pending.last_mut().and_then(TreeFrame::next_path);
        let Some(path) = next else {
            pending.pop();
            continue;
        };
        if state.files >= MAX_FILES {
            return Err(invalid("zephyr-tree-file-limit"));
        }
        state.files = state.files.saturating_add(1);
        let relative = path.strip_prefix(root)?.to_str().ok_or_else(|| invalid("zephyr-non-utf8-path"))?;
        if relative.len() > 4096 {
            return Err(invalid("zephyr-tree-path-limit"));
        }
        frame(&mut state.hasher, relative.as_bytes());
        let meta = fs::symlink_metadata(&path)?;
        if meta.is_dir() {
            state.hasher.update(b"d");
            let child_depth = pending.last().map(|parent| parent.depth).unwrap_or(depth).saturating_add(1);
            let child = TreeFrame::new(&path, child_depth)?;
            check_frame_capacity(&child, state)?;
            pending.push(child);
        } else if meta.is_file() {
            hash_file(&path, &meta, state)?;
        } else if meta.file_type().is_symlink() {
            state.hasher.update(b"l");
            let target = fs::read_link(&path)?;
            frame(&mut state.hasher, target.to_str().ok_or_else(|| invalid("zephyr-non-utf8-link"))?.as_bytes());
        } else {
            return Err(invalid(format!("zephyr-special-input: {}", path.display())));
        }
    }
    debug_assert!(state.files <= MAX_FILES);
    debug_assert!(state.bytes <= MAX_BYTES);
    Ok(())
}
/// A directory may not hold more entries than the remaining file budget.
fn check_frame_capacity(directory: &TreeFrame, state: &TreeState) -> Result<()> {
    if directory.entries.len() as u64 > MAX_FILES.saturating_sub(state.files) {
        return Err(invalid("zephyr-tree-file-limit"));
    }
    Ok(())
}
/// Hash one regular file with its executable bit, its length and its exact bytes.
fn hash_file(path: &Path, meta: &fs::Metadata, state: &mut TreeState) -> Result<()> {
    state.hasher.update(b"f");
    state.hasher.update(&[u8::from(meta.permissions().mode() & 0o111 != 0)]);
    state.bytes = state.bytes.checked_add(meta.len()).ok_or_else(|| invalid("zephyr-tree-byte-limit"))?;
    if state.bytes > MAX_BYTES {
        return Err(invalid("zephyr-tree-byte-limit"));
    }
    state.hasher.update(&meta.len().to_le_bytes());
    let mut file = fs::File::open(path)?;
    let mut buffer = [0; CHUNK_BYTES];
    for _ in 0..MAX_TREE_CHUNKS {
        let n = file.read(&mut buffer)?;
        if n == 0 {
            break;
        }
        state.hasher.update(&buffer[..n]);
    }
    Ok(())
}
fn bound(plan: &Plan, path: &Path) -> Result<PathBuf> {
    let resolved =
        fs::canonicalize(path).map_err(|e| invalid(format!("zephyr-missing-input: {}: {e}", path.display())))?;
    if !plan.roles.values().any(|input| resolved.starts_with(&input.root)) {
        return Err(invalid(format!("zephyr-path-escape: {}", path.display())));
    }
    Ok(resolved)
}
/// One directory of the explicit depth first link audit.
struct AuditFrame {
    entries: fs::ReadDir,
    depth: u32,
}
impl AuditFrame {
    fn new(dir: &Path, depth: u32) -> Result<Self> {
        if depth > MAX_DEPTH {
            return Err(invalid("zephyr-tree-depth-limit"));
        }
        Ok(Self {
            entries: fs::read_dir(dir)?,
            depth,
        })
    }
}
fn audit_links(plan: &Plan, dir: &Path, depth: u32) -> Result<()> {
    let mut pending: Vec<AuditFrame> = Vec::with_capacity(FRAME_CAPACITY);
    pending.push(AuditFrame::new(dir, depth)?);
    while !pending.is_empty() {
        let next = pending.last_mut().and_then(|parent| parent.entries.next());
        let Some(entry) = next else {
            pending.pop();
            continue;
        };
        audit_entry(plan, &mut pending, entry?, depth)?;
    }
    Ok(())
}
/// Audit one entry and queue the subdirectory it names.
fn audit_entry(plan: &Plan, pending: &mut Vec<AuditFrame>, entry: fs::DirEntry, depth: u32) -> Result<()> {
    let path = entry.path();
    if entry.file_type()?.is_symlink() {
        bound(plan, &path)?;
    } else if entry.file_type()?.is_dir() {
        let child_depth = pending.last().map(|parent| parent.depth).unwrap_or(depth).saturating_add(1);
        pending.push(AuditFrame::new(&path, child_depth)?);
    }
    Ok(())
}
fn member(plan: &Plan, item: &Member) -> Result<PathBuf> {
    let path = PathBuf::from(plan.member(item).map_err(|e| invalid(e.to_string()))?);
    bound(plan, &path)?;
    Ok(path)
}
fn executable(plan: &Plan, tool: &Tool) -> Result<PathBuf> {
    let path = PathBuf::from(plan.tool(tool).map_err(|e| invalid(e.to_string()))?);
    bound(plan, &path)?;
    let metadata = fs::metadata(&path)?;
    if !metadata.is_file() || metadata.permissions().mode() & 0o111 == 0 {
        return Err(invalid(format!("zephyr-tool-not-executable: {}", path.display())));
    }
    Ok(path)
}

pub fn identity(plan: WirePlan) -> Result<String> {
    let plan = crunch_zephyr_core::validate(plan).map_err(|e| invalid(e.to_string()))?;
    identity_ref(&plan)
}
fn identity_ref(plan: &WirePlan) -> Result<String> {
    let bytes = serde_json::to_vec(plan)?;
    let mut hash = blake3::Hasher::new();
    hash.update(b"mantle-zephyr-plan-v1\0");
    hash.update(&bytes);
    Ok(hash.finalize().to_hex().to_string())
}
pub fn lower(plan: WirePlan, runner: Tool) -> Result<Derivation> {
    let RunnerAdmission { plan, builder } =
        crunch_zephyr_core::admit_runner(RunnerRequest { plan, runner }).map_err(|e| invalid(e.to_string()))?;
    let json = serde_json::to_string(&plan)?;
    Ok(Derivation {
        name: plan.name,
        builder,
        args: vec!["run".into()],
        system: plan.build_platform,
        outputs: vec!["out".into()],
        addressing_mode: "input-addressed".into(),
        inputs: plan.roles.into_values().map(|input| input.root).collect(),
        env: BTreeMap::from([("MANTLE_ZEPHYR_PLAN".into(), json)]),
    })
}
fn environment(plan: &Plan) -> Result<BTreeMap<String, String>> {
    let mut dirs = Vec::with_capacity(plan.tools().count());
    for tool in plan.tools() {
        let path = executable(plan, tool)?;
        dirs.push(path.parent().ok_or_else(|| invalid("zephyr-tool-parent"))?.display().to_string());
    }
    dirs.sort();
    dirs.dedup();
    debug_assert!(!dirs.is_empty());
    debug_assert!(dirs.is_sorted());
    Ok(BTreeMap::from([
        ("PATH".into(), dirs.join(":")),
        ("HOME".into(), format!("{WORK}/home")),
        ("TMPDIR".into(), format!("{WORK}/tmp")),
        ("XDG_CACHE_HOME".into(), format!("{WORK}/cache")),
        ("XDG_CONFIG_HOME".into(), format!("{WORK}/home/config")),
        ("LC_ALL".into(), "C".into()),
        ("TZ".into(), "UTC".into()),
        ("SOURCE_DATE_EPOCH".into(), plan.source_date_epoch.to_string()),
        ("PYTHONNOUSERSITE".into(), "1".into()),
        ("PYTHONDONTWRITEBYTECODE".into(), "1".into()),
        ("PYTHONHASHSEED".into(), "0".into()),
        ("PIP_NO_INDEX".into(), "1".into()),
        ("ZEPHYR_BASE".into(), plan.root(&plan.zephyr)?.into()),
        ("ZEPHYR_SDK_INSTALL_DIR".into(), plan.root(&plan.sdk)?.into()),
        ("ZEPHYR_TOOLCHAIN_VARIANT".into(), "zephyr".into()),
        ("CCACHE_DISABLE".into(), "1".into()),
        ("ZERO_AR_DATE".into(), "1".into()),
    ]))
}
fn command(path: &Path, env: &BTreeMap<String, String>) -> Command {
    let mut cmd = Command::new(path);
    cmd.env_clear().envs(env).current_dir(WORK).stdin(Stdio::null());
    cmd
}
fn run(cmd: &mut Command, phase: &str) -> Result<()> {
    let status = cmd.status()?;
    if !status.success() {
        return Err(invalid(format!(
            "zephyr-{phase}-failed: {status}; offline build: acquire and admit dependencies separately"
        )));
    }
    Ok(())
}
fn versions(plan: &Plan, env: &BTreeMap<String, String>) -> Result<BTreeMap<String, String>> {
    let mut observations = Vec::with_capacity(plan.tools().count());
    for tool in plan.tools() {
        let path = executable(plan, tool)?;
        let result = command(&path, env).arg("--version").output()?;
        let output = String::from_utf8(result.stdout)?;
        let line = output.lines().next().unwrap_or("");
        if !result.status.success() || !line.contains(&tool.version) {
            return Err(invalid(format!(
                "zephyr-tool-mismatch: {} expected {} observed {line}",
                path.display(),
                tool.version
            )));
        }
        observations.push((path.display().to_string(), line.into()));
    }
    Ok(observations.into_iter().collect())
}
const CHECK_CLOSURE: &str = r#"
import importlib.metadata as m, importlib.util, json, pathlib, sys, yaml
p=json.loads(sys.argv[1])
if importlib.util.find_spec('west') is not None:
    raise SystemExit('zephyr-python-mismatch: build Python environment must not contain west')
z=pathlib.Path(p['roles'][p['zephyr']]['root'])
w=yaml.safe_load((z/'west.yml').read_text())['manifest']
cmsis=[x for x in w['projects'] if x['name']=='cmsis']
expected=p['west']['modules']['cmsis']
if len(cmsis)!=1 or any(cmsis[0][k]!=expected[k] for k in ['revision','path']):
    raise SystemExit('zephyr-manifest-mismatch: upstream west.yml does not bind resolved CMSIS')
for package,version in {'PyYAML':'6.0.2','pykwalify':'1.8.0','pyelftools':'0.31','packaging':'24.1','anytree':'2.12.1','intelhex':'2.3.0'}.items():
    if m.version(package)!=version: raise SystemExit('zephyr-python-mismatch: '+package)
print(json.dumps({package:m.version(package) for package in ['PyYAML','pykwalify','pyelftools','packaging','anytree','intelhex']},sort_keys=True))
"#;
fn verify_inputs(plan: &Plan, env: &BTreeMap<String, String>) -> Result<()> {
    verify_role_trees(plan)?;
    verify_resolved_west(plan)?;
    verify_release_versions(plan)?;
    verify_closure(plan, env)?;
    verify_sdk_target(plan, env)
}
/// Every declared role must match its admitted tree identity and carry no escaping link.
fn verify_role_trees(plan: &Plan) -> Result<()> {
    for (role, input) in &plan.roles {
        if tree_blake3(Path::new(&input.root))? != input.tree_blake3 {
            return Err(invalid(format!("zephyr-content-mismatch: {role}")));
        }
        audit_links(plan, Path::new(&input.root), 0)?;
    }
    Ok(())
}
/// The acquired resolved west manifest must be the one the plan names.
fn verify_resolved_west(plan: &Plan) -> Result<()> {
    let resolved: ResolvedWest = serde_json::from_slice(&read_bounded(&member(plan, &plan.resolved_west)?, 65536)?)?;
    if resolved != plan.west {
        return Err(invalid("zephyr-manifest-mismatch: acquired resolved-west.json differs from plan"));
    }
    for path in &plan.0.module_paths {
        bound(plan, Path::new(path))?;
    }
    Ok(())
}
/// The acquired SDK and Zephyr releases must be the pinned ones.
fn verify_release_versions(plan: &Plan) -> Result<()> {
    let sdk = Path::new(plan.root(&plan.sdk)?);
    if fs::read_to_string(sdk.join("sdk_version"))?.trim() != plan.sdk_version.as_str() {
        return Err(invalid("zephyr-sdk-mismatch: requires 0.16.8"));
    }
    let version = fs::read_to_string(Path::new(plan.root(&plan.zephyr)?).join("VERSION"))?;
    for expected in [
        "VERSION_MAJOR = 3",
        "VERSION_MINOR = 7",
        "PATCHLEVEL = 0",
        "EXTRAVERSION =",
    ] {
        if !version.lines().any(|line| line.trim() == expected) {
            return Err(invalid("zephyr-release-mismatch: requires 3.7.0"));
        }
    }
    Ok(())
}
/// The build Python environment must bind the pinned module closure.
fn verify_closure(plan: &Plan, env: &BTreeMap<String, String>) -> Result<()> {
    run(
        command(&executable(plan, &plan.tools.python)?, env).args(["-c", CHECK_CLOSURE, &serde_json::to_string(plan)?]),
        "closure-validation",
    )
}
/// The pinned cross compiler must target the declared target triple.
fn verify_sdk_target(plan: &Plan, env: &BTreeMap<String, String>) -> Result<()> {
    let sdk = Path::new(plan.root(&plan.sdk)?);
    let cc = sdk.join(format!("{}/bin/{}-gcc", plan.target, plan.target));
    bound(plan, &cc)?;
    let target = command(&cc, env).arg("-dumpmachine").output()?;
    if !target.status.success() || target.stdout.strip_suffix(b"\n") != Some(plan.target.as_bytes()) {
        return Err(invalid("zephyr-sdk-target-mismatch"));
    }
    Ok(())
}
fn read_bounded(path: &Path, limit: u64) -> Result<Vec<u8>> {
    let file = fs::File::open(path)?;
    if file.metadata()?.len() > limit {
        return Err(invalid("zephyr-file-byte-limit"));
    }
    let mut bytes = Vec::new();
    file.take(limit.saturating_add(1)).read_to_end(&mut bytes)?;
    if bytes.len() as u64 > limit {
        return Err(invalid("zephyr-file-byte-limit"));
    }
    Ok(bytes)
}
fn setting(text: &mut String, key: SettingKey<'_>, value: &str) {
    text.push_str(&format!("set({} {} CACHE STRING \"Mantle owned\" FORCE)\n", key.0, cmake_literal(value.into())));
}
fn configuration(plan: &Plan) -> Result<String> {
    let mut text = format!("include({})\n", cmake_literal(format!("{WORK}/guard.cmake")));
    set_core_settings(&mut text, plan)?;
    set_tool_settings(&mut text, plan)?;
    set_offline_settings(&mut text);
    set_module_settings(&mut text, plan)?;
    set_file_settings(&mut text, plan)?;
    Ok(text)
}
/// The board, toolchain and cache locations the build reads.
fn set_core_settings(text: &mut String, plan: &Plan) -> Result<()> {
    let sdk = plan.root(&plan.sdk)?;
    let zephyr = plan.root(&plan.zephyr)?;
    for (key, value) in [
        ("BOARD", plan.board.as_str()),
        ("ZEPHYR_BASE", zephyr),
        ("ZEPHYR_TOOLCHAIN_VARIANT", "zephyr"),
        ("ZEPHYR_SDK_INSTALL_DIR", sdk),
        ("CMAKE_GENERATOR", "Ninja"),
        ("USER_CACHE_DIR", &format!("{WORK}/cache")),
        ("Zephyr-sdk_DIR", &format!("{sdk}/cmake")),
        ("Zephyr_DIR", &format!("{zephyr}/share/zephyr-package/cmake")),
        ("CMAKE_PROJECT_INCLUDE_BEFORE", &format!("{WORK}/guard.cmake")),
        ("CMAKE_MODULE_PATH", &format!("{WORK}/modules")),
    ] {
        setting(text, SettingKey(key), value);
    }
    Ok(())
}
/// The tool paths and the program directories the build resolves.
fn set_tool_settings(text: &mut String, plan: &Plan) -> Result<()> {
    for (key, tool) in [
        ("CMAKE_MAKE_PROGRAM", &plan.tools.ninja),
        ("PYTHON_EXECUTABLE", &plan.tools.python),
        ("Python3_EXECUTABLE", &plan.tools.python),
        ("DTC", &plan.tools.dtc),
    ] {
        setting(text, SettingKey(key), plan.tool(tool).map_err(|e| invalid(e.to_string()))?);
    }
    let program_dirs = plan
        .tools()
        .map(|tool| {
            let path = executable(plan, tool)?;
            Ok(path.parent().ok_or_else(|| invalid("zephyr-tool-parent"))?.display().to_string())
        })
        .collect::<Result<Vec<_>>>()?;
    setting(text, SettingKey("CMAKE_PROGRAM_PATH"), &program_dirs.join(";"));
    Ok(())
}
/// Every registry, cache and network lookup stays disabled.
fn set_offline_settings(text: &mut String) {
    for key in [
        "CMAKE_FIND_USE_PACKAGE_REGISTRY",
        "CMAKE_FIND_USE_SYSTEM_PACKAGE_REGISTRY",
        "CMAKE_EXPORT_PACKAGE_REGISTRY",
        "CMAKE_FIND_USE_SYSTEM_ENVIRONMENT_PATH",
        "CMAKE_FIND_USE_CMAKE_ENVIRONMENT_PATH",
        "CMAKE_FIND_USE_CMAKE_SYSTEM_PATH",
        "USE_CCACHE",
    ] {
        setting(text, SettingKey(key), "OFF");
    }
    for key in ["FETCHCONTENT_FULLY_DISCONNECTED", "FETCHCONTENT_UPDATES_DISCONNECTED"] {
        setting(text, SettingKey(key), "ON");
    }
    setting(text, SettingKey("CMAKE_POLICY_DEFAULT_CMP0170"), "NEW");
}
/// The resolved west modules are the only module roots the build sees.
fn set_module_settings(text: &mut String, plan: &Plan) -> Result<()> {
    let modules = plan
        .west
        .modules
        .values()
        .map(|module| plan.root(&module.role))
        .collect::<std::result::Result<Vec<_>, _>>()?;
    setting(text, SettingKey("ZEPHYR_MODULES"), &modules.join(";"));
    Ok(())
}
/// The configuration, overlay and build identity files the build reads.
fn set_file_settings(text: &mut String, plan: &Plan) -> Result<()> {
    for (key, members) in [("CONF_FILE", &plan.kconfig), ("DTC_OVERLAY_FILE", &plan.overlays)] {
        let paths = members
            .iter()
            .map(|item| member(plan, item).map(|path| path.display().to_string()))
            .collect::<Result<Vec<_>>>()?;
        setting(text, SettingKey(key), &paths.join(";"));
    }
    // An empty EXTRA_* value suppresses application/board auto-discovery additions.
    for key in ["EXTRA_CONF_FILE", "EXTRA_DTC_OVERLAY_FILE", "EXTRA_ZEPHYR_MODULES"] {
        setting(text, SettingKey(key), "");
    }
    setting(text, SettingKey("BUILD_VERSION"), crunch_zephyr_core::ZEPHYR_REVISION);
    Ok(())
}
#[derive(Serialize)]
struct Provenance<'a> {
    schema: &'static str,
    plan_identity_blake3: String,
    plan: &'a Plan,
    tools: BTreeMap<String, String>,
    artifacts_blake3: BTreeMap<String, String>,
}
pub fn build(plan: WirePlan, out: &Path) -> Result<()> {
    let admitted = checked(plan)?;
    let plan = &admitted;
    if !store_root(StoreRootRequest {
        value: out.display().to_string(),
        prefix: plan.store_prefix.clone(),
    }) {
        return Err(invalid("zephyr-output-path-invalid"));
    }
    fs::create_dir(WORK)?; // fail closed rather than reuse an ambient build cache
    for dir in ["home/config", "tmp", "cache", "modules"] {
        fs::create_dir_all(Path::new(WORK).join(dir))?;
    }
    let env = environment(plan)?;
    verify_inputs(plan, &env)?;
    let tools = versions(plan, &env)?;
    let source = Path::new(plan.root(&plan.project)?).join(&plan.project_subdir);
    bound(plan, &source.join("CMakeLists.txt"))?;
    fs::write(Path::new(WORK).join("initial.cmake"), configuration(plan)?)?;
    fs::write(Path::new(WORK).join("guard.cmake"), crunch_cmake::OFFLINE_GUARD)?;
    fs::write(
        Path::new(WORK).join("modules/ExternalProject.cmake"),
        "message(FATAL_ERROR \"zephyr-undeclared-download: ExternalProject must be acquired separately\")\n",
    )?;
    let cmake = executable(plan, &plan.tools.cmake)?;
    run(
        command(&cmake, &env).args([
            "-S",
            source.to_str().ok_or_else(|| invalid("zephyr-source-utf8"))?,
            "-B",
            &format!("{WORK}/build"),
            "-G",
            "Ninja",
            "-C",
            &format!("{WORK}/initial.cmake"),
        ]),
        "configure",
    )?;
    run(
        command(&cmake, &env).args([
            "--build",
            &format!("{WORK}/build"),
            "--parallel",
            &plan.jobs.to_string(),
        ]),
        "compile",
    )?;
    let artifacts_blake3 = publish(out)?;
    let provenance = Provenance {
        schema: "mantle-zephyr-provenance-v1",
        plan_identity_blake3: identity_ref(&plan.0.plan)?,
        plan,
        tools,
        artifacts_blake3,
    };
    fs::write(out.join("manifest.json"), serde_json::to_vec_pretty(&provenance)?)?;
    debug_assert!(!provenance.tools.is_empty());
    debug_assert!(!provenance.artifacts_blake3.is_empty());
    Ok(())
}
fn publish(out: &Path) -> Result<BTreeMap<String, String>> {
    fs::create_dir_all(out)?;
    let mut hashes = Vec::with_capacity(ARTIFACTS.len());
    for artifact in ARTIFACTS {
        let source = Path::new(WORK).join("build/zephyr").join(artifact);
        if !fs::symlink_metadata(&source)?.is_file() {
            return Err(invalid(format!("zephyr-artifact-missing: {artifact}")));
        }
        let target = out.join(artifact);
        fs::create_dir_all(target.parent().ok_or_else(|| invalid("zephyr-artifact-parent"))?)?;
        hashes.push((artifact.into(), copy_artifact(&source, &target, artifact == "zephyr.elf")?));
    }
    Ok(hashes.into_iter().collect())
}
fn copy_artifact(source: &Path, target: &Path, elf: bool) -> Result<String> {
    let mut input = fs::File::open(source)?;
    let size_bytes = input.metadata()?.len();
    if size_bytes == 0 || size_bytes > MAX_ARTIFACT_BYTES {
        return Err(invalid("zephyr-artifact-size-invalid"));
    }
    let mut output = fs::File::create(target)?;
    let mut hasher = blake3::Hasher::new();
    let mut copied = 0_u64;
    let mut buffer = [0; CHUNK_BYTES];
    if elf {
        input.read_exact(&mut buffer[..20])?;
        check_elf(&buffer[..20])?;
        output.write_all(&buffer[..20])?;
        hasher.update(&buffer[..20]);
        copied = 20;
    }
    for _ in 0..MAX_ARTIFACT_CHUNKS {
        let n = input.read(&mut buffer)?;
        if n == 0 {
            break;
        }
        copied = copied.saturating_add(n as u64);
        if copied > size_bytes {
            return Err(invalid("zephyr-artifact-size-changed"));
        }
        hasher.update(&buffer[..n]);
        output.write_all(&buffer[..n])?;
    }
    if copied != size_bytes {
        return Err(invalid("zephyr-artifact-size-changed"));
    }
    debug_assert!(copied == size_bytes);
    debug_assert!(size_bytes <= MAX_ARTIFACT_BYTES);
    Ok(hasher.finalize().to_hex().to_string())
}
fn check_elf(header: &[u8]) -> Result<()> {
    let is_long_enough = header.len() >= 20;
    let is_elf_magic = header.get(..4) == Some(b"\x7fELF".as_slice());
    let is_elf_class_32 = header.get(4) == Some(&1);
    let is_little_endian = header.get(5) == Some(&1);
    let is_arm_executable = header.get(16..20) == Some([2, 0, 40, 0].as_slice());
    let is_expected_elf = is_long_enough && is_elf_magic && is_elf_class_32 && is_little_endian && is_arm_executable;
    if !is_expected_elf {
        return Err(invalid("zephyr-artifact-target-mismatch: expected ELF32 little-endian ARM executable"));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn tree_content_tracks_source_and_link_targets_but_not_mtime() {
        let temp = tempfile::tempdir().unwrap();
        fs::write(temp.path().join("main.c"), "int main(void) { return 0; }").unwrap();
        let first = tree_blake3(temp.path()).unwrap();
        fs::write(temp.path().join("main.c"), "int main(void) { return 0; }").unwrap();
        assert_eq!(first, tree_blake3(temp.path()).unwrap());
        fs::write(temp.path().join("main.c"), "int main(void) { return 1; }").unwrap();
        assert_ne!(first, tree_blake3(temp.path()).unwrap());
        std::os::unix::fs::symlink("main.c", temp.path().join("alias")).unwrap();
        let linked = tree_blake3(temp.path()).unwrap();
        fs::remove_file(temp.path().join("alias")).unwrap();
        std::os::unix::fs::symlink("other.c", temp.path().join("alias")).unwrap();
        assert_ne!(linked, tree_blake3(temp.path()).unwrap());
    }
    #[test]
    fn executable_bit_and_empty_directory_are_content() {
        let temp = tempfile::tempdir().unwrap();
        let file = temp.path().join("tool");
        fs::write(&file, "tool").unwrap();
        fs::set_permissions(&file, fs::Permissions::from_mode(0o644)).unwrap();
        let plain = tree_blake3(temp.path()).unwrap();
        fs::set_permissions(&file, fs::Permissions::from_mode(0o755)).unwrap();
        assert_ne!(plain, tree_blake3(temp.path()).unwrap());
        let executable = tree_blake3(temp.path()).unwrap();
        fs::create_dir(temp.path().join("empty")).unwrap();
        assert_ne!(executable, tree_blake3(temp.path()).unwrap());
    }
}
