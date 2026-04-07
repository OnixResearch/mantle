use crate::build_cmd::state_dir;
use crate::errors::RunError;

pub fn cmd_store(action: crate::StoreAction) -> Result<(), RunError> {
    use snix_store::pathinfoservice::{RedbPathInfoService, RedbPathInfoServiceConfig};

    let state = state_dir();
    let db_path = state.join("pathinfo.redb");
    if !db_path.exists() {
        return Err(RunError::Internal(format!(
            "PathInfo database {} does not exist (no builds yet?)",
            db_path.display()
        )));
    }

    let rt = tokio::runtime::Runtime::new()
        .map_err(|e| RunError::Internal(format!("tokio runtime: {e}")))?;

    rt.block_on(async {
        let read_only = matches!(action, crate::StoreAction::List | crate::StoreAction::Info { .. });
        let svc = RedbPathInfoService::new(
            "crunch".to_string(),
            RedbPathInfoServiceConfig {
                path: Some(db_path.clone()),
                read_only,
                cache_size: None,
            },
        )
        .await
        .map_err(|e| RunError::Internal(format!("opening PathInfo database: {e}")))?;

        match action {
            crate::StoreAction::List => cmd_store_list(&svc).await,
            crate::StoreAction::Info { path } => cmd_store_info(&svc, &path).await,
            crate::StoreAction::Verify { path } => cmd_store_verify(&svc, path.as_deref()).await,
        }
    })
}

async fn cmd_store_list(
    svc: &impl snix_store::pathinfoservice::PathInfoService,
) -> Result<(), RunError> {
    let entries = crunch_store::store_list(svc)
        .await
        .map_err(|e| RunError::Internal(format!("{e}")))?;
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

async fn cmd_store_info(
    svc: &impl snix_store::pathinfoservice::PathInfoService,
    path: &str,
) -> Result<(), RunError> {
    let details = crunch_store::store_info(svc, path)
        .await
        .map_err(|e| RunError::Internal(format!("{e}")))?;
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
        println!("node:       {}", detail.node);
    }
    Ok(())
}

async fn cmd_store_verify(
    svc: &impl snix_store::pathinfoservice::PathInfoService,
    path_filter: Option<&str>,
) -> Result<(), RunError> {
    let results = crunch_store::store_verify(svc, path_filter)
        .await
        .map_err(|e| RunError::Internal(format!("{e}")))?;

    let mut checked: u32 = 0;
    let mut mismatches: u32 = 0;
    for result in &results {
        match result {
            crunch_store::VerifyResult::Ok(path) => {
                println!("OK {path}");
            }
            crunch_store::VerifyResult::Missing(path) => {
                println!("MISSING {path}");
                mismatches = mismatches.saturating_add(1);
            }
            crunch_store::VerifyResult::Mismatch {
                path,
                stored_hash,
                actual_hash,
            } => {
                println!("MISMATCH {path}  stored={stored_hash}  actual={actual_hash}");
                mismatches = mismatches.saturating_add(1);
            }
        }
        checked = checked.saturating_add(1);
    }

    eprintln!("{checked} checked, {mismatches} mismatches");
    if mismatches > 0 {
        return Err(RunError::Build(format!(
            "{mismatches} path(s) failed verification"
        )));
    }
    Ok(())
}
