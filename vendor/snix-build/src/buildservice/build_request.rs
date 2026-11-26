use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::collections::HashSet;
use std::path::PathBuf;

use bytes::Bytes;
use snix_castore::Node;
use snix_castore::PathComponent;
/// A BuildRequest describes the request of something to be run on the builder.
/// It is distinct from an actual \[Build\] that has already happened, or might be
/// currently ongoing.
///
/// A BuildRequest can be seen as a more normalized version of a Derivation
/// (parsed from A-Term), "writing out" some of the Nix-internal details about
/// how e.g. environment variables in the build are set.
///
/// Nix has some impurities when building a Derivation, for example the --cores option
/// ends up as an environment variable in the build, that's not part of the ATerm.
///
/// As of now, we serialize this into the BuildRequest, so builders can stay dumb.
/// This might change in the future.
///
/// There's also a big difference when it comes to how inputs are modelled:
///
/// * Nix only uses store path (strings) to describe the inputs. As store paths can be
///   input-addressed, a certain store path can contain different contents (as not all store paths
///   are binary reproducible). This requires that for every input-addressed input, the builder has
///   access to either the input's deriver (and needs to build it) or else a trusted source for the
///   built input. to upload input-addressed paths, requiring the trusted users concept.
/// * snix-build records a list of snix.castore.v1.Node as inputs. These map from the store path
///   base name to their contents, relieving the builder from having to "trust" any input-addressed
///   paths, contrary to Nix.
///
/// While this approach gives a better hermeticity, it has one downside:
/// A BuildRequest can only be sent once the contents of all its inputs are known.
///
/// As of now, we're okay to accept this, but it prevents uploading an
/// entirely-non-IFD subgraph of BuildRequests eagerly.
#[derive(Default, Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct BuildRequest {
    /// The list of all root nodes that should be visible in `inputs_dir` at the
    /// time of the build.
    /// As all references are content-addressed, no additional signatures are
    /// needed to substitute / make these available in the build environment.
    pub inputs: BTreeMap<PathComponent, Node>,
    /// The command (and its args) executed as the build script.
    /// In the case of a Nix derivation, this is usually
    /// \["/path/to/some-bash/bin/bash", "-e", "/path/to/some/builder.sh"\].
    pub command_args: Vec<String>,
    /// The working dir of the command, relative to the build root.
    /// "build", in the case of Nix.
    /// This MUST be a clean relative path, without any ".", "..", or superfluous
    /// slashes.
    pub working_dir: PathBuf,
    /// A list of "scratch" paths, relative to the build root.
    /// These will be write-able during the build.
    /// \[build, nix/store\] in the case of Nix.
    /// These MUST be clean relative paths, without any ".", "..", or superfluous
    /// slashes, and sorted.
    pub scratch_paths: Vec<PathBuf>,
    /// The path where the castore input nodes will be located at,
    /// "nix/store" in case of Nix.
    /// Builds might also write into here (Nix builds do that).
    /// This MUST be a clean relative path, without any ".", "..", or superfluous
    /// slashes.
    pub inputs_dir: PathBuf,
    /// The list of output paths the build is expected to produce,
    /// relative to the root.
    /// If the path is not produced, the build is considered to have failed.
    /// These MUST be clean relative paths, without any ".", "..", or superfluous
    /// slashes, and sorted.
    pub outputs: Vec<PathBuf>,
    /// The list of environment variables and their values that should be set
    /// inside the build environment.
    /// This includes both environment vars set inside the derivation, as well as
    /// more "ephemeral" ones like NIX_BUILD_CORES, controlled by the `--cores`
    /// CLI option of `nix-build`.
    /// For now, we consume this as an option when turning a Derivation into a BuildRequest,
    /// similar to how Nix has a `--cores` option.
    /// We don't want to bleed these very nix-specific sandbox impl details into
    /// (dumber) builders if we don't have to.
    /// Environment variables are sorted by their keys.
    pub environment_vars: Vec<EnvVar>,
    /// A set of constraints that need to be satisfied on a build host before a
    /// Build can be started.
    pub constraints: HashSet<BuildConstraints>,
    /// Additional (small) files and their contents that should be placed into the
    /// build environment, but outside inputs_dir.
    /// Used for passAsFile and structuredAttrs in Nix.
    pub additional_files: Vec<AdditionalFile>,
    /// If this is an non-empty list, all paths in `outputs` are scanned for these.
    /// For Nix, `refscan_needles` would be populated with the nixbase32 hash parts of
    /// every input store path and output store path. The latter is necessary to scan
    /// for references between multi-output derivations.
    pub refscan_needles: Vec<String>,
    /// Optional declared retained-tool workspace mount. Host storage paths are
    /// runtime-only and are never serialized into action or transport identity.
    pub workspace: Option<StatefulWorkspaceRequest>,
    /// Explicit validated DNS resolver configuration for admitted network-enabled
    /// sandboxes. Must be `None` when network access is denied.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub dns_config: Option<SandboxDnsConfig>,
}

pub const MAX_DNS_NAMESERVERS: usize = 8;
pub const MAX_DNS_SEARCH_DOMAINS: usize = 6;
pub const MAX_DNS_SEARCH_DOMAIN_BYTES: usize = 253;
pub const MAX_DNS_RESOLV_CONF_BYTES: usize = 4 * 1024;

#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct SandboxDnsConfig {
    pub nameservers: Vec<std::net::IpAddr>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub search_domains: Vec<String>,
}

impl SandboxDnsConfig {
    pub fn new(nameservers: Vec<std::net::IpAddr>, search_domains: Vec<String>) -> Result<Self, String> {
        let config = Self {
            nameservers,
            search_domains,
        };
        config.validate()?;
        Ok(config)
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.nameservers.is_empty() {
            return Err("sandbox DNS configuration must specify at least one nameserver".to_string());
        }
        if self.nameservers.len() > MAX_DNS_NAMESERVERS {
            return Err(format!(
                "sandbox DNS configuration specifies {} nameservers, exceeding bound {MAX_DNS_NAMESERVERS}",
                self.nameservers.len()
            ));
        }
        if self.search_domains.len() > MAX_DNS_SEARCH_DOMAINS {
            return Err(format!(
                "sandbox DNS configuration specifies {} search domains, exceeding bound {MAX_DNS_SEARCH_DOMAINS}",
                self.search_domains.len()
            ));
        }
        for domain in &self.search_domains {
            validate_dns_search_domain(domain)?;
        }
        let rendered = self.render_unchecked();
        if rendered.len() > MAX_DNS_RESOLV_CONF_BYTES {
            return Err(format!(
                "rendered sandbox resolv.conf is {} bytes, exceeding bound {MAX_DNS_RESOLV_CONF_BYTES}",
                rendered.len()
            ));
        }
        Ok(())
    }

    pub fn render_resolv_conf(&self) -> Result<String, String> {
        self.validate()?;
        Ok(self.render_unchecked())
    }

    fn render_unchecked(&self) -> String {
        let mut out = String::new();
        for nameserver in &self.nameservers {
            out.push_str("nameserver ");
            out.push_str(&nameserver.to_string());
            out.push('\n');
        }
        if !self.search_domains.is_empty() {
            out.push_str("search ");
            out.push_str(&self.search_domains.join(" "));
            out.push('\n');
        }
        out
    }

    pub fn parse_resolv_conf(contents: &str) -> Result<Self, String> {
        if contents.len() > MAX_DNS_RESOLV_CONF_BYTES {
            return Err(format!(
                "resolv.conf input is {} bytes, exceeding bound {MAX_DNS_RESOLV_CONF_BYTES}",
                contents.len()
            ));
        }
        if contents.as_bytes().contains(&0) {
            return Err("resolv.conf input contains NUL byte".to_string());
        }
        let mut nameservers = Vec::new();
        let mut search_domains = Vec::new();
        for (line_index, raw_line) in contents.lines().enumerate() {
            let line = raw_line.trim();
            if line.is_empty() || line.starts_with('#') || line.starts_with(';') {
                continue;
            }
            let mut parts = line.split_whitespace();
            let Some(keyword) = parts.next() else {
                continue;
            };
            match keyword {
                "nameserver" => {
                    let Some(addr_str) = parts.next() else {
                        return Err(format!("resolv.conf line {} missing nameserver address", line_index + 1));
                    };
                    if parts.next().is_some() {
                        return Err(format!(
                            "resolv.conf line {} has trailing tokens after nameserver address",
                            line_index + 1
                        ));
                    }
                    let ip = addr_str.parse::<std::net::IpAddr>().map_err(|_| {
                        format!("resolv.conf line {} has invalid nameserver IP {addr_str:?}", line_index + 1)
                    })?;
                    nameservers.push(ip);
                }
                "search" => {
                    let domains: Vec<String> = parts.map(str::to_string).collect();
                    if domains.is_empty() {
                        return Err(format!("resolv.conf line {} missing search domain", line_index + 1));
                    }
                    search_domains = domains;
                }
                "domain" => {
                    let Some(domain) = parts.next() else {
                        return Err(format!("resolv.conf line {} missing domain value", line_index + 1));
                    };
                    if parts.next().is_some() {
                        return Err(format!(
                            "resolv.conf line {} has trailing tokens after domain value",
                            line_index + 1
                        ));
                    }
                    search_domains = vec![domain.to_string()];
                }
                "options" | "sortlist" => {}
                other => {
                    return Err(format!(
                        "resolv.conf line {} contains unsupported directive {other:?}",
                        line_index + 1
                    ));
                }
            }
        }
        Self::new(nameservers, search_domains)
    }

    pub fn from_host_resolv_conf(path: &std::path::Path) -> Result<Self, String> {
        use std::io::Read;

        let file = std::fs::File::open(path)
            .map_err(|error| format!("opening host resolv.conf {}: {error}", path.display()))?;
        let mut bytes = Vec::new();
        file.take((MAX_DNS_RESOLV_CONF_BYTES as u64).saturating_add(1))
            .read_to_end(&mut bytes)
            .map_err(|error| format!("reading host resolv.conf {}: {error}", path.display()))?;
        if bytes.len() > MAX_DNS_RESOLV_CONF_BYTES {
            return Err(format!("host resolv.conf {} exceeds bound {MAX_DNS_RESOLV_CONF_BYTES} bytes", path.display()));
        }
        let text =
            String::from_utf8(bytes).map_err(|_| format!("host resolv.conf {} is not valid UTF-8", path.display()))?;
        Self::parse_resolv_conf(&text)
    }
}

fn validate_dns_search_domain(domain: &str) -> Result<(), String> {
    if domain.is_empty() {
        return Err("sandbox DNS search domain must not be empty".to_string());
    }
    if domain.len() > MAX_DNS_SEARCH_DOMAIN_BYTES {
        return Err(format!("sandbox DNS search domain {domain:?} exceeds {MAX_DNS_SEARCH_DOMAIN_BYTES} bytes"));
    }
    if domain.starts_with('.') || domain.ends_with('.') || domain.contains("..") {
        return Err(format!("sandbox DNS search domain {domain:?} has empty label"));
    }
    for label in domain.split('.') {
        if label.is_empty() || label.len() > 63 {
            return Err(format!("sandbox DNS search domain {domain:?} has invalid label length"));
        }
        if label.starts_with('-') || label.ends_with('-') {
            return Err(format!("sandbox DNS search domain {domain:?} label must not start or end with '-'"));
        }
        if !label.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_') {
            return Err(format!("sandbox DNS search domain {domain:?} contains invalid characters"));
        }
    }
    Ok(())
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum StatefulWorkspaceMode {
    #[default]
    None,
    ImmutableSnapshot,
    MutableSession,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct StatefulWorkspaceLeaseBinding {
    pub worker_id: String,
    pub authority_class: String,
    pub job_id: String,
    pub attempt_id: String,
    pub fence_generation: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct StatefulWorkspaceRequest {
    pub mode: StatefulWorkspaceMode,
    pub workspace_id: Option<String>,
    pub guest_path: PathBuf,
    /// Root name beneath the declared castore input directory for immutable snapshots.
    pub snapshot_input_name: Option<PathBuf>,
    pub compatibility_digest_blake3: String,
    pub toolchain_refs: Vec<String>,
    pub quota_bytes_max: u64,
    pub quota_files_max: u32,
    pub quota_snapshots_max: u32,
    pub retention_class: String,
    pub retention_workspace_count_max: u32,
    pub retention_idle_generations_max: u64,
    pub retention_age_generations_max: u64,
    pub retention_quarantine_count_max: u32,
    pub generation: u64,
    pub lease: Option<StatefulWorkspaceLeaseBinding>,
    pub sensitive_paths: Vec<String>,
    pub secret_markers: Vec<String>,
    pub scan_depth_max: u32,
    pub path_bytes_max: u32,
    pub snapshot_enabled: bool,
    pub clean_rebuild_enabled: bool,
    pub clean_rebuild_require_declared_inputs: bool,
    /// Resolved by the local imperative shell after lease/containment checks.
    /// This value is deliberately absent from serialized requests.
    #[serde(skip, default)]
    pub runtime_host_path: Option<PathBuf>,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct EnvVar {
    /// name of the environment variable. Must not contain =.
    pub key: String,
    pub value: Bytes,
}
/// BuildConstraints represents certain conditions that must be fulfilled
/// inside the build environment to be able to build this.
/// Constraints can be things like required architecture and minimum amount of memory.
/// The required input paths are *not* represented in here, because it
/// wouldn't be hermetic enough - see the comment around inputs too.
#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum BuildConstraints {
    /// The system that's needed to execute the build.
    /// Must not be empty.
    System(String),
    /// The amount of memory required to be available for the build, in bytes.
    MinMemory(u64),
    /// An absolute path that need to be available in the build
    /// environment, like `/dev/kvm`.
    /// This is distinct from the castore nodes in inputs.
    /// These MUST be clean absolute paths, without any ".", "..", or superfluous
    /// slashes, and sorted.
    AvailableReadOnlyPath(PathBuf),
    /// Whether the build should be able to access the network.
    NetworkAccess,
    /// Whether to provide a /bin/sh inside the build environment, usually a static bash.
    ProvideBinSh,
    /// Whether normal kernel `/proc` metadata files must remain visible.
    ///
    /// This exposes host-kernel observations such as `/proc/cpuinfo`. Callers
    /// must request it explicitly for foreign build protocols that require it.
    ProvideProcMetadata,
    /// Whether real `/dev/random` and `/dev/urandom` devices must remain visible.
    ProvideRandomDevices,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct AdditionalFile {
    pub path: PathBuf,
    pub contents: Bytes,
}

/// Measured local sandbox admission wait and execution durations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct SandboxExecutionTiming {
    /// Milliseconds spent queued waiting for a local sandbox admission permit.
    pub admission_wait_ms: u64,
    /// Milliseconds spent executing after sandbox admission was acquired.
    pub execution_ms: u64,
}

/// Describes the result of a [BuildRequest].
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct BuildResult {
    /// The outputs that were produced after successfully building.
    // They are sorted by the order specified in the build request.
    pub outputs: Vec<BuildOutput>,
    /// Captured stdout+stderr from the build, if available.
    pub log: Option<String>,
    /// Separate local sandbox admission-wait and execution timing, when available.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub timing: Option<SandboxExecutionTiming>,
}

/// Specific information about an individual output in [BuildResult].
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct BuildOutput {
    /// The castore node describing the contents.
    pub node: Node,
    /// Indexes into the found [BuildRequest::refscan_needles] in that output.
    pub output_needles: BTreeSet<u64>,
}
