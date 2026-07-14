use alloc::format;
use alloc::string::String;
use alloc::string::ToString;
use alloc::vec;
use alloc::vec::Vec;

use super::*;

const TEST_OBJECT_SIZE_BYTES: u64 = 1;
const EXPECTED_SOURCE_REQUESTS: usize = 1;
const EXPECTED_COMPOSITION_NODES: usize = 2;
const EXPECTED_REPORT_NODES: usize = 2;
const LOCK_BYTES: &[u8] = b"deterministic-wkg-lock-v1";
const PACKAGE_NAME: &str = "wasi:cli";
const PACKAGE_REQUIREMENT: &str = "=0.2.0";
const PACKAGE_VERSION: &str = "0.2.0";
const REGISTRY_NAME: &str = "wasi.dev";
const RUNTIME_NON_CLAIM: &str = "not-runtime-authority";
const RELEASE_NON_CLAIM: &str = "not-release-eligibility";

fn blake3(seed: char) -> Blake3Identity {
    Blake3Identity::parse(seed.to_string().repeat(BLAKE3_HEX_LENGTH)).unwrap()
}

fn oci_sha(seed: char) -> OciSha256Digest {
    OciSha256Digest::parse(format!("{OCI_SHA256_PREFIX}{}", seed.to_string().repeat(SHA256_HEX_LENGTH))).unwrap()
}

fn object(name: &str, seed: char) -> StoreObject {
    StoreObject {
        logical_path: format!("/mantle/store/{name}"),
        digest_blake3: blake3(seed),
        size_bytes: TEST_OBJECT_SIZE_BYTES,
    }
}

fn tool(name: &str, seed: char) -> ToolIdentity {
    ToolIdentity {
        version: String::from("test-version"),
        executable: object(name, seed),
        configuration_identity_blake3: blake3(seed),
    }
}

fn cohort() -> ToolCohort {
    ToolCohort {
        rust_toolchain: tool("rust", '1'),
        rust_target: String::from("wasm32-wasip2"),
        wkg: tool("wkg", '2'),
        wit_bindgen: tool("wit-bindgen", '2'),
        wasm_component_ld: tool("wasm-component-ld", '3'),
        wasm_tools: tool("wasm-tools", '4'),
        wac: tool("wac", '5'),
        wasi_virt: tool("wasi-virt", '6'),
        wizer: tool("wizer", '7'),
        wasmtime: tool("wasmtime", '8'),
    }
}

fn composition() -> Composition {
    Composition {
        package: String::from("demo:composition"),
        source_wac: String::from("package demo:composition; export app.run;"),
        nodes: vec![
            CompositionNode {
                id: String::from("provider"),
                package: String::from("demo:provider"),
                world: String::from("demo:provider-world"),
                artifact: object("provider.wasm", '9'),
                required_imports: Vec::new(),
            },
            CompositionNode {
                id: String::from("app"),
                package: String::from("demo:app"),
                world: String::from("demo:app-world"),
                artifact: object("app.wasm", 'a'),
                required_imports: vec![RequiredImport {
                    name: String::from("provider"),
                    world: String::from("demo:provider-world"),
                }],
            },
        ],
        edges: vec![CompositionEdge {
            provider_node: String::from("provider"),
            provider_export: String::from("run"),
            consumer_node: String::from("app"),
            consumer_import: String::from("provider"),
            world: String::from("demo:provider-world"),
        }],
        output_world: String::from("demo:app-world"),
        import_dependencies: false,
    }
}

fn virtualization_config() -> VirtualizationConfig {
    VirtualizationConfig {
        defaults_overridden: true,
        rules: vec![
            VirtualizationRule {
                subsystem: WasiSubsystem::Random,
                mode: VirtualizationMode::FixedValue,
                value: Some(String::from("deterministic-random")),
                value_identity_blake3: Some(blake3('b')),
                input: None,
                guest_path: None,
                review_id: None,
            },
            VirtualizationRule {
                subsystem: WasiSubsystem::Filesystem,
                mode: VirtualizationMode::VirtualMount,
                value: None,
                value_identity_blake3: None,
                input: Some(object("virtual-files", 'c')),
                guest_path: Some(String::from("/data")),
                review_id: None,
            },
        ],
        expected_remaining_imports: vec![String::from("wasi:cli/stdout")],
    }
}

fn manifest() -> ComponentManifest {
    ComponentManifest {
        schema: String::from(COMPONENT_MANIFEST_SCHEMA),
        name: String::from("demo-component"),
        package_resolution: PackageResolution {
            default_registry: String::from(REGISTRY_NAME),
            registries: vec![RegistryMapping {
                namespace: String::from("wasi"),
                registry: String::from(REGISTRY_NAME),
                backend: RegistryBackend::Oci,
                oci_registry: Some(String::from("ghcr.io")),
                namespace_prefix: String::from("webassembly/"),
                protocol: OciProtocol::Https,
                local_root: None,
                credential_handle: Some(String::from("secret://registry/ghcr")),
            }],
            requirements: vec![PackageRequirement {
                package: String::from(PACKAGE_NAME),
                requirement: String::from(PACKAGE_REQUIREMENT),
                registry: String::from(REGISTRY_NAME),
                kind: PackageKind::Wit,
            }],
            local_overrides: vec![LocalPackageOverride {
                package: String::from(PACKAGE_NAME),
                path: String::from("/mantle/store/wasi-cli"),
            }],
            lock_path: String::from("wkg.lock"),
        },
        wit: WitSelection {
            package: String::from("demo:app"),
            world: String::from("demo:app-world"),
            source: object("wit", 'd'),
        },
        implementation: RustImplementation {
            source: object("source", 'e'),
            package: String::from("demo"),
            crate_name: String::from("demo"),
            features: vec![String::from("component")],
            profile: RustProfile::Release,
        },
        cohort: cohort(),
        composition: composition(),
        virtualization: virtualization_config(),
        validation_profiles: ValidationProfiles {
            octet_artifact_profile_identity_blake3: blake3('f'),
            expected_runtime_profile_identity_blake3: blake3('0'),
        },
        wizer: WizerConfig {
            mode: WizerMode::Deterministic,
            initialization_entrypoint: Some(String::from("wizer.initialize")),
            deterministic_virtual_imports: vec![String::from("virtual:random")],
        },
        aot: AotConfig {
            mode: AotMode::TrustedNative,
            target: Some(String::from("x86_64-linux")),
            cpu_features: vec![String::from("sse2")],
            wasmtime_configuration_identity_blake3: Some(blake3('1')),
        },
        outputs: vec![OutputDeclaration {
            name: String::from("component"),
            path: String::from("component.wasm"),
            class: OutputClass::ValidatedPortableComponent,
        }],
        non_claims: vec![String::from(RUNTIME_NON_CLAIM), String::from(RELEASE_NON_CLAIM)],
    }
}

fn lock_request() -> LockValidationRequest {
    let manifest = manifest();
    let lock = WkgLock {
        version: WKG_LOCK_VERSION,
        packages: vec![WkgLockPackage {
            name: String::from(PACKAGE_NAME),
            registry: String::from(REGISTRY_NAME),
            versions: vec![WkgLockedVersion {
                requirement: String::from(PACKAGE_REQUIREMENT),
                version: String::from(PACKAGE_VERSION),
                digest: oci_sha('a'),
            }],
        }],
    };
    let facts = LockFacts {
        lock,
        lock_bytes_blake3: Blake3Identity::from_bytes(LOCK_BYTES.to_vec()),
        registry_config_blake3: registry_config_identity(manifest.package_resolution.clone()).unwrap(),
        materializations: vec![PackageMaterialization {
            package: String::from(PACKAGE_NAME),
            version: String::from(PACKAGE_VERSION),
            registry: String::from(REGISTRY_NAME),
            protocol_digest: oci_sha('a'),
            object: object("wasi-cli", '2'),
        }],
    };
    LockValidationRequest {
        manifest,
        facts,
        lock_bytes: LOCK_BYTES.to_vec(),
    }
}

fn portable_admission() -> PortableAdmission {
    let artifact = object("component.wasm", '3');
    bind_portable_admission(PortableAdmissionRequest {
        artifact: artifact.clone(),
        build_validation: BuildValidationBinding {
            artifact_blake3: artifact.digest_blake3.clone(),
            cohort_blake3: blake3('4'),
            report_blake3: blake3('5'),
            decision: ValidationDecision::Pass,
        },
        octet_validation: Some(OctetValidationBinding {
            artifact_blake3: artifact.digest_blake3.clone(),
            profile_blake3: blake3('6'),
            cohort_blake3: blake3('7'),
            report_blake3: blake3('8'),
            decision: ValidationDecision::Pass,
        }),
        octet_required: true,
        expected_octet_profile_blake3: blake3('6'),
        expected_octet_cohort_blake3: blake3('7'),
    })
    .admission
    .unwrap()
}

#[test]
fn digest_roles_preserve_external_sha_and_mantle_blake3() {
    let mantle = parse_role_digest(DigestRole::MantleBlake3, blake3('a').into_hex()).unwrap();
    let external = parse_role_digest(DigestRole::OciSha256, oci_sha('b').into_value()).unwrap();

    assert!(matches!(mantle, RoleDigest::Mantle(_)));
    assert!(matches!(external, RoleDigest::ExternalOci(_)));
}

#[test]
fn digest_roles_reject_sha256_in_mantle_identity_position() {
    let result = parse_role_digest(DigestRole::MantleBlake3, oci_sha('a').into_value());

    assert_eq!(result, Err(DigestError::InvalidBlake3));
    assert!(Blake3Identity::parse(String::from("not-a-digest")).is_err());
}

#[test]
fn valid_manifest_and_cohort_have_stable_blake3_identities() {
    let first = validate_manifest(manifest());
    let second = validate_manifest(manifest());

    assert!(first.blockers.is_empty(), "blockers: {:?}", first.blockers);
    assert_eq!(first.manifest_identity_blake3, second.manifest_identity_blake3);
    assert_eq!(first.cohort_identity_blake3, second.cohort_identity_blake3);
}

#[test]
fn manifest_rejects_non_exact_packages_and_missing_non_claims() {
    let mut invalid = manifest();
    invalid.package_resolution.requirements[0].requirement = String::from("0.2");
    invalid.non_claims.clear();
    let validation = validate_manifest(invalid);

    assert!(validation.manifest_identity_blake3.is_none());
    assert!(validation.blockers.iter().any(|item| item.code == "non-exact-package-requirement"));
    assert!(validation.blockers.iter().any(|item| item.code == "missing-required-non-claim"));
}

#[test]
fn manifest_rejects_unmapped_registry_and_incomplete_optional_stages() {
    let mut invalid = manifest();
    invalid.package_resolution.requirements[0].registry = String::from("unmapped.example");
    invalid.wizer.initialization_entrypoint = None;
    invalid.aot.target = None;
    let validation = validate_manifest(invalid);

    assert!(validation.blockers.iter().any(|item| item.code == "unmapped-package-registry"));
    assert!(validation.blockers.iter().any(|item| item.code == "missing-wizer-entrypoint"));
    assert!(validation.blockers.iter().any(|item| item.code == "incomplete-aot-configuration"));
}

#[test]
fn cohort_member_change_invalidates_identity() {
    let first = cohort_identity(cohort()).unwrap();
    let mut changed = cohort();
    changed.wasm_tools.version = String::from("changed-version");
    let second = cohort_identity(changed).unwrap();

    assert_ne!(first, second);
    assert_eq!(first.into_hex().len(), BLAKE3_HEX_LENGTH);
}

#[test]
fn checked_lock_produces_deterministic_source_acquisition_plan() {
    let first = plan_source_acquisition(lock_request());
    let second = plan_source_acquisition(lock_request());
    let plan = first.plan.unwrap();

    assert!(first.blockers.is_empty());
    assert_eq!(plan.requests.len(), EXPECTED_SOURCE_REQUESTS);
    assert_eq!(plan.plan_identity_blake3, second.plan.unwrap().plan_identity_blake3);
    assert_eq!(plan.requests[0].credential_handle.as_deref(), Some("secret://registry/ghcr"));
}

#[test]
fn stale_lock_bytes_fail_before_source_acquisition() {
    let mut request = lock_request();
    request.lock_bytes = b"tampered-lock".to_vec();
    let result = plan_source_acquisition(request);

    assert!(result.plan.is_none());
    assert!(result.blockers.iter().any(|item| item.code == "stale-lock-bytes"));
}

#[test]
fn registry_drift_and_missing_materialization_are_denied() {
    let mut request = lock_request();
    request.facts.registry_config_blake3 = blake3('9');
    request.facts.materializations.clear();
    let validation = validate_lock(request);

    assert!(validation.blockers.iter().any(|item| item.code == "registry-config-drift"));
    assert!(validation.blockers.iter().any(|item| item.code == "missing-package-materialization"));
}

#[test]
fn duplicate_or_unlocked_materializations_are_denied() {
    let mut request = lock_request();
    let duplicate = request.facts.materializations[0].clone();
    let mut unexpected = duplicate.clone();
    unexpected.package = String::from("wasi:unlocked");
    request.facts.materializations.push(duplicate);
    request.facts.materializations.push(unexpected);
    let validation = validate_lock(request);

    assert!(validation.blockers.iter().any(|item| item.code == "duplicate-package-materialization"));
    assert!(validation.blockers.iter().any(|item| item.code == "unexpected-package-materialization"));
}

#[test]
fn exact_local_composition_graph_is_identified() {
    let first = validate_composition(composition());
    let second = validate_composition(composition());
    let plan = first.plan.unwrap();

    assert!(first.blockers.is_empty());
    assert_eq!(plan.composition.nodes.len(), EXPECTED_COMPOSITION_NODES);
    assert_eq!(plan.identity_blake3, second.plan.unwrap().identity_blake3);
}

#[test]
fn composition_rejects_missing_dependency_and_wrong_world() {
    let mut missing = composition();
    missing.edges[0].provider_node = String::from("missing");
    let missing_validation = validate_composition(missing);
    let mut wrong_world = composition();
    wrong_world.edges[0].world = String::from("wrong:world");
    let world_validation = validate_composition(wrong_world);

    assert!(missing_validation.plan.is_none());
    assert!(missing_validation.blockers.iter().any(|item| item.code == "missing-composition-dependency"));
    assert!(world_validation.blockers.iter().any(|item| item.code == "wrong-world-composition-edge"));
}

#[test]
fn composition_rejects_cycles_and_non_local_artifacts() {
    let mut cyclic = composition();
    cyclic.nodes[0].required_imports.push(RequiredImport {
        name: String::from("app"),
        world: String::from("demo:app-world"),
    });
    cyclic.edges.push(CompositionEdge {
        provider_node: String::from("app"),
        provider_export: String::from("run"),
        consumer_node: String::from("provider"),
        consumer_import: String::from("app"),
        world: String::from("demo:app-world"),
    });
    let cycle_validation = validate_composition(cyclic);
    let mut non_local = composition();
    non_local.nodes[0].artifact.logical_path = String::from("registry:ambient");
    let local_validation = validate_composition(non_local);

    assert!(cycle_validation.blockers.iter().any(|item| item.code == "composition-cycle"));
    assert!(local_validation.blockers.iter().any(|item| item.code == "non-local-composition-artifact"));
}

#[test]
fn virtualization_starts_deny_all_and_binds_virtual_inputs() {
    let result = plan_virtualization(virtualization_config());
    let plan = result.plan.unwrap();
    let clocks = plan.entries.iter().find(|entry| entry.subsystem == WasiSubsystem::Clocks).unwrap();
    let filesystem = plan.entries.iter().find(|entry| entry.subsystem == WasiSubsystem::Filesystem).unwrap();

    assert!(result.blockers.is_empty());
    assert_eq!(plan.entries.len(), WASI_SUBSYSTEM_COUNT);
    assert_eq!(clocks.action, VirtualizationAction::Deny);
    assert!(matches!(filesystem.action, VirtualizationAction::VirtualMount { .. }));
}

#[test]
fn virtualization_rejects_defaults_and_unreviewed_passthrough() {
    let mut invalid = virtualization_config();
    invalid.defaults_overridden = false;
    invalid.rules.push(VirtualizationRule {
        subsystem: WasiSubsystem::Network,
        mode: VirtualizationMode::Passthrough,
        value: None,
        value_identity_blake3: None,
        input: None,
        guest_path: None,
        review_id: None,
    });
    let result = plan_virtualization(invalid);

    assert!(result.plan.is_none());
    assert!(result.blockers.iter().any(|item| item.code == "wasi-virt-defaults-not-overridden"));
    assert!(result.blockers.iter().any(|item| item.code == "unreviewed-passthrough"));
}

#[test]
fn remaining_imports_must_match_post_composition_inspection() {
    let plan = plan_virtualization(virtualization_config()).plan.unwrap();
    let positive = validate_remaining_imports(plan.clone(), vec![String::from("wasi:cli/stdout")]);
    let negative = validate_remaining_imports(plan, vec![String::from("wasi:sockets/network")]);

    assert!(positive.matches_plan);
    assert!(!negative.matches_plan);
    assert!(negative.blockers.iter().any(|item| item.code == "remaining-import-drift"));
}

#[test]
fn portable_admission_binds_same_bytes_to_both_validation_layers() {
    let admission = portable_admission();

    assert!(admission.admitted);
    assert!(admission.octet_validation_report_blake3.is_some());
    assert_eq!(admission.schema, PORTABLE_ADMISSION_SCHEMA);
}

#[test]
fn portable_admission_rejects_octet_report_for_different_bytes() {
    let artifact = object("component.wasm", '3');
    let result = bind_portable_admission(PortableAdmissionRequest {
        artifact: artifact.clone(),
        build_validation: BuildValidationBinding {
            artifact_blake3: artifact.digest_blake3.clone(),
            cohort_blake3: blake3('4'),
            report_blake3: blake3('5'),
            decision: ValidationDecision::Pass,
        },
        octet_validation: Some(OctetValidationBinding {
            artifact_blake3: blake3('9'),
            profile_blake3: blake3('6'),
            cohort_blake3: blake3('7'),
            report_blake3: blake3('8'),
            decision: ValidationDecision::Pass,
        }),
        octet_required: true,
        expected_octet_profile_blake3: blake3('6'),
        expected_octet_cohort_blake3: blake3('7'),
    });

    assert!(result.admission.is_none());
    assert!(result.blockers.iter().any(|item| item.code == "octet-artifact-mismatch"));
}

#[test]
fn deterministic_wizer_outputs_are_admitted() {
    let output = object("wizer-output.wasm", 'a');
    let result = admit_transform(TransformAdmissionRequest {
        mode: WizerMode::Deterministic,
        input: object("component.wasm", '3'),
        first_output: output.clone(),
        repeated_output: Some(output),
        cohort_blake3: blake3('b'),
        initialization_entrypoint: String::from("wizer.initialize"),
        virtual_imports: vec![TransformImportFact {
            name: String::from("virtual:random"),
            deterministic: true,
            input_identity_blake3: Some(blake3('c')),
        }],
        ambient_observations: Vec::new(),
    });
    let admission = result.admission.unwrap();

    assert!(result.blockers.is_empty());
    assert!(admission.deterministic_outputs_match);
    assert!(admission.eligible_for_bundle);
}

#[test]
fn wizer_drift_and_ambient_state_are_denied() {
    let result = admit_transform(TransformAdmissionRequest {
        mode: WizerMode::Deterministic,
        input: object("component.wasm", '3'),
        first_output: object("first.wasm", 'a'),
        repeated_output: Some(object("second.wasm", 'b')),
        cohort_blake3: blake3('c'),
        initialization_entrypoint: String::from("wizer.initialize"),
        virtual_imports: Vec::new(),
        ambient_observations: vec![String::from("clock")],
    });

    assert!(result.admission.is_none());
    assert!(result.blockers.iter().any(|item| item.code == "wizer-output-drift"));
    assert!(result.blockers.iter().any(|item| item.code == "wizer-ambient-state"));
}

#[test]
fn exact_aot_receipt_is_admitted_as_trusted_native() {
    let portable = portable_admission();
    let source_digest = portable.artifact.digest_blake3.clone();
    let result = admit_aot(AotAdmissionRequest {
        mode: AotMode::TrustedNative,
        portable_admission: portable,
        receipt: AotReceipt {
            source_component_blake3: source_digest,
            output: object("component.cwasm", '9'),
            target: String::from("x86_64-linux"),
            cpu_features: vec![String::from("sse2")],
            wasmtime_configuration_blake3: blake3('a'),
            cohort_blake3: blake3('b'),
            wit_profile_blake3: blake3('c'),
            build_inputs_blake3: blake3('d'),
        },
        expected_target: String::from("x86_64-linux"),
        expected_cpu_features: vec![String::from("sse2")],
        expected_wasmtime_configuration_blake3: blake3('a'),
        expected_cohort_blake3: blake3('b'),
        expected_wit_profile_blake3: blake3('c'),
        expected_build_inputs_blake3: blake3('d'),
    });
    let admission = result.admission.unwrap();

    assert!(result.blockers.is_empty());
    assert_eq!(admission.trust_class, AOT_TRUST_CLASS);
    assert_eq!(admission.schema, AOT_ADMISSION_SCHEMA);
    assert_eq!(admission.wit_profile_blake3, blake3('c'));
    assert_eq!(admission.build_inputs_blake3, blake3('d'));
}

#[test]
fn cross_target_or_tampered_aot_receipt_is_denied() {
    let portable = portable_admission();
    let result = admit_aot(AotAdmissionRequest {
        mode: AotMode::TrustedNative,
        portable_admission: portable,
        receipt: AotReceipt {
            source_component_blake3: blake3('0'),
            output: object("component.cwasm", '9'),
            target: String::from("aarch64-linux"),
            cpu_features: vec![String::from("neon")],
            wasmtime_configuration_blake3: blake3('a'),
            cohort_blake3: blake3('b'),
            wit_profile_blake3: blake3('c'),
            build_inputs_blake3: blake3('d'),
        },
        expected_target: String::from("x86_64-linux"),
        expected_cpu_features: vec![String::from("sse2")],
        expected_wasmtime_configuration_blake3: blake3('a'),
        expected_cohort_blake3: blake3('b'),
        expected_wit_profile_blake3: blake3('e'),
        expected_build_inputs_blake3: blake3('f'),
    });

    assert!(result.admission.is_none());
    assert!(result.blockers.iter().any(|item| item.code == "aot-source-mismatch"));
    assert!(result.blockers.iter().any(|item| item.code == "aot-target-mismatch"));
    assert!(result.blockers.iter().any(|item| item.code == "aot-configuration-mismatch"));
}

fn generated_candidates() -> Vec<GeneratedInputCandidate> {
    vec![
        GeneratedInputCandidate {
            name: String::from("wkg-config"),
            target: String::from(".mantle/wasm-component/wkg-config.toml"),
            output_class: OutputClass::ToolInput,
            owner: GeneratedInputOwner {
                schema: String::from(GENERATED_INPUT_RECEIPT_SCHEMA),
                generator: String::from(GENERATED_INPUT_OWNER),
                export_name: String::from("wkg-config"),
                source_identity_blake3: blake3('1'),
                dependency_identity_blake3: blake3('2'),
            },
            content: String::from("default_registry = \"wasi.dev\"\n"),
        },
        GeneratedInputCandidate {
            name: String::from("wac-source"),
            target: String::from(".mantle/wasm-component/composition.wac"),
            output_class: OutputClass::CompositionInput,
            owner: GeneratedInputOwner {
                schema: String::from(GENERATED_INPUT_RECEIPT_SCHEMA),
                generator: String::from(GENERATED_INPUT_OWNER),
                export_name: String::from("wac-source"),
                source_identity_blake3: blake3('1'),
                dependency_identity_blake3: blake3('2'),
            },
            content: String::from("package demo:composition;"),
        },
    ]
}

#[test]
fn generated_inputs_bind_content_and_owner_receipts() {
    let result = finalize_generated_inputs(generated_candidates());
    let plan = result.plan.unwrap();
    let observed: Vec<ObservedGeneratedInput> = plan
        .inputs
        .iter()
        .map(|input| ObservedGeneratedInput {
            target: input.target.clone(),
            content: Some(input.content.as_bytes().to_vec()),
            receipt_identity_blake3: Some(input.receipt_identity_blake3.clone()),
        })
        .collect();
    let freshness = verify_generated_freshness(plan.clone(), observed);

    assert!(result.blockers.is_empty());
    assert!(freshness.fresh);
    assert_eq!(plan.schema, GENERATED_INPUT_PLAN_SCHEMA);
}

#[test]
fn stale_generated_bytes_or_owner_receipt_fail_closed() {
    let plan = finalize_generated_inputs(generated_candidates()).plan.unwrap();
    let observed = plan
        .inputs
        .iter()
        .map(|input| ObservedGeneratedInput {
            target: input.target.clone(),
            content: Some(b"stale".to_vec()),
            receipt_identity_blake3: Some(blake3('9')),
        })
        .collect();
    let freshness = verify_generated_freshness(plan, observed);

    assert!(!freshness.fresh);
    assert!(freshness.blockers.iter().any(|item| item.code == "stale-generated-input"));
    assert!(freshness.blockers.iter().any(|item| item.code == "stale-generated-input-receipt"));
}

#[test]
fn forged_generated_plan_integrity_fails_closed() {
    let plan = finalize_generated_inputs(generated_candidates()).plan.unwrap();
    let observed: Vec<ObservedGeneratedInput> = plan
        .inputs
        .iter()
        .map(|input| ObservedGeneratedInput {
            target: input.target.clone(),
            content: Some(input.content.as_bytes().to_vec()),
            receipt_identity_blake3: Some(input.receipt_identity_blake3.clone()),
        })
        .collect();
    let mut forged = plan;
    forged.schema = String::from("mantle-wasm-component-generated-input-plan-v2");
    forged.inputs[0].content.push_str("\nforged");
    let freshness = verify_generated_freshness(forged, observed);

    assert!(!freshness.fresh);
    assert!(freshness.blockers.iter().any(|item| item.code == "invalid-generated-plan-schema"));
    assert!(freshness.blockers.iter().any(|item| item.code == "stale-generated-receipt-content"));
    assert!(freshness.blockers.iter().any(|item| item.code == "stale-generated-plan-identity"));
}

#[test]
fn unexpected_or_duplicate_observed_inputs_fail_closed() {
    let plan = finalize_generated_inputs(generated_candidates()).plan.unwrap();
    let mut unexpected: Vec<ObservedGeneratedInput> = plan
        .inputs
        .iter()
        .map(|input| ObservedGeneratedInput {
            target: input.target.clone(),
            content: Some(input.content.as_bytes().to_vec()),
            receipt_identity_blake3: Some(input.receipt_identity_blake3.clone()),
        })
        .collect();
    let duplicate = unexpected[0].clone();
    unexpected.push(ObservedGeneratedInput {
        target: String::from(".mantle/wasm-component/unowned.toml"),
        content: Some(Vec::new()),
        receipt_identity_blake3: Some(blake3('7')),
    });
    let unexpected_result = verify_generated_freshness(plan.clone(), unexpected);
    let mut duplicated = plan
        .inputs
        .iter()
        .map(|input| ObservedGeneratedInput {
            target: input.target.clone(),
            content: Some(input.content.as_bytes().to_vec()),
            receipt_identity_blake3: Some(input.receipt_identity_blake3.clone()),
        })
        .collect::<Vec<_>>();
    duplicated.push(duplicate);
    let duplicate_result = verify_generated_freshness(plan, duplicated);

    assert!(!unexpected_result.fresh);
    assert!(unexpected_result.blockers.iter().any(|item| item.code == "unexpected-generated-input"));
    assert!(!duplicate_result.fresh);
    assert!(duplicate_result.blockers.iter().any(|item| item.code == "duplicate-observed-input"));
}

#[test]
fn generated_plan_rejects_mixed_source_ownership() {
    let mut candidates = generated_candidates();
    candidates[0].owner.source_identity_blake3 = blake3('8');
    let result = finalize_generated_inputs(candidates);

    assert!(result.plan.is_none());
    assert!(result.blockers.iter().any(|item| item.code == "mixed-generated-input-ownership"));
}

fn report_non_claims() -> Vec<String> {
    vec![String::from(RUNTIME_NON_CLAIM), String::from(RELEASE_NON_CLAIM)]
}

#[test]
fn report_dto_binds_ordered_stage_graph_and_bounded_claims() {
    let package_input = StageReportInput {
        stage_key: String::from("package-resolution"),
        kind: ComponentStageKind::PackageResolution,
        status: ComponentStageStatus::Succeeded,
        parents: Vec::new(),
        artifact: Some(object("wasi-cli", '2')),
        tool_identity_blake3: Some(blake3('3')),
        profile_identity_blake3: None,
        claims: vec![BoundedComponentClaim::ExactInputIdentities],
        non_claims: Vec::new(),
    };
    let package_identity = stage_report_identity(package_input.clone()).unwrap();
    let validation_input = StageReportInput {
        stage_key: String::from("build-validation"),
        kind: ComponentStageKind::BuildValidation,
        status: ComponentStageStatus::Succeeded,
        parents: vec![package_identity],
        artifact: Some(object("component.wasm", '4')),
        tool_identity_blake3: Some(blake3('5')),
        profile_identity_blake3: Some(blake3('6')),
        claims: vec![BoundedComponentClaim::PortableBytesValidated],
        non_claims: Vec::new(),
    };
    let result = build_component_report(vec![package_input, validation_input], report_non_claims());
    let report = result.report.unwrap();

    assert!(result.blockers.is_empty());
    assert_eq!(report.nodes.len(), EXPECTED_REPORT_NODES);
    assert_eq!(report.schema, COMPONENT_BUILD_REPORT_SCHEMA);
}

#[test]
fn report_rejects_unknown_or_circular_parent_and_missing_non_claims() {
    let input = StageReportInput {
        stage_key: String::from("composition"),
        kind: ComponentStageKind::Composition,
        status: ComponentStageStatus::Succeeded,
        parents: vec![blake3('a')],
        artifact: Some(object("component.wasm", 'b')),
        tool_identity_blake3: Some(blake3('c')),
        profile_identity_blake3: None,
        claims: Vec::new(),
        non_claims: Vec::new(),
    };
    let result = build_component_report(vec![input], Vec::new());

    assert!(result.report.is_none());
    assert!(result.blockers.iter().any(|item| item.code == "unknown-or-circular-stage-parent"));
    assert!(result.blockers.iter().any(|item| item.code == "missing-report-non-claim"));
}

#[test]
fn report_claim_enum_rejects_runtime_authority_overclaim() {
    let parsed: Result<BoundedComponentClaim, _> = serde_json::from_str("\"runtime-authority-proven\"");

    assert!(parsed.is_err());
    assert!(serde_json::from_str::<BoundedComponentClaim>("\"portable-bytes-validated\"").is_ok());
}

fn required_bundle_receipts(final_portable: &StoreObject) -> Vec<StageReceiptReference> {
    let stages = [
        ("package-resolution", ComponentStageKind::PackageResolution, '1'),
        ("lock", ComponentStageKind::Lock, '2'),
        ("binding-generation", ComponentStageKind::BindingGeneration, '3'),
        ("compilation", ComponentStageKind::Compilation, '4'),
        ("composition", ComponentStageKind::Composition, '5'),
        ("virtualization", ComponentStageKind::Virtualization, '6'),
        ("build-validation", ComponentStageKind::BuildValidation, '7'),
        ("octet-validation", ComponentStageKind::OctetValidation, '8'),
    ];
    stages
        .into_iter()
        .map(|(stage_key, kind, seed)| StageReceiptReference {
            stage_key: String::from(stage_key),
            kind,
            receipt_blake3: blake3(seed),
            receipt: object(&format!("{stage_key}.receipt.json"), seed),
            artifact: matches!(
                kind,
                ComponentStageKind::Compilation
                    | ComponentStageKind::Composition
                    | ComponentStageKind::Virtualization
                    | ComponentStageKind::BuildValidation
                    | ComponentStageKind::OctetValidation
            )
            .then(|| final_portable.clone()),
        })
        .collect()
}

fn materialization_bundle_request() -> MaterializationBundleRequest {
    let final_portable = object("final.component.wasm", '9');
    MaterializationBundleRequest {
        name: String::from("demo-component"),
        manifest_blake3: blake3('a'),
        cohort_blake3: blake3('b'),
        wit_inputs: vec![object("wit", 'c')],
        package_inputs: lock_request().facts.materializations,
        source_closure: object("source", 'd'),
        lock: object("wkg.lock", 'e'),
        final_portable: final_portable.clone(),
        expected_octet_profile_blake3: blake3('f'),
        expected_runtime_profile_blake3: blake3('0'),
        stage_receipts: required_bundle_receipts(&final_portable),
        wizer: None,
        aot: None,
        non_claims: report_non_claims(),
    }
}

#[test]
fn materialization_bundle_is_canonical_and_reverifiable() {
    let first = build_materialization_bundle(materialization_bundle_request()).bundle.unwrap();
    let second = build_materialization_bundle(materialization_bundle_request()).bundle.unwrap();
    let verified = verify_materialization_bundle(first.clone());

    assert_eq!(first, second);
    assert_eq!(first.schema, MATERIALIZATION_BUNDLE_SCHEMA);
    assert_eq!(verified.bundle, Some(first));
    assert!(verified.blockers.is_empty());
}

#[test]
fn materialization_bundle_rejects_missing_stage_and_stale_identity() {
    let mut incomplete = materialization_bundle_request();
    incomplete.stage_receipts.retain(|receipt| receipt.kind != ComponentStageKind::OctetValidation);
    let incomplete_result = build_materialization_bundle(incomplete);
    let mut stale = build_materialization_bundle(materialization_bundle_request()).bundle.unwrap();
    stale.bundle_identity_blake3 = blake3('1');
    let stale_result = verify_materialization_bundle(stale);

    assert!(incomplete_result.bundle.is_none());
    assert!(incomplete_result.blockers.iter().any(|item| item.code == "missing-bundle-stage-receipt"));
    assert!(stale_result.bundle.is_none());
    assert!(stale_result.blockers.iter().any(|item| item.code == "materialization-bundle-identity-mismatch"));
}

#[test]
fn materialization_bundle_stage_kind_rejects_post_materialization_authority() {
    let parsed: Result<StageReceiptReference, _> = serde_json::from_str(
        r#"{"stage_key":"cairn","kind":"cairn-acceptance","receipt_blake3":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa","receipt":{"logical_path":"/mantle/store/fixture-receipt","digest_blake3":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa","size_bytes":1},"artifact":null}"#,
    );
    let accepted: Result<ComponentStageKind, _> = serde_json::from_str("\"octet-validation\"");

    assert!(parsed.is_err());
    assert_eq!(accepted.unwrap(), ComponentStageKind::OctetValidation);
}

fn component_evidence_request() -> ComponentEvidenceRequest {
    let bundle = build_materialization_bundle(materialization_bundle_request()).bundle.unwrap();
    let octet_report = object("octet-report.json", '8');
    let octet_input = StageReportInput {
        stage_key: String::from("octet-validation"),
        kind: ComponentStageKind::OctetValidation,
        status: ComponentStageStatus::Succeeded,
        parents: Vec::new(),
        artifact: Some(octet_report.clone()),
        tool_identity_blake3: Some(blake3('7')),
        profile_identity_blake3: Some(bundle.expected_octet_profile_blake3.clone()),
        claims: vec![BoundedComponentClaim::OctetReportBound],
        non_claims: report_non_claims(),
    };
    let octet_identity = stage_report_identity(octet_input.clone()).unwrap();
    let bundle_object = object("materialization-bundle.json", '6');
    let materialization_input = StageReportInput {
        stage_key: String::from("materialization-bundle"),
        kind: ComponentStageKind::MaterializationBundle,
        status: ComponentStageStatus::Succeeded,
        parents: vec![octet_identity],
        artifact: Some(bundle_object.clone()),
        tool_identity_blake3: None,
        profile_identity_blake3: Some(bundle.bundle_identity_blake3.clone()),
        claims: vec![BoundedComponentClaim::MaterializationObjectsRehashable],
        non_claims: report_non_claims(),
    };
    let report = build_component_report(vec![octet_input, materialization_input], report_non_claims())
        .report
        .unwrap();
    ComponentEvidenceRequest {
        bundle,
        bundle_object,
        report,
        octet_profile_blake3: blake3('f'),
        octet_cohort_blake3: blake3('7'),
        octet_report,
    }
}

#[test]
fn component_attestation_and_release_binding_follow_verified_stage_graph() {
    let result = build_component_evidence(component_evidence_request());
    let attestation = result.artifact_attestation.unwrap();
    let release = result.release_binding.unwrap();

    assert!(result.blockers.is_empty());
    assert_eq!(attestation.schema, COMPONENT_ARTIFACT_ATTESTATION_SCHEMA);
    assert_eq!(release.schema, COMPONENT_RELEASE_BINDING_SCHEMA);
    assert_eq!(release.artifact_attestation_blake3, attestation.attestation_blake3);
    assert!(!release.release_eligible);
}

#[test]
fn component_evidence_rejects_swapped_bundle_object_and_stale_report_identity() {
    let mut swapped = component_evidence_request();
    swapped.bundle_object = object("swapped-bundle.json", '5');
    let swapped_result = build_component_evidence(swapped);
    let mut stale = component_evidence_request().report;
    stale.report_identity_blake3 = blake3('4');
    let stale_result = verify_component_report(stale);

    assert!(swapped_result.artifact_attestation.is_none());
    assert!(swapped_result.blockers.iter().any(|item| item.code == "component-attestation-bundle-mismatch"));
    assert!(stale_result.report.is_none());
    assert!(stale_result.blockers.iter().any(|item| item.code == "component-report-identity-mismatch"));
}
