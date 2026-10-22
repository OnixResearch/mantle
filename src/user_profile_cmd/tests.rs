#![cfg(target_os = "linux")]

use std::os::unix::fs::PermissionsExt;

use nix_compat::derivation::Derivation;
use nix_compat::derivation::Output;
use nix_compat::narinfo::SigningKey;
use nix_compat::narinfo::fingerprint_with_store_dir;
use nix_compat::store_path::StorePath;
use snix_store::pathinfoservice::PathInfo;

use super::*;

const PREFIX: &str = "/mantle/store";

struct Fixture {
    root: tempfile::TempDir,
    backend: crunch_store::StoreBackend,
    signing: SigningKey<ed25519_dalek::SigningKey>,
    verifying: VerifyingKey,
}

impl Fixture {
    fn new(backend: crunch_store::StoreBackend) -> Self {
        let root = tempfile::tempdir().unwrap();
        std::fs::create_dir(root.path().join("state")).unwrap();
        std::fs::create_dir(root.path().join("outputs")).unwrap();
        let raw = ed25519_dalek::SigningKey::from_bytes(&[39; 32]);
        let verifying = VerifyingKey::new("profile-fixture".into(), raw.verifying_key());
        let signing = SigningKey::new("profile-fixture".into(), raw);
        std::fs::write(root.path().join("state/casita-trusted-public-keys"), format!("{verifying}\n")).unwrap();
        Self {
            root,
            backend,
            signing,
            verifying,
        }
    }

    fn config(&self) -> StoreConfig {
        StoreConfig::new(self.backend, self.root.path().join("state"), self.root.path().join("outputs"), PREFIX.into())
    }

    fn profile(&self) -> PathBuf {
        self.root.path().join("profile")
    }

    async fn run(
        &self,
        operation: ProfileOperation,
        expected: Option<u64>,
        apply: bool,
    ) -> Result<ProfileReport, String> {
        execute_profile(ProfileRequest {
            profile: self.profile(),
            store: self.config(),
            trusted_keys: vec![self.verifying.clone()],
            expected_active: expected,
            apply,
            operation,
        })
        .await
    }

    async fn package(&self, name: &str, version: u8, binary: &str) -> Package {
        let source = self.root.path().join(format!("source-{name}-{version}"));
        std::fs::create_dir_all(source.join("bin")).unwrap();
        let executable = source.join("bin").join(binary);
        std::fs::write(&executable, format!("#!/bin/sh\nprintf 'version-{version}\\n'\n")).unwrap();
        std::fs::set_permissions(&executable, std::fs::Permissions::from_mode(0o755)).unwrap();
        let store = StoreHandle::open(self.config()).await.unwrap();
        let node = store.transfer_objects().ingest_host_path(&source).await.unwrap();
        let nar = crunch_nar::observe_path_blocking(
            source,
            crunch_nar::FilesystemNarRequest::new(
                nix_compat::nixhash::HashAlgo::Sha256,
                crunch_nar::CaseHackPolicy::native(),
                1_048_576,
            ),
        )
        .await
        .unwrap();
        let output = StorePath::from_name_and_digest_fixed(&format!("{name}-{version}"), [version; 20]).unwrap();
        let mut derivation = Derivation {
            system: host_platform().unwrap(),
            builder: "/bin/sh".into(),
            ..Derivation::default()
        };
        derivation.outputs.insert("out".into(), Output {
            path: Some(output.clone()),
            ca_hash: None,
        });
        derivation.environment.insert("name".into(), format!("{name}-{version}").into());
        let drv = derivation.calculate_derivation_path_with_store_dir(&format!("{name}-{version}"), PREFIX).unwrap();
        let mut info = PathInfo {
            store_path: output.clone(),
            node: node.clone(),
            references: Vec::new(),
            nar_size: nar.nar_size,
            nar_sha256: nar.digest.digest_as_bytes().try_into().unwrap(),
            signatures: Vec::new(),
            deriver: Some(drv.clone()),
            ca: None,
        };
        let references = Vec::<nix_compat::store_path::StorePathRef<'_>>::new();
        let fingerprint =
            fingerprint_with_store_dir(&output.as_ref(), &info.nar_sha256, info.nar_size, references.iter(), PREFIX);
        info.signatures.push(self.signing.sign(fingerprint.as_bytes()).to_owned());
        store.pathinfo_service().put(info.clone()).await.unwrap();
        store
            .transfer_objects()
            .export_node_to_output(&node, &output.to_absolute_path_with_prefix(PREFIX))
            .await
            .unwrap();
        package_from_derivation(name.into(), &derivation, &drv, "out".into(), &info, PREFIX).unwrap()
    }

    fn observed_version(&self) -> String {
        let output = std::process::Command::new(self.profile().join("current/bin/tool")).output().unwrap();
        assert!(output.status.success());
        String::from_utf8(output.stdout).unwrap()
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn exact_install_upgrade_remove_rollback_and_gc_on_both_backends() {
    for backend in [crunch_store::StoreBackend::Snix, crunch_store::StoreBackend::Casita] {
        let fixture = Fixture::new(backend);
        let one = fixture.package("tool", 1, "tool").await;
        let two = fixture.package("tool", 2, "tool").await;
        let planned = fixture
            .run(
                ProfileOperation::Install {
                    package: one.clone(),
                    replace: false,
                },
                None,
                false,
            )
            .await
            .unwrap();
        assert_eq!(planned.target_generation, Some(1));
        assert!(!fixture.profile().join("current").exists());
        fixture
            .run(
                ProfileOperation::Install {
                    package: one.clone(),
                    replace: false,
                },
                None,
                true,
            )
            .await
            .unwrap();
        assert_eq!(fixture.observed_version(), "version-1\n");
        assert!(
            fixture
                .run(
                    ProfileOperation::Install {
                        package: two.clone(),
                        replace: false
                    },
                    Some(1),
                    true
                )
                .await
                .is_err()
        );
        fixture
            .run(
                ProfileOperation::Install {
                    package: two.clone(),
                    replace: true,
                },
                Some(1),
                true,
            )
            .await
            .unwrap();
        assert_eq!(fixture.observed_version(), "version-2\n");
        fixture.run(ProfileOperation::Remove { name: "tool".into() }, Some(2), true).await.unwrap();
        assert!(!fixture.profile().join("current/bin/tool").exists());
        fixture.run(ProfileOperation::Rollback, Some(3), true).await.unwrap();
        assert_eq!(fixture.observed_version(), "version-2\n");
        fixture.run(ProfileOperation::Switch { generation: 1 }, Some(2), true).await.unwrap();
        assert_eq!(fixture.observed_version(), "version-1\n");
        let listing = fixture.run(ProfileOperation::List, None, false).await.unwrap();
        assert_eq!(listing.manifest.unwrap().packages, vec![one.clone()]);
        assert_eq!(listing.retained_generations, vec![1, 2, 3]);
        let guard = StoreMutationGuard::try_acquire(&fixture.config().state_dir).unwrap();
        let mut store = StoreHandle::open(fixture.config()).await.unwrap();
        store.recover_casita_gc_under_guard(&guard).await.unwrap();
        let retained = store.garbage_collect_under_guard(&guard, None).await.unwrap();
        assert!(!retained.candidate_paths.contains(&one.output_path));
        assert!(!retained.candidate_paths.contains(&two.output_path));
        drop(store);
        drop(guard);
        assert!(fixture.run(ProfileOperation::RemoveGeneration { generation: 1 }, Some(1), true).await.is_err());
        fixture.run(ProfileOperation::RemoveGeneration { generation: 2 }, Some(1), true).await.unwrap();
        let guard = StoreMutationGuard::try_acquire(&fixture.config().state_dir).unwrap();
        let mut store = StoreHandle::open(fixture.config()).await.unwrap();
        let planned = store.garbage_collect_under_guard(&guard, None).await.unwrap();
        assert!(planned.candidate_paths.contains(&two.output_path));
        assert!(!planned.candidate_paths.contains(&one.output_path));
        let collected = store.garbage_collect_under_guard(&guard, Some(&planned.plan_id)).await.unwrap();
        assert!(collected.execution_complete);
        assert_eq!(fixture.observed_version(), "version-1\n");
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn rejects_conflict_missing_tamper_platform_and_untrusted_outputs_without_switching() {
    let fixture = Fixture::new(crunch_store::StoreBackend::Snix);
    let one = fixture.package("tool", 1, "tool").await;
    let conflict = fixture.package("other", 2, "tool").await;
    fixture
        .run(
            ProfileOperation::Install {
                package: one.clone(),
                replace: false,
            },
            None,
            true,
        )
        .await
        .unwrap();
    let rejected = fixture
        .run(
            ProfileOperation::Install {
                package: conflict,
                replace: false,
            },
            Some(1),
            true,
        )
        .await
        .unwrap_err();
    assert!(rejected.contains("conflict"));
    let two = fixture.package("tool", 3, "tool").await;
    let physical = fixture.config().output_dir.join(two.output_path.rsplit('/').next().unwrap());
    std::fs::set_permissions(physical.join("bin/tool"), std::fs::Permissions::from_mode(0o755)).unwrap();
    std::fs::write(physical.join("bin/tool"), b"tampered").unwrap();
    assert!(
        fixture
            .run(
                ProfileOperation::Install {
                    package: two.clone(),
                    replace: true
                },
                Some(1),
                true
            )
            .await
            .unwrap_err()
            .contains("tampered")
    );
    writable_directories(&physical);
    std::fs::remove_dir_all(&physical).unwrap();
    assert!(
        fixture
            .run(
                ProfileOperation::Install {
                    package: two,
                    replace: true
                },
                Some(1),
                true
            )
            .await
            .is_err()
    );
    let mut wrong_platform = one.clone();
    wrong_platform.derivation_aterm = wrong_platform.derivation_aterm.replace(&host_platform().unwrap(), "wasm32-wasi");
    assert!(
        fixture
            .run(
                ProfileOperation::Install {
                    package: wrong_platform,
                    replace: true
                },
                Some(1),
                true
            )
            .await
            .unwrap_err()
            .contains("platform")
    );
    let raw = ed25519_dalek::SigningKey::from_bytes(&[88; 32]);
    let request = ProfileRequest {
        profile: fixture.profile(),
        store: fixture.config(),
        trusted_keys: vec![VerifyingKey::new("untrusted".into(), raw.verifying_key())],
        expected_active: Some(1),
        apply: true,
        operation: ProfileOperation::Switch { generation: 1 },
    };
    assert!(execute_profile(request).await.unwrap_err().contains("trusted signature"));
    assert_eq!(fixture.observed_version(), "version-1\n");
    assert_eq!(publication::active(&fixture.profile()).unwrap(), Some(1));
    assert_eq!(publication::generations(&fixture.profile()).unwrap(), vec![1]);
}

#[tokio::test(flavor = "multi_thread")]
async fn rejects_concurrent_and_stale_writers_and_preserves_shared_generation_owners() {
    let fixture = Fixture::new(crunch_store::StoreBackend::Snix);
    let one = fixture.package("tool", 1, "tool").await;
    fixture
        .run(
            ProfileOperation::Install {
                package: one.clone(),
                replace: false,
            },
            None,
            true,
        )
        .await
        .unwrap();
    let lock = publication::lock_profile(&fixture.profile()).unwrap();
    assert!(
        fixture
            .run(ProfileOperation::Remove { name: "tool".into() }, Some(1), true)
            .await
            .unwrap_err()
            .contains("profile writer")
    );
    drop(lock);
    let guard = StoreMutationGuard::try_acquire(&fixture.config().state_dir).unwrap();
    assert!(
        fixture
            .run(ProfileOperation::Remove { name: "tool".into() }, Some(1), true)
            .await
            .unwrap_err()
            .contains("active")
    );
    drop(guard);
    assert!(
        fixture
            .run(ProfileOperation::Remove { name: "tool".into() }, None, true)
            .await
            .unwrap_err()
            .contains("stale")
    );
    let other = fixture.package("other", 2, "other").await;
    fixture
        .run(
            ProfileOperation::Install {
                package: other,
                replace: false,
            },
            Some(1),
            true,
        )
        .await
        .unwrap();
    fixture.run(ProfileOperation::RemoveGeneration { generation: 1 }, Some(2), true).await.unwrap();
    let store = StoreHandle::open(fixture.config()).await.unwrap();
    let roots = store.into_pipeline_store_parts().root_registry.list().unwrap();
    assert!(roots.iter().any(|root| root.logical_path == one.output_path));
    assert_eq!(fixture.observed_version(), "version-1\n");
}

#[tokio::test(flavor = "multi_thread")]
async fn publication_interruption_and_pending_root_recovery_preserve_old_active() {
    let fixture = Fixture::new(crunch_store::StoreBackend::Snix);
    let one = fixture.package("tool", 1, "tool").await;
    fixture
        .run(
            ProfileOperation::Install {
                package: one,
                replace: false,
            },
            None,
            true,
        )
        .await
        .unwrap();
    let current = publication::read_generation(&fixture.profile(), 1).unwrap();
    let pending = Generation {
        number: 2,
        parent: Some(1),
        ..current
    };
    publication::allocate(&fixture.profile(), 2).unwrap();
    let stage = publication::stage(&fixture.profile(), &pending).unwrap();
    publication::record_pending(&fixture.profile(), &pending).unwrap();
    let store = StoreHandle::open(fixture.config()).await.unwrap();
    let parts = store.into_pipeline_store_parts();
    let binding: publication::Binding =
        publication::read_optional(&fixture.profile().join("binding.json")).unwrap().unwrap();
    let keys = [fixture.verifying.clone()];
    let context = MutationContext {
        profile: &fixture.profile(),
        binding: &binding,
        roots: &parts.root_registry,
        authority: admission::Admission {
            lookup: &parts.output_lookup,
            content: &parts.build_service_store,
            output_dir: &binding.output_dir,
            trusted_keys: &keys,
        },
    };
    retain(&context, &pending).await.unwrap();
    drop(stage);
    drop(parts);
    assert_eq!(fixture.observed_version(), "version-1\n");
    let pending_report = fixture.run(ProfileOperation::Generations, None, false).await.unwrap();
    assert_eq!(pending_report.pending_generations, vec![2]);
    fixture.run(ProfileOperation::RemoveGeneration { generation: 2 }, Some(1), true).await.unwrap();
    let reopened = StoreHandle::open(fixture.config()).await.unwrap().into_pipeline_store_parts();
    let roots = reopened.root_registry.list().unwrap();
    assert!(
        roots
            .iter()
            .all(|root| root.owner_scope != format!("operator:{}", owner_label(&fixture.profile(), 2)))
    );
    assert_eq!(fixture.observed_version(), "version-1\n");
    assert_eq!(publication::next_number(&fixture.profile(), &[1]).unwrap(), 3);
}

#[tokio::test(flavor = "multi_thread")]
async fn simultaneous_updates_have_one_winner_and_retained_forest_tamper_cannot_activate() {
    let fixture = Fixture::new(crunch_store::StoreBackend::Snix);
    let one = fixture.package("tool", 1, "tool").await;
    let two = fixture.package("tool", 2, "tool").await;
    fixture
        .run(
            ProfileOperation::Install {
                package: one,
                replace: false,
            },
            None,
            true,
        )
        .await
        .unwrap();
    let (left, right) = tokio::join!(
        fixture.run(
            ProfileOperation::Install {
                package: two.clone(),
                replace: true
            },
            Some(1),
            true
        ),
        fixture.run(
            ProfileOperation::Install {
                package: two,
                replace: true
            },
            Some(1),
            true
        ),
    );
    assert_ne!(left.is_ok(), right.is_ok(), "exactly one compare-and-swap writer may win");
    assert_eq!(publication::active(&fixture.profile()).unwrap(), Some(2));
    let leaf = fixture.profile().join("generations/1/bin/tool");
    std::fs::remove_file(&leaf).unwrap();
    std::os::unix::fs::symlink("/bin/false", &leaf).unwrap();
    let error = fixture.run(ProfileOperation::Switch { generation: 1 }, Some(2), true).await.unwrap_err();
    assert!(error.contains("tampered"));
    assert_eq!(fixture.observed_version(), "version-2\n");
    assert_eq!(publication::active(&fixture.profile()).unwrap(), Some(2));
}

#[tokio::test(flavor = "multi_thread")]
async fn published_but_unselected_generation_is_retained_without_changing_current() {
    let fixture = Fixture::new(crunch_store::StoreBackend::Snix);
    let one = fixture.package("tool", 1, "tool").await;
    fixture
        .run(
            ProfileOperation::Install {
                package: one,
                replace: false,
            },
            None,
            true,
        )
        .await
        .unwrap();
    let current = publication::read_generation(&fixture.profile(), 1).unwrap();
    let candidate = Generation {
        number: 2,
        parent: Some(1),
        ..current
    };
    publication::allocate(&fixture.profile(), 2).unwrap();
    let stage = publication::stage(&fixture.profile(), &candidate).unwrap();
    publication::record_pending(&fixture.profile(), &candidate).unwrap();
    let parts = StoreHandle::open(fixture.config()).await.unwrap().into_pipeline_store_parts();
    let binding: publication::Binding =
        publication::read_optional(&fixture.profile().join("binding.json")).unwrap().unwrap();
    let keys = [fixture.verifying.clone()];
    let context = MutationContext {
        profile: &fixture.profile(),
        binding: &binding,
        roots: &parts.root_registry,
        authority: admission::Admission {
            lookup: &parts.output_lookup,
            content: &parts.build_service_store,
            output_dir: &binding.output_dir,
            trusted_keys: &keys,
        },
    };
    retain(&context, &candidate).await.unwrap();
    publication::publish(&fixture.profile(), stage, 2).unwrap();
    publication::finish_pending(&fixture.profile(), 2).unwrap();
    drop(parts);
    assert_eq!(fixture.observed_version(), "version-1\n");
    assert_eq!(publication::generations(&fixture.profile()).unwrap(), vec![1, 2]);
    fixture.run(ProfileOperation::Switch { generation: 2 }, Some(1), true).await.unwrap();
    assert_eq!(publication::active(&fixture.profile()).unwrap(), Some(2));
    assert_eq!(fixture.observed_version(), "version-1\n");
}

#[tokio::test(flavor = "multi_thread")]
async fn deferred_native_output_requires_exact_recorded_ca_realization() {
    let fixture = Fixture::new(crunch_store::StoreBackend::Snix);
    let original = fixture.package("tool", 1, "tool").await;
    let mut derivation = Derivation::from_aterm_bytes(original.derivation_aterm.as_bytes()).unwrap();
    derivation.outputs.get_mut("out").unwrap().path = None;
    let drv = derivation.calculate_derivation_path_with_store_dir("tool-1", PREFIX).unwrap();
    let mut store = StoreHandle::open(fixture.config()).await.unwrap();
    let output = admission::parse_path(&original.output_path, admission::StorePrefix(PREFIX)).unwrap();
    let mut info = store.pathinfo_service().get(*output.digest()).await.unwrap().unwrap();
    info.deriver = Some(drv.clone());
    store.pathinfo_service().put(info.clone()).await.unwrap();
    let package = package_from_derivation("tool".into(), &derivation, &drv, "out".into(), &info, PREFIX).unwrap();
    drop(store);
    let missing = fixture
        .run(
            ProfileOperation::Install {
                package: package.clone(),
                replace: false,
            },
            None,
            true,
        )
        .await
        .unwrap_err();
    assert!(missing.contains("realization mapping"));
    assert_eq!(publication::active(&fixture.profile()).unwrap(), None);
    store = StoreHandle::open(fixture.config()).await.unwrap();
    store.insert_ca_mapping(&package.derivation_path, "out", &package.output_path);
    drop(store);
    fixture
        .run(
            ProfileOperation::Install {
                package,
                replace: false,
            },
            None,
            true,
        )
        .await
        .unwrap();
    assert_eq!(fixture.observed_version(), "version-1\n");
    assert_eq!(publication::active(&fixture.profile()).unwrap(), Some(1));
}

#[tokio::test(flavor = "multi_thread")]
async fn cached_export_bytes_without_pathinfo_are_not_an_installable_package() {
    let fixture = Fixture::new(crunch_store::StoreBackend::Snix);
    let mut package = fixture.package("tool", 1, "tool").await;
    let cached: StorePath<String> = StorePath::from_name_and_digest_fixed("cached-only", [31; 20]).unwrap();
    let source = fixture.config().output_dir.join(package.output_path.rsplit('/').next().unwrap()).join("bin/tool");
    let physical = fixture.config().output_dir.join(cached.to_string());
    std::fs::create_dir_all(physical.join("bin")).unwrap();
    std::fs::copy(source, physical.join("bin/tool")).unwrap();
    package.output_path = cached.to_absolute_path_with_prefix(PREFIX);
    let error = fixture
        .run(
            ProfileOperation::Install {
                package,
                replace: false,
            },
            None,
            true,
        )
        .await
        .unwrap_err();
    assert!(error.contains("not admitted"));
    assert_eq!(publication::active(&fixture.profile()).unwrap(), None);
    assert!(physical.join("bin/tool").is_file());
}

#[tokio::test(flavor = "multi_thread")]
async fn runtime_reference_closure_is_admitted_and_retained_not_just_the_root() {
    let fixture = Fixture::new(crunch_store::StoreBackend::Snix);
    let package = fixture.package("tool", 1, "tool").await;
    let runtime = fixture.package("runtime", 2, "runtime").await;
    let path = admission::parse_path(&package.output_path, admission::StorePrefix(PREFIX)).unwrap();
    let reference = admission::parse_path(&runtime.output_path, admission::StorePrefix(PREFIX)).unwrap();
    let store = StoreHandle::open(fixture.config()).await.unwrap();
    let mut info = store.pathinfo_service().get(*path.digest()).await.unwrap().unwrap();
    info.references.push(reference.clone());
    let refs: Vec<_> = info.references.iter().map(|path| path.as_ref()).collect();
    let fingerprint = fingerprint_with_store_dir(&path.as_ref(), &info.nar_sha256, info.nar_size, refs.iter(), PREFIX);
    info.signatures = vec![fixture.signing.sign(fingerprint.as_bytes()).to_owned()];
    store.pathinfo_service().put(info).await.unwrap();
    drop(store);
    fixture
        .run(
            ProfileOperation::Install {
                package,
                replace: false,
            },
            None,
            true,
        )
        .await
        .unwrap();
    let guard = StoreMutationGuard::try_acquire(&fixture.config().state_dir).unwrap();
    let mut store = StoreHandle::open(fixture.config()).await.unwrap();
    let planned = store.garbage_collect_under_guard(&guard, None).await.unwrap();
    assert!(!planned.candidate_paths.contains(&runtime.output_path));
    drop(store);
    drop(guard);
    writable_directories(&fixture.config().output_dir.join(reference.to_string()));
    std::fs::remove_dir_all(fixture.config().output_dir.join(reference.to_string())).unwrap();
    assert!(fixture.run(ProfileOperation::Switch { generation: 1 }, Some(1), true).await.is_err());
    assert_eq!(fixture.observed_version(), "version-1\n");
    assert_eq!(publication::active(&fixture.profile()).unwrap(), Some(1));
}

impl Drop for Fixture {
    fn drop(&mut self) {
        // Exported directories are deliberately read-only; restore permissions
        // only inside this owned isolated fixture before TempDir cleanup.
        writable_directories(self.root.path());
    }
}

fn writable_directories(root: &Path) {
    let mut pending = vec![root.to_path_buf()];
    while let Some(path) = pending.pop() {
        let Ok(metadata) = std::fs::symlink_metadata(&path) else {
            continue;
        };
        if !metadata.is_dir() {
            continue;
        }
        let _ = std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o755));
        if let Ok(entries) = std::fs::read_dir(path) {
            pending.extend(entries.filter_map(Result::ok).map(|entry| entry.path()));
        }
    }
}
