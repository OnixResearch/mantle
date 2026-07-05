use std::path::Path;

use serde::Serialize;

use crate::build_cmd::load_configured_trusted_public_keys;
use crate::build_cmd::load_or_generate_signing_keypair;
use crate::errors::RunError;

pub fn cmd_store(
    action: crate::StoreAction,
    output_dir: &Path,
    state_dir: &Path,
    store_dir: &str,
    json_output: bool,
) -> Result<(), RunError> {
    let rt = tokio::runtime::Runtime::new().map_err(|e| RunError::Internal(format!("tokio runtime: {e}")))?;
    rt.block_on(async { cmd_store_async(action, output_dir, state_dir, store_dir, json_output).await })
}

async fn cmd_store_async(
    action: crate::StoreAction,
    output_dir: &Path,
    state_dir: &Path,
    store_dir: &str,
    json_output: bool,
) -> Result<(), RunError> {
    match action {
        crate::StoreAction::List => {
            let svc = open_pathinfo_service(state_dir, true).await?;
            cmd_store_list(&svc).await
        }
        crate::StoreAction::Info { path } => {
            let svc = open_pathinfo_service(state_dir, true).await?;
            cmd_store_info(&svc, &path).await
        }
        crate::StoreAction::Roots => {
            let store = open_store(output_dir, state_dir, store_dir).await?;
            cmd_store_roots(&store)
        }
        crate::StoreAction::Pin { path } => {
            let _guard = crunch_store::StoreMutationGuard::acquire_wait(state_dir)
                .map_err(|e| RunError::Internal(format!("acquiring store mutation lock: {e}")))?;
            let store = open_store(output_dir, state_dir, store_dir).await?;
            cmd_store_pin(&store, &path).await
        }
        crate::StoreAction::Unpin { path } => {
            let _guard = crunch_store::StoreMutationGuard::acquire_wait(state_dir)
                .map_err(|e| RunError::Internal(format!("acquiring store mutation lock: {e}")))?;
            let store = open_store(output_dir, state_dir, store_dir).await?;
            cmd_store_unpin(&store, &path)
        }
        crate::StoreAction::Gc { is_dry_run } => {
            let _guard = crunch_store::StoreMutationGuard::try_acquire(state_dir)
                .map_err(|e| RunError::Build(format!("{e}")))?;
            let mut store = open_store(output_dir, state_dir, store_dir).await?;
            cmd_store_gc(&mut store, is_dry_run).await
        }
        crate::StoreAction::Verify {
            path,
            signing_key,
            trusted_public_keys,
            trust_unsigned,
        } => {
            let svc = open_pathinfo_service(state_dir, true).await?;
            cmd_store_verify(
                &svc,
                path.as_deref(),
                signing_key.as_deref(),
                &trusted_public_keys,
                trust_unsigned,
                state_dir,
            )
            .await
        }
        crate::StoreAction::Sign { path, all, signing_key } => {
            let _guard = crunch_store::StoreMutationGuard::acquire_wait(state_dir)
                .map_err(|e| RunError::Internal(format!("acquiring store mutation lock: {e}")))?;
            let svc = open_pathinfo_service(state_dir, false).await?;
            cmd_store_sign(&svc, path.as_deref(), all, signing_key.as_deref(), state_dir).await
        }
        crate::StoreAction::Push {
            to,
            all,
            trust_unsigned,
            paths,
        } => {
            let _guard = crunch_store::StoreMutationGuard::acquire_wait(state_dir)
                .map_err(|e| RunError::Internal(format!("acquiring store mutation lock: {e}")))?;
            let store = open_store(output_dir, state_dir, store_dir).await?;
            cmd_store_push(&store, &to, all, trust_unsigned, &paths).await
        }
        crate::StoreAction::Pull {
            from,
            all,
            trust_unsigned,
            trusted_public_keys,
            paths,
        } => {
            let _guard = crunch_store::StoreMutationGuard::acquire_wait(state_dir)
                .map_err(|e| RunError::Internal(format!("acquiring store mutation lock: {e}")))?;
            let store = open_store(output_dir, state_dir, store_dir).await?;
            cmd_store_pull(&store, &from, all, trust_unsigned, &trusted_public_keys, &paths, state_dir).await
        }
        crate::StoreAction::Archive { action } => {
            cmd_store_archive(action, output_dir, state_dir, store_dir, json_output).await
        }
    }
}

async fn open_pathinfo_service(
    state_dir: &Path,
    read_only: bool,
) -> Result<snix_store::pathinfoservice::RedbPathInfoService, RunError> {
    use snix_store::pathinfoservice::RedbPathInfoService;
    use snix_store::pathinfoservice::RedbPathInfoServiceConfig;

    let db_path = state_dir.join("pathinfo.redb");
    if !db_path.exists() {
        return Err(RunError::Internal(format!(
            "PathInfo database {} does not exist (no builds yet?)",
            db_path.display()
        )));
    }
    RedbPathInfoService::new("crunch".to_string(), RedbPathInfoServiceConfig {
        path: Some(db_path.clone()),
        read_only,
        cache_size: None,
    })
    .await
    .map_err(|e| RunError::Internal(format!("opening PathInfo database {}: {e}", db_path.display())))
}

async fn open_store(
    output_dir: &Path,
    state_dir: &Path,
    store_dir: &str,
) -> Result<crunch_store::StoreHandle, RunError> {
    crunch_store::StoreHandle::open(crunch_store::StoreConfig {
        state_dir: state_dir.to_path_buf(),
        output_dir: output_dir.to_path_buf(),
        remote_cache_urls: Vec::new(),
        fallback_mode: crunch_store::StoreFallbackMode::Practical,
        store_dir: store_dir.to_string(),
    })
    .await
    .map_err(|e| RunError::Internal(format!("opening store: {e}")))
}

async fn cmd_store_list(svc: &impl snix_store::pathinfoservice::PathInfoService) -> Result<(), RunError> {
    let entries = crunch_store::store_list(svc).await.map_err(|e| RunError::Internal(format!("{e}")))?;
    if entries.is_empty() {
        eprintln!("No paths in PathInfo database.");
    } else {
        for (store_path, deriver, nar_size) in &entries {
            println!("{store_path}  deriver={deriver}  nar_size={nar_size}");
        }
        eprintln!("{} path(s)", entries.len());
    }
    Ok(())
}

async fn cmd_store_info(svc: &impl snix_store::pathinfoservice::PathInfoService, path: &str) -> Result<(), RunError> {
    let details = crunch_store::store_info(svc, path).await.map_err(|e| RunError::Internal(format!("{e}")))?;
    if details.is_empty() {
        return Err(RunError::Internal(format!("no PathInfo matching '{path}'")));
    }
    for detail in &details {
        println!("store_path: {}", detail.store_path);
        println!("nar_size:   {}", detail.nar_size);
        println!("nar_sha256: {}", data_encoding::HEXLOWER.encode(&detail.nar_sha256));
        if let Some(ref deriver) = detail.deriver {
            println!("deriver:    {deriver}");
        }
        if !detail.references.is_empty() {
            println!("references:");
            for reference in &detail.references {
                println!("  {reference}");
            }
        }
        if let Some(ref ca) = detail.ca {
            println!("ca:         {ca}");
        }
        if detail.signatures.is_empty() {
            println!("signatures: (none)");
        } else {
            println!("signatures:");
            for sig in &detail.signatures {
                println!("  {sig}");
            }
        }
        println!("node:       {}", detail.node);
    }
    Ok(())
}

fn cmd_store_roots(store: &crunch_store::StoreHandle) -> Result<(), RunError> {
    let roots = store.list_retained_roots().map_err(|e| RunError::Internal(format!("{e}")))?;
    if roots.is_empty() {
        eprintln!("No retained GC roots.");
        return Ok(());
    }
    for root in &roots {
        println!("{}  source={}  created_unix_s={}", root.logical_path, root.source, root.created_unix_s);
    }
    eprintln!("{} retained root(s)", roots.len());
    Ok(())
}

async fn cmd_store_pin(store: &crunch_store::StoreHandle, path: &str) -> Result<(), RunError> {
    let root = store.pin_retained_root(path).await.map_err(|e| RunError::Build(format!("{e}")))?;
    println!("PINNED {}  source={}  created_unix_s={}", root.logical_path, root.source, root.created_unix_s);
    Ok(())
}

fn cmd_store_unpin(store: &crunch_store::StoreHandle, path: &str) -> Result<(), RunError> {
    let removed = store.unpin_retained_root(path).map_err(|e| RunError::Build(format!("{e}")))?;
    let Some(root) = removed else {
        return Err(RunError::Build(format!("retained root not found: {path}")));
    };
    println!("UNPINNED {}", root.logical_path);
    Ok(())
}

async fn cmd_store_gc(store: &mut crunch_store::StoreHandle, is_dry_run: bool) -> Result<(), RunError> {
    let report = store.garbage_collect(is_dry_run).await.map_err(|e| RunError::Build(format!("{e}")))?;
    if report.candidate_paths.is_empty() {
        println!("retained_roots={}  candidate_paths=0  reclaimable_bytes=0", report.retained_root_count);
        if is_dry_run {
            eprintln!("dry-run: no changes made");
        } else {
            eprintln!("gc: nothing to do");
        }
        return Ok(());
    }

    println!(
        "retained_roots={}  candidate_paths={}  reclaimable_bytes={}",
        report.retained_root_count, report.candidate_path_count, report.reclaimable_bytes_total
    );
    println!(
        "candidate_exported_outputs={}  candidate_artifact_attestations={}  candidate_closure_attestations={}  candidate_blob_indexes={}  candidate_blob_chunks={}",
        report.candidate_exported_output_count,
        report.candidate_artifact_attestation_count,
        report.candidate_closure_attestation_count,
        report.candidate_blob_index_count,
        report.candidate_blob_chunk_count,
    );
    for path in &report.candidate_paths {
        println!("DELETE {path}");
    }
    if is_dry_run {
        eprintln!("dry-run: no changes made");
    } else {
        eprintln!("gc: removed {} candidate path(s)", report.candidate_path_count);
    }
    Ok(())
}

async fn cmd_store_verify(
    svc: &impl snix_store::pathinfoservice::PathInfoService,
    path_filter: Option<&str>,
    signing_key_path: Option<&std::path::Path>,
    explicit_trusted_public_keys: &[String],
    trust_unsigned: bool,
    state_dir: &Path,
) -> Result<(), RunError> {
    let trusted_keys = resolve_store_verify_keys(signing_key_path, explicit_trusted_public_keys, state_dir)?;
    let hash_results =
        crunch_store::store_verify(svc, path_filter).await.map_err(|e| RunError::Internal(format!("{e}")))?;
    let signature_results = crunch_store::store_verify_signatures(svc, path_filter, &trusted_keys)
        .await
        .map_err(|e| RunError::Internal(format!("{e}")))?;
    let signature_by_path = index_signature_results(signature_results)?;
    let summary = print_store_verify_results(&hash_results, &signature_by_path, trust_unsigned)?;

    eprintln!("{} checked, {} mismatches", summary.checked, summary.mismatches);
    if summary.mismatches > 0 {
        return Err(RunError::Build(format!("{} path(s) failed verification", summary.mismatches)));
    }
    Ok(())
}

struct VerifySummary {
    checked: u32,
    mismatches: u32,
}

fn resolve_store_verify_keys(
    signing_key_path: Option<&std::path::Path>,
    explicit_trusted_public_keys: &[String],
    state_dir: &Path,
) -> Result<Vec<nix_compat::narinfo::VerifyingKey>, RunError> {
    let parsed_explicit_keys = parse_trusted_public_keys(explicit_trusted_public_keys)?;
    let keypair = load_or_generate_signing_keypair(signing_key_path, state_dir, true)?;
    let configured_trusted_keys = load_configured_trusted_public_keys(parsed_explicit_keys.as_deref(), state_dir)?;
    Ok(crunch_build::build_trusted_keys(&keypair, configured_trusted_keys.as_deref()))
}

fn parse_trusted_public_keys(
    explicit_trusted_public_keys: &[String],
) -> Result<Option<Vec<nix_compat::narinfo::VerifyingKey>>, RunError> {
    if explicit_trusted_public_keys.is_empty() {
        return Ok(None);
    }
    let mut keys = Vec::new();
    for key_str in explicit_trusted_public_keys {
        let key = nix_compat::narinfo::VerifyingKey::parse(key_str)
            .map_err(|e| RunError::Internal(format!("invalid trusted public key '{key_str}': {e}")))?;
        keys.push(key);
    }
    Ok(Some(keys))
}

fn index_signature_results(
    signature_results: Vec<crunch_store::SignatureVerifyResult>,
) -> Result<std::collections::HashMap<String, crunch_store::SignatureVerifyResult>, RunError> {
    let mut signature_by_path = std::collections::HashMap::new();
    for result in signature_results {
        let replaced = signature_by_path.insert(result.path.clone(), result);
        if replaced.is_some() {
            return Err(RunError::Internal("duplicate signature verification result for store path".to_string()));
        }
    }
    Ok(signature_by_path)
}

fn print_store_verify_results(
    hash_results: &[crunch_store::VerifyResult],
    signature_by_path: &std::collections::HashMap<String, crunch_store::SignatureVerifyResult>,
    trust_unsigned: bool,
) -> Result<VerifySummary, RunError> {
    let mut summary = VerifySummary {
        checked: 0,
        mismatches: 0,
    };
    for result in hash_results {
        let mismatched = print_store_verify_result(result, signature_by_path, trust_unsigned)?;
        if mismatched {
            summary.mismatches = summary.mismatches.saturating_add(1);
        }
        summary.checked = summary.checked.saturating_add(1);
    }
    Ok(summary)
}

fn print_store_verify_result(
    result: &crunch_store::VerifyResult,
    signature_by_path: &std::collections::HashMap<String, crunch_store::SignatureVerifyResult>,
    trust_unsigned: bool,
) -> Result<bool, RunError> {
    match result {
        crunch_store::VerifyResult::Ok(path) => print_verified_ok(path, signature_by_path, trust_unsigned),
        crunch_store::VerifyResult::Missing(path) => {
            println!("MISSING {path}");
            Ok(true)
        }
        crunch_store::VerifyResult::Mismatch {
            path,
            stored_hash,
            actual_hash,
        } => {
            println!("MISMATCH {path}  stored={stored_hash}  actual={actual_hash}");
            Ok(true)
        }
    }
}

fn print_verified_ok(
    path: &str,
    signature_by_path: &std::collections::HashMap<String, crunch_store::SignatureVerifyResult>,
    trust_unsigned: bool,
) -> Result<bool, RunError> {
    let sig_result = signature_by_path
        .get(path)
        .ok_or_else(|| RunError::Internal(format!("missing signature result for {path}")))?;
    if trust_unsigned {
        println!("OK {path}  signatures=skipped");
        return Ok(false);
    }
    if sig_result.is_trusted() {
        println!("OK {path}  trusted_signatures={}/{}", sig_result.trusted_count, sig_result.total_signatures);
        return Ok(false);
    }
    if sig_result.total_signatures == 0 {
        println!("UNSIGNED {path}");
        return Ok(true);
    }
    let untrusted = sig_result.untrusted_names.join(",");
    println!("UNTRUSTED {path}  trusted_signatures=0/{}  signers={}", sig_result.total_signatures, untrusted);
    Ok(true)
}

async fn cmd_store_sign(
    svc: &impl snix_store::pathinfoservice::PathInfoService,
    path_filter: Option<&str>,
    is_sign_all: bool,
    signing_key_path: Option<&std::path::Path>,
    state_dir: &Path,
) -> Result<(), RunError> {
    if path_filter.is_none() && !is_sign_all {
        return Err(RunError::Internal("provide a store path or use --all to sign all entries".to_string()));
    }

    let keypair = crate::build_cmd::load_or_generate_signing_keypair(signing_key_path, state_dir, true)?;

    let results = crunch_store::store_sign(svc, &keypair.signing_key, path_filter, is_sign_all)
        .await
        .map_err(|e| RunError::Internal(format!("{e}")))?;

    let mut signed: u32 = 0;
    let mut appended: u32 = 0;
    let mut replaced: u32 = 0;
    for result in &results {
        if result.newly_signed {
            println!("SIGNED  {}", result.store_path);
            signed = signed.saturating_add(1);
            continue;
        }

        if result.appended {
            println!("APPEND  {}", result.store_path);
            appended = appended.saturating_add(1);
            continue;
        }

        if result.replaced {
            println!("REPLACE {}", result.store_path);
            replaced = replaced.saturating_add(1);
        }
    }

    eprintln!("{} signed, {} appended, {} replaced, {} total", signed, appended, replaced, results.len());
    Ok(())
}

async fn cmd_store_push(
    store: &crunch_store::StoreHandle,
    dest: &Path,
    is_push_all: bool,
    trust_unsigned: bool,
    path_selectors: &[String],
) -> Result<(), RunError> {
    if path_selectors.is_empty() && !is_push_all {
        return Err(RunError::Internal("provide store paths or use --all".to_string()));
    }

    // Collect matching PathInfo entries.
    let selected = if is_push_all {
        collect_all_pathinfos(store).await?
    } else {
        collect_matching_pathinfos(store, path_selectors).await?
    };

    if selected.is_empty() {
        eprintln!("no matching paths found");
        return Ok(());
    }

    let options = crunch_store::PushOptions { trust_unsigned };
    let report = crunch_store::export_paths_to_cache_dir(store, &selected, dest, &options)
        .await
        .map_err(|e| RunError::Internal(format!("push: {e}")))?;

    for pushed in &report.paths {
        println!("PUSH {}", pushed.store_path);
    }

    if report.skipped_unsigned_count > 0 {
        eprintln!(
            "warning: {} unsigned path(s) skipped (use --trust-unsigned to include)",
            report.skipped_unsigned_count
        );
    }

    eprintln!(
        "pushed={} skipped_unsigned={} skipped_present={} nar_bytes={} narinfo_bytes={}",
        report.pushed_count,
        report.skipped_unsigned_count,
        report.skipped_already_present_count,
        report.total_nar_bytes,
        report.total_narinfo_bytes,
    );
    Ok(())
}

fn parse_pull_source(source: &str) -> Result<crunch_store::PullSource, RunError> {
    let is_http_source = source.starts_with("http://") || source.starts_with("https://");
    if is_http_source {
        let url = url::Url::parse(source)
            .map_err(|e| RunError::Internal(format!("invalid HTTP pull source '{source}': {e}")))?;
        if !url.username().is_empty() || url.password().is_some() {
            return Err(RunError::Internal(format!("HTTP pull source must not include URL credentials: {source}")));
        }
        return Ok(crunch_store::PullSource::Http(url));
    }
    if source.contains("://") {
        return Err(RunError::Internal(format!(
            "unsupported pull source URL scheme in '{source}'; only http:// and https:// are supported"
        )));
    }
    Ok(crunch_store::PullSource::Directory(std::path::PathBuf::from(source)))
}

fn parse_http_pull_paths(
    path_selectors: &[String],
    store_dir: &str,
) -> Result<Vec<nix_compat::store_path::StorePath<String>>, RunError> {
    let mut parsed_paths = Vec::with_capacity(path_selectors.len());
    for selector in path_selectors {
        let store_path =
            nix_compat::store_path::StorePath::from_absolute_path_with_prefix(selector.as_bytes(), store_dir).map_err(
                |_| {
                    RunError::Internal(format!(
                        "HTTP pull requires explicit logical store paths under {store_dir}; got '{selector}'"
                    ))
                },
            )?;
        parsed_paths.push(store_path);
    }
    Ok(parsed_paths)
}

async fn cmd_store_pull(
    store: &crunch_store::StoreHandle,
    source: &str,
    is_pull_all: bool,
    trust_unsigned: bool,
    explicit_trusted_public_keys: &[String],
    path_selectors: &[String],
    state_dir: &Path,
) -> Result<(), RunError> {
    let pull_source = parse_pull_source(source)?;
    if matches!(&pull_source, crunch_store::PullSource::Http(_)) && is_pull_all {
        return Err(RunError::Internal("--all is not supported for HTTP caches; specify paths explicitly".to_string()));
    }
    if path_selectors.is_empty() && !is_pull_all {
        let detail = if matches!(&pull_source, crunch_store::PullSource::Http(_)) {
            "HTTP pull requires explicit store path selectors"
        } else {
            "provide store paths or use --all"
        };
        return Err(RunError::Internal(detail.to_string()));
    }

    if let crunch_store::PullSource::Directory(source_dir) = &pull_source {
        if !source_dir.exists() {
            return Err(RunError::Internal(format!("pull source directory does not exist: {}", source_dir.display())));
        }
    }

    // Resolve trusted keys: explicit CLI keys + store-configured keys.
    let parsed_explicit = parse_trusted_public_keys(explicit_trusted_public_keys)?;
    let keypair = load_or_generate_signing_keypair(None, state_dir, true)?;
    let configured_keys = load_configured_trusted_public_keys(parsed_explicit.as_deref(), state_dir)?;
    let trusted_keys = crunch_build::build_trusted_keys(&keypair, configured_keys.as_deref());
    let options = crunch_store::PullOptions {
        trust_unsigned,
        trusted_public_keys: trusted_keys,
    };

    let report = match pull_source {
        crunch_store::PullSource::Directory(source_dir) => {
            let paths_filter = if is_pull_all {
                None
            } else {
                Some(path_selectors.to_vec())
            };
            crunch_store::import_paths_from_cache_dir(store, &source_dir, paths_filter.as_deref(), &options)
                .await
                .map_err(|e| RunError::Internal(format!("pull: {e}")))?
        }
        crunch_store::PullSource::Http(cache_url) => {
            let requested_paths = parse_http_pull_paths(path_selectors, store.store_dir())?;
            crunch_store::import_paths_from_http_cache(store, &cache_url, &requested_paths, &options)
                .await
                .map_err(|e| RunError::Internal(format!("pull: {e}")))?
        }
    };

    for pulled in &report.paths {
        println!("PULL {}", pulled.store_path);
    }

    if report.skipped_untrusted_count > 0 {
        eprintln!(
            "warning: {} path(s) skipped (untrusted signature, use --trust-unsigned to include)",
            report.skipped_untrusted_count
        );
    }
    if report.skipped_store_dir_mismatch_count > 0 {
        eprintln!(
            "warning: {} path(s) skipped (store directory prefix mismatch)",
            report.skipped_store_dir_mismatch_count
        );
    }

    eprintln!(
        "imported={} skipped_present={} skipped_untrusted={} skipped_hash_mismatch={} skipped_missing_nar={} skipped_parse_error={} nar_bytes={}",
        report.imported_count,
        report.skipped_already_present_count,
        report.skipped_untrusted_count,
        report.skipped_hash_mismatch_count,
        report.skipped_missing_nar_count,
        report.skipped_parse_error_count,
        report.total_nar_bytes,
    );
    Ok(())
}

async fn cmd_store_archive(
    action: crate::StoreArchiveAction,
    output_dir: &Path,
    state_dir: &Path,
    store_dir: &str,
    json_output: bool,
) -> Result<(), RunError> {
    match action {
        crate::StoreArchiveAction::Export {
            to,
            all,
            trust_unsigned,
            paths,
        } => {
            let store = open_store(output_dir, state_dir, store_dir).await?;
            cmd_store_archive_export(&store, &to, all, trust_unsigned, &paths, json_output).await
        }
        crate::StoreArchiveAction::Import {
            from,
            trust_unsigned,
            trusted_public_keys,
            no_materialize,
        } => {
            let _guard = crunch_store::StoreMutationGuard::acquire_wait(state_dir)
                .map_err(|e| RunError::Internal(format!("acquiring store mutation lock: {e}")))?;
            let store = open_store(output_dir, state_dir, store_dir).await?;
            cmd_store_archive_import(
                &store,
                &from,
                trust_unsigned,
                &trusted_public_keys,
                !no_materialize,
                state_dir,
                json_output,
            )
            .await
        }
        crate::StoreArchiveAction::List { from } => cmd_store_archive_list(&from, json_output).await,
    }
}

async fn cmd_store_archive_export(
    store: &crunch_store::StoreHandle,
    dest: &Path,
    is_export_all: bool,
    trust_unsigned: bool,
    path_selectors: &[String],
    json_output: bool,
) -> Result<(), RunError> {
    if path_selectors.is_empty() && !is_export_all {
        return Err(RunError::Internal("provide store paths or use --all".to_string()));
    }
    let selected = if is_export_all {
        collect_all_pathinfos(store).await?
    } else {
        collect_matching_pathinfos(store, path_selectors).await?
    };
    if selected.is_empty() {
        return Err(RunError::Internal("no matching paths found".to_string()));
    }
    let options = crunch_store::ArchiveExportOptions { trust_unsigned };
    let writes_archive_to_stdout = is_stdio_path(dest);
    if writes_archive_to_stdout && json_output {
        return Err(RunError::Internal("--json cannot be combined with archive export --to -".to_string()));
    }
    let report = if writes_archive_to_stdout {
        let mut stdout = tokio::io::stdout();
        crunch_store::export_store_archive(store, &selected, &mut stdout, &options)
            .await
            .map_err(|e| RunError::Internal(format!("archive export: {e}")))?
    } else {
        let file = tokio::fs::File::create(dest)
            .await
            .map_err(|e| RunError::Internal(format!("creating archive {}: {e}", dest.display())))?;
        let mut writer = tokio::io::BufWriter::new(file);
        crunch_store::export_store_archive(store, &selected, &mut writer, &options)
            .await
            .map_err(|e| RunError::Internal(format!("archive export: {e}")))?
    };
    if writes_archive_to_stdout {
        eprintln!("exported={} payload_bytes={}", report.exported_count, report.total_payload_bytes);
        return Ok(());
    }
    print_archive_export_report(&report, json_output)
}

async fn cmd_store_archive_import(
    store: &crunch_store::StoreHandle,
    source: &Path,
    trust_unsigned: bool,
    explicit_trusted_public_keys: &[String],
    materialize: bool,
    state_dir: &Path,
    json_output: bool,
) -> Result<(), RunError> {
    let trusted_public_keys = resolve_store_verify_keys(None, explicit_trusted_public_keys, state_dir)?;
    let options = crunch_store::ArchiveImportOptions {
        trust_unsigned,
        trusted_public_keys,
        materialize,
    };
    let report = if is_stdio_path(source) {
        let mut stdin = tokio::io::stdin();
        crunch_store::import_store_archive(store, &mut stdin, &options)
            .await
            .map_err(|e| RunError::Internal(format!("archive import: {e}")))?
    } else {
        let file = tokio::fs::File::open(source)
            .await
            .map_err(|e| RunError::Internal(format!("opening archive {}: {e}", source.display())))?;
        let mut reader = tokio::io::BufReader::new(file);
        crunch_store::import_store_archive(store, &mut reader, &options)
            .await
            .map_err(|e| RunError::Internal(format!("archive import: {e}")))?
    };
    print_archive_import_report(&report, json_output)
}

async fn cmd_store_archive_list(source: &Path, json_output: bool) -> Result<(), RunError> {
    let report = if is_stdio_path(source) {
        let mut stdin = tokio::io::stdin();
        crunch_store::list_store_archive(&mut stdin)
            .await
            .map_err(|e| RunError::Internal(format!("archive list: {e}")))?
    } else {
        let file = tokio::fs::File::open(source)
            .await
            .map_err(|e| RunError::Internal(format!("opening archive {}: {e}", source.display())))?;
        let mut reader = tokio::io::BufReader::new(file);
        crunch_store::list_store_archive(&mut reader)
            .await
            .map_err(|e| RunError::Internal(format!("archive list: {e}")))?
    };
    print_archive_list_report(&report, json_output)
}

fn print_archive_export_report(report: &crunch_store::ArchiveExportReport, json_output: bool) -> Result<(), RunError> {
    if json_output {
        return print_json_report(report, "serializing archive export report");
    }
    for path in &report.paths {
        println!("ARCHIVE_EXPORT {} nar_size={} nar_sha256={}", path.store_path, path.nar_size, path.nar_sha256_hex);
    }
    eprintln!("exported={} payload_bytes={}", report.exported_count, report.total_payload_bytes);
    Ok(())
}

fn print_archive_import_report(report: &crunch_store::ArchiveImportReport, json_output: bool) -> Result<(), RunError> {
    if json_output {
        return print_json_report(report, "serializing archive import report");
    }
    for path in &report.paths {
        println!("ARCHIVE_IMPORT {} nar_size={} nar_sha256={}", path.store_path, path.nar_size, path.nar_sha256_hex);
    }
    eprintln!(
        "imported={} skipped_present={} payload_bytes={}",
        report.imported_count, report.skipped_already_present_count, report.total_payload_bytes
    );
    Ok(())
}

fn print_archive_list_report(report: &crunch_store::ArchiveListReport, json_output: bool) -> Result<(), RunError> {
    if json_output {
        return print_json_report(report, "serializing archive list report");
    }
    println!(
        "format={} store_prefix={} records={} payload_bytes={} compatibility={}",
        crunch_store::ARCHIVE_FORMAT_NAME,
        report.store_prefix,
        report.record_count,
        report.total_payload_bytes,
        report.compatibility
    );
    for path in &report.paths {
        println!(
            "ARCHIVE_PATH {} nar_size={} refs={} signatures={} ca={} root={} blake3={}",
            path.store_path,
            path.nar_size,
            path.reference_count,
            path.signature_count,
            path.ca.as_deref().unwrap_or("-"),
            path.root,
            path.payload_blake3
        );
    }
    Ok(())
}

fn print_json_report(report: &impl Serialize, context: &str) -> Result<(), RunError> {
    let rendered = serde_json::to_string_pretty(report).map_err(|e| RunError::Internal(format!("{context}: {e}")))?;
    println!("{rendered}");
    Ok(())
}

fn is_stdio_path(path: &Path) -> bool {
    path == Path::new("-")
}

async fn collect_all_pathinfos(
    store: &crunch_store::StoreHandle,
) -> Result<Vec<snix_store::path_info::PathInfo>, RunError> {
    use futures::StreamExt;
    use snix_store::pathinfoservice::PathInfoService;

    const MAX_PUSH_SCAN: usize = 1_000_000;
    let mut stream = store.pathinfo_service().list();
    let mut results = Vec::with_capacity(256);
    for _ in 0..MAX_PUSH_SCAN {
        let Some(result) = stream.next().await else { break };
        let pi = result.map_err(|e| RunError::Internal(format!("listing pathinfo: {e}")))?;
        results.push(pi);
    }
    Ok(results)
}

async fn collect_matching_pathinfos(
    store: &crunch_store::StoreHandle,
    selectors: &[String],
) -> Result<Vec<snix_store::path_info::PathInfo>, RunError> {
    use futures::StreamExt;
    use snix_store::pathinfoservice::PathInfoService;

    const MAX_PUSH_SCAN: usize = 1_000_000;
    let mut stream = store.pathinfo_service().list();
    let mut results = Vec::with_capacity(selectors.len());
    for _ in 0..MAX_PUSH_SCAN {
        let Some(result) = stream.next().await else { break };
        let pi = result.map_err(|e| RunError::Internal(format!("listing pathinfo: {e}")))?;
        let sp_str = pi.store_path.to_string();
        if selectors.iter().any(|sel| sp_str.contains(sel.as_str())) {
            results.push(pi);
        }
    }
    Ok(results)
}
