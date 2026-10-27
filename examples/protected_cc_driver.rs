//! Native C/C++ fixture runner: existing protected-exec policy, never proof authority.
#[cfg(target_os = "linux")]
mod linux {
    use std::error::Error;
    use std::ffi::OsString;
    use std::fs::File;
    use std::io::Read;
    use std::os::unix::process::ExitStatusExt;
    use std::path::PathBuf;
    use std::process::Command;

    use clap::Parser;
    use mantle::protected_exec::ProtectedExecPolicy;
    use mantle::protected_exec::Stage0Inventory;
    use mantle::protected_exec_seccomp::install_current_thread_exec_supervisor;

    const INVENTORY_BYTES_MAX: u64 = 1_048_576;

    #[derive(serde::Serialize)]
    struct Observation<'a> {
        schema: &'static str,
        inventory_digest_blake3: &'a str,
        exit_code: Option<i32>,
        signal: Option<i32>,
        launch_error: Option<&'a str>,
        audit_error: Option<&'a str>,
        events: &'a [mantle::protected_exec::ProtectedSeccompAuditEvent],
        claim_class: &'static str,
        proof_evidence: bool,
        non_claims: [&'static str; 3],
    }

    #[derive(Parser)]
    struct Arguments {
        #[arg(long)]
        inventory: PathBuf,
        #[arg(long)]
        audit: PathBuf,
        #[arg(last = true, required = true)]
        command: Vec<OsString>,
    }

    fn policy(path: &std::path::Path) -> Result<ProtectedExecPolicy, Box<dyn Error>> {
        let mut input = File::open(path)?.take(INVENTORY_BYTES_MAX.saturating_add(1));
        let inventory: Stage0Inventory = serde_json::from_reader(std::io::BufReader::new(&mut input))?;
        if input.limit() == 0 {
            return Err("protected compiler inventory exceeds byte bound".into());
        }
        let policy = ProtectedExecPolicy::from_inventory(inventory)?;
        assert!(input.limit() > 0);
        assert_eq!(policy.inventory_digest_blake3().len(), 64);
        Ok(policy)
    }

    pub fn run() -> Result<i32, Box<dyn Error>> {
        let arguments = Arguments::parse();
        let executable = arguments.command.first().ok_or("missing compiler-driver command")?;
        let policy = policy(&arguments.inventory)?;
        let inventory_digest = policy.inventory_digest_blake3().to_owned();
        // Claim an exclusive audit file before installing the irreversible filter.
        let audit_file = File::create_new(&arguments.audit)?;
        let supervisor = install_current_thread_exec_supervisor(policy)?;
        let status = Command::new(executable).args(&arguments.command[1..]).status();
        let quiescence = supervisor.wait_for_audit_quiescence();
        let events = supervisor.audit_events();
        let launch_error = status.as_ref().err().map(ToString::to_string);
        let exit_code = status.as_ref().ok().and_then(std::process::ExitStatus::code);
        let signal = status.as_ref().ok().and_then(|status| status.signal());
        let audit_error = quiescence.as_ref().err().map(ToString::to_string);
        serde_json::to_writer_pretty(audit_file, &Observation {
            schema: "mantle-protected-cc-driver-observation-v1",
            inventory_digest_blake3: &inventory_digest,
            exit_code,
            signal,
            launch_error: launch_error.as_deref(),
            audit_error: audit_error.as_deref(),
            events: &events,
            claim_class: "bounded-native-fixture",
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
}

#[cfg(target_os = "linux")]
fn main() {
    match linux::run() {
        Ok(status) => std::process::exit(status),
        Err(error) => {
            eprintln!("protected-cc-driver: {error}");
            std::process::exit(1);
        }
    }
}

#[cfg(not(target_os = "linux"))]
fn main() {
    eprintln!("protected-cc-driver requires the Linux seccomp supervisor");
    std::process::exit(1);
}
