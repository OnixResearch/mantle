use alloc::string::String;
use alloc::string::ToString;

const BUILD_KEY_DOMAIN: &str = "remote-coordinator-build-key";
const ALLOCATION_DOMAIN: &str = "mantle-external-batch-allocation-identity-v1";

#[derive(Debug, Clone, Copy)]
pub enum BuildKeySource<'a> {
    Action {
        action_id: &'a str,
        schema: &'a str,
        spec_digest_blake3: &'a str,
    },
    Derivation {
        declared_drv_path: &'a str,
        computed_drv_path: &'a str,
        drv_name: &'a str,
        drv_digest_blake3: &'a str,
    },
}

#[derive(Debug, Clone, Copy)]
pub struct ExpectedOutputKeyFacts<'a> {
    pub name: &'a str,
    pub logical_path: Option<&'a str>,
}

pub struct NormalizedBuildKeyFacts<'a, A, E, I, SI, O, F, C> {
    pub store_prefix: &'a str,
    pub source: BuildKeySource<'a>,
    pub system: &'a str,
    pub command_args: A,
    /// The adapter supplies entries in canonical sorted-map order.
    pub command_env: E,
    pub input_refs: I,
    pub source_input_refs: SI,
    pub expected_outputs: O,
    pub failure_replay: Option<(&'a str, &'a str)>,
    pub required_system: &'a str,
    pub required_features: F,
    pub required_sandbox_mode: &'a str,
    pub required_network_mode: &'a str,
    pub semantic_accelerator_classes: C,
}

pub fn normalized_remote_build_key<'a, A, E, I, SI, O, F, C>(
    facts: NormalizedBuildKeyFacts<'a, A, E, I, SI, O, F, C>,
) -> String
where
    A: IntoIterator<Item = &'a str>,
    E: IntoIterator<Item = (&'a str, &'a str)>,
    I: IntoIterator<Item = &'a str>,
    SI: IntoIterator<Item = &'a str>,
    O: IntoIterator<Item = ExpectedOutputKeyFacts<'a>>,
    F: IntoIterator<Item = &'a str>,
    C: IntoIterator<Item = &'a str>,
{
    let mut hasher = blake3::Hasher::new();
    hash_labeled_str(&mut hasher, "kind", BUILD_KEY_DOMAIN);
    hash_labeled_str(&mut hasher, "store-prefix", facts.store_prefix);
    match facts.source {
        BuildKeySource::Action {
            action_id,
            schema,
            spec_digest_blake3,
        } => {
            hash_labeled_str(&mut hasher, "source-kind", "action");
            hash_labeled_str(&mut hasher, "action-id", action_id);
            hash_labeled_str(&mut hasher, "schema", schema);
            hash_labeled_str(&mut hasher, "spec-digest", spec_digest_blake3);
        }
        BuildKeySource::Derivation {
            declared_drv_path,
            computed_drv_path,
            drv_name,
            drv_digest_blake3,
        } => {
            hash_labeled_str(&mut hasher, "source-kind", "derivation");
            hash_labeled_str(&mut hasher, "declared-drv-path", declared_drv_path);
            hash_labeled_str(&mut hasher, "computed-drv-path", computed_drv_path);
            hash_labeled_str(&mut hasher, "drv-name", drv_name);
            hash_labeled_str(&mut hasher, "drv-digest", drv_digest_blake3);
        }
    }
    hash_labeled_str(&mut hasher, "system", facts.system);
    for arg in facts.command_args {
        hash_labeled_str(&mut hasher, "command-arg", arg);
    }
    for (name, value) in facts.command_env {
        hash_labeled_str(&mut hasher, "env-name", name);
        hash_labeled_str(&mut hasher, "env-value", value);
    }
    for input_ref in facts.input_refs {
        hash_labeled_str(&mut hasher, "input-ref", input_ref);
    }
    for input_ref in facts.source_input_refs {
        hash_labeled_str(&mut hasher, "source-input-ref", input_ref);
    }
    for output in facts.expected_outputs {
        hash_labeled_str(&mut hasher, "expected-output-name", output.name);
        hash_labeled_str(&mut hasher, "expected-output-path", output.logical_path.unwrap_or("<content-addressed>"));
    }
    if let Some((source_bundle, execution)) = facts.failure_replay {
        hash_labeled_str(&mut hasher, "failure-replay-source-bundle", source_bundle);
        hash_labeled_str(&mut hasher, "failure-replay-execution", execution);
    }
    hash_labeled_str(&mut hasher, "required-system", facts.required_system);
    for feature in facts.required_features {
        hash_labeled_str(&mut hasher, "required-feature", feature);
    }
    hash_labeled_str(&mut hasher, "required-sandbox", facts.required_sandbox_mode);
    hash_labeled_str(&mut hasher, "required-network", facts.required_network_mode);
    for class in facts.semantic_accelerator_classes {
        hash_labeled_str(&mut hasher, "semantic-accelerator-class", class);
    }
    hasher.finalize().to_hex().to_string()
}

pub fn external_batch_allocation_identity(
    normalized_build_key: &str,
    adapter_instance_id: &str,
    dispatcher_generation: u64,
    allocation_attempt: u32,
) -> String {
    let mut hasher = blake3::Hasher::new();
    hash_labeled_str(&mut hasher, "domain", ALLOCATION_DOMAIN);
    hash_labeled_str(&mut hasher, "normalized-build-key", normalized_build_key);
    hash_labeled_str(&mut hasher, "adapter-instance", adapter_instance_id);
    let mut generation_digits = [0_u8; 20];
    hash_labeled_bytes(&mut hasher, "dispatcher-generation", decimal(dispatcher_generation, &mut generation_digits));
    let mut attempt_digits = [0_u8; 20];
    hash_labeled_bytes(&mut hasher, "allocation-attempt", decimal(u64::from(allocation_attempt), &mut attempt_digits));
    hasher.finalize().to_hex().to_string()
}

pub fn external_batch_semantic_capability_class<'a>(
    required_system: &str,
    required_features: impl IntoIterator<Item = &'a str>,
    required_sandbox_mode: &str,
    required_network_mode: &str,
) -> String {
    let mut hasher = blake3::Hasher::new();
    hash_labeled_str(&mut hasher, "system", required_system);
    for feature in required_features {
        hash_labeled_str(&mut hasher, "feature", feature);
    }
    hash_labeled_str(&mut hasher, "sandbox", required_sandbox_mode);
    hash_labeled_str(&mut hasher, "network", required_network_mode);
    hasher.finalize().to_hex().to_string()
}

pub fn hashed_transfer_artifact_id(domain: &str, identity: &str) -> String {
    let mut hasher = blake3::Hasher::new();
    hash_labeled_str(&mut hasher, "domain", domain);
    hash_labeled_str(&mut hasher, "identity", identity);
    alloc::format!("{domain}:{}", hasher.finalize().to_hex())
}

pub fn hashed_output_transfer_artifact_id(domain: &str, output_name: &str, logical_path: &str) -> String {
    let mut hasher = blake3::Hasher::new();
    hash_labeled_str(&mut hasher, "domain", domain);
    let identity_len = output_name
        .len()
        .checked_add(1)
        .and_then(|len| len.checked_add(logical_path.len()))
        .expect("an in-memory artifact identity cannot exceed usize");
    hash_labeled_prefix(&mut hasher, "identity", identity_len);
    hasher.update(output_name.as_bytes());
    hasher.update(b"\0");
    hasher.update(logical_path.as_bytes());
    alloc::format!("{domain}:{}", hasher.finalize().to_hex())
}

pub fn production_input_requested_content_digest<'a>(
    request_id: &str,
    input_refs: impl IntoIterator<Item = &'a str>,
) -> String {
    let mut hasher = blake3::Hasher::new();
    hash_labeled_str(&mut hasher, "kind", "production-input-transfer");
    hash_labeled_str(&mut hasher, "request-id", request_id);
    for input_ref in input_refs {
        hash_labeled_str(&mut hasher, "input-ref", input_ref);
    }
    hasher.finalize().to_hex().to_string()
}

fn hash_labeled_str(hasher: &mut blake3::Hasher, label: &str, value: &str) {
    hash_labeled_bytes(hasher, label, value.as_bytes());
}

fn hash_labeled_bytes(hasher: &mut blake3::Hasher, label: &str, value: &[u8]) {
    hash_labeled_prefix(hasher, label, value.len());
    hasher.update(value);
}

fn hash_labeled_prefix(hasher: &mut blake3::Hasher, label: &str, len: usize) {
    hasher.update(label.as_bytes());
    hasher.update(b":");
    let mut digits = [0_u8; 20];
    hasher.update(decimal(len as u64, &mut digits));
    hasher.update(b":");
}

fn decimal(mut number: u64, digits: &mut [u8; 20]) -> &[u8] {
    let mut index = digits.len();
    loop {
        index -= 1;
        digits[index] = b'0' + (number % 10) as u8;
        number /= 10;
        if number == 0 {
            break;
        }
    }
    &digits[index..]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalized_key_distinguishes_source_output_and_resource_requirements() {
        let key = |path: Option<&str>, feature: &str, source: BuildKeySource<'_>| {
            normalized_remote_build_key(NormalizedBuildKeyFacts {
                store_prefix: "/store",
                source,
                system: "x86_64-linux",
                command_args: ["build"],
                command_env: [("PATH", "/bin")],
                input_refs: ["input"],
                source_input_refs: ["source"],
                expected_outputs: [ExpectedOutputKeyFacts {
                    name: "out",
                    logical_path: path,
                }],
                failure_replay: None,
                required_system: "x86_64-linux",
                required_features: [feature],
                required_sandbox_mode: "strict",
                required_network_mode: "off",
                semantic_accelerator_classes: ["gpu-a"],
            })
        };
        let action = BuildKeySource::Action {
            action_id: "build",
            schema: "v1",
            spec_digest_blake3: "digest",
        };
        let baseline = key(Some("/store/out"), "feature-a", action);
        assert_eq!(baseline.len(), 64);
        assert_ne!(baseline, key(None, "feature-a", action));
        assert_ne!(baseline, key(Some("/store/out"), "feature-b", action));
        assert_ne!(
            baseline,
            key(Some("/store/out"), "feature-a", BuildKeySource::Derivation {
                declared_drv_path: "/store/a.drv",
                computed_drv_path: "/store/a.drv",
                drv_name: "a",
                drv_digest_blake3: "digest",
            })
        );
    }

    #[test]
    fn output_artifact_id_uses_exact_nul_separated_v1_identity_without_intermediate_allocation() {
        let domain = "remote-output-nar-artifact-v1";
        assert_eq!(
            hashed_output_transfer_artifact_id(domain, "out", "/store/out"),
            hashed_transfer_artifact_id(domain, "out\0/store/out"),
        );
        assert_ne!(
            hashed_output_transfer_artifact_id(domain, "out", "/store/out"),
            hashed_output_transfer_artifact_id(domain, "out/", "store/out"),
        );
    }

    #[test]
    fn external_allocation_retries_are_distinct_but_reproducible() {
        let first = external_batch_allocation_identity("key", "adapter", 9, 1);
        assert_eq!(first, external_batch_allocation_identity("key", "adapter", 9, 1));
        assert_ne!(first, external_batch_allocation_identity("key", "adapter", 9, 2));
        assert_ne!(first, external_batch_allocation_identity("key", "adapter", 10, 1));
    }
}
