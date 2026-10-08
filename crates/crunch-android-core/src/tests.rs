use super::*;

fn identity(component: &str) -> SourceIdentity {
    SourceIdentity {
        component: component.into(),
        version: "1".into(),
        url: "file:///fixture.zip".into(),
        sha256: "1".repeat(64),
        record_identity_blake3: "2".repeat(64),
        unpack_shape: UnpackShape {
            archive: "zip".into(),
            root: "tools/".into(),
        },
        platform: "x86_64-linux".into(),
    }
}

pub(crate) fn valid() -> ApkPlan {
    ApkPlan {
        module: "demo".into(),
        application_id: "org.example.demo".into(),
        version_code: 1,
        manifest_member: "AndroidManifest.xml".into(),
        resources: alloc::vec!["res/values/strings.xml".into(), "res/drawable/icon.png".into()],
        java_sources: alloc::vec!["src/Demo.java".into()],
        toolchain: Toolchain {
            build_tools: identity("android-build-tools"),
            jdk: identity("temurin-jdk"),
            platform: identity("android-platform"),
        },
        signing: Some(Signing {
            keystore_member: "keys/demo.jks".into(),
            key_alias: "demo".into(),
            schemes: alloc::vec!["v2".into()],
        }),
        reproducibility: Reproducibility {
            entry_timestamp_epoch: MIN_ZIP_EPOCH,
            locale: "C".into(),
            timezone: "UTC".into(),
        },
    }
}

#[test]
fn signed_and_unsigned_order_and_membership() {
    let mut plan = valid();
    let signed = lower(&plan).unwrap();
    assert_eq!(signed.steps.iter().map(|step| step.kind).collect::<Vec<_>>(), alloc::vec![
        StepKind::Aapt2Compile,
        StepKind::Aapt2Link,
        StepKind::Javac,
        StepKind::D8,
        StepKind::Zipalign,
        StepKind::Apksigner
    ]);
    assert_eq!(signed.steps[0].inputs[0], StepInput::SourceMember("res/drawable/icon.png".into()));
    assert_eq!(signed.steps[5].inputs[1], StepInput::KeystoreMember("keys/demo.jks".into()));
    plan.signing = None;
    let unsigned = lower(&plan).unwrap();
    assert_eq!(unsigned.steps.len(), 5);
    assert_eq!(unsigned.final_output, StepKind::Zipalign);
    assert_eq!(unsigned.steps.last().unwrap().output, "unsigned.apk");
}

#[test]
fn invalid_plans_return_distinct_rejections_before_lowering() {
    let mut cases = Vec::new();
    let mut plan = valid();
    plan.manifest_member = "".into();
    cases.push((plan, Rejection::MissingManifestMember));
    let mut plan = valid();
    plan.toolchain.build_tools.record_identity_blake3 = "0".repeat(64);
    cases.push((plan, Rejection::UnboundToolchainIdentity));
    let mut plan = valid();
    plan.reproducibility.entry_timestamp_epoch = MIN_ZIP_EPOCH - 1;
    cases.push((plan, Rejection::UnpinnedTimestampPolicy));
    let mut plan = valid();
    plan.signing.as_mut().unwrap().keystore_member = "keys/../secret".into();
    cases.push((plan, Rejection::KeystorePathEscape));
    let mut plan = valid();
    plan.signing.as_mut().unwrap().schemes = alloc::vec!["v4".into()];
    cases.push((plan, Rejection::UnknownSignatureScheme));
    let mut plan = valid();
    plan.java_sources.clear();
    cases.push((plan, Rejection::EmptySourceSet));
    let mut plan = valid();
    plan.resources = alloc::vec!["res/file".into(); MAX_MEMBERS + 1];
    cases.push((plan, Rejection::TooManyMembers));
    let mut plan = valid();
    plan.java_sources = alloc::vec!["/tmp/host.java".into()];
    cases.push((plan, Rejection::InvalidSourceMember));
    for (plan, expected) in cases {
        assert_eq!(lower(&plan), Err(expected));
    }
}
