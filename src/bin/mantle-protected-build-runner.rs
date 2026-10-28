//! Protected build observer: existing execution policy, never proof authority.
#[cfg(target_os = "linux")]
mod linux {
    use std::error::Error;
    use std::ffi::OsString;
    use std::fs;
    use std::fs::File;
    use std::io::Read;
    use std::os::unix::ffi::OsStrExt;
    use std::os::unix::process::ExitStatusExt;
    use std::path::Path;
    use std::path::PathBuf;
    use std::process::Command;

    use clap::Parser;
    use mantle::protected_exec::ExecutableSeedEntry;
    use mantle::protected_exec::ProtectedExecPolicy;
    use mantle::protected_exec::Stage0Inventory;
    use mantle::protected_exec::blake3_file_hex;
    use mantle::protected_exec_seccomp::install_current_thread_exec_supervisor;

    const INVENTORY_BYTES_MAX: u64 = 1_048_576;

    #[derive(serde::Serialize)]
    struct Observation<'a> {
        schema: &'static str,
        inventory_digest_blake3: &'a str,
        source_inventory_digest_blake3: &'a str,
        materialized_aliases: &'a [MaterializedAlias],
        exit_code: Option<i32>,
        signal: Option<i32>,
        launch_error: Option<&'a str>,
        audit_error: Option<&'a str>,
        events: &'a [mantle::protected_exec::ProtectedSeccompAuditEvent],
        claim_class: &'static str,
        proof_evidence: bool,
        non_claims: [&'static str; 3],
    }

    #[derive(Debug, serde::Serialize)]
    struct MaterializedAlias {
        inventory_entry_id: String,
        source_path: PathBuf,
        materialized_path: PathBuf,
        digest_blake3: String,
    }

    struct PreparedPolicy {
        policy: ProtectedExecPolicy,
        source_digest: String,
        aliases: Vec<MaterializedAlias>,
    }

    #[derive(Parser)]
    struct Arguments {
        #[arg(long)]
        inventory: PathBuf,
        #[arg(long)]
        audit: PathBuf,
        #[arg(long, requires = "store_prefix")]
        materialized_input_root: Option<PathBuf>,
        #[arg(long, requires = "materialized_input_root")]
        store_prefix: Option<PathBuf>,
        #[arg(last = true, required = true)]
        command: Vec<OsString>,
    }

    fn policy(path: &Path, materialization: Option<(&Path, &Path)>) -> Result<PreparedPolicy, Box<dyn Error>> {
        let mut input = File::open(path)?.take(INVENTORY_BYTES_MAX.saturating_add(1));
        let inventory: Stage0Inventory = serde_json::from_reader(std::io::BufReader::new(&mut input))?;
        if input.limit() == 0 {
            return Err("protected build inventory exceeds byte bound".into());
        }
        let prepared = prepare_policy(inventory, materialization)?;
        assert!(input.limit() > 0);
        assert_eq!(prepared.source_digest.len(), 64);
        Ok(prepared)
    }

    fn require_clean_absolute_path(path: &Path) -> Result<(), Box<dyn Error>> {
        let bytes = path.as_os_str().as_bytes();
        if !path.is_absolute() {
            return Err(format!("unclean protected build path: {}", path.display()).into());
        }
        if bytes.contains(&0) {
            return Err(format!("unclean protected build path: {}", path.display()).into());
        }
        if bytes.split(|byte| *byte == b'/').skip(1).any(|component| matches!(component, b"" | b"." | b"..")) {
            return Err(format!("unclean protected build path: {}", path.display()).into());
        }
        assert!(path.is_absolute());
        assert!(!bytes.contains(&0));
        Ok(())
    }

    fn prepare_policy(
        mut inventory: Stage0Inventory,
        materialization: Option<(&Path, &Path)>,
    ) -> Result<PreparedPolicy, Box<dyn Error>> {
        for entry in &inventory.executable_entries {
            require_clean_absolute_path(&entry.executable_path)?;
        }
        let Some((input_root, store_prefix)) = materialization else {
            let policy = ProtectedExecPolicy::from_inventory(inventory)?;
            assert_eq!(policy.inventory_digest_blake3().len(), 64);
            assert!(policy.required_sandbox_entry().is_ok());
            return Ok(PreparedPolicy {
                source_digest: policy.inventory_digest_blake3().to_owned(),
                policy,
                aliases: Vec::new(),
            });
        };
        // Validate the declared authority before touching any materialized bytes.
        // This also bounds aliases to the policy's executable-entry limit (4096).
        let source_policy = ProtectedExecPolicy::from_inventory(inventory.clone())?;
        let source_digest = source_policy.inventory_digest_blake3().to_owned();
        drop(source_policy);
        require_clean_absolute_path(input_root)?;
        require_clean_absolute_path(store_prefix)?;
        let input_root = input_root.canonicalize()?;
        if !input_root.is_dir() {
            return Err("materialized input root must be a directory".into());
        }
        let source_entry_count = inventory.executable_entries.len();
        let mut aliases = Vec::with_capacity(source_entry_count);
        for index in 0..source_entry_count {
            let entry = &inventory.executable_entries[index];
            let Some(alias) = materialized_alias(entry, &input_root, store_prefix)? else {
                continue;
            };
            let mut materialized_entry = entry.clone();
            materialized_entry.executable_path = alias.materialized_path.clone();
            // An alias is not another required-role candidate. The original
            // declaration still authorizes host-bound launchers and shells.
            materialized_entry.required = false;
            inventory.executable_entries.push(materialized_entry);
            aliases.push(alias);
        }
        assert!(aliases.len() <= source_entry_count);
        // Original declarations and aliases share the existing 4096-entry bound.
        let policy = ProtectedExecPolicy::from_inventory(inventory)?;
        assert_eq!(source_digest.len(), 64);
        Ok(PreparedPolicy {
            policy,
            source_digest,
            aliases,
        })
    }

    fn materialized_alias(
        entry: &ExecutableSeedEntry,
        input_root: &Path,
        store_prefix: &Path,
    ) -> Result<Option<MaterializedAlias>, Box<dyn Error>> {
        let Ok(relative) = entry.executable_path.strip_prefix(store_prefix) else {
            return Ok(None);
        };
        if relative.as_os_str().is_empty() {
            return Err("declared executable cannot be the store prefix itself".into());
        }
        let candidate = input_root.join(relative);
        match fs::symlink_metadata(&candidate) {
            Ok(_) => {}
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
            Err(error) => return Err(error.into()),
        }
        let materialized_path = candidate.canonicalize()?;
        if !materialized_path.starts_with(input_root) {
            return Err(format!("materialized executable escapes input root: {}", candidate.display()).into());
        }
        if !fs::metadata(&materialized_path)?.is_file() {
            return Err(format!("materialized executable is not a file: {}", candidate.display()).into());
        }
        let digest_blake3 = blake3_file_hex(&materialized_path)?;
        if entry.digest.algorithm != "blake3" || digest_blake3 != entry.digest.hex {
            return Err(format!("materialized executable digest mismatch: {}", candidate.display()).into());
        }
        assert!(materialized_path.starts_with(input_root));
        assert_eq!(digest_blake3, entry.digest.hex);
        if materialized_path == entry.executable_path {
            return Ok(None);
        }
        Ok(Some(MaterializedAlias {
            inventory_entry_id: entry.id.clone(),
            source_path: entry.executable_path.clone(),
            materialized_path,
            digest_blake3,
        }))
    }

    pub fn run() -> Result<i32, Box<dyn Error>> {
        let arguments = Arguments::parse();
        let executable = arguments.command.first().ok_or("missing protected build command")?;
        let materialization = arguments.materialized_input_root.as_deref().zip(arguments.store_prefix.as_deref());
        let prepared = policy(&arguments.inventory, materialization)?;
        let inventory_digest = prepared.policy.inventory_digest_blake3().to_owned();
        // Claim an exclusive audit file before installing the irreversible filter.
        let audit_file = File::create_new(&arguments.audit)?;
        let supervisor = install_current_thread_exec_supervisor(prepared.policy)?;
        let status = Command::new(executable).args(&arguments.command[1..]).status();
        let quiescence = supervisor.wait_for_audit_quiescence();
        let events = supervisor.audit_events();
        let launch_error = status.as_ref().err().map(ToString::to_string);
        let exit_code = status.as_ref().ok().and_then(std::process::ExitStatus::code);
        let signal = status.as_ref().ok().and_then(|status| status.signal());
        let audit_error = quiescence.as_ref().err().map(ToString::to_string);
        serde_json::to_writer_pretty(audit_file, &Observation {
            schema: "mantle-protected-build-observation-v1",
            inventory_digest_blake3: &inventory_digest,
            source_inventory_digest_blake3: &prepared.source_digest,
            materialized_aliases: &prepared.aliases,
            exit_code,
            signal,
            launch_error: launch_error.as_deref(),
            audit_error: audit_error.as_deref(),
            events: &events,
            claim_class: "execution-only-performance-observation",
            proof_evidence: false,
            non_claims: [
                "not-bootstrap-acceptance",
                "not-rebuild-proof",
                "not-filesystem-isolation-proof",
            ],
        })?;
        quiescence?;
        assert!(!arguments.command.is_empty());
        assert_eq!(inventory_digest.len(), 64);
        Ok(exit_code.unwrap_or(1))
    }

    #[cfg(test)]
    mod tests {
        use std::os::unix::fs::symlink;

        use mantle::protected_exec::DigestSpec;
        use mantle::protected_exec::ExecRequest;
        use mantle::protected_exec::ExecutableSeedEntry;
        use mantle::protected_exec::bounded_version_evidence;

        use super::*;

        const STORE_PREFIX: &str = "/declared/store";
        const COMPILER_PATH: &str = "/declared/store/compiler/bin/cc";
        const COMPILER_BYTES: &[u8] = b"declared compiler bytes";

        fn entry(id: &str, role: &str, path: &str) -> ExecutableSeedEntry {
            ExecutableSeedEntry {
                schema_version: "host-tool-free-stage0-v1".to_owned(),
                id: id.to_owned(),
                role: role.to_owned(),
                phase: "protected".to_owned(),
                executable_path: PathBuf::from(path),
                digest: DigestSpec {
                    algorithm: "blake3".to_owned(),
                    hex: blake3::hash(COMPILER_BYTES).to_hex().to_string(),
                    interoperability_reason: None,
                },
                version_evidence: Some(bounded_version_evidence(
                    vec![path.to_owned(), "--version".to_owned()],
                    "declared fixture version\n".to_owned(),
                    0,
                )),
                provenance_category: "test-fixture".to_owned(),
                provenance: "explicit materialization regression fixture".to_owned(),
                allowed_reason: "exercise declared execution authority".to_owned(),
                owner: "mantle-compile-cache".to_owned(),
                required: role == "sandbox-entry" || role == "sandbox-shell",
            }
        }

        fn inventory() -> Stage0Inventory {
            Stage0Inventory {
                executable_entries: vec![
                    entry("sandbox", "sandbox-entry", "/host/bwrap"),
                    entry("shell", "sandbox-shell", "/declared/store/shell/bin/busybox"),
                    entry("compiler", "bootstrap-toolchain-tool", COMPILER_PATH),
                ],
                source_entries: Vec::new(),
            }
        }

        fn materialize(root: &Path, bytes: &[u8]) -> PathBuf {
            let candidate = root.join("compiler/bin/cc");
            fs::create_dir_all(candidate.parent().unwrap()).unwrap();
            fs::write(&candidate, bytes).unwrap();
            candidate.canonicalize().unwrap()
        }

        #[test]
        fn declared_host_shell_and_materialized_copy_keep_exact_authority() {
            let root = tempfile::tempdir().unwrap();
            let compiler = materialize(root.path(), COMPILER_BYTES);
            let shell = root.path().join("shell/bin/busybox");
            fs::create_dir_all(shell.parent().unwrap()).unwrap();
            fs::write(&shell, COMPILER_BYTES).unwrap();
            let original = inventory();
            let direct = prepare_policy(original.clone(), None).unwrap();
            let digest = original.executable_entries[1].digest.hex.clone();
            let shell_source = original.executable_entries[1].executable_path.clone();
            let prepared = prepare_policy(original, Some((root.path(), Path::new(STORE_PREFIX)))).unwrap();
            for (path, entry_id) in [
                (shell_source.clone(), "shell"),
                (shell, "shell"),
                (compiler.clone(), "compiler"),
            ] {
                let decision = prepared
                    .policy
                    .decide_exec(&ExecRequest {
                        path,
                        digest_hex: digest.clone(),
                    })
                    .unwrap();
                assert!(decision.allowed);
                assert_eq!(decision.entry_id.as_deref(), Some(entry_id));
            }
            assert_eq!(prepared.policy.required_sandbox_shell().unwrap().executable_path, shell_source);
            assert!(
                prepared
                    .policy
                    .decide_exec(&ExecRequest {
                        path: compiler,
                        digest_hex: blake3::hash(b"changed bytes").to_hex().to_string(),
                    })
                    .is_err()
            );
            assert!(
                prepared
                    .policy
                    .decide_exec(&ExecRequest {
                        path: root.path().join("undeclared-compiler"),
                        digest_hex: digest,
                    })
                    .is_err()
            );
            assert_eq!(prepared.source_digest, direct.source_digest);
            assert_ne!(prepared.policy.inventory_digest_blake3(), prepared.source_digest);
        }

        #[test]
        fn changed_bytes_escaped_links_and_parent_traversal_cannot_acquire_authority() {
            let root = tempfile::tempdir().unwrap();
            materialize(root.path(), b"changed compiler bytes");
            let error = prepare_policy(inventory(), Some((root.path(), Path::new(STORE_PREFIX))))
                .err()
                .expect("changed bytes must not acquire declared authority");
            assert!(error.to_string().contains("digest mismatch"));

            let linked = tempfile::tempdir().unwrap();
            let outside = tempfile::tempdir().unwrap();
            let outside_file = materialize(outside.path(), COMPILER_BYTES);
            let link = linked.path().join("compiler/bin/cc");
            fs::create_dir_all(link.parent().unwrap()).unwrap();
            symlink(outside_file, &link).unwrap();
            let error = prepare_policy(inventory(), Some((linked.path(), Path::new(STORE_PREFIX))))
                .err()
                .expect("even identical bytes must not escape the input root");
            assert!(error.to_string().contains("escapes input root"));

            let mut declared = inventory();
            declared.executable_entries[2].executable_path = PathBuf::from("/declared/store/../compiler/bin/cc");
            let error = prepare_policy(declared, Some((root.path(), Path::new(STORE_PREFIX))))
                .err()
                .expect("parent traversal must not be normalized into authority");
            assert!(error.to_string().contains("unclean protected build path"));
        }

        #[test]
        fn inventory_exact_byte_bound_is_preserved() {
            let root = tempfile::tempdir().unwrap();
            let inventory_path = root.path().join("inventory.json");
            let mut bytes = serde_json::to_vec(&inventory()).unwrap();
            bytes.resize(usize::try_from(INVENTORY_BYTES_MAX).unwrap(), b' ');
            fs::write(&inventory_path, &bytes).unwrap();
            let admitted = policy(&inventory_path, None).unwrap();
            assert!(
                admitted
                    .policy
                    .decide_exec(&ExecRequest {
                        path: PathBuf::from(COMPILER_PATH),
                        digest_hex: blake3::hash(COMPILER_BYTES).to_hex().to_string(),
                    })
                    .unwrap()
                    .allowed
            );
            bytes.push(b' ');
            fs::write(&inventory_path, bytes).unwrap();
            let error = policy(&inventory_path, None).err().expect("one byte over the inventory bound must fail");
            assert!(error.to_string().contains("exceeds byte bound"));
        }
    }
}

#[cfg(target_os = "linux")]
fn main() {
    match linux::run() {
        Ok(status) => std::process::exit(status),
        Err(error) => {
            eprintln!("mantle-protected-build-runner: {error}");
            std::process::exit(1);
        }
    }
}

#[cfg(not(target_os = "linux"))]
fn main() {
    eprintln!("mantle-protected-build-runner requires the Linux seccomp supervisor");
    std::process::exit(1);
}
