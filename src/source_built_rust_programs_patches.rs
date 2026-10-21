//! Original selected musl-host source repairs, materialized only by cold leaves.
//! The native author compiles `HELPER_SOURCE` once; this fragment consumes that
//! exact Produced executable and never prepares patched/configured snapshots.
use std::collections::BTreeMap;

use crate::errors::RunError;
use crate::source_built_program_authoring::AuthoringRequest;
use crate::source_built_recipe::ArtifactKind;
use crate::source_built_recipe::ArtifactRef;
use crate::source_built_recipe::RecipeExitPolicy;
use crate::source_built_recipe::RecipeOperation;
use crate::source_built_recipe::RecipeOutput;
use crate::source_built_recipe::RecipeStep;
use crate::source_built_recipe::RecipeWord;
use crate::source_built_recipe::WordPart;

pub(crate) const HELPER_PATH: &str = "mantle-source-authoring/rust-patches.c";
pub(crate) const HELPER_SOURCE: &str = include_str!("source_built_rust_programs_patches.c");
pub(crate) const METADATA_PATH: &str = "mantle-source-authoring/mrustc-metadata.hpp";
pub(crate) const METADATA_SOURCE: &str = include_str!("source_built_rust_programs_mrustc_metadata.h");
pub(crate) const PROC_MACRO_PATH: &str = "mantle-source-authoring/mrustc-proc-macro.hpp";
pub(crate) const PROC_MACRO_SOURCE: &str = include_str!("source_built_rust_programs_mrustc_proc_macro.h");
const STAGE: &str = "full-source-rust-provider";

pub(crate) struct Fragment {
    pub steps: Vec<RecipeStep>,
    pub files: BTreeMap<String, String>,
    pub exports: BTreeMap<String, ArtifactRef>,
}

/// All tree references retain their authenticated archive top-level component.
/// FirstRust190 takes the ORIGINAL mrustc extraction, not the normalized tree:
/// its original patch is consumed as source by the processless transformer.
pub(crate) enum PatchSelection<'a> {
    Mrustc012 {
        /// Exact cold normalized runtime, mounted at the source-owned namespace.
        runtime: ArtifactRef,
    },
    FirstRust190 {
        mrustc_source: ArtifactRef,
    },
    Successor {
        version: &'a str,
    },
}

fn error(message: impl Into<String>) -> RunError {
    RunError::Build(format!("original Rust patch authoring: {}", message.into()))
}
fn literal(text: impl Into<String>) -> RecipeWord {
    RecipeWord {
        parts: vec![WordPart::Literal { text: text.into() }],
    }
}
fn input(name: &str) -> RecipeWord {
    RecipeWord {
        parts: vec![WordPart::Input {
            name: name.into(),
            path: String::new(),
        }],
    }
}
fn require_produced(reference: &ArtifactRef, label: &str) -> Result<(), RunError> {
    if matches!(reference, ArtifactRef::Produced { .. }) {
        Ok(())
    } else {
        Err(error(format!("{label} must be an exact Produced artifact")))
    }
}

/// Materialize the original selected source transformations before compiling a
/// stage. Runtime/library/configuration producers remain separate from source
/// repair; the compiler link/env/overlay source-mode repairs are parent-owned.
pub(crate) fn author(
    _request: &AuthoringRequest<'_>,
    source_tree: ArtifactRef,
    transformer: ArtifactRef,
    selection: PatchSelection<'_>,
) -> Result<Fragment, RunError> {
    require_produced(&source_tree, "original extracted source tree")?;
    require_produced(&transformer, "processless source transformer")?;
    let mut inputs = BTreeMap::from([("original".into(), source_tree), ("transformer".into(), transformer)]);
    let mut extra_arguments = Vec::new();
    let mut runtime_mounts = BTreeMap::new();
    let (mode, version, policy) = match selection {
        PatchSelection::Mrustc012 { runtime } => {
            require_produced(&runtime, "original normalized runtime")?;
            inputs.insert("rust-runtime".into(), runtime);
            runtime_mounts.insert("/mantle-rust-runtime".into(), "rust-runtime".into());
            inputs.insert("metadata-source".into(), super::helper_source_ref(METADATA_PATH));
            inputs.insert("proc-macro-source".into(), super::helper_source_ref(PROC_MACRO_PATH));
            extra_arguments.extend([input("metadata-source"), input("proc-macro-source")]);
            ("mrustc", "0.12.0", RecipeWord {
                parts: vec![WordPart::Runtime {
                    mount: "/mantle-rust-runtime".into(),
                    path: String::new(),
                }],
            })
        }
        PatchSelection::FirstRust190 { mrustc_source } => {
            require_produced(&mrustc_source, "original mrustc patch tree")?;
            inputs.insert("original-mrustc".into(), mrustc_source);
            ("first-rust", "1.90.0", input("original-mrustc"))
        }
        PatchSelection::Successor { version } => {
            if !["1.91.1", "1.92.0", "1.93.1", "1.94.1"].contains(&version) {
                return Err(error(format!("version {version} is outside the selected original route")));
            }
            ("successor", version, literal("-"))
        }
    };
    let id = format!("patches-{mode}-{version}");
    let patched = ArtifactRef::Produced {
        action_id: format!("{STAGE}/{id}"),
        output_id: "patched".into(),
    };
    let helper = super::helper_source_ref(HELPER_PATH);
    let mut arguments = vec![
        literal(mode),
        literal(version),
        input("original"),
        RecipeWord {
            parts: vec![WordPart::Output {
                name: "patched".into(),
                path: String::new(),
            }],
        },
        policy,
    ];
    arguments.extend(extra_arguments);
    Ok(Fragment {
        steps: vec![RecipeStep {
            id,
            inputs,
            outputs: BTreeMap::from([
                ("patched".into(), RecipeOutput {
                    kind: ArtifactKind::Tree,
                    bytes_max: 32 * 1024 * 1024 * 1024,
                }),
                ("stdout".into(), RecipeOutput {
                    kind: ArtifactKind::Text,
                    bytes_max: 16 * 1024 * 1024,
                }),
                ("stderr".into(), RecipeOutput {
                    kind: ArtifactKind::Text,
                    bytes_max: 16 * 1024 * 1024,
                }),
            ]),
            operation: RecipeOperation::Exec {
                invocation_input: None,
                merge_stderr: false,
                executable_input: "transformer".into(),
                arguments,
                environment: BTreeMap::new(),
                runtime_mounts,
                guard: None,
                cwd_input: None,
                cwd_output: None,
                stdin_input: None,
                output_seeds: BTreeMap::new(),
                command_state: None,
                exit_policy: RecipeExitPolicy::Exact { code: 0 },
            },
        }],
        files: BTreeMap::from([
            (HELPER_PATH.into(), HELPER_SOURCE.into()),
            (METADATA_PATH.into(), METADATA_SOURCE.into()),
            (PROC_MACRO_PATH.into(), PROC_MACRO_SOURCE.into()),
        ]),
        exports: BTreeMap::from([("patched.source".into(), patched), ("helper.source".into(), helper)]),
    })
}
