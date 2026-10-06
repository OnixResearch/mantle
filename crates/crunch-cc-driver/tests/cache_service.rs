use std::{
    env, fs,
    io::{Read, Write},
    os::unix::net::UnixListener,
    path::{Path, PathBuf},
    process::{Command, Output},
    sync::{atomic::{AtomicBool, Ordering}, Arc},
    thread,
    time::Duration,
};

use crunch_rust_cache::RustCache;
use crunch_rust_cache_core::cc::{CcWireRequest, MAX_CC_FRAME_BYTES, MAX_CC_PROBE_DIAGNOSTIC_BYTES};
use serde_json::Value;
use snix_castore::{
    blobservice::MemoryBlobService,
    directoryservice::{RedbDirectoryService, RedbDirectoryServiceConfig},
};

struct CacheService {
    socket: PathBuf,
    stop: Arc<AtomicBool>,
    worker: Option<thread::JoinHandle<()>>,
}

impl CacheService {
    fn start(root: &Path) -> Self {
        let runtime = tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap();
        let directories = runtime.block_on(RedbDirectoryService::new(
            "cc-driver-service-test".to_string(),
            RedbDirectoryServiceConfig { path: None, read_only: false, cache_size: None },
        )).unwrap();
        let cache = RustCache::new(root.join("state"), Arc::new(MemoryBlobService::default()), Arc::new(directories)).unwrap();
        let socket = root.join("cache.sock");
        let listener = UnixListener::bind(&socket).unwrap();
        listener.set_nonblocking(true).unwrap();
        let stop = Arc::new(AtomicBool::new(false));
        let stopping = Arc::clone(&stop);
        let worker = thread::spawn(move || {
            while !stopping.load(Ordering::Relaxed) {
                let (mut stream, _) = match listener.accept() {
                    Ok(connection) => connection,
                    Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                        thread::sleep(Duration::from_millis(2));
                        continue;
                    }
                    Err(error) => panic!("cache listener failed: {error}"),
                };
                let mut size = [0; 8];
                stream.read_exact(&mut size).unwrap();
                let len = u64::from_be_bytes(size);
                assert!((1..=MAX_CC_FRAME_BYTES).contains(&len));
                let mut body = vec![0; len as usize];
                stream.read_exact(&mut body).unwrap();
                let request: CcWireRequest = serde_json::from_slice(&body).unwrap();
                let reply = cache.serve_cc_request(&request).unwrap();
                let bytes = serde_json::to_vec(&reply).unwrap();
                assert!(!bytes.is_empty() && bytes.len() as u64 <= MAX_CC_FRAME_BYTES);
                stream.write_all(&(bytes.len() as u64).to_be_bytes()).unwrap();
                stream.write_all(&bytes).unwrap();
            }
        });
        Self { socket, stop, worker: Some(worker) }
    }
}

impl Drop for CacheService {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Relaxed);
        self.worker.take().unwrap().join().unwrap();
    }
}

fn compiler(name: &str) -> PathBuf {
    env::split_paths(&env::var_os("PATH").expect("PATH required for GCC/G++ integration"))
        .map(|dir| dir.join(name))
        .find(|path| path.is_file())
        .unwrap_or_else(|| panic!("real {name} compiler required"))
        .canonicalize().unwrap()
}

fn compiler_args(source: &Path, object: &Path, depfile: &Path, extras: &[&str]) -> Vec<String> {
    let root = source.parent().unwrap();
    let mut args = vec![
        "-c".to_string(), source.display().to_string(),
        "-o".to_string(), object.display().to_string(),
        "-MMD".to_string(), "-MF".to_string(), depfile.display().to_string(),
        "-nostdinc".to_string(), "-w".to_string(),
        format!("-ffile-prefix-map={}=/cc-root-0", root.display()),
    ];
    args.extend(extras.iter().map(|arg| (*arg).to_string()));
    args
}

fn run_compiler(compiler: &Path, args: &[String]) -> Output {
    Command::new(compiler).env_clear().env("LC_ALL", "C").args(args).output().unwrap()
}

fn run_driver(
    compiler: &Path, service: &CacheService, receipt: &Path, script: Option<&Path>, args: &[String], platform: &str,
) -> (Output, Value) {
    let mut command = Command::new(env!("CARGO_BIN_EXE_mantle-cc-cache-driver"));
    command.env_clear().env("LC_ALL", "C")
        .arg("--compiler").arg(compiler)
        .arg("--socket").arg(&service.socket)
        .arg("--receipt").arg(receipt)
        .arg("--platform-digest").arg(platform);
    if let Some(script) = script { command.arg("--probe-script").arg(script); }
    let output = command.arg("--").args(args).output().unwrap();
    let record = serde_json::from_slice(&fs::read(receipt).unwrap()).unwrap();
    (output, record)
}

fn discard_artifacts(object: &Path, depfile: &Path) {
    if object.exists() { fs::remove_file(object).unwrap(); }
    if depfile.exists() { fs::remove_file(depfile).unwrap(); }
}

#[test]
fn real_c_and_cpp_objects_and_depfiles_match_uncached_compilation_at_same_and_relocated_roots() {
    let temp = tempfile::Builder::new().prefix("mantle_cc_service_").tempdir_in("/tmp").unwrap();
    let service = CacheService::start(temp.path());
    let artifacts = temp.path().join("artifacts");
    fs::create_dir(&artifacts).unwrap();
    for (compiler_name, suffix) in [("gcc", "c"), ("g++", "cpp")] {
        let compiler = compiler(compiler_name);
        let original = temp.path().join(suffix).join("original-source");
        let relocated = temp.path().join(suffix).join("relocated-source");
        fs::create_dir_all(&original).unwrap();
        fs::create_dir_all(&relocated).unwrap();
        let source_name = format!("ordered.{suffix}");
        for root in [&original, &relocated] {
            fs::write(root.join(&source_name), "#include \"z.h\"\n#include \"a.h\"\nint answer(void) { return Z + A; }\n").unwrap();
            fs::write(root.join("z.h"), "#define Z 20\n").unwrap();
            fs::write(root.join("a.h"), "#define A 22\n").unwrap();
        }
        let object = artifacts.join(format!("{suffix}.o"));
        let depfile = artifacts.join(format!("{suffix}.d"));
        let receipt = artifacts.join(format!("{suffix}-receipt.json"));
        let mut cold_key = String::new();
        let mut cold_object = Vec::new();
        for (iteration, root) in [&original, &original, &relocated].into_iter().enumerate() {
            let args = compiler_args(&root.join(&source_name), &object, &depfile, &[]);
            discard_artifacts(&object, &depfile);
            let direct = run_compiler(&compiler, &args);
            assert!(direct.status.success(), "direct {compiler_name}: {}", String::from_utf8_lossy(&direct.stderr));
            let direct_object = fs::read(&object).unwrap();
            let direct_depfile = fs::read(&depfile).unwrap();
            assert!(direct_depfile.windows(3).any(|bytes| bytes == b" \\\n"));
            let dep_text = String::from_utf8(direct_depfile.clone()).unwrap();
            assert!(dep_text.find("z.h").unwrap() < dep_text.find("a.h").unwrap());
            discard_artifacts(&object, &depfile);
            let (output, record) = run_driver(&compiler, &service, &receipt, None, &args, &"a".repeat(64));
            assert!(output.status.success(), "cached {compiler_name}: {}", String::from_utf8_lossy(&output.stderr));
            assert_eq!(record["disposition"], if iteration == 0 { "published" } else { "hit" });
            assert_eq!(fs::read(&object).unwrap(), direct_object, "{compiler_name} object differs on iteration {iteration}");
            assert_eq!(fs::read(&depfile).unwrap(), direct_depfile, "{compiler_name} depfile differs on iteration {iteration}");
            if iteration == 0 {
                cold_key = record["action_key"].as_str().unwrap().to_string();
                cold_object = direct_object;
            } else {
                assert_eq!(record["action_key"], cold_key);
                assert_eq!(direct_object, cold_object);
            }
        }
        fs::write(relocated.join("a.h"), "#define A 23\n").unwrap();
        discard_artifacts(&object, &depfile);
        let changed_args = compiler_args(&relocated.join(&source_name), &object, &depfile, &[]);
        let (changed, record) = run_driver(&compiler, &service, &receipt, None, &changed_args, &"a".repeat(64));
        assert!(changed.status.success(), "{compiler_name} changed dependency");
        assert_eq!(record["disposition"], "published");
        assert_ne!(record["action_key"], cold_key);
        assert_ne!(fs::read(&object).unwrap(), cold_object);
    }
}

#[test]
fn classified_probe_replays_only_complete_bound_failures_and_changes_each_declared_identity() {
    let temp = tempfile::Builder::new().prefix("mantle_cc_probe_").tempdir_in("/tmp").unwrap();
    let service = CacheService::start(temp.path());
    let source_root = temp.path().join("source");
    let artifacts = temp.path().join("artifacts");
    fs::create_dir(&source_root).unwrap();
    fs::create_dir(&artifacts).unwrap();
    let source = source_root.join("fail.c");
    let header = source_root.join("probe.h");
    let script = temp.path().join("configure-script.sh");
    fs::write(&source, "#include \"probe.h\"\nint probe(void) { return NOT_DECLARED; }\n").unwrap();
    fs::write(&header, "#define PROBE 1\n").unwrap();
    fs::write(&script, "#!/bin/sh\n# first probe\n").unwrap();
    let gcc = compiler("gcc");
    let object = artifacts.join("probe.o");
    let depfile = artifacts.join("probe.d");
    let receipt = artifacts.join("probe-receipt.json");
    let stable_flags = ["-fdiagnostics-color=never", "-fmessage-length=0"];
    let args = compiler_args(&source, &object, &depfile, &stable_flags);
    let direct = run_compiler(&gcc, &args);
    assert!(!direct.status.success());
    assert!(!object.exists());
    let baseline_depfile = fs::read(&depfile).unwrap();
    discard_artifacts(&object, &depfile);
    let platform = "b".repeat(64);
    let (cold, published) = run_driver(&gcc, &service, &receipt, Some(&script), &args, &platform);
    assert_eq!(cold.status.code(), direct.status.code());
    assert_eq!(cold.stdout, direct.stdout);
    assert_eq!(cold.stderr, direct.stderr);
    assert_eq!(fs::read(&depfile).unwrap(), baseline_depfile);
    assert_eq!(published["disposition"], "probe-failure-published");
    assert_eq!(published["probe_reads"][0]["disposition"], "miss");
    let first_key = published["action_key"].as_str().unwrap().to_string();
    discard_artifacts(&object, &depfile);
    let (hit, reused) = run_driver(&gcc, &service, &receipt, Some(&script), &args, &platform);
    assert_eq!(reused["disposition"], "probe-failure-hit");
    assert_eq!(reused["probe_reads"][0]["disposition"], "hit");
    assert_eq!(reused["probe_reads"][1]["disposition"], "hit");
    assert_eq!(reused["action_key"], first_key);
    assert_eq!(hit.status.code(), cold.status.code());
    assert_eq!(hit.stdout, cold.stdout);
    assert_eq!(hit.stderr, cold.stderr);
    assert_eq!(fs::read(&depfile).unwrap(), baseline_depfile);
    assert!(!object.exists());

    fs::write(&script, "#!/bin/sh\n# changed probe script\n").unwrap();
    discard_artifacts(&object, &depfile);
    let (_, script_changed) = run_driver(&gcc, &service, &receipt, Some(&script), &args, &platform);
    assert_eq!(script_changed["disposition"], "probe-failure-published");
    assert_ne!(script_changed["action_key"], first_key);
    fs::write(&header, "#define PROBE 2\n").unwrap();
    discard_artifacts(&object, &depfile);
    let (_, dependency_changed) = run_driver(&gcc, &service, &receipt, Some(&script), &args, &platform);
    assert_eq!(dependency_changed["disposition"], "probe-failure-published");
    assert_ne!(dependency_changed["action_key"], script_changed["action_key"]);
    discard_artifacts(&object, &depfile);
    let (_, platform_changed) = run_driver(&gcc, &service, &receipt, Some(&script), &args, &"c".repeat(64));
    assert_eq!(platform_changed["disposition"], "probe-failure-published");
    assert_ne!(platform_changed["action_key"], dependency_changed["action_key"]);
    discard_artifacts(&object, &depfile);
    let flag_args = compiler_args(&source, &object, &depfile, &[stable_flags[0], stable_flags[1], "-DPROBE_FLAG=1"]);
    let (_, flags_changed) = run_driver(&gcc, &service, &receipt, Some(&script), &flag_args, &platform);
    assert_eq!(flags_changed["disposition"], "probe-failure-published");
    assert_ne!(flags_changed["action_key"], dependency_changed["action_key"]);
    discard_artifacts(&object, &depfile);
    let (_, tool_changed) = run_driver(&compiler("g++"), &service, &receipt, Some(&script), &args, &platform);
    assert_eq!(tool_changed["disposition"], "probe-failure-published");
    assert_ne!(tool_changed["action_key"], dependency_changed["action_key"]);

    let unknown = source_root.join("missing.c");
    fs::write(&unknown, "#include \"header-does-not-exist.h\"\nint probe(void) { return 1; }\n").unwrap();
    let unknown_args = compiler_args(&unknown, &object, &depfile, &stable_flags);
    for _ in 0..2 {
        discard_artifacts(&object, &depfile);
        let (output, record) = run_driver(&gcc, &service, &receipt, Some(&script), &unknown_args, &platform);
        assert!(!output.status.success());
        assert_eq!(record["disposition"], "probe-compiler-failure");
        assert_eq!(record["probe_reads"][0]["disposition"], "miss");
        assert!(!object.exists());
        assert!(!depfile.exists());
    }

    let oversized = source_root.join("oversized.c");
    fs::write(&oversized, format!(
        "#include \"probe.h\"\nint probe(void) {{ return {}; }}\n",
        "X".repeat(MAX_CC_PROBE_DIAGNOSTIC_BYTES + 1024),
    )).unwrap();
    let oversized_args = compiler_args(&oversized, &object, &depfile, &stable_flags);
    let direct = run_compiler(&gcc, &oversized_args);
    assert!(!direct.status.success());
    assert!(direct.stderr.len() > MAX_CC_PROBE_DIAGNOSTIC_BYTES);
    assert!(depfile.exists(), "oversized diagnostic must still have a complete depfile");
    for _ in 0..2 {
        discard_artifacts(&object, &depfile);
        let (output, record) = run_driver(&gcc, &service, &receipt, Some(&script), &oversized_args, &platform);
        assert_eq!(output.status.code(), direct.status.code());
        assert_eq!(output.stderr, direct.stderr, "full diagnostics must stream even when not cacheable");
        assert_eq!(record["disposition"], "probe-compiler-failure");
        assert!(depfile.exists());
        assert!(!object.exists());
    }

    let success = source_root.join("success.c");
    fs::write(&success, "#include \"probe.h\"\nint supported(void) { return PROBE; }\n").unwrap();
    let success_args = compiler_args(&success, &object, &depfile, &stable_flags);
    let mut published_key = None;
    for iteration in 0..2 {
        discard_artifacts(&object, &depfile);
        let direct = run_compiler(&gcc, &success_args);
        assert!(direct.status.success());
        let direct_object = fs::read(&object).unwrap();
        let direct_depfile = fs::read(&depfile).unwrap();
        discard_artifacts(&object, &depfile);
        let (output, record) = run_driver(&gcc, &service, &receipt, Some(&script), &success_args, &platform);
        assert!(output.status.success());
        assert_eq!(record["disposition"], if iteration == 0 { "published" } else { "hit" });
        assert_eq!(record["probe_reads"][0]["kind"], "failure");
        assert_eq!(record["probe_reads"][0]["disposition"], "miss");
        assert_eq!(record["probe_reads"][1]["kind"], "object");
        assert_eq!(record["probe_reads"][1]["disposition"], if iteration == 0 { "miss" } else { "hit" });
        if iteration == 1 {
            assert_eq!(record["probe_reads"][2]["operation"], "read");
            assert_eq!(record["probe_reads"][2]["disposition"], "hit");
            assert_eq!(&record["action_key"], published_key.as_ref().unwrap());
        } else {
            published_key = Some(record["action_key"].clone());
        }
        assert_eq!(fs::read(&object).unwrap(), direct_object);
        assert_eq!(fs::read(&depfile).unwrap(), direct_depfile);
    }
}
