//! Typed operator configuration for remote build farms.
//!
//! Loads the `remote-builders.ncl` contract and produces provider-neutral
//! capability facts for route planning and coordinator matching.
//! Provider-specific transport parameters remain inside adapters.
//!
//! r[impl remote_builds.production_operator_configuration]

use serde::Deserialize;

/// Transport binding for a remote builder endpoint.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
pub enum RemoteTransportMode {
    #[default]
    Stdio,
    SshStdio,
    P2p,
}

/// Sandbox enforcement mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
pub enum RemoteSandboxMode {
    #[default]
    Practical,
    Strict,
}

/// Network access policy during remote builds.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
pub enum RemoteNetworkMode {
    #[default]
    None,
    Local,
    Full,
}

/// Fallback policy when a remote build fails.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
pub enum RemoteFallbackPolicy {
    #[default]
    Never,
    Always,
    TrustedOnly,
}

/// Capability profile for a remote builder.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct RemoteCapabilityProfile {
    #[serde(default = "default_system")]
    pub system: String,
    #[serde(default = "default_sandbox_mode")]
    pub sandbox_mode: RemoteSandboxMode,
    #[serde(default = "default_network_mode")]
    pub network_mode: RemoteNetworkMode,
    #[serde(default)]
    pub features: Vec<String>,
    #[serde(default = "default_concurrency")]
    pub max_concurrency: u32,
    #[serde(default = "default_upload_bytes")]
    pub max_upload_bytes: u64,
    #[serde(default = "default_build_time_secs")]
    pub max_build_time_secs: u64,
}

fn default_sandbox_mode() -> RemoteSandboxMode {
    RemoteSandboxMode::Practical
}
fn default_system() -> String {
    "x86_64-linux".to_string()
}
fn default_network_mode() -> RemoteNetworkMode {
    RemoteNetworkMode::None
}
fn default_concurrency() -> u32 {
    1
}
fn default_upload_bytes() -> u64 {
    1_073_741_824
}
fn default_build_time_secs() -> u64 {
    3_600
}

impl Default for RemoteCapabilityProfile {
    fn default() -> Self {
        Self {
            system: "x86_64-linux".to_string(),
            sandbox_mode: RemoteSandboxMode::Practical,
            network_mode: RemoteNetworkMode::None,
            features: Vec::new(),
            max_concurrency: 1,
            max_upload_bytes: 1_073_741_824,
            max_build_time_secs: 3_600,
        }
    }
}

/// A remote builder endpoint with transport and capability facts.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Default)]
pub struct RemoteEndpoint {
    pub endpoint_id: String,
    #[serde(default = "default_transport")]
    pub transport: RemoteTransportMode,
    #[serde(default)]
    pub profile: RemoteCapabilityProfile,
}

fn default_transport() -> RemoteTransportMode {
    RemoteTransportMode::Stdio
}

/// A cryptographic trust root for verifying remote outputs.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Default)]
pub struct TrustRoot {
    pub public_key_name: String,
    pub public_key_base64: String,
}

/// Publication target configuration.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
pub enum PublisherMode {
    #[default]
    NixCache,
    LocalArchive,
    S3,
}

fn default_publisher_mode() -> PublisherMode {
    PublisherMode::NixCache
}

/// Publisher profile configuration.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Default)]
pub struct PublisherProfile {
    #[serde(default = "default_publisher_mode")]
    pub mode: PublisherMode,
    #[serde(default)]
    pub target_url: String,
    #[serde(default)]
    pub trusted_public_keys: Vec<TrustRoot>,
}

/// A configured remote builder pool with typed capability and trust facts.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Default)]
pub struct RemoteBuilderPool {
    pub pool_id: String,
    #[serde(default)]
    pub endpoints: Vec<RemoteEndpoint>,
    #[serde(default = "default_fallback_policy")]
    pub fallback_policy: RemoteFallbackPolicy,
    #[serde(default)]
    pub output_trust_roots: Vec<TrustRoot>,
    #[serde(default)]
    pub publishers: Vec<PublisherProfile>,
}

fn default_fallback_policy() -> RemoteFallbackPolicy {
    RemoteFallbackPolicy::TrustedOnly
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct RemoteBuildFarmConfig {
    #[serde(default)]
    pub pools: Vec<RemoteBuilderPool>,
}

impl RemoteBuildFarmConfig {
    /// Maximum number of configured pools.
    pub const MAX_POOLS: usize = 32;

    /// Maximum number of endpoints per pool.
    pub const MAX_ENDPOINTS_PER_POOL: usize = 64;

    /// Validate the configuration against bounded limits.
    pub fn validate(&self) -> Result<(), String> {
        if self.pools.len() > Self::MAX_POOLS {
            return Err(format!("too many remote builder pools: {} > {}", self.pools.len(), Self::MAX_POOLS));
        }
        for pool in &self.pools {
            if pool.endpoints.len() > Self::MAX_ENDPOINTS_PER_POOL {
                return Err(format!(
                    "pool '{}' has too many endpoints: {} > {}",
                    pool.pool_id,
                    pool.endpoints.len(),
                    Self::MAX_ENDPOINTS_PER_POOL
                ));
            }
            if pool.pool_id.is_empty() {
                return Err("pool_id must not be empty".to_string());
            }
            for endpoint in &pool.endpoints {
                if endpoint.endpoint_id.is_empty() {
                    return Err("endpoint_id must not be empty".to_string());
                }
            }
            // Check for duplicate endpoint IDs within a pool.
            let mut seen = std::collections::BTreeSet::new();
            for endpoint in &pool.endpoints {
                if !seen.insert(&endpoint.endpoint_id) {
                    return Err(format!("duplicate endpoint_id '{}' in pool '{}'", endpoint.endpoint_id, pool.pool_id));
                }
            }
        }
        // Check for duplicate pool IDs.
        let mut seen_pools = std::collections::BTreeSet::new();
        for pool in &self.pools {
            if !seen_pools.insert(&pool.pool_id) {
                return Err(format!("duplicate pool_id '{}'", pool.pool_id));
            }
        }
        Ok(())
    }
}

impl Default for RemoteBuildFarmConfig {
    fn default() -> Self {
        Self { pools: Vec::new() }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_config() -> RemoteBuildFarmConfig {
        RemoteBuildFarmConfig {
            pools: vec![RemoteBuilderPool {
                pool_id: "build-farm-1".to_string(),
                endpoints: vec![RemoteEndpoint {
                    endpoint_id: "builder-01".to_string(),
                    transport: RemoteTransportMode::SshStdio,
                    profile: RemoteCapabilityProfile {
                        system: "x86_64-linux".to_string(),
                        sandbox_mode: RemoteSandboxMode::Strict,
                        network_mode: RemoteNetworkMode::None,
                        features: vec!["tigerstyle".to_string()],
                        max_concurrency: 4,
                        max_upload_bytes: 1_073_741_824,
                        max_build_time_secs: 7_200,
                    },
                }],
                fallback_policy: RemoteFallbackPolicy::Always,
                output_trust_roots: vec![TrustRoot {
                    public_key_name: "builder-01".to_string(),
                    public_key_base64: "dGVzdC1rZXk=".to_string(),
                }],
                publishers: vec![PublisherProfile {
                    mode: PublisherMode::NixCache,
                    target_url: "https://cache.example.com".to_string(),
                    trusted_public_keys: Vec::new(),
                }],
            }],
        }
    }

    #[test]
    fn valid_config_passes_validation() {
        let config = sample_config();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn too_many_pools_rejected() {
        let pools = (0..RemoteBuildFarmConfig::MAX_POOLS + 1)
            .map(|i| RemoteBuilderPool {
                pool_id: format!("pool-{i}"),
                endpoints: Vec::new(),
                fallback_policy: RemoteFallbackPolicy::Never,
                output_trust_roots: Vec::new(),
                publishers: Vec::new(),
            })
            .collect();
        let config = RemoteBuildFarmConfig { pools };
        assert!(config.validate().is_err());
    }

    #[test]
    fn duplicate_pool_id_rejected() {
        let config = RemoteBuildFarmConfig {
            pools: vec![
                RemoteBuilderPool {
                    pool_id: "dup".to_string(),
                    ..Default::default()
                },
                RemoteBuilderPool {
                    pool_id: "dup".to_string(),
                    ..Default::default()
                },
            ],
        };
        assert!(config.validate().is_err());
    }

    #[test]
    fn duplicate_endpoint_id_rejected_in_pool() {
        let config = RemoteBuildFarmConfig {
            pools: vec![RemoteBuilderPool {
                pool_id: "pool-1".to_string(),
                endpoints: vec![
                    RemoteEndpoint {
                        endpoint_id: "builder-a".to_string(),
                        ..Default::default()
                    },
                    RemoteEndpoint {
                        endpoint_id: "builder-a".to_string(),
                        ..Default::default()
                    },
                ],
                ..Default::default()
            }],
        };
        assert!(config.validate().is_err());
    }

    #[test]
    fn empty_endpoint_id_rejected() {
        let config = RemoteBuildFarmConfig {
            pools: vec![RemoteBuilderPool {
                pool_id: "pool-1".to_string(),
                endpoints: vec![RemoteEndpoint {
                    endpoint_id: "".to_string(),
                    ..Default::default()
                }],
                ..Default::default()
            }],
        };
        assert!(config.validate().is_err());
    }

    #[test]
    fn empty_pool_id_rejected() {
        let config = RemoteBuildFarmConfig {
            pools: vec![RemoteBuilderPool {
                pool_id: "".to_string(),
                ..Default::default()
            }],
        };
        assert!(config.validate().is_err());
    }

    #[test]
    fn default_config_valid() {
        let config = RemoteBuildFarmConfig::default();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn config_deserializes_from_json() {
        let json = r#"{
            "pools": [{
                "pool_id": "test-pool",
                "endpoints": [{
                    "endpoint_id": "builder-01",
                    "transport": "ssh-stdio",
                    "profile": {
                        "system": "aarch64-linux",
                        "sandbox_mode": "strict",
                        "max_concurrency": 8
                    }
                }],
                "output_trust_roots": [{
                    "public_key_name": "builder-01",
                    "public_key_base64": "dGVzdA=="
                }]
            }]
        }"#;
        let config: RemoteBuildFarmConfig = serde_json::from_str(json).unwrap();
        assert!(config.validate().is_ok());
        assert_eq!(config.pools.len(), 1);
        assert_eq!(config.pools[0].endpoints[0].profile.system, "aarch64-linux");
        assert_eq!(config.pools[0].endpoints[0].profile.max_concurrency, 8);
    }

    #[test]
    fn config_defaults_are_safe() {
        // A pool with minimal fields should get safe defaults.
        let json = r#"{
            "pools": [{
                "pool_id": "minimal-pool",
                "endpoints": [{
                    "endpoint_id": "minimal-builder"
                }]
            }]
        }"#;
        let config: RemoteBuildFarmConfig = serde_json::from_str(json).unwrap();
        assert!(config.validate().is_ok());
        let endpoint = &config.pools[0].endpoints[0];
        assert_eq!(endpoint.transport, RemoteTransportMode::Stdio);
        assert_eq!(endpoint.profile.system, "x86_64-linux");
        assert_eq!(endpoint.profile.sandbox_mode, RemoteSandboxMode::Practical);
        assert_eq!(endpoint.profile.network_mode, RemoteNetworkMode::None);
        assert_eq!(endpoint.profile.max_concurrency, 1);
    }
}
