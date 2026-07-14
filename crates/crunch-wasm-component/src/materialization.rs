use std::collections::BTreeMap;
use std::fs;
use std::path::Path;
use std::path::PathBuf;

use crunch_wasm_component_core::AotAdmission;
use crunch_wasm_component_core::BoundedComponentClaim;
use crunch_wasm_component_core::ComponentStageKind;
use crunch_wasm_component_core::ComponentStageStatus;
use crunch_wasm_component_core::MaterializationBundle;
use crunch_wasm_component_core::MaterializationBundleRequest;
use crunch_wasm_component_core::PackageMaterialization;
use crunch_wasm_component_core::StageReceiptReference;
use crunch_wasm_component_core::StoreObject;
use crunch_wasm_component_core::build_materialization_bundle;
use crunch_wasm_component_core::stage_report_identity;
use crunch_wasm_component_core::verify_materialization_bundle;

use crate::Error;
use crate::copy_source_tree;
use crate::files::copy_regular_file_new;
use crate::files::write_new;
use crate::preflight::PreparedPipeline;
use crate::reporting::ExecutionState;
use crate::stages::StageWorkspace;
use crate::toolchain::hash_file_bounded;
use crate::write_json_new;

const MATERIALIZATION_BUNDLE_FILE: &str = "materialization-bundle.json";
const MAX_STAGE_KEY_BYTES: usize = 128;
const OBJECTS_PER_STAGE_MAX: usize = 2;
const REQUIRED_TOP_LEVEL_OBJECTS: usize = 3;

pub(crate) fn materialize_bundle(
    prepared: &PreparedPipeline,
    workspace: &StageWorkspace,
    state: &mut ExecutionState,
    final_portable: &StoreObject,
    aot: Option<AotAdmission>,
) -> Result<MaterializationBundle, Error> {
    let inputs = publish_bundle_inputs(prepared, workspace)?;
    let stage_receipts = publish_stage_receipts(state, workspace, &inputs, final_portable)?;
    let result = build_materialization_bundle(MaterializationBundleRequest {
        name: prepared.request.manifest.name.clone(),
        manifest_blake3: prepared.manifest_blake3.clone(),
        cohort_blake3: prepared.toolchain.manifest.cohort_identity_blake3.clone(),
        wit_inputs: vec![inputs.wit],
        package_inputs: inputs.packages,
        source_closure: inputs.source,
        lock: inputs.lock,
        final_portable: final_portable.clone(),
        expected_octet_profile_blake3: prepared.toolchain.manifest.octet.profile_identity_blake3.clone(),
        expected_runtime_profile_blake3: prepared
            .request
            .manifest
            .validation_profiles
            .expected_runtime_profile_identity_blake3
            .clone(),
        stage_receipts,
        wizer: None,
        aot,
        non_claims: prepared.request.manifest.non_claims.clone(),
    });
    let Some(bundle) = result.bundle else {
        return Err(crate::preflight::core_blockers("materialization bundle", &result.blockers));
    };
    let bundle_path = workspace.publication_root.join(MATERIALIZATION_BUNDLE_FILE);
    write_json_new(&bundle_path, &bundle)?;
    let bundle_object = file_object_for_publication(&bundle_path, MATERIALIZATION_BUNDLE_FILE, workspace)?;
    verify_materialized_objects(&bundle, |object| publication_path(object, workspace))?;
    let reparsed: MaterializationBundle = serde_json::from_slice(
        &fs::read(&bundle_path)
            .map_err(|error| Error::io("reading materialization bundle for verification", &bundle_path, error))?,
    )
    .map_err(|error| Error::Invalid(format!("parsing materialized bundle: {error}")))?;
    let verified = verify_materialization_bundle(reparsed);
    if verified.bundle.as_ref() != Some(&bundle) || !verified.blockers.is_empty() {
        return Err(crate::preflight::core_blockers("materialized bundle verification", &verified.blockers));
    }
    state.add_artifact("component-materialization-bundle", &bundle_object)?;
    state.push_stage(
        "materialization-bundle",
        ComponentStageKind::MaterializationBundle,
        ComponentStageStatus::Succeeded,
        Some(bundle_object.clone()),
        None,
        Some(bundle.bundle_identity_blake3.clone()),
        vec![BoundedComponentClaim::MaterializationObjectsRehashable],
    )?;
    state.set_materialization_bundle(bundle.clone(), bundle_object)?;
    debug_assert!(workspace.publication_root.join(MATERIALIZATION_BUNDLE_FILE).is_file());
    debug_assert_eq!(bundle.final_portable, *final_portable);
    Ok(bundle)
}

pub fn verify_materialization_bundle_files(bundle: &MaterializationBundle) -> Result<(), Error> {
    let result = verify_materialization_bundle(bundle.clone());
    if result.bundle.as_ref() != Some(bundle) || !result.blockers.is_empty() {
        return Err(crate::preflight::core_blockers("materialization bundle", &result.blockers));
    }
    verify_materialized_objects(bundle, |object| Ok(PathBuf::from(&object.logical_path)))
}

struct PublishedInputs {
    source: StoreObject,
    wit: StoreObject,
    lock: StoreObject,
    packages: Vec<PackageMaterialization>,
}

fn publish_bundle_inputs(prepared: &PreparedPipeline, workspace: &StageWorkspace) -> Result<PublishedInputs, Error> {
    let input_root = workspace.publication_root.join("inputs");
    let package_root = input_root.join("packages");
    fs::create_dir_all(&package_root)
        .map_err(|error| Error::io("creating materialization input directories", &package_root, error))?;
    let measured_source = copy_source_tree(Path::new(&prepared.request.source_root), &input_root.join("source"))?;
    let source = StoreObject {
        logical_path: workspace.final_output.join("inputs/source").display().to_string(),
        digest_blake3: measured_source.digest_blake3,
        size_bytes: measured_source.size_bytes,
    };
    let wit_source = Path::new(&prepared.request.manifest.wit.source.logical_path);
    let wit_destination = input_root.join("selected-world.wit");
    copy_regular_file_new(wit_source, &wit_destination, "selected WIT input")?;
    let wit = file_object_for_publication(&wit_destination, "inputs/selected-world.wit", workspace)?;
    if wit.digest_blake3 != prepared.request.manifest.wit.source.digest_blake3 {
        return Err(Error::Invalid("published WIT input differs from the typed manifest".to_string()));
    }
    let packages = publish_packages(prepared, workspace, &package_root)?;
    let lock = state_artifact(prepared, workspace, "artifacts/wkg.lock")?;
    debug_assert_eq!(source.digest_blake3, prepared.request.manifest.implementation.source.digest_blake3);
    debug_assert_eq!(packages.len(), prepared.request.package_materializations.len());
    Ok(PublishedInputs {
        source,
        wit,
        lock,
        packages,
    })
}

fn publish_packages(
    prepared: &PreparedPipeline,
    workspace: &StageWorkspace,
    package_root: &Path,
) -> Result<Vec<PackageMaterialization>, Error> {
    let mut packages = Vec::with_capacity(prepared.request.package_materializations.len());
    for (index, package) in prepared.request.package_materializations.iter().enumerate() {
        let name = format!("package-{index}.wasm");
        let destination = package_root.join(&name);
        copy_regular_file_new(Path::new(&package.object.logical_path), &destination, "component package input")?;
        let object = file_object_for_publication(&destination, &format!("inputs/packages/{name}"), workspace)?;
        if object.digest_blake3 != package.object.digest_blake3 || object.size_bytes != package.object.size_bytes {
            return Err(Error::Invalid(format!("published package input differs for `{}`", package.package)));
        }
        let mut published = package.clone();
        published.object = object;
        packages.push(published);
    }
    debug_assert_eq!(packages.len(), prepared.request.package_materializations.len());
    debug_assert!(packages.iter().all(|package| package.object.logical_path.starts_with('/')));
    Ok(packages)
}

fn state_artifact(
    _prepared: &PreparedPipeline,
    workspace: &StageWorkspace,
    relative_path: &str,
) -> Result<StoreObject, Error> {
    let path = workspace.publication_root.join(relative_path);
    file_object_for_publication(&path, relative_path, workspace)
}

fn publish_stage_receipts(
    state: &ExecutionState,
    workspace: &StageWorkspace,
    inputs: &PublishedInputs,
    final_portable: &StoreObject,
) -> Result<Vec<StageReceiptReference>, Error> {
    let root = workspace.publication_root.join("stage-receipts");
    fs::create_dir(&root).map_err(|error| Error::io("creating stage receipt directory", &root, error))?;
    let mut receipts = Vec::with_capacity(state.stage_inputs.len());
    for input in &state.stage_inputs {
        validate_stage_key(&input.stage_key)?;
        let identity = stage_report_identity(input.clone())
            .map_err(|error| Error::Invalid(format!("identifying stage receipt `{}`: {error}", input.stage_key)))?;
        let bytes = serde_json::to_vec(input)
            .map_err(|error| Error::Invalid(format!("serializing stage receipt `{}`: {error}", input.stage_key)))?;
        let name = format!("{}.json", input.stage_key);
        let path = root.join(&name);
        write_new(&path, &bytes)?;
        let receipt_object = file_object_for_publication(&path, &format!("stage-receipts/{name}"), workspace)?;
        if receipt_object.digest_blake3 != identity {
            return Err(Error::Invalid(format!("stage receipt bytes drifted for `{}`", input.stage_key)));
        }
        receipts.push(StageReceiptReference {
            stage_key: input.stage_key.clone(),
            kind: input.kind,
            receipt_blake3: identity,
            receipt: receipt_object,
            artifact: input.artifact.as_ref().map(|artifact| remap_artifact(artifact, inputs, final_portable)),
        });
    }
    debug_assert_eq!(receipts.len(), state.stage_inputs.len());
    debug_assert!(receipts.iter().all(|receipt| receipt.receipt.digest_blake3 == receipt.receipt_blake3));
    Ok(receipts)
}

fn remap_artifact(artifact: &StoreObject, inputs: &PublishedInputs, final_portable: &StoreObject) -> StoreObject {
    if artifact.digest_blake3 == inputs.lock.digest_blake3 {
        return inputs.lock.clone();
    }
    if artifact.digest_blake3 == final_portable.digest_blake3 {
        return final_portable.clone();
    }
    if let Some(package) = inputs.packages.iter().find(|package| package.object.digest_blake3 == artifact.digest_blake3)
    {
        return package.object.clone();
    }
    artifact.clone()
}

fn verify_materialized_objects<F>(bundle: &MaterializationBundle, mut resolve: F) -> Result<(), Error>
where F: FnMut(&StoreObject) -> Result<PathBuf, Error> {
    let mut objects = bundle_objects(bundle);
    objects.sort_by(|left, right| left.logical_path.cmp(&right.logical_path));
    let mut expected = BTreeMap::new();
    for object in objects {
        if let Some(prior) = expected.insert(object.logical_path.clone(), object.clone())
            && prior != object
        {
            return Err(Error::Invalid(format!(
                "materialization locator has conflicting identities: {}",
                object.logical_path
            )));
        }
    }
    for object in expected.values() {
        let path = resolve(object)?;
        remeasure_object(&path, object)?;
    }
    debug_assert!(!expected.is_empty());
    debug_assert!(expected.values().all(|object| object.size_bytes > 0));
    Ok(())
}

fn bundle_objects(bundle: &MaterializationBundle) -> Vec<StoreObject> {
    let stage_object_count_max = bundle.stage_receipts.len().saturating_mul(OBJECTS_PER_STAGE_MAX);
    let input_object_count = bundle.wit_inputs.len().saturating_add(bundle.package_inputs.len());
    let optional_object_count = usize::from(bundle.wizer.is_some()).saturating_add(usize::from(bundle.aot.is_some()));
    let object_count_max = input_object_count
        .saturating_add(stage_object_count_max)
        .saturating_add(optional_object_count)
        .saturating_add(REQUIRED_TOP_LEVEL_OBJECTS);
    let mut objects = Vec::with_capacity(object_count_max);
    objects.extend(bundle.wit_inputs.clone());
    objects.extend(bundle.package_inputs.iter().map(|package| package.object.clone()));
    objects.extend([
        bundle.source_closure.clone(),
        bundle.lock.clone(),
        bundle.final_portable.clone(),
    ]);
    for stage in &bundle.stage_receipts {
        objects.push(stage.receipt.clone());
        if let Some(artifact) = &stage.artifact {
            objects.push(artifact.clone());
        }
    }
    if let Some(wizer) = &bundle.wizer {
        objects.push(wizer.output.clone());
    }
    if let Some(aot) = &bundle.aot {
        objects.push(aot.output.clone());
    }
    debug_assert!(objects.len() <= object_count_max);
    debug_assert!(objects.len() >= input_object_count.saturating_add(REQUIRED_TOP_LEVEL_OBJECTS));
    objects
}

fn remeasure_object(path: &Path, expected: &StoreObject) -> Result<(), Error> {
    let metadata = fs::symlink_metadata(path)
        .map_err(|error| Error::io("reading materialization object metadata", path, error))?;
    let observed = if metadata.file_type().is_dir() {
        let temporary = tempfile::Builder::new()
            .prefix("mantle-materialization-remeasure-")
            .tempdir()
            .map_err(|error| Error::io("creating materialization remeasurement directory", path, error))?;
        copy_source_tree(path, &temporary.path().join("source"))?
    } else if metadata.file_type().is_file() {
        StoreObject {
            logical_path: expected.logical_path.clone(),
            digest_blake3: hash_file_bounded(path)?,
            size_bytes: metadata.len(),
        }
    } else {
        return Err(Error::Invalid(format!(
            "materialization object is not a regular file or directory: {}",
            path.display()
        )));
    };
    if observed.digest_blake3 != expected.digest_blake3 || observed.size_bytes != expected.size_bytes {
        return Err(Error::Invalid(format!("materialization object bytes drifted: {}", expected.logical_path)));
    }
    debug_assert_eq!(observed.digest_blake3, expected.digest_blake3);
    debug_assert_eq!(observed.size_bytes, expected.size_bytes);
    Ok(())
}

fn publication_path(object: &StoreObject, workspace: &StageWorkspace) -> Result<PathBuf, Error> {
    let logical = Path::new(&object.logical_path);
    let relative = logical.strip_prefix(&workspace.final_output).map_err(|_| {
        Error::Invalid(format!("materialization object locator escapes the publication root: {}", object.logical_path))
    })?;
    if relative.components().any(|component| !matches!(component, std::path::Component::Normal(_))) {
        return Err(Error::Invalid(format!("materialization locator is not canonical: {}", object.logical_path)));
    }
    Ok(workspace.publication_root.join(relative))
}

fn file_object_for_publication(
    path: &Path,
    relative_path: &str,
    workspace: &StageWorkspace,
) -> Result<StoreObject, Error> {
    let metadata = fs::symlink_metadata(path)
        .map_err(|error| Error::io("reading published materialization object metadata", path, error))?;
    if !metadata.file_type().is_file() || metadata.len() == 0 {
        return Err(Error::Invalid(format!(
            "published materialization object is empty or not regular: {}",
            path.display()
        )));
    }
    let object = StoreObject {
        logical_path: workspace.final_output.join(relative_path).display().to_string(),
        digest_blake3: hash_file_bounded(path)?,
        size_bytes: metadata.len(),
    };
    debug_assert!(object.logical_path.starts_with('/'));
    debug_assert!(object.size_bytes > 0);
    Ok(object)
}

fn validate_stage_key(stage_key: &str) -> Result<(), Error> {
    let is_safe = !stage_key.is_empty()
        && stage_key.len() <= MAX_STAGE_KEY_BYTES
        && stage_key.bytes().all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-');
    if !is_safe {
        return Err(Error::Invalid(format!("stage key is unsafe for receipt publication: `{stage_key}`")));
    }
    debug_assert!(!stage_key.is_empty());
    debug_assert!(stage_key.len() <= MAX_STAGE_KEY_BYTES);
    Ok(())
}

#[cfg(test)]
mod tests {
    use crunch_wasm_component_core::Blake3Identity;
    use crunch_wasm_component_core::REQUIRED_RELEASE_ELIGIBILITY_NON_CLAIM;
    use crunch_wasm_component_core::REQUIRED_RUNTIME_AUTHORITY_NON_CLAIM;

    use super::*;

    #[test]
    fn consumer_rehashes_every_materialized_file_and_directory() {
        let root = tempfile::tempdir().unwrap();
        let bundle = fixture_bundle(root.path());

        verify_materialization_bundle_files(&bundle).unwrap();

        fs::write(&bundle.final_portable.logical_path, b"tampered-final-component").unwrap();
        let error = verify_materialization_bundle_files(&bundle).expect_err("tampered materialized bytes must fail");
        assert!(error.to_string().contains("materialization object bytes drifted"));
        assert!(error.to_string().contains(&bundle.final_portable.logical_path));
    }

    fn fixture_bundle(root: &Path) -> MaterializationBundle {
        let source = root.join("source");
        fs::create_dir(&source).unwrap();
        fs::write(source.join("lib.rs"), b"pub fn component() {}\n").unwrap();
        let measured = copy_source_tree(&source, &root.join("source-measurement")).unwrap();
        let source_object = StoreObject {
            logical_path: source.display().to_string(),
            digest_blake3: measured.digest_blake3,
            size_bytes: measured.size_bytes,
        };
        let wit = fixture_file(root, "world.wit", b"package test:component;\nworld demo {}\n");
        let lock = fixture_file(root, "wkg.lock", b"version = 1\n");
        let final_portable = fixture_file(root, "final.component.wasm", b"final-component");
        let stages = [
            ("package-resolution", ComponentStageKind::PackageResolution),
            ("checked-lock", ComponentStageKind::Lock),
            ("binding-generation", ComponentStageKind::BindingGeneration),
            ("compilation", ComponentStageKind::Compilation),
            ("composition", ComponentStageKind::Composition),
            ("virtualization", ComponentStageKind::Virtualization),
            ("build-validation", ComponentStageKind::BuildValidation),
            ("octet-validation", ComponentStageKind::OctetValidation),
        ];
        let receipts = stages
            .into_iter()
            .map(|(stage_key, kind)| {
                let receipt = fixture_file(
                    root,
                    &format!("{stage_key}.receipt.json"),
                    format!("{{\"stage\":\"{stage_key}\"}}\n").as_bytes(),
                );
                StageReceiptReference {
                    stage_key: stage_key.to_string(),
                    kind,
                    receipt_blake3: receipt.digest_blake3.clone(),
                    receipt,
                    artifact: Some(final_portable.clone()),
                }
            })
            .collect();
        build_materialization_bundle(MaterializationBundleRequest {
            name: "consumer-verification-fixture".to_string(),
            manifest_blake3: Blake3Identity::from_slice(b"manifest"),
            cohort_blake3: Blake3Identity::from_slice(b"cohort"),
            wit_inputs: vec![wit],
            package_inputs: Vec::new(),
            source_closure: source_object,
            lock,
            final_portable,
            expected_octet_profile_blake3: Blake3Identity::from_slice(b"octet-profile"),
            expected_runtime_profile_blake3: Blake3Identity::from_slice(b"runtime-profile"),
            stage_receipts: receipts,
            wizer: None,
            aot: None,
            non_claims: vec![
                REQUIRED_RUNTIME_AUTHORITY_NON_CLAIM.to_string(),
                REQUIRED_RELEASE_ELIGIBILITY_NON_CLAIM.to_string(),
            ],
        })
        .bundle
        .unwrap()
    }

    fn fixture_file(root: &Path, name: &str, bytes: &[u8]) -> StoreObject {
        let path = root.join(name);
        fs::write(&path, bytes).unwrap();
        let metadata = fs::metadata(&path).unwrap();
        StoreObject {
            logical_path: path.display().to_string(),
            digest_blake3: hash_file_bounded(&path).unwrap(),
            size_bytes: metadata.len(),
        }
    }
}
