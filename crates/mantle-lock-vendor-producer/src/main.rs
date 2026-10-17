//! Standalone declared Cargo.lock vendor selector and native dynamic-plan producer.
//! Usage: producer --selected-table <Cargo.lock> <full-shared-table>
//! Or: producer <Cargo.lock> <selected-shared-table> <python-store-executable> <bundle-store-path>
//! <rust-store-path> [extra-runtime-store-path ...]
use std::collections::BTreeSet;
use std::env;
use std::fs;
use std::io::Read;
use std::io::Write;
use std::io::{self};
use std::path::Path;

use mantle_lock_vendor_core::Limits;
use mantle_lock_vendor_core::MAX_LOCK_BYTES;
use mantle_lock_vendor_core::plan_cargo_vendor_with_table;
use mantle_lock_vendor_core::shared_table::HashMode;
use mantle_lock_vendor_core::shared_table::MAX_TABLE_BYTES;
use mantle_lock_vendor_core::shared_table::SharedHashTable;

const MAX_PLAN: usize = 4 * 1024 * 1024;
const SHARD_SIZE: usize = 240;
const MAX_EXTRA_RUNTIME_SOURCES: usize = 12;
fn escape(value: &str) -> String {
    let mut out = String::from("\"");
    for c in value.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if c <= '\u{1f}' => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}
fn store_path(value: &str, executable: bool) -> Result<(), String> {
    let mut components = value.split('/');
    if components.next() != Some("")
        || components.next().is_none_or(str::is_empty)
        || components.next() != Some("store")
    {
        return Err(format!("not an absolute store path: {value}"));
    }
    let name = components.next().ok_or("missing store basename")?;
    let suffix: Vec<_> = components.collect();
    if (executable && (suffix.is_empty() || suffix.iter().any(|s| s.is_empty() || *s == "." || *s == "..")))
        || (!executable && !suffix.is_empty())
        || name.len() < 34
        || name.as_bytes()[32] != b'-'
        || !name[..32].bytes().all(|b| b.is_ascii_digit() || b.is_ascii_lowercase())
    {
        return Err(format!("not a declared store path: {value}"));
    }
    Ok(())
}
fn unit(
    id: &str,
    name: &str,
    builder: &str,
    system: &str,
    args: &[String],
    env_vars: &[(&str, &str)],
    inputs: &[String],
    fixed: Option<(&str, &str)>,
) -> String {
    let arr = |values: &[String]| values.iter().map(|s| escape(s)).collect::<Vec<_>>().join(",");
    let vars = env_vars.iter().map(|(k, v)| format!("{}:{}", escape(k), escape(v))).collect::<Vec<_>>().join(",");
    let fixed = fixed.map_or("null".to_string(), |(mode, hash)| {
        format!("{{\"mode\":{},\"algo\":\"sha256\",\"hash\":{}}}", escape(mode), escape(hash))
    });
    format!(
        "{{\"id\":{},\"derivation\":{{\"name\":{},\"builder\":{},\"system\":{},\"args\":[{}],\"outputs\":[\"out\"],\"env\":{{{}}},\"inputs\":[{}],\"fixed_output\":{},\"addressing_mode\":\"input-addressed\",\"sandbox\":\"native\",\"dynamic_plan_outputs\":[]}},\"requested_outputs\":[\"out\"],\"policy\":{{\"sandbox\":\"inherit\",\"substitutions\":\"inherit\",\"store_prefix\":\"inherit\",\"host_paths\":\"none\"}}}}",
        escape(id),
        escape(name),
        escape(builder),
        escape(system),
        arr(args),
        vars,
        inputs.join(","),
        fixed
    )
}
fn output_ref(id: &str) -> String {
    format!("{{\"kind\":\"unit_output\",\"unit\":{},\"output\":\"out\"}}", escape(id))
}
fn source_ref(id: &str) -> String {
    format!("{{\"kind\":\"source\",\"source\":{}}}", escape(id))
}
fn marker(id: &str) -> String {
    format!("{{{{mantle-unit-output:{id}:out}}}}")
}
/// Stable bounded derivation-name stem. Plan-local sorted ordinals distinguish
/// equal truncated stems even when their fixed-output checksums also match.
fn fetch_stem(name: &str, version: &str) -> String {
    let mut stem = String::with_capacity(48);
    for byte in name.bytes().chain(std::iter::once(b'-')).chain(version.bytes()).take(48) {
        stem.push(match byte {
            b'A'..=b'Z' => byte.to_ascii_lowercase() as char,
            b'a'..=b'z' | b'0'..=b'9' | b'.' | b'-' => byte as char,
            _ => '-',
        });
    }
    stem
}
fn plan(
    lock: &Path,
    table: &Path,
    builder: &str,
    bundle: &str,
    rust: &str,
    runtime: &[&str],
) -> Result<String, String> {
    store_path(bundle, false)?;
    if lock != Path::new(bundle).join("workspace/Cargo.lock") {
        return Err("producer Cargo.lock must be the declared source bundle's workspace/Cargo.lock".into());
    }
    store_path(table.to_str().ok_or("non-UTF-8 selected table path")?, false)?;
    let lock_bytes = bounded_read(lock, MAX_LOCK_BYTES as u64)?;
    let table_bytes = bounded_read(table, MAX_TABLE_BYTES as u64)?;
    plan_from_bytes(lock, table, builder, bundle, rust, runtime, &lock_bytes, &table_bytes)
}
fn plan_from_bytes(
    lock: &Path,
    table: &Path,
    builder: &str,
    bundle: &str,
    rust: &str,
    runtime: &[&str],
    lock_bytes: &[u8],
    table_bytes: &[u8],
) -> Result<String, String> {
    store_path(builder, true)?;
    store_path(bundle, false)?;
    store_path(rust, false)?;
    if lock != Path::new(bundle).join("workspace/Cargo.lock") {
        return Err("producer Cargo.lock must be the declared source bundle's workspace/Cargo.lock".into());
    }
    let table_path = table.to_str().ok_or("non-UTF-8 selected table path")?;
    store_path(table_path, false)?;
    if runtime.len() > MAX_EXTRA_RUNTIME_SOURCES {
        return Err("extra runtime source budget exceeded".into());
    }
    let python = builder.split("/bin/").next().ok_or("missing Python store root")?;
    let mut declared = BTreeSet::from([bundle, table_path, python, rust]);
    let mut runtime_sources = Vec::with_capacity(runtime.len());
    let mut runtime_inputs = Vec::with_capacity(runtime.len());
    for (index, path) in runtime.iter().enumerate() {
        store_path(path, false)?;
        if !declared.insert(path) {
            return Err(format!("duplicate runtime source: {path}"));
        }
        let id = format!("src.runtime.{index}");
        runtime_sources.push(format!("{{\"id\":{},\"path\":{},\"nar_blake3\":null}}", escape(&id), escape(path)));
        runtime_inputs.push(source_ref(&id));
    }
    let git_tool = runtime
        .iter()
        .enumerate()
        .find(|(_, root)| Path::new(root).join("bin/git").is_file())
        .map(|(index, _)| index);
    let patch_tool = runtime
        .iter()
        .enumerate()
        .find(|(_, root)| Path::new(root).join("bin/patch").is_file())
        .map(|(index, _)| index);
    if runtime.iter().filter(|root| Path::new(root).join("bin/git").is_file()).count() > 1
        || runtime.iter().filter(|root| Path::new(root).join("bin/patch").is_file()).count() > 1
    {
        return Err("ambiguous declared Git or patch tool".into());
    }
    if lock_bytes.len() > MAX_LOCK_BYTES as usize || table_bytes.len() > MAX_TABLE_BYTES as usize {
        return Err("producer lock or selected table exceeds declared byte bound".into());
    }
    let shared = SharedHashTable::parse(table_bytes).map_err(|e| format!("shared hashes: {e:?}"))?;
    let planned = plan_cargo_vendor_with_table(lock_bytes, Limits::default(), Some(&shared))
        .map_err(|e| format!("Cargo.lock: {e:?}"))?;
    let selected = planned.selected_shared_hashes(&shared).map_err(|e| format!("selected shared hashes: {e:?}"))?;
    if selected.render().as_bytes() != table_bytes {
        return Err("producer requires the canonical selected table, not a mutable full pin table".into());
    }
    if git_tool.is_some() != planned.artifacts.iter().any(|artifact| artifact.git_rev.is_some()) {
        return Err("Git lock entries require exactly one declared Git tool".into());
    }
    if patch_tool.is_some() != planned.artifacts.iter().any(|artifact| artifact.name == "casita") {
        return Err("Casita lock entry requires exactly one declared patch tool".into());
    }
    if planned.artifacts.is_empty() {
        return Err("a vendor plan needs at least one external artifact".into());
    }
    let mut units = Vec::new();
    let mut fetch_ids = Vec::with_capacity(planned.artifacts.len());
    for (index, artifact) in planned.artifacts.iter().enumerate() {
        let stem = fetch_stem(&artifact.name, &artifact.version);
        // The sorted position disambiguates distinct long names with the
        // same 48-byte stem and identical upstream artifact checksum.
        let id = format!("fetch.{index:04}.{stem}.{}", artifact.sha256);
        fetch_ids.push(id.clone());
        let url = artifact.fetch_url.as_str();
        if !url.starts_with("https://") {
            return Err(format!("no reviewed HTTPS fixed-output transport for {}", artifact.name));
        }
        let env = if let Some(rev) = artifact.git_rev.as_deref() {
            if rev.len() != 40 {
                return Err(format!("unsupported Git revision for {}", artifact.name));
            }
            vec![("url", url), ("type", "git"), ("rev", rev)]
        } else {
            vec![("url", url)]
        };
        let mode = if artifact.hash_mode == HashMode::FlatSha256 {
            "flat"
        } else {
            "recursive"
        };
        units.push(unit(
            &id,
            &format!("lock-fetch-{stem}-{}", &artifact.sha256[..16]),
            "builtin:fetchurl",
            "x86_64-linux",
            &[],
            &env,
            &[],
            Some((mode, &artifact.sha256)),
        ));
    }
    let mut shard_ids = Vec::new();
    if planned.artifacts.len() > SHARD_SIZE {
        for (n, group) in planned.artifacts.chunks(SHARD_SIZE).enumerate() {
            let id = format!("shard.{n:03}");
            let mut args = vec![
                "{{mantle-source:src.bundle}}/vendor-dynamic-assemble.py".to_string(),
                "shard".to_string(),
                "--lock".to_string(),
                "{{mantle-source:src.bundle}}/workspace/Cargo.lock".to_string(),
                "--table".to_string(),
                "{{mantle-source:src.table}}".to_string(),
                "--start".to_string(),
                (n * SHARD_SIZE).to_string(),
            ];
            let mut inputs = vec![
                source_ref("src.bundle"),
                source_ref("src.table"),
                source_ref("src.python"),
            ];
            for (offset, artifact) in group.iter().enumerate() {
                let fetch_id = &fetch_ids[n * SHARD_SIZE + offset];
                args.extend([
                    "--artifact".to_string(),
                    format!("{}@{}", artifact.name, artifact.version),
                    marker(fetch_id),
                ]);
                inputs.push(output_ref(fetch_id));
            }
            if inputs.len() > 256 {
                return Err("shard dependency budget exceeded".into());
            }
            units.push(unit(
                &id,
                &format!("lock-vendor-shard-{n:03}"),
                builder,
                "x86_64-linux",
                &args,
                &[],
                &inputs,
                None,
            ));
            shard_ids.push(id);
        }
    }
    if shard_ids.len() + 4 + runtime_inputs.len() > 256 {
        return Err("final assembly dependency budget exceeded".into());
    }
    let mut args = vec![
        "{{mantle-source:src.bundle}}/vendor-dynamic-assemble.py".to_string(),
        "final".to_string(),
        "--lock".to_string(),
        "{{mantle-source:src.bundle}}/workspace/Cargo.lock".to_string(),
        "--table".to_string(),
        "{{mantle-source:src.table}}".to_string(),
        "--bundle".to_string(),
        "{{mantle-source:src.bundle}}".to_string(),
        "--rust".to_string(),
        "{{mantle-source:src.rust}}".to_string(),
    ];
    if let Some(index) = git_tool {
        args.extend([
            "--git".to_string(),
            format!("{{{{mantle-source:src.runtime.{index}}}}}/bin/git"),
        ]);
    }
    if let Some(index) = patch_tool {
        args.extend([
            "--patch".to_string(),
            format!("{{{{mantle-source:src.runtime.{index}}}}}/bin/patch"),
        ]);
    }
    let mut inputs = vec![
        source_ref("src.bundle"),
        source_ref("src.table"),
        source_ref("src.python"),
        source_ref("src.rust"),
    ];
    inputs.extend(runtime_inputs.iter().cloned());
    if shard_ids.is_empty() {
        for (artifact, fetch_id) in planned.artifacts.iter().zip(fetch_ids.iter()) {
            args.extend([
                "--artifact".to_string(),
                format!("{}@{}", artifact.name, artifact.version),
                marker(fetch_id),
            ]);
            inputs.push(output_ref(fetch_id));
        }
    } else {
        for id in &shard_ids {
            args.extend(["--shard".to_string(), marker(id)]);
            inputs.push(output_ref(id));
        }
    }
    if inputs.len() > 256 {
        return Err("final assembly dependency budget exceeded".into());
    }
    units.push(unit("vendor.complete", "lock-vendor-complete", builder, "x86_64-linux", &args, &[], &inputs, None));
    if units.len() > 4096 {
        return Err("native unit budget exceeded".into());
    }
    let mut sources = vec![
        format!("{{\"id\":\"src.bundle\",\"path\":{},\"nar_blake3\":null}}", escape(bundle)),
        format!("{{\"id\":\"src.table\",\"path\":{},\"nar_blake3\":null}}", escape(table_path)),
        format!("{{\"id\":\"src.python\",\"path\":{},\"nar_blake3\":null}}", escape(python)),
        format!("{{\"id\":\"src.rust\",\"path\":{},\"nar_blake3\":null}}", escape(rust)),
    ];
    sources.extend(runtime_sources);
    let result = format!(
        "{{\"schema\":\"mantle-plan-v1\",\"producer\":{{\"logical_name\":\"cargo-lock-vendor\",\"goal_hint\":null}},\"sources\":[{}],\"units\":[{}],\"roots\":[\"vendor.complete\"],\"provenance\":{{}}}}",
        sources.join(","),
        units.join(",")
    );
    if result.len() > MAX_PLAN {
        return Err(format!("native dynamic plan exceeds {MAX_PLAN} bytes"));
    }
    Ok(result)
}
fn bounded_read(path: &Path, limit: u64) -> Result<Vec<u8>, String> {
    let file = fs::File::open(path).map_err(|e| e.to_string())?;
    let size = file.metadata().map_err(|e| e.to_string())?.len();
    if size > limit {
        return Err(format!("bounded input exceeds {limit} bytes: {}", path.display()));
    }
    let mut bytes = Vec::with_capacity(size as usize);
    file.take(limit + 1).read_to_end(&mut bytes).map_err(|e| e.to_string())?;
    if bytes.len() as u64 > limit {
        return Err(format!("bounded input grew beyond {limit} bytes: {}", path.display()));
    }
    Ok(bytes)
}
fn publish_new(output: &Path, bytes: &[u8]) -> io::Result<()> {
    if output.try_exists()? {
        return Err(io::Error::new(io::ErrorKind::AlreadyExists, "declared output already exists"));
    }
    let stage = output.with_extension("mantle-vendor-stage");
    let mut file = fs::OpenOptions::new().write(true).create_new(true).open(&stage)?;
    if let Err(error) = file.write_all(bytes).and_then(|()| file.sync_all()).and_then(|()| fs::rename(&stage, output)) {
        let _ = fs::remove_file(&stage);
        return Err(error);
    }
    Ok(())
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = env::args_os().collect();
    if args.get(1).is_some_and(|value| value == "--selected-table") {
        if args.len() != 4 {
            return Err("usage: producer --selected-table <Cargo.lock> <full-shared-table>".into());
        }
        let lock_bytes = bounded_read(Path::new(&args[2]), MAX_LOCK_BYTES as u64).map_err(io::Error::other)?;
        let table_bytes = bounded_read(Path::new(&args[3]), MAX_TABLE_BYTES as u64).map_err(io::Error::other)?;
        let shared = SharedHashTable::parse(&table_bytes)
            .map_err(|error| io::Error::other(format!("shared hashes: {error:?}")))?;
        let planned = plan_cargo_vendor_with_table(&lock_bytes, Limits::default(), Some(&shared))
            .map_err(|error| io::Error::other(format!("Cargo.lock: {error:?}")))?;
        let selected = planned
            .selected_shared_hashes(&shared)
            .map_err(|error| io::Error::other(format!("selected shared hashes: {error:?}")))?;
        let output = std::path::PathBuf::from(env::var_os("out").ok_or("selector requires declared out output")?);
        return publish_new(&output, selected.render().as_bytes()).map_err(Into::into);
    }
    if args.len() < 6 || args.len() > 6 + MAX_EXTRA_RUNTIME_SOURCES {
        return Err("usage: producer <Cargo.lock> <selected-shared-table> <python-store-executable> <bundle-store-path> <rust-store-path> [extra-runtime-store-path ...]".into());
    }
    let builder = args[3].to_str().ok_or("non-UTF-8 builder")?;
    let bundle = args[4].to_str().ok_or("non-UTF-8 source bundle")?;
    let rust = args[5].to_str().ok_or("non-UTF-8 Rust provider")?;
    let runtime = args[6..]
        .iter()
        .map(|path| path.to_str().ok_or("non-UTF-8 runtime path"))
        .collect::<Result<Vec<_>, _>>()?;
    let json =
        plan(Path::new(&args[1]), Path::new(&args[2]), builder, bundle, rust, &runtime).map_err(io::Error::other)?;
    let output = std::path::PathBuf::from(env::var_os("plan").ok_or("producer requires declared plan output")?);
    publish_new(&output, json.as_bytes()).map_err(Into::into)
}

#[cfg(test)]
mod tests {
    use serde_json::Value;

    use super::*;

    const BUNDLE: &str = "/nix/store/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa-vendor-bundle";
    const TABLE: &str = "/nix/store/bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb-selected-hashes";
    const PYTHON: &str = "/nix/store/cccccccccccccccccccccccccccccccc-python/bin/python3";
    const RUST: &str = "/nix/store/dddddddddddddddddddddddddddddddd-rust";
    const EMPTY_TABLE: &str = "mantle-shared-lock-hashes-v1\n";
    const REGISTRY: &str = "registry+https://github.com/rust-lang/crates.io-index";

    fn lock_for(names: impl Iterator<Item = String>) -> String {
        let mut lock = "version = 4\n".to_string();
        for (index, name) in names.enumerate() {
            let digit = format!("{:x}", index % 16);
            lock.push_str(&format!(
                "\n[[package]]\nname = \"{name}\"\nversion = \"1.0.0\"\nsource = \"{REGISTRY}\"\nchecksum = \"{}\"\n",
                digit.repeat(64)
            ));
        }
        lock
    }

    fn emit(lock: &str, table: &str) -> Result<Value, String> {
        let json = plan_from_bytes(
            &Path::new(BUNDLE).join("workspace/Cargo.lock"),
            Path::new(TABLE),
            PYTHON,
            BUNDLE,
            RUST,
            &[],
            lock.as_bytes(),
            table.as_bytes(),
        )?;
        serde_json::from_str(&json).map_err(|error| error.to_string())
    }

    #[test]
    fn emits_exact_locked_flat_fetches_and_single_assembly_root() {
        let lock = lock_for(["zeta".to_string(), "alpha".to_string()].into_iter());
        let value = emit(&lock, EMPTY_TABLE).unwrap();
        let units = value["units"].as_array().unwrap();
        assert_eq!(units.len(), 3);
        let fetches: Vec<_> = units.iter().filter(|unit| unit["id"].as_str().unwrap().starts_with("fetch.")).collect();
        assert_eq!(fetches.len(), 2);
        for (index, (name, hash)) in [("alpha", "1".repeat(64)), ("zeta", "0".repeat(64))].iter().enumerate() {
            let fetch = fetches[index];
            assert_eq!(fetch["derivation"]["builder"], "builtin:fetchurl");
            assert_eq!(fetch["derivation"]["fixed_output"]["hash"], hash.as_str());
            assert_eq!(fetch["derivation"]["fixed_output"]["mode"], "flat");
            assert_eq!(
                fetch["derivation"]["env"]["url"],
                format!("https://static.crates.io/crates/{name}/{name}-1.0.0.crate")
            );
            assert!(fetch["derivation"]["inputs"].as_array().unwrap().is_empty());
        }
        let assembler = &units[2];
        assert_eq!(assembler["id"], "vendor.complete");
        assert_eq!(assembler["derivation"]["fixed_output"], Value::Null);
        assert_eq!(value["roots"], serde_json::json!(["vendor.complete"]));
    }

    #[test]
    fn all_locked_artifacts_remain_reachable_across_sharding_and_name_collisions() {
        let cases = [
            (0..240).map(|index| format!("crate{index:04}")).collect::<Vec<_>>(),
            (0..241).map(|index| format!("crate{index:04}")).collect::<Vec<_>>(),
            vec![format!("{}a", "x".repeat(49)), format!("{}b", "x".repeat(49))],
        ];
        for names in cases {
            let lock = lock_for(names.clone().into_iter()).replace(&"1".repeat(64), &"0".repeat(64));
            let value = emit(&lock, EMPTY_TABLE).unwrap();
            let units: std::collections::BTreeMap<_, _> =
                value["units"].as_array().unwrap().iter().map(|unit| (unit["id"].as_str().unwrap(), unit)).collect();
            let mut pending =
                value["roots"].as_array().unwrap().iter().map(|id| id.as_str().unwrap()).collect::<Vec<_>>();
            let mut visited = BTreeSet::new();
            let mut fetched_urls = BTreeSet::new();
            while let Some(id) = pending.pop() {
                assert!(visited.insert(id), "an artifact must not be lost behind a cycle or repeated dependency");
                let unit = units.get(id).expect("every assembly edge must resolve to a declared unit");
                let inputs = unit["derivation"]["inputs"].as_array().unwrap();
                assert!(inputs.len() <= 256, "a generated unit must fit native admission's input budget");
                if unit["derivation"]["builder"] == "builtin:fetchurl" {
                    let url = unit["derivation"]["env"]["url"].as_str().unwrap().to_string();
                    assert!(fetched_urls.insert(url), "every locked artifact must be consumed exactly once");
                }
                for input in inputs {
                    if input["kind"] == "unit_output" {
                        assert_eq!(input["output"], "out");
                        pending.push(input["unit"].as_str().unwrap());
                    }
                }
            }
            let expected_urls = names
                .iter()
                .map(|name| format!("https://static.crates.io/crates/{name}/{name}-1.0.0.crate"))
                .collect::<BTreeSet<_>>();
            assert_eq!(fetched_urls, expected_urls, "sharding and colliding stems must preserve the complete lock");
        }
    }

    #[test]
    fn refuses_undeclared_lock_full_table_and_incompatible_hash_modes() {
        let lock = lock_for(["alpha".to_string()].into_iter());
        let wrong = Path::new("/nix/store/eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee-unrelated/Cargo.lock");
        assert!(
            plan_from_bytes(
                wrong,
                Path::new(TABLE),
                PYTHON,
                BUNDLE,
                RUST,
                &[],
                lock.as_bytes(),
                EMPTY_TABLE.as_bytes()
            )
            .unwrap_err()
            .contains("declared source bundle")
        );
        let extra = format!(
            "{EMPTY_TABLE}cargo/unused@1/git+https://example.test/repo#0123456789abcdef0123456789abcdef01234567\tflat-sha256\t{}\n",
            "a".repeat(64)
        );
        assert!(emit(&lock, &extra).unwrap_err().contains("canonical selected table"));
        let invalid_lock = lock.replace(&"0".repeat(64), "invalid");
        assert!(emit(&invalid_lock, EMPTY_TABLE).unwrap_err().contains("InvalidField"));
        let missing = lock.replace(&format!("checksum = \"{}\"\n", "0".repeat(64)), "");
        assert!(emit(&missing, EMPTY_TABLE).unwrap_err().contains("MissingHash"));
        let git_rev = "0123456789abcdef0123456789abcdef01234567";
        let git_source = format!("git+https://example.test/one.git#{git_rev}");
        let git_lock =
            format!("version = 4\n[[package]]\nname = \"dep\"\nversion = \"1.0.0\"\nsource = \"{git_source}\"\n");
        let git_table = format!("{EMPTY_TABLE}cargo/dep@1.0.0/{git_source}\tflat-sha256\t{}\n", "a".repeat(64));
        assert!(emit(&git_lock, &git_table).unwrap_err().contains("InvalidField"));
        assert!(
            plan_from_bytes(
                &Path::new(BUNDLE).join("workspace/Cargo.lock"),
                Path::new(TABLE),
                PYTHON,
                BUNDLE,
                RUST,
                &[],
                &vec![b' '; MAX_LOCK_BYTES as usize + 1],
                EMPTY_TABLE.as_bytes(),
            )
            .unwrap_err()
            .contains("byte bound")
        );
    }
}
