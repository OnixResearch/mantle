use crunch_project_core::ApplyOutcomesRequest;
use crunch_project_core::PatchResolution;
use crunch_project_core::PatchResolutionPlanRequest;
use crunch_project_core::RefreshFailure;
use crunch_project_core::RefreshInputsPlanRequest;
use crunch_project_core::RefreshInputsRequest;
use crunch_project_core::ResolvedInput;
use crunch_project_core::ResolvedInputState;

use crate::Error;
use crate::GitReference;
use crate::HashAlgo;
use crate::HashResolutionMode;
use crate::InputKind;
use crate::LockEntry;
use crate::LockedHash;
use crate::LockedKind;
use crate::LockedPatch;
use crate::LockedPatchSource;
use crate::Lockfile;
use crate::ManifestInput;
use crate::PatchDef;
use crate::PatchSource;
use crate::ProjectManifest;
use crate::refresh::ApplyResult;
use crate::refresh::RefreshOutcome;
use crate::refresh::StaleReport;

pub trait RefreshResolver {
    fn resolve_git_rev(&self, repository: &str, reference: &GitReference) -> Result<Option<String>, Error>;

    fn hash_url_content(&self, url: &str, algo: &HashAlgo, mode: HashResolutionMode) -> Result<Option<String>, Error>;

    fn hash_git_checkout(&self, repository: &str, rev: &str, algo: &HashAlgo) -> Result<Option<String>, Error> {
        let _ = (repository, rev, algo);
        Ok(None)
    }

    fn hash_local_file(&self, path: &str, algo: &HashAlgo) -> Result<Option<String>, Error> {
        let _ = (path, algo);
        Ok(None)
    }
}

pub fn refresh_inputs(
    manifest: &ProjectManifest,
    lock: &Lockfile,
    selected: &[String],
    resolver: &dyn RefreshResolver,
) -> Vec<RefreshOutcome> {
    let request = build_refresh_request(manifest, lock, selected, resolver);
    crunch_project_core::refresh_inputs(request)
}

pub fn apply_outcomes(
    manifest: &ProjectManifest,
    lock: &Lockfile,
    outcomes: &[RefreshOutcome],
    resolver: &dyn RefreshResolver,
) -> ApplyResult {
    let patch_resolutions = resolve_needed_patches(manifest, lock, outcomes, resolver);
    crunch_project_core::apply_outcomes(ApplyOutcomesRequest {
        manifest: manifest.clone(),
        lock: lock.clone(),
        outcomes: outcomes.to_vec(),
        patch_resolutions,
    })
}

pub fn list_stale(manifest: &ProjectManifest, lock: &Lockfile, resolver: &dyn RefreshResolver) -> StaleReport {
    let request = build_refresh_request(manifest, lock, &[], resolver);
    crunch_project_core::list_stale(request)
}

fn build_refresh_request(
    manifest: &ProjectManifest,
    lock: &Lockfile,
    selected: &[String],
    resolver: &dyn RefreshResolver,
) -> RefreshInputsRequest {
    let plan = crunch_project_core::plan_refresh_inputs(RefreshInputsPlanRequest {
        manifest: manifest.clone(),
        selected: selected.to_vec(),
    });
    let resolutions = plan
        .into_iter()
        .filter(|input| !input.frozen)
        .map(|input| resolve_manifest_input(&input, resolver))
        .collect();
    RefreshInputsRequest {
        manifest: manifest.clone(),
        lock: lock.clone(),
        selected: selected.to_vec(),
        resolutions,
    }
}

fn resolve_manifest_input(input: &ManifestInput, resolver: &dyn RefreshResolver) -> ResolvedInputState {
    match resolve_input(input, resolver) {
        Ok(entry) => ResolvedInputState::Resolved(ResolvedInput {
            name: input.name.clone(),
            entry,
        }),
        Err(err) => ResolvedInputState::Failed(RefreshFailure {
            name: input.name.clone(),
            reason: err.to_string(),
        }),
    }
}

fn require_resolution(value: Option<String>, what: &str) -> Result<String, Error> {
    let resolved = value.ok_or_else(|| Error::Manifest(format!("unable to resolve {what}")))?;
    if resolved.is_empty() {
        return Err(Error::Manifest(format!("resolver returned empty {what}")));
    }
    Ok(resolved)
}

fn resolve_input(input: &ManifestInput, resolver: &dyn RefreshResolver) -> Result<LockEntry, Error> {
    assert!(!input.name.is_empty(), "input name must not be empty");
    assert!(
        input.patches.len() as u64 <= crate::manifest::MAX_PATCHES_PER_INPUT as u64,
        "patch count must stay within manifest limit"
    );
    let (kind, hash) = match &input.kind {
        InputKind::File { url } => {
            let hash_value = require_resolution(
                resolver.hash_url_content(url, &input.hash.algo, HashResolutionMode::Flat)?,
                &format!("flat hash for {}", input.name),
            )?;
            (LockedKind::File { url: url.clone() }, LockedHash {
                algo: input.hash.algo.clone(),
                value: hash_value,
            })
        }
        InputKind::Tarball { url } => {
            let hash_value = require_resolution(
                resolver.hash_url_content(url, &input.hash.algo, HashResolutionMode::Recursive)?,
                &format!("tarball tree hash for {}", input.name),
            )?;
            (LockedKind::Tarball { url: url.clone() }, LockedHash {
                algo: input.hash.algo.clone(),
                value: hash_value,
            })
        }
        InputKind::Git { repository, reference } => {
            let rev = require_resolution(
                resolver.resolve_git_rev(repository, reference)?,
                &format!("git revision for {}", input.name),
            )?;
            let ref_name = match reference {
                GitReference::Branch(branch) => Some(branch.clone()),
                GitReference::Tag(tag) => Some(tag.clone()),
                GitReference::Rev(_) => None,
            };
            let hash_value = require_resolution(
                resolver.hash_git_checkout(repository, &rev, &input.hash.algo)?,
                &format!("git tree hash for {}", input.name),
            )?;
            (
                LockedKind::Git {
                    repository: repository.clone(),
                    rev,
                    ref_name,
                },
                LockedHash {
                    algo: input.hash.algo.clone(),
                    value: hash_value,
                },
            )
        }
    };

    Ok(LockEntry {
        kind,
        hash,
        patches: input.patches.clone(),
        mirrors: input.mirrors.clone(),
    })
}

fn resolve_needed_patches(
    manifest: &ProjectManifest,
    lock: &Lockfile,
    outcomes: &[RefreshOutcome],
    resolver: &dyn RefreshResolver,
) -> Vec<PatchResolution> {
    let plan = crunch_project_core::plan_patch_resolutions(PatchResolutionPlanRequest {
        manifest: manifest.clone(),
        lock: lock.clone(),
        outcomes: outcomes.to_vec(),
    });
    plan.into_iter().map(|patch_def| resolve_patch_def(&patch_def, resolver)).collect()
}

fn resolve_patch_def(def: &PatchDef, resolver: &dyn RefreshResolver) -> PatchResolution {
    match resolve_patch(def, resolver) {
        Ok(patch) => PatchResolution::Resolved {
            name: def.name.clone(),
            patch,
        },
        Err(err) => PatchResolution::Failed {
            name: def.name.clone(),
            reason: err.to_string(),
        },
    }
}

fn resolve_patch(def: &PatchDef, resolver: &dyn RefreshResolver) -> Result<LockedPatch, Error> {
    match &def.source {
        PatchSource::Local { path } => {
            let hash_value = require_resolution(
                resolver.hash_local_file(path, &HashAlgo::Sha256)?,
                &format!("local patch hash for {path}"),
            )?;
            Ok(LockedPatch {
                source: LockedPatchSource::Local { path: path.clone() },
                hash: LockedHash {
                    algo: HashAlgo::Sha256,
                    value: hash_value,
                },
            })
        }
        PatchSource::Remote { url, hash } => {
            let hash_value = require_resolution(
                resolver.hash_url_content(url, &hash.algo, HashResolutionMode::Flat)?,
                &format!("remote patch hash for {url}"),
            )?;
            Ok(LockedPatch {
                source: LockedPatchSource::Remote { url: url.clone() },
                hash: LockedHash {
                    algo: hash.algo.clone(),
                    value: hash_value,
                },
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::*;
    use crate::HashSpec;
    use crate::SchemaVersion;

    struct MockResolver {
        git_rev: Option<String>,
        git_hash: Option<String>,
        url_hash: Option<String>,
        local_hash: Option<String>,
    }

    impl RefreshResolver for MockResolver {
        fn resolve_git_rev(&self, _repository: &str, _reference: &GitReference) -> Result<Option<String>, Error> {
            Ok(self.git_rev.clone())
        }

        fn hash_url_content(
            &self,
            _url: &str,
            _algo: &HashAlgo,
            _mode: HashResolutionMode,
        ) -> Result<Option<String>, Error> {
            Ok(self.url_hash.clone())
        }

        fn hash_git_checkout(&self, _repository: &str, _rev: &str, _algo: &HashAlgo) -> Result<Option<String>, Error> {
            Ok(self.git_hash.clone())
        }

        fn hash_local_file(&self, _path: &str, _algo: &HashAlgo) -> Result<Option<String>, Error> {
            Ok(self.local_hash.clone())
        }
    }

    #[test]
    fn shell_adapter_keeps_refresh_io_outside_core() {
        let manifest = ProjectManifest {
            version: "1.0.0".into(),
            inputs: vec![ManifestInput {
                name: "pkg".into(),
                kind: InputKind::File {
                    url: "https://example.com/pkg".into(),
                },
                hash: HashSpec::default(),
                frozen: false,
                mirrors: vec![],
                patches: vec!["fix1".into()],
            }],
            patches: vec![PatchDef {
                name: "fix1".into(),
                source: PatchSource::Local {
                    path: "patches/fix1.patch".into(),
                },
            }],
        };
        let resolver = MockResolver {
            git_rev: None,
            git_hash: None,
            url_hash: Some("sha256-data=".into()),
            local_hash: Some("sha256-patch=".into()),
        };
        let lock = Lockfile {
            version: SchemaVersion::CURRENT,
            inputs: BTreeMap::new(),
            patches: BTreeMap::new(),
        };

        let outcomes = refresh_inputs(&manifest, &lock, &[], &resolver);
        let result = apply_outcomes(&manifest, &lock, &outcomes, &resolver);

        assert!(matches!(&outcomes[0], RefreshOutcome::Updated(resolved) if resolved.name == "pkg"));
        assert_eq!(result.lock.inputs["pkg"].hash.value, "sha256-data=");
        assert_eq!(result.lock.patches["fix1"].hash.value, "sha256-patch=");
    }

    #[test]
    fn adapter_preserves_failed_resolution_behavior() {
        let manifest = ProjectManifest {
            version: "1.0.0".into(),
            inputs: vec![ManifestInput {
                name: "broken".into(),
                kind: InputKind::File {
                    url: "https://example.com/broken".into(),
                },
                hash: HashSpec::default(),
                frozen: false,
                mirrors: vec![],
                patches: vec![],
            }],
            patches: vec![],
        };
        struct FailingResolver;
        impl RefreshResolver for FailingResolver {
            fn resolve_git_rev(&self, _: &str, _: &GitReference) -> Result<Option<String>, Error> {
                Err(Error::Manifest("network down".into()))
            }
            fn hash_url_content(&self, _: &str, _: &HashAlgo, _: HashResolutionMode) -> Result<Option<String>, Error> {
                Err(Error::Manifest("network down".into()))
            }
        }

        let outcomes = refresh_inputs(
            &manifest,
            &Lockfile {
                version: SchemaVersion::CURRENT,
                inputs: BTreeMap::new(),
                patches: BTreeMap::new(),
            },
            &[],
            &FailingResolver,
        );
        assert!(
            matches!(&outcomes[0], RefreshOutcome::Failed { name, reason } if name == "broken" && reason.contains("network down"))
        );
    }
}
