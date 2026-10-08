use alloc::collections::BTreeMap;
use alloc::string::String;

const MAX_EXECUTOR_ARGS: usize = 512;
const MAX_EXECUTOR_ENV_VARS: usize = 512;
const MAX_EXECUTOR_ENV_VALUE_BYTES: usize = 65_536;
const MAX_EXPECTED_OUTPUTS: usize = 128;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlanBlocker {
    ArgCountExceeded,
    BuilderEmpty,
    EnvCountExceeded,
    EnvNameEmpty,
    EnvValueExceeded,
    ExpectedOutputsEmpty,
    ExpectedOutputCountExceeded,
}

impl PlanBlocker {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ArgCountExceeded => "remote-executor-arg-count-exceeds-512",
            Self::BuilderEmpty => "remote-executor-builder-empty",
            Self::EnvCountExceeded => "remote-executor-env-count-exceeds-512",
            Self::EnvNameEmpty => "remote-executor-env-name-empty",
            Self::EnvValueExceeded => "remote-executor-env-value-exceeds-65536",
            Self::ExpectedOutputsEmpty => "remote-expected-outputs-empty",
            Self::ExpectedOutputCountExceeded => "remote-expected-output-count-exceeds-128",
        }
    }
}

/// Fully normalized source identity. The adapter resolves derivations and
/// validates action JSON before constructing these digest facts.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExecutableSourceFacts<'a> {
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ExpectedOutputFacts<'a> {
    pub name: &'a str,
    pub logical_path: Option<&'a str>,
}

pub struct ExecutablePlanFacts<'input, 'output, I>
where I: IntoIterator<Item = ExpectedOutputFacts<'output>>
{
    pub request_id: &'input str,
    pub store_prefix: &'input str,
    pub source: ExecutableSourceFacts<'input>,
    pub system: &'input str,
    pub command_args: &'input [String],
    pub command_env: &'input BTreeMap<String, String>,
    pub expected_outputs: I,
}

/// Exactly the accepted labeled-length preimage and field ordering. Render
/// hex at the boundary; the core returns the digest bytes without allocating
/// a second canonical JSON, copying input vectors, or reading host authority.
// r[impl remote_builds.hexagonal_core]
pub fn executable_plan_digest<'input, 'output, I>(
    facts: ExecutablePlanFacts<'input, 'output, I>,
) -> Result<[u8; 32], PlanBlocker>
where I: IntoIterator<Item = ExpectedOutputFacts<'output>> {
    if facts.command_args.len() > MAX_EXECUTOR_ARGS {
        return Err(PlanBlocker::ArgCountExceeded);
    }
    if facts.command_args.first().is_none_or(String::is_empty) {
        return Err(PlanBlocker::BuilderEmpty);
    }
    if facts.command_env.len() > MAX_EXECUTOR_ENV_VARS {
        return Err(PlanBlocker::EnvCountExceeded);
    }
    for (name, value) in facts.command_env {
        if name.is_empty() {
            return Err(PlanBlocker::EnvNameEmpty);
        }
        if value.len() > MAX_EXECUTOR_ENV_VALUE_BYTES {
            return Err(PlanBlocker::EnvValueExceeded);
        }
    }
    let mut hasher = blake3::Hasher::new();
    hash_labeled(&mut hasher, "request-id", facts.request_id);
    hash_labeled(&mut hasher, "store-prefix", facts.store_prefix);
    hash_source(&mut hasher, facts.source);
    hash_labeled(&mut hasher, "system", facts.system);
    for arg in facts.command_args {
        hash_labeled(&mut hasher, "command-arg", arg);
    }
    for (name, value) in facts.command_env {
        hash_labeled(&mut hasher, "env-name", name);
        hash_labeled(&mut hasher, "env-value", value);
    }
    let mut output_count = 0_usize;
    for output in facts.expected_outputs {
        output_count += 1;
        if output_count > MAX_EXPECTED_OUTPUTS {
            return Err(PlanBlocker::ExpectedOutputCountExceeded);
        }
        hash_labeled(&mut hasher, "output-name", output.name);
        match output.logical_path {
            Some(logical_path) => hash_labeled(&mut hasher, "output-path", logical_path),
            None => hash_labeled(&mut hasher, "output-path-state", "content-addressed-unknown"),
        }
    }
    if output_count == 0 {
        return Err(PlanBlocker::ExpectedOutputsEmpty);
    }
    Ok(*hasher.finalize().as_bytes())
}

/// The ordered v1 output-receipt preimage. Callers may append one output at a
/// time so a failed host-side NAR inspection cannot require an intermediate
/// allocation or publish a partial digest.
#[derive(Debug, Clone, Copy)]
pub struct OutputReceiptFields<'a> {
    pub name: &'a str,
    pub logical_path: &'a str,
    pub content_digest_blake3: &'a str,
    pub artifact_attestation_digest_blake3: &'a str,
    pub size_bytes: u64,
    pub nar_payload_digest_blake3: Option<&'a str>,
    pub nar_payload_size_bytes: Option<u64>,
}

pub struct OrderedOutputReceipt {
    hasher: blake3::Hasher,
}

impl Default for OrderedOutputReceipt {
    fn default() -> Self {
        Self::new()
    }
}

impl OrderedOutputReceipt {
    pub fn new() -> Self {
        Self {
            hasher: blake3::Hasher::new(),
        }
    }

    pub fn append(&mut self, fields: OutputReceiptFields<'_>) {
        hash_labeled(&mut self.hasher, "output-name", fields.name);
        hash_labeled(&mut self.hasher, "output-path", fields.logical_path);
        hash_labeled(&mut self.hasher, "content-digest", fields.content_digest_blake3);
        hash_labeled(&mut self.hasher, "artifact-digest", fields.artifact_attestation_digest_blake3);
        hash_labeled_u64(&mut self.hasher, "size-bytes", fields.size_bytes);
        hash_labeled(&mut self.hasher, "nar-payload-digest", fields.nar_payload_digest_blake3.unwrap_or("<absent>"));
        match fields.nar_payload_size_bytes {
            Some(size) => hash_labeled_u64(&mut self.hasher, "nar-payload-size", size),
            None => hash_labeled(&mut self.hasher, "nar-payload-size", "<absent>"),
        }
    }

    pub fn finish(self) -> [u8; 32] {
        *self.hasher.finalize().as_bytes()
    }
}

fn hash_labeled_u64(hasher: &mut blake3::Hasher, label: &str, value: u64) {
    let mut decimal = [0_u8; 20];
    let mut remaining = value;
    let mut start = decimal.len();
    loop {
        start -= 1;
        decimal[start] = b'0' + (remaining % 10) as u8;
        remaining /= 10;
        if remaining == 0 {
            break;
        }
    }
    hash_labeled_bytes(hasher, label, &decimal[start..]);
}

fn hash_source(hasher: &mut blake3::Hasher, source: ExecutableSourceFacts<'_>) {
    match source {
        ExecutableSourceFacts::Action {
            action_id,
            schema,
            spec_digest_blake3,
        } => {
            hash_labeled(hasher, "source-kind", "action");
            hash_labeled(hasher, "action-id", action_id);
            hash_labeled(hasher, "schema", schema);
            hash_labeled(hasher, "spec-digest", spec_digest_blake3);
        }
        ExecutableSourceFacts::Derivation {
            declared_drv_path,
            computed_drv_path,
            drv_name,
            drv_digest_blake3,
        } => {
            hash_labeled(hasher, "source-kind", "derivation");
            hash_labeled(hasher, "declared-drv-path", declared_drv_path);
            hash_labeled(hasher, "computed-drv-path", computed_drv_path);
            hash_labeled(hasher, "drv-name", drv_name);
            hash_labeled(hasher, "drv-digest", drv_digest_blake3);
        }
    }
}

fn hash_labeled(hasher: &mut blake3::Hasher, label: &str, value: &str) {
    hash_labeled_bytes(hasher, label, value.as_bytes());
}

fn hash_labeled_bytes(hasher: &mut blake3::Hasher, label: &str, value: &[u8]) {
    hasher.update(label.as_bytes());
    hasher.update(b":");
    let mut decimal = [0_u8; 20];
    let mut remaining = value.len();
    let mut start = decimal.len();
    loop {
        start -= 1;
        decimal[start] = b'0' + (remaining % 10) as u8;
        remaining /= 10;
        if remaining == 0 {
            break;
        }
    }
    hasher.update(&decimal[start..]);
    hasher.update(b":");
    hasher.update(value);
}

#[cfg(test)]
mod tests {
    use alloc::vec;

    use super::*;

    fn fixture_digest<'a>(outputs: impl IntoIterator<Item = ExpectedOutputFacts<'a>>, request_id: &'a str) -> [u8; 32] {
        let env = BTreeMap::new();
        executable_plan_digest(ExecutablePlanFacts {
            request_id,
            store_prefix: "/mantle/store",
            source: ExecutableSourceFacts::Action {
                action_id: "action-1",
                schema: "mantle-remote-action-v1",
                spec_digest_blake3: "0e384d867379130b704093f3a6a5a8c9ab5fa4bb3d62d476816bc671309abba8",
            },
            system: "x86_64-linux",
            command_args: &[String::from("builtin:fixture"), String::from("--emit")],
            command_env: &env,
            expected_outputs: outputs,
        })
        .unwrap()
    }

    #[test]
    fn request_and_expected_path_are_bound_into_receipt_identity() {
        let with_path = fixture_digest(
            [ExpectedOutputFacts {
                name: "out",
                logical_path: Some("/mantle/store/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa-fixture"),
            }],
            "request-1",
        );
        let without_path = fixture_digest(
            [ExpectedOutputFacts {
                name: "out",
                logical_path: None,
            }],
            "request-1",
        );
        let other_request = fixture_digest(
            [ExpectedOutputFacts {
                name: "out",
                logical_path: Some("/mantle/store/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa-fixture"),
            }],
            "request-2",
        );
        assert_eq!(
            blake3::Hash::from_bytes(with_path).to_hex().as_str(),
            "ffb523566c54d9a2d6ea4be3fbeed00d80df3db4b5a7472f77aca535583dd53b"
        );
        assert_ne!(with_path, without_path);
        assert_ne!(with_path, other_request);
    }

    fn digest_with(
        args: &[String],
        env: &BTreeMap<String, String>,
        outputs: impl IntoIterator<Item = ExpectedOutputFacts<'static>>,
    ) -> Result<[u8; 32], PlanBlocker> {
        executable_plan_digest(ExecutablePlanFacts {
            request_id: "request-1",
            store_prefix: "/mantle/store",
            source: ExecutableSourceFacts::Action {
                action_id: "action-1",
                schema: "mantle-remote-action-v1",
                spec_digest_blake3: "abcd",
            },
            system: "x86_64-linux",
            command_args: args,
            command_env: env,
            expected_outputs: outputs,
        })
    }

    #[test]
    fn malformed_executor_and_oversized_outputs_never_get_a_digest() {
        let mut env = BTreeMap::new();
        let output = ExpectedOutputFacts {
            name: "out",
            logical_path: None,
        };
        assert_eq!(digest_with(&[], &env, [output]), Err(PlanBlocker::BuilderEmpty));
        let args = vec![String::from("builtin:fixture")];
        assert_eq!(digest_with(&args, &env, []), Err(PlanBlocker::ExpectedOutputsEmpty));
        assert_eq!(
            digest_with(&args, &env, core::iter::repeat_n(output, MAX_EXPECTED_OUTPUTS + 1)),
            Err(PlanBlocker::ExpectedOutputCountExceeded)
        );
        env.insert(String::from("KEY"), String::from("x").repeat(MAX_EXECUTOR_ENV_VALUE_BYTES + 1));
        assert_eq!(digest_with(&args, &env, [output]), Err(PlanBlocker::EnvValueExceeded));
    }

    #[test]
    fn ordered_output_receipt_preserves_v1_absence_large_decimal_and_field_order() {
        let first = OutputReceiptFields {
            name: "out",
            logical_path: "/store/out",
            content_digest_blake3: "a",
            artifact_attestation_digest_blake3: "b",
            size_bytes: u64::MAX,
            nar_payload_digest_blake3: None,
            nar_payload_size_bytes: None,
        };
        let mut receipt = OrderedOutputReceipt::new();
        receipt.append(first);
        assert_eq!(
            receipt.finish(),
            *blake3::hash(
                b"output-name:3:outoutput-path:10:/store/outcontent-digest:1:aartifact-digest:1:bsize-bytes:20:18446744073709551615nar-payload-digest:8:<absent>nar-payload-size:8:<absent>"
            )
            .as_bytes(),
        );
        let second = OutputReceiptFields {
            name: "second",
            nar_payload_digest_blake3: Some("c"),
            nar_payload_size_bytes: Some(0),
            ..first
        };
        let mut ordered = OrderedOutputReceipt::new();
        ordered.append(first);
        ordered.append(second);
        let mut reversed = OrderedOutputReceipt::new();
        reversed.append(second);
        reversed.append(first);
        assert_ne!(ordered.finish(), reversed.finish());
    }
}
