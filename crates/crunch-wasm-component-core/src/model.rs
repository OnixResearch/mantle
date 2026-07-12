use alloc::string::String;
use alloc::vec::Vec;

use serde::Deserialize;
use serde::Serialize;

use crate::Blake3Identity;
use crate::OciSha256Digest;

pub const COMPONENT_MANIFEST_SCHEMA: &str = "mantle-wasm-component-manifest-v1";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StoreObject {
    pub logical_path: String,
    pub digest_blake3: Blake3Identity,
    pub size_bytes: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum OciProtocol {
    Http,
    Https,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum PackageKind {
    Wit,
    ComponentLibrary,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RegistryMapping {
    pub namespace: String,
    pub registry: String,
    pub oci_registry: String,
    pub namespace_prefix: String,
    pub protocol: OciProtocol,
    pub credential_handle: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PackageRequirement {
    pub package: String,
    pub requirement: String,
    pub registry: String,
    pub kind: PackageKind,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LocalPackageOverride {
    pub package: String,
    pub path: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PackageResolution {
    pub default_registry: String,
    pub registries: Vec<RegistryMapping>,
    pub requirements: Vec<PackageRequirement>,
    pub local_overrides: Vec<LocalPackageOverride>,
    pub lock_path: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WitSelection {
    pub package: String,
    pub world: String,
    pub source: StoreObject,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum RustProfile {
    Debug,
    Release,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RustImplementation {
    pub source: StoreObject,
    pub package: String,
    pub crate_name: String,
    pub features: Vec<String>,
    pub profile: RustProfile,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ToolIdentity {
    pub version: String,
    pub executable: StoreObject,
    pub configuration_identity_blake3: Blake3Identity,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ToolCohort {
    pub rust_toolchain: ToolIdentity,
    pub rust_target: String,
    pub wit_bindgen: ToolIdentity,
    pub wasm_component_ld: ToolIdentity,
    pub wasm_tools: ToolIdentity,
    pub wac: ToolIdentity,
    pub wasi_virt: ToolIdentity,
    pub wizer: ToolIdentity,
    pub wasmtime: ToolIdentity,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct RequiredImport {
    pub name: String,
    pub world: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CompositionNode {
    pub id: String,
    pub package: String,
    pub world: String,
    pub artifact: StoreObject,
    pub required_imports: Vec<RequiredImport>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct CompositionEdge {
    pub provider_node: String,
    pub provider_export: String,
    pub consumer_node: String,
    pub consumer_import: String,
    pub world: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Composition {
    pub package: String,
    pub source_wac: String,
    pub nodes: Vec<CompositionNode>,
    pub edges: Vec<CompositionEdge>,
    pub output_world: String,
    pub import_dependencies: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum WasiSubsystem {
    Cli,
    Clocks,
    Environment,
    Filesystem,
    Http,
    Network,
    Poll,
    Random,
    Sockets,
    Stdio,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum VirtualizationMode {
    Deny,
    Allow,
    Ignore,
    FixedValue,
    VirtualMount,
    Passthrough,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VirtualizationRule {
    pub subsystem: WasiSubsystem,
    pub mode: VirtualizationMode,
    pub value: Option<String>,
    pub value_identity_blake3: Option<Blake3Identity>,
    pub input: Option<StoreObject>,
    pub guest_path: Option<String>,
    pub review_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VirtualizationConfig {
    pub defaults_overridden: bool,
    pub rules: Vec<VirtualizationRule>,
    pub expected_remaining_imports: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ValidationProfiles {
    pub octet_artifact_profile_identity_blake3: Blake3Identity,
    pub expected_runtime_profile_identity_blake3: Blake3Identity,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum WizerMode {
    Disabled,
    Diagnostic,
    Deterministic,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WizerConfig {
    pub mode: WizerMode,
    pub initialization_entrypoint: Option<String>,
    pub deterministic_virtual_imports: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum AotMode {
    Disabled,
    TrustedNative,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AotConfig {
    pub mode: AotMode,
    pub target: Option<String>,
    pub cpu_features: Vec<String>,
    pub wasmtime_configuration_identity_blake3: Option<Blake3Identity>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum OutputClass {
    ToolInput,
    CompositionInput,
    RuntimeProfile,
    WitPackage,
    GeneratedBindings,
    PortableComponent,
    ValidatedPortableComponent,
    TransformedPortableComponent,
    PrecompiledNative,
    MaterializationBundle,
    BuildReport,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OutputDeclaration {
    pub name: String,
    pub path: String,
    pub class: OutputClass,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ComponentManifest {
    pub schema: String,
    pub name: String,
    pub package_resolution: PackageResolution,
    pub wit: WitSelection,
    pub implementation: RustImplementation,
    pub cohort: ToolCohort,
    pub composition: Composition,
    pub virtualization: VirtualizationConfig,
    pub validation_profiles: ValidationProfiles,
    pub wizer: WizerConfig,
    pub aot: AotConfig,
    pub outputs: Vec<OutputDeclaration>,
    pub non_claims: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WkgLock {
    pub version: u32,
    pub packages: Vec<WkgLockPackage>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WkgLockPackage {
    pub name: String,
    pub registry: String,
    pub versions: Vec<WkgLockedVersion>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WkgLockedVersion {
    pub requirement: String,
    pub version: String,
    pub digest: OciSha256Digest,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PackageMaterialization {
    pub package: String,
    pub version: String,
    pub registry: String,
    pub protocol_digest: OciSha256Digest,
    pub object: StoreObject,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LockFacts {
    pub lock: WkgLock,
    pub lock_bytes_blake3: Blake3Identity,
    pub registry_config_blake3: Blake3Identity,
    pub materializations: Vec<PackageMaterialization>,
}
