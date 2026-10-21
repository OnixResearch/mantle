//! Original mrustc C production, strict deferred-command decoding and native leaves.
//! mrustc's `.rlib` is an empty marker plus sibling `.hir` and `.rlib.o`, NOT
//! an ar archive. Keep those exact original names when assembling dependency trees.
use std::collections::BTreeMap;
use std::collections::BTreeSet;

use crate::errors::RunError;
use crate::source_built_native_programs::phases::GccInvocation;
use crate::source_built_native_programs::phases::GccPhase;
use crate::source_built_native_programs::phases::GccResponseInvocation;
use crate::source_built_native_programs::phases::append_gcc_invocation;
use crate::source_built_native_programs::phases::append_gcc_response_phases;
use crate::source_built_program_authoring::AuthoringRequest;
use crate::source_built_recipe::ArtifactKind;
use crate::source_built_recipe::ArtifactRef;
use crate::source_built_recipe::RecipeExitPolicy;
use crate::source_built_recipe::RecipeInvocation;
use crate::source_built_recipe::RecipeOperation;
use crate::source_built_recipe::RecipeOutput;
use crate::source_built_recipe::RecipePreparation;
use crate::source_built_recipe::RecipeStep;
use crate::source_built_recipe::RecipeWord;
use crate::source_built_recipe::RecipeWorkspaceMount;
use crate::source_built_recipe::SourceRecipe;
use crate::source_built_recipe::WordPart;
use crate::source_built_recipe::relative;

#[path = "source_built_rust_programs_macros.rs"]
mod macros;

/// Static step capacity of the immutable macro controller (8/slot + 6).
/// This is a resource calculation, never a source expansion-count prediction.
pub(crate) fn macro_reservation_slots_max(remaining_steps: usize) -> Result<u32, RunError> {
    let slots = remaining_steps
        .checked_sub(6)
        .ok_or_else(|| error("macro controller needs six fixed recipe steps before any continuation reservation"))?
        / 8;
    if slots == 0 {
        return Err(error("no macro continuation slot fits the remaining immutable recipe capacity"));
    }
    u32::try_from(slots).map_err(|_| error("macro reservation capacity exceeds u32"))
}

pub(crate) const TRANSLATION_HELPER_SOURCE: &str = include_str!("source_built_rust_programs_translation.c");
const STAGE: &str = "full-source-rust-provider";
const MAX: u64 = 2 * 1024 * 1024 * 1024;
const GCC_TARGET: &str = "x86_64-unknown-linux-musl";
const GCC_VERSION: &str = "10.5.0";

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum TranslationKind {
    Rlib,
    Bin,
    ProcMacro,
    // Pending wiring in the source-only preparation path (Cairn prove-source-built-mantle-fixed-point I3).
    #[allow(dead_code)]
    Dylib,
    BuildScript,
}
impl TranslationKind {
    fn name(self) -> &'static str {
        match self {
            Self::Rlib => "rlib",
            Self::Bin => "bin",
            Self::ProcMacro => "proc-macro",
            Self::Dylib => "dylib",
            Self::BuildScript => "build-script",
        }
    }
    fn metadata(self) -> bool {
        matches!(self, Self::Rlib | Self::ProcMacro)
    }
}

/// One original crate translation, not a minicargo process or emitted shell.
/// `arguments` are the source-selected mrustc arguments excluding `-o` and
/// `-C emit-build-command=...`; this author adds those exact deferred controls.
/// All dependency trees retain their original `.rlib/.hir/.o` sibling names.
/// `filename` is one basename, including the original crate suffix/extension.
/// `helper` is an exact Produced compilation of TRANSLATION_HELPER_SOURCE.
/// Original compiler flags and link constituents come only from mrustc output.
pub(crate) struct TranslationInvocation {
    pub id: String,
    pub mrustc: ArtifactRef,
    pub helper: ArtifactRef,
    pub native_runtime: ArtifactRef,
    pub native_controller: ArtifactRef,
    pub native_wrapper_path: String,
    pub logical_output: String,
    pub native_commands_max: usize,
    pub inputs: BTreeMap<String, ArtifactRef>,
    pub arguments: Vec<RecipeWord>,
    pub environment: BTreeMap<String, RecipeWord>,
    pub runtime_mounts: BTreeMap<String, String>,
    pub generated: BTreeMap<String, ArtifactRef>,
    pub cwd_input: Option<String>,
    pub filename: String,
    pub kind: TranslationKind,
    /// Frames decoded from actual preceding build-script stdout. The source
    /// compiler loads them at process entry before parsing arguments or env!.
    pub metadata_arguments: Option<RecipeWord>,
    pub metadata_environment: Option<RecipeWord>,
    /// Source-produced I/O controller, required only for actual proc-macro deps.
    pub macro_helper: Option<ArtifactRef>,
    pub proc_macro_executables: BTreeMap<String, ArtifactRef>,
    /// Authored continuation ceiling; exhaustion is an explicit proof failure.
    pub proc_macro_slots: usize,
}
pub(crate) struct Fragment {
    pub steps: Vec<RecipeStep>,
    pub files: BTreeMap<String, String>,
    pub exports: BTreeMap<String, ArtifactRef>,
}
fn error(s: impl Into<String>) -> RunError {
    RunError::Build(format!("mrustc translation authoring: {}", s.into()))
}
fn lit(s: impl Into<String>) -> RecipeWord {
    RecipeWord {
        parts: vec![WordPart::Literal { text: s.into() }],
    }
}
fn input(n: &str, p: &str) -> RecipeWord {
    RecipeWord {
        parts: vec![WordPart::Input {
            name: n.into(),
            path: p.into(),
        }],
    }
}
fn output(n: &str, p: &str) -> RecipeWord {
    RecipeWord {
        parts: vec![WordPart::Output {
            name: n.into(),
            path: p.into(),
        }],
    }
}
fn text_input(name: &str) -> RecipeWord {
    RecipeWord {
        parts: vec![WordPart::TextInput { name: name.into() }],
    }
}
fn source_data(program: &mut SourceRecipe, id: &str, text: String) -> ArtifactRef {
    program.steps.push(RecipeStep {
        id: id.into(),
        inputs: BTreeMap::new(),
        outputs: BTreeMap::from([("data".into(), artifact(ArtifactKind::Text))]),
        operation: RecipeOperation::WriteText {
            output: "data".into(),
            text,
        },
    });
    product(id, "data")
}
fn attached(prefix: &str, n: &str, p: &str) -> RecipeWord {
    RecipeWord {
        parts: vec![WordPart::Literal { text: prefix.into() }, WordPart::Input {
            name: n.into(),
            path: p.into(),
        }],
    }
}
fn product(id: &str, n: &str) -> ArtifactRef {
    ArtifactRef::Produced {
        action_id: format!("{STAGE}/{id}"),
        output_id: n.into(),
    }
}
fn produced(r: &ArtifactRef) -> Result<(), RunError> {
    if matches!(r, ArtifactRef::Produced { .. }) {
        Ok(())
    } else {
        Err(error("native tools and generated files must have exact Produced authority"))
    }
}
fn artifact(k: ArtifactKind) -> RecipeOutput {
    RecipeOutput {
        kind: k,
        bytes_max: MAX,
    }
}
fn logs() -> BTreeMap<String, RecipeOutput> {
    BTreeMap::from([
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
fn outputs(n: &str, k: ArtifactKind) -> BTreeMap<String, RecipeOutput> {
    let mut o = logs();
    o.insert(n.into(), artifact(k));
    o
}
fn insert(m: &mut BTreeMap<String, ArtifactRef>, n: &str, r: ArtifactRef) -> Result<(), RunError> {
    if let Some(prior) = m.get(n) {
        if prior == &r {
            return Ok(());
        }
        return Err(error(format!("reserved input {n}")));
    }
    m.insert(n.into(), r);
    Ok(())
}
fn predecessor(request: &AuthoringRequest<'_>, n: &str) -> Result<ArtifactRef, RunError> {
    let r = request
        .predecessor_exports
        .get(&format!("full-source-native-provider:{n}"))
        .ok_or_else(|| error(format!("missing native export {n}")))?
        .clone();
    produced(&r)?;
    Ok(r)
}
pub(crate) fn captured_tmp(
    program: &mut SourceRecipe,
    id: &str,
    generated: &BTreeMap<String, ArtifactRef>,
    disk: u64,
) -> Result<Option<ArtifactRef>, RunError> {
    if generated.is_empty() {
        return Ok(None);
    }
    let mut inputs = BTreeMap::new();
    let mut entries = BTreeMap::new();
    for (index, (logical, reference)) in generated.iter().enumerate() {
        produced(reference)?;
        let path = logical
            .strip_prefix("/tmp/")
            .ok_or_else(|| error("captured generated output is outside the original tmp workspace"))?;
        relative(path, false)?;
        let alias = format!("root-{index}");
        inputs.insert(alias.clone(), reference.clone());
        entries.insert(path.to_string(), alias);
    }
    let id = format!("{id}.captured-tmp");
    program.steps.push(RecipeStep {
        id: id.clone(),
        inputs,
        outputs: BTreeMap::from([("tree".into(), RecipeOutput {
            kind: ArtifactKind::Tree,
            bytes_max: disk,
        })]),
        operation: RecipeOperation::AssembleTree {
            output: "tree".into(),
            entries,
        },
    });
    Ok(Some(product(&id, "tree")))
}

pub(crate) fn bind_tmp_step(
    program: &mut SourceRecipe,
    mut step: RecipeStep,
    tmp: ArtifactRef,
    disk: u64,
) -> Result<RecipeStep, RunError> {
    produced(&tmp)?;
    let RecipeOperation::Exec {
        executable_input,
        arguments,
        environment,
        invocation_input,
        runtime_mounts,
        cwd_input,
        cwd_output,
        output_seeds,
        command_state,
        guard,
        stdin_input,
        ..
    } = &mut step.operation
    else {
        return Err(error("captured tmp requires an executable leaf"));
    };
    if invocation_input.is_some() {
        return Err(error("captured tmp binding requires the original typed static invocation"));
    }
    let cwd = match (cwd_input.take(), cwd_output.take()) {
        (Some(name), None) => Some(input(&name, "")),
        (None, Some(name)) => Some(output(&name, "")),
        (None, None) => None,
        _ => return Err(error("captured tmp command has conflicting cwd roots")),
    };
    let invocation = RecipeInvocation {
        arguments: std::mem::take(arguments),
        environment: std::mem::take(environment),
        cwd,
        argv0: None,
        executable_path: None,
        workspace_mounts: BTreeMap::from([("/tmp".into(), RecipeWorkspaceMount {
            output: "generated-tmp".into(),
            path: String::new(),
        })]),
        readonly_input_roots: Vec::new(),
        process_attributes: None,
        descriptors: None,
    };
    insert(&mut step.inputs, "generated-tmp-seed", tmp)?;
    if step
        .outputs
        .insert("generated-tmp".into(), RecipeOutput {
            kind: ArtifactKind::Tree,
            bytes_max: disk,
        })
        .is_some()
        || output_seeds.insert("generated-tmp".into(), "generated-tmp-seed".into()).is_some()
    {
        return Err(error("captured tmp output shadows an existing command output"));
    }
    let frame_id = format!("{}.workspace-invocation", step.id);
    let text = serde_json::to_string(&invocation).map_err(|e| error(format!("captured tmp invocation: {e}")))?;
    program.steps.push(RecipeStep {
        id: frame_id.clone(),
        inputs: BTreeMap::new(),
        outputs: BTreeMap::from([("invocation".into(), RecipeOutput {
            kind: ArtifactKind::Text,
            bytes_max: text.len() as u64,
        })]),
        operation: RecipeOperation::WriteText {
            output: "invocation".into(),
            text,
        },
    });
    insert(&mut step.inputs, "workspace-invocation", product(&frame_id, "invocation"))?;
    *invocation_input = Some("workspace-invocation".into());
    let mut used = std::collections::BTreeSet::from([executable_input.clone(), "workspace-invocation".into()]);
    used.extend(runtime_mounts.values().cloned());
    used.extend(output_seeds.values().cloned());
    used.extend(stdin_input.iter().cloned());
    if let Some(state) = command_state {
        used.extend(state.prior_times_input.iter().cloned());
    }
    if let Some(guard) = guard {
        used.extend(guard.conditions.iter().map(|condition| condition.status_input.clone()));
    }
    for word in invocation.arguments.iter().chain(invocation.environment.values()).chain(invocation.cwd.iter()) {
        for part in &word.parts {
            match part {
                WordPart::Input { name, .. } | WordPart::TextInput { name } | WordPart::TextSlice { name, .. } => {
                    used.insert(name.clone());
                }
                _ => {}
            }
        }
    }
    step.inputs.retain(|name, _| used.contains(name));
    Ok(step)
}

pub(crate) fn sdk_arguments(cxx: bool, runtime: bool) -> Vec<RecipeWord> {
    let mut arguments = vec![
        attached("-B", "phase-tools", ""),
        attached("-B", "native-provider", &format!("lib/gcc/{GCC_TARGET}/{GCC_VERSION}")),
        attached("--sysroot=", "native-sysroot", "x86_64-linux-musl"),
        lit("-isystem"),
        input("native-provider", &format!("lib/gcc/{GCC_TARGET}/{GCC_VERSION}/include")),
        attached("-L", "native-provider", &format!("lib/gcc/{GCC_TARGET}/{GCC_VERSION}")),
    ];
    if runtime {
        arguments.extend([
            attached("-B", "rust-runtime", ""),
            attached("-L", "rust-runtime", ""),
            RecipeWord {
                parts: vec![
                    WordPart::Literal {
                        text: "-Wl,-dynamic-linker,".into(),
                    },
                    WordPart::Runtime {
                        mount: "/mantle-rust-runtime".into(),
                        path: "libc.so".into(),
                    },
                ],
            },
            RecipeWord {
                parts: vec![
                    WordPart::Literal {
                        text: "-Wl,-rpath,".into(),
                    },
                    WordPart::Runtime {
                        mount: "/mantle-rust-runtime".into(),
                        path: String::new(),
                    },
                ],
            },
        ]);
    } else {
        arguments.extend([
            attached("-Wl,-dynamic-linker,", "native-sysroot", "x86_64-linux-musl/lib/libc.so"),
            attached("-Wl,-rpath,", "native-sysroot", "x86_64-linux-musl/lib"),
        ]);
    }
    if cxx {
        for path in [
            format!("include/c++/{GCC_VERSION}"),
            format!("include/c++/{GCC_VERSION}/{GCC_TARGET}"),
            format!("include/c++/{GCC_VERSION}/backward"),
        ] {
            arguments.extend([lit("-isystem"), input("native-provider", &path)]);
        }
    }
    arguments
}

pub(crate) struct Native {
    pub(crate) inputs: BTreeMap<String, ArtifactRef>,
    prefixes: BTreeMap<String, String>,
    pub(crate) gcc: ArtifactRef,
    // Pending wiring in the source-only preparation path (Cairn prove-source-built-mantle-fixed-point I3).
    #[allow(dead_code)]
    pub(crate) gxx: ArtifactRef,
    pub(crate) splitter: ArtifactRef,
    pub(crate) cc1: ArtifactRef,
    // Pending wiring in the source-only preparation path (Cairn prove-source-built-mantle-fixed-point I3).
    #[allow(dead_code)]
    pub(crate) cc1plus: ArtifactRef,
    pub(crate) assembler: ArtifactRef,
    pub(crate) collect2: ArtifactRef,
    pub(crate) ld: ArtifactRef,
    pub(crate) runtime_mounts: BTreeMap<String, String>,
}
impl Native {
    pub(crate) fn new(
        request: &AuthoringRequest<'_>,
        p: &mut SourceRecipe,
        id: &str,
        mut runtime_mounts: BTreeMap<String, String>,
        runtime: Option<ArtifactRef>,
    ) -> Result<Self, RunError> {
        let gcc = predecessor(request, "native.gcc")?;
        let splitter = predecessor(request, "native.gcc-splitter")?;
        let cc1 = predecessor(request, "native.cc1")?;
        let gxx = predecessor(request, "native.gxx")?;
        let cc1plus = predecessor(request, "native.cc1plus")?;
        let assembler = predecessor(request, "native.as")?;
        let collect2 = predecessor(request, "native.collect2-query")?;
        let ld = predecessor(request, "native.ld")?;
        let tree_id = format!("{id}.native-tools");
        p.steps.push(RecipeStep {
            id: tree_id.clone(),
            inputs: BTreeMap::from([
                ("cc1".into(), cc1.clone()),
                ("cc1plus".into(), cc1plus.clone()),
                ("as".into(), assembler.clone()),
                ("collect2".into(), collect2.clone()),
                ("ld".into(), ld.clone()),
            ]),
            outputs: BTreeMap::from([("tree".into(), artifact(ArtifactKind::Tree))]),
            operation: RecipeOperation::AssembleTree {
                output: "tree".into(),
                entries: BTreeMap::from([
                    ("cc1".into(), "cc1".into()),
                    ("cc1plus".into(), "cc1plus".into()),
                    ("as".into(), "as".into()),
                    ("collect2".into(), "collect2".into()),
                    ("ld".into(), "ld".into()),
                ]),
            },
        });
        let mut inputs = BTreeMap::from([("phase-tools".into(), ArtifactRef::Produced {
            action_id: format!("{}/{}", p.stage_id, tree_id),
            output_id: "tree".into(),
        })]);
        for n in ["native.provider", "native.sysroot", "linux.headers"] {
            insert(&mut inputs, &n.replace('.', "-"), predecessor(request, n)?)?;
        }
        if let Some(runtime) = runtime {
            produced(&runtime)?;
            insert(&mut inputs, "rust-runtime", runtime)?;
            if runtime_mounts.get("/mantle-rust-runtime").is_some_and(|name| name != "rust-runtime") {
                return Err(error("runtime mount alias shadows the original Rust runtime"));
            }
            runtime_mounts.insert("/mantle-rust-runtime".into(), "rust-runtime".into());
        }
        let logical = crate::source_built_native_programs::logical_prefixes(request)?;
        let prefixes = BTreeMap::from([
            (logical.provider, "/inputs/native-provider".into()),
            (logical.gcc, "/inputs/native-provider".into()),
            (logical.binutils, "/inputs/native-provider".into()),
            (logical.musl, "/inputs/native-sysroot/x86_64-linux-musl".into()),
        ]);
        Ok(Self {
            inputs,
            prefixes,
            gcc,
            gxx,
            splitter,
            cc1,
            cc1plus,
            assembler,
            collect2,
            ld,
            runtime_mounts,
        })
    }
    pub(crate) fn environment(&self) -> BTreeMap<String, RecipeWord> {
        let runtime = |path: &str| {
            if self.inputs.contains_key("rust-runtime") {
                input("rust-runtime", path)
            } else {
                input("native-sysroot", &format!("x86_64-linux-musl/lib/{path}"))
            }
        };
        BTreeMap::from([
            ("COMPILER_PATH".into(), input("phase-tools", "")),
            ("MANTLE_GCC_ASSEMBLER".into(), input("phase-tools", "as")),
            ("MANTLE_GCC_LINKER".into(), input("phase-tools", "ld")),
            ("MANTLE_GCC_LINK_LIBC".into(), runtime("libc.so")),
            ("MANTLE_GCC_LINK_LIBM".into(), runtime("libc.so")),
            ("MANTLE_GCC_LINK_LIBGCC".into(), runtime("libgcc.a")),
            ("MANTLE_GCC_LINK_LIBGCC_EH".into(), runtime("libgcc_eh.a")),
            ("MANTLE_GCC_LINK_LIBGCC_S".into(), runtime("libgcc_s.so.1")),
            ("MANTLE_GCC_LINK_CRT".into(), runtime("")),
        ])
    }
    pub(crate) fn arguments(&self) -> Vec<RecipeWord> {
        sdk_arguments(false, self.inputs.contains_key("rust-runtime"))
    }
    pub(crate) fn cxx_arguments(&self) -> Vec<RecipeWord> {
        sdk_arguments(true, self.inputs.contains_key("rust-runtime"))
    }
    pub(crate) fn mappings(&self, inputs: &BTreeMap<String, ArtifactRef>) -> BTreeMap<String, String> {
        let mut m = self.prefixes.clone();
        m.extend(inputs.keys().map(|n| {
            let p = format!("/inputs/{n}");
            (p.clone(), p)
        }));
        m
    }
    pub(crate) fn phase(
        &self,
        executable: ArtifactRef,
        image: &str,
        outputs: BTreeMap<String, RecipeOutput>,
        map: BTreeMap<String, String>,
    ) -> GccPhase {
        GccPhase {
            invocation_input: None,
            merge_stderr: false,
            executable,
            expected_image: input("phase-tools", image),
            inputs: BTreeMap::new(),
            outputs,
            path_mappings: map,
            runtime_mounts: self.runtime_mounts.clone(),
            environment: BTreeMap::new(),
            output_seeds: BTreeMap::new(),
            cwd_output: None,
            guard: None,
            exit_policy: RecipeExitPolicy::Exact { code: 0 },
        }
    }
}

pub(crate) fn author(request: &AuthoringRequest<'_>, invocation: TranslationInvocation) -> Result<Fragment, RunError> {
    produced(&invocation.mrustc)?;
    produced(&invocation.helper)?;
    produced(&invocation.native_controller)?;
    relative(&invocation.id, false)?;
    relative(&invocation.filename, false)?;
    relative(&invocation.logical_output, false)?;
    if invocation.filename.contains('/')
        || invocation.filename.contains("..")
        || invocation.logical_output.rsplit('/').next() != Some(invocation.filename.as_str())
        || !invocation.logical_output.contains('/')
    {
        return Err(error("crate output must retain its original directory and exact basename"));
    }
    let wrapper = invocation
        .native_wrapper_path
        .strip_prefix("/inputs/package/")
        .ok_or_else(|| error("deferred CC wrapper must belong to the genuine package world"))?;
    relative(wrapper, false)?;
    for word in &invocation.arguments {
        for part in &word.parts {
            if let WordPart::Literal { text } = part
                && (text == "-o" || text.contains("emit-build-command=") || text.starts_with('@'))
            {
                return Err(error("caller shadows deferred translation controls"));
            }
        }
    }
    if invocation
        .environment
        .keys()
        .any(|n| n == "CC" || n.starts_with("CC_") || n == "MRUSTC_CCACHE" || n.starts_with("MRUSTC_CAIRN_"))
    {
        return Err(error("caller shadows the original deferred compiler selection"));
    }
    let mut p = SourceRecipe {
        schema: crate::source_built_recipe::SCHEMA.into(),
        stage_id: STAGE.into(),
        preparation: RecipePreparation {
            tool: "mrustc-original-deferred-command".into(),
            provenance: "Original mrustc command, original target CC wrapper and original SDK/GCC algorithms".into(),
        },
        steps: Vec::new(),
        roots: Vec::new(),
        exports: BTreeMap::new(),
        semantic_roots: BTreeMap::new(),
        timeout_groups: Vec::new(),
    };
    let disk = request.proof.resource_bounds.disk_bytes_max;
    let package = invocation
        .inputs
        .get("package")
        .ok_or_else(|| error("deferred compiler lacks its original package world"))?
        .clone();
    let cwd_name = invocation
        .cwd_input
        .clone()
        .ok_or_else(|| error("deferred compiler requires its actual source cwd"))?;
    let cwd_tree = invocation.inputs.get(&cwd_name).ok_or_else(|| error("deferred compiler cwd is not bound"))?.clone();
    let runtime = invocation.native_runtime.clone();
    let mut captured = captured_tmp(&mut p, &invocation.id, &invocation.generated, disk)?;
    if captured.is_none() {
        let id = format!("{}.empty-tmp", invocation.id);
        p.steps.push(RecipeStep {
            id: id.clone(),
            inputs: BTreeMap::from([("helper".into(), invocation.helper.clone())]),
            outputs: outputs("tree", ArtifactKind::Tree),
            operation: RecipeOperation::Exec {
                executable_input: "helper".into(),
                invocation_input: None,
                arguments: vec![lit("empty"), output("tree", "")],
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
        });
        captured = Some(product(&id, "tree"));
    }
    let tid = format!("{}.translate", invocation.id);
    let mut inputs = invocation.inputs;
    insert(&mut inputs, "mrustc", invocation.mrustc)?;
    let mut arguments = invocation.arguments;
    arguments.extend([
        lit("-o"),
        output("translation", &invocation.filename),
        lit("-C"),
        RecipeWord {
            parts: vec![
                WordPart::Literal {
                    text: "emit-build-command=".into(),
                },
                WordPart::Output {
                    name: "translation".into(),
                    path: format!("{}-codegen.sh", invocation.filename),
                },
            ],
        },
    ]);
    let mut environment = invocation.environment;
    for (variable, frame) in [
        ("MRUSTC_CAIRN_ARGUMENTS", invocation.metadata_arguments),
        ("MRUSTC_CAIRN_COMPILE_ENV", invocation.metadata_environment),
    ] {
        if let Some(frame) = frame {
            environment.insert(variable.into(), frame);
        }
    }
    for name in ["CC", "CC_x86_64_linux_musl", "CC_x86_64_unknown_linux_musl"] {
        environment.insert(name.into(), input("package", wrapper));
    }
    let compiler = RecipeStep {
        id: tid.clone(),
        inputs,
        outputs: outputs("translation", ArtifactKind::Tree),
        operation: RecipeOperation::Exec {
            invocation_input: None,
            merge_stderr: false,
            executable_input: "mrustc".into(),
            arguments,
            environment,
            runtime_mounts: invocation.runtime_mounts.clone(),
            cwd_input: invocation.cwd_input,
            cwd_output: None,
            output_seeds: BTreeMap::new(),
            command_state: None,
            guard: None,
            stdin_input: None,
            exit_policy: RecipeExitPolicy::Exact { code: 0 },
        },
    };
    let (translation, native_tmp) = if invocation.proc_macro_executables.is_empty() {
        let compiler = bind_tmp_step(&mut p, compiler, captured.expect("seeded actual tmp"), disk)?;
        p.steps.push(compiler);
        (product(&tid, "translation"), product(&tid, "generated-tmp"))
    } else {
        macros::append(
            &mut p,
            &invocation.id,
            compiler,
            invocation
                .macro_helper
                .ok_or_else(|| error("proc-macro dependencies require the source continuation helper"))?,
            &invocation.proc_macro_executables,
            invocation.proc_macro_slots,
            captured,
            disk,
        )?
    };
    let prepare = format!("{}.native-entry", invocation.id);
    let mut declared = outputs("workspace", ArtifactKind::Tree);
    declared.get_mut("workspace").expect("declared workspace").bytes_max = disk;
    let logical_data =
        source_data(&mut p, &format!("{}.native-output-path", invocation.id), invocation.logical_output.clone());
    for name in ["environment", "generated-c", "command", "native-command"] {
        declared.insert(name.into(), artifact(ArtifactKind::Text));
    }
    let mut args = vec![
        lit("prepare"),
        input("translation", ""),
        lit(&invocation.filename),
        lit(invocation.kind.name()),
        input("package", wrapper),
        text_input("logical-output"),
        input("prior-tmp", ""),
        output("workspace", ""),
        output("environment", ""),
        output("generated-c", ""),
        output("command", ""),
        output("native-command", ""),
    ];
    if invocation.kind.metadata() {
        declared.insert("metadata".into(), artifact(ArtifactKind::Object));
        args.push(output("metadata", ""));
    }
    if invocation.kind == TranslationKind::Rlib {
        declared.insert("marker".into(), artifact(ArtifactKind::Text));
        args.push(output("marker", ""));
    }
    p.steps.push(RecipeStep {
        id: prepare.clone(),
        inputs: BTreeMap::from([
            ("helper".into(), invocation.helper.clone()),
            ("package".into(), package.clone()),
            ("logical-output".into(), logical_data.clone()),
            ("translation".into(), translation.clone()),
            ("prior-tmp".into(), native_tmp),
        ]),
        outputs: declared,
        operation: RecipeOperation::Exec {
            executable_input: "helper".into(),
            invocation_input: None,
            arguments: args,
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
    });
    let world = super::llvm::native_command_world(request)?;
    let mut command_inputs = world.inputs;
    insert(&mut command_inputs, "package", package)?;
    insert(&mut command_inputs, "rust-runtime", runtime)?;
    insert(&mut command_inputs, &cwd_name, cwd_tree)?;
    let readonly_inputs = command_inputs.keys().cloned().collect();
    let mut runtime_mounts = world.runtime_mounts;
    for (root, name) in invocation.runtime_mounts {
        if runtime_mounts.insert(root, name.clone()).is_some_and(|prior| prior != name) {
            return Err(error("native runtime alias collision"));
        }
    }
    let quote = |value: &str| format!("'{}'", value.replace('\'', "'\\''"));
    let response = format!("/tmp/{}_cmd.txt", invocation.logical_output);
    let workspace = super::llvm::append_native_source_script(
        request,
        &mut p,
        invocation.native_controller,
        super::llvm::NativeSourceScript {
            id: format!("{}.native", invocation.id),
            script: format!("exec {} @{}\n", quote(&invocation.native_wrapper_path), quote(&response)),
            initial_workspace: product(&prepare, "workspace"),
            initial_cwd: format!("/inputs/{cwd_name}"),
            initial_cwd_input: None,
            original_zero: format!("/tmp/{}-codegen.sh", invocation.logical_output),
            environment: BTreeMap::new(),
            environment_input: Some((product(&prepare, "environment"), String::new())),
            inputs: command_inputs,
            readonly_inputs,
            runtime_mounts,
            inventory: world.inventory,
            commands_max: invocation.native_commands_max,
        },
    )?;
    let collect = format!("{}.native-products", invocation.id);
    let artifact_kind = if invocation.kind == TranslationKind::Rlib {
        ArtifactKind::Text
    } else {
        ArtifactKind::Executable
    };
    let mut declared = outputs("artifact", artifact_kind);
    for name in ["bundle", "generated-tmp"] {
        declared.insert(name.into(), RecipeOutput {
            kind: ArtifactKind::Tree,
            bytes_max: disk,
        });
    }
    p.steps.push(RecipeStep {
        id: collect.clone(),
        inputs: BTreeMap::from([
            ("helper".into(), invocation.helper),
            ("workspace".into(), workspace),
            ("logical-output".into(), logical_data),
        ]),
        outputs: declared,
        operation: RecipeOperation::Exec {
            executable_input: "helper".into(),
            invocation_input: None,
            arguments: vec![
                lit("collect"),
                input("workspace", ""),
                text_input("logical-output"),
                lit(&invocation.filename),
                lit(invocation.kind.name()),
                output("artifact", ""),
                output("bundle", ""),
                output("generated-tmp", ""),
            ],
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
    });
    for (name, reference) in [
        ("artifact", product(&collect, "artifact")),
        ("bundle", product(&collect, "bundle")),
        ("generated-tmp", product(&collect, "generated-tmp")),
        ("translation", translation),
        ("c", product(&prepare, "generated-c")),
        ("command", product(&prepare, "command")),
        ("native-command", product(&prepare, "native-command")),
    ] {
        p.exports.insert(format!("{}.{name}", invocation.id), reference);
    }
    if invocation.kind.metadata() {
        p.exports.insert(format!("{}.metadata", invocation.id), product(&prepare, "metadata"));
    }
    p.exports
        .insert(format!("{}.{}", invocation.id, invocation.kind.name()), product(&collect, "artifact"));
    Ok(Fragment {
        steps: p.steps,
        files: BTreeMap::new(),
        exports: p.exports,
    })
}

fn compile_c(
    program: &mut SourceRecipe,
    n: &Native,
    id: &str,
    source: ArtifactRef,
    include_trees: BTreeMap<String, ArtifactRef>,
    flags: Vec<RecipeWord>,
) -> Result<ArtifactRef, RunError> {
    let mut inputs = n.inputs.clone();
    insert(&mut inputs, "helper-source", source)?;
    let mut arguments = n.arguments();
    arguments.extend(flags);
    arguments.extend([
        lit("-x"),
        lit("c"),
        lit("-S"),
        input("helper-source", ""),
        lit("-o"),
        input("phase-tools", "helper-query-assembly"),
    ]);
    for (name, tree) in include_trees {
        relative(&name, false)?;
        insert(&mut inputs, &name, tree)?;
        arguments.extend([lit("-I"), input(&name, "")]);
    }
    let mut mappings = n.mappings(&inputs);
    mappings.insert("/inputs/phase-tools/helper-query-assembly".into(), "/outputs/assembly".into());
    let expansion = append_gcc_invocation(program, GccInvocation {
        id: format!("{id}.compile"),
        driver: n.gcc.clone(),
        splitter: n.splitter.clone(),
        inputs,
        arguments,
        environment: n.environment(),
        runtime_mounts: BTreeMap::new(),
        cwd_input: None,
        phases: vec![n.phase(n.cc1.clone(), "cc1", outputs("assembly", ArtifactKind::Text), mappings)],
    })?;
    let mut inputs = n.inputs.clone();
    insert(&mut inputs, "assembly", expansion.phase_outputs[0]["assembly"].clone())?;
    let mut arguments = n.arguments();
    arguments.extend([
        lit("-x"),
        lit("assembler"),
        lit("-c"),
        input("assembly", ""),
        lit("-o"),
        input("phase-tools", "helper-query-object"),
    ]);
    let mut mappings = n.mappings(&inputs);
    mappings.insert("/inputs/phase-tools/helper-query-object".into(), "/outputs/object".into());
    let expansion = append_gcc_invocation(program, GccInvocation {
        id: format!("{id}.assemble"),
        driver: n.gcc.clone(),
        splitter: n.splitter.clone(),
        inputs,
        arguments,
        environment: n.environment(),
        runtime_mounts: BTreeMap::new(),
        cwd_input: None,
        phases: vec![n.phase(n.assembler.clone(), "as", outputs("object", ArtifactKind::Object), mappings)],
    })?;
    Ok(expansion.phase_outputs[0]["object"].clone())
}

pub(crate) fn append_c_object(
    request: &AuthoringRequest<'_>,
    program: &mut SourceRecipe,
    id: &str,
    source: ArtifactRef,
    flags: Vec<RecipeWord>,
) -> Result<ArtifactRef, RunError> {
    relative(id, false)?;
    let n = Native::new(request, program, id, BTreeMap::new(), None)?;
    compile_c(program, &n, id, source, BTreeMap::new(), flags)
}

/// Compile an inert admitted helper source with the original native GCC specs.
/// Driver queries, cc1, assembler, collect2 query and ld remain distinct leaves.
pub(crate) fn append_c_helper(
    request: &AuthoringRequest<'_>,
    program: &mut SourceRecipe,
    id: &str,
    source: ArtifactRef,
    include_trees: BTreeMap<String, ArtifactRef>,
) -> Result<ArtifactRef, RunError> {
    relative(id, false)?;
    let n = Native::new(request, program, id, BTreeMap::new(), None)?;
    let object = compile_c(program, &n, id, source, include_trees, vec![lit("-std=c99"), lit("-O2"), lit("-static")])?;
    let mut inputs = n.inputs.clone();
    insert(&mut inputs, "object", object)?;
    let mut arguments = n.arguments();
    arguments.extend([
        lit("-static"),
        input("object", ""),
        lit("-o"),
        input("phase-tools", "helper-query-executable"),
    ]);
    let query_id = format!("{id}.collect2");
    let mut phase = n.phase(n.collect2.clone(), "collect2", logs(), n.mappings(&inputs));
    phase.inputs.insert("query-driver-log".into(), ArtifactRef::Produced {
        action_id: format!("{}/{query_id}.specs", program.stage_id),
        output_id: "stderr".into(),
    });
    phase.environment = BTreeMap::from([
        ("MANTLE_COLLECT2_PHASE_QUERY".into(), lit("1")),
        ("MANTLE_COLLECT2_PHASE_LINKER".into(), input("phase-tools", "ld")),
        ("MANTLE_COLLECT2_PHASE_DRIVER_LOG".into(), input("query-driver-log", "")),
    ]);
    let query = append_gcc_invocation(program, GccInvocation {
        id: query_id,
        driver: n.gcc.clone(),
        splitter: n.splitter.clone(),
        inputs: inputs.clone(),
        arguments,
        environment: n.environment(),
        runtime_mounts: BTreeMap::new(),
        cwd_input: None,
        phases: vec![phase],
    })?;
    let mut mappings = n.mappings(&inputs);
    mappings.insert("/inputs/phase-tools/helper-query-executable".into(), "/outputs/executable".into());
    let expansion = append_gcc_response_phases(program, GccResponseInvocation {
        id: format!("{id}.link"),
        splitter: n.splitter.clone(),
        inputs,
        log: query.phase_outputs[0]["stdout"].clone(),
        environment: n.environment(),
        cwd_input: None,
        phases: vec![n.phase(n.ld.clone(), "ld", outputs("executable", ArtifactKind::Executable), mappings)],
    })?;
    Ok(expansion.phase_outputs[0]["executable"].clone())
}

/// Interpret the untouched compiler command through its original target wrapper.
#[allow(clippy::too_many_arguments)]
pub(crate) fn append_link_response(
    request: &AuthoringRequest<'_>,
    program: &mut SourceRecipe,
    id: &str,
    mut inputs: BTreeMap<String, ArtifactRef>,
    link_work: ArtifactRef,
    native_runtime: ArtifactRef,
    mut runtime_mounts: BTreeMap<String, String>,
    output_name: &str,
    output_kind: ArtifactKind,
    generated: &BTreeMap<String, ArtifactRef>,
    native_cwd: Option<RecipeWord>,
    readonly_input_roots: BTreeSet<String>,
    native_controller: ArtifactRef,
    native_helper: ArtifactRef,
    logical_output: &str,
    commands_max: usize,
) -> Result<ArtifactRef, RunError> {
    for reference in [&link_work, &native_runtime, &native_controller, &native_helper] {
        produced(reference)?;
    }
    relative(id, false)?;
    relative(output_name, false)?;
    relative(logical_output, false)?;
    let cwd = native_cwd.ok_or_else(|| error("original compiler native command lacks an authenticated source cwd"))?;
    let package = inputs
        .get("package")
        .ok_or_else(|| error("native linker lacks its original target wrapper package"))?
        .clone();
    let disk = request.proof.resource_bounds.disk_bytes_max;
    let prior = if let Some(prior) = captured_tmp(program, id, generated, disk)? {
        prior
    } else {
        let empty = format!("{id}.empty-tmp");
        program.steps.push(RecipeStep {
            id: empty.clone(),
            inputs: BTreeMap::from([("helper".into(), native_helper.clone())]),
            outputs: outputs("tree", ArtifactKind::Tree),
            operation: RecipeOperation::Exec {
                executable_input: "helper".into(),
                invocation_input: None,
                arguments: vec![lit("empty"), output("tree", "")],
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
        });
        product(&empty, "tree")
    };
    let logical_data = source_data(program, &format!("{id}.logical-output"), logical_output.into());
    let prepare = format!("{id}.native-entry");
    program.steps.push(RecipeStep {
        id: prepare.clone(),
        inputs: BTreeMap::from([
            ("helper".into(), native_helper.clone()),
            ("package".into(), package),
            ("link-work".into(), link_work),
            ("prior-tmp".into(), prior),
            ("logical-output".into(), logical_data.clone()),
        ]),
        outputs: BTreeMap::from([
            ("workspace".into(), RecipeOutput {
                kind: ArtifactKind::Tree,
                bytes_max: disk,
            }),
            ("entry".into(), artifact(ArtifactKind::Tree)),
            ("stdout".into(), artifact(ArtifactKind::Text)),
            ("stderr".into(), artifact(ArtifactKind::Text)),
            ("exit-status".into(), artifact(ArtifactKind::Text)),
        ]),
        operation: RecipeOperation::Exec {
            executable_input: "helper".into(),
            invocation_input: None,
            arguments: vec![
                lit("link-prepare"),
                input("link-work", ""),
                input("prior-tmp", ""),
                text_input("logical-output"),
                input("package", "target-wrappers/cc"),
                output("workspace", ""),
                output("entry", ""),
                cwd,
            ],
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
    });
    let mut readonly = readonly_input_roots;
    for name in &readonly {
        if !inputs.contains_key(name) {
            return Err(error(format!("native source root {name} lacks its actual input")));
        }
    }
    let world = super::llvm::native_command_world(request)?;
    for (name, reference) in world.inputs {
        insert(&mut inputs, &name, reference)?;
        readonly.insert(name);
    }
    insert(&mut inputs, "rust-runtime", native_runtime)?;
    readonly.insert("rust-runtime".into());
    insert(&mut inputs, "native-entry", product(&prepare, "entry"))?;
    readonly.insert("native-entry".into());
    for (root, name) in world.runtime_mounts {
        if runtime_mounts.insert(root, name.clone()).is_some_and(|previous| previous != name) {
            return Err(error("native original runtime root collision"));
        }
    }
    let workspace = super::llvm::append_native_source_script(
        request,
        program,
        native_controller,
        super::llvm::NativeSourceScript {
            id: format!("{id}.native"),
            script: ". /inputs/native-entry/command.sh\n".into(),
            initial_workspace: product(&prepare, "workspace"),
            initial_cwd: String::new(),
            initial_cwd_input: Some((product(&prepare, "entry"), "cwd".into())),
            original_zero: format!("/tmp/{logical_output}"),
            environment: BTreeMap::new(),
            environment_input: Some((product(&prepare, "entry"), "environment".into())),
            inputs,
            readonly_inputs: readonly,
            runtime_mounts,
            inventory: world.inventory,
            commands_max,
        },
    )?;
    let finish = format!("{id}.native-products");
    let mut declared = outputs(output_name, output_kind);
    declared.insert("generated-tmp".into(), RecipeOutput {
        kind: ArtifactKind::Tree,
        bytes_max: disk,
    });
    program.steps.push(RecipeStep {
        id: finish.clone(),
        inputs: BTreeMap::from([
            ("helper".into(), native_helper),
            ("workspace".into(), workspace),
            ("logical-output".into(), logical_data),
        ]),
        outputs: declared,
        operation: RecipeOperation::Exec {
            executable_input: "helper".into(),
            invocation_input: None,
            arguments: vec![
                lit("link-collect"),
                input("workspace", ""),
                text_input("logical-output"),
                output(output_name, ""),
                output("generated-tmp", ""),
            ],
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
    });
    program.exports.insert(format!("{id}.generated-tmp"), product(&finish, "generated-tmp"));
    Ok(product(&finish, output_name))
}
