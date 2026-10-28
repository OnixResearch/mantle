//! Opt-in runtime bindings, downstream of all derivation/action identity.
//! Drivers and policies remain declared inputs. Optional protected execution
//! checks their materialized bytes without adding cache state to those inputs.
use std::io;
use std::path::Component;
use std::path::Path;
use std::path::PathBuf;

use async_trait::async_trait;
use serde::Deserialize;
use serde::Serialize;
use snix_build::buildservice::BuildRequest;
use snix_build::buildservice::BuildResult;
use snix_build::buildservice::BuildService;
use snix_build::buildservice::EnvVar;
use tokio::sync::watch;

pub const GUEST_SOCKET: &str = "/run/mantle/compile-cache.sock";
pub const SOCKET_ENV: &str = "MANTLE_CC_CACHE_SOCKET";
pub const BUILD_ID_ENV: &str = "MANTLE_COMPILE_BUILD_ID";

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CompileDriverRuntime {
    pub schema: String,
    pub host_socket: PathBuf,
    pub build_id: String,
    /// Exact admitted builder paths; no ambient PATH or arbitrary goal opt-in.
    pub admitted_builders: Vec<String>,
    /// Execution-only observation file, outside every derivation output.
    pub receipt_path: PathBuf,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub protected_exec: Option<ProtectedBuildRuntime>,
}

/// Host-side execution supervision; never mounted as a derivation input.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProtectedBuildRuntime {
    pub runner: PathBuf,
    pub inventory: PathBuf,
    pub audit_dir: PathBuf,
}

impl ProtectedBuildRuntime {
    pub fn validate(&self) -> io::Result<()> {
        for path in [&self.runner, &self.inventory, &self.audit_dir] {
            if !clean_absolute(path) {
                return Err(io::Error::other("compile-driver-protected-path-invalid"));
            }
            if path.as_os_str().as_encoded_bytes().len() > 4096 {
                return Err(io::Error::other("compile-driver-protected-path-too-long"));
            }
        }
        if self.runner == self.inventory {
            return Err(io::Error::other("compile-driver-protected-paths-overlap"));
        }
        if self.audit_dir == self.runner {
            return Err(io::Error::other("compile-driver-protected-paths-overlap"));
        }
        if self.audit_dir == self.inventory {
            return Err(io::Error::other("compile-driver-protected-paths-overlap"));
        }
        assert!(self.runner.is_absolute());
        assert_ne!(self.audit_dir, self.inventory);
        Ok(())
    }

    /// Keep execution observations outside the physical store export tree.
    pub fn validate_output_directory(&self, output_directory: &Path) -> io::Result<()> {
        self.validate()?;
        let absolute;
        let output_directory = if output_directory.is_absolute() {
            output_directory
        } else {
            absolute = std::path::absolute(output_directory)?;
            &absolute
        };
        if self.audit_dir.starts_with(output_directory) {
            return Err(io::Error::other("compile-driver-audit-is-build-output"));
        }
        assert!(output_directory.is_absolute());
        assert!(self.audit_dir.is_absolute());
        Ok(())
    }
}

#[cfg(target_os = "linux")]
impl snix_build::bwrap::BwrapCommandFactory for ProtectedBuildRuntime {
    fn command(&self, invocation: snix_build::bwrap::BwrapInvocation<'_>) -> io::Result<tokio::process::Command> {
        self.validate()?;
        let store_prefix = Path::new("/").join(invocation.inputs_dir);
        if !clean_absolute(&store_prefix) {
            return Err(io::Error::other("compile-driver-protected-input-prefix-invalid"));
        }
        let name =
            invocation.host_workdir.file_name().ok_or_else(|| io::Error::other("compile-driver-sandbox-name"))?;
        let audit_path = self.audit_dir.join(name);
        std::fs::create_dir_all(&self.audit_dir)?;
        let mut command = tokio::process::Command::new(&self.runner);
        command
            .arg("--inventory")
            .arg(&self.inventory)
            .arg("--audit")
            .arg(audit_path)
            .arg("--materialized-input-root")
            .arg(invocation.host_inputs_dir)
            .arg("--store-prefix")
            .arg(store_prefix)
            .arg("--")
            .arg(invocation.program)
            .args(invocation.args);
        assert!(self.runner.is_absolute());
        assert!(self.audit_dir.is_absolute());
        Ok(command)
    }
}

impl CompileDriverRuntime {
    /// Read one bounded, explicitly selected execution policy. Never discover it
    /// through the environment or derivation inputs.
    pub fn read(path: &Path) -> io::Result<Self> {
        use std::io::Read as _;
        let mut bytes = Vec::new();
        std::fs::File::open(path)?.take(65_537).read_to_end(&mut bytes)?;
        if bytes.len() > 65_536 {
            return Err(io::Error::other("compile-driver-runtime-too-large"));
        }
        let runtime: Self = serde_json::from_slice(&bytes)?;
        runtime.validate()?;
        Ok(runtime)
    }

    pub fn validate(&self) -> io::Result<()> {
        if let Some(protected) = &self.protected_exec {
            protected.validate()?;
        }
        let is_known_schema = self.schema == "mantle-compile-driver-runtime-v1";
        let has_absolute_host_socket = clean_absolute(&self.host_socket);
        let has_absolute_receipt_path = clean_absolute(&self.receipt_path);
        let has_bounded_host_socket = self.host_socket.as_os_str().as_encoded_bytes().len() <= 107;
        let has_bounded_receipt_path = self.receipt_path.as_os_str().as_encoded_bytes().len() <= 4096;
        let has_distinct_paths = self.host_socket != self.receipt_path;
        let has_usable_build_id =
            !self.build_id.is_empty() && self.build_id.len() <= 256 && !self.build_id.chars().any(char::is_control);
        let has_admitted_builders = !self.admitted_builders.is_empty() && self.admitted_builders.len() <= 256;
        let has_valid_builders = self.admitted_builders.iter().all(|builder| {
            clean_absolute(Path::new(builder)) && builder.len() <= 4096 && !builder.chars().any(char::is_control)
        });
        let has_unique_builders = self
            .admitted_builders
            .iter()
            .enumerate()
            .all(|(index, builder)| !self.admitted_builders[..index].contains(builder));
        let is_valid = is_known_schema
            && has_absolute_host_socket
            && has_absolute_receipt_path
            && has_bounded_host_socket
            && has_bounded_receipt_path
            && has_distinct_paths
            && has_usable_build_id
            && has_admitted_builders
            && has_valid_builders
            && has_unique_builders;
        if !is_valid {
            return Err(io::Error::other("compile-driver-runtime-invalid"));
        }
        Ok(())
    }

    #[cfg(target_os = "linux")]
    pub fn command_factory(&self) -> Option<std::sync::Arc<dyn snix_build::bwrap::BwrapCommandFactory>> {
        self.protected_exec
            .clone()
            .map(|runtime| std::sync::Arc::new(runtime) as std::sync::Arc<dyn snix_build::bwrap::BwrapCommandFactory>)
    }
}

fn clean_absolute(path: &Path) -> bool {
    path.is_absolute()
        && path.components().all(|c| !matches!(c, Component::CurDir | Component::ParentDir))
        && !path.as_os_str().as_encoded_bytes().contains(&0)
        && path != Path::new("/")
}

pub fn is_runtime_environment_key(key: &str) -> bool {
    key.starts_with("MANTLE_CC_CACHE_") || key.starts_with("MANTLE_COMPILE_")
}

/// Reject ambient authority instead of silently treating it as explicit policy.
pub fn reject_inherited_runtime_environment() -> io::Result<()> {
    if std::env::vars_os().any(|(key, _)| {
        let key = key.as_encoded_bytes();
        key.starts_with(b"MANTLE_CC_CACHE_") || key.starts_with(b"MANTLE_COMPILE_")
    }) {
        return Err(io::Error::other("compile-driver-inherited-runtime-binding"));
    }
    Ok(())
}

pub struct CompileDriverBuildService<S> {
    inner: S,
    runtime: Option<CompileDriverRuntime>,
}
impl<S> CompileDriverBuildService<S> {
    /// Parent constructs the local socket mapping and optional command factory
    /// before this wrapper. Strict proofs must pass `None` and use neither.
    pub fn new(inner: S, runtime: Option<CompileDriverRuntime>) -> io::Result<Self> {
        if let Some(runtime) = &runtime {
            runtime.validate()?;
        }
        Ok(Self { inner, runtime })
    }

    fn prepare(&self, mut request: BuildRequest) -> io::Result<BuildRequest> {
        let Some(runtime) = &self.runtime else {
            return Ok(request);
        };
        if !request.command_args.first().is_some_and(|b| runtime.admitted_builders.contains(b)) {
            return Err(io::Error::other("compile-driver-builder-not-admitted"));
        }
        if request.environment_vars.iter().any(|e| is_runtime_environment_key(&e.key)) {
            return Err(io::Error::other("compile-driver-runtime-binding-in-derivation"));
        }
        let receipt_relative = runtime.receipt_path.strip_prefix("/").map_err(io::Error::other)?;
        if request.outputs.iter().any(|o| receipt_relative.starts_with(o)) {
            return Err(io::Error::other("compile-driver-receipt-is-build-output"));
        }
        if let Some(protected) = &runtime.protected_exec {
            let audit_relative = protected.audit_dir.strip_prefix("/").map_err(io::Error::other)?;
            if request.outputs.iter().any(|output| audit_relative.starts_with(output)) {
                return Err(io::Error::other("compile-driver-audit-is-build-output"));
            }
        }
        request.environment_vars.push(EnvVar {
            key: SOCKET_ENV.into(),
            value: GUEST_SOCKET.as_bytes().to_vec().into(),
        });
        request.environment_vars.push(EnvVar {
            key: BUILD_ID_ENV.into(),
            value: runtime.build_id.as_bytes().to_vec().into(),
        });
        request.environment_vars.sort_by(|a, b| a.key.cmp(&b.key));
        let parent = runtime.receipt_path.parent().ok_or_else(|| io::Error::other("compile-runtime-receipt-parent"))?;
        std::fs::create_dir_all(parent)?;
        let mut file = std::fs::OpenOptions::new().append(true).create(true).open(&runtime.receipt_path)?;
        serde_json::to_writer(
            &mut file,
            &serde_json::json!({"schema":"mantle-compile-driver-mapping-v1",
            "host_socket":runtime.host_socket,"guest_socket":GUEST_SOCKET,"build_id":runtime.build_id,
            "claim_class":"execution-only-performance-state","proof_evidence":false}),
        )?;
        std::io::Write::write_all(&mut file, b"\n")?;
        Ok(request)
    }
}

#[async_trait]
impl<S: BuildService + 'static> BuildService for CompileDriverBuildService<S> {
    async fn do_build(&self, request: BuildRequest) -> io::Result<BuildResult> {
        self.inner.do_build(self.prepare(request)?).await
    }
    async fn do_build_cancellable(
        &self,
        request: BuildRequest,
        cancellation: watch::Receiver<bool>,
    ) -> io::Result<BuildResult> {
        self.inner.do_build_cancellable(self.prepare(request)?, cancellation).await
    }
}

/// Keep the ordinary executor exactly as before unless runtime policy opts in.
/// The enabled branch is local-only; no runtime endpoint crosses remote dispatch.
pub enum CompileDriverExecutionService<L, D> {
    Enabled(CompileDriverBuildService<L>),
    Default(D),
}

#[async_trait]
impl<L: BuildService + 'static, D: BuildService + 'static> BuildService for CompileDriverExecutionService<L, D> {
    async fn do_build(&self, request: BuildRequest) -> io::Result<BuildResult> {
        match self {
            Self::Enabled(service) => service.do_build(request).await,
            Self::Default(service) => service.do_build(request).await,
        }
    }

    async fn do_build_cancellable(
        &self,
        request: BuildRequest,
        cancellation: watch::Receiver<bool>,
    ) -> io::Result<BuildResult> {
        match self {
            Self::Enabled(service) => service.do_build_cancellable(request, cancellation).await,
            Self::Default(service) => service.do_build_cancellable(request, cancellation).await,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn runtime(root: &Path) -> CompileDriverRuntime {
        CompileDriverRuntime {
            schema: "mantle-compile-driver-runtime-v1".into(),
            host_socket: root.join("daemon.sock"),
            build_id: "build-1".into(),
            admitted_builders: vec!["/declared/bin/builder".into()],
            receipt_path: root.join("mapping.ndjson"),
            protected_exec: None,
        }
    }

    fn request() -> BuildRequest {
        BuildRequest {
            command_args: vec!["/declared/bin/builder".into()],
            outputs: vec!["store/output".into()],
            ..BuildRequest::default()
        }
    }

    #[test]
    fn declared_runtime_cannot_be_overridden_by_derivation_cache_or_slot_keys() {
        let root = tempfile::tempdir().unwrap();
        let runtime = runtime(root.path());
        let receipt = runtime.receipt_path.clone();
        let service = CompileDriverBuildService::new((), Some(runtime)).unwrap();
        for key in [
            SOCKET_ENV,
            BUILD_ID_ENV,
            "MANTLE_CC_CACHE_POLICY",
            "MANTLE_COMPILE_SLOT_SOCKET",
            "MANTLE_COMPILE_FUTURE",
        ] {
            let mut request = request();
            request.environment_vars.push(EnvVar {
                key: key.into(),
                value: b"forged".to_vec().into(),
            });
            assert_eq!(
                service.prepare(request).unwrap_err().to_string(),
                "compile-driver-runtime-binding-in-derivation"
            );
            assert!(!receipt.exists(), "rejected authority must not produce an execution mapping");
        }
    }

    #[test]
    fn exact_builder_admission_rejects_similar_path_and_receipt_output_overlap() {
        let root = tempfile::tempdir().unwrap();
        let runtime = runtime(root.path());
        let receipt = runtime.receipt_path.clone();
        let service = CompileDriverBuildService::new((), Some(runtime)).unwrap();
        let mut unadmitted = request();
        unadmitted.command_args[0].push_str("-other");
        assert_eq!(service.prepare(unadmitted).unwrap_err().to_string(), "compile-driver-builder-not-admitted");
        let mut overlap = request();
        overlap.outputs = vec![receipt.parent().unwrap().strip_prefix("/").unwrap().to_path_buf()];
        assert_eq!(service.prepare(overlap).unwrap_err().to_string(), "compile-driver-receipt-is-build-output");
        assert!(!receipt.exists());
    }

    #[test]
    fn runtime_loader_rejects_oversize_and_unknown_authority_fields() {
        let root = tempfile::tempdir().unwrap();
        let path = root.path().join("runtime.json");
        std::fs::write(&path, vec![b' '; 65_537]).unwrap();
        assert_eq!(CompileDriverRuntime::read(&path).unwrap_err().to_string(), "compile-driver-runtime-too-large");
        let mut value = serde_json::to_value(runtime(root.path())).unwrap();
        value["guest_socket"] = serde_json::json!("/arbitrary/guest.sock");
        std::fs::write(&path, serde_json::to_vec(&value).unwrap()).unwrap();
        assert!(CompileDriverRuntime::read(&path).unwrap_err().to_string().contains("unknown field"));
    }

    #[test]
    fn runtime_paths_and_inventory_are_bounded_and_unambiguous() {
        let root = tempfile::tempdir().unwrap();
        let mut policy = runtime(root.path());
        policy.admitted_builders.push(policy.admitted_builders[0].clone());
        assert!(policy.validate().is_err());
        policy.admitted_builders = vec!["/declared/../other/builder".into()];
        assert!(policy.validate().is_err());
        policy.admitted_builders = vec!["builder-from-path".into()];
        assert!(policy.validate().is_err());
        policy.admitted_builders = vec!["/declared/bin/builder".into()];
        policy.host_socket = policy.receipt_path.clone();
        assert!(policy.validate().is_err());
    }

    #[test]
    fn protected_paths_reject_relative_traversal_and_ambiguous_files() {
        let root = tempfile::tempdir().unwrap();
        let mut runtime = ProtectedBuildRuntime {
            runner: root.path().join("runner"),
            inventory: root.path().join("inventory.json"),
            audit_dir: root.path().join("audits"),
        };
        assert!(runtime.validate().is_ok());
        runtime.runner = PathBuf::from("relative-runner");
        assert_eq!(runtime.validate().unwrap_err().to_string(), "compile-driver-protected-path-invalid");
        runtime.runner = root.path().join("runner");
        runtime.inventory = root.path().join("../inventory.json");
        assert_eq!(runtime.validate().unwrap_err().to_string(), "compile-driver-protected-path-invalid");
        runtime.inventory = runtime.runner.clone();
        assert_eq!(runtime.validate().unwrap_err().to_string(), "compile-driver-protected-paths-overlap");
        runtime.inventory = root.path().join("inventory.json");
        runtime.audit_dir = runtime.inventory.clone();
        assert_eq!(runtime.validate().unwrap_err().to_string(), "compile-driver-protected-paths-overlap");
    }

    #[test]
    fn physical_output_audit_overlap_is_refused_without_rejecting_siblings() {
        let root = tempfile::tempdir().unwrap();
        let output = root.path().join("store");
        let mut runtime = ProtectedBuildRuntime {
            runner: root.path().join("runner"),
            inventory: root.path().join("inventory.json"),
            audit_dir: root.path().join("store-observations"),
        };
        assert!(runtime.validate_output_directory(&output).is_ok());
        runtime.audit_dir = output.join("selected-output/audits");
        assert_eq!(
            runtime.validate_output_directory(&output).unwrap_err().to_string(),
            "compile-driver-audit-is-build-output"
        );
        assert!(!output.exists());
    }

    #[test]
    fn output_audit_overlap_is_refused_before_creating_runtime_files() {
        let root = tempfile::tempdir().unwrap();
        let audit_dir = root.path().join("audits");
        let receipt_path = root.path().join("mapping.ndjson");
        let runtime = CompileDriverRuntime {
            schema: "mantle-compile-driver-runtime-v1".into(),
            host_socket: root.path().join("daemon.sock"),
            build_id: "protected-build".into(),
            admitted_builders: vec!["/declared/bin/builder".into()],
            receipt_path: receipt_path.clone(),
            protected_exec: Some(ProtectedBuildRuntime {
                runner: root.path().join("runner"),
                inventory: root.path().join("inventory.json"),
                audit_dir: audit_dir.clone(),
            }),
        };
        let service = CompileDriverBuildService::new((), Some(runtime)).unwrap();
        let request = BuildRequest {
            command_args: vec!["/declared/bin/builder".into()],
            outputs: vec![audit_dir.strip_prefix("/").unwrap().to_path_buf()],
            ..BuildRequest::default()
        };
        assert_eq!(service.prepare(request).unwrap_err().to_string(), "compile-driver-audit-is-build-output");
        assert!(!receipt_path.exists());
        assert!(!audit_dir.exists());
    }
}
