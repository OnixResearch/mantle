//! NDJSON fresh/persistent-root differential research runner.
use std::collections::BTreeMap;
use std::ffi::OsString;
use std::io::BufRead;
use std::io::Write;
use std::path::PathBuf;
use std::time::Instant;

use crunch_eval::session::EvaluationSession;
use crunch_eval::session::incremental::PersistentRootCache;
use crunch_eval::session::incremental::ReuseDiagnostics;
use crunch_glue::ConversionCache;
use crunch_glue::CrunchDerivation;
use serde::Deserialize;
use serde_json::Value;
use serde_json::json;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Request {
    root: PathBuf,
    #[serde(default)]
    import_paths: Vec<OsString>,
    #[serde(default)]
    mode: String,
    #[serde(default)]
    configuration: String,
}

fn memory_kib(field: &str) -> Option<u64> {
    // Linux observations only; unsupported platforms must report null, not zero.
    std::fs::read_to_string("/proc/self/status").ok()?.lines().find_map(|line| {
        let value = line.strip_prefix(field)?;
        value.split_whitespace().next()?.parse().ok()
    })
}

fn plan(roots: Vec<(String, CrunchDerivation)>) -> Result<BTreeMap<String, Value>, String> {
    let mut cache = ConversionCache::default();
    let mut result = BTreeMap::new();
    for (label, derivation) in roots {
        let (path, _) = crunch_glue::convert(&derivation, &mut cache).map_err(|error| error.to_string())?;
        result.insert(label, json!({"drv_path": path.to_absolute_path(), "derivation": derivation}));
        drop(cache.drain_pending());
    }
    Ok(result)
}

fn measure(request: &Request, cache: &mut PersistentRootCache<CrunchDerivation>) -> Result<Value, String> {
    let start = Instant::now();
    let scan_start = Instant::now();
    let dependencies = crunch_eval::watch_imports::source_dependencies(&request.root, &request.import_paths)
        .map_err(|error| error.to_string())?;
    let scan_ns = scan_start.elapsed().as_nanos();
    let evaluation_start = Instant::now();
    let old_labels = cache.retained_labels();
    let candidate = match request.mode.as_str() {
        "" | "fresh" => {
            cache.clear();
            None
        }
        "independent-import-roots" => cache
            .prepare(&request.root, &request.import_paths, request.configuration.as_bytes(), || false)
            .map_err(|error| error.to_string())?,
        _ => return Err("unknown research evaluation mode".to_string()),
    };
    let (roots, force_count, mode, diagnostics) = if let Some(candidate) = &candidate {
        (
            candidate.roots().map(|(label, value)| (label.to_string(), value.clone())).collect(),
            None,
            "independent-import-roots",
            candidate.diagnostics().clone(),
        )
    } else {
        let mut session =
            EvaluationSession::open_file(&request.root, &request.import_paths).map_err(|error| error.to_string())?;
        let roots = session.force_all_roots::<CrunchDerivation>().map_err(|error| error.to_string())?;
        let diagnostics = ReuseDiagnostics {
            executed_roots: roots.iter().map(|(label, _)| label.clone()).collect(),
            evicted_roots: old_labels,
            ..ReuseDiagnostics::default()
        };
        let mode = if request.mode == "independent-import-roots" {
            "fresh-fallback-unsupported-or-capacity"
        } else {
            "fresh-session"
        };
        (roots, Some(session.explicit_force_count()), mode, diagnostics)
    };
    let evaluation_ns = evaluation_start.elapsed().as_nanos();
    let plan_start = Instant::now();
    let planned = plan(roots)?;
    let planning_ns = plan_start.elapsed().as_nanos();
    let candidate_wall_ns = start.elapsed().as_nanos();
    let oracle_start = Instant::now();
    let oracle =
        crunch_eval::evaluate_and_extract_named_roots::<CrunchDerivation>(&request.root, &request.import_paths)
            .map_err(|error| error.to_string())?;
    let oracle = plan(oracle)?;
    if planned != oracle {
        return Err("candidate differs from separate fresh eager oracle".to_string());
    }
    let oracle_ns = oracle_start.elapsed().as_nanos();
    let validation_start = Instant::now();
    if let Some(candidate) = candidate {
        cache
            .commit(candidate, request.configuration.as_bytes(), || false)
            .map_err(|error| error.to_string())?;
    }
    Ok(json!({
        "schema": "mantle-evaluation-baseline-v1", "mode": mode, "dependencies": dependencies,
        "scan_ns": scan_ns, "evaluation_ns": evaluation_ns, "planning_ns": planning_ns,
        "candidate_wall_ns": candidate_wall_ns, "oracle_ns": oracle_ns,
        "admission_validation_ns": validation_start.elapsed().as_nanos(),
        "explicit_root_forces": force_count, "thunk_evaluations": null,
        "executed_root_computations": diagnostics.executed_roots.len(),
        "reused_root_computations": diagnostics.reused_roots.len(), "diagnostics": diagnostics,
        "oracle_equal": true, "rss_kib": memory_kib("VmRSS:"),
        "process_peak_rss_kib": memory_kib("VmHWM:"),
        "compile_count": 0, "builder_execution_count": 0,
        "execution_scope": "evaluation-and-conversion-only", "roots": planned,
    }))
}

fn context_probe() -> Result<(), String> {
    let directory = tempfile::tempdir().map_err(|error| error.to_string())?;
    let middle = directory.path().join("middle.ncl");
    let leaf = directory.path().join("leaf.ncl");
    std::fs::write(&middle, "import \"leaf.ncl\"").map_err(|error| error.to_string())?;
    std::fs::write(&leaf, "1").map_err(|error| error.to_string())?;
    let mut persistent =
        crunch_eval::Context::new().with_added_import_paths(vec![directory.path().as_os_str().to_owned()]);
    let source = "import \"middle.ncl\"";
    let first = persistent
        .eval_deep(source)
        .map_err(|_| "initial probe evaluation failed")?
        .to_serde::<u32>()
        .map_err(|error| error.to_string())?;
    std::fs::write(&leaf, "2").map_err(|error| error.to_string())?;
    let reused = persistent
        .eval_deep(source)
        .map_err(|_| "repeated probe evaluation failed")?
        .to_serde::<u32>()
        .map_err(|error| error.to_string())?;
    let fresh = crunch_eval::Context::new()
        .with_added_import_paths(vec![directory.path().as_os_str().to_owned()])
        .eval_deep(source)
        .map_err(|_| "fresh probe evaluation failed")?
        .to_serde::<u32>()
        .map_err(|error| error.to_string())?;
    println!(
        "{}",
        json!({"schema": "mantle-context-reuse-probe-v1", "first": first,
        "reused": reused, "fresh": fresh, "oracle_equal": reused == fresh,
        "admitted_to_watch": false})
    );
    if first != 1 || fresh != 2 {
        return Err("probe fixture did not establish the intended edit".to_string());
    }
    Ok(())
}

fn main() -> Result<(), String> {
    if std::env::args().nth(1).as_deref() == Some("--context-probe") {
        return context_probe();
    }
    let stdin = std::io::stdin();
    let mut stdout = std::io::stdout().lock();
    let mut cache = PersistentRootCache::default();
    for line in stdin.lock().lines() {
        let line = line.map_err(|error| error.to_string())?;
        let result = serde_json::from_str::<Request>(&line)
            .map_err(|error| error.to_string())
            .and_then(|request| measure(&request, &mut cache));
        let response = match result {
            Ok(value) => value,
            Err(error) => {
                cache.clear();
                json!({"schema": "mantle-evaluation-baseline-v1", "mode": "rejected-cache-evicted",
                    "error": error, "admitted_to_watch": false})
            }
        };
        writeln!(stdout, "{response}").map_err(|error| error.to_string())?;
        stdout.flush().map_err(|error| error.to_string())?;
    }
    Ok(())
}
