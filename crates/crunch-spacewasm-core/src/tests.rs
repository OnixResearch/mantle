use alloc::format;
use alloc::string::String;
use alloc::string::ToString;
use alloc::vec;
use alloc::vec::Vec;

use crate::*;

const PREVIOUS_REVISION: &str = "30cd6e9b91f84a39278edcb5d66514b773011ccc";
const SELECTED_REVISION: &str = "e24cf09355a90497148eb5029fdb8e3400bd63e3";
const HOST_POINTER_WIDTH_BITS: u32 = 64;
const WASM_POINTER_WIDTH_BITS: u32 = 32;
const DEPENDENCY_PACKAGE_COUNT: u32 = 53;
const FIXTURE_BYTES: u64 = 16;
const BUNDLE_MEMBER_MAX_BYTES: u64 = 1_048_576;
const BUNDLE_TOTAL_MAX_BYTES: u64 = 16_777_216;

fn digest(value: char) -> Blake3Digest {
    Blake3Digest::parse(value.to_string().repeat(BLAKE3_HEX_LENGTH)).unwrap()
}

fn target(role: TargetRole, triple: &str, pointer_width_bits: u32) -> TargetProfile {
    TargetProfile {
        role,
        triple: String::from(triple),
        pointer_width_bits,
        features: vec![String::from("default-features-disabled")],
    }
}

fn fixture(id: &str, class: FixtureClass, marker: char) -> FixtureProfile {
    FixtureProfile {
        fixture_id: String::from(id),
        class,
        artifact_path: format!("fixtures/wasm/{id}.wasm"),
        descriptor_path: format!("fixtures/descriptors/{id}.json"),
        artifact_blake3: digest(marker),
        descriptor_blake3: digest('b'),
        generator_input_blake3: digest('f'),
        expected_status: CheckStatus::Passed,
        expected_code: format!("expected-{id}"),
    }
}

fn profile() -> ReferenceProfile {
    ReferenceProfile {
        schema: String::from(PROFILE_SCHEMA),
        profile_id: String::from("nasa-spacewasm-e24cf093-reference-v1"),
        profile_update: ProfileUpdate {
            previous_review_revision: String::from(PREVIOUS_REVISION),
            selected_revision: String::from(SELECTED_REVISION),
            reason: String::from("align with Octet's reviewed spacewasm-mvp support projection"),
            replay_required: true,
            replay_evidence_id: String::from("spacewasm-e24-complete-fixture-replay-v1"),
        },
        source: SourceProfile {
            repository: String::from("https://github.com/nasa/spacewasm"),
            revision: String::from(SELECTED_REVISION),
            archive_url: format!("https://github.com/nasa/spacewasm/archive/{SELECTED_REVISION}.tar.gz"),
            archive_sha256_sri: String::from("sha256-iNgipIlAPOJU38oOqvqxXQF15FJ0ooN3rH5DzHiKOTU="),
            archive_blake3: digest('a'),
            cargo_lock_blake3: digest('b'),
            dependency_manifest_blake3: digest('c'),
            dependency_package_count: DEPENDENCY_PACKAGE_COUNT,
            octet_support_projection_blake3: digest('d'),
            license_members: vec![String::from("LICENSE"), String::from("licenses/spectest-MIT.txt")],
            notice_members: vec![String::from("NOTICE")],
        },
        toolchain: ToolchainProfile {
            rust_channel: String::from("stable"),
            rust_version: String::from("1.91.1"),
            host_triple: String::from("x86_64-unknown-linux-gnu"),
            wasm_triple: String::from("wasm32-unknown-unknown"),
        },
        targets: vec![
            target(TargetRole::HostLibrary, "x86_64-unknown-linux-gnu", HOST_POINTER_WIDTH_BITS),
            target(TargetRole::WasmLibrary, "wasm32-unknown-unknown", WASM_POINTER_WIDTH_BITS),
            target(TargetRole::HostDiagnosticRunner, "x86_64-unknown-linux-gnu", HOST_POINTER_WIDTH_BITS),
        ],
        support_matrix: vec![
            SupportEntry {
                feature: String::from("wasm1"),
                status: SupportStatus::Supported,
            },
            SupportEntry {
                feature: String::from("mutable-globals"),
                status: SupportStatus::Supported,
            },
            SupportEntry {
                feature: String::from("bulk-memory"),
                status: SupportStatus::Unsupported,
            },
            SupportEntry {
                feature: String::from("component-model"),
                status: SupportStatus::Unsupported,
            },
        ],
        requested_features: vec![String::from("wasm1"), String::from("mutable-globals")],
        runner: RunnerProfile {
            runner_id: String::from("mantle-spacewasm-diagnostic-runner-v1"),
            max_input_bytes: BUNDLE_MEMBER_MAX_BYTES,
            max_stream_chunks: 128,
            default_fuel: 1_024,
            max_code_pages: 64,
            max_control_frames: 64,
            max_stack_words: 1_024,
        },
        fixtures: vec![
            fixture("mvp-positive", FixtureClass::MvpPositive, '1'),
            fixture("mvp-negative", FixtureClass::MvpNegative, '2'),
            fixture("streaming-positive", FixtureClass::StreamingPositive, '3'),
            fixture("streaming-negative", FixtureClass::StreamingNegative, '4'),
            fixture("allocation-failure", FixtureClass::AllocationFailure, '5'),
            fixture("unsupported-feature", FixtureClass::UnsupportedFeature, '6'),
            fixture("trap", FixtureClass::Trap, '7'),
            fixture("out-of-fuel", FixtureClass::OutOfFuel, '8'),
        ],
        corpora: vec![
            CorpusProfile {
                corpus_id: String::from("upstream-spectest"),
                role: CorpusRole::UpstreamSpectest,
                descriptor_path: String::from("corpora/spectest.json"),
                source_path: String::from("tests/core"),
                provenance: String::from("WasmEdge/wasmedge-spectest curated upstream corpus"),
                license_path: String::from("licenses/spectest-MIT.txt"),
                descriptor_blake3: digest('9'),
                expected_status: CheckStatus::Passed,
            },
            CorpusProfile {
                corpus_id: String::from("upstream-fuzz"),
                role: CorpusRole::UpstreamFuzz,
                descriptor_path: String::from("corpora/fuzz.json"),
                source_path: String::from("fuzz/fuzz_targets"),
                provenance: String::from("NASA SpaceWasm checked source archive"),
                license_path: String::from("LICENSE"),
                descriptor_blake3: digest('e'),
                expected_status: CheckStatus::Skipped,
            },
        ],
        checks: vec![
            CheckProfile {
                check_id: String::from("host-library-build"),
                expected_status: CheckStatus::Passed,
            },
            CheckProfile {
                check_id: String::from("wasm-library-build"),
                expected_status: CheckStatus::Passed,
            },
            CheckProfile {
                check_id: String::from("fixture-replay"),
                expected_status: CheckStatus::Passed,
            },
            CheckProfile {
                check_id: String::from("full-spectest-suite"),
                expected_status: CheckStatus::Skipped,
            },
            CheckProfile {
                check_id: String::from("spacewasm-check"),
                expected_status: CheckStatus::Unavailable,
            },
            CheckProfile {
                check_id: String::from("continuous-fuzzing"),
                expected_status: CheckStatus::Unsupported,
            },
        ],
        retention: RetentionProfile {
            source_archive: true,
            dependency_closure: true,
            toolchain: true,
            binaries: true,
            fixtures: true,
            reports: true,
            licenses_and_notices: true,
            non_claims: true,
        },
        bounds: ReferenceBounds {
            max_profile_text_bytes: 262_144,
            max_collection_items: 128,
            max_bundle_members: 512,
            max_parent_edges: 2_048,
            max_bundle_member_bytes: BUNDLE_MEMBER_MAX_BYTES,
            max_bundle_total_bytes: BUNDLE_TOTAL_MAX_BYTES,
        },
        non_claims: REQUIRED_NON_CLAIMS.iter().map(|value| String::from(*value)).collect(),
    }
}

fn target_facts(profile: &ReferenceProfile) -> Vec<TargetFact> {
    profile
        .targets
        .iter()
        .map(|target| TargetFact {
            role: target.role,
            triple: target.triple.clone(),
            pointer_width_bits: target.pointer_width_bits,
            features: target.features.clone(),
        })
        .collect()
}

fn source_facts(profile: &ReferenceProfile) -> SourceFacts {
    SourceFacts {
        reference_kind: ReferenceKind::ExactCommit,
        revision: profile.source.revision.clone(),
        archive_blake3: profile.source.archive_blake3.clone(),
        cargo_lock_blake3: profile.source.cargo_lock_blake3.clone(),
        dependency_manifest_blake3: profile.source.dependency_manifest_blake3.clone(),
        dependency_package_count: profile.source.dependency_package_count,
        octet_support_projection_blake3: profile.source.octet_support_projection_blake3.clone(),
        rust_version: profile.toolchain.rust_version.clone(),
        targets: target_facts(profile),
        present_licenses: profile.source.license_members.clone(),
        present_notices: profile.source.notice_members.clone(),
        corpora: profile
            .corpora
            .iter()
            .map(|corpus| CorpusFact {
                corpus_id: corpus.corpus_id.clone(),
                descriptor_blake3: corpus.descriptor_blake3.clone(),
            })
            .collect(),
        network_attempted_during_build: false,
        fallback_acquisition_used: false,
    }
}

fn observed_checks(profile: &ReferenceProfile) -> Vec<ObservedCheck> {
    profile
        .checks
        .iter()
        .map(|check| ObservedCheck {
            check_id: check.check_id.clone(),
            status: check.expected_status,
            result_code: String::from("recorded-exactly"),
            command_blake3: digest('1'),
            configuration_blake3: digest('2'),
            input_blake3: digest('3'),
            output_blake3: Some(digest('4')),
        })
        .collect()
}

#[test]
fn profile_identity_and_decisions_are_deterministic() {
    let first = validate_profile(profile());
    let mut reordered = profile();
    reordered.targets.reverse();
    reordered.fixtures.reverse();
    reordered.non_claims.reverse();
    let second = validate_profile(reordered);

    assert!(first.diagnostics.is_empty(), "{:?}", first.diagnostics);
    assert_eq!(first.profile_identity_blake3, second.profile_identity_blake3);
    assert!(first.profile_identity_blake3.is_some());
}

#[test]
fn exact_source_support_and_declared_outcomes_are_admitted() {
    let profile = profile();
    let source = admit_source(profile.clone(), source_facts(&profile));
    let support = compare_support_matrix(profile.clone(), profile.support_matrix.clone());
    let checks = evaluate_checks(profile.clone(), observed_checks(&profile));

    assert!(source.admitted, "{:?}", source.diagnostics);
    assert!(support.matches_profile, "{:?}", support.diagnostics);
    assert!(checks.complete, "{:?}", checks.diagnostics);
    assert!(checks.decisions.iter().any(|check| check.observed_status == CheckStatus::Unavailable));
    assert!(checks.decisions.iter().any(|check| check.observed_status == CheckStatus::Unsupported));
}

#[test]
fn source_admission_rejects_floating_stale_wrong_legal_and_corpus_inputs() {
    let profile = profile();
    let mut facts = source_facts(&profile);
    facts.reference_kind = ReferenceKind::FloatingRef;
    facts.cargo_lock_blake3 = digest('0');
    facts.rust_version = String::from("1.87.0");
    facts.present_licenses.clear();
    facts.corpora[0].descriptor_blake3 = digest('0');
    let admission = admit_source(profile, facts);
    let codes: Vec<_> = admission.diagnostics.iter().map(|item| item.code.as_str()).collect();

    assert!(!admission.admitted);
    assert!(codes.contains(&"floating-source-ref"));
    assert!(codes.contains(&"stale-cargo-lock"));
    assert!(codes.contains(&"wrong-rust-toolchain"));
    assert!(codes.contains(&"omitted-license"));
    assert!(codes.contains(&"corpus-drift"));
}

#[test]
fn support_comparison_rejects_unsupported_request_and_projection_drift() {
    let mut profile = profile();
    profile.requested_features.push(String::from("bulk-memory"));
    let validation = validate_profile(profile.clone());
    let mut observed = profile.support_matrix.clone();
    observed[0].status = SupportStatus::Unreviewed;
    let comparison = compare_support_matrix(profile, observed);

    assert!(validation.diagnostics.iter().any(|item| item.code == "unsupported-feature-requested"));
    assert!(!comparison.matches_profile);
    assert!(comparison.diagnostics.iter().any(|item| item.code == "support-status-mismatch"));
    assert!(comparison.diagnostics.iter().any(|item| item.code == "unsupported-feature-requested"));
}

#[test]
fn report_rejects_overclaim_and_detects_tamper() {
    let profile = profile();
    let validation = validate_profile(profile.clone());
    let source = admit_source(profile.clone(), source_facts(&profile));
    let support = compare_support_matrix(profile.clone(), profile.support_matrix.clone());
    let checks = evaluate_checks(profile.clone(), observed_checks(&profile));
    let overclaim = build_materialization_report(ReportBuildInput {
        profile: profile.clone(),
        profile_identity_blake3: validation.profile_identity_blake3.clone().unwrap(),
        cohort_identity_blake3: digest('a'),
        source_admission: source.clone(),
        support_comparison: support.clone(),
        checks: checks.decisions.clone(),
        check_evaluation_complete: checks.complete,
        diagnostics: Vec::new(),
        requested_claim_class: String::from("flight-qualified"),
    });
    assert!(overclaim.is_err());

    let mut report = build_materialization_report(ReportBuildInput {
        profile,
        profile_identity_blake3: validation.profile_identity_blake3.unwrap(),
        cohort_identity_blake3: digest('a'),
        source_admission: source,
        support_comparison: support,
        checks: checks.decisions,
        check_evaluation_complete: checks.complete,
        diagnostics: Vec::new(),
        requested_claim_class: String::from("exact-reference-materialization-facts"),
    })
    .unwrap();
    assert_eq!(report.disposition, ReportDisposition::Complete);
    report.profile_id.push_str("-tampered");
    let result = validate_materialization_report(report);
    assert!(!result.valid);
    assert!(result.diagnostics.iter().any(|item| item.code == "report-tamper"));
}

#[test]
fn complete_bundle_remeasures_and_missing_report_fails() {
    let profile = profile();
    let validation = validate_profile(profile.clone());
    let members = complete_members(&profile);
    let parent_edges = parent_edges(&members);
    let manifest = build_bundle_manifest(BundleManifestInput {
        profile: profile.clone(),
        profile_identity_blake3: validation.profile_identity_blake3.unwrap(),
        cohort_identity_blake3: digest('a'),
        members: members.clone(),
        parent_edges,
        non_claims: profile.non_claims.clone(),
    })
    .unwrap();
    let verified = verify_bundle_manifest(manifest.clone(), members.clone());
    assert!(verified.valid, "{:?}", verified.diagnostics);

    let without_report: Vec<_> = members
        .into_iter()
        .filter(|member| member.role != BundleRole::MaterializationReport)
        .collect();
    let rejected = verify_bundle_manifest(manifest, without_report);
    assert!(!rejected.valid);
    assert!(rejected.diagnostics.iter().any(|item| item.code == "incomplete-bundle"));
}

#[test]
fn bundle_planning_rejects_omitted_license_and_incomplete_parent_graph() {
    let profile = profile();
    let validation = validate_profile(profile.clone());
    let mut members = complete_members(&profile);
    members.retain(|member| member.role != BundleRole::License);
    let result = build_bundle_manifest(BundleManifestInput {
        profile: profile.clone(),
        profile_identity_blake3: validation.profile_identity_blake3.unwrap(),
        cohort_identity_blake3: digest('a'),
        parent_edges: Vec::new(),
        members,
        non_claims: profile.non_claims,
    });
    let diagnostics = result.unwrap_err();

    assert!(diagnostics.iter().any(|item| item.code == "missing-or-wrong-role-member"));
    assert!(diagnostics.iter().any(|item| item.code == "missing-parent-edge"));
}

#[test]
fn diagnostics_are_stably_ordered() {
    let mut profile = profile();
    profile.source.license_members.clear();
    profile.requested_features.push(String::from("threads"));
    let first = validate_profile(profile.clone()).diagnostics;
    let second = validate_profile(profile).diagnostics;

    assert_eq!(first, second);
    assert!(first.windows(ADJACENT_WINDOW_LENGTH).all(|pair| pair[0] <= pair[1]));
    assert!(!first.is_empty());
}

fn complete_members(profile: &ReferenceProfile) -> Vec<BundleMember> {
    let mut members = vec![
        member("profile/profile.ncl", BundleRole::ProfileSource, digest('1')),
        member("profile/profile.json", BundleRole::ProfileExport, digest('2')),
        member("source/spacewasm.tar.gz", BundleRole::SourceArchive, profile.source.archive_blake3.clone()),
        member("source/Cargo.lock", BundleRole::CargoLock, profile.source.cargo_lock_blake3.clone()),
        member("dependencies/manifest.json", BundleRole::DependencyManifest, profile.source.dependency_manifest_blake3.clone()),
        member("dependencies/vendor.tar", BundleRole::DependencyClosure, digest('3')),
        member("toolchain/rustc", BundleRole::RustcBinary, digest('4')),
        member("toolchain/cargo", BundleRole::CargoBinary, digest('5')),
        member("toolchain/toolchain.tar", BundleRole::ToolchainArchive, digest('a')),
        member("binaries/host/libspacewasm.rlib", BundleRole::HostLibrary, digest('6')),
        member("binaries/wasm/libspacewasm.rlib", BundleRole::WasmLibrary, digest('7')),
        member("binaries/host/spacewasm-diagnostic-runner", BundleRole::HostRunner, digest('8')),
        member("profile/octet-support-projection.json", BundleRole::SupportProjection, profile.source.octet_support_projection_blake3.clone()),
        member("reports/results.json", BundleRole::ResultReport, digest('b')),
        member("reports/replay-evidence.json", BundleRole::ReplayEvidence, digest('c')),
        member("reports/materialization.json", BundleRole::MaterializationReport, digest('9')),
        member("non-claims.json", BundleRole::NonClaims, digest('a')),
    ];
    for fixture in &profile.fixtures {
        members.push(member(&fixture.artifact_path, BundleRole::FixtureArtifact, fixture.artifact_blake3.clone()));
        members.push(member(&fixture.descriptor_path, BundleRole::FixtureDescriptor, digest('b')));
    }
    for corpus in &profile.corpora {
        members.push(member(
            &format!("corpora/{}.tar", corpus.corpus_id),
            BundleRole::CorpusArtifact,
            digest('f'),
        ));
        members.push(member(&corpus.descriptor_path, BundleRole::CorpusDescriptor, corpus.descriptor_blake3.clone()));
    }
    for license in &profile.source.license_members {
        members.push(member(license, BundleRole::License, digest('c')));
    }
    for notice in &profile.source.notice_members {
        members.push(member(notice, BundleRole::Notice, digest('d')));
    }
    members
}

fn member(path: &str, role: BundleRole, digest_blake3: Blake3Digest) -> BundleMember {
    BundleMember {
        path: String::from(path),
        role,
        digest_blake3,
        size_bytes: FIXTURE_BYTES,
    }
}

fn parent_edges(members: &[BundleMember]) -> Vec<ParentEdge> {
    members
        .iter()
        .filter(|member| member.role != BundleRole::ProfileSource)
        .map(|member| ParentEdge {
            parent_path: String::from("profile/profile.ncl"),
            child_path: member.path.clone(),
            relation: String::from("profile-binds-member"),
        })
        .collect()
}
