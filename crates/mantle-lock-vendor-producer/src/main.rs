//! Standalone declared Cargo.lock vendor selector and native dynamic-plan producer.
//! Usage: producer --selected-table <Cargo.lock> <full-shared-table>
//! Or: producer <Cargo.lock> <selected-shared-table> <python-store-executable> <bundle-store-path> <rust-store-path> [extra-runtime-store-path ...]
use mantle_lock_vendor_core::{plan_cargo_vendor_with_table, shared_table::{HashMode, SharedHashTable, MAX_TABLE_BYTES}, Limits, MAX_LOCK_BYTES};
use std::{collections::BTreeSet, env, fs, io::{self, Read, Write}, path::Path};

const MAX_PLAN: usize = 4 * 1024 * 1024;
const SHARD_SIZE: usize = 240;
const MAX_EXTRA_RUNTIME_SOURCES: usize = 12;
fn escape(value: &str) -> String {
    let mut out = String::from("\"");
    for c in value.chars() {
        match c {
            '"' => out.push_str("\\\""), '\\' => out.push_str("\\\\"), '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"), '\t' => out.push_str("\\t"),
            c if c <= '\u{1f}' => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}
fn store_path(value: &str, executable: bool) -> Result<(), String> {
    let mut components = value.split('/');
    if components.next() != Some("") || components.next().is_none_or(str::is_empty) || components.next() != Some("store") {
        return Err(format!("not an absolute store path: {value}"));
    }
    let name = components.next().ok_or("missing store basename")?;
    let suffix: Vec<_> = components.collect();
    if (executable && (suffix.is_empty() || suffix.iter().any(|s| s.is_empty() || *s == "." || *s == "..")))
        || (!executable && !suffix.is_empty())
        || name.len() < 34 || name.as_bytes()[32] != b'-'
        || !name[..32].bytes().all(|b| b.is_ascii_digit() || (b'a'..=b'z').contains(&b)) {
        return Err(format!("not a declared store path: {value}"));
    }
    Ok(())
}
fn unit(id: &str, name: &str, builder: &str, system: &str, args: &[String], env_vars: &[(&str, &str)],
    inputs: &[String], fixed: Option<(&str, &str)>) -> String {
    let arr = |values: &[String]| values.iter().map(|s| escape(s)).collect::<Vec<_>>().join(",");
    let vars = env_vars.iter().map(|(k,v)| format!("{}:{}", escape(k), escape(v))).collect::<Vec<_>>().join(",");
    let fixed = fixed.map_or("null".to_string(), |(mode, hash)| format!("{{\"mode\":{},\"algo\":\"sha256\",\"hash\":{}}}",escape(mode),escape(hash)));
    format!("{{\"id\":{},\"derivation\":{{\"name\":{},\"builder\":{},\"system\":{},\"args\":[{}],\"outputs\":[\"out\"],\"env\":{{{}}},\"inputs\":[{}],\"fixed_output\":{},\"addressing_mode\":\"input-addressed\",\"sandbox\":\"native\",\"dynamic_plan_outputs\":[]}},\"requested_outputs\":[\"out\"],\"policy\":{{\"sandbox\":\"inherit\",\"substitutions\":\"inherit\",\"store_prefix\":\"inherit\",\"host_paths\":\"none\"}}}}",escape(id),escape(name),escape(builder),escape(system),arr(args),vars,inputs.join(","),fixed)
}
fn output_ref(id: &str) -> String { format!("{{\"kind\":\"unit_output\",\"unit\":{},\"output\":\"out\"}}",escape(id)) }
fn source_ref(id: &str) -> String { format!("{{\"kind\":\"source\",\"source\":{}}}", escape(id)) }
fn marker(id: &str) -> String { format!("{{{{mantle-unit-output:{id}:out}}}}") }
/// Stable bounded derivation-name stem; the full fixed SHA disambiguates
/// packages whose names and versions share the displayed 48-byte prefix.
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
fn plan(lock: &Path, table: &Path, builder: &str, bundle: &str, rust: &str, runtime: &[&str]) -> Result<String, String> {
    store_path(builder, true)?;
    store_path(bundle, false)?;
    store_path(rust, false)?;
    let table_path = table.to_str().ok_or("non-UTF-8 selected table path")?;
    store_path(table_path, false)?;
    if runtime.len() > MAX_EXTRA_RUNTIME_SOURCES { return Err("extra runtime source budget exceeded".into()); }
    let python = builder.split("/bin/").next().ok_or("missing Python store root")?;
    let mut declared = BTreeSet::from([bundle, table_path, python, rust]);
    let mut runtime_sources = Vec::with_capacity(runtime.len());
    let mut runtime_inputs = Vec::with_capacity(runtime.len());
    for (index, path) in runtime.iter().enumerate() {
        store_path(path, false)?;
        if !declared.insert(path) { return Err(format!("duplicate runtime source: {path}")); }
        let id = format!("src.runtime.{index}");
        runtime_sources.push(format!("{{\"id\":{},\"path\":{},\"nar_blake3\":null}}", escape(&id), escape(path)));
        runtime_inputs.push(source_ref(&id));
    }
    let git_tool = runtime.iter().enumerate().find(|(_, root)| Path::new(root).join("bin/git").is_file()).map(|(index, _)| index);
    let patch_tool = runtime.iter().enumerate().find(|(_, root)| Path::new(root).join("bin/patch").is_file()).map(|(index, _)| index);
    if runtime.iter().filter(|root| Path::new(root).join("bin/git").is_file()).count() > 1
        || runtime.iter().filter(|root| Path::new(root).join("bin/patch").is_file()).count() > 1 {
        return Err("ambiguous declared Git or patch tool".into());
    }
    let lock_bytes = bounded_read(lock, MAX_LOCK_BYTES as u64)?;
    let table_bytes = bounded_read(table, MAX_TABLE_BYTES as u64)?;
    let shared = SharedHashTable::parse(&table_bytes).map_err(|e| format!("shared hashes: {e:?}"))?;
    let planned = plan_cargo_vendor_with_table(&lock_bytes, Limits::default(), Some(&shared)).map_err(|e| format!("Cargo.lock: {e:?}"))?;
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
    if planned.artifacts.is_empty() { return Err("a vendor plan needs at least one external artifact".into()); }
    let mut units = Vec::new();
    let mut fetch_ids = Vec::with_capacity(planned.artifacts.len());
    let mut seen_fetch_ids = BTreeSet::new();
    for artifact in &planned.artifacts {
        let stem = fetch_stem(&artifact.name, &artifact.version);
        let id = format!("fetch.{stem}.{}", artifact.sha256);
        if !seen_fetch_ids.insert(id.clone()) {
            return Err(format!("two lock dependencies have the same stable fetch identity: {id}"));
        }
        fetch_ids.push(id.clone());
        let url = artifact.fetch_url.as_str();
        if !url.starts_with("https://") { return Err(format!("no reviewed HTTPS fixed-output transport for {}", artifact.name)); }
        let env = if let Some(rev) = artifact.git_rev.as_deref() {
            if rev.len() != 40 { return Err(format!("unsupported Git revision for {}", artifact.name)); }
            vec![("url", url), ("type", "git"), ("rev", rev)]
        } else { vec![("url", url)] };
        let mode = if artifact.hash_mode == HashMode::FlatSha256 { "flat" } else { "recursive" };
        units.push(unit(&id, &format!("lock-fetch-{stem}-{}", &artifact.sha256[..16]), "builtin:fetchurl", "x86_64-linux", &[], &env, &[], Some((mode, &artifact.sha256))));
    }
    let mut shard_ids = Vec::new();
    if planned.artifacts.len() > SHARD_SIZE {
        for (n, group) in planned.artifacts.chunks(SHARD_SIZE).enumerate() {
            let id = format!("shard.{n:03}");
            let mut args = vec!["{{mantle-source:src.bundle}}/vendor-dynamic-assemble.py".to_string(), "shard".to_string(),
                "--lock".to_string(), "{{mantle-source:src.bundle}}/workspace/Cargo.lock".to_string(),
                "--table".to_string(), "{{mantle-source:src.table}}".to_string(),
                "--start".to_string(), (n * SHARD_SIZE).to_string()];
            let mut inputs = vec![source_ref("src.bundle"), source_ref("src.table"), source_ref("src.python")];
            for (offset, artifact) in group.iter().enumerate() {
                let fetch_id = &fetch_ids[n * SHARD_SIZE + offset];
                args.extend(["--artifact".to_string(), format!("{}@{}", artifact.name, artifact.version), marker(fetch_id)]);
                inputs.push(output_ref(fetch_id));
            }
            if inputs.len() > 256 { return Err("shard dependency budget exceeded".into()); }
            units.push(unit(&id, &format!("lock-vendor-shard-{n:03}"), builder, "x86_64-linux", &args, &[], &inputs, None));
            shard_ids.push(id);
        }
    }
    if shard_ids.len() + 4 + runtime_inputs.len() > 256 { return Err("final assembly dependency budget exceeded".into()); }
    let mut args = vec!["{{mantle-source:src.bundle}}/vendor-dynamic-assemble.py".to_string(), "final".to_string(),
        "--lock".to_string(), "{{mantle-source:src.bundle}}/workspace/Cargo.lock".to_string(),
        "--table".to_string(), "{{mantle-source:src.table}}".to_string(),
        "--bundle".to_string(), "{{mantle-source:src.bundle}}".to_string(),
        "--rust".to_string(), "{{mantle-source:src.rust}}".to_string()];
    if let Some(index) = git_tool {
        args.extend(["--git".to_string(), format!("{{{{mantle-source:src.runtime.{index}}}}}/bin/git")]);
    }
    if let Some(index) = patch_tool {
        args.extend(["--patch".to_string(), format!("{{{{mantle-source:src.runtime.{index}}}}}/bin/patch")]);
    }
    let mut inputs = vec![source_ref("src.bundle"), source_ref("src.table"), source_ref("src.python"), source_ref("src.rust")];
    inputs.extend(runtime_inputs.iter().cloned());
    if shard_ids.is_empty() {
        for (artifact, fetch_id) in planned.artifacts.iter().zip(fetch_ids.iter()) {
            args.extend(["--artifact".to_string(), format!("{}@{}", artifact.name, artifact.version), marker(fetch_id)]);
            inputs.push(output_ref(fetch_id));
        }
    } else {
        for id in &shard_ids { args.extend(["--shard".to_string(), marker(id)]); inputs.push(output_ref(id)); }
    }
    if inputs.len() > 256 { return Err("final assembly dependency budget exceeded".into()); }
    units.push(unit("vendor.complete", "lock-vendor-complete", builder, "x86_64-linux", &args, &[], &inputs, None));
    if units.len() > 4096 { return Err("native unit budget exceeded".into()); }
    let mut sources = vec![
        format!("{{\"id\":\"src.bundle\",\"path\":{},\"nar_blake3\":null}}", escape(bundle)),
        format!("{{\"id\":\"src.table\",\"path\":{},\"nar_blake3\":null}}", escape(table_path)),
        format!("{{\"id\":\"src.python\",\"path\":{},\"nar_blake3\":null}}", escape(python)),
        format!("{{\"id\":\"src.rust\",\"path\":{},\"nar_blake3\":null}}", escape(rust)),
    ];
    sources.extend(runtime_sources);
    let result = format!("{{\"schema\":\"mantle-plan-v1\",\"producer\":{{\"logical_name\":\"cargo-lock-vendor\",\"goal_hint\":null}},\"sources\":[{}],\"units\":[{}],\"roots\":[\"vendor.complete\"],\"provenance\":{{}}}}",sources.join(","),units.join(","));
    if result.len() > MAX_PLAN { return Err(format!("native dynamic plan exceeds {MAX_PLAN} bytes")); }
    Ok(result)
}
fn bounded_read(path: &Path, limit: u64) -> Result<Vec<u8>, String> {
    let file = fs::File::open(path).map_err(|e| e.to_string())?;
    let size = file.metadata().map_err(|e| e.to_string())?.len();
    if size > limit { return Err(format!("bounded input exceeds {limit} bytes: {}", path.display())); }
    let mut bytes = Vec::with_capacity(size as usize);
    file.take(limit + 1).read_to_end(&mut bytes).map_err(|e| e.to_string())?;
    if bytes.len() as u64 > limit { return Err(format!("bounded input grew beyond {limit} bytes: {}", path.display())); }
    Ok(bytes)
}
fn publish_new(output: &Path, bytes: &[u8]) -> io::Result<()> {
    if output.try_exists()? { return Err(io::Error::new(io::ErrorKind::AlreadyExists, "declared output already exists")); }
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
        if args.len() != 4 { return Err("usage: producer --selected-table <Cargo.lock> <full-shared-table>".into()); }
        let lock_bytes = bounded_read(Path::new(&args[2]), MAX_LOCK_BYTES as u64).map_err(io::Error::other)?;
        let table_bytes = bounded_read(Path::new(&args[3]), MAX_TABLE_BYTES as u64).map_err(io::Error::other)?;
        let shared = SharedHashTable::parse(&table_bytes).map_err(|error| io::Error::other(format!("shared hashes: {error:?}")))?;
        let planned = plan_cargo_vendor_with_table(&lock_bytes, Limits::default(), Some(&shared)).map_err(|error| io::Error::other(format!("Cargo.lock: {error:?}")))?;
        let selected = planned.selected_shared_hashes(&shared).map_err(|error| io::Error::other(format!("selected shared hashes: {error:?}")))?;
        let output = std::path::PathBuf::from(env::var_os("out").ok_or("selector requires declared out output")?);
        return publish_new(&output, selected.render().as_bytes()).map_err(Into::into);
    }
    if args.len() < 6 || args.len() > 6 + MAX_EXTRA_RUNTIME_SOURCES { return Err("usage: producer <Cargo.lock> <selected-shared-table> <python-store-executable> <bundle-store-path> <rust-store-path> [extra-runtime-store-path ...]".into()); }
    let builder = args[3].to_str().ok_or("non-UTF-8 builder")?;
    let bundle = args[4].to_str().ok_or("non-UTF-8 source bundle")?;
    let rust = args[5].to_str().ok_or("non-UTF-8 Rust provider")?;
    let runtime = args[6..].iter().map(|path| path.to_str().ok_or("non-UTF-8 runtime path")).collect::<Result<Vec<_>, _>>()?;
    let json = plan(Path::new(&args[1]), Path::new(&args[2]), builder, bundle, rust, &runtime).map_err(io::Error::other)?;
    let output = std::path::PathBuf::from(env::var_os("plan").ok_or("producer requires declared plan output")?);
    publish_new(&output, json.as_bytes()).map_err(Into::into)
}
