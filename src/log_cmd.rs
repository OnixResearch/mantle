use crate::build_cmd::log_dir;
use crate::errors::RunError;

/// Effect identity of the build-log read.
const LOG_READ_EFFECT: &str = "read-files";

/// Classify the completed log read before its terminal report.
fn classify_log_read() -> Result<(), RunError> {
    let plan =
        mantle_application_contract::plan_effects(mantle_application_contract::CommandFamily::StoreAdministration, &[
            LOG_READ_EFFECT,
        ])
        .ok_or_else(|| RunError::Internal("log read effect plan exceeds its bound".to_string()))?;
    let observation = mantle_application_contract::Observation {
        effect_id: mantle_application_contract::EffectId(String::from(LOG_READ_EFFECT)),
        status: mantle_application_contract::ObservationStatus::Succeeded,
        diagnostics_code: None,
    };
    match mantle_application_contract::classify_observations(&plan, &[observation]) {
        mantle_application_contract::ApplicationOutcome::Completed => Ok(()),
        other => Err(RunError::Internal(format!("log observations were inconsistent: {other:?}"))),
    }
}

pub fn cmd_log(query: Option<&str>, list: bool) -> Result<(), RunError> {
    let dir = log_dir();
    if !dir.exists() {
        return Err(RunError::Internal(format!("log directory {} does not exist (no builds yet?)", dir.display())));
    }

    let mut entries: Vec<_> = std::fs::read_dir(&dir)
        .map_err(|e| RunError::Internal(format!("reading log dir: {e}")))?
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().is_some_and(|ext| ext == "log"))
        .collect();
    entries.sort_by_key(|e| e.file_name());
    debug_assert!(entries.iter().all(|entry| entry.path().extension().is_some_and(|ext| ext == "log")));
    debug_assert!(entries.windows(2).all(|pair| pair[0].file_name() <= pair[1].file_name()));

    if list || query.is_none() {
        if entries.is_empty() {
            eprintln!("No build logs found in {}", dir.display());
            return classify_log_read();
        }
        for entry in &entries {
            let path = entry.path();
            let name = path.file_stem().unwrap_or_default().to_string_lossy();
            if let Ok(content) = std::fs::read_to_string(&path) {
                let status = content
                    .lines()
                    .find(|line| line.starts_with("# status:"))
                    .map(|line| line.trim_start_matches("# status: "))
                    .unwrap_or("unknown");
                let drv_label = content
                    .lines()
                    .find(|line| line.starts_with("# derivation:"))
                    .map(|line| line.trim_start_matches("# derivation: "))
                    .unwrap_or("");
                println!("{name}  [{status}]  {drv_label}");
            } else {
                println!("{name}");
            }
        }
        return classify_log_read();
    }

    let Some(query) = query else {
        return Err(RunError::Internal("log query is None after guard".to_string()));
    };
    let matched = entries.iter().find(|entry| {
        let name = entry.file_name();
        let name_str = name.to_string_lossy();
        name_str.contains(query)
    });

    match matched {
        Some(entry) => {
            let content =
                std::fs::read_to_string(entry.path()).map_err(|e| RunError::Internal(format!("reading log: {e}")))?;
            print!("{content}");
            classify_log_read()
        }
        None => Err(RunError::Internal(format!("no log matching '{query}' in {}", dir.display(),))),
    }
}
