use crate::build_cmd::log_dir;
use crate::errors::RunError;

pub fn cmd_log(query: Option<&str>, list: bool) -> Result<(), RunError> {
    let dir = log_dir();
    if !dir.exists() {
        return Err(RunError::Internal(format!(
            "log directory {} does not exist (no builds yet?)",
            dir.display()
        )));
    }

    let mut entries: Vec<_> = std::fs::read_dir(&dir)
        .map_err(|e| RunError::Internal(format!("reading log dir: {e}")))?
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().is_some_and(|ext| ext == "log"))
        .collect();
    entries.sort_by_key(|e| e.file_name());

    if list || query.is_none() {
        if entries.is_empty() {
            eprintln!("No build logs found in {}", dir.display());
            return Ok(());
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
        return Ok(());
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
            let content = std::fs::read_to_string(entry.path())
                .map_err(|e| RunError::Internal(format!("reading log: {e}")))?;
            print!("{content}");
            Ok(())
        }
        None => Err(RunError::Internal(format!(
            "no log matching '{query}' in {}",
            dir.display(),
        ))),
    }
}
