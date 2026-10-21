//! Explicit preparation of inert source programs. This path never executes a
//! provider, imports build outputs, publishes a proof receipt, or updates aliases.
use mantle_application_contract::bootstrap_effects::BootstrapEffect::ReadClock;
use mantle_application_contract::bootstrap_effects::BootstrapEffect::ReadInput;
use mantle_application_contract::bootstrap_effects::BootstrapEffect::VerifyOutput;
use mantle_application_contract::bootstrap_effects::BootstrapEffect::WriteOutput;

use super::*;
use crate::bootstrap_effect_port::BootstrapPort;
use crate::source_built_program_authoring::AuthoredPrograms;
use crate::source_built_program_authoring::AuthoringRequest;
use crate::source_built_recipe::ArtifactRef;
use crate::source_built_recipe::{self as recipe};
use crate::source_bundle::SourceBundleManifest;
use crate::source_bundle::SourceFileEntry;
use crate::source_bundle::SourceFileType;
use crate::source_bundle::SourceRecordKind;

const PROFILE_FILE: &str = "source-profile.json";
const REPORT_FILE: &str = "source-program-preparation.json";
const SELECTOR: &str = "bootstrap/source-built-cold-route.json";
const PROGRAMS_DIR: &str = "program-sources";
const NON_CLAIM: &str = "inert source preparation only; no provider execution, fixed-point proof, compiler correctness, reproducibility, or release eligibility";

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct PreparationReport {
    schema: String,
    input_profile_blake3: String,
    input_plan_blake3: String,
    output_profile_blake3: String,
    source_profile: PathBuf,
    program_records: BTreeMap<String, String>,
    non_claim: String,
}

pub(crate) fn cmd_prepare_source_programs(options: SourceBuiltFixedPointOptions<'_>) -> Result<(), RunError> {
    if is_development_run(&options) {
        return Err(proof_error("source-program preparation rejects development caches and resume modes".into()));
    }
    let mut port = BootstrapPort::new(&[(ReadInput, 2), (ReadClock, 1), (WriteOutput, 3), (VerifyOutput, 2)])?;
    let result = (|| {
        port.call(ReadInput, || validate_options(&options))?;
        let started_at = port.call(ReadClock, || Ok::<_, RunError>(Instant::now()))?;
        let profile = port.call(ReadInput, || crate::source_bundle::read_source_bundle(options.source_profile))?;
        let prepared = port.call(WriteOutput, || prepare_profile_attempt(&options, started_at, &profile, true))?;
        let roots = port.call(VerifyOutput, || cold::cold_source_roots(&prepared.staging_dir, &prepared.plan))?;
        let report = port.call(WriteOutput, || prepare_source_profile(&options, &prepared, &roots, profile))?;
        port.call(WriteOutput, || publish_preparation(&prepared))?;
        port.call(VerifyOutput, || verify_preparation(&prepared.final_dir, &report))?;
        Ok(report)
    })();
    let report = port.finish(result)?;
    if options.json {
        println!("{}", serde_json::to_string_pretty(&report).map_err(|error| proof_error(error.to_string()))?);
    } else {
        println!("source_profile={}", report.source_profile.display());
        println!("source_profile_blake3={}", report.output_profile_blake3);
        println!("non_claim={}", report.non_claim);
    }
    assert_eq!(report.input_profile_blake3, options.expected_source_profile_blake3);
    assert_eq!(report.non_claim, NON_CLAIM);
    Ok(())
}

fn prepare_source_profile(
    options: &SourceBuiltFixedPointOptions<'_>,
    prepared: &PreparedAttempt,
    roots: &BTreeMap<SourceAuthorityRole, PathBuf>,
    mut profile: SourceBundleManifest,
) -> Result<PreparationReport, RunError> {
    let selected = source_built_fixed_point_profile_records(&profile)?;
    let mantle_identity = selected.mantle_source.identity.clone();
    let native_identity = selected.native_source_manifest.identity.clone();
    let (programs, mantle_files) = author_programs(prepared, roots, options.disk_bytes_max)?;
    let program_records =
        programs.iter().map(|record| (record.identity.clone(), record.content_blake3.clone())).collect();
    let native_path = &roots[&SourceAuthorityRole::NativeSourceBundle];
    let native = crate::source_bundle::read_source_bundle(native_path)?;
    let mut native_records = native.records;
    append_program_records(&mut native_records, &programs, options.source_records_max)?;
    let native = assemble_source_bundle(native_records, &profile.store_prefix)?;
    crate::source_bundle::write_source_bundle(native_path, &native)?;
    write_source_files(&prepared.source_root, &mantle_files, options.disk_bytes_max)?;
    write_selector(&prepared.source_root)?;
    for record in &mut profile.records {
        if record.identity == mantle_identity {
            crate::source_bundle::recapture_source_record_payload(record, &prepared.source_root)?;
        } else if record.identity == native_identity {
            crate::source_bundle::recapture_source_record_payload(record, native_path)?;
        }
    }
    append_program_records(&mut profile.records, &programs, options.source_records_max)?;
    let profile = assemble_source_bundle(profile.records, &profile.store_prefix)?;
    source_built_fixed_point_profile_records(&profile)?;
    crate::source_bundle::write_source_bundle(&prepared.staging_dir.join(PROFILE_FILE), &profile)?;
    let report = PreparationReport {
        schema: "mantle-source-built-source-programs-v1".into(),
        input_profile_blake3: options.expected_source_profile_blake3.into(),
        input_plan_blake3: prepared.plan.plan_digest_blake3.clone(),
        output_profile_blake3: profile.manifest_blake3,
        source_profile: prepared.final_dir.join(PROFILE_FILE),
        program_records,
        non_claim: NON_CLAIM.into(),
    };
    write_json_create_new(&prepared.staging_dir.join(REPORT_FILE), &report)?;
    assert!(!report.program_records.is_empty());
    assert_ne!(report.input_profile_blake3, report.output_profile_blake3);
    Ok(report)
}

fn author_programs(
    prepared: &PreparedAttempt,
    roots: &BTreeMap<SourceAuthorityRole, PathBuf>,
    bytes_max: u64,
) -> Result<(Vec<SourceRecord>, BTreeMap<String, String>), RunError> {
    match fs::symlink_metadata(prepared.source_root.join(SELECTOR)) {
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => return Err(proof_error(format!("checking original source selector: {error}"))),
        Ok(_) => {
            return Err(proof_error(
                "source profile already has a cold selector; prepare from original unexpanded source".into(),
            ));
        }
    }
    let mut exports = BTreeMap::new();
    let targets = [TARGET_TRIPLE.to_string()];
    let mut records = Vec::with_capacity(4);
    let stagex = crate::source_built_stagex_programs::author(&authoring_request(prepared, roots, &exports, &targets))?;
    retain_authored(
        prepared,
        crate::source_built_stagex_programs::PROGRAM_RECORD,
        stagex,
        &mut records,
        &mut exports,
        bytes_max,
    )?;
    let native = crate::source_built_native_programs::author(&authoring_request(prepared, roots, &exports, &targets))?;
    retain_authored(
        prepared,
        crate::source_built_native_programs::RECORD_IDENTITY,
        native,
        &mut records,
        &mut exports,
        bytes_max,
    )?;
    let reservations = crate::source_built_program_authoring::read_continuation_policy(&prepared.source_root)?;
    let rust = crate::source_built_rust_programs::author(
        &authoring_request(prepared, roots, &exports, &targets),
        &reservations,
    )?;
    retain_authored(
        prepared,
        crate::source_built_rust_programs::RECORD_IDENTITY,
        rust,
        &mut records,
        &mut exports,
        bytes_max,
    )?;
    let mantle = crate::source_built_mantle_programs::author(&authoring_request(prepared, roots, &exports, &targets))?;
    let helpers = crate::source_built_mantle_programs::source_files();
    records.push(package_programs(
        &prepared.staging_dir,
        &prepared.final_dir,
        crate::source_built_mantle_programs::PROGRAM_RECORD,
        helpers,
        bytes_max,
    )?);
    assert_eq!(records.len(), 4);
    assert!(!mantle.files.is_empty());
    Ok((records, mantle.files))
}

fn authoring_request<'a>(
    prepared: &'a PreparedAttempt,
    roots: &'a BTreeMap<SourceAuthorityRole, PathBuf>,
    exports: &'a BTreeMap<String, ArtifactRef>,
    targets: &'a [String],
) -> AuthoringRequest<'a> {
    AuthoringRequest {
        proof: &prepared.plan,
        source_roots: roots,
        predecessor_exports: exports,
        scratch_root: &prepared.staging_dir,
        targets,
    }
}

fn retain_authored(
    prepared: &PreparedAttempt,
    identity: &str,
    authored: AuthoredPrograms,
    records: &mut Vec<SourceRecord>,
    exports: &mut BTreeMap<String, ArtifactRef>,
    bytes_max: u64,
) -> Result<(), RunError> {
    for key in authored.exports.keys() {
        if exports.contains_key(key) {
            return Err(proof_error(format!("duplicate authored predecessor export {key}")));
        }
    }
    let record = package_programs(&prepared.staging_dir, &prepared.final_dir, identity, authored.files, bytes_max)?;
    exports.extend(authored.exports);
    records.push(record);
    assert!(!records.is_empty());
    assert!(!exports.is_empty());
    Ok(())
}

fn append_program_records(
    records: &mut Vec<SourceRecord>,
    programs: &[SourceRecord],
    records_max: u32,
) -> Result<(), RunError> {
    let count = records
        .len()
        .checked_add(programs.len())
        .ok_or_else(|| proof_error("source record count overflow".into()))?;
    if count > records_max as usize || count > crate::source_bundle::MAX_SOURCE_RECORDS {
        return Err(proof_error("authored source records exceed the admitted record bound".into()));
    }
    for program in programs {
        if records.iter().any(|record| record.identity == program.identity) {
            return Err(proof_error(format!("source program identity already exists: {}", program.identity)));
        }
    }
    records.extend_from_slice(programs);
    assert_eq!(records.len(), count);
    assert!(records.len() <= records_max as usize);
    Ok(())
}

fn package_programs(
    staging: &Path,
    final_root: &Path,
    identity: &str,
    files: BTreeMap<String, String>,
    bytes_max: u64,
) -> Result<SourceRecord, RunError> {
    recipe::relative(identity, false)?;
    if identity.contains('/') {
        return Err(proof_error("source program identity must be one path component".into()));
    }
    let directory = staging.join(PROGRAMS_DIR).join(identity);
    fs::create_dir_all(&directory)
        .map_err(|error| proof_error(format!("creating source program directory: {error}")))?;
    let payload_bytes = write_source_files(&directory, &files, bytes_max)?;
    write_program_archive(&staging.join(PROGRAMS_DIR).join(format!("{identity}.tar")), &files)?;
    let nar = crunch_nar::observe_path(
        &directory,
        crunch_nar::FilesystemNarRequest::new(
            nix_compat::nixhash::HashAlgo::Blake3,
            crunch_nar::CaseHackPolicy::native(),
            bytes_max,
        ),
    )
    .map_err(|error| proof_error(format!("hashing authored source NAR: {error}")))?;
    let url = url::Url::from_file_path(final_root.join(PROGRAMS_DIR).join(format!("{identity}.tar")))
        .map_err(|()| proof_error("source program archive URL must be absolute".into()))?;
    let metadata = BTreeMap::from([
        ("builder".into(), "builtin:fetchurl".into()),
        ("name".into(), identity.into()),
        ("url".into(), url.to_string()),
        ("hash_algo".into(), "blake3".into()),
        ("hash".into(), data_encoding::HEXLOWER.encode(nar.digest.digest_as_bytes())),
        ("hash_mode".into(), "recursive".into()),
        ("unpack".into(), "1".into()),
    ]);
    let files = files
        .into_iter()
        .map(|(path, text)| SourceFileEntry {
            path,
            file_type: SourceFileType::Regular,
            executable: false,
            size: text.len() as u64,
            content_hex: Some(data_encoding::HEXLOWER.encode(text.as_bytes())),
            symlink_target: None,
            chunk_index: None,
            chunk_count: None,
            blake3: blake3::hash(text.as_bytes()).to_hex().to_string(),
        })
        .collect::<Vec<_>>();
    let content_blake3 =
        crate::source_bundle::digest_source_record_content(&SourceRecordKind::FixedUrl, &metadata, &files)?;
    assert!(!files.is_empty());
    assert!(payload_bytes <= bytes_max);
    Ok(SourceRecord {
        kind: SourceRecordKind::FixedUrl,
        identity: identity.into(),
        store_prefix: Some(LOGICAL_STORE_PREFIX.into()),
        adapter: None,
        metadata,
        payload_bytes,
        content_blake3,
        files,
        store_path_attestation: None,
    })
}

fn validate_source_files(files: &BTreeMap<String, String>, bytes_max: u64) -> Result<u64, RunError> {
    if files.is_empty() || files.len() > crate::source_bundle::MAX_SOURCE_FILES_PER_RECORD {
        return Err(proof_error("authored source program file count is outside its bound".into()));
    }
    let mut total = 0u64;
    for (path, text) in files {
        recipe::relative(path, false)?;
        recipe::validate_text(text)?;
        let bytes = text.len() as u64;
        if bytes > crate::source_bundle::MAX_SOURCE_FILE_BYTES {
            return Err(proof_error(format!("authored source member is too large: {path}")));
        }
        total = total.checked_add(bytes).ok_or_else(|| proof_error("authored source byte count overflow".into()))?;
        if total > bytes_max {
            return Err(proof_error("authored source program exceeds the admitted byte bound".into()));
        }
    }
    assert!(files.len() <= crate::source_bundle::MAX_SOURCE_FILES_PER_RECORD);
    assert!(total <= bytes_max);
    Ok(total)
}

fn write_source_files(root: &Path, files: &BTreeMap<String, String>, bytes_max: u64) -> Result<u64, RunError> {
    let bytes = validate_source_files(files, bytes_max)?;
    for (relative, text) in files {
        let path = source_destination(root, relative)?;
        write_bytes_create_new(&path, text.as_bytes())?;
    }
    assert!(!files.is_empty());
    assert!(bytes <= bytes_max);
    Ok(bytes)
}

fn source_destination(root: &Path, relative: &str) -> Result<PathBuf, RunError> {
    recipe::relative(relative, false)?;
    let mut path = root.to_path_buf();
    let parent = Path::new(relative).parent().expect("relative source path has a parent");
    for component in std::iter::once(Path::new("")).chain(parent.iter().map(Path::new)) {
        path.push(component);
        match fs::create_dir(&path) {
            Ok(()) => {}
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {}
            Err(error) => return Err(proof_error(format!("creating source parent {}: {error}", path.display()))),
        }
        if !fs::symlink_metadata(&path).map_err(|error| proof_error(error.to_string()))?.is_dir() {
            return Err(proof_error(format!("source parent is not a real directory: {}", path.display())));
        }
    }
    let name = Path::new(relative).file_name().expect("validated source path has a leaf");
    path.push(name);
    assert!(path.starts_with(root));
    assert_ne!(path, root);
    Ok(path)
}

fn write_program_archive(path: &Path, files: &BTreeMap<String, String>) -> Result<(), RunError> {
    let file = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .map_err(|error| proof_error(format!("creating source archive: {error}")))?;
    let mut archive = tar::Builder::new(file);
    for (path, text) in files {
        let mut header = tar::Header::new_gnu();
        header.set_entry_type(tar::EntryType::Regular);
        header.set_size(text.len() as u64);
        header.set_mode(0o644);
        header.set_uid(0);
        header.set_gid(0);
        header.set_mtime(0);
        header.set_cksum();
        archive
            .append_data(&mut header, format!("source/{path}"), text.as_bytes())
            .map_err(|error| proof_error(format!("writing source archive member: {error}")))?;
    }
    let file = archive.into_inner().map_err(|error| proof_error(format!("finishing source archive: {error}")))?;
    file.sync_all().map_err(|error| proof_error(format!("syncing source archive: {error}")))?;
    assert!(!files.is_empty());
    assert!(file.metadata().map_err(|error| proof_error(error.to_string()))?.len() > 0);
    Ok(())
}

fn write_selector(root: &Path) -> Result<(), RunError> {
    let manifest = |identity: &str, member: &str| ArtifactRef::ManifestSource {
        role: SourceAuthorityRole::NativeSourceBundle,
        record_identity: identity.into(),
        member: member.into(),
    };
    let route = serde_json::json!({
        "schema": "mantle-source-built-cold-route-v1",
        "stagex_transition": manifest(crate::source_built_stagex_programs::PROGRAM_RECORD, "stagex-transition.json"),
        "stagex_publication": manifest(crate::source_built_stagex_programs::PROGRAM_RECORD, "stagex-provider-publication.json"),
        "native": {
            "record_identity": crate::source_built_native_programs::RECORD_IDENTITY,
            "recipe_member": crate::source_built_native_programs::RECIPE_MEMBER,
            "index_member": crate::source_built_native_programs::INDEX_MEMBER,
        },
        "rust_provider": manifest(crate::source_built_rust_programs::RECORD_IDENTITY, crate::source_built_rust_programs::RECIPE_MEMBER),
        "mantle_recipes": ["bootstrap/source-built/mantle-stage1.json", "bootstrap/source-built/mantle-stage2.json"],
    });
    let path = source_destination(root, SELECTOR)?;
    write_json_create_new(&path, &route)
}

fn publish_preparation(prepared: &PreparedAttempt) -> Result<(), RunError> {
    crate::linux_rename::rename_path_no_replace(&prepared.staging_dir, &prepared.final_dir)
        .map_err(|error| proof_error(format!("publishing source preparation: {error}")))?;
    let parent = prepared.final_dir.parent().expect("validated preparation root has a parent");
    fs::File::open(parent)
        .and_then(|directory| directory.sync_all())
        .map_err(|error| proof_error(format!("syncing source preparation publication: {error}")))?;
    assert!(prepared.final_dir.is_dir());
    assert!(!prepared.staging_dir.exists());
    Ok(())
}

fn verify_preparation(root: &Path, expected: &PreparationReport) -> Result<(), RunError> {
    let bytes = fs::read(root.join(REPORT_FILE)).map_err(|error| proof_error(error.to_string()))?;
    let observed: PreparationReport = serde_json::from_slice(&bytes).map_err(|error| proof_error(error.to_string()))?;
    if &observed != expected || observed.non_claim != NON_CLAIM {
        return Err(proof_error("persisted source preparation report differs".into()));
    }
    let profile = crate::source_bundle::read_source_bundle(&expected.source_profile)?;
    if profile.manifest_blake3 != expected.output_profile_blake3 {
        return Err(proof_error("persisted authored source profile identity differs".into()));
    }
    let selected = source_built_fixed_point_profile_records(&profile)?;
    let native = crate::source_bundle::read_source_bundle(
        &root
            .join(INPUTS_DIR)
            .join(NATIVE_SOURCE_AUTHORITY_DIR)
            .join(&selected.native_source_manifest.files[0].path),
    )?;
    let stagex = crate::source_bundle::read_source_bundle(
        &root
            .join(INPUTS_DIR)
            .join(STAGEX_SOURCE_BUNDLE_DIR)
            .join(&selected.stagex_source_bundle.files[0].path),
    )?;
    validate_materialized_source_records(&native, &stagex, &selected.native_source_records)?;
    for (identity, digest) in &expected.program_records {
        if !selected
            .native_source_records
            .iter()
            .any(|record| &record.identity == identity && &record.content_blake3 == digest)
        {
            return Err(proof_error(format!("persisted source program identity differs: {identity}")));
        }
    }
    assert_eq!(observed.output_profile_blake3, profile.manifest_blake3);
    assert_eq!(observed.non_claim, NON_CLAIM);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn packaged_source_round_trip_rejects_payload_tampering() {
        let temp = tempfile::tempdir().unwrap();
        let files = BTreeMap::from([("helpers/main.c".into(), "int main(void) { return 0; }\n".into())]);
        let record = package_programs(temp.path(), temp.path(), "source-program", files, 1024 * 1024).unwrap();
        let output = temp.path().join("decoded");
        crate::source_bundle::materialize_source_record_for_offline_use(&record, &output).unwrap();
        assert_eq!(fs::read_to_string(output.join("helpers/main.c")).unwrap(), "int main(void) { return 0; }\n");
        let mut altered = record;
        altered.files[0].content_hex = Some(data_encoding::HEXLOWER.encode(b"int main(void) { return 1; }\n"));
        altered.files[0].blake3 = blake3::hash(b"int main(void) { return 1; }\n").to_hex().to_string();
        assert!(
            crate::source_bundle::materialize_source_record_for_offline_use(&altered, &temp.path().join("altered"))
                .is_err()
        );
    }

    #[test]
    fn source_writes_reject_escape_binary_payloads_and_symlinked_parents() {
        let temp = tempfile::tempdir().unwrap();
        for (path, text) in [
            ("../escape.c", "int main(void){}"),
            ("helper.c", "\u{7f}ELFbinary"),
            ("helper.c", "a\0b"),
        ] {
            let files = BTreeMap::from([(path.into(), text.into())]);
            assert!(write_source_files(temp.path(), &files, 1024).is_err());
        }
        let outside = tempfile::tempdir().unwrap();
        std::os::unix::fs::symlink(outside.path(), temp.path().join("linked")).unwrap();
        assert!(
            write_source_files(temp.path(), &BTreeMap::from([("linked/helper.c".into(), "int x;".into())]), 1024)
                .is_err()
        );
        assert!(!outside.path().join("helper.c").exists());
        write_source_files(temp.path(), &BTreeMap::from([("safe/helper.c".into(), "int x;".into())]), 1024).unwrap();
        assert_eq!(fs::read_to_string(temp.path().join("safe/helper.c")).unwrap(), "int x;");
    }
}
