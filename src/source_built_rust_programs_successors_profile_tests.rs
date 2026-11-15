use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::path::PathBuf;

use super::*;
use crate::source_built_recipe::ArtifactBinding;
use crate::source_built_recipe::ArtifactKind;
use crate::source_built_recipe::ArtifactRef;
use crate::source_built_recipe::RecipeExitPolicy;
use crate::source_built_recipe::RecipeInvocation;
use crate::source_built_recipe::RecipeOperation;
use crate::source_built_recipe::RecipeStep;
use crate::source_built_recipe::WordPart;
use crate::source_built_recipe::decode_invocation;

fn invocation_fixture() -> (resolve::Sources, RecipeStep, BTreeMap<ArtifactRef, ArtifactBinding>) {
    let sources = resolve::Sources {
        top: "fixture".into(),
        manifests: BTreeMap::new(),
        locks: BTreeMap::new(),
        members: BTreeSet::new(),
        policy: BTreeMap::from([(
            "src/bootstrap/defaults/bootstrap.compiler.toml".into(),
            "[rust]\nrpath = true\n".into(),
        )]),
    };
    let request = ArtifactRef::Produced {
        action_id: "full-source-rust-provider/profile.invocation".into(),
        output_id: "invocation".into(),
    };
    let step = RecipeStep {
        id: "profile.compile".into(),
        inputs: BTreeMap::from([
            ("request".into(), request.clone()),
            ("compiler".into(), ArtifactRef::Produced {
                action_id: "full-source-rust-provider/compiler".into(),
                output_id: "binary".into(),
            }),
        ]),
        outputs: BTreeMap::new(),
        operation: RecipeOperation::Exec {
            executable_input: "compiler".into(),
            invocation_input: Some("request".into()),
            arguments: Vec::new(),
            environment: BTreeMap::new(),
            runtime_mounts: BTreeMap::new(),
            cwd_input: None,
            cwd_output: None,
            stdin_input: None,
            output_seeds: BTreeMap::new(),
            command_state: None,
            guard: None,
            merge_stderr: false,
            exit_policy: RecipeExitPolicy::Exact { code: 0 },
        },
    };
    let bindings = BTreeMap::from([(request.clone(), ArtifactBinding {
        reference: request,
        path: PathBuf::from("unused-in-memory-request"),
        kind: ArtifactKind::Text,
        bytes_max: 65536,
    })]);
    (sources, step, bindings)
}

#[test]
fn original_rpath_profile_survives_strict_deferred_invocation_admission() {
    const RPATH: &str = "-Clink-args=-Wl,-rpath,$ORIGIN/../lib";
    let (sources, step, bindings) = invocation_fixture();
    for version in ["1.90.0", "1.94.1"] {
        let profile = select(&sources, version, "std-initial", false).unwrap();
        let mut invocation: RecipeInvocation = serde_json::from_value(serde_json::json!({
            "arguments": profile.arguments,
            "environment": {},
        }))
        .unwrap();
        let decoded = decode_invocation(&step, &serde_json::to_vec(&invocation).unwrap(), &bindings).unwrap();
        let index = decoded
            .arguments
            .iter()
            .position(|word| matches!(word.parts.as_slice(), [WordPart::LiteralData { text }] if text.starts_with("-Clink-args=-Wl,-rpath,")))
            .expect("the original rpath data must survive admission byte-for-byte");
        assert_eq!(decoded.arguments[index].parts, vec![WordPart::LiteralData { text: RPATH.into() }]);
        invocation.arguments[index].parts = vec![WordPart::Literal { text: RPATH.into() }];
        assert!(decode_invocation(&step, &serde_json::to_vec(&invocation).unwrap(), &bindings).is_err());
    }
}
