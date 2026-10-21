//! Original gcc-generator-base-v4 installation and relocated compiler smoke.
//! Source scripts are installed as source data. Their assembler/linker behavior
//! is expanded into the actual independent GCC/binutils leaves, never executed
//! by a shell or a process-launching replacement driver.
use std::collections::BTreeMap;

use super::FILE_MAX;
use super::Gcc4LinkSeam;
use super::Gcc4LinkSeamInvocation;
use super::append_gcc4_link_seam;
use super::bounded;
use super::error;
use super::input;
use super::literal;
use super::predecessor;
use super::produced;
use crate::errors::RunError;
use crate::source_built_native_programs::NativeNodeProgram;
use crate::source_built_native_programs::NativeNodeRequest;
use crate::source_built_native_programs::STAGE;
use crate::source_built_native_programs::phases::GccInvocation;
use crate::source_built_native_programs::phases::GccPhase;
use crate::source_built_native_programs::phases::GccResponseInvocation;
use crate::source_built_native_programs::phases::append_gcc_request_invocation;
use crate::source_built_native_programs::phases::append_gcc_response_phases;
use crate::source_built_native_programs::phases::gcc_phase_output;
use crate::source_built_recipe::ArtifactKind;
use crate::source_built_recipe::ArtifactRef;
use crate::source_built_recipe::RecipeExitPolicy;
use crate::source_built_recipe::RecipeInvocation;
use crate::source_built_recipe::RecipeOperation;
use crate::source_built_recipe::RecipeOutput;
use crate::source_built_recipe::RecipePreparation;
use crate::source_built_recipe::RecipeStep;
use crate::source_built_recipe::RecipeWord;
use crate::source_built_recipe::SourceRecipe;
use crate::source_built_recipe::WordPart;

const ORIGINAL: &str = include_str!("../bootstrap/gcc-generator-base.ncl");
const GCC_EXEC: &str = "gcc/libexec/gcc/x86_64-unknown-linux-musl/4.0.4";
const GCC_LIB: &str = "gcc/lib/gcc/x86_64-unknown-linux-musl/4.0.4";
type Files = BTreeMap<String, ArtifactRef>;
fn exact() -> RecipeExitPolicy {
    RecipeExitPolicy::Exact { code: 0 }
}
fn out(path: &str) -> RecipeWord {
    RecipeWord {
        parts: vec![WordPart::Output {
            name: "result".into(),
            path: path.into(),
        }],
    }
}
fn runtime(root: &str, path: &str) -> RecipeWord {
    RecipeWord {
        parts: vec![WordPart::Runtime {
            mount: root.into(),
            path: path.into(),
        }],
    }
}
fn runtime_flag(flag: &str, root: &str, path: &str) -> RecipeWord {
    RecipeWord {
        parts: vec![
            WordPart::Literal { text: flag.into() },
            WordPart::Runtime {
                mount: root.into(),
                path: path.into(),
            },
            WordPart::DirectorySeparator {},
        ],
    }
}
fn dep<'a>(request: &'a NativeNodeRequest<'_, '_>, name: &str) -> Result<(String, &'a NativeNodeProgram), RunError> {
    let suffix = format!("-{name}.drv");
    let mut candidates = request.dependencies.iter().filter(|(path, _)| path.ends_with(&suffix));
    let (path, program) = candidates
        .next()
        .ok_or_else(|| error(format!("generator bundle lacks original dependency {name}")))?;
    if candidates.next().is_some() {
        return Err(error(format!("ambiguous generator bundle original dependency {name}")));
    }
    let prefix = request
        .logical_outputs
        .get(&format!("{path}:out"))
        .ok_or_else(|| error("generator dependency original output path is missing"))?;
    Ok((prefix.clone(), *program))
}
fn member(program: &NativeNodeProgram, path: &str) -> Result<ArtifactRef, RunError> {
    let reference = program
        .members
        .get("out")
        .and_then(|members| members.get(path))
        .ok_or_else(|| error(format!("original generator dependency lacks installed member {path}")))?;
    if !matches!(reference, ArtifactRef::Produced { .. }) {
        return Err(error("generator dependency installed member is not Produced"));
    }
    Ok(reference.clone())
}
fn source_block(start: &str) -> Result<String, RunError> {
    let (_, tail) = ORIGINAL
        .split_once(start)
        .ok_or_else(|| error("original generator bundle source block is missing"))?;
    if tail.contains(start) {
        return Err(error("ambiguous original generator bundle source block"));
    }
    let (body, _) = tail
        .split_once("\nEOF\n")
        .ok_or_else(|| error("unterminated original generator bundle source block"))?;
    Ok(format!("{body}\n"))
}
fn recipe(program: &mut NativeNodeProgram) -> SourceRecipe {
    SourceRecipe {
        schema: crate::source_built_recipe::SCHEMA.into(),
        stage_id: STAGE.into(),
        preparation: RecipePreparation {
            tool: "original gcc-generator-base source planner".into(),
            provenance: "original bundle recipe, original GCC4 specs and collect2, source-built dependency members"
                .into(),
        },
        steps: std::mem::take(&mut program.steps),
        timeout_groups: std::mem::take(&mut program.timeout_groups),
        roots: Vec::new(),
        exports: BTreeMap::new(),
        semantic_roots: BTreeMap::new(),
    }
}

struct Bundle<'a, 'b, 'c> {
    request: &'a NativeNodeRequest<'b, 'c>,
    program: NativeNodeProgram,
    tree: Option<ArtifactRef>,
    assertions: Vec<ArtifactRef>,
    members: Files,
}
impl Bundle<'_, '_, '_> {
    fn id(&self, suffix: &str) -> String {
        format!("{}.generator-base.{suffix}", self.request.id)
    }
    fn project_compiler(
        &mut self,
        program: &NativeNodeProgram,
        path: &str,
        suffix: &str,
        cp: ArtifactRef,
    ) -> Result<ArtifactRef, RunError> {
        let tree = program
            .outputs
            .get("out")
            .cloned()
            .ok_or_else(|| error("original GCC4 dependency has no complete output tree"))?;
        let id = self.id(suffix);
        let mut inputs = Files::from([("source".into(), tree)]);
        for (index, reference) in self.assertions.drain(..).enumerate() {
            inputs.insert(format!("assertion-{index}"), reference);
        }
        let reference = super::exec(
            &mut self.program,
            id,
            cp,
            inputs,
            vec![input("source", path), out("")],
            ArtifactKind::Executable,
        );
        self.members.insert(format!("gcc/{path}"), reference.clone());
        Ok(reference)
    }
    fn change(
        &mut self,
        suffix: &str,
        executable: ArtifactRef,
        mut inputs: Files,
        arguments: Vec<RecipeWord>,
    ) -> ArtifactRef {
        let id = self.id(suffix);
        inputs.insert("executable".into(), executable);
        let seeds = if let Some(tree) = &self.tree {
            inputs.insert("prior".into(), tree.clone());
            BTreeMap::from([("result".into(), "prior".into())])
        } else {
            BTreeMap::new()
        };
        for (index, reference) in self.assertions.drain(..).enumerate() {
            inputs.insert(format!("assertion-{index}"), reference);
        }
        self.program.steps.push(RecipeStep {
            id: id.clone(),
            inputs,
            outputs: BTreeMap::from([
                (
                    "result".into(),
                    bounded(ArtifactKind::Tree, self.request.request.proof.resource_bounds.disk_bytes_max),
                ),
                ("stdout".into(), bounded(ArtifactKind::Text, FILE_MAX)),
                ("stderr".into(), bounded(ArtifactKind::Text, FILE_MAX)),
                ("exit-status".into(), bounded(ArtifactKind::Text, 4)),
            ]),
            operation: RecipeOperation::Exec {
                executable_input: "executable".into(),
                arguments,
                environment: BTreeMap::new(),
                runtime_mounts: BTreeMap::new(),
                cwd_input: None,
                cwd_output: Some("result".into()),
                output_seeds: seeds,
                guard: None,
                stdin_input: None,
                merge_stderr: false,
                invocation_input: None,
                exit_policy: exact(),
                command_state: None,
            },
        });
        let tree = produced(&id, "result");
        self.tree = Some(tree.clone());
        tree
    }
    fn check(&mut self, suffix: &str, reference: ArtifactRef, path: &str, flag: &str) -> Result<(), RunError> {
        let id = self.id(suffix);
        self.program.steps.push(RecipeStep {
            id: id.clone(),
            inputs: Files::from([
                (
                    "test".into(),
                    predecessor(self.request, "stagex-transition:member:coreutils-stage/runtime/output/bin/test")?,
                ),
                ("file".into(), reference),
            ]),
            outputs: BTreeMap::from([
                ("stdout".into(), bounded(ArtifactKind::Text, FILE_MAX)),
                ("stderr".into(), bounded(ArtifactKind::Text, FILE_MAX)),
                ("exit-status".into(), bounded(ArtifactKind::Text, 4)),
            ]),
            operation: RecipeOperation::Exec {
                executable_input: "test".into(),
                arguments: vec![literal(flag), input("file", path)],
                environment: BTreeMap::new(),
                runtime_mounts: BTreeMap::new(),
                cwd_input: None,
                cwd_output: None,
                output_seeds: BTreeMap::new(),
                guard: None,
                stdin_input: None,
                merge_stderr: false,
                invocation_input: None,
                exit_policy: exact(),
                command_state: None,
            },
        });
        self.assertions.push(produced(&id, "exit-status"));
        Ok(())
    }
    fn text(&mut self, suffix: &str, text: String) -> ArtifactRef {
        let id = self.id(suffix);
        self.program.steps.push(RecipeStep {
            id: id.clone(),
            inputs: Files::new(),
            outputs: BTreeMap::from([("text".into(), bounded(ArtifactKind::Text, FILE_MAX))]),
            operation: RecipeOperation::WriteText {
                output: "text".into(),
                text,
            },
        });
        produced(&id, "text")
    }
    fn executable_text(&mut self, suffix: &str, text: ArtifactRef, chmod: ArtifactRef) -> ArtifactRef {
        let id = self.id(suffix);
        self.program.steps.push(RecipeStep {
            id: id.clone(),
            inputs: Files::from([("chmod".into(), chmod), ("source".into(), text)]),
            outputs: BTreeMap::from([
                ("result".into(), bounded(ArtifactKind::Text, FILE_MAX)),
                ("stdout".into(), bounded(ArtifactKind::Text, FILE_MAX)),
                ("stderr".into(), bounded(ArtifactKind::Text, FILE_MAX)),
            ]),
            operation: RecipeOperation::Exec {
                executable_input: "chmod".into(),
                arguments: vec![literal("755"), out("")],
                environment: BTreeMap::new(),
                runtime_mounts: BTreeMap::new(),
                cwd_input: None,
                cwd_output: None,
                output_seeds: BTreeMap::from([("result".into(), "source".into())]),
                guard: None,
                stdin_input: None,
                merge_stderr: false,
                invocation_input: None,
                exit_policy: exact(),
                command_state: None,
            },
        });
        produced(&id, "result")
    }
    fn stagex_tree(&mut self, suffix: &str, entries: &[(&str, &str)]) -> Result<ArtifactRef, RunError> {
        let id = self.id(suffix);
        let mut inputs = Files::new();
        let mut paths = BTreeMap::new();
        for (index, (path, key)) in entries.iter().enumerate() {
            let name = format!("member-{index}");
            let reference = predecessor(self.request, key)?;
            inputs.insert(name.clone(), reference.clone());
            paths.insert((*path).into(), name);
            self.members.insert(format!("{suffix}/{path}"), reference);
        }
        self.program.steps.push(RecipeStep {
            id: id.clone(),
            inputs,
            outputs: BTreeMap::from([(
                "tree".into(),
                bounded(ArtifactKind::Tree, self.request.request.proof.resource_bounds.disk_bytes_max),
            )]),
            operation: RecipeOperation::AssembleTree {
                output: "tree".into(),
                entries: paths,
            },
        });
        Ok(produced(&id, "tree"))
    }
}

pub(crate) fn lower(request: &NativeNodeRequest<'_, '_>) -> Result<NativeNodeProgram, RunError> {
    let (_, gcc) = dep(request, "gcc-4.0.4-musl-pass4-v7")?;
    let (_, musl) = dep(request, "musl-1.1.24-gcc-pass4-v2")?;
    let (_, binutils) = dep(request, "binutils-2.30-gcc-pass4-v17")?;
    let (_, make) = dep(request, "make-3.82-tcc")?;
    let (_, stage0) = dep(request, "stage0-posix")?;
    let own_drv = request.node.drv_path.to_absolute_path_with_prefix(&request.request.proof.logical_store_prefix);
    let prefix = request
        .logical_outputs
        .get(&format!("{own_drv}:out"))
        .ok_or_else(|| error("generator bundle original output path is missing"))?;
    let mut b = Bundle {
        request,
        program: NativeNodeProgram {
            steps: Vec::new(),
            timeout_groups: Vec::new(),
            outputs: BTreeMap::new(),
            members: BTreeMap::new(),
            phase_tools: BTreeMap::new(),
            files: BTreeMap::new(),
        },
        tree: None,
        assertions: Vec::new(),
        members: Files::new(),
    };
    let cp = predecessor(request, "stagex-transition:member:coreutils-stage/runtime/output/bin/cp")?;
    let chmod = predecessor(request, "stagex-transition:member:coreutils-stage/runtime/output/bin/chmod")?;
    let mkdir = predecessor(request, "stagex-transition:member:coreutils-stage/runtime/output/bin/mkdir")?;
    let sed = b.stagex_tree("sed", &[("bin/sed", "stagex-transition:sed-4.0.9")])?;
    let diffutils = b.stagex_tree("diffutils", &[
        ("bin/cmp", "stagex-transition:diffutils-cmp-2.7"),
        ("bin/diff", "stagex-transition:diffutils-diff-2.7"),
    ])?;
    let gcc_tree = gcc
        .outputs
        .get("out")
        .cloned()
        .ok_or_else(|| error("original GCC4 dependency has no output tree"))?;
    b.check("require-gcc", gcc_tree.clone(), "bin/gcc", "-x")?;
    b.check("require-cc1", gcc_tree, "libexec/gcc/x86_64-unknown-linux-musl/4.0.4/cc1", "-x")?;
    for (label, reference, flag) in [
        ("libc", member(musl, "lib/libc.a")?, "-s"),
        ("make", member(make, "bin/make")?, "-x"),
    ] {
        b.check(&format!("require-{label}"), reference, "", flag)?;
    }
    for tool in ["as", "ld", "ar", "ranlib", "readelf"] {
        b.check(&format!("require-{tool}"), member(binutils, &format!("bin/{tool}"))?, "", "-x")?;
    }
    b.check("require-sed", sed.clone(), "bin/sed", "-x")?;
    b.check("require-diff", diffutils.clone(), "bin/diff", "-x")?;
    let mut archives = request
        .sources
        .values()
        .filter(|source| source.original_output.ends_with("-automake-1.15.1-config-sub-src"));
    let archive = archives
        .next()
        .ok_or_else(|| error("generator bundle requires original automake-1.15.1-config-sub-src"))?;
    if archives.next().is_some() {
        return Err(error("ambiguous original Automake config.sub source"));
    }
    let config_sub = ["lib/config.sub", "automake-1.15.1/lib/config.sub"]
        .into_iter()
        .find(|path| archive.preparation_path.join(path).is_file())
        .ok_or_else(|| error("missing original Automake 1.15.1 lib/config.sub"))?;
    b.check("require-config-sub", archive.artifact.clone(), config_sub, "-s")?;
    let gcc_driver = b.project_compiler(gcc, "bin/gcc", "select-gcc", cp.clone())?;
    let gcc_cc1 =
        b.project_compiler(gcc, "libexec/gcc/x86_64-unknown-linux-musl/4.0.4/cc1", "select-cc1", cp.clone())?;
    let collect_tool =
        b.project_compiler(gcc, "libexec/gcc/x86_64-unknown-linux-musl/4.0.4/collect2", "select-collect2", cp.clone())?;
    for (destination, dependency) in [
        ("gcc", gcc),
        ("musl", musl),
        ("binutils", binutils),
        ("make", make),
        ("stage0", stage0),
    ] {
        let tree = dependency
            .outputs
            .get("out")
            .cloned()
            .ok_or_else(|| error("original generator dependency has no out tree"))?;
        b.change(&format!("copy-{destination}"), cp.clone(), Files::from([("source".into(), tree)]), vec![
            literal("-R"),
            input("source", ""),
            out(destination),
        ]);
        if let Some(members) = dependency.members.get("out") {
            for (path, reference) in members {
                b.members.entry(format!("{destination}/{path}")).or_insert_with(|| reference.clone());
            }
        }
    }
    for (destination, tree) in [("sed", sed), ("diffutils", diffutils)] {
        b.change(&format!("copy-{destination}"), cp.clone(), Files::from([("source".into(), tree)]), vec![
            literal("-R"),
            input("source", ""),
            out(destination),
        ]);
    }
    b.change("gcc-writable", chmod.clone(), Files::new(), vec![literal("-R"), literal("u+w"), out("gcc")]);
    for name in ["as", "ld"] {
        let template = source_block(&format!("      $BB cat > \"$out/gcc/bin/{name}\" <<EOF\n"))?;
        let text = template
            .replace("$out", prefix)
            .replace("$TARGET", "x86_64-unknown-linux-musl")
            .replace("$GCC_VERSION", "4.0.4")
            .replace("\\$", "$");
        let text = b.text(&format!("{name}-source"), text);
        let text = b.executable_text(&format!("{name}-mode"), text, chmod.clone());
        b.change(&format!("install-{name}-source"), cp.clone(), Files::from([("source".into(), text.clone())]), vec![
            input("source", ""),
            out(&format!("gcc/bin/{name}")),
        ]);
        b.members.insert(format!("gcc/bin/{name}"), text);
    }
    let tree = b.tree.clone().ok_or_else(|| error("generator bundle installation produced no tree"))?;
    for (suffix, path, flag) in [
        ("gcc", "gcc/bin/gcc", "-x"),
        ("libc", "musl/lib/libc.a", "-s"),
        ("ld", "binutils/bin/ld", "-x"),
        ("make", "make/bin/make", "-x"),
    ] {
        b.check(&format!("installed-{suffix}"), tree.clone(), path, flag)?;
    }
    let smoke =
        b.text("relocated-smoke-source", source_block("      $BB cat > \"$WORK/relocated-smoke.c\" <<'EOF'\n")?);
    let target = b.text(
        "relocated-smoke-target",
        "Original specs-query output pathname; the actual linker owns its executable output.\n".into(),
    );
    let mounts = BTreeMap::from([(prefix.clone(), "bundle".into())]);
    let mut inputs = Files::from([
        ("bundle".into(), tree),
        ("unit.c".into(), smoke),
        ("target".into(), target),
    ]);
    for (index, reference) in b.assertions.drain(..).enumerate() {
        inputs.insert(format!("assertion-{index}"), reference);
    }
    let environment = BTreeMap::from([
        ("MANTLE_GCC_ASSEMBLER".into(), runtime(prefix, "binutils/bin/as")),
        ("MANTLE_GCC_LINKER".into(), runtime(prefix, "binutils/bin/ld")),
        ("MANTLE_GCC_LINK_LIBC".into(), runtime(prefix, "musl/lib/libc.a")),
        ("MANTLE_GCC_LINK_LIBM".into(), runtime(prefix, "musl/lib/libm.a")),
        ("MANTLE_GCC_LINK_CRT".into(), runtime(prefix, "musl/lib")),
    ]);
    let maps = || {
        let mut maps = inputs
            .keys()
            .map(|name| (format!("/inputs/{name}"), format!("/inputs/{name}")))
            .collect::<BTreeMap<_, _>>();
        maps.insert(prefix.clone(), prefix.clone());
        maps
    };
    let phase = |executable: ArtifactRef,
                 image: RecipeWord,
                 extra: Files,
                 outputs: BTreeMap<String, RecipeOutput>,
                 path_mappings: BTreeMap<String, String>| GccPhase {
        executable,
        expected_image: image,
        inputs: extra,
        outputs,
        path_mappings,
        runtime_mounts: mounts.clone(),
        environment: BTreeMap::new(),
        output_seeds: BTreeMap::new(),
        cwd_output: None,
        guard: None,
        exit_policy: exact(),
        merge_stderr: false,
        invocation_input: None,
    };
    let id = b.id("relocated-smoke");
    let disk = request.request.proof.resource_bounds.disk_bytes_max;
    let mut cc_map = maps();
    cc_map.insert("/tmp".into(), "/outputs/temps".into());
    let mut cc1 = phase(
        gcc_cc1,
        runtime(prefix, &format!("{GCC_EXEC}/cc1")),
        Files::new(),
        BTreeMap::from([("temps".into(), bounded(ArtifactKind::Tree, disk))]),
        cc_map,
    );
    cc1.cwd_output = Some("temps".into());
    let mut as_map = maps();
    as_map.insert("/tmp".into(), "/outputs/temps".into());
    let mut assembler = phase(
        member(binutils, "bin/as")?,
        runtime(prefix, "gcc/bin/as"),
        Files::from([("assembly".into(), gcc_phase_output(STAGE, &id, 0, "temps"))]),
        BTreeMap::from([("temps".into(), bounded(ArtifactKind::Tree, disk))]),
        as_map,
    );
    assembler.cwd_output = Some("temps".into());
    assembler.output_seeds.insert("temps".into(), "assembly".into());
    let mut collect_map = maps();
    collect_map.insert("/tmp".into(), "/inputs/objects".into());
    // The original bundle explicitly normalizes absolute predecessor libgcc
    // archives. Bind that file-only policy before collect2's query; no prior
    // compiler prefix or unbound executable becomes authority.
    for (identity, original_prefix) in request.logical_outputs {
        if !identity.contains("-gcc-4.0.4-") {
            continue;
        }
        for archive in ["libgcc.a", "libgcc_eh.a"] {
            collect_map.insert(
                format!("{original_prefix}/lib/gcc/x86_64-unknown-linux-musl/4.0.4/{archive}"),
                format!("{prefix}/{GCC_LIB}/libgcc.a"),
            );
        }
    }
    let objects = gcc_phase_output(STAGE, &id, 1, "temps");
    let mut collect = phase(
        collect_tool.clone(),
        runtime(prefix, &format!("{GCC_EXEC}/collect2")),
        Files::from([("objects".into(), objects.clone())]),
        BTreeMap::from([("stdout".into(), bounded(ArtifactKind::Text, FILE_MAX))]),
        collect_map,
    );
    collect.environment.insert("MANTLE_COLLECT2_PHASE_QUERY".into(), literal("1"));
    collect.environment.insert("MANTLE_COLLECT2_PHASE_LINKER".into(), runtime(prefix, "gcc/bin/ld"));
    let args = vec![
        runtime_flag("-B", prefix, "gcc/bin"),
        runtime_flag("-B", prefix, GCC_EXEC),
        runtime_flag("-B", prefix, GCC_LIB),
        RecipeWord {
            parts: vec![WordPart::Literal { text: "-I".into() }, WordPart::Runtime {
                mount: prefix.clone(),
                path: "musl/include".into(),
            }],
        },
        literal("-static"),
        input("unit.c", ""),
        literal("-o"),
        input("target", ""),
        literal("-###"),
    ];
    let driver = gcc_driver;
    let command = b.text(
        "relocated-smoke-request",
        serde_json::to_string(&RecipeInvocation {
            arguments: args,
            environment: environment.clone(),
            cwd: None,
            argv0: Some(runtime(prefix, "gcc/bin/gcc")),
            executable_path: None,
            workspace_mounts: BTreeMap::new(),
            readonly_input_roots: Vec::new(),
            process_attributes: None,
            descriptors: None,
        })
        .map_err(|cause| error(format!("original generator smoke request: {cause}")))?,
    );
    let mut query_inputs = inputs.clone();
    query_inputs.insert("original-invocation".into(), command);
    let mut r = recipe(&mut b.program);
    let query = append_gcc_request_invocation(
        &mut r,
        GccInvocation {
            id: id.clone(),
            driver: driver.clone(),
            splitter: request.gcc_splitter.clone(),
            inputs: query_inputs,
            arguments: Vec::new(),
            environment: BTreeMap::new(),
            runtime_mounts: mounts.clone(),
            cwd_input: None,
            phases: vec![cc1, assembler, collect],
        },
        "original-invocation".into(),
    )?;
    b.program.steps = r.steps;
    b.program.timeout_groups = r.timeout_groups;
    inputs.insert("objects".into(), objects);
    let adapted = append_gcc4_link_seam(request, &mut b.program, Gcc4LinkSeamInvocation {
        id: format!("{id}.linker-seam"),
        query: query.phase_outputs[2]["stdout"].clone(),
        seam: Gcc4LinkSeam::GeneratorBase,
        inputs: inputs.clone(),
        expected_image: runtime(prefix, "gcc/bin/ld"),
        linker_image: runtime(prefix, "binutils/bin/ld"),
        crt_directory: runtime(prefix, "musl/lib"),
        libc: runtime(prefix, "musl/lib/libc.a"),
        libm: runtime(prefix, "musl/lib/libm.a"),
        libgcc: runtime(prefix, &format!("{GCC_LIB}/libgcc.a")),
        runtime_supplement: literal(""),
        compatibility_object: literal(""),
        runtime_mounts: mounts.clone(),
        guard: None,
        exit_policy: exact(),
    })?;
    let mut ld_map = inputs
        .keys()
        .map(|name| (format!("/inputs/{name}"), format!("/inputs/{name}")))
        .collect::<BTreeMap<_, _>>();
    ld_map.insert(prefix.clone(), prefix.clone());
    ld_map.insert("/inputs/target".into(), "/outputs/result".into());
    let linker = phase(
        member(binutils, "bin/ld")?,
        runtime(prefix, "binutils/bin/ld"),
        Files::new(),
        BTreeMap::from([("result".into(), bounded(ArtifactKind::Executable, FILE_MAX))]),
        ld_map,
    );
    let mut r = recipe(&mut b.program);
    let linked = append_gcc_response_phases(&mut r, GccResponseInvocation {
        id: format!("{id}.ld"),
        splitter: request.gcc_splitter.clone(),
        inputs,
        log: adapted.query,
        environment,
        cwd_input: None,
        phases: vec![linker],
    })?;
    b.program.steps = r.steps;
    b.program.timeout_groups = r.timeout_groups;
    let run_id = b.id("relocated-smoke-run");
    b.program.steps.push(RecipeStep {
        id: run_id.clone(),
        inputs: Files::from([("executable".into(), linked.phase_outputs[0]["result"].clone())]),
        outputs: BTreeMap::from([
            ("stdout".into(), bounded(ArtifactKind::Text, FILE_MAX)),
            ("stderr".into(), bounded(ArtifactKind::Text, FILE_MAX)),
            ("exit-status".into(), bounded(ArtifactKind::Text, 4)),
        ]),
        operation: RecipeOperation::Exec {
            executable_input: "executable".into(),
            arguments: Vec::new(),
            environment: BTreeMap::new(),
            runtime_mounts: BTreeMap::new(),
            cwd_input: None,
            cwd_output: None,
            output_seeds: BTreeMap::new(),
            guard: None,
            stdin_input: None,
            merge_stderr: false,
            invocation_input: None,
            exit_policy: exact(),
            command_state: None,
        },
    });
    b.assertions.extend([produced(&run_id, "exit-status"), adapted.outcome]);
    b.change("config-sub-directory", mkdir, Files::new(), vec![literal("-p"), out("share/automake")]);
    let config_id = b.id("config-sub-copy");
    let config = super::exec(
        &mut b.program,
        config_id,
        cp.clone(),
        Files::from([("source".into(), archive.artifact.clone())]),
        vec![input("source", config_sub), out("")],
        ArtifactKind::Text,
    );
    let config = b.executable_text("config-sub-mode", config, chmod);
    let tree = b.change("install-config-sub", cp, Files::from([("source".into(), config.clone())]), vec![
        input("source", ""),
        out("share/automake/config.sub"),
    ]);
    b.members.insert("share/automake/config.sub".into(), config);
    b.program.outputs.insert("out".into(), tree);
    b.program.members.insert("out".into(), b.members);
    b.program.phase_tools.insert("gcc".into(), driver);
    b.program.phase_tools.insert("collect2".into(), collect_tool);
    if let Some(tool) = make.phase_tools.get("make") {
        b.program.phase_tools.insert("make".into(), tool.clone());
    }
    Ok(b.program)
}
