use std::fs;
use std::process::Command;

use crunch_android_core::MIN_ZIP_EPOCH;
use crunch_android_core::Reproducibility;
use crunch_android_core::Signing;
use crunch_android_core::Toolchain;
use crunch_android_core::UnpackShape;

use super::*;

fn fixture_archive(root: &Path, component: &str, tools: &[&str]) -> BoundArchive {
    let content = root.join(format!("{component}-content"));
    fs::create_dir_all(content.join("tools/bin")).unwrap();
    fs::create_dir_all(content.join("tools/lib")).unwrap();
    for tool in tools {
        let path = if *tool == "javac" || *tool == "jar" || *tool == "java" {
            content.join("tools/bin").join(tool)
        } else {
            content.join("tools").join(tool)
        };
        fs::write(&path, "#!/bin/sh\nprintf 'unexpected synthetic SDK execution\\n' >&2\nexit 99\n").unwrap();
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&path, fs::Permissions::from_mode(0o755)).unwrap();
    }
    if component == "android-platform" {
        fs::write(content.join("tools/android.jar"), "jar").unwrap();
    }
    let archive = root.join(format!("{component}.tar.gz"));
    let status = Command::new("tar")
        .args([
            "-czf",
            archive.to_str().unwrap(),
            "-C",
            content.to_str().unwrap(),
            "tools",
        ])
        .status()
        .unwrap();
    assert!(status.success());
    let digest = format!("{:x}", Sha256::digest(fs::read(&archive).unwrap()));
    BoundArchive {
        identity: SourceIdentity {
            component: component.into(),
            version: "1-fixture".into(),
            url: format!("file://{}", archive.display()),
            sha256: digest,
            record_identity_blake3: "2".repeat(64),
            unpack_shape: UnpackShape {
                archive: "tar-gz".into(),
                root: "tools/".into(),
            },
            platform: "x86_64-linux".into(),
        },
        store_path: archive,
    }
}

pub(super) fn fixture(root: &Path, signed: bool) -> (ApkPlan, BuildInputs) {
    let sources = root.join("source");
    fs::create_dir_all(sources.join("src")).unwrap();
    fs::create_dir_all(sources.join("res/values")).unwrap();
    fs::write(sources.join("AndroidManifest.xml"), "<manifest/>").unwrap();
    fs::write(sources.join("src/Demo.java"), "class Demo {}").unwrap();
    fs::write(sources.join("res/values/strings.xml"), "<resources/>").unwrap();
    let build_tools = fixture_archive(root, "android-build-tools", &["aapt2", "zipalign"]);
    let jdk = fixture_archive(root, "temurin-jdk", &["javac", "jar", "java"]);
    let platform = fixture_archive(root, "android-platform", &[]);
    let runtime_glibc_root = root.join("runtime-glibc");
    let runtime_libgcc_root = root.join("runtime-libgcc");
    fs::create_dir_all(runtime_glibc_root.join("lib")).unwrap();
    fs::create_dir_all(runtime_libgcc_root.join("lib")).unwrap();
    let loader = runtime_glibc_root.join("lib/ld-linux-x86-64.so.2");
    fs::write(&loader, "#!/bin/sh\n[ \"$1\" = --library-path ] || exit 81\nshift 2\nexec \"$@\"\n").unwrap();
    fs::set_permissions(&loader, fs::Permissions::from_mode(0o755)).unwrap();
    fs::write(runtime_libgcc_root.join("lib/libgcc_s.so.1"), "fixture-libgcc").unwrap();
    let runtime_glibc = BoundRuntime {
        identity: RuntimeIdentity {
            source_cohort: "signed-fixture-glibc".into(),
            nar_sha256: "3".repeat(64),
            nar_size: 1,
        },
        store_path: runtime_glibc_root,
    };
    let runtime_libgcc = BoundRuntime {
        identity: RuntimeIdentity {
            source_cohort: "signed-fixture-libgcc".into(),
            nar_sha256: "4".repeat(64),
            nar_size: 1,
        },
        store_path: runtime_libgcc_root,
    };
    let utility = root.join("utility");
    fs::write(
        &utility,
        format!("#!/bin/sh\nPATH='{}'\nexport PATH\nexec \"$@\"\n", std::env::var("PATH").unwrap()),
    )
    .unwrap();
    use std::os::unix::fs::PermissionsExt;
    fs::set_permissions(&utility, fs::Permissions::from_mode(0o755)).unwrap();
    fs::create_dir_all(sources.join("keys")).unwrap();
    let keystore = sources.join("keys/key.jks");
    let password = root.join("key.password");
    fs::write(&keystore, "fixture-key").unwrap();
    fs::write(&password, "fixture-password").unwrap();
    let plan = ApkPlan {
        module: "demo".into(),
        application_id: "org.example.demo".into(),
        version_code: 1,
        manifest_member: "AndroidManifest.xml".into(),
        resources: vec!["res/values/strings.xml".into()],
        java_sources: vec!["src/Demo.java".into()],
        toolchain: Toolchain {
            build_tools: build_tools.identity.clone(),
            jdk: jdk.identity.clone(),
            platform: platform.identity.clone(),
        },
        signing: signed.then(|| Signing {
            keystore_member: "keys/key.jks".into(),
            key_alias: "fixture".into(),
            schemes: vec!["v2".into()],
        }),
        reproducibility: Reproducibility {
            entry_timestamp_epoch: MIN_ZIP_EPOCH,
            locale: "C".into(),
            timezone: "UTC".into(),
        },
    };
    (plan, BuildInputs {
        source_root: sources,
        build_tools,
        jdk,
        platform,
        runtime_glibc,
        runtime_libgcc,
        utility,
        keystore: signed.then_some(keystore),
        key_password_file: signed.then_some(password),
    })
}
pub(super) fn runtime_cohort(inputs: &BuildInputs) -> RuntimeCohort {
    RuntimeCohort {
        glibc: inputs.runtime_glibc.identity.clone(),
        libgcc: inputs.runtime_libgcc.identity.clone(),
    }
}

#[test]
fn invalid_plans_and_archive_drift_reject_before_graph_generation() {
    let dir = tempfile::tempdir().unwrap();
    let (mut plan, inputs) = fixture(dir.path(), false);
    let reviewed = plan.toolchain.clone();
    let runtime = runtime_cohort(&inputs);
    let failures = [
        (Rejection::MissingManifestMember, "manifest"),
        (Rejection::UnboundToolchainIdentity, "identity"),
        (Rejection::UnpinnedTimestampPolicy, "timestamp"),
        (Rejection::KeystorePathEscape, "keystore"),
        (Rejection::UnknownSignatureScheme, "signature"),
        (Rejection::EmptySourceSet, "source"),
    ];
    for (category, _) in failures {
        let mut candidate = plan.clone();
        match category {
            Rejection::MissingManifestMember => candidate.manifest_member.clear(),
            Rejection::UnboundToolchainIdentity => candidate.toolchain.build_tools.sha256 = "0".repeat(64),
            Rejection::UnpinnedTimestampPolicy => candidate.reproducibility.entry_timestamp_epoch = 0,
            Rejection::KeystorePathEscape => {
                candidate.signing = Some(Signing {
                    keystore_member: "../bad".into(),
                    key_alias: "key".into(),
                    schemes: vec!["v2".into()],
                })
            }
            Rejection::UnknownSignatureScheme => {
                candidate.signing = Some(Signing {
                    keystore_member: "keys/key".into(),
                    key_alias: "key".into(),
                    schemes: vec!["v4".into()],
                })
            }
            Rejection::EmptySourceSet => candidate.java_sources.clear(),
            _ => unreachable!(),
        }
        assert!(
            matches!(PreparedApk::bind(candidate, inputs.clone(), &reviewed, &runtime, dir.path()), Err(Error::Plan(observed)) if observed == category)
        );
    }
    let mut changed_runtime = inputs.clone();
    changed_runtime.runtime_glibc.identity.nar_sha256 = "f".repeat(64);
    assert!(matches!(
        PreparedApk::bind(plan.clone(), changed_runtime, &reviewed, &runtime, dir.path()),
        Err(Error::InvalidInput(_))
    ));
    let original = plan.clone();
    plan.toolchain.build_tools.sha256 = "f".repeat(64);
    assert!(matches!(
        PreparedApk::bind(plan, inputs.clone(), &reviewed, &runtime, dir.path()),
        Err(Error::Plan(Rejection::UnboundToolchainIdentity))
    ));
    fs::remove_file(inputs.source_root.join("AndroidManifest.xml")).unwrap();
    assert!(matches!(
        PreparedApk::bind(original.clone(), inputs.clone(), &reviewed, &runtime, dir.path()),
        Err(Error::Plan(Rejection::MissingManifestMember))
    ));
    fs::write(inputs.source_root.join("AndroidManifest.xml"), "<manifest/>").unwrap();
    fs::write(&inputs.build_tools.store_path, b"archive bytes drift").unwrap();
    assert!(matches!(
        PreparedApk::bind(original, inputs, &reviewed, &runtime, dir.path()),
        Err(Error::DigestDrift(_))
    ));
}

#[test]
fn signing_key_must_be_the_declared_source_member_even_with_identical_bytes() {
    let dir = tempfile::tempdir().unwrap();
    let (plan, mut inputs) = fixture(dir.path(), true);
    let reviewed = plan.toolchain.clone();
    let runtime = runtime_cohort(&inputs);
    let alternate = dir.path().join("other-key.jks");
    fs::copy(inputs.keystore.as_ref().unwrap(), &alternate).unwrap();
    inputs.keystore = Some(alternate);
    assert!(matches!(
        PreparedApk::bind(plan, inputs, &reviewed, &runtime, dir.path()),
        Err(Error::Plan(Rejection::KeystorePathEscape))
    ));
}

#[test]
fn symlinked_archive_utility_and_signing_inputs_are_rejected() {
    use std::os::unix::fs::symlink;
    let dir = tempfile::tempdir().unwrap();
    let outside = tempfile::tempdir().unwrap();
    let (plan, inputs) = fixture(dir.path(), true);
    let reviewed = plan.toolchain.clone();
    let runtime = runtime_cohort(&inputs);
    let archive_link = inputs.clone();
    let external_archive = outside.path().join("archive.tar.gz");
    fs::copy(&archive_link.build_tools.store_path, &external_archive).unwrap();
    fs::remove_file(&archive_link.build_tools.store_path).unwrap();
    symlink(&external_archive, &archive_link.build_tools.store_path).unwrap();
    assert!(matches!(
        PreparedApk::bind(plan.clone(), archive_link.clone(), &reviewed, &runtime, dir.path()),
        Err(Error::MissingArchive(_))
    ));
    fs::remove_file(&archive_link.build_tools.store_path).unwrap();
    fs::copy(&external_archive, &archive_link.build_tools.store_path).unwrap();

    let external_utility = outside.path().join("utility");
    fs::copy(&archive_link.utility, &external_utility).unwrap();
    fs::remove_file(&archive_link.utility).unwrap();
    symlink(&external_utility, &archive_link.utility).unwrap();
    assert!(matches!(
        PreparedApk::bind(plan.clone(), archive_link.clone(), &reviewed, &runtime, dir.path()),
        Err(Error::InvalidInput(_))
    ));
    fs::remove_file(&archive_link.utility).unwrap();
    fs::copy(&external_utility, &archive_link.utility).unwrap();
    use std::os::unix::fs::PermissionsExt;
    fs::set_permissions(&archive_link.utility, fs::Permissions::from_mode(0o755)).unwrap();
    let runtime_loader = archive_link.runtime_glibc.store_path.join("lib/ld-linux-x86-64.so.2");
    let external_loader = outside.path().join("runtime-loader");
    fs::copy(&runtime_loader, &external_loader).unwrap();
    fs::remove_file(&runtime_loader).unwrap();
    symlink(&external_loader, &runtime_loader).unwrap();
    assert!(matches!(
        PreparedApk::bind(plan.clone(), archive_link.clone(), &reviewed, &runtime, dir.path()),
        Err(Error::Plan(Rejection::UnboundToolchainIdentity))
    ));
    fs::remove_file(&runtime_loader).unwrap();
    fs::copy(&external_loader, &runtime_loader).unwrap();

    let external_key = outside.path().join("secret.jks");
    fs::copy(archive_link.keystore.as_ref().unwrap(), &external_key).unwrap();
    fs::remove_file(archive_link.keystore.as_ref().unwrap()).unwrap();
    symlink(&external_key, archive_link.keystore.as_ref().unwrap()).unwrap();
    assert!(matches!(
        PreparedApk::bind(plan, archive_link, &reviewed, &runtime, dir.path()),
        Err(Error::InvalidInput(_))
    ));
}
