//! Local Cargo observation, attached only after identity-bearing request construction.
use std::ffi::OsStr;
use std::io;
use std::path::Component;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;

use async_trait::async_trait;
use crunch_rustc_wrapper::waits::Digest;
use crunch_rustc_wrapper::waits::RunOutcome;
use crunch_rustc_wrapper::waits::RuntimeOptions;
use crunch_rustc_wrapper::waits::WaitCollector;
use crunch_rustc_wrapper::waits::WaitContext;
use serde::Deserialize;
use snix_build::buildservice::BuildRequest;
use snix_build::buildservice::BuildResult;
use snix_build::buildservice::BuildService;
use snix_build::buildservice::EnvVar;
use tokio::sync::watch;

pub const GUEST_SOCKET: &str = "/run/mantle/rust-waits.sock";
pub const INVENTORY_ENV: &str = "MANTLE_OFFLINE_CARGO_WAIT_TOOLS";
/// 16 MiB of retained compiler log per observed build.
const MAX_LOG_BYTES: usize = 16_777_216;
/// 64 KiB per structured Cargo fact.
const MAX_FACT_BYTES: usize = 65_536;
const MAX_INVENTORY_BYTES: usize = 16_384;
const MAX_RUNTIME_INPUTS: usize = 64;
const MAX_TRIPLE_BYTES: usize = 256;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct CargoTools {
    wrapper: PathBuf,
    cargo: PathBuf,
    rustc: PathBuf,
    host: String,
    target: String,
    runtime_inputs: Vec<PathBuf>,
}

pub fn validate_runtime(runtime: &RuntimeOptions) -> io::Result<()> {
    if !clean_absolute(&runtime.directory) || runtime.wrapper.as_deref().is_some_and(|wrapper| !clean_absolute(wrapper))
    {
        return Err(io::Error::other("rust-wait-runtime-path-invalid"));
    }
    if std::env::vars_os().any(|(key, _)| {
        key.as_encoded_bytes().starts_with(b"MANTLE_WAIT_")
            || key.as_encoded_bytes().starts_with(b"MANTLE_NATIVE_WAIT_")
            || key == OsStr::new("RUSTC_WRAPPER")
            || key == OsStr::new("RUSTC_WORKSPACE_WRAPPER")
    }) {
        return Err(io::Error::other("rust-wait-inherited-wrapper-binding"));
    }
    Ok(())
}

fn clean_absolute(path: &Path) -> bool {
    path.is_absolute()
        && path != Path::new("/")
        && path.as_os_str().as_encoded_bytes().len() <= 4096
        && !path.as_os_str().as_encoded_bytes().contains(&0)
        && path.components().all(|part| !matches!(part, Component::CurDir | Component::ParentDir))
}

fn declared_tool(request: &BuildRequest, tool: &Path) -> bool {
    if !clean_absolute(tool) {
        return false;
    }
    let Ok(relative) = tool.strip_prefix(Path::new("/").join(&request.inputs_dir)) else {
        return false;
    };
    let mut components = relative.components();
    let Some(Component::Normal(root)) = components.next() else {
        return false;
    };
    components.next().is_some()
        && request.inputs.keys().any(|name| AsRef::<[u8]>::as_ref(name) == root.as_encoded_bytes())
}

fn declared_runtime_root(request: &BuildRequest, root: &Path) -> bool {
    clean_absolute(root)
        && root.parent() == Some(Path::new("/").join(&request.inputs_dir).as_path())
        && root.file_name().is_some_and(|root| {
            request.inputs.keys().any(|name| AsRef::<[u8]>::as_ref(name) == root.as_encoded_bytes())
        })
}

fn admitted_tools(request: &BuildRequest, runtime: &RuntimeOptions) -> io::Result<Option<CargoTools>> {
    if request.environment_vars.iter().any(|entry| {
        entry.key.starts_with("MANTLE_WAIT_")
            || matches!(entry.key.as_str(), "RUSTC_WRAPPER" | "RUSTC_WORKSPACE_WRAPPER")
    }) {
        return Err(io::Error::other("rust-wait-runtime-binding-in-derivation"));
    }
    let mut inventories = request.environment_vars.iter().filter(|entry| entry.key == INVENTORY_ENV);
    let Some(inventory) = inventories.next() else {
        if request.environment_vars.iter().any(|entry| entry.key == "MANTLE_BUILD_EVIDENCE_SCHEMA") {
            return Err(io::Error::other("rust-wait-tool-inventory-required"));
        }
        // Unrelated dependencies execute ordinarily, without compiler observations.
        return Ok(None);
    };
    if inventory.value.len() > MAX_INVENTORY_BYTES || inventories.next().is_some() {
        return Err(io::Error::other("rust-wait-tool-inventory-invalid"));
    }
    if request.outputs.iter().any(|output| self_contained_output(&runtime.directory, output)) {
        return Err(io::Error::other("rust-wait-directory-is-build-output"));
    }
    let tools: CargoTools = serde_json::from_slice(&inventory.value)?;
    if !is_admitted_cargo_inventory(request, runtime, &tools) {
        return Err(io::Error::other("rust-wait-tool-not-admitted"));
    }
    assert!(inventory.value.len() <= MAX_INVENTORY_BYTES);
    assert!(tools.runtime_inputs.len() <= MAX_RUNTIME_INPUTS);
    Ok(Some(tools))
}

/// The selected wrapper, Cargo, compiler and runtime roots must all be exact
/// declared inputs of the shell-built Cargo derivation.
fn is_admitted_cargo_inventory(request: &BuildRequest, runtime: &RuntimeOptions, tools: &CargoTools) -> bool {
    if runtime.wrapper.as_deref() != Some(tools.wrapper.as_path()) {
        return false;
    }
    if ![&tools.wrapper, &tools.cargo, &tools.rustc].iter().all(|path| declared_tool(request, path)) {
        return false;
    }
    if tools.runtime_inputs.len() > MAX_RUNTIME_INPUTS {
        return false;
    }
    if !tools.runtime_inputs.iter().all(|root| declared_runtime_root(request, root)) {
        return false;
    }
    let is_host_bounded = (1..=MAX_TRIPLE_BYTES).contains(&tools.host.len());
    let is_target_bounded = (1..=MAX_TRIPLE_BYTES).contains(&tools.target.len());
    if !(is_host_bounded && is_target_bounded) {
        return false;
    }
    debug_assert!(declared_tool(request, &tools.wrapper));
    debug_assert!(tools.runtime_inputs.len() <= MAX_RUNTIME_INPUTS);
    request.command_args.first().map(String::as_str) == Some("/bin/sh")
}
fn self_contained_output(directory: &Path, output: &Path) -> bool {
    directory.strip_prefix("/").is_ok_and(|relative| relative.starts_with(output))
}

/// The factory mounts a fresh, exclusive collector endpoint per real execution.
/// No endpoint or invocation value is serialized into a derivation or cache key.
pub struct RustWaitBuildService<F> {
    factory: F,
    runtime: RuntimeOptions,
}

impl<F> RustWaitBuildService<F> {
    pub fn new(factory: F, runtime: RuntimeOptions) -> io::Result<Self> {
        validate_runtime(&runtime)?;
        use std::os::unix::fs::PermissionsExt as _;
        let metadata = std::fs::symlink_metadata(&runtime.directory)?;
        if !metadata.is_dir() || metadata.file_type().is_symlink() || metadata.permissions().mode() & 0o077 != 0 {
            return Err(io::Error::other("rust-wait-root-must-be-private-directory"));
        }
        Ok(Self { factory, runtime })
    }
}

struct InputSequence<'a>(&'a std::collections::BTreeMap<snix_castore::PathComponent, snix_castore::Node>);

impl serde::Serialize for InputSequence<'_> {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.collect_seq(self.0.iter().map(|(name, node)| (AsRef::<[u8]>::as_ref(name), node)))
    }
}

fn request_identity(request: &BuildRequest) -> io::Result<Digest> {
    use snix_build::buildservice::BuildConstraints;
    let BuildRequest {
        inputs,
        command_args,
        working_dir,
        scratch_paths,
        inputs_dir,
        outputs,
        environment_vars,
        constraints,
        additional_files,
        refscan_needles,
        workspace,
    } = request;
    debug_assert!(environment_vars.windows(2).all(|pair| pair[0].key < pair[1].key));
    debug_assert!(outputs.windows(2).all(|pair| pair[0] < pair[1]));
    let mut constraints: Vec<_> = constraints.iter().collect();
    constraints.sort_unstable_by_key(|constraint| match *constraint {
        BuildConstraints::System(system) => ("system", system.as_str(), 0, None),
        BuildConstraints::MinMemory(memory_bytes) => ("min-memory", "", *memory_bytes, None),
        BuildConstraints::AvailableReadOnlyPath(path) => ("available-read-only-path", "", 0, Some(path.as_path())),
        BuildConstraints::NetworkAccess => ("network-access", "", 0, None),
        BuildConstraints::ProvideBinSh => ("provide-bin-sh", "", 0, None),
        BuildConstraints::ProvideProcMetadata => ("provide-proc-metadata", "", 0, None),
        BuildConstraints::ProvideRandomDevices => ("provide-random-devices", "", 0, None),
    });
    let mut identity = blake3::Hasher::new();
    identity.update(b"mantle-rust-wait-build-request-v1\0");
    serde_json::to_writer(
        &mut identity,
        &(
            InputSequence(inputs),
            command_args,
            working_dir,
            scratch_paths,
            inputs_dir,
            outputs,
            environment_vars,
            constraints,
            additional_files,
            refscan_needles,
            workspace,
        ),
    )?;
    Digest::parse(identity.finalize().to_hex().to_string())
        .map_err(|error| io::Error::other(format!("rust-wait-request-identity: {error:?}")))
}

fn context(request: &BuildRequest, tools: &CargoTools) -> io::Result<WaitContext> {
    let mut invocation = [0u8; 32];
    use rand::RngCore as _;
    rand::thread_rng().fill_bytes(&mut invocation);
    Ok(WaitContext {
        invocation: Digest::from_slice(&invocation),
        clock: Digest::from_slice(&invocation),
        toolchain: Digest::from_slice(tools.rustc.as_os_str().as_encoded_bytes()),
        configuration: request_identity(request)?,
        host: Digest::from_slice(tools.host.as_bytes()),
        target: Digest::from_slice(tools.target.as_bytes()),
    })
}

fn observe_log(collector: &mut WaitCollector, log: &str) {
    // Cargo's structured completion facts are not live metadata-ready notifications.
    let bytes = log.as_bytes();
    if bytes.len() > MAX_LOG_BYTES {
        collector.observation_lost();
    }
    for line in bytes[..bytes.len().min(MAX_LOG_BYTES)].split(|byte| *byte == b'\n') {
        if line.len() > MAX_FACT_BYTES {
            collector.observation_lost();
        }
        if line.len() <= MAX_FACT_BYTES
            && let Ok(value) = serde_json::from_slice(line)
        {
            collector.observe_cargo(&value);
        }
    }
}

fn configure_cargo(request: &mut BuildRequest, collector: &WaitCollector, tools: &CargoTools) -> io::Result<u64> {
    assert!(tools.rustc.is_absolute(), "the admitted compiler is a declared absolute input");
    let mut environment = Command::new(&tools.cargo);
    collector.configure(&mut environment);
    let origin_ns = environment
        .get_envs()
        .find_map(|(key, value)| {
            (key == OsStr::new("MANTLE_WAIT_ORIGIN_NS")).then(|| value?.to_str()?.parse::<u64>().ok()).flatten()
        })
        .ok_or_else(|| io::Error::other("rust-wait-collector-clock-unavailable"))?;
    for (key, value) in environment.get_envs() {
        if let Some(value) = value {
            request.environment_vars.push(EnvVar {
                key: key.to_string_lossy().into_owned(),
                value: if key == OsStr::new("MANTLE_WAIT_SOCKET") {
                    GUEST_SOCKET.as_bytes().to_vec().into()
                } else {
                    value.as_encoded_bytes().to_vec().into()
                },
            });
        }
    }
    request.environment_vars.push(EnvVar {
        key: "MANTLE_WAIT_ADMITTED_COMPILER".into(),
        value: tools.rustc.as_os_str().as_encoded_bytes().to_vec().into(),
    });
    request.environment_vars.sort_by(|left, right| left.key.cmp(&right.key));
    debug_assert!(request.environment_vars.windows(2).all(|pair| pair[0].key <= pair[1].key));
    Ok(origin_ns)
}

#[async_trait]
impl<F, S> BuildService for RustWaitBuildService<F>
where
    F: Fn(Option<PathBuf>) -> io::Result<S> + Send + Sync + 'static,
    S: BuildService + 'static,
{
    async fn do_build(&self, mut request: BuildRequest) -> io::Result<BuildResult> {
        if request
            .environment_vars
            .iter()
            .any(|entry| entry.key.starts_with("MANTLE_WAIT_") || entry.key.starts_with("MANTLE_NATIVE_WAIT_"))
        {
            return Err(io::Error::other("rust-wait-runtime-binding-in-derivation"));
        }
        if let Some(tools) = native_tools(&request, &self.runtime)? {
            return self.do_native(request, tools).await;
        }
        let Some(tools) = admitted_tools(&request, &self.runtime)? else {
            return (self.factory)(None)?.do_build(request).await;
        };
        let context = context(&request, &tools)?;
        let setup = (|| {
            let root = tempfile::Builder::new().prefix("cargo-waits-").tempdir_in(&self.runtime.directory)?;
            let collector = WaitCollector::start(root.path(), &tools.wrapper, context)?;
            Ok::<_, io::Error>((root, collector))
        })();
        let (root, mut collector) = match setup {
            Ok(setup) => setup,
            Err(error) => {
                tracing::warn!(%error, "Rust wait diagnostics unavailable; ordinary Cargo execution continues");
                return (self.factory)(None)?.do_build(request).await;
            }
        };
        let service = match (self.factory)(Some(root.path().join("wait.sock"))) {
            Ok(service) => service,
            Err(error) => {
                tracing::warn!(%error, "Rust wait diagnostic transport unavailable; ordinary execution continues");
                return (self.factory)(None)?.do_build(request).await;
            }
        };
        let origin_ns = configure_cargo(&mut request, &collector, &tools)?;
        let result = service.do_build(request).await;
        let outcome = match &result {
            Ok(result) => {
                if let Some(log) = &result.log {
                    observe_log(&mut collector, log);
                }
                RunOutcome::Success
            }
            Err(error) => {
                observe_log(&mut collector, &error.to_string());
                RunOutcome::Infrastructure
            }
        };
        let elapsed = collector.offset_ns();
        // The sandbox API exposes no descendant-reap timestamp or resource samples.
        // Keep those unavailable rather than substituting outer completion.
        let diagnostic = collector.finish(outcome, origin_ns, elapsed, None);
        let retained = root.keep();
        if diagnostic.publication_error.is_some() {
            tracing::warn!(directory = %retained.display(), "Rust wait diagnostic publication failed");
        }
        result
    }

    async fn do_build_cancellable(
        &self,
        _request: BuildRequest,
        _cancellation: watch::Receiver<bool>,
    ) -> io::Result<BuildResult> {
        Err(io::Error::other("rust-wait-diagnostics-unsupported-for-watch"))
    }
}

pub enum RustWaitExecutionService<W, D> {
    Observed(W),
    Ordinary(D),
}

#[async_trait]
impl<W: BuildService + 'static, D: BuildService + 'static> BuildService for RustWaitExecutionService<W, D> {
    async fn do_build(&self, request: BuildRequest) -> io::Result<BuildResult> {
        match self {
            Self::Observed(service) => service.do_build(request).await,
            Self::Ordinary(service) => service.do_build(request).await,
        }
    }

    async fn do_build_cancellable(
        &self,
        request: BuildRequest,
        cancellation: watch::Receiver<bool>,
    ) -> io::Result<BuildResult> {
        match self {
            Self::Observed(service) => service.do_build_cancellable(request, cancellation).await,
            Self::Ordinary(service) => service.do_build_cancellable(request, cancellation).await,
        }
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct NativeTools {
    helper: PathBuf,
    rustc: PathBuf,
    unit: Option<Digest>,
}

struct NativeAdmission {
    tools: Option<NativeTools>,
    units: Option<Vec<crunch_rustc_wrapper::waits::UnitIdentity>>,
}

fn native_tools(request: &BuildRequest, runtime: &RuntimeOptions) -> io::Result<Option<NativeAdmission>> {
    let mut entries = request.environment_vars.iter().filter(|entry| entry.key == "MANTLE_UNIT_WAIT_TOOLS");
    let Some(entry) = entries.next() else {
        return Ok(None);
    };
    if runtime.wrapper.is_some() {
        return Err(io::Error::other("native-rust-wait-wrapper-unsupported"));
    }
    if entry.value.len() > MAX_INVENTORY_BYTES || entries.next().is_some() {
        return Ok(Some(NativeAdmission {
            tools: None,
            units: None,
        }));
    }
    let tools: NativeTools = match serde_json::from_slice(&entry.value) {
        Ok(tools) => tools,
        Err(_) => {
            return Ok(Some(NativeAdmission {
                tools: None,
                units: None,
            }));
        }
    };
    let compiler = request.environment_vars.iter().find(|entry| entry.key == "MANTLE_UNIT_RUSTC");
    if !is_admitted_native_inventory(request, runtime, &tools, compiler) {
        return Err(io::Error::other("native-rust-wait-tool-not-admitted"));
    }
    assert!(runtime.wrapper.is_none(), "native compilers run without a Cargo wrapper");
    assert!(entry.value.len() <= MAX_INVENTORY_BYTES);
    let units = match native_unit_snapshot(runtime, tools.unit.as_ref()) {
        Ok(units) => Some(units),
        Err(error) => {
            tracing::warn!(%error, "Native Rust wait attribution unavailable; compiler execution remains ordinary");
            None
        }
    };
    Ok(Some(NativeAdmission {
        tools: Some(tools),
        units,
    }))
}

/// The per-unit helper and its exact compiler must be declared inputs, the
/// helper must be the derivation builder, and no output may hold the collector.
fn is_admitted_native_inventory(
    request: &BuildRequest,
    runtime: &RuntimeOptions,
    tools: &NativeTools,
    compiler: Option<&EnvVar>,
) -> bool {
    if !declared_tool(request, &tools.helper) || !declared_tool(request, &tools.rustc) {
        return false;
    }
    if request.command_args.first().map(Path::new) != Some(tools.helper.as_path()) {
        return false;
    }
    if compiler.is_none_or(|entry| entry.value.as_ref() != tools.rustc.as_os_str().as_encoded_bytes()) {
        return false;
    }
    debug_assert!(tools.helper.is_absolute());
    debug_assert!(tools.rustc.is_absolute());
    !request.outputs.iter().any(|output| self_contained_output(&runtime.directory, output))
}

fn native_unit_snapshot(
    runtime: &RuntimeOptions,
    selected: Option<&Digest>,
) -> io::Result<Vec<crunch_rustc_wrapper::waits::UnitIdentity>> {
    let selected = selected.ok_or_else(|| io::Error::other("native-rust-wait-unit-unavailable"))?;
    let registry = runtime
        .native_units
        .as_ref()
        .ok_or_else(|| io::Error::other("native-rust-wait-unit-table-required"))?
        .lock()
        .map_err(|_| io::Error::other("native-rust-wait-unit-table-unavailable"))?;
    if registry.len() > crunch_rustc_wrapper::waits::MAX_UNITS || !registry.contains_key(selected) {
        return Err(io::Error::other("native-rust-wait-unit-not-admitted"));
    }
    for (id, unit) in registry.iter() {
        if unit.identity().map_err(io::Error::other)? != *id
            || unit.dependencies.iter().any(|dependency| !registry.contains_key(&dependency.unit))
        {
            return Err(io::Error::other("native-rust-wait-identity-closure-invalid"));
        }
    }
    assert!(registry.len() <= crunch_rustc_wrapper::waits::MAX_UNITS);
    assert!(registry.contains_key(selected));
    Ok(registry.values().cloned().collect())
}

fn register_native(
    collector: &mut crunch_rustc_wrapper::waits::NativeWaitCollector,
    request: &BuildRequest,
    tools: NativeTools,
    units: Vec<crunch_rustc_wrapper::waits::UnitIdentity>,
) -> io::Result<Digest> {
    use crunch_rustc_wrapper::waits::Attempt;
    let selected = tools.unit.ok_or_else(|| io::Error::other("native-rust-wait-unit-unavailable"))?;
    let unit_count = units.len();
    assert!(unit_count <= crunch_rustc_wrapper::waits::MAX_UNITS);
    let mut registered_count: usize = 0;
    let mut selected_role = None;
    for facts in units {
        let role = facts.role.clone();
        let id = collector.register_unit(facts).map_err(io::Error::other)?;
        registered_count = registered_count.saturating_add(1);
        if id == selected {
            selected_role = Some(role);
        }
    }
    assert_eq!(registered_count, unit_count, "every admitted unit was registered");
    let role = selected_role.ok_or_else(|| io::Error::other("native-rust-wait-unit-not-admitted"))?;
    collector
        .register_attempt(Attempt {
            id: Digest::from_slice(b"unassigned"),
            invocation: collector.invocation().clone(),
            clock: collector.clock().clone(),
            unit: Some(selected),
            role,
            number: 1,
            resolved_process: request_identity(request)?,
            prior: None,
            unmapped: None,
        })
        .map_err(io::Error::other)
}

impl<F, S> RustWaitBuildService<F>
where
    F: Fn(Option<PathBuf>) -> io::Result<S> + Send + Sync + 'static,
    S: BuildService + 'static,
{
    async fn do_native(&self, mut request: BuildRequest, admitted: NativeAdmission) -> io::Result<BuildResult> {
        use crunch_rustc_wrapper::waits::NativeWaitCollector;
        let NativeAdmission { tools, units } = admitted;
        let setup = (|| {
            let root = tempfile::Builder::new().prefix("native-waits-").tempdir_in(&self.runtime.directory)?;
            let mut invocation = [0u8; 32];
            use rand::RngCore as _;
            rand::thread_rng().fill_bytes(&mut invocation);
            let invocation = Digest::from_slice(&invocation);
            let mut collector = NativeWaitCollector::start(root.path(), invocation.clone(), invocation)?;
            let attempt = units
                .zip(tools)
                .and_then(|(units, tools)| register_native(&mut collector, &request, tools, units).ok());
            if attempt.is_none() {
                collector.observation_lost();
            }
            Ok::<_, io::Error>((root, collector, attempt))
        })();
        let (root, mut collector, attempt) = match setup {
            Ok(setup) => setup,
            Err(error) => {
                tracing::warn!(%error, "Native Rust wait diagnostics unavailable; ordinary execution continues");
                return (self.factory)(None)?.do_build(request).await;
            }
        };
        let endpoint = attempt.as_ref().map(|_| collector.socket_path().to_path_buf());
        let service = match (self.factory)(endpoint) {
            Ok(service) => service,
            Err(error) => {
                tracing::warn!(%error, "Native Rust wait transport unavailable; ordinary execution continues");
                return (self.factory)(None)?.do_build(request).await;
            }
        };
        if let Some(context) = attempt.as_ref().and_then(|attempt| collector.native_context(attempt)) {
            request.environment_vars.push(EnvVar {
                key: "MANTLE_NATIVE_WAIT_SOCKET".into(),
                value: GUEST_SOCKET.as_bytes().to_vec().into(),
            });
            request.environment_vars.push(EnvVar {
                key: "MANTLE_NATIVE_WAIT_CONTEXT".into(),
                value: serde_json::to_vec(&context)?.into(),
            });
            request.environment_vars.sort_by(|left, right| left.key.cmp(&right.key));
        }
        let result = service.do_build(request).await;
        collector.drain_native_frames();
        let diagnostic = collector.finish(if result.is_ok() {
            RunOutcome::Success
        } else {
            RunOutcome::Infrastructure
        });
        let retained = root.keep();
        if diagnostic.publication_error.is_some() {
            tracing::warn!(directory = %retained.display(), "Native Rust wait diagnostic publication failed");
        }
        result
    }
}

/// Admit diagnostic metadata only after the generic dynamic plan was validated.
pub(crate) fn admit_native_provenance(
    registry: &std::sync::Mutex<std::collections::BTreeMap<Digest, crunch_rustc_wrapper::waits::UnitIdentity>>,
    provenance: &std::collections::BTreeMap<String, String>,
) -> io::Result<()> {
    const PREFIX: &str = "mantle-rust-wait-units-v1.";
    // 4 MiB across all numbered provenance chunks.
    const MAX_TABLE_BYTES: usize = 4_194_304;
    let total_bytes = provenance
        .iter()
        .filter(|(key, _)| key.starts_with(PREFIX))
        .try_fold(0usize, |total, (_, chunk)| total.checked_add(chunk.len()))
        .filter(|total| *total <= MAX_TABLE_BYTES)
        .ok_or_else(|| io::Error::other("native-rust-wait-provenance-overflow"))?;
    assert!(total_bytes <= MAX_TABLE_BYTES);
    let mut bytes = Vec::with_capacity(total_bytes);
    for (index, (key, chunk)) in provenance.iter().filter(|(key, _)| key.starts_with(PREFIX)).enumerate() {
        if key != &format!("{PREFIX}{index:04}")
            || chunk.len() > 16_384
            || bytes.len().saturating_add(chunk.len()) > MAX_TABLE_BYTES
        {
            return Err(io::Error::other("native-rust-wait-provenance-invalid"));
        }
        bytes.extend_from_slice(chunk.as_bytes());
    }
    assert_eq!(bytes.len(), total_bytes, "every numbered chunk was retained");
    if bytes.is_empty() {
        return Ok(());
    }
    let units: Vec<crunch_rustc_wrapper::waits::UnitIdentity> = serde_json::from_slice(&bytes)?;
    if units.len() > crunch_rustc_wrapper::waits::MAX_UNITS {
        return Err(io::Error::other("native-rust-wait-unit-table-overflow"));
    }
    let count = units.len();
    let admitted = units
        .into_iter()
        .map(|facts| facts.identity().map(|id| (id, facts)).map_err(io::Error::other))
        .collect::<io::Result<std::collections::BTreeMap<_, _>>>()?;
    if admitted.len() != count
        || admitted.iter().any(|(id, facts)| {
            facts
                .dependencies
                .iter()
                .any(|dependency| dependency.unit == *id || !admitted.contains_key(&dependency.unit))
        })
    {
        return Err(io::Error::other("native-rust-wait-identity-closure-invalid"));
    }
    let mut registry = registry.lock().map_err(|_| io::Error::other("native-rust-wait-unit-table-unavailable"))?;
    let new_count = admitted.keys().filter(|id| !registry.contains_key(*id)).count();
    if registry.len().saturating_add(new_count) > crunch_rustc_wrapper::waits::MAX_UNITS {
        return Err(io::Error::other("native-rust-wait-unit-table-overflow"));
    }
    registry.extend(admitted);
    Ok(())
}

#[cfg(test)]
mod tests {
    use snix_build::buildservice::BuildConstraints;
    use snix_castore::Node;

    use super::*;

    fn tools() -> CargoTools {
        CargoTools {
            wrapper: "/tools/wrapper".into(),
            cargo: "/tools/cargo".into(),
            rustc: "/tools/rustc".into(),
            host: "x86_64-linux".into(),
            target: "x86_64-linux".into(),
            runtime_inputs: Vec::new(),
        }
    }

    #[test]
    fn cargo_configuration_binds_binary_input_names_and_content() {
        let mut request = BuildRequest::default();
        request.inputs.insert(b"source\x80".as_slice().try_into().unwrap(), Node::File {
            digest: (&[1u8; 32]).into(),
            size: 17,
            executable: false,
        });
        let original = context(&request, &tools()).unwrap().configuration;
        assert_eq!(original, context(&request, &tools()).unwrap().configuration);
        let (_, node) = request.inputs.pop_first().unwrap();
        request.inputs.insert(b"source\x81".as_slice().try_into().unwrap(), node);
        let renamed = context(&request, &tools()).unwrap().configuration;
        assert_ne!(original, renamed);
        *request.inputs.values_mut().next().unwrap() = Node::File {
            digest: (&[2u8; 32]).into(),
            size: 17,
            executable: false,
        };
        assert_ne!(renamed, context(&request, &tools()).unwrap().configuration);
    }

    #[test]
    fn cargo_configuration_canonicalizes_constraints_without_omitting_them() {
        let constraints = [
            BuildConstraints::System("x86_64-linux".into()),
            BuildConstraints::MinMemory(4096),
            BuildConstraints::ProvideBinSh,
            BuildConstraints::AvailableReadOnlyPath("/dev/kvm".into()),
        ];
        let request = BuildRequest {
            constraints: constraints.iter().cloned().collect(),
            ..BuildRequest::default()
        };
        let mut reordered = BuildRequest {
            constraints: constraints.iter().rev().cloned().collect(),
            ..BuildRequest::default()
        };
        let original = context(&request, &tools()).unwrap().configuration;
        assert_eq!(original, context(&reordered, &tools()).unwrap().configuration);
        reordered.constraints.remove(&BuildConstraints::MinMemory(4096));
        reordered.constraints.insert(BuildConstraints::MinMemory(8192));
        assert_ne!(original, context(&reordered, &tools()).unwrap().configuration);
    }
}
