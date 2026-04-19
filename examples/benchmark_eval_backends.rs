use std::ffi::OsString;
use std::path::PathBuf;
use std::time::Instant;

use crunch_eval::session::EvaluationSession;
use crunch_eval::session::RootForceExecutionPolicy;
use crunch_glue::CrunchDerivation;

const REPEAT_COUNT: u32 = 10;
const ROOT_COUNT_EXPECTED: usize = 16;
const MAX_CONCURRENCY: u32 = 4;
const FIXTURE_PATH: &str = "tests/fixtures/wide_package_set.ncl";

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn fixture_path() -> PathBuf {
    repo_root().join(FIXTURE_PATH)
}

fn import_paths() -> Vec<OsString> {
    vec![crunch_eval::stdlib::stdlib_import_path().expect("stdlib import path").into_os_string()]
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let inline_wall_ns = benchmark_policy(RootForceExecutionPolicy::Inline)?;
    let threaded_wall_ns = benchmark_policy(RootForceExecutionPolicy::PreferThreaded)?;

    println!("workload: eval-backend-wide-package-set");
    println!("  repeat_count: {REPEAT_COUNT}");
    println!("  max_concurrency: {MAX_CONCURRENCY}");
    println!("  inline_total_wall_ns: {inline_wall_ns}");
    println!("  threaded_total_wall_ns: {threaded_wall_ns}");
    Ok(())
}

fn benchmark_policy(policy: RootForceExecutionPolicy) -> Result<u128, Box<dyn std::error::Error>> {
    let mut samples_ns = Vec::with_capacity(REPEAT_COUNT as usize);
    let fixture_path = fixture_path();
    let import_paths = import_paths();
    assert!(REPEAT_COUNT >= 3, "repeat count must be at least 3");
    assert!(MAX_CONCURRENCY >= 1, "max concurrency must be at least 1");

    for _sample_index in 0..REPEAT_COUNT {
        let start = Instant::now();
        let session = EvaluationSession::open_file(&fixture_path, &import_paths)?;
        let roots = session.force_all_roots_with_policy::<CrunchDerivation>(MAX_CONCURRENCY, policy)?;
        assert_eq!(roots.len(), ROOT_COUNT_EXPECTED, "fixture root count must stay stable");
        samples_ns.push(start.elapsed().as_nanos());
    }

    Ok(median_u128(&mut samples_ns))
}

fn median_u128(values: &mut [u128]) -> u128 {
    assert!(!values.is_empty(), "median requires at least one value");
    assert!(values.len() <= REPEAT_COUNT as usize, "median input must stay bounded");
    values.sort_unstable();
    values[values.len() / 2]
}
