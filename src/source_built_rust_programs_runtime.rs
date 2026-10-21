//! Exact original musl runtime normalization from rust_source_provider.rs.
//! These are cold object/archive/objcopy leaves, not preproduced runtime files.
use std::collections::BTreeMap;

use crate::errors::RunError;
use crate::source_built_fixed_point::SourceAuthorityRole;
use crate::source_built_program_authoring::AuthoringRequest;
use crate::source_built_recipe::ArtifactKind;
use crate::source_built_recipe::ArtifactRef;
use crate::source_built_recipe::RecipeExitPolicy;
use crate::source_built_recipe::RecipeOperation;
use crate::source_built_recipe::RecipeOutput;
use crate::source_built_recipe::RecipeStep;
use crate::source_built_recipe::RecipeWord;
use crate::source_built_recipe::SourceRecipe;
use crate::source_built_recipe::WordPart;

const STAGE: &str = "full-source-rust-provider";
// Pending wiring in the source-only preparation path (Cairn prove-source-built-mantle-fixed-point
// I3).
#[allow(dead_code)]
pub(crate) const RUNTIME_PATH: &str = "/mantle-rust-runtime";
const LFS_SOURCE: &str = "cairn/rust/musl-lfs-compat.c";
const ATOMIC_SOURCE: &str = "cairn/rust/mantle-libatomic-shim.c";
const GCC_LIB: &str = "lib/gcc/x86_64-unknown-linux-musl/10.5.0";
fn error(text: impl Into<String>) -> RunError {
    RunError::Build(format!("Rust runtime source authoring: {}", text.into()))
}
fn lit(text: impl Into<String>) -> RecipeWord {
    RecipeWord {
        parts: vec![WordPart::Literal { text: text.into() }],
    }
}
fn input(name: &str, path: &str) -> RecipeWord {
    RecipeWord {
        parts: vec![WordPart::Input {
            name: name.into(),
            path: path.into(),
        }],
    }
}
fn output(name: &str) -> RecipeWord {
    RecipeWord {
        parts: vec![WordPart::Output {
            name: name.into(),
            path: String::new(),
        }],
    }
}
fn product(id: &str, name: &str) -> ArtifactRef {
    ArtifactRef::Produced {
        action_id: format!("{STAGE}/{id}"),
        output_id: name.into(),
    }
}
fn spec(kind: ArtifactKind) -> RecipeOutput {
    RecipeOutput {
        kind,
        bytes_max: 256 * 1024 * 1024,
    }
}
fn native(request: &AuthoringRequest<'_>, role: &str) -> Result<ArtifactRef, RunError> {
    match request.predecessor_exports.get(&format!("full-source-native-provider:{role}")) {
        Some(reference @ ArtifactRef::Produced { .. }) => Ok(reference.clone()),
        _ => Err(error(format!("missing individually produced {role}"))),
    }
}
fn command_outputs(name: &str, kind: ArtifactKind) -> BTreeMap<String, RecipeOutput> {
    BTreeMap::from([
        (name.into(), spec(kind)),
        ("stdout".into(), RecipeOutput {
            kind: ArtifactKind::Text,
            bytes_max: 16 * 1024 * 1024,
        }),
        ("stderr".into(), RecipeOutput {
            kind: ArtifactKind::Text,
            bytes_max: 16 * 1024 * 1024,
        }),
    ])
}
fn exec(
    program: &mut SourceRecipe,
    id: &str,
    executable: ArtifactRef,
    mut inputs: BTreeMap<String, ArtifactRef>,
    arguments: Vec<RecipeWord>,
    kind: ArtifactKind,
) -> ArtifactRef {
    inputs.insert("executable".into(), executable);
    program.steps.push(RecipeStep {
        id: id.into(),
        inputs,
        outputs: command_outputs("artifact", kind),
        operation: RecipeOperation::Exec {
            invocation_input: None,
            merge_stderr: false,
            executable_input: "executable".into(),
            arguments,
            environment: BTreeMap::new(),
            runtime_mounts: BTreeMap::new(),
            cwd_input: None,
            cwd_output: None,
            output_seeds: BTreeMap::new(),
            command_state: None,
            guard: None,
            stdin_input: None,
            exit_policy: RecipeExitPolicy::Exact { code: 0 },
        },
    });
    product(id, "artifact")
}
fn raw_constant<'a>(source: &'a str, name: &str) -> Result<&'a str, RunError> {
    let anchor = format!("const {name}: &str = r#\"");
    let (before, tail) =
        source.split_once(&anchor).ok_or_else(|| error(format!("original runtime source lacks {name}")))?;
    if before.contains(&anchor) || tail.contains(&anchor) {
        return Err(error("ambiguous original runtime source constant"));
    }
    tail.split_once("\"#;")
        .map(|(body, _)| body)
        .ok_or_else(|| error("unterminated original runtime source constant"))
}
pub(crate) fn source_files(request: &AuthoringRequest<'_>) -> Result<BTreeMap<String, String>, RunError> {
    let root = request
        .source_roots
        .get(&SourceAuthorityRole::MantleSource)
        .ok_or_else(|| error("missing original Mantle source authority"))?;
    let path = root.join("src/rust_source_provider.rs");
    let source = std::fs::read_to_string(&path).map_err(|e| error(format!("{}: {e}", path.display())))?;
    let capacity = source
        .split_once("const FIRST_STAGE_TARGET_MUSL_TLS_KEY_CAPACITY: u32 = ")
        .and_then(|(_, tail)| tail.split_once(';'))
        .map(|(value, _)| value.replace('_', ""))
        .ok_or_else(|| error("missing original TLS key capacity"))?;
    let capacity: u32 = capacity.parse().map_err(|_| error("invalid original TLS key capacity"))?;
    let lfs = format!(
        "{}#define MANTLE_PTHREAD_TLS_KEY_CAPACITY {capacity}u\n\n{}",
        raw_constant(&source, "FIRST_STAGE_TARGET_MUSL_LFS_COMPAT_HEADER_TEXT")?,
        raw_constant(&source, "FIRST_STAGE_TARGET_MUSL_LFS_COMPAT_BODY_TEXT")?
    );
    Ok(BTreeMap::from([
        (LFS_SOURCE.into(), lfs),
        (ATOMIC_SOURCE.into(), raw_constant(&source, "FIRST_STAGE_TARGET_MUSL_LIBATOMIC_SOURCE_TEXT")?.into()),
    ]))
}

pub(crate) fn append(
    request: &AuthoringRequest<'_>,
    program: &mut SourceRecipe,
    selector: ArtifactRef,
) -> Result<BTreeMap<String, ArtifactRef>, RunError> {
    let mut members = BTreeMap::new();
    let sysroot = native(request, "native.sysroot")?;
    let provider = native(request, "native.provider")?;
    for basename in [
        "crt1.o",
        "Scrt1.o",
        "rcrt1.o",
        "crti.o",
        "crtn.o",
        "libgcc.a",
        "libgcc_eh.a",
    ] {
        let selected = exec(
            program,
            &format!("rust-runtime.select-{basename}"),
            selector.clone(),
            BTreeMap::from([("tree".into(), sysroot.clone())]),
            vec![
                lit("--select"),
                input("tree", &format!("x86_64-linux-musl/lib/{basename}")),
                output("artifact"),
            ],
            if basename.ends_with(".a") {
                ArtifactKind::Archive
            } else {
                ArtifactKind::Object
            },
        );
        members.insert(basename.to_string(), selected);
    }
    members.insert("libunwind.a".into(), members["libgcc_eh.a"].clone());
    members.insert("libgcc_s.a".into(), members["libgcc.a"].clone());
    members.insert("libc.so".into(), native(request, "native.loader")?);
    members.insert("libgcc_s.so.1".into(), native(request, "native.libgcc-s")?);
    for basename in ["crtbeginS.o", "crtendS.o"] {
        let selected = exec(
            program,
            &format!("rust-runtime.select-{basename}"),
            selector.clone(),
            BTreeMap::from([("tree".into(), provider.clone())]),
            vec![
                lit("--select"),
                input("tree", &format!("{GCC_LIB}/{basename}")),
                output("artifact"),
            ],
            ArtifactKind::Object,
        );
        members.insert(basename.into(), selected);
    }
    let normalized = exec(
        program,
        "rust-runtime.normalize-crtbeginS",
        native(request, "native.objcopy")?,
        BTreeMap::from([("original".into(), members["crtbeginS.o"].clone())]),
        vec![
            lit("--remove-section"),
            lit(".init_array"),
            lit("--remove-section"),
            lit(".rela.init_array"),
            lit("--remove-section"),
            lit(".fini_array"),
            lit("--remove-section"),
            lit(".rela.fini_array"),
            input("original", ""),
            output("artifact"),
        ],
        ArtifactKind::Object,
    );
    members.insert("crtbeginS.o".into(), normalized);
    for (id, path, basename, largefile) in [
        ("lfs", LFS_SOURCE, "musl-lfs-compat.o", true),
        ("atomic", ATOMIC_SOURCE, "mantle-libatomic-shim.o", false),
    ] {
        let mut flags = vec![lit("-fPIC"), lit("-fno-asynchronous-unwind-tables")];
        if largefile {
            flags.push(lit("-D_LARGEFILE64_SOURCE"));
        }
        let object = super::translation::append_c_object(
            request,
            program,
            &format!("rust-runtime.{id}"),
            super::helper_source_ref(path),
            flags,
        )?;
        members.insert(basename.into(), object);
    }
    let object_tree_id = "rust-runtime.atomic-object-name";
    program.steps.push(RecipeStep {
        id: object_tree_id.into(),
        inputs: BTreeMap::from([("object".into(), members["mantle-libatomic-shim.o"].clone())]),
        outputs: BTreeMap::from([("tree".into(), spec(ArtifactKind::Tree))]),
        operation: RecipeOperation::AssembleTree {
            output: "tree".into(),
            entries: BTreeMap::from([("mantle-libatomic-shim.o".into(), "object".into())]),
        },
    });
    let archive_id = "rust-runtime.atomic-archive";
    let archive = exec(
        program,
        archive_id,
        native(request, "native.ar")?,
        BTreeMap::from([("objects".into(), product(object_tree_id, "tree"))]),
        vec![
            lit("rcs"),
            output("artifact"),
            input("objects", "mantle-libatomic-shim.o"),
        ],
        ArtifactKind::Archive,
    );
    // Original ranlib updates its archive in a separately writable seeded tree.
    let tree_id = "rust-runtime.atomic-before-ranlib";
    program.steps.push(RecipeStep {
        id: tree_id.into(),
        inputs: BTreeMap::from([("archive".into(), archive)]),
        outputs: BTreeMap::from([("tree".into(), spec(ArtifactKind::Tree))]),
        operation: RecipeOperation::AssembleTree {
            output: "tree".into(),
            entries: BTreeMap::from([("libatomic.a".into(), "archive".into())]),
        },
    });
    let ranlib_id = "rust-runtime.atomic-ranlib";
    program.steps.push(RecipeStep {
        id: ranlib_id.into(),
        inputs: BTreeMap::from([
            ("executable".into(), native(request, "native.ranlib")?),
            ("prior".into(), product(tree_id, "tree")),
        ]),
        outputs: command_outputs("tree", ArtifactKind::Tree),
        operation: RecipeOperation::Exec {
            invocation_input: None,
            merge_stderr: false,
            executable_input: "executable".into(),
            arguments: vec![RecipeWord {
                parts: vec![WordPart::Output {
                    name: "tree".into(),
                    path: "libatomic.a".into(),
                }],
            }],
            environment: BTreeMap::new(),
            runtime_mounts: BTreeMap::new(),
            cwd_input: None,
            cwd_output: None,
            output_seeds: BTreeMap::from([("tree".into(), "prior".into())]),
            command_state: None,
            guard: None,
            stdin_input: None,
            exit_policy: RecipeExitPolicy::Exact { code: 0 },
        },
    });
    let atomic = exec(
        program,
        "rust-runtime.select-libatomic",
        selector,
        BTreeMap::from([("tree".into(), product(ranlib_id, "tree"))]),
        vec![lit("--select"), input("tree", "libatomic.a"), output("artifact")],
        ArtifactKind::Archive,
    );
    members.insert("libatomic.a".into(), atomic);
    let mut inputs = BTreeMap::new();
    let mut entries = BTreeMap::new();
    for (ordinal, (basename, member)) in members.iter().enumerate() {
        let name = format!("member-{ordinal}");
        inputs.insert(name.clone(), member.clone());
        entries.insert(basename.clone(), name);
        program.exports.insert(format!("rust-runtime.member:{basename}"), member.clone());
    }
    program.steps.push(RecipeStep {
        id: "rust-runtime.tree".into(),
        inputs,
        outputs: BTreeMap::from([("tree".into(), spec(ArtifactKind::Tree))]),
        operation: RecipeOperation::AssembleTree {
            output: "tree".into(),
            entries,
        },
    });
    let tree = product("rust-runtime.tree", "tree");
    program.exports.insert("rust-runtime".into(), tree.clone());
    members.insert("tree".into(), tree);
    Ok(members)
}
