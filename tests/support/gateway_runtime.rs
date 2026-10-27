#![allow(dead_code)]
//! Real private gateway fixture: normal signed store inputs, issued tickets,
//! exact UCAN holder invocations and the production CLI transport. Never grants
//! a session, mutates a coordinator job, or fabricates a worker observation.
use std::collections::HashMap;
use std::fs;
use std::io::Read;
use std::io::Seek;
use std::io::Write;
use std::os::fd::AsRawFd;
use std::os::unix::fs::OpenOptionsExt;
use std::os::unix::fs::PermissionsExt;
use std::os::unix::process::CommandExt;
use std::path::Path;
use std::path::PathBuf;
use std::process::Child;
use std::process::Command;
use std::process::Output;
use std::time::Duration;
use std::time::Instant;
use std::time::SystemTime;
use std::time::UNIX_EPOCH;

use basalt::ucan::TokenSigner as _;
use basalt::ucan::{self};
use serde_json::Value;
use serde_json::json;
use snix_store::nar::NarCalculationService as _;
use tokio::io::AsyncWriteExt as _;

#[path = "remote_endpoint_binding.rs"]
mod remote_endpoint_binding;

pub const ENDPOINT: &str = "gateway-runtime-worker";
pub const PROJECT: &str = "runtime-project";
pub const ACCOUNT: &str = "runtime-account";
pub const FIRST_MEMORY: u64 = 64 << 20;
pub const NEXT_MEMORY: u64 = 512 << 20;
pub const SCRATCH: u64 = 256 << 20;

pub fn private_dir(path: &Path) {
    fs::create_dir_all(path).unwrap();
    fs::set_permissions(path, fs::Permissions::from_mode(0o700)).unwrap();
}
pub fn private_bytes(path: &Path, bytes: &[u8]) {
    let mut file = fs::OpenOptions::new().write(true).create_new(true).mode(0o600).open(path).unwrap();
    file.write_all(bytes).unwrap();
    file.sync_all().unwrap();
}
pub fn private_json(path: &Path, value: &Value) {
    private_bytes(path, &serde_json::to_vec_pretty(value).unwrap());
}
pub fn read_json(path: &Path) -> Value {
    serde_json::from_slice(&fs::read(path).unwrap()).unwrap()
}
fn seed() -> [u8; 32] {
    let mut bytes = [0; 32];
    fs::File::open("/dev/urandom").unwrap().read_exact(&mut bytes).unwrap();
    bytes
}
pub fn vector(memory: u64) -> Value {
    json!({"cpu_units":1,"memory_bytes":memory,"scratch_bytes":SCRATCH,"accelerators":[],"named_tokens":[]})
}
pub fn requirements(memory: u64) -> Value {
    json!({"quantities":vector(memory),"semantic_accelerator_classes":[]})
}
pub fn now() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs()
}

pub struct GatewayRuntime {
    pub root: PathBuf,
    pub state: PathBuf,
    pub store: PathBuf,
    pub socket: PathBuf,
    pub config: Value,
    pub drv_path: String,
    pub output_path: String,
    pub source: PathBuf,
    pub live_socket: Option<PathBuf>,
    pub binary: PathBuf,
    pub credentials: PathBuf,
    pub config_dir: PathBuf,
    issuer: ucan::Ed25519InMemorySigner,
    holder: ucan::Ed25519InMemorySigner,
    started: u64,
    sequence: u64,
    _socket_directory: tempfile::TempDir,
}

impl GatewayRuntime {
    /// `root` must be a new retained directory. `paths` is a signed input-cache
    /// descriptor, not ambient trust: its full key is explicitly configured and
    /// the normal cache importer verifies NAR bytes/signatures.
    pub fn prepare(root: &Path, paths: &Path, cgroup: Option<&Path>, workload: &str) -> Self {
        assert!(!root.exists(), "use a fresh retained runtime root");
        private_dir(root);
        let started = now();
        let issuer_seed = seed();
        let holder_seed = seed();
        let socket_directory = tempfile::Builder::new().prefix("m-gw-").tempdir_in("/tmp").unwrap();
        fs::set_permissions(socket_directory.path(), fs::Permissions::from_mode(0o700)).unwrap();
        let mut fixture = Self {
            root: root.into(),
            state: root.join("state"),
            store: root.join("store"),
            socket: socket_directory.path().join("gateway.sock"),
            config: Value::Null,
            drv_path: String::new(),
            output_path: String::new(),
            binary: PathBuf::from(env!("CARGO_BIN_EXE_mantle")),
            source: root.join("derivation.ncl"),
            live_socket: None,
            credentials: root.join("private/credentials"),
            config_dir: root.join("private/config"),
            issuer: ucan::Ed25519InMemorySigner::from_seed_bytes(issuer_seed),
            holder: ucan::Ed25519InMemorySigner::from_seed_bytes(holder_seed),
            started,
            sequence: 0,
            _socket_directory: socket_directory,
        };
        for name in [
            "state",
            "store",
            "private",
            "private/credentials",
            "private/config",
            "replay",
            "journal",
            "quota",
            "evidence",
            "calls",
        ] {
            private_dir(&root.join(name));
        }
        let (keys, secret) = crunch_build::signing::generate_keypair();
        fixture.provision_credentials(&secret);
        fixture.provision_authority(
            ed25519_dalek::SigningKey::from_bytes(&issuer_seed).verifying_key().to_bytes(),
            ed25519_dalek::SigningKey::from_bytes(&holder_seed).verifying_key().to_bytes(),
        );
        let inputs = read_json(paths);
        let runtime = tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap();
        let (drv, output) = runtime.block_on(fixture.signed_derivation(&inputs, &keys, workload));
        fixture.drv_path = drv;
        fixture.output_path = output;
        private_bytes(
            &fixture.config_dir.join("trusted-public-keys"),
            format!("{}\n{}\n", keys.verifying_key, inputs["trusted_public_key"].as_str().unwrap()).as_bytes(),
        );
        fixture.provision_config(cgroup, &keys.verifying_key.to_string());
        fixture
    }

    fn provision_credentials(&self, signing_key: &str) {
        use base64::Engine as _;
        let verifier =
            format!("gateway-runtime-ticket:{}", base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(seed()));
        private_bytes(&self.credentials.join("TICKET_VERIFIER_KEY"), verifier.as_bytes());
        private_bytes(&self.credentials.join("RESULT_SIGNING_KEY"), signing_key.as_bytes());
        private_bytes(&self.root.join("private/cursor-key"), &seed());
        private_bytes(&self.root.join("private/completion-key"), &seed());
    }

    fn provision_authority(&self, issuer_key: [u8; 32], holder_key: [u8; 32]) {
        let issuer = self.issuer.issuer().unwrap();
        private_json(
            &self.root.join("private/authority.json"),
            &json!({
                "version":1,"revision":1,"generation":1,"issued_unix_s":self.started-1,
                "expires_unix_s":self.started+1800,"endpoint":ENDPOINT,"audience":"build-service",
                "policy_json":basalt::DEFAULT_POLICY_JSON,
                "policy_blake3":blake3::hash(basalt::DEFAULT_POLICY_JSON.as_bytes()).to_hex().to_string(),
                "trusted_roots":[issuer.as_str()],"keys":[issuer_key,holder_key],"proofs":[],"revoked":[]
            }),
        );
    }

    pub fn command(&self) -> Command {
        let mut command = Command::new(&self.binary);
        command
            .args(["--json", "--store-prefix", "/nix/store", "--state-dir"])
            .arg(&self.state)
            .arg("--store")
            .arg(&self.store)
            .env("CREDENTIALS_DIRECTORY", &self.credentials)
            .env("CRUNCH_CONFIG_DIR", &self.config_dir)
            .env("CRUNCH_NO_FUSE", "1")
            .env("TMPDIR", &self.root);
        if let Some(socket) = &self.live_socket {
            command.env("MANTLE_LIVE_STATE_SOCKET", socket);
        } else {
            command.env_remove("MANTLE_LIVE_STATE_SOCKET");
        }
        command
    }

    pub fn persist_config(&self) {
        let path = self.root.join("private/gateway.json");
        if path.exists() {
            fs::remove_file(&path).unwrap();
        }
        private_json(&path, &self.config);
    }

    pub fn issue_ticket(&self) -> fs::File {
        let mut delivery = tempfile::tempfile_in(self.root.join("private")).unwrap();
        let mut command = self.command();
        command
            .args([
                "remote",
                "ticket",
                "create",
                "--display-name",
                "scoped-gateway-runtime",
                "--ttl-secs",
                "1800",
                "--uses",
                "16",
                "--max-build-time-secs",
                "120",
                "--max-upload-bytes",
                "67108864",
                "--ticket-fd",
                "9",
                "--secret-manifest",
            ])
            .arg(Path::new(env!("CARGO_MANIFEST_DIR")).join("secretspec.toml"))
            .args([
                "--secret-profile",
                "production",
                "--secret-provider",
                "systemd-credential://",
            ]);
        inherit_fd(&mut command, delivery.as_raw_fd());
        let output = command.output().unwrap();
        retain_output(&self.root.join("ticket-issuance"), &output);
        assert!(output.status.success(), "ticket issuer failed; see retained stderr");
        delivery.rewind().unwrap();
        delivery
    }

    pub fn start(&self) -> RunningGateway {
        private_json(&self.root.join("source-identities-start.json"), &self.source_identity());
        self.persist_config();
        let provision = self
            .command()
            .args(["remote", "gateway", "provision-quota", "--directory"])
            .arg(self.root.join("quota"))
            .output()
            .unwrap();
        retain_output(&self.root.join("quota-provision"), &provision);
        assert!(provision.status.success(), "quota provision failed");
        let delivery = self.issue_ticket();
        let mut command = self.command();
        command
            .args(["remote", "gateway", "serve", "--config"])
            .arg(self.root.join("private/gateway.json"))
            .arg("--socket")
            .arg(&self.socket)
            .args(["--ticket-fd", "9"])
            .stdout(fs::File::create(self.root.join("gateway.stdout")).unwrap())
            .stderr(fs::File::create(self.root.join("gateway.stderr")).unwrap());
        command.process_group(0);
        inherit_fd(&mut command, delivery.as_raw_fd());
        let mut gateway = RunningGateway(command.spawn().unwrap());
        let deadline = Instant::now() + Duration::from_secs(20);
        while !self.socket.exists() {
            assert!(gateway.0.try_wait().unwrap().is_none(), "gateway exited: see retained stderr");
            assert!(Instant::now() < deadline, "gateway startup deadline exceeded");
            std::thread::sleep(Duration::from_millis(25));
        }
        gateway
    }

    pub fn call(&mut self, request: Value, project: &str) -> (bool, Value) {
        self.call_bound(request.clone(), request, project)
    }

    pub fn call_bound(&mut self, signed_request: Value, request: Value, project: &str) -> (bool, Value) {
        self.sequence += 1;
        let prefix = self.root.join("calls").join(format!("{:04}", self.sequence));
        let request_file = prefix.with_extension("request.json");
        private_json(&request_file, &signed_request);
        let binding = self
            .command()
            .args(["remote", "gateway", "binding", "--request"])
            .arg(&request_file)
            .args(["--project", project, "--endpoint-generation", "1"])
            .output()
            .unwrap();
        retain_output(&prefix.with_extension("binding"), &binding);
        assert!(binding.status.success(), "binding command failed");
        let binding: Value = serde_json::from_slice(&binding.stdout).unwrap();
        let envelope = self.envelope(&binding, request, project);
        let path = self.root.join("private").join(format!("invocation-{}.json", self.sequence));
        private_json(&path, &envelope);
        let response = self
            .command()
            .args(["remote", "gateway", "call", "--socket"])
            .arg(&self.socket)
            .arg("--envelope")
            .arg(&path)
            .output()
            .unwrap();
        retain_output(&prefix.with_extension("response"), &response);
        let bytes = if response.stdout.is_empty() {
            &response.stderr
        } else {
            &response.stdout
        };
        let value = serde_json::from_slice(bytes).expect("real CLI omitted a JSON outcome");
        (response.status.success(), value)
    }

    fn envelope(&self, binding: &Value, request: Value, project: &str) -> Value {
        let operation: basalt::mantle::Operation = serde_json::from_value(binding["operation"].clone()).unwrap();
        let digest: [u8; 32] = serde_json::from_value(binding["request_digest"].clone()).unwrap();
        let authority = basalt::mantle::Request {
            operation,
            subject: self.holder.issuer().unwrap().as_str().into(),
            account: ACCOUNT.into(),
            project: project.into(),
            endpoint: ENDPOINT.into(),
            audience: "build-service".into(),
            generation: 1,
            request_digest: digest,
            target: binding["target"].as_str().unwrap().into(),
        };
        let policy = basalt::parse_policy_json(basalt::DEFAULT_POLICY_JSON).unwrap();
        let policy_ref = basalt::authority_core::BasaltPolicyRef::new(
            format!("blake3:{}", blake3::hash(basalt::DEFAULT_POLICY_JSON.as_bytes()).to_hex()),
            basalt::SUPPORTED_SCHEMA_VERSION,
            "mantle-build-submit",
        );
        let roots = vec![self.issuer.issuer().unwrap().as_str().to_string()];
        let verifier = basalt::mantle::Verifier {
            policy: &policy,
            policy_ref: &policy_ref,
            generation: 1,
            endpoint: ENDPOINT,
            audience: "build-service",
            trusted_roots: &roots,
        };
        let exact = verifier.enforcement(&authority).unwrap();
        let audience = ucan::AudienceDid::from(self.holder.issuer().unwrap());
        let token = ucan::issue_token_with_signer(
            &self.issuer,
            &audience,
            &ucan::CapabilitySet::new(vec![
                ucan::CapabilityDocument::new(exact.resource().into(), exact.ability().into()).unwrap(),
            ])
            .unwrap(),
            &ucan::ProofReferences::empty(),
            ucan::TokenTimeBounds::new(self.started - 1, self.started + 1800).unwrap(),
        )
        .unwrap();
        let nonce = seed();
        let action = basalt::ucan_enforcement::bind_enforcement_action(operation.action(), &exact).unwrap();
        let signature = ucan::sign_holder_invocation(
            &ucan::HolderInvocationSigningRequest::new(
                token.as_str(),
                &audience,
                exact.resource(),
                exact.ability(),
                action,
                ucan::HolderInvocationNonce::from_bytes(nonce),
            ),
            &self.holder,
            ucan::VerificationLimits::default(),
            ucan::HolderInvocationLimits::default(),
        )
        .unwrap();
        json!({"version":1,"authority":authority,"compact_token":token.as_str(),"nonce":nonce,
            "holder_signature":signature.as_bytes(),"request":request})
    }

    pub fn coordinator(&self) -> Value {
        read_json(&self.state.join("remote-coordinator-state.json"))
    }
    pub fn source_identity(&self) -> Value {
        let paths = [
            "src/nix_gateway/api_host.rs",
            "src/nix_gateway/authority.rs",
            "src/nix_gateway/resource_host.rs",
            "src/nix_gateway/evidence.rs",
            "src/remote_build.rs",
            "src/remote_build/usage.rs",
            "src/remote_build/usage_child.rs",
            "src/remote_build/gateway.rs",
            "crates/crunch-remote-core/src/resource_policy.rs",
            "crates/crunch-remote-core/src/resource_quota.rs",
            "crates/crunch-build/src/distributed/resource_quota_store.rs",
            "crates/crunch-nix-gateway/src/api.rs",
            "tests/support/gateway_runtime.rs",
            "tests/support/remote_endpoint_binding.rs",
            "tests/resource_gateway_runtime.rs",
        ];
        let files: serde_json::Map<String, Value> = paths
            .into_iter()
            .map(|path| (path.into(), Value::String(file_digest(&Path::new(env!("CARGO_MANIFEST_DIR")).join(path)))))
            .collect();
        json!({"schema":"mantle-gateway-runtime-source-binding-v1","binary":self.binary,
            "binary_blake3":file_digest(&self.binary),"source_blake3":files})
    }
    pub fn retain_final_identity(&self) {
        let final_identity = self.source_identity();
        private_json(&self.root.join("source-identities-end.json"), &final_identity);
        assert_eq!(final_identity, read_json(&self.root.join("source-identities-start.json")), "runtime source drift");
    }
    pub fn wait_terminal(&mut self, id: &str) -> Value {
        let deadline = Instant::now() + Duration::from_secs(150);
        loop {
            let (ok, value) = self.call(json!({"operation":"status","attempt_id":id}), PROJECT);
            assert!(ok, "authorized status failed: {value}");
            let phase = value["response"]["Status"]["observation"]["phase"].as_str().unwrap();
            if ["Succeeded", "Failed", "Cancelled"].contains(&phase) {
                return value;
            }
            assert!(Instant::now() < deadline, "gateway terminal deadline exceeded");
            std::thread::sleep(Duration::from_millis(100));
        }
    }

    pub fn write_observability(&self, memory: u64) {
        self.write_observability_with_concurrency(memory, 1);
    }

    pub fn write_observability_with_concurrency(&self, memory: u64, concurrency: u32) {
        let source = format!(
            r#"let remote = import "{}" in {{
  pools = [({{ pool_id = "runtime-pool", endpoints = [{{
    endpoint_id = "{ENDPOINT}",
    profile = {{ max_concurrency = {concurrency}, max_upload_bytes = 67108864, max_build_time_secs = 120,
      resource_inventory = ({{ total = {{ cpu_units = 1, memory_bytes = {memory},
        scratch_bytes = {SCRATCH}, accelerators = [], named_tokens = [] }} }} | remote.ResourceInventory) }},
  }}] }} | remote.RemoteBuilderPool)],
}}
"#,
            Path::new(env!("CARGO_MANIFEST_DIR")).join("lib/remote-builders.ncl").display()
        );
        let path = self.root.join("private/observability.ncl");
        if path.exists() {
            fs::remove_file(&path).unwrap();
        }
        private_bytes(&path, source.as_bytes());
    }

    fn provision_config(&mut self, cgroup: Option<&Path>, public_key: &str) {
        let binding_file = self.root.join("private/endpoint.json");
        remote_endpoint_binding::write_endpoint_bindings(
            &self.state,
            &binding_file,
            remote_endpoint_binding::Bindings {
                store_prefix: "/nix/store",
                builder: Some(ENDPOINT),
                substituters: &[],
            },
        );
        let endpoint = read_json(&binding_file);
        let features = json!({"architecture":"x86_64","platform":"linux","trust_domain":"runtime-dev",
            "isolation":"sandbox","kvm":false});
        let classes: Vec<Value> = [("small", FIRST_MEMORY, 1, 100), ("large", NEXT_MEMORY, 2, 200)]
            .into_iter()
            .map(|(id, memory, order, units)| {
                json!({"id":id,"features":features,"order":order,
                "quantities":{"cpu_units":1,"memory_bytes":memory,"scratch_bytes":SCRATCH},
                "reservation_units":units,"worker_endpoint_id":ENDPOINT,"inventory":{"total":vector(NEXT_MEMORY)}})
            })
            .collect();
        let observability = self.root.join("private/observability.ncl");
        self.write_observability(NEXT_MEMORY);
        let worker_root =
            self.state.join("remote-workers").join(blake3::hash(ENDPOINT.as_bytes()).to_hex().to_string());
        let mut args = vec![
            "--state-dir".into(),
            self.state.display().to_string(),
            "--store".into(),
            worker_root.join("store").display().to_string(),
            "--store-prefix".into(),
            "/nix/store".into(),
            "--remote-binding-config".into(),
            binding_file.display().to_string(),
            "remote".into(),
            "serve".into(),
            "--endpoint-id".into(),
            ENDPOINT.into(),
            "--binding".into(),
            "stdio-once".into(),
            "--executor".into(),
            "local-build".into(),
            "--execution-state-dir".into(),
            worker_root.join("state").display().to_string(),
            "--secret-manifest".into(),
            Path::new(env!("CARGO_MANIFEST_DIR")).join("secretspec.toml").display().to_string(),
            "--secret-profile".into(),
            "production".into(),
            "--secret-provider".into(),
            "systemd-credential://".into(),
        ];
        if let Some(root) = cgroup {
            args.extend(["--usage-cgroup-root".into(), root.display().to_string()]);
        }
        let resources = json!({"selection":{"version":1,"min_samples":1,"max_age_s":3600,"margin_basis_points":0,
                "max_measurement":{"cpu_units":1024,"memory_bytes":4294967296_u64,"scratch_bytes":SCRATCH},
                "max_duration_ms":120000,"max_io_bytes":67108864},
            "retry":{"version":1,"max_retries":1,"largest_class_order":2,"max_cumulative_units":300,"max_wall_ms":120000},
            "mode":"ProjectOptIn","selection_enabled":true,"oom_retry_enabled":true,
            "quota_enforcement_enabled":true,"sharing_enabled":false,"opted_in_projects":[PROJECT],
            "projects":{(PROJECT):{"account":ACCOUNT,"project_units":10000,"account_units":20000}},
            "schedule":{"version":1,"unit":"runtime-compute-units","window_id":"runtime-window",
                "window_start_s":self.started-1,"window_end_s":self.started+1800,"cpu_ms_per_unit":1000,
                "memory_byte_ms_per_unit":1125899906842624_u64,"executor_output_bytes_per_unit":1048576},
            "classes":classes,"sharing_scopes":[]});
        self.config = json!({"version":1,"endpoint":ENDPOINT,"audience":"build-service",
            "policy":blake3::hash(basalt::DEFAULT_POLICY_JSON.as_bytes()).as_bytes(),
            "authority":{"authority_file":self.root.join("private/authority.json"),"replay_directory":self.root.join("replay")},
            "builder_endpoint_state_directory":self.state,"builder_endpoint_binding":endpoint["build"]["endpoint_binding"],
            "builder_endpoint_scope":endpoint["build"]["scope"],"builder_endpoint_trust_requirement":endpoint["build"]["trust_requirement"],
            "journal_directory":self.root.join("journal"),"cursor_secret_file":self.root.join("private/cursor-key"),
            "completion_key_file":self.root.join("private/completion-key"),"quota_directory":self.root.join("quota"),
            "evidence_directory":self.root.join("evidence"),"features":{"x86_64-linux":features},
            "required_resources":{"x86_64-linux":requirements(FIRST_MEMORY)},"builder_program":self.binary,
            "builder_args":args,"trusted_builder_keys":[public_key],"observability_config":observability,
            "build_time_limit_secs":30,"secret_manifest":Path::new(env!("CARGO_MANIFEST_DIR")).join("secretspec.toml"),
            "secret_profile":"production","secret_provider":"systemd-credential://",
            "resources":resources});
    }

    async fn signed_derivation(
        &self,
        inputs: &Value,
        keys: &crunch_build::KeyPair,
        workload: &str,
    ) -> (String, String) {
        let mut store = crunch_store::StoreHandle::open(crunch_store::StoreConfig::new(
            crunch_store::StoreBackend::Snix,
            self.state.clone(),
            self.store.clone(),
            "/nix/store".into(),
        ))
        .await
        .unwrap();
        let root = inputs["busybox_root"].as_str().unwrap();
        let selector = Path::new(root).file_name().unwrap().to_str().unwrap();
        let trusted = nix_compat::narinfo::VerifyingKey::parse(inputs["trusted_public_key"].as_str().unwrap()).unwrap();
        let report = crunch_store::import_paths_from_cache_dir(
            &store,
            Path::new(inputs["input_cache"].as_str().unwrap()),
            Some(&[selector.into()]),
            &crunch_store::PullOptions {
                trust_unsigned: false,
                trusted_public_keys: vec![trusted],
            },
        )
        .await
        .unwrap();
        private_json(
            &self.root.join("signed-input-import.json"),
            &json!({
                "imported_count": report.imported_count,
                "skipped_already_present_count": report.skipped_already_present_count,
                "skipped_untrusted_count": report.skipped_untrusted_count,
                "skipped_hash_mismatch_count": report.skipped_hash_mismatch_count,
                "skipped_missing_nar_count": report.skipped_missing_nar_count,
                "skipped_parse_error_count": report.skipped_parse_error_count,
                "skipped_store_dir_mismatch_count": report.skipped_store_dir_mismatch_count,
                "total_nar_bytes": report.total_nar_bytes,
                "paths": report.paths.iter().map(|path| json!({
                    "store_path": path.store_path,
                    "nar_hash_hex": path.nar_hash_hex,
                    "nar_size": path.nar_size,
                })).collect::<Vec<_>>(),
            }),
        );
        assert_eq!(report.imported_count, 1, "declared signed input was not imported");
        assert_eq!(report.paths[0].store_path, selector);
        assert_eq!(
            [
                report.skipped_already_present_count,
                report.skipped_untrusted_count,
                report.skipped_hash_mismatch_count,
                report.skipped_missing_nar_count,
                report.skipped_parse_error_count,
                report.skipped_store_dir_mismatch_count,
            ],
            [0; 6],
            "signed input import rejected a selected member"
        );
        let script = match workload {
            "oom" => format!(
                "{root}/bin/busybox awk 'BEGIN {{s=sprintf(\"%100663296s\",\"x\");if(length(s)!=100663296)exit 9}}' || exit $?; echo gateway-success > \"$out\""
            ),
            "sigkill" => "kill -9 $$".into(),
            "simple" => "echo gateway-success > \"$out\"".into(),
            _ => panic!("unsupported bounded workload"),
        };
        let crunch = crunch_glue::CrunchDerivation {
            name: format!("gateway-runtime-{workload}"),
            builder: format!("{root}/bin/busybox"),
            system: "x86_64-linux".into(),
            args: vec!["sh".into(), "-c".into(), script],
            outputs: vec!["out".into()],
            dynamic_plan_outputs: vec![],
            env: HashMap::new(),
            inputs: vec![crunch_glue::Input::Source(root.into())],
            fixed_output: None,
            addressing_mode: "input-addressed".into(),
            provenance: None,
        };
        let source = format!(
            "{{ name = {}, builder = {}, system = \"x86_64-linux\", args = {}, outputs = [\"out\"], inputs = [{}], env = {{}}, addressing_mode = \"input-addressed\" }}\n",
            serde_json::to_string(&crunch.name).unwrap(),
            serde_json::to_string(&crunch.builder).unwrap(),
            serde_json::to_string(&crunch.args).unwrap(),
            serde_json::to_string(root).unwrap()
        );
        private_bytes(&self.source, source.as_bytes());
        let (path, drv) = crunch_glue::convert(&crunch, &mut crunch_glue::ConversionCache::new("/nix/store")).unwrap();
        let output = drv.outputs["out"].path.as_ref().unwrap().to_absolute_path();
        let bytes = drv.to_aterm_bytes_with_store_dir("/nix/store");
        private_bytes(&self.root.join("derivation.aterm"), &bytes);
        let blobs = store.blob_service();
        let mut writer = blobs.open_write().await;
        writer.write_all(&bytes).await.unwrap();
        let node = snix_castore::Node::File {
            digest: writer.close().await.unwrap(),
            size: bytes.len() as u64,
            executable: false,
        };
        let (nar_size, nar_sha256) = snix_store::nar::SimpleRenderer::new(blobs, store.directory_service())
            .calculate_nar(&node)
            .await
            .unwrap();
        let mut info = snix_store::path_info::PathInfo {
            store_path: path.clone(),
            node: node.clone(),
            references: drv.input_sources.iter().cloned().collect(),
            nar_size,
            nar_sha256,
            signatures: vec![],
            deriver: None,
            ca: None,
        };
        crunch_build::signing::sign_pathinfo_with_store_dir(&mut info, &keys.signing_key, "/nix/store");
        store
            .persist_and_export_signed_output(crunch_store::PersistOutputRequest {
                output_name: "drv",
                output_path: &path,
                path_info: info,
                final_node: node,
                provenance: None,
                is_root: true,
                root_source: None,
            })
            .await
            .unwrap();
        (path.to_absolute_path(), output)
    }
}

pub struct RunningGateway(Child);
impl Drop for RunningGateway {
    fn drop(&mut self) {
        unsafe {
            libc::kill(self.0.id() as i32, libc::SIGTERM);
        }
        let deadline = Instant::now() + Duration::from_secs(30);
        while self.0.try_wait().unwrap().is_none() && Instant::now() < deadline {
            std::thread::sleep(Duration::from_millis(25));
        }
        if self.0.try_wait().unwrap().is_none() {
            unsafe {
                libc::kill(-(self.0.id() as i32), libc::SIGKILL);
            }
        }
        let _ = self.0.wait();
    }
}
pub fn inherit_fd(command: &mut Command, fd: i32) {
    unsafe {
        command.pre_exec(move || {
            if libc::dup2(fd, 9) < 0 || libc::fcntl(9, libc::F_SETFD, 0) < 0 {
                return Err(std::io::Error::last_os_error());
            }
            Ok(())
        });
    }
}
fn retain_output(prefix: &Path, output: &Output) {
    fs::write(format!("{}.stdout", prefix.display()), &output.stdout).unwrap();
    fs::write(format!("{}.stderr", prefix.display()), &output.stderr).unwrap();
    fs::write(format!("{}.exit", prefix.display()), format!("{:?}\n", output.status.code())).unwrap();
}

fn file_digest(path: &Path) -> String {
    let mut file = fs::File::open(path).unwrap();
    let mut hasher = blake3::Hasher::new();
    let mut bytes = [0_u8; 65536];
    loop {
        let count = file.read(&mut bytes).unwrap();
        if count == 0 {
            break;
        }
        hasher.update(&bytes[..count]);
    }
    hasher.finalize().to_hex().to_string()
}
