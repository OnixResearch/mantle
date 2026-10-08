//! Imperative shell for constructing the full-source Rust binding receipt.

use std::collections::BTreeMap;
use std::fs;
use std::fs::File;
use std::io::BufReader;
use std::io::Read;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;

use crunch_attestation::ArtifactAttestation;
use crunch_attestation::Canonicalize;
use crunch_attestation::EdgeKind;
use mantle_application_contract::ApplicationOutcome;
use mantle_application_contract::CommandFamily;
use mantle_application_contract::EffectId;
use mantle_application_contract::EffectKind;
use mantle_application_contract::EffectMeasure;
use mantle_application_contract::EffectOutput;
use mantle_application_contract::EffectSpec;
use mantle_application_contract::ExpectedOutput;
use mantle_application_contract::Observation;
use mantle_application_contract::ObservationStatus;
use mantle_application_contract::classify_observations;
use mantle_application_contract::plan_effects;
use sha2::Sha256;

use crate::full_source_provider::FullSourceProviderAdmissionReport;
use crate::full_source_provider::admit_full_source_provider;
use crate::full_source_rust_binding::FULL_SOURCE_RUST_BINDING_SCHEMA;
use crate::full_source_rust_binding::FullSourceNativeArtifactBinding;
use crate::full_source_rust_binding::FullSourceNativeProviderAdmissionIdentity;
use crate::full_source_rust_binding::FullSourceRustArtifactBinding;
use crate::full_source_rust_binding::FullSourceRustHostSupportInputBinding;
use crate::full_source_rust_binding::FullSourceRustHostToolConstructionReceipt;
use crate::full_source_rust_binding::FullSourceRustHostToolManifest;
use crate::full_source_rust_binding::FullSourceRustProviderBindingReceipt;
use crate::full_source_rust_binding::FullSourceRustReceiptBinding;
use crate::full_source_rust_binding::canonical_full_source_rust_binding_bytes;
use crate::full_source_rust_binding::required_full_source_native_artifacts;
use crate::full_source_rust_binding::validate_full_source_rust_host_tool_construction_receipt;
use crate::full_source_rust_binding::validate_full_source_rust_host_tool_manifest;
use crate::full_source_rust_binding::validate_full_source_rust_provider_binding;
use crate::rust_source_provider::RustSourceProviderError;
use crate::rust_source_provider::observed_provider_receipts;
use crate::rust_source_provider::validate_materialized_rust_source_provider;
use crate::source_toolchain_closure::RustSourceProviderMetadata;

pub(crate) const FULL_SOURCE_RUST_BINDING_RELATIVE_PATH: &str =
    "share/mantle-rust-provider/receipts/full-source-binding.json";
pub(crate) const FULL_SOURCE_RUST_HOST_TOOL_MANIFEST_FILE: &str = "full-source-rust-host-tools.json";
const HOST_TOOL_INPUT_EFFECT: &str = "bootstrap-rust-host-tool-inputs";
const HOST_TOOL_PUBLISH_EFFECT: &str = "bootstrap-rust-host-tool-publication";
const HOST_TOOL_READBACK_EFFECT: &str = "bootstrap-rust-host-tool-manifest-readback";
const FULL_SOURCE_LINUX_HEADERS_ATTESTATION_FILE: &str = "linux-headers-attestation.json";
const FULL_SOURCE_BINDING_RECEIPT_ID: &str = "full-source-rust-provider-construction";
const FULL_SOURCE_PROVIDER_TARGET: &str = "x86_64-linux-musl";
const FULL_SOURCE_COMPILER_TARGET: &str = "x86_64-unknown-linux-musl";
const FULL_SOURCE_SOURCE_POLICY: &str = "authenticated-offline-only";
const FULL_SOURCE_LOGICAL_STORE_PREFIX: &str = "/mantle/store";
const SOURCE_ARCHIVE_PATH: &str = "archive";
const SOURCE_HASH_ALGORITHM: &str = "sha256";
const SOURCE_HASH_MODE: &str = "recursive";
const SOURCE_PAYLOAD_ENCODING: &str = "tarball-archive-v1";
const SOURCE_UNPACK_ENABLED: &str = "1";
const LINUX_HEADERS_SOURCE_ID: &str = "linux-6.6";
const LINUX_HEADERS_SOURCE_NAME: &str = "linux-6.6-src";
const LINUX_HEADERS_SOURCE_URL: &str = "https://cdn.kernel.org/pub/linux/kernel/v6.x/linux-6.6.tar.xz";
const LINUX_HEADERS_SOURCE_SHA256_HEX: &str = "d926a06c63dd8ac7df3f86ee1ffc2ce2a3b81a2d168484e76b5b389aba8e56d0";
const LINUX_HEADERS_SOURCE_RECURSIVE_HASH_SRI: &str = "sha256-+GqM1/8Qw0N6Z2t45jjapVZrG0JysMs9PVi79GIK1Ec=";
const LINUX_HEADERS_MARKER_RELATIVE_PATH: &str = "share/mantle-bootstrap/full-source-rust-host-linux-headers.txt";
const LINUX_HEADERS_SUPPORT_INPUT_ID: &str = "linux-headers";
const LINUX_HEADERS_REQUIRED_RELATIVE_PATHS: [&str; 2] = ["include/asm/prctl.h", "include/linux/prctl.h"];
const HEX_RADIX: u32 = 16;
const HEX_ALPHA_OFFSET: u8 = 10;
const BITS_PER_HEX_DIGIT: u32 = 4;
const HEX_DIGITS_PER_BYTE: usize = 2;
const HEX_DIGITS_PER_BYTE_U64: u64 = 2;
const SHA256_HEX_LENGTH: usize = 64;
const BLAKE3_HEX_LENGTH: usize = 64;
const NATIVE_ARTIFACT_BYTES_MAX: u64 = 536_870_912;
const HOST_TOOL_MANIFEST_BYTES_MAX: u64 = 1_048_576;
const HOST_TOOL_RECEIPT_BYTES_MAX: u64 = 1_048_576;
const HOST_TOOL_ATTESTATION_BYTES_MAX: u64 = 1_048_576;
const HASH_BUFFER_BYTES: usize = 65_536;
const ELF_MAGIC: &[u8; 4] = b"\x7fELF";
#[cfg(unix)]
const EXECUTABLE_PERMISSION_MASK: u32 = 0o111;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct FullSourceRustBindingPublication {
    pub(crate) path: PathBuf,
    pub(crate) content_digest_blake3: String,
    pub(crate) native_artifact_count: usize,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct FullSourceRustHostToolObservation {
    pub(crate) manifest: FullSourceRustHostToolManifest,
    pub(crate) manifest_digest_blake3: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct FullSourceRustExecutionContext {
    pub(crate) native_provider_dir: PathBuf,
    pub(crate) rust_source_archive_dir: PathBuf,
    pub(crate) linux_headers_root: PathBuf,
    pub(crate) admission: FullSourceNativeProviderAdmissionIdentity,
    pub(crate) host_tools: FullSourceRustHostToolObservation,
}

pub(crate) struct FullSourceRustHostToolMaterializationRequest<'a> {
    pub(crate) admission_report_path: &'a Path,
    pub(crate) make_root: &'a Path,
    pub(crate) make_attestation_path: &'a Path,
    pub(crate) cmake_root: &'a Path,
    pub(crate) cmake_attestation_path: &'a Path,
    pub(crate) python_root: &'a Path,
    pub(crate) python_attestation_path: &'a Path,
    pub(crate) perl_root: &'a Path,
    pub(crate) perl_attestation_path: &'a Path,
    pub(crate) busybox_root: &'a Path,
    pub(crate) busybox_attestation_path: &'a Path,
    pub(crate) linux_headers_root: &'a Path,
    pub(crate) linux_headers_attestation_path: &'a Path,
    pub(crate) output_dir: &'a Path,
}

#[derive(Clone, Copy)]
struct FullSourceRustHostToolSpec {
    role: crate::full_source_rust_binding::FullSourceRustHostToolRole,
    source_id: &'static str,
    source_name: &'static str,
    source_url: &'static str,
    source_sha256_hex: &'static str,
    source_recursive_hash_sri: &'static str,
    executable_relative_path: &'static str,
    marker_relative_path: &'static str,
    positive_checks: &'static [&'static str],
    rejection_checks: &'static [&'static str],
}

struct ObservedFullSourceRustHostTool {
    spec: FullSourceRustHostToolSpec,
    executable_path: PathBuf,
    executable_digest_blake3: String,
    attestation_path: PathBuf,
    attestation_digest_blake3: String,
    attestation_file_digest_blake3: String,
}

const FULL_SOURCE_RUST_HOST_TOOL_SPECS: &[FullSourceRustHostToolSpec] = &[
    FullSourceRustHostToolSpec {
        role: crate::full_source_rust_binding::FullSourceRustHostToolRole::Make,
        source_id: "gnu-make-4.4.1",
        source_name: "make-4.4.1-full-source-src",
        source_url: "https://mirrors.kernel.org/gnu/make/make-4.4.1.tar.gz",
        source_sha256_hex: "dd16fb1d67bfab79a72f5e8390735c49e3e8e70b4945a15ab1f81ddb78658fb3",
        source_recursive_hash_sri: "sha256-+Cg7R8wougcWcFf6u5B3lVX6RRgMRYZ1CD+e21URP5A=",
        executable_relative_path: "bin/make",
        marker_relative_path: "share/mantle-bootstrap/full-source-rust-host-make.txt",
        positive_checks: &["version", "recipe-execution", "copied-tree-version"],
        rejection_checks: &["malformed-makefile-rejected"],
    },
    FullSourceRustHostToolSpec {
        role: crate::full_source_rust_binding::FullSourceRustHostToolRole::Cmake,
        source_id: "cmake-3.31.8",
        source_name: "cmake-3.31.8-src",
        source_url: "https://github.com/Kitware/CMake/releases/download/v3.31.8/cmake-3.31.8.tar.gz",
        source_sha256_hex: "e3cde3ca83dc2d3212105326b8f1b565116be808394384007e7ef1c253af6caa",
        source_recursive_hash_sri: "sha256-1sQqWhxydhTeQ0PUpW/wToX/u4kVmd6YE97O40G2sV8=",
        executable_relative_path: "bin/cmake",
        marker_relative_path: "share/mantle-bootstrap/full-source-rust-host-cmake.txt",
        positive_checks: &["version", "c-configure-build-run", "copied-tree-version"],
        rejection_checks: &["malformed-project-rejected"],
    },
    FullSourceRustHostToolSpec {
        role: crate::full_source_rust_binding::FullSourceRustHostToolRole::Python,
        source_id: "cpython-3.13.5",
        source_name: "python-3.13.5-src",
        source_url: "https://www.python.org/ftp/python/3.13.5/Python-3.13.5.tar.xz",
        source_sha256_hex: "93e583f243454e6e9e4588ca2c2662206ad961659863277afcdb96801647d640",
        source_recursive_hash_sri: "sha256-CViHmIaqSfZ4TNLtDEGqlWrq1rLQgKwfZiRgh7ohehU=",
        executable_relative_path: "bin/python3.13",
        marker_relative_path: "share/mantle-bootstrap/full-source-rust-host-python.txt",
        positive_checks: &["version", "rust-bootstrap-stdlib-imports", "copied-tree-execution"],
        rejection_checks: &["malformed-python-rejected"],
    },
    FullSourceRustHostToolSpec {
        role: crate::full_source_rust_binding::FullSourceRustHostToolRole::Perl,
        source_id: "perl-5.10.1",
        source_name: "perl-5.10.1-src",
        source_url: "https://www.cpan.org/src/5.0/perl-5.10.1.tar.gz",
        source_sha256_hex: "cb7f26ea4b2b28d6644354d87a269d01cac1b635287dae64e88eeafa24b44f35",
        source_recursive_hash_sri: "sha256-syEkocmqP1eN/w+VRDc+Gg5jHQryrKxrL7FyNuWBzHE=",
        executable_relative_path: "bin/perl",
        marker_relative_path: "share/mantle-bootstrap/full-source-rust-host-perl.txt",
        positive_checks: &[
            "version",
            "openssl-configure-core-modules",
            "openssl-generator-re",
            "openssl-param-decoder-scalar-capture",
            "copied-tree-execution",
        ],
        rejection_checks: &["malformed-perl-rejected"],
    },
    FullSourceRustHostToolSpec {
        role: crate::full_source_rust_binding::FullSourceRustHostToolRole::Busybox,
        source_id: "busybox-1.37.0",
        source_name: "busybox-1.37.0-full-source-src",
        source_url: "https://busybox.net/downloads/busybox-1.37.0.tar.bz2",
        source_sha256_hex: "3311dff32e746499f4df0d5df04d7eb396382d7e108bb9250e7b519b837043a4",
        source_recursive_hash_sri: "sha256-tDFhcU0GUX3/ct2Hw+Ad8iNyMl0GWovhQynLgooh+qU=",
        executable_relative_path: "bin/busybox",
        marker_relative_path: "share/mantle-bootstrap/full-source-rust-host-busybox.txt",
        positive_checks: &["static-shell-core-utilities", "copied-tree-execution"],
        rejection_checks: &["malformed-grep-pattern-rejected"],
    },
];

trait FullSourceHostToolPort {
    fn output_exists(&mut self, output_dir: &Path) -> bool;
    fn materialize(
        &mut self,
        request: &FullSourceRustHostToolMaterializationRequest<'_>,
    ) -> Result<PathBuf, RustSourceProviderError>;
    fn readback(&mut self, path: &Path) -> Result<FullSourceRustHostToolManifest, RustSourceProviderError>;
}

struct LocalFullSourceHostTools;

impl FullSourceHostToolPort for LocalFullSourceHostTools {
    fn output_exists(&mut self, output_dir: &Path) -> bool {
        output_dir.exists()
    }

    fn materialize(
        &mut self,
        request: &FullSourceRustHostToolMaterializationRequest<'_>,
    ) -> Result<PathBuf, RustSourceProviderError> {
        fs::create_dir(request.output_dir).map_err(|error| {
            RustSourceProviderError::Copy(format!(
                "creating full-source Rust host-tool evidence directory {}: {error}",
                request.output_dir.display()
            ))
        })?;
        let result = materialize_full_source_rust_host_tools_inner(request);
        if result.is_err() {
            let _ = fs::remove_dir_all(request.output_dir);
        }
        result
    }

    fn readback(&mut self, path: &Path) -> Result<FullSourceRustHostToolManifest, RustSourceProviderError> {
        let bytes = read_nonempty_bounded(path, "full-source Rust host-tool manifest", HOST_TOOL_MANIFEST_BYTES_MAX)?;
        let manifest: FullSourceRustHostToolManifest = serde_json::from_slice(&bytes).map_err(|error| {
            RustSourceProviderError::Parse(format!("parsing host-tool manifest {}: {error}", path.display()))
        })?;
        validate_full_source_rust_host_tool_manifest(&manifest)
            .map_err(|error| RustSourceProviderError::Validate(error.to_string()))?;
        Ok(manifest)
    }
}

pub(crate) fn prepare_full_source_rust_execution_context(
    admission_report_path: &Path,
    host_tool_manifest_path: &Path,
    rust_source_archive_dir: &Path,
) -> Result<FullSourceRustExecutionContext, RustSourceProviderError> {
    let report_bytes = read_nonempty(admission_report_path, "full-source admission report")?;
    let parsed_report = parse_admission_report(admission_report_path, &report_bytes)?;
    let revalidated_report = revalidate_admission_report(&parsed_report)?;
    validate_report_matches_revalidation(&parsed_report, &revalidated_report)?;
    let admission = admission_identity(&revalidated_report, &report_bytes)?;
    let host_tools = observe_full_source_rust_host_tools(host_tool_manifest_path, &admission.output_digest_blake3)?;
    let native_provider_dir = fs::canonicalize(&revalidated_report.provider_path).map_err(|error| {
        RustSourceProviderError::Read(format!(
            "canonicalizing admitted native provider {}: {error}",
            revalidated_report.provider_path.display()
        ))
    })?;
    if !native_provider_dir.is_absolute() {
        return Err(RustSourceProviderError::Validate(
            "canonical admitted native provider path is not absolute".to_string(),
        ));
    }
    let rust_source_archive_dir = canonical_source_archive_dir(rust_source_archive_dir)?;
    let linux_headers_root = canonical_linux_headers_root(&host_tools.manifest)?;
    Ok(FullSourceRustExecutionContext {
        native_provider_dir,
        rust_source_archive_dir,
        linux_headers_root,
        admission,
        host_tools,
    })
}

fn canonical_source_archive_dir(path: &Path) -> Result<PathBuf, RustSourceProviderError> {
    canonical_non_symlink_directory(path, "full-source Rust archive")
}

fn canonical_linux_headers_root(manifest: &FullSourceRustHostToolManifest) -> Result<PathBuf, RustSourceProviderError> {
    let binding = manifest
        .support_inputs
        .iter()
        .find(|input| input.id == LINUX_HEADERS_SUPPORT_INPUT_ID)
        .ok_or_else(|| RustSourceProviderError::Validate("Linux header support input is missing".to_string()))?;
    canonical_non_symlink_directory(Path::new(&binding.path), "full-source Linux header support")
}

fn canonical_non_symlink_directory(path: &Path, label: &str) -> Result<PathBuf, RustSourceProviderError> {
    let metadata = fs::symlink_metadata(path)
        .map_err(|error| RustSourceProviderError::Read(format!("{label} directory {}: {error}", path.display())))?;
    if !metadata.file_type().is_dir() {
        return Err(RustSourceProviderError::Validate(format!(
            "{label} path must be a directory without symlink traversal: {}",
            path.display()
        )));
    }
    let canonical = fs::canonicalize(path).map_err(|error| {
        RustSourceProviderError::Read(format!("canonicalizing {label} directory {}: {error}", path.display()))
    })?;
    if !canonical.is_absolute() {
        return Err(RustSourceProviderError::Validate(format!("canonical {label} directory is not absolute")));
    }
    Ok(canonical)
}

pub(crate) fn materialize_full_source_rust_host_tools(
    request: FullSourceRustHostToolMaterializationRequest<'_>,
) -> Result<PathBuf, RustSourceProviderError> {
    let plan = plan_effects(CommandFamily::Bootstrap, &[
        EffectSpec {
            effect_id: HOST_TOOL_INPUT_EFFECT,
            kind: EffectKind::ReadFiles,
            limit: EffectMeasure::Calls(1),
            expected_output: ExpectedOutput::None,
        },
        EffectSpec {
            effect_id: HOST_TOOL_PUBLISH_EFFECT,
            kind: EffectKind::WriteFiles,
            limit: EffectMeasure::Calls(1),
            expected_output: ExpectedOutput::None,
        },
        EffectSpec {
            effect_id: HOST_TOOL_READBACK_EFFECT,
            kind: EffectKind::ReadFiles,
            limit: EffectMeasure::Calls(1),
            expected_output: ExpectedOutput::Identity(
                crate::full_source_rust_binding::FULL_SOURCE_RUST_HOST_TOOL_SCHEMA,
            ),
        },
    ])
    .map_err(|error| RustSourceProviderError::Validate(format!("planning host-tool publication: {}", error.code())))?;
    let mut observed = [
        host_tool_effect(HOST_TOOL_INPUT_EFFECT, EffectKind::ReadFiles),
        host_tool_effect(HOST_TOOL_PUBLISH_EFFECT, EffectKind::WriteFiles),
        host_tool_effect(HOST_TOOL_READBACK_EFFECT, EffectKind::ReadFiles),
    ];
    let result = execute_host_tool_materialization(&request, &mut LocalFullSourceHostTools, &mut observed);
    match (classify_observations(&plan, &observed), result) {
        (ApplicationOutcome::Completed, Ok(path)) => Ok(path),
        (ApplicationOutcome::Failed { .. }, Err(error)) => Err(error),
        (outcome, _) => Err(RustSourceProviderError::Validate(format!(
            "full-source Rust host-tool observations inconsistent: {outcome:?}"
        ))),
    }
}

fn host_tool_effect(id: &str, kind: EffectKind) -> Observation {
    Observation {
        effect_id: EffectId(id.to_string()),
        kind,
        status: ObservationStatus::Skipped,
        output: EffectOutput::None,
        usage: EffectMeasure::Calls(0),
        diagnostics_code: None,
    }
}

fn host_tool_record<T>(observation: &mut Observation, result: &Result<T, RustSourceProviderError>) {
    observation.status = if result.is_ok() {
        ObservationStatus::Succeeded
    } else {
        ObservationStatus::Failed
    };
    observation.usage = EffectMeasure::Calls(1);
    observation.diagnostics_code = result.is_err().then(|| format!("{}-failed", observation.effect_id.0));
}

fn execute_host_tool_materialization(
    request: &FullSourceRustHostToolMaterializationRequest<'_>,
    port: &mut impl FullSourceHostToolPort,
    observed: &mut [Observation; 3],
) -> Result<PathBuf, RustSourceProviderError> {
    let exists = port.output_exists(request.output_dir);
    observed[0].status = ObservationStatus::Succeeded;
    observed[0].usage = EffectMeasure::Calls(1);
    if exists {
        return Err(RustSourceProviderError::Copy(format!(
            "full-source Rust host-tool evidence directory already exists: {}",
            request.output_dir.display()
        )));
    }
    let published = port.materialize(request);
    host_tool_record(&mut observed[1], &published);
    let path = published?;
    let manifest = port.readback(&path);
    host_tool_record(&mut observed[2], &manifest);
    let manifest = manifest?;
    observed[2].output = EffectOutput::Identity(manifest.schema.clone());
    Ok(path)
}

fn materialize_full_source_rust_host_tools_inner(
    request: &FullSourceRustHostToolMaterializationRequest<'_>,
) -> Result<PathBuf, RustSourceProviderError> {
    let report_bytes = read_nonempty(request.admission_report_path, "full-source admission report")?;
    let parsed_report = parse_admission_report(request.admission_report_path, &report_bytes)?;
    let revalidated_report = revalidate_admission_report(&parsed_report)?;
    validate_report_matches_revalidation(&parsed_report, &revalidated_report)?;
    let source_closure =
        crate::source_bundle::read_source_bundle(&revalidated_report.source_closure_path).map_err(|error| {
            RustSourceProviderError::Parse(format!("reading admitted host-tool source closure: {error}"))
        })?;
    for spec in FULL_SOURCE_RUST_HOST_TOOL_SPECS {
        validate_host_tool_source_record(*spec, &source_closure)?;
    }
    validate_source_record(
        LINUX_HEADERS_SOURCE_ID,
        LINUX_HEADERS_SOURCE_NAME,
        LINUX_HEADERS_SOURCE_URL,
        LINUX_HEADERS_SOURCE_SHA256_HEX,
        LINUX_HEADERS_SOURCE_RECURSIVE_HASH_SRI,
        &source_closure,
    )?;
    let (linux_headers_attestation_digest_blake3, linux_headers_attestation_bytes) =
        observe_host_dependency_attestation(
            request.linux_headers_root,
            request.linux_headers_attestation_path,
            LINUX_HEADERS_SOURCE_NAME,
            LINUX_HEADERS_MARKER_RELATIVE_PATH,
        )?;
    write_create_new(
        &request.output_dir.join(FULL_SOURCE_LINUX_HEADERS_ATTESTATION_FILE),
        &linux_headers_attestation_bytes,
    )?;
    let input_pairs = [
        (request.make_root, request.make_attestation_path),
        (request.cmake_root, request.cmake_attestation_path),
        (request.python_root, request.python_attestation_path),
        (request.perl_root, request.perl_attestation_path),
        (request.busybox_root, request.busybox_attestation_path),
    ];
    assert_eq!(input_pairs.len(), FULL_SOURCE_RUST_HOST_TOOL_SPECS.len());
    let mut observed = Vec::with_capacity(FULL_SOURCE_RUST_HOST_TOOL_SPECS.len());
    for (spec, (root, attestation_path)) in FULL_SOURCE_RUST_HOST_TOOL_SPECS.iter().zip(input_pairs) {
        observed.push(observe_full_source_rust_host_tool(*spec, root, attestation_path)?);
    }
    let (_, linux_headers_digest_blake3) = crate::release_tree_copy::hash_directory_tree(request.linux_headers_root)
        .map_err(|error| {
            RustSourceProviderError::Digest(format!(
                "hashing Linux header tool dependency {}: {error}",
                request.linux_headers_root.display()
            ))
        })?;
    let make_digest = observed
        .iter()
        .find(|tool| tool.spec.role == crate::full_source_rust_binding::FullSourceRustHostToolRole::Make)
        .map(|tool| tool.executable_digest_blake3.clone())
        .ok_or_else(|| RustSourceProviderError::Validate("observed Make host tool is missing".to_string()))?;
    let busybox_digest = observed
        .iter()
        .find(|tool| tool.spec.role == crate::full_source_rust_binding::FullSourceRustHostToolRole::Busybox)
        .map(|tool| tool.executable_digest_blake3.clone())
        .ok_or_else(|| RustSourceProviderError::Validate("observed BusyBox host tool is missing".to_string()))?;
    let mut bindings = Vec::with_capacity(observed.len());
    for tool in observed {
        let dependencies = host_tool_dependency_digests(
            tool.spec.role,
            &revalidated_report.output_digest_blake3,
            &make_digest,
            &busybox_digest,
            &linux_headers_digest_blake3,
            &linux_headers_attestation_digest_blake3,
        );
        bindings.push(write_full_source_rust_host_tool_receipt(request.output_dir, tool, dependencies)?);
    }
    let manifest = FullSourceRustHostToolManifest {
        schema: crate::full_source_rust_binding::FULL_SOURCE_RUST_HOST_TOOL_SCHEMA.to_string(),
        source_policy: FULL_SOURCE_SOURCE_POLICY.to_string(),
        ambient_tool_discovery: false,
        tools: bindings,
        support_inputs: vec![FullSourceRustHostSupportInputBinding {
            id: "linux-headers".to_string(),
            path: request.linux_headers_root.display().to_string(),
            content_digest_blake3: linux_headers_digest_blake3,
            source_id: LINUX_HEADERS_SOURCE_ID.to_string(),
            attestation_path: request.output_dir.join(FULL_SOURCE_LINUX_HEADERS_ATTESTATION_FILE).display().to_string(),
            attestation_digest_blake3: linux_headers_attestation_digest_blake3,
        }],
    };
    validate_full_source_rust_host_tool_manifest(&manifest)
        .map_err(|error| RustSourceProviderError::Validate(error.to_string()))?;
    let mut bytes = serde_json::to_vec_pretty(&manifest)
        .map_err(|error| RustSourceProviderError::Parse(format!("serializing host-tool manifest: {error}")))?;
    bytes.push(b'\n');
    let path = request.output_dir.join(FULL_SOURCE_RUST_HOST_TOOL_MANIFEST_FILE);
    write_create_new(&path, &bytes)?;
    Ok(path)
}

fn validate_host_tool_source_record(
    spec: FullSourceRustHostToolSpec,
    manifest: &crate::source_bundle::SourceBundleManifest,
) -> Result<(), RustSourceProviderError> {
    validate_source_record(
        spec.source_id,
        spec.source_name,
        spec.source_url,
        spec.source_sha256_hex,
        spec.source_recursive_hash_sri,
        manifest,
    )
}

fn validate_source_record(
    source_id: &str,
    source_name: &str,
    source_url: &str,
    source_sha256_hex: &str,
    source_recursive_hash_sri: &str,
    manifest: &crate::source_bundle::SourceBundleManifest,
) -> Result<(), RustSourceProviderError> {
    let matches = manifest
        .records
        .iter()
        .filter(|record| {
            record.kind == crate::source_bundle::SourceRecordKind::FixedUrl
                && record.metadata.get("name").map(String::as_str) == Some(source_name)
        })
        .collect::<Vec<_>>();
    if matches.len() != 1 {
        return Err(RustSourceProviderError::Validate(format!(
            "admitted source closure must contain one {source_name} record, got {}",
            matches.len()
        )));
    }
    let record = matches[0];
    let expected_metadata = [
        ("builder", "builtin:fetchurl"),
        ("hash", source_recursive_hash_sri),
        ("hash_algo", SOURCE_HASH_ALGORITHM),
        ("hash_mode", SOURCE_HASH_MODE),
        ("name", source_name),
        ("payload_encoding", SOURCE_PAYLOAD_ENCODING),
        ("unpack", SOURCE_UNPACK_ENABLED),
        ("url", source_url),
    ];
    for (key, expected) in expected_metadata {
        let observed = record.metadata.get(key).map(String::as_str);
        if observed != Some(expected) {
            return Err(RustSourceProviderError::Validate(format!(
                "admitted {source_id} source metadata {key} mismatch: expected {expected}, got {}",
                observed.unwrap_or("<missing>")
            )));
        }
    }
    let observed_sha256_hex = validate_source_archive_entries(source_id, &record.files, record.payload_bytes)?;
    if observed_sha256_hex != source_sha256_hex {
        return Err(RustSourceProviderError::Validate(format!(
            "admitted {source_id} source archive SHA-256 mismatch: expected {source_sha256_hex}, got {observed_sha256_hex}"
        )));
    }
    assert_eq!(observed_sha256_hex.len(), SHA256_HEX_LENGTH);
    debug_assert!(!record.files.is_empty());
    Ok(())
}

fn validate_source_archive_entries(
    source_id: &str,
    files: &[crate::source_bundle::SourceFileEntry],
    payload_bytes: u64,
) -> Result<String, RustSourceProviderError> {
    if files.is_empty() {
        return Err(RustSourceProviderError::Validate(format!(
            "admitted {source_id} source archive has no captured chunks"
        )));
    }
    let chunk_count = u32::try_from(files.len())
        .map_err(|_| RustSourceProviderError::Validate("source archive chunk count exceeds u32".to_string()))?;
    let is_chunked = files.len() > 1;
    let mut observed_payload_bytes = 0u64;
    for (index, archive) in files.iter().enumerate() {
        let expected_index = u32::try_from(index)
            .map_err(|_| RustSourceProviderError::Validate("source archive chunk index exceeds u32".to_string()))?;
        validate_source_archive_entry(source_id, archive, is_chunked, expected_index, chunk_count)?;
        observed_payload_bytes = observed_payload_bytes.checked_add(archive.size).ok_or_else(|| {
            RustSourceProviderError::Validate("source archive payload byte count overflows u64".to_string())
        })?;
    }
    if observed_payload_bytes != payload_bytes {
        return Err(RustSourceProviderError::Validate(format!(
            "admitted {source_id} source archive size does not match payload bytes"
        )));
    }
    let digest = sha256_content_hex_parts(files.iter().map(|archive| archive.content_hex.as_deref().unwrap()))?;
    assert_eq!(digest.len(), SHA256_HEX_LENGTH);
    debug_assert_eq!(observed_payload_bytes, payload_bytes);
    Ok(digest)
}

fn validate_source_archive_entry(
    source_id: &str,
    archive: &crate::source_bundle::SourceFileEntry,
    is_chunked: bool,
    expected_index: u32,
    chunk_count: u32,
) -> Result<(), RustSourceProviderError> {
    if archive.path != SOURCE_ARCHIVE_PATH || archive.file_type != crate::source_bundle::SourceFileType::Regular {
        return Err(RustSourceProviderError::Validate(format!(
            "admitted {source_id} source payload is not one regular archive"
        )));
    }
    let expected_chunk = is_chunked.then_some((expected_index, chunk_count));
    if (archive.chunk_index, archive.chunk_count)
        != expected_chunk.map_or((None, None), |value| (Some(value.0), Some(value.1)))
    {
        return Err(RustSourceProviderError::Validate(format!(
            "admitted {source_id} source archive chunk sequence is not contiguous"
        )));
    }
    let content_hex = archive.content_hex.as_deref().ok_or_else(|| {
        RustSourceProviderError::Validate(format!("admitted {source_id} source archive has no captured bytes"))
    })?;
    let expected_hex_len = archive
        .size
        .checked_mul(HEX_DIGITS_PER_BYTE_U64)
        .and_then(|length| usize::try_from(length).ok())
        .ok_or_else(|| RustSourceProviderError::Validate("source archive hex length overflows usize".to_string()))?;
    if expected_hex_len != content_hex.len() {
        return Err(RustSourceProviderError::Validate(format!(
            "admitted {source_id} source archive hex length mismatch"
        )));
    }
    assert_eq!(content_hex.len() % HEX_DIGITS_PER_BYTE, 0);
    debug_assert!(archive.size > 0);
    Ok(())
}

#[cfg(test)]
fn sha256_content_hex(content_hex: &str) -> Result<String, RustSourceProviderError> {
    sha256_content_hex_parts([content_hex])
}

fn sha256_content_hex_parts<'a>(
    content_hex_parts: impl IntoIterator<Item = &'a str>,
) -> Result<String, RustSourceProviderError> {
    let mut hasher = <Sha256 as sha2::Digest>::new();
    let mut decoded = Vec::with_capacity(HASH_BUFFER_BYTES);
    let mut part_count = 0u64;
    for content_hex in content_hex_parts {
        if content_hex.is_empty() || !content_hex.len().is_multiple_of(HEX_DIGITS_PER_BYTE) {
            return Err(RustSourceProviderError::Validate(
                "source archive content hex must be nonempty and byte-aligned".to_string(),
            ));
        }
        part_count = part_count.checked_add(1).ok_or_else(|| {
            RustSourceProviderError::Validate("source archive content part count overflows u64".to_string())
        })?;
        let (byte_pairs, remainder) = content_hex.as_bytes().as_chunks::<HEX_DIGITS_PER_BYTE>();
        assert!(remainder.is_empty());
        for pair in byte_pairs {
            let high = hex_digit(pair[0])?;
            let low = hex_digit(pair[1])?;
            decoded.push((high << BITS_PER_HEX_DIGIT) | low);
            if decoded.len() == HASH_BUFFER_BYTES {
                sha2::Digest::update(&mut hasher, &decoded);
                decoded.clear();
            }
        }
    }
    if part_count == 0 {
        return Err(RustSourceProviderError::Validate("source archive content parts must not be empty".to_string()));
    }
    sha2::Digest::update(&mut hasher, &decoded);
    let digest = format!("{:x}", sha2::Digest::finalize(hasher));
    assert_eq!(digest.len(), SHA256_HEX_LENGTH);
    debug_assert!(part_count > 0);
    Ok(digest)
}

fn hex_digit(byte: u8) -> Result<u8, RustSourceProviderError> {
    let value = match byte {
        b'0'..=b'9' => byte - b'0',
        b'a'..=b'f' => byte - b'a' + HEX_ALPHA_OFFSET,
        b'A'..=b'F' => byte - b'A' + HEX_ALPHA_OFFSET,
        _ => {
            return Err(RustSourceProviderError::Validate(format!(
                "source archive content contains non-hex byte 0x{byte:02x}"
            )));
        }
    };
    assert!(value < HEX_RADIX as u8);
    debug_assert!(byte.is_ascii_hexdigit());
    Ok(value)
}

fn validate_host_tool_attestation_binding(
    attestation: &ArtifactAttestation,
    expected_logical_path: &str,
    source_name: &str,
) -> Result<(), RustSourceProviderError> {
    if expected_logical_path.is_empty() || source_name.is_empty() {
        return Err(RustSourceProviderError::Validate(
            "host-tool attestation binding inputs must not be empty".to_string(),
        ));
    }
    let expected_subject = format!("artifact:{expected_logical_path}");
    if attestation.facts.logical_path != expected_logical_path || attestation.subject_node_id != expected_subject {
        return Err(RustSourceProviderError::Validate(format!(
            "host-tool attestation subject does not match {expected_logical_path}"
        )));
    }
    let recipe_ids = attestation
        .edges
        .iter()
        .filter(|edge| edge.from_node_id == expected_subject && edge.kind == EdgeKind::ProducedBy)
        .map(|edge| edge.to_node_id.as_str())
        .collect::<Vec<_>>();
    if recipe_ids.len() != 1 {
        return Err(RustSourceProviderError::Validate(format!(
            "host-tool attestation must bind one producing recipe, got {}",
            recipe_ids.len()
        )));
    }
    let source_node_ids = attestation
        .nodes
        .iter()
        .filter(|node| node.node_id.rsplit('/').next().is_some_and(|basename| basename.ends_with(source_name)))
        .map(|node| node.node_id.as_str())
        .collect::<Vec<_>>();
    if source_node_ids.len() != 1 {
        let node_ids = attestation.nodes.iter().map(|node| node.node_id.as_str()).collect::<Vec<_>>().join(", ");
        return Err(RustSourceProviderError::Validate(format!(
            "host-tool attestation must bind one {source_name} source artifact, got {}; nodes: {node_ids}",
            source_node_ids.len()
        )));
    }
    let has_source_edge = attestation.edges.iter().any(|edge| {
        edge.from_node_id == recipe_ids[0] && edge.kind == EdgeKind::BuildInput && edge.to_node_id == source_node_ids[0]
    });
    if !has_source_edge {
        return Err(RustSourceProviderError::Validate(format!(
            "host-tool attestation producing recipe does not bind {source_name} as a build input"
        )));
    }
    assert_eq!(recipe_ids.len(), 1);
    debug_assert_eq!(source_node_ids.len(), 1);
    Ok(())
}

fn observe_host_dependency_attestation(
    root: &Path,
    attestation_path: &Path,
    source_name: &str,
    marker_relative_path: &str,
) -> Result<(String, Vec<u8>), RustSourceProviderError> {
    let metadata = fs::symlink_metadata(root)
        .map_err(|error| RustSourceProviderError::Read(format!("host dependency root {}: {error}", root.display())))?;
    if !metadata.file_type().is_dir() {
        return Err(RustSourceProviderError::Validate(format!(
            "host dependency root must be a directory without symlink traversal: {}",
            root.display()
        )));
    }
    validate_host_tool_marker(&root.join(marker_relative_path))?;
    let bytes = read_nonempty_bounded(
        attestation_path,
        "full-source Rust host dependency artifact attestation",
        HOST_TOOL_ATTESTATION_BYTES_MAX,
    )?;
    let (attestation, digest) = parse_canonical_artifact_attestation(&bytes).map_err(|error| {
        RustSourceProviderError::Parse(format!("host dependency attestation {}: {error}", attestation_path.display()))
    })?;
    let root_name = root.file_name().and_then(|name| name.to_str()).ok_or_else(|| {
        RustSourceProviderError::Validate(format!("host dependency root has no UTF-8 basename: {}", root.display()))
    })?;
    let expected_logical_path = format!("{FULL_SOURCE_LOGICAL_STORE_PREFIX}/{root_name}");
    validate_host_tool_attestation_binding(&attestation, &expected_logical_path, source_name)?;
    assert_eq!(digest.len(), BLAKE3_HEX_LENGTH);
    debug_assert!(!bytes.is_empty());
    Ok((digest, bytes))
}

fn observe_full_source_rust_host_tool(
    spec: FullSourceRustHostToolSpec,
    root: &Path,
    attestation_path: &Path,
) -> Result<ObservedFullSourceRustHostTool, RustSourceProviderError> {
    let metadata = fs::symlink_metadata(root)
        .map_err(|error| RustSourceProviderError::Read(format!("host-tool root {}: {error}", root.display())))?;
    if !metadata.file_type().is_dir() {
        return Err(RustSourceProviderError::Validate(format!(
            "host-tool root must be a directory without symlink traversal: {}",
            root.display()
        )));
    }
    let executable_path = root.join(spec.executable_relative_path);
    validate_regular_executable(&executable_path)?;
    validate_host_tool_marker(&root.join(spec.marker_relative_path))?;
    let executable_digest_blake3 = hash_bounded_file(&executable_path)?;
    let attestation_bytes = read_nonempty_bounded(
        attestation_path,
        "full-source Rust host-tool artifact attestation",
        HOST_TOOL_ATTESTATION_BYTES_MAX,
    )?;
    let (attestation, attestation_digest_blake3) =
        parse_canonical_artifact_attestation(&attestation_bytes).map_err(|error| {
            RustSourceProviderError::Parse(format!("host-tool attestation {}: {error}", attestation_path.display()))
        })?;
    let root_name = root.file_name().and_then(|name| name.to_str()).ok_or_else(|| {
        RustSourceProviderError::Validate(format!("host-tool root has no UTF-8 basename: {}", root.display()))
    })?;
    let expected_logical_path = format!("{FULL_SOURCE_LOGICAL_STORE_PREFIX}/{root_name}");
    validate_host_tool_attestation_binding(&attestation, &expected_logical_path, spec.source_name)?;
    Ok(ObservedFullSourceRustHostTool {
        spec,
        executable_path,
        executable_digest_blake3,
        attestation_path: attestation_path.to_path_buf(),
        attestation_digest_blake3,
        attestation_file_digest_blake3: blake3::hash(&attestation_bytes).to_hex().to_string(),
    })
}

fn parse_canonical_artifact_attestation(bytes: &[u8]) -> Result<(ArtifactAttestation, String), String> {
    assert!(!bytes.is_empty(), "artifact attestation bytes must not be empty");
    let attestation = serde_json::from_slice::<ArtifactAttestation>(bytes)
        .map_err(|error| format!("parsing canonical artifact attestation: {error}"))?;
    let canonical_bytes = attestation
        .canonical_bytes()
        .map_err(|error| format!("canonicalizing artifact attestation: {error}"))?;
    if canonical_bytes != bytes {
        return Err("artifact attestation bytes are not canonical".to_string());
    }
    let digest = attestation
        .canonical_digest()
        .map_err(|error| format!("digesting canonical artifact attestation: {error}"))?
        .to_hex();
    if digest.len() != BLAKE3_HEX_LENGTH {
        return Err("artifact attestation canonical digest has invalid length".to_string());
    }
    debug_assert_eq!(digest, blake3::hash(&canonical_bytes).to_hex().to_string());
    Ok((attestation, digest))
}

fn canonical_artifact_attestation_identity(bytes: &[u8]) -> Result<String, String> {
    parse_canonical_artifact_attestation(bytes).map(|(_, digest)| digest)
}

fn validate_host_tool_marker(path: &Path) -> Result<(), RustSourceProviderError> {
    let marker = fs::read_to_string(path)
        .map_err(|error| RustSourceProviderError::Read(format!("host-tool marker {}: {error}", path.display())))?;
    let has_claim = marker.lines().any(|line| line.starts_with("claim:"));
    let has_positive = marker.lines().any(|line| line.starts_with("positive:"));
    let has_negative = marker.lines().any(|line| line.starts_with("negative:"));
    if has_claim && has_positive && has_negative {
        return Ok(());
    }
    Err(RustSourceProviderError::Validate(format!(
        "host-tool marker lacks claim, positive, or negative evidence: {}",
        path.display()
    )))
}

fn host_tool_dependency_digests(
    role: crate::full_source_rust_binding::FullSourceRustHostToolRole,
    native_provider_digest: &str,
    make_digest: &str,
    busybox_digest: &str,
    linux_headers_digest: &str,
    linux_headers_attestation_digest: &str,
) -> BTreeMap<String, String> {
    let mut dependencies = BTreeMap::from([("native-provider".to_string(), native_provider_digest.to_string())]);
    if role != crate::full_source_rust_binding::FullSourceRustHostToolRole::Make {
        dependencies.insert("make".to_string(), make_digest.to_string());
        dependencies.insert("linux-headers".to_string(), linux_headers_digest.to_string());
        dependencies.insert("linux-headers-attestation".to_string(), linux_headers_attestation_digest.to_string());
    }
    if matches!(
        role,
        crate::full_source_rust_binding::FullSourceRustHostToolRole::Cmake
            | crate::full_source_rust_binding::FullSourceRustHostToolRole::Python
            | crate::full_source_rust_binding::FullSourceRustHostToolRole::Perl
    ) {
        dependencies.insert("busybox".to_string(), busybox_digest.to_string());
    }
    assert!(!dependencies.is_empty());
    debug_assert!(dependencies.values().all(|digest| digest.len() == BLAKE3_HEX_LENGTH));
    dependencies
}

fn write_full_source_rust_host_tool_receipt(
    output_dir: &Path,
    tool: ObservedFullSourceRustHostTool,
    dependency_digests_blake3: BTreeMap<String, String>,
) -> Result<crate::full_source_rust_binding::FullSourceRustHostToolBinding, RustSourceProviderError> {
    let role_label = format!("{:?}", tool.spec.role).to_ascii_lowercase();
    let receipt_path = output_dir.join(format!("{role_label}-construction.json"));
    let native_provider_output_digest_blake3 =
        dependency_digests_blake3.get("native-provider").cloned().ok_or_else(|| {
            RustSourceProviderError::Validate("host-tool native-provider dependency is missing".to_string())
        })?;
    let receipt = FullSourceRustHostToolConstructionReceipt {
        schema: crate::full_source_rust_binding::FULL_SOURCE_RUST_HOST_TOOL_RECEIPT_SCHEMA.to_string(),
        receipt_id: format!("full-source-rust-host-{role_label}-construction"),
        role: tool.spec.role,
        source_policy: FULL_SOURCE_SOURCE_POLICY.to_string(),
        source_id: tool.spec.source_id.to_string(),
        source_url: tool.spec.source_url.to_string(),
        source_sha256_hex: tool.spec.source_sha256_hex.to_string(),
        native_provider_output_digest_blake3,
        executable_path: tool.executable_path.display().to_string(),
        executable_digest_blake3: tool.executable_digest_blake3.clone(),
        artifact_attestation_path: tool.attestation_path.display().to_string(),
        artifact_attestation_digest_blake3: tool.attestation_digest_blake3,
        artifact_attestation_file_digest_blake3: tool.attestation_file_digest_blake3,
        positive_checks: tool.spec.positive_checks.iter().map(|check| (*check).to_string()).collect(),
        rejection_checks: tool.spec.rejection_checks.iter().map(|check| (*check).to_string()).collect(),
        dependency_digests_blake3,
        ambient_tool_discovery: false,
        fallback_events: Vec::new(),
        environmental_assumptions: vec![
            "sandbox-orchestration:declared-bwrap-and-builder-shell-outside-installed-host-tool-claim".to_string(),
        ],
    };
    let mut receipt_bytes = serde_json::to_vec_pretty(&receipt)
        .map_err(|error| RustSourceProviderError::Parse(format!("serializing {role_label} receipt: {error}")))?;
    receipt_bytes.push(b'\n');
    let binding = crate::full_source_rust_binding::FullSourceRustHostToolBinding {
        role: tool.spec.role,
        path: tool.executable_path.display().to_string(),
        content_digest_blake3: tool.executable_digest_blake3,
        source_id: tool.spec.source_id.to_string(),
        construction_receipt_path: receipt_path.display().to_string(),
        construction_receipt_digest_blake3: blake3::hash(&receipt_bytes).to_hex().to_string(),
    };
    validate_full_source_rust_host_tool_construction_receipt(
        &binding,
        &receipt,
        &receipt.native_provider_output_digest_blake3,
    )
    .map_err(|error| RustSourceProviderError::Validate(error.to_string()))?;
    write_create_new(&receipt_path, &receipt_bytes)?;
    Ok(binding)
}

pub(crate) fn bind_full_source_rust_provider_candidate(
    provider_candidate_dir: &Path,
    admission_report_path: &Path,
    host_tool_manifest_path: &Path,
) -> Result<FullSourceRustBindingPublication, RustSourceProviderError> {
    let report_bytes = read_nonempty(admission_report_path, "full-source admission report")?;
    let parsed_report = parse_admission_report(admission_report_path, &report_bytes)?;
    let revalidated_report = revalidate_admission_report(&parsed_report)?;
    validate_report_matches_revalidation(&parsed_report, &revalidated_report)?;
    let admission = admission_identity(&revalidated_report, &report_bytes)?;
    let native_artifacts = observe_native_artifacts(&revalidated_report.provider_path)?;
    let host_tools = observe_full_source_rust_host_tools(host_tool_manifest_path, &admission.output_digest_blake3)?;
    let provider = validate_materialized_rust_source_provider(provider_candidate_dir)?;
    let observed_receipts = observed_provider_receipts(provider_candidate_dir, &provider.metadata)?;
    let binding = binding_receipt(
        &provider.metadata,
        &provider.validation.policy_digest_blake3,
        admission.clone(),
        native_artifacts.clone(),
        &host_tools,
    );
    let validation = validate_full_source_rust_provider_binding(
        &provider.metadata,
        &observed_receipts,
        &binding,
        &admission,
        &native_artifacts,
        &host_tools.manifest,
    )
    .map_err(|error| RustSourceProviderError::Validate(error.to_string()))?;
    let canonical_bytes = canonical_full_source_rust_binding_bytes(&binding)
        .map_err(|error| RustSourceProviderError::Validate(error.to_string()))?;
    let content_digest_blake3 = blake3::hash(&canonical_bytes).to_hex().to_string();
    if content_digest_blake3 != validation.binding_receipt_digest_blake3 {
        return Err(RustSourceProviderError::Digest(
            "canonical full-source binding digest changed between validation and publication".to_string(),
        ));
    }
    let path = provider_candidate_dir.join(FULL_SOURCE_RUST_BINDING_RELATIVE_PATH);
    write_create_new(&path, &canonical_bytes)?;
    Ok(FullSourceRustBindingPublication {
        path,
        content_digest_blake3,
        native_artifact_count: validation.native_artifact_count,
    })
}

fn parse_admission_report(
    path: &Path,
    bytes: &[u8],
) -> Result<FullSourceProviderAdmissionReport, RustSourceProviderError> {
    serde_json::from_slice::<FullSourceProviderAdmissionReport>(bytes).map_err(|error| {
        RustSourceProviderError::Parse(format!("full-source admission report {}: {error}", path.display()))
    })
}

fn observe_full_source_rust_host_tools(
    manifest_path: &Path,
    native_provider_output_digest_blake3: &str,
) -> Result<FullSourceRustHostToolObservation, RustSourceProviderError> {
    let bytes =
        read_nonempty_bounded(manifest_path, "full-source Rust host-tool manifest", HOST_TOOL_MANIFEST_BYTES_MAX)?;
    let manifest = serde_json::from_slice::<FullSourceRustHostToolManifest>(&bytes).map_err(|error| {
        RustSourceProviderError::Parse(format!(
            "full-source Rust host-tool manifest {}: {error}",
            manifest_path.display()
        ))
    })?;
    let manifest_digest_blake3 = validate_full_source_rust_host_tool_manifest(&manifest)
        .map_err(|error| RustSourceProviderError::Validate(error.to_string()))?;
    for tool in &manifest.tools {
        validate_observed_host_tool(tool, native_provider_output_digest_blake3)?;
    }
    for support_input in &manifest.support_inputs {
        validate_observed_host_support_input(support_input)?;
    }
    Ok(FullSourceRustHostToolObservation {
        manifest,
        manifest_digest_blake3,
    })
}

fn validate_observed_host_support_input(
    input: &FullSourceRustHostSupportInputBinding,
) -> Result<(), RustSourceProviderError> {
    if input.id != LINUX_HEADERS_SUPPORT_INPUT_ID {
        return Err(RustSourceProviderError::Validate(format!(
            "unsupported full-source Rust host support input '{}'",
            input.id
        )));
    }
    if input.source_id != LINUX_HEADERS_SOURCE_ID {
        return Err(RustSourceProviderError::Validate(format!(
            "Linux header support source substitution: expected {LINUX_HEADERS_SOURCE_ID}, got {}",
            input.source_id
        )));
    }
    let root = Path::new(&input.path);
    let (_, observed_content_digest) = crate::release_tree_copy::hash_directory_tree(root).map_err(|error| {
        RustSourceProviderError::Digest(format!("hashing Linux header support input {}: {error}", root.display()))
    })?;
    if observed_content_digest != input.content_digest_blake3 {
        return Err(RustSourceProviderError::Digest(format!(
            "Linux header support input digest expected {}, got {}",
            input.content_digest_blake3, observed_content_digest
        )));
    }
    for relative_path in LINUX_HEADERS_REQUIRED_RELATIVE_PATHS {
        let path = root.join(relative_path);
        if !path.is_file() {
            return Err(RustSourceProviderError::Validate(format!(
                "Linux header support input is missing required file {}",
                path.display()
            )));
        }
    }
    let (observed_attestation_digest, _) = observe_host_dependency_attestation(
        root,
        Path::new(&input.attestation_path),
        LINUX_HEADERS_SOURCE_NAME,
        LINUX_HEADERS_MARKER_RELATIVE_PATH,
    )?;
    if observed_attestation_digest != input.attestation_digest_blake3 {
        return Err(RustSourceProviderError::Digest(format!(
            "Linux header support attestation digest expected {}, got {}",
            input.attestation_digest_blake3, observed_attestation_digest
        )));
    }
    Ok(())
}

fn validate_observed_host_tool(
    tool: &crate::full_source_rust_binding::FullSourceRustHostToolBinding,
    native_provider_output_digest_blake3: &str,
) -> Result<(), RustSourceProviderError> {
    let tool_path = Path::new(&tool.path);
    validate_regular_executable(tool_path)?;
    let observed_tool_digest = hash_bounded_file(tool_path)?;
    if observed_tool_digest != tool.content_digest_blake3 {
        return Err(RustSourceProviderError::Digest(format!(
            "full-source host-tool {:?} digest expected {}, got {}",
            tool.role, tool.content_digest_blake3, observed_tool_digest
        )));
    }
    let receipt_path = Path::new(&tool.construction_receipt_path);
    let receipt_bytes = read_nonempty_bounded(
        receipt_path,
        "full-source Rust host-tool construction receipt",
        HOST_TOOL_RECEIPT_BYTES_MAX,
    )?;
    let observed_receipt_digest = blake3::hash(&receipt_bytes).to_hex().to_string();
    if observed_receipt_digest != tool.construction_receipt_digest_blake3 {
        return Err(RustSourceProviderError::Digest(format!(
            "full-source host-tool {:?} construction receipt digest expected {}, got {}",
            tool.role, tool.construction_receipt_digest_blake3, observed_receipt_digest
        )));
    }
    let receipt =
        serde_json::from_slice::<FullSourceRustHostToolConstructionReceipt>(&receipt_bytes).map_err(|error| {
            RustSourceProviderError::Parse(format!(
                "full-source Rust host-tool construction receipt {}: {error}",
                receipt_path.display()
            ))
        })?;
    validate_full_source_rust_host_tool_construction_receipt(tool, &receipt, native_provider_output_digest_blake3)
        .map_err(|error| RustSourceProviderError::Validate(error.to_string()))?;
    validate_observed_host_tool_attestation(&receipt)
}

fn validate_observed_host_tool_attestation(
    receipt: &FullSourceRustHostToolConstructionReceipt,
) -> Result<(), RustSourceProviderError> {
    let path = Path::new(&receipt.artifact_attestation_path);
    let bytes = read_nonempty_bounded(
        path,
        "full-source Rust host-tool artifact attestation",
        HOST_TOOL_ATTESTATION_BYTES_MAX,
    )?;
    let observed_file_digest = blake3::hash(&bytes).to_hex().to_string();
    if observed_file_digest != receipt.artifact_attestation_file_digest_blake3 {
        return Err(RustSourceProviderError::Digest(format!(
            "host-tool artifact attestation file digest expected {}, got {}",
            receipt.artifact_attestation_file_digest_blake3, observed_file_digest
        )));
    }
    let observed_identity = canonical_artifact_attestation_identity(&bytes).map_err(|error| {
        RustSourceProviderError::Parse(format!(
            "full-source host-tool artifact attestation {}: {error}",
            path.display()
        ))
    })?;
    if observed_identity != receipt.artifact_attestation_digest_blake3 {
        return Err(RustSourceProviderError::Digest(format!(
            "host-tool artifact attestation identity expected {}, got {}",
            receipt.artifact_attestation_digest_blake3, observed_identity
        )));
    }
    Ok(())
}

fn validate_regular_executable(path: &Path) -> Result<(), RustSourceProviderError> {
    let metadata = fs::symlink_metadata(path)
        .map_err(|error| RustSourceProviderError::Read(format!("host-tool {}: {error}", path.display())))?;
    if !metadata.file_type().is_file() || metadata.len() == 0 {
        return Err(RustSourceProviderError::MissingArtifact(format!(
            "host-tool must be a nonempty regular file: {}",
            path.display()
        )));
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if metadata.permissions().mode() & EXECUTABLE_PERMISSION_MASK == 0 {
            return Err(RustSourceProviderError::MissingArtifact(format!(
                "host-tool is not executable: {}",
                path.display()
            )));
        }
    }
    let mut file = File::open(path)
        .map_err(|error| RustSourceProviderError::Read(format!("host-tool {}: {error}", path.display())))?;
    let mut magic = [0u8; ELF_MAGIC.len()];
    file.read_exact(&mut magic)
        .map_err(|error| RustSourceProviderError::Read(format!("host-tool ELF header {}: {error}", path.display())))?;
    if &magic != ELF_MAGIC {
        return Err(RustSourceProviderError::Validate(format!(
            "host-tool must be a compiled ELF executable, not a delegating script or wrapper: {}",
            path.display()
        )));
    }
    Ok(())
}

fn revalidate_admission_report(
    report: &FullSourceProviderAdmissionReport,
) -> Result<FullSourceProviderAdmissionReport, RustSourceProviderError> {
    admit_full_source_provider(
        &report.provider_path,
        &report.expected_output_digest_blake3,
        &report.source_closure_path,
        &report.expected_source_closure_manifest_blake3,
    )
    .map_err(|error| RustSourceProviderError::Validate(format!("revalidating full-source admission: {error}")))
}

fn validate_report_matches_revalidation(
    parsed: &FullSourceProviderAdmissionReport,
    revalidated: &FullSourceProviderAdmissionReport,
) -> Result<(), RustSourceProviderError> {
    let parsed_facts = admission_report_facts(parsed);
    let revalidated_facts = admission_report_facts(revalidated);
    if parsed_facts == revalidated_facts {
        return Ok(());
    }
    Err(RustSourceProviderError::Validate(
        "full-source admission report does not match independent revalidation".to_string(),
    ))
}

fn admission_report_facts(
    report: &FullSourceProviderAdmissionReport,
) -> (&str, &str, &str, &Path, &str, &str, &str, &Path, &str, &str, usize) {
    (
        &report.schema,
        &report.status,
        &report.provider_id,
        &report.provider_path,
        &report.metadata_digest_blake3,
        &report.output_digest_blake3,
        &report.expected_output_digest_blake3,
        &report.source_closure_path,
        &report.source_closure_manifest_blake3,
        &report.expected_source_closure_manifest_blake3,
        report.source_closure_record_count,
    )
}

fn admission_identity(
    report: &FullSourceProviderAdmissionReport,
    report_bytes: &[u8],
) -> Result<FullSourceNativeProviderAdmissionIdentity, RustSourceProviderError> {
    let source_closure_record_count = u32::try_from(report.source_closure_record_count)
        .map_err(|_| RustSourceProviderError::Validate("full-source admission record count exceeds u32".to_string()))?;
    Ok(FullSourceNativeProviderAdmissionIdentity {
        schema: report.schema.clone(),
        status: report.status.clone(),
        provider_id: report.provider_id.clone(),
        provider_target: FULL_SOURCE_PROVIDER_TARGET.to_string(),
        compiler_target: FULL_SOURCE_COMPILER_TARGET.to_string(),
        admission_report_digest_blake3: blake3::hash(report_bytes).to_hex().to_string(),
        metadata_digest_blake3: report.metadata_digest_blake3.clone(),
        output_digest_blake3: report.output_digest_blake3.clone(),
        expected_output_digest_blake3: report.expected_output_digest_blake3.clone(),
        source_closure_manifest_blake3: report.source_closure_manifest_blake3.clone(),
        expected_source_closure_manifest_blake3: report.expected_source_closure_manifest_blake3.clone(),
        source_closure_record_count,
    })
}

fn observe_native_artifacts(
    provider_dir: &Path,
) -> Result<Vec<FullSourceNativeArtifactBinding>, RustSourceProviderError> {
    let required = required_full_source_native_artifacts();
    let mut observed = Vec::with_capacity(required.len());
    for (relative_path, role) in required {
        let path = provider_dir.join(relative_path);
        observed.push(FullSourceNativeArtifactBinding {
            role: *role,
            path: (*relative_path).to_string(),
            content_digest_blake3: hash_bounded_file(&path)?,
        });
    }
    if observed.len() != required.len() {
        return Err(RustSourceProviderError::Validate(
            "native artifact observation count changed unexpectedly".to_string(),
        ));
    }
    Ok(observed)
}

fn binding_receipt(
    metadata: &RustSourceProviderMetadata,
    policy_digest_blake3: &str,
    native_provider: FullSourceNativeProviderAdmissionIdentity,
    native_artifacts: Vec<FullSourceNativeArtifactBinding>,
    host_tools: &FullSourceRustHostToolObservation,
) -> FullSourceRustProviderBindingReceipt {
    FullSourceRustProviderBindingReceipt {
        schema: FULL_SOURCE_RUST_BINDING_SCHEMA.to_string(),
        receipt_id: FULL_SOURCE_BINDING_RECEIPT_ID.to_string(),
        rust_provider_id: metadata.provider_id.clone(),
        host_triple: metadata.host_triple.clone(),
        target_triple: metadata.target_triple.clone(),
        source_policy: FULL_SOURCE_SOURCE_POLICY.to_string(),
        ambient_tool_discovery: false,
        rust_provider_policy_digest_blake3: policy_digest_blake3.to_string(),
        host_tool_manifest_digest_blake3: host_tools.manifest_digest_blake3.clone(),
        host_tools: host_tools.manifest.tools.clone(),
        host_support_inputs: host_tools.manifest.support_inputs.clone(),
        native_provider,
        native_artifacts,
        rust_source_ids: metadata.sources.iter().map(|source| source.id.clone()).collect(),
        rust_build_receipts: metadata
            .build_receipts
            .iter()
            .map(|receipt| FullSourceRustReceiptBinding {
                id: receipt.id.clone(),
                kind: receipt.kind,
                name: receipt.name.clone(),
                path: receipt.path.clone(),
                digest_blake3: receipt.digest_blake3.clone(),
            })
            .collect(),
        rust_artifacts: metadata
            .artifacts
            .iter()
            .map(|artifact| FullSourceRustArtifactBinding {
                role: artifact.role,
                name: artifact.name.clone(),
                path: artifact.path.clone(),
                content_digest_blake3: artifact.content_digest_blake3.clone(),
                source_id: artifact.source_id.clone(),
                build_receipt_id: artifact.build_receipt_id.clone(),
            })
            .collect(),
        fallback_events: Vec::new(),
        seed_exceptions: Vec::new(),
    }
}

fn hash_bounded_file(path: &Path) -> Result<String, RustSourceProviderError> {
    let file = File::open(path)
        .map_err(|error| RustSourceProviderError::Read(format!("native artifact {}: {error}", path.display())))?;
    let metadata = file.metadata().map_err(|error| {
        RustSourceProviderError::Read(format!("native artifact metadata {}: {error}", path.display()))
    })?;
    if !metadata.is_file() || metadata.len() == 0 || metadata.len() > NATIVE_ARTIFACT_BYTES_MAX {
        return Err(RustSourceProviderError::MissingArtifact(format!(
            "native artifact is empty, non-file, or over limit: {}",
            path.display()
        )));
    }
    let mut reader = BufReader::with_capacity(HASH_BUFFER_BYTES, file);
    let mut buffer = [0u8; HASH_BUFFER_BYTES];
    let mut total_bytes = 0u64;
    let mut hasher = blake3::Hasher::new();
    loop {
        let read_bytes = reader
            .read(&mut buffer)
            .map_err(|error| RustSourceProviderError::Read(format!("native artifact {}: {error}", path.display())))?;
        if read_bytes == 0 {
            break;
        }
        total_bytes =
            total_bytes
                .checked_add(u64::try_from(read_bytes).map_err(|_| {
                    RustSourceProviderError::Digest("native artifact read length exceeds u64".to_string())
                })?)
                .ok_or_else(|| RustSourceProviderError::Digest("native artifact size overflow".to_string()))?;
        if total_bytes > NATIVE_ARTIFACT_BYTES_MAX {
            return Err(RustSourceProviderError::Digest(format!(
                "native artifact exceeded byte limit while hashing: {}",
                path.display()
            )));
        }
        hasher.update(&buffer[..read_bytes]);
    }
    if total_bytes != metadata.len() {
        return Err(RustSourceProviderError::Digest(format!(
            "native artifact size changed while hashing: {}",
            path.display()
        )));
    }
    Ok(hasher.finalize().to_hex().to_string())
}

fn read_nonempty(path: &Path, label: &str) -> Result<Vec<u8>, RustSourceProviderError> {
    let bytes = fs::read(path)
        .map_err(|error| RustSourceProviderError::Read(format!("{label} {}: {error}", path.display())))?;
    if bytes.is_empty() {
        return Err(RustSourceProviderError::Read(format!("{label} {} is empty", path.display())));
    }
    Ok(bytes)
}

fn read_nonempty_bounded(path: &Path, label: &str, byte_limit: u64) -> Result<Vec<u8>, RustSourceProviderError> {
    let metadata = fs::symlink_metadata(path)
        .map_err(|error| RustSourceProviderError::Read(format!("{label} metadata {}: {error}", path.display())))?;
    if !metadata.is_file() || metadata.len() == 0 || metadata.len() > byte_limit {
        return Err(RustSourceProviderError::Read(format!(
            "{label} {} is empty, non-file, or over its byte limit",
            path.display()
        )));
    }
    read_nonempty(path, label)
}

fn write_create_new(path: &Path, bytes: &[u8]) -> Result<(), RustSourceProviderError> {
    if bytes.is_empty() {
        return Err(RustSourceProviderError::Copy(
            "refusing to publish an empty full-source binding receipt".to_string(),
        ));
    }
    let parent = path
        .parent()
        .ok_or_else(|| RustSourceProviderError::Copy(format!("binding path {} has no parent", path.display())))?;
    fs::create_dir_all(parent)
        .map_err(|error| RustSourceProviderError::Copy(format!("create {}: {error}", parent.display())))?;
    let mut candidate = tempfile::NamedTempFile::new_in(parent)
        .map_err(|error| RustSourceProviderError::Copy(format!("create candidate in {}: {error}", parent.display())))?;
    candidate
        .write_all(bytes)
        .map_err(|error| RustSourceProviderError::Copy(format!("write candidate for {}: {error}", path.display())))?;
    candidate
        .as_file()
        .sync_all()
        .map_err(|error| RustSourceProviderError::Copy(format!("sync candidate for {}: {error}", path.display())))?;
    candidate
        .persist_noclobber(path)
        .map_err(|error| RustSourceProviderError::Copy(format!("create-new {}: {error}", path.display())))?;
    let digest = blake3::hash(bytes).to_hex().to_string();
    if digest.len() != BLAKE3_HEX_LENGTH {
        return Err(RustSourceProviderError::Digest(
            "published full-source binding digest has invalid length".to_string(),
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_BYTES: &[u8] = b"full-source-native-artifact";
    #[cfg(unix)]
    const TEST_EXECUTABLE_MODE: u32 = 0o755;

    #[test]
    fn host_tool_marker_accepts_positive_and_negative_evidence() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("host-tool.txt");
        fs::write(&path, "claim: bounded\npositive: runs\nnegative: rejects malformed input\n").unwrap();

        let result = validate_host_tool_marker(&path);

        assert!(result.is_ok());
        assert!(path.is_file());
    }

    #[test]
    fn host_tool_marker_rejects_missing_negative_evidence() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("host-tool.txt");
        fs::write(&path, "claim: bounded\npositive: runs\n").unwrap();

        let error = validate_host_tool_marker(&path).unwrap_err().to_string();

        assert!(error.contains("lacks claim, positive, or negative evidence"));
        assert!(error.contains("host-tool.txt"));
    }

    #[test]
    fn bounded_file_hash_accepts_nonempty_file() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("artifact");
        fs::write(&path, TEST_BYTES).unwrap();

        let digest = hash_bounded_file(&path).unwrap();

        assert_eq!(digest, blake3::hash(TEST_BYTES).to_hex().to_string());
        assert_eq!(digest.len(), BLAKE3_HEX_LENGTH);
    }

    #[test]
    fn bounded_file_hash_rejects_empty_file() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("artifact");
        fs::write(&path, []).unwrap();

        let error = hash_bounded_file(&path).unwrap_err();

        assert!(error.to_string().contains("empty"));
        assert!(!error.to_string().contains("digest mismatch"));
    }

    #[cfg(unix)]
    #[test]
    fn regular_executable_validation_accepts_compiled_elf_shape() {
        use std::os::unix::fs::PermissionsExt;

        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("tool");
        fs::write(&path, b"\x7fELFbound-tool").unwrap();
        fs::set_permissions(&path, fs::Permissions::from_mode(TEST_EXECUTABLE_MODE)).unwrap();

        validate_regular_executable(&path).unwrap();

        assert!(path.is_file());
        assert_ne!(fs::metadata(&path).unwrap().permissions().mode() & EXECUTABLE_PERMISSION_MASK, 0);
    }

    #[cfg(unix)]
    #[test]
    fn regular_executable_validation_rejects_delegating_script() {
        use std::os::unix::fs::PermissionsExt;

        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("tool");
        fs::write(&path, b"#!/bin/sh\nexec /ambient/tool \"$@\"\n").unwrap();
        fs::set_permissions(&path, fs::Permissions::from_mode(TEST_EXECUTABLE_MODE)).unwrap();

        let error = validate_regular_executable(&path).unwrap_err();

        assert!(error.to_string().contains("not a delegating script or wrapper"));
        assert!(!error.to_string().contains("not executable"));
    }

    #[test]
    fn host_tool_attestation_observation_accepts_canonical_store_attestation() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("artifact.json");
        let bytes = canonical_artifact_attestation_bytes();
        let identity = canonical_artifact_attestation_identity(&bytes).unwrap();
        fs::write(&path, &bytes).unwrap();
        let receipt = host_tool_attestation_receipt(&path, &bytes, &identity);

        validate_observed_host_tool_attestation(&receipt).unwrap();

        assert_eq!(receipt.artifact_attestation_digest_blake3, identity);
        assert_eq!(receipt.artifact_attestation_file_digest_blake3, blake3::hash(&bytes).to_hex().to_string());
    }

    #[test]
    fn host_tool_attestation_observation_rejects_identity_substitution() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("artifact.json");
        let bytes = canonical_artifact_attestation_bytes();
        let identity = canonical_artifact_attestation_identity(&bytes).unwrap();
        fs::write(&path, &bytes).unwrap();
        let mut receipt = host_tool_attestation_receipt(&path, &bytes, &identity);
        receipt.artifact_attestation_digest_blake3 = "e".repeat(BLAKE3_HEX_LENGTH);

        let error = validate_observed_host_tool_attestation(&receipt).unwrap_err();

        assert!(error.to_string().contains("attestation identity expected"));
        assert!(!error.to_string().contains("file digest expected"));
    }

    #[test]
    fn host_tool_attestation_binding_accepts_matching_artifact_and_source_lineage() {
        let attestation = host_tool_binding_attestation();

        validate_host_tool_attestation_binding(&attestation, "/mantle/store/test-host-tool", "perl-5.10.1-src")
            .unwrap();

        assert_eq!(attestation.facts.logical_path, "/mantle/store/test-host-tool");
        assert!(attestation.nodes.iter().any(|node| node.node_id.ends_with("perl-5.10.1-src")));
    }

    #[test]
    fn host_tool_attestation_binding_rejects_identity_and_source_substitution() {
        let attestation = host_tool_binding_attestation();

        let identity_error =
            validate_host_tool_attestation_binding(&attestation, "/mantle/store/other-host-tool", "perl-5.10.1-src")
                .unwrap_err();
        let source_error =
            validate_host_tool_attestation_binding(&attestation, "/mantle/store/test-host-tool", "perl-5.6.2-src")
                .unwrap_err();

        assert!(identity_error.to_string().contains("subject does not match"));
        assert!(source_error.to_string().contains("one perl-5.6.2-src source artifact"));
    }

    #[test]
    fn source_archive_hex_hash_accepts_bytes_and_rejects_malformed_hex() {
        const ABC_SHA256: &str = "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad";

        let digest = sha256_content_hex("616263").unwrap();
        let error = sha256_content_hex("6z").unwrap_err();

        assert_eq!(digest, ABC_SHA256);
        assert!(error.to_string().contains("non-hex byte"));
    }

    #[test]
    fn source_archive_validation_accepts_contiguous_chunks_and_rejects_duplicate_index() {
        const ABC_SHA256: &str = "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad";
        const TEST_CHUNK_COUNT: u32 = 2;
        const TEST_PAYLOAD_BYTES: u64 = 3;
        const TEST_SECOND_CHUNK_BYTES: u64 = 2;
        let mut chunks = vec![
            source_archive_chunk("61", 1, 0, TEST_CHUNK_COUNT),
            source_archive_chunk("6263", TEST_SECOND_CHUNK_BYTES, 1, TEST_CHUNK_COUNT),
        ];

        let digest = validate_source_archive_entries("test-source", &chunks, TEST_PAYLOAD_BYTES).unwrap();
        chunks[1].chunk_index = Some(0);
        let error = validate_source_archive_entries("test-source", &chunks, TEST_PAYLOAD_BYTES).unwrap_err();

        assert_eq!(digest, ABC_SHA256);
        assert!(error.to_string().contains("chunk sequence is not contiguous"));
    }

    fn source_archive_chunk(
        content_hex: &str,
        size: u64,
        chunk_index: u32,
        chunk_count: u32,
    ) -> crate::source_bundle::SourceFileEntry {
        crate::source_bundle::SourceFileEntry {
            path: SOURCE_ARCHIVE_PATH.to_string(),
            file_type: crate::source_bundle::SourceFileType::Regular,
            executable: false,
            size,
            content_hex: Some(content_hex.to_string()),
            symlink_target: None,
            chunk_index: Some(chunk_index),
            chunk_count: Some(chunk_count),
            blake3: blake3::hash(content_hex.as_bytes()).to_hex().to_string(),
        }
    }

    #[test]
    fn host_tool_attestation_observation_rejects_envelope_substitution() {
        let bytes = serde_json::to_vec(&serde_json::json!({ "digest": "d".repeat(BLAKE3_HEX_LENGTH) })).unwrap();

        let error = canonical_artifact_attestation_identity(&bytes).unwrap_err();

        assert!(error.contains("parsing canonical artifact attestation"));
        assert!(!error.contains("canonical digest has invalid length"));
    }

    fn host_tool_binding_attestation() -> ArtifactAttestation {
        let value = serde_json::json!({
            "schema_version": 1,
            "claims": {},
            "facts": {
                "logical_path": "/mantle/store/test-host-tool",
                "output_name": "out",
                "content_digest": "nar-sha256:0123456789abcdef"
            },
            "subject_node_id": "artifact:/mantle/store/test-host-tool",
            "nodes": [
                {
                    "node_id": "artifact:/mantle/store/test-host-tool",
                    "kind": "artifact",
                    "attributes": { "logical_path": "/mantle/store/test-host-tool" }
                },
                {
                    "node_id": "recipe:/mantle/store/test-host-tool.drv",
                    "kind": "recipe",
                    "attributes": { "logical_path": "/mantle/store/test-host-tool.drv" }
                },
                {
                    "node_id": "artifact:/mantle/store/source-perl-5.10.1-src",
                    "kind": "artifact",
                    "attributes": { "logical_path": "/mantle/store/source-perl-5.10.1-src" }
                }
            ],
            "edges": [
                {
                    "from_node_id": "artifact:/mantle/store/test-host-tool",
                    "kind": "produced-by",
                    "to_node_id": "recipe:/mantle/store/test-host-tool.drv"
                },
                {
                    "from_node_id": "recipe:/mantle/store/test-host-tool.drv",
                    "kind": "build-input",
                    "to_node_id": "artifact:/mantle/store/source-perl-5.10.1-src"
                }
            ]
        });
        serde_json::from_value(value).unwrap()
    }

    fn canonical_artifact_attestation_bytes() -> Vec<u8> {
        let value = serde_json::json!({
            "schema_version": 1,
            "claims": {},
            "facts": {
                "logical_path": "/mantle/store/test-host-tool",
                "output_name": "out",
                "content_digest": "nar-sha256:0123456789abcdef"
            },
            "subject_node_id": "artifact:/mantle/store/test-host-tool",
            "nodes": [{
                "node_id": "artifact:/mantle/store/test-host-tool",
                "kind": "artifact",
                "attributes": { "logical_path": "/mantle/store/test-host-tool" }
            }],
            "edges": []
        });
        let attestation = serde_json::from_value::<ArtifactAttestation>(value).unwrap();
        attestation.canonical_bytes().unwrap()
    }

    fn host_tool_attestation_receipt(
        path: &Path,
        bytes: &[u8],
        identity: &str,
    ) -> FullSourceRustHostToolConstructionReceipt {
        FullSourceRustHostToolConstructionReceipt {
            schema: crate::full_source_rust_binding::FULL_SOURCE_RUST_HOST_TOOL_RECEIPT_SCHEMA.to_string(),
            receipt_id: "test-host-tool-construction".to_string(),
            role: crate::full_source_rust_binding::FullSourceRustHostToolRole::Make,
            source_policy: FULL_SOURCE_SOURCE_POLICY.to_string(),
            source_id: "test-source".to_string(),
            source_url: "https://example.invalid/source.tar.gz".to_string(),
            source_sha256_hex: "a".repeat(BLAKE3_HEX_LENGTH),
            native_provider_output_digest_blake3: "b".repeat(BLAKE3_HEX_LENGTH),
            executable_path: "/test/tool".to_string(),
            executable_digest_blake3: "c".repeat(BLAKE3_HEX_LENGTH),
            artifact_attestation_path: path.display().to_string(),
            artifact_attestation_digest_blake3: identity.to_string(),
            artifact_attestation_file_digest_blake3: blake3::hash(bytes).to_hex().to_string(),
            positive_checks: vec!["version".to_string()],
            rejection_checks: vec!["malformed-input".to_string()],
            dependency_digests_blake3: Default::default(),
            ambient_tool_discovery: false,
            fallback_events: Vec::new(),
            environmental_assumptions: vec!["sandbox-orchestration:/bin/sh".to_string()],
        }
    }

    fn linux_header_support_attestation_bytes(logical_path: &str) -> Vec<u8> {
        let subject_node_id = format!("artifact:{logical_path}");
        let recipe_node_id = format!("recipe:{logical_path}.drv");
        let source_node_id = "artifact:/mantle/store/source-linux-6.6-src";
        let value = serde_json::json!({
            "schema_version": 1,
            "claims": {},
            "facts": {
                "logical_path": logical_path,
                "output_name": "out",
                "content_digest": "nar-sha256:0123456789abcdef"
            },
            "subject_node_id": subject_node_id,
            "nodes": [
                {
                    "node_id": subject_node_id,
                    "kind": "artifact",
                    "attributes": { "logical_path": logical_path }
                },
                {
                    "node_id": recipe_node_id,
                    "kind": "recipe",
                    "attributes": { "logical_path": format!("{logical_path}.drv") }
                },
                {
                    "node_id": source_node_id,
                    "kind": "artifact",
                    "attributes": { "logical_path": "/mantle/store/source-linux-6.6-src" }
                }
            ],
            "edges": [
                {
                    "from_node_id": subject_node_id,
                    "kind": "produced-by",
                    "to_node_id": recipe_node_id
                },
                {
                    "from_node_id": recipe_node_id,
                    "kind": "build-input",
                    "to_node_id": source_node_id
                }
            ]
        });
        serde_json::from_value::<ArtifactAttestation>(value).unwrap().canonical_bytes().unwrap()
    }

    #[test]
    fn linux_header_support_observation_accepts_bound_attestation_and_rejects_digest_substitution() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().join("test-linux-headers");
        for relative_path in LINUX_HEADERS_REQUIRED_RELATIVE_PATHS {
            let path = root.join(relative_path);
            fs::create_dir_all(path.parent().unwrap()).unwrap();
            fs::write(path, TEST_BYTES).unwrap();
        }
        let marker_path = root.join(LINUX_HEADERS_MARKER_RELATIVE_PATH);
        fs::create_dir_all(marker_path.parent().unwrap()).unwrap();
        fs::write(
            marker_path,
            b"claim: test Linux header binding\npositive: required headers present\nnegative: ambient headers absent\n",
        )
        .unwrap();
        let logical_path = format!("{FULL_SOURCE_LOGICAL_STORE_PREFIX}/test-linux-headers");
        let attestation_bytes = linux_header_support_attestation_bytes(&logical_path);
        let attestation_path = dir.path().join("linux-headers-attestation.json");
        fs::write(&attestation_path, &attestation_bytes).unwrap();
        let (_, content_digest_blake3) = crate::release_tree_copy::hash_directory_tree(&root).unwrap();
        let attestation_digest_blake3 = canonical_artifact_attestation_identity(&attestation_bytes).unwrap();
        let mut binding = FullSourceRustHostSupportInputBinding {
            id: LINUX_HEADERS_SUPPORT_INPUT_ID.to_string(),
            path: root.display().to_string(),
            content_digest_blake3,
            source_id: LINUX_HEADERS_SOURCE_ID.to_string(),
            attestation_path: attestation_path.display().to_string(),
            attestation_digest_blake3,
        };

        validate_observed_host_support_input(&binding).unwrap();
        binding.attestation_digest_blake3 = "f".repeat(BLAKE3_HEX_LENGTH);
        let error = validate_observed_host_support_input(&binding).unwrap_err();

        assert!(error.to_string().contains("attestation digest expected"));
        assert!(!error.to_string().contains("not found"));
    }

    #[test]
    fn create_new_publication_refuses_replacement() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("binding.json");
        write_create_new(&path, TEST_BYTES).unwrap();

        let error = write_create_new(&path, b"replacement").unwrap_err();

        assert_eq!(fs::read(&path).unwrap(), TEST_BYTES);
        assert_eq!(fs::read_dir(dir.path()).unwrap().count(), 1);
        assert!(error.to_string().contains("create-new"));
    }

    #[test]
    fn host_tool_observation_rejects_ambient_discovery_manifest() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("host-tools.json");
        let manifest = FullSourceRustHostToolManifest {
            schema: crate::full_source_rust_binding::FULL_SOURCE_RUST_HOST_TOOL_SCHEMA.to_string(),
            source_policy: FULL_SOURCE_SOURCE_POLICY.to_string(),
            ambient_tool_discovery: true,
            tools: Vec::new(),
            support_inputs: Vec::new(),
        };
        fs::write(&path, serde_json::to_vec(&manifest).unwrap()).unwrap();

        let error = observe_full_source_rust_host_tools(&path, &"a".repeat(BLAKE3_HEX_LENGTH)).unwrap_err();
        let rendered = error.to_string();

        assert!(rendered.contains("ambient discovery"));
        assert!(!rendered.contains("not found"));
    }
}
