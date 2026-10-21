//! Child-free lowering of the original native host-tool source recipes.
//! Compiler argument selection belongs to the original GCC specs and collect2;
//! the host planner never translates driver flags into cc1 or linker flags.
use std::collections::BTreeMap;
use std::path::Path;
use std::path::PathBuf;

use crate::errors::RunError;
use crate::source_built_fixed_point::SourceAuthorityRole;
use crate::source_built_native_programs::NativeLogicalPrefixes;
use crate::source_built_native_programs::NativeNodeProgram;
use crate::source_built_native_programs::NativeNodeRequest;
use crate::source_built_native_programs::NativeSource;
use crate::source_built_native_programs::RECORD_IDENTITY;
use crate::source_built_native_programs::STAGE;
use crate::source_built_native_programs::phases::GccInvocation;
use crate::source_built_native_programs::phases::GccPhase;
use crate::source_built_native_programs::phases::GccResponseInvocation;
use crate::source_built_native_programs::phases::append_gcc_invocation;
use crate::source_built_native_programs::phases::append_gcc_response_phases;
use crate::source_built_native_programs::phases::gcc_phase_output;
use crate::source_built_recipe::ArtifactKind;
use crate::source_built_recipe::ArtifactRef;
use crate::source_built_recipe::RecipeExitPolicy;
use crate::source_built_recipe::RecipeOperation;
use crate::source_built_recipe::RecipeOutput;
use crate::source_built_recipe::RecipePreparation;
use crate::source_built_recipe::RecipeStep;
use crate::source_built_recipe::RecipeWord;
use crate::source_built_recipe::SourceRecipe;
use crate::source_built_recipe::WordPart;

#[path = "source_built_native_programs_host_controller.rs"]
pub(crate) mod controller;
#[path = "source_built_native_programs_host_root.rs"]
pub(crate) mod root;
pub(crate) use controller::ControllerHooks;
pub(crate) use controller::controller_source;
#[path = "source_built_native_programs_host_original.rs"]
mod original;
pub(crate) use original::original_node_prepared;
pub(crate) use original::original_node_with;
#[path = "source_built_native_programs_host_bootstrap.rs"]
mod bootstrap;
#[path = "source_built_native_programs_host_packages.rs"]
mod packages;

const LIMIT: u64 = 512 * 1024 * 1024;
const HELPER_MEMBER: &str = "source-built/native/host-transform.c";
const MAKE_RECIPE_MEMBER: &str = "source-built/native/host-make-original.ncl";
const LINUX_RECIPE_MEMBER: &str = "source-built/native/host-linux-headers-original.ncl";
const MAKE_CONTROL_MEMBER: &str = "source-built/native/host-make-control.c";
const MAKE_CONTROL_HEADER_MEMBER: &str = "source-built/native/host-make-control.h";
const GCC_RUNTIME: &str = "lib/gcc/x86_64-unknown-linux-musl/10.5.0";
const GCC_EXEC: &str = "libexec/gcc/x86_64-unknown-linux-musl/10.5.0";
type Files = BTreeMap<String, ArtifactRef>;

fn fail(message: impl Into<String>) -> RunError {
    RunError::Build(format!("original native host source lowering: {}", message.into()))
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
fn runtime(mount: &str, path: &str) -> RecipeWord {
    RecipeWord {
        parts: vec![WordPart::Runtime {
            mount: mount.into(),
            path: path.into(),
        }],
    }
}
fn runtime_flag(flag: &str, mount: &str, path: &str) -> RecipeWord {
    RecipeWord {
        parts: vec![WordPart::Literal { text: flag.into() }, WordPart::Runtime {
            mount: mount.into(),
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
fn spec(kind: ArtifactKind) -> RecipeOutput {
    RecipeOutput { kind, bytes_max: LIMIT }
}
fn produced(id: &str, name: &str) -> ArtifactRef {
    ArtifactRef::Produced {
        action_id: format!("{STAGE}/{id}"),
        output_id: name.into(),
    }
}
fn source(member: &str) -> ArtifactRef {
    ArtifactRef::ManifestSource {
        role: SourceAuthorityRole::NativeSourceBundle,
        record_identity: RECORD_IDENTITY.into(),
        member: member.into(),
    }
}
fn exact() -> RecipeExitPolicy {
    RecipeExitPolicy::Exact { code: 0 }
}

struct Host<'a, 'b, 'c> {
    request: &'a NativeNodeRequest<'b, 'c>,
    recipe: SourceRecipe,
    prefixes: NativeLogicalPrefixes,
    provider: &'a NativeNodeProgram,
    tool_inputs: Files,
    mounts: BTreeMap<String, String>,
}
impl<'a, 'b, 'c> Host<'a, 'b, 'c> {
    fn new(request: &'a NativeNodeRequest<'b, 'c>) -> Result<Self, RunError> {
        let provider = dependency(request, "-full-source-seed-toolchain.drv")?;
        let prefixes = crate::source_built_native_programs::logical_prefixes(request.request)?;
        if !provider.members.contains_key("out") {
            return Err(fail("normalized provider has no exact out members"));
        }
        let tool_inputs = Files::from([(
            "provider".into(),
            provider.outputs.get("out").cloned().ok_or_else(|| fail("provider out is absent"))?,
        )]);
        let mounts = BTreeMap::from([
            (prefixes.provider.clone(), "provider".into()),
            (prefixes.gcc.clone(), "provider".into()),
        ]);
        Ok(Self {
            request,
            recipe: SourceRecipe {
                schema: crate::source_built_recipe::SCHEMA.into(),
                stage_id: STAGE.into(),
                preparation: RecipePreparation {
                    tool: "original native host source planner".into(),
                    provenance:
                        "original bootstrap recipes, authenticated archives, original GCC specs and source generators"
                            .into(),
                },
                steps: Vec::new(),
                timeout_groups: Vec::new(),
                roots: Vec::new(),
                exports: BTreeMap::new(),
                semantic_roots: BTreeMap::new(),
            },
            prefixes,
            provider,
            tool_inputs,
            mounts,
        })
    }
    fn member(&self, path: &str) -> Result<ArtifactRef, RunError> {
        self.provider
            .members
            .get("out")
            .and_then(|files| files.get(path))
            .cloned()
            .ok_or_else(|| fail(format!("missing exact normalized provider member {path}")))
    }
    fn id(&self, suffix: &str) -> String {
        format!("{}.host.{suffix}", self.request.id)
    }
    fn add(&mut self, suffix: &str, inputs: Files, kind: ArtifactKind, operation: RecipeOperation) -> ArtifactRef {
        let id = self.id(suffix);
        self.recipe.steps.push(RecipeStep {
            id: id.clone(),
            inputs,
            outputs: BTreeMap::from([("result".into(), spec(kind))]),
            operation,
        });
        produced(&id, "result")
    }
    fn exec(
        &mut self,
        suffix: &str,
        executable: ArtifactRef,
        mut inputs: Files,
        arguments: Vec<RecipeWord>,
        kind: ArtifactKind,
        exit_policy: RecipeExitPolicy,
    ) -> ArtifactRef {
        inputs.insert("tool".into(), executable);
        self.add(suffix, inputs, kind, RecipeOperation::Exec {
            executable_input: "tool".into(),
            arguments,
            environment: BTreeMap::from([("LC_ALL".into(), lit("C")), ("SOURCE_DATE_EPOCH".into(), lit("1"))]),
            runtime_mounts: BTreeMap::new(),
            cwd_input: None,
            cwd_output: None,
            output_seeds: BTreeMap::new(),
            command_state: None,
            guard: None,
            stdin_input: None,
            merge_stderr: false,
            invocation_input: None,
            exit_policy,
        })
    }
    fn helper(&mut self) -> Result<ArtifactRef, RunError> {
        let object = self.compile("transform-helper", source(HELPER_MEMBER), "", &[], &[], &[])?;
        self.link("transform-helper", &[object])
    }
    fn compiler_environment(&self) -> BTreeMap<String, RecipeWord> {
        let root = &self.prefixes.provider;
        BTreeMap::from([
            ("LC_ALL".into(), lit("C")),
            ("SOURCE_DATE_EPOCH".into(), lit("1")),
            ("MANTLE_GCC_ASSEMBLER".into(), runtime(root, "bin/as")),
            ("MANTLE_GCC_LINKER".into(), runtime(root, "bin/ld")),
            ("MANTLE_GCC_LINK_LIBC".into(), runtime(root, "x86_64-linux-musl/lib/libc.a")),
            ("MANTLE_GCC_LINK_LIBM".into(), runtime(root, "x86_64-linux-musl/lib/libm.a")),
            ("MANTLE_GCC_LINK_LIBGCC".into(), runtime(root, "x86_64-linux-musl/lib/libgcc.a")),
            ("MANTLE_GCC_LINK_LIBGCC_EH".into(), runtime(root, "x86_64-linux-musl/lib/libgcc_eh.a")),
            ("MANTLE_GCC_LINK_CRT".into(), runtime(root, "x86_64-linux-musl/lib")),
        ])
    }
    fn driver_arguments(&self) -> Vec<RecipeWord> {
        let root = &self.prefixes.provider;
        vec![
            lit("-B"),
            runtime(root, "bin"),
            lit("-B"),
            runtime(root, GCC_RUNTIME),
            runtime_flag("--sysroot=", root, "x86_64-linux-musl"),
            runtime_flag("-Wl,-dynamic-linker,", root, "x86_64-linux-musl/lib/ld-musl-x86_64.so.1"),
            runtime_flag("-Wl,-rpath,", root, "x86_64-linux-musl/lib"),
        ]
    }
    fn maps(&self, inputs: &Files) -> BTreeMap<String, String> {
        let mut maps = BTreeMap::new();
        for key in inputs.keys() {
            maps.insert(format!("/inputs/{key}"), format!("/inputs/{key}"));
        }
        maps.insert(self.prefixes.provider.clone(), "/inputs/provider".into());
        maps.insert(self.prefixes.gcc.clone(), "/inputs/provider".into());
        maps.insert(self.prefixes.musl.clone(), "/inputs/provider/x86_64-linux-musl".into());
        maps
    }
    fn phase(
        &self,
        executable: ArtifactRef,
        image: RecipeWord,
        inputs: Files,
        outputs: BTreeMap<String, RecipeOutput>,
        path_mappings: BTreeMap<String, String>,
    ) -> GccPhase {
        GccPhase {
            executable,
            expected_image: image,
            inputs,
            outputs,
            path_mappings,
            runtime_mounts: self.mounts.clone(),
            environment: BTreeMap::new(),
            output_seeds: BTreeMap::new(),
            cwd_output: None,
            guard: None,
            merge_stderr: false,
            invocation_input: None,
            exit_policy: exact(),
        }
    }
    fn compile(
        &mut self,
        suffix: &str,
        unit: ArtifactRef,
        path: &str,
        flags: &[&str],
        includes: &[&str],
        extra: &[(&str, ArtifactRef)],
    ) -> Result<ArtifactRef, RunError> {
        let id = self.id(&format!("{suffix}.compile"));
        let mut inputs = self.tool_inputs.clone();
        inputs.insert("unit.c".into(), unit);
        let target = self.add(
            &format!("{suffix}.compile-target"),
            Files::new(),
            ArtifactKind::Text,
            RecipeOperation::WriteText {
                output: "result".into(),
                text: "Inert specs-query output target; actual output belongs to its selected phase.\n".into(),
            },
        );
        inputs.insert("phase-target".into(), target);
        for (key, reference) in extra {
            inputs.insert((*key).into(), reference.clone());
        }
        let mut args = self.driver_arguments();
        args.extend(flags.iter().map(|flag| lit(*flag)));
        for include in includes {
            args.push(lit("-I"));
            args.push(input("unit.c", include));
        }
        args.extend([lit("-c"), input("unit.c", path), lit("-o"), input("phase-target", "")]);
        let mut compiler_map = self.maps(&inputs);
        compiler_map.insert("/tmp".into(), "/outputs/temps".into());
        let assembly = gcc_phase_output(STAGE, &id, 0, "temps");
        let mut assembler_map = self.maps(&inputs);
        assembler_map.insert("/tmp".into(), "/inputs/assembly".into());
        assembler_map.insert("/inputs/phase-target".into(), "/outputs/result".into());
        let mut compiler = self.phase(
            self.member(&format!("{GCC_EXEC}/cc1"))?,
            runtime(&self.prefixes.gcc, &format!("{GCC_EXEC}/cc1")),
            Files::new(),
            BTreeMap::from([("temps".into(), spec(ArtifactKind::Tree))]),
            compiler_map,
        );
        compiler.cwd_output = Some("temps".into());
        let phases = vec![
            compiler,
            self.phase(
                self.member("bin/as")?,
                runtime(&self.prefixes.provider, "bin/as"),
                Files::from([("assembly".into(), assembly)]),
                BTreeMap::from([("result".into(), spec(ArtifactKind::Object))]),
                assembler_map,
            ),
        ];
        let driver = self.member("bin/gcc.real")?;
        let environment = self.compiler_environment();
        let expansion = append_gcc_invocation(&mut self.recipe, GccInvocation {
            id,
            driver,
            splitter: self.request.gcc_splitter.clone(),
            inputs,
            arguments: args,
            environment,
            runtime_mounts: self.mounts.clone(),
            cwd_input: None,
            phases,
        })?;
        expansion
            .phase_outputs
            .last()
            .and_then(|outputs| outputs.get("result"))
            .cloned()
            .ok_or_else(|| fail("compiler phases omitted object"))
    }
    fn link(&mut self, suffix: &str, objects: &[ArtifactRef]) -> Result<ArtifactRef, RunError> {
        let id = self.id(&format!("{suffix}.link"));
        let mut inputs = self.tool_inputs.clone();
        let mut args = self.driver_arguments();
        let target =
            self.add(&format!("{suffix}.link-target"), Files::new(), ArtifactKind::Text, RecipeOperation::WriteText {
                output: "result".into(),
                text: "Inert specs-query output target; collect2 selects the actual linker invocation.\n".into(),
            });
        inputs.insert("phase-target".into(), target);
        args.extend([lit("-static"), lit("-o"), input("phase-target", "")]);
        for (index, reference) in objects.iter().enumerate() {
            let key = format!("object-{index}");
            inputs.insert(key.clone(), reference.clone());
            args.push(input(&key, ""));
        }
        let map = self.maps(&inputs);
        let mut collect = self.phase(
            self.member(&format!("{GCC_EXEC}/collect2"))?,
            runtime(&self.prefixes.gcc, &format!("{GCC_EXEC}/collect2")),
            Files::new(),
            BTreeMap::from([("stdout".into(), spec(ArtifactKind::Text))]),
            map,
        );
        collect.environment = BTreeMap::from([
            ("MANTLE_COLLECT2_PHASE_QUERY".into(), lit("1")),
            ("MANTLE_COLLECT2_PHASE_LINKER".into(), runtime(&self.prefixes.provider, "bin/ld")),
        ]);
        let driver = self.member("bin/gcc.real")?;
        let environment = self.compiler_environment();
        let query = append_gcc_invocation(&mut self.recipe, GccInvocation {
            id: id.clone(),
            driver,
            splitter: self.request.gcc_splitter.clone(),
            inputs: inputs.clone(),
            arguments: args,
            environment: environment.clone(),
            runtime_mounts: self.mounts.clone(),
            cwd_input: None,
            phases: vec![collect],
        })?;
        let mut linker_map = self.maps(&inputs);
        linker_map.insert("/inputs/phase-target".into(), "/outputs/result".into());
        let linker = self.phase(
            self.member("bin/ld")?,
            runtime(&self.prefixes.provider, "bin/ld"),
            Files::new(),
            BTreeMap::from([("result".into(), spec(ArtifactKind::Executable))]),
            linker_map,
        );
        let expansion = append_gcc_response_phases(&mut self.recipe, GccResponseInvocation {
            id: format!("{id}.ld"),
            splitter: self.request.gcc_splitter.clone(),
            inputs,
            log: query.phase_outputs[0]["stdout"].clone(),
            environment,
            cwd_input: None,
            phases: vec![linker],
        })?;
        Ok(expansion.phase_outputs[0]["result"].clone())
    }
    fn finish(self, members: Files, files: BTreeMap<String, String>) -> Result<NativeNodeProgram, RunError> {
        let mut inputs = Files::new();
        let mut entries = BTreeMap::new();
        for (index, (path, reference)) in members.iter().enumerate() {
            let key = format!("installed-{index}");
            inputs.insert(key.clone(), reference.clone());
            entries.insert(path.clone(), key);
        }
        let id = self.id("install");
        let tree = produced(&id, "result");
        let mut steps = self.recipe.steps;
        steps.push(RecipeStep {
            id,
            inputs,
            outputs: BTreeMap::from([("result".into(), spec(ArtifactKind::Tree))]),
            operation: RecipeOperation::AssembleTree {
                output: "result".into(),
                entries,
            },
        });
        if self.request.node.derivation.outputs.keys().any(|name| name != "out") {
            return Err(fail("original host output contract changed"));
        }
        Ok(NativeNodeProgram {
            steps,
            timeout_groups: self.recipe.timeout_groups,
            outputs: BTreeMap::from([("out".into(), tree)]),
            members: BTreeMap::from([("out".into(), members)]),
            phase_tools: BTreeMap::new(),
            files,
        })
    }
}
fn dependency<'a>(request: &'a NativeNodeRequest<'_, '_>, suffix: &str) -> Result<&'a NativeNodeProgram, RunError> {
    let mut found = request.dependencies.iter().filter(|(path, _)| path.ends_with(suffix));
    let (_, program) = found.next().ok_or_else(|| fail(format!("missing exact original dependency {suffix}")))?;
    if found.next().is_some() {
        return Err(fail(format!("ambiguous original dependency {suffix}")));
    }
    Ok(program)
}
fn archive<'a>(request: &'a NativeNodeRequest<'_, '_>, suffix: &str) -> Result<&'a NativeSource, RunError> {
    let mut found = request.sources.values().filter(|source| source.original_output.ends_with(suffix));
    let source = found.next().ok_or_else(|| fail(format!("missing authenticated original source {suffix}")))?;
    if found.next().is_some() {
        return Err(fail("ambiguous original source"));
    }
    Ok(source)
}
fn archive_root(source: &NativeSource, nested: &str, probe: &str) -> Result<(PathBuf, String), RunError> {
    if source.preparation_path.join(probe).is_file() {
        return Ok((source.preparation_path.clone(), String::new()));
    }
    if source.preparation_path.join(nested).join(probe).is_file() {
        return Ok((source.preparation_path.join(nested), nested.into()));
    }
    Err(fail(format!("original source root lacks {probe}")))
}
fn helper_files() -> BTreeMap<String, String> {
    BTreeMap::from([(HELPER_MEMBER.into(), include_str!("source_built_native_programs_host_transform.c").into())])
}
fn original_block(recipe: &str, start: &str, end: &str) -> Result<String, RunError> {
    let (_, tail) = recipe.split_once(start).ok_or_else(|| fail("missing original recipe source block"))?;
    if tail.contains(start) {
        return Err(fail("ambiguous original recipe source block"));
    }
    let (body, _) = tail.split_once(end).ok_or_else(|| fail("unterminated original recipe source block"))?;
    Ok(body.into())
}
fn make(request: &NativeNodeRequest<'_, '_>) -> Result<NativeNodeProgram, RunError> {
    let src = archive(request, "-make-4.4.1-full-source-src")?;
    let (_, nested) = archive_root(src, "make-4.4.1", "src/main.c")?;
    let recipe = include_str!("../bootstrap/make-4.4.1-gcc10.ncl");
    let mut h = Host::new(request)?;
    let helper = h.helper()?;
    let transformed = h.exec(
        "make-source",
        helper,
        Files::from([
            ("archive".into(), src.artifact.clone()),
            ("recipe".into(), source(MAKE_RECIPE_MEMBER)),
            ("control".into(), source(MAKE_CONTROL_HEADER_MEMBER)),
            ("unit".into(), source(MAKE_CONTROL_MEMBER)),
        ]),
        vec![
            lit("make-source"),
            input("archive", &nested),
            input("recipe", ""),
            input("control", ""),
            input("unit", ""),
            output("result"),
        ],
        ArtifactKind::Tree,
        exact(),
    );
    let mut control_source = crate::source_built_stagex_programs::shellvm::library_source()?;
    control_source.push_str(&include_str!("source_built_native_programs_host_make_control.c").replace(
        "#include \"mantle_make_control.h\"",
        include_str!("source_built_native_programs_host_make_control.h"),
    ));
    let mut files = helper_files();
    files.insert(MAKE_RECIPE_MEMBER.into(), recipe.into());
    files.insert(MAKE_CONTROL_MEMBER.into(), control_source);
    files.insert(
        MAKE_CONTROL_HEADER_MEMBER.into(),
        include_str!("source_built_native_programs_host_make_control.h").into(),
    );
    let script = request.node.derivation.arguments.get(1).ok_or_else(|| fail("original Make script missing"))?;
    let anchor = "src/guile.c\"";
    if script.matches(anchor).count() != 1 {
        return Err(fail("original Make compilation-list seam changed"));
    }
    let script = script.replacen(anchor, "src/guile.c src/mantle_make_control.c\"", 1);
    original::original_node_prepared_script(
        request,
        Files::from([(src.original_output.clone(), transformed)]),
        Files::new(),
        BTreeMap::from([("out".into(), BTreeMap::from([("bin/make".into(), ArtifactKind::Executable)]))]),
        BTreeMap::from([("make".into(), ("out".into(), "bin/make".into()))]),
        &[],
        h.recipe.steps,
        h.recipe.timeout_groups,
        files,
        Some(script),
    )
}

fn relative_source(nested: &str, path: &str) -> String {
    if nested.is_empty() {
        path.into()
    } else {
        format!("{nested}/{path}")
    }
}
fn read_source(path: &Path) -> Result<String, RunError> {
    std::fs::read_to_string(path).map_err(|e| fail(format!("reading original source {}: {e}", path.display())))
}
fn kbuild_list(text: &str, variable: &str) -> Result<Vec<String>, RunError> {
    let mut result = Vec::new();
    for line in text.replace("\\\n", " ").lines() {
        let line = line.split('#').next().unwrap_or("").trim();
        if let Some(tail) = line.strip_prefix(variable) {
            let tail = tail.trim_start();
            if let Some(values) =
                tail.strip_prefix("+=").or_else(|| tail.strip_prefix(":=")).or_else(|| tail.strip_prefix('='))
            {
                for word in values.split_whitespace() {
                    if word.contains('$') || word.contains('/') || !word.ends_with(".h") {
                        return Err(fail("unexpanded original UAPI Kbuild member"));
                    }
                    result.push(word.into());
                }
            }
        }
    }
    Ok(result)
}

// These are source-level Make conditions, not answers to compiler probes. The
// original headers_install target evaluates them against its source tree and
// the headers generated earlier in that same target.
fn excluded_uapi_headers(root: &Path, generated: &std::collections::BTreeSet<String>) -> Result<Vec<String>, RunError> {
    let text = read_source(&root.join("include/uapi/Kbuild"))?;
    let mut conditions = Vec::new();
    let mut excluded = Vec::new();
    for raw in text.lines() {
        let line = raw.split('#').next().unwrap_or("").trim();
        if line.is_empty() {
            continue;
        }
        if let Some(test) = line.strip_prefix("ifeq ($(wildcard ").and_then(|s| s.strip_suffix("),)")) {
            let path = test.replace("$(SRCARCH)", "x86");
            let absent = if let Some(path) = path.strip_prefix("$(srctree)/") {
                !root.join(path).is_file()
            } else if let Some(path) = path.strip_prefix("$(objtree)/") {
                !generated.contains(path)
            } else {
                return Err(fail("unsupported original UAPI wildcard root"));
            };
            conditions.push(absent);
        } else if line == "endif" {
            conditions.pop().ok_or_else(|| fail("unbalanced original UAPI export condition"))?;
        } else if let Some(values) = line.strip_prefix("no-export-headers +=") {
            for path in values.split_whitespace() {
                if !path.ends_with(".h")
                    || Path::new(path).components().any(|c| !matches!(c, std::path::Component::Normal(_)))
                {
                    return Err(fail("unexpanded original UAPI exclusion"));
                }
                if conditions.iter().all(|active| *active) {
                    excluded.push(path.into());
                }
            }
        } else {
            return Err(fail(format!("unsupported original UAPI export condition: {line}")));
        }
    }
    if !conditions.is_empty() {
        return Err(fail("unterminated original UAPI export condition"));
    }
    Ok(excluded)
}

struct SyscallHeader {
    name: String,
    table: String,
    abis: String,
    offset: String,
    prefix: String,
}

fn syscall_headers(root: &Path, generated: &[String]) -> Result<Vec<SyscallHeader>, RunError> {
    let text = read_source(&root.join("arch/x86/entry/syscalls/Makefile"))?.replace("\\\n", " ");
    let mut variables = BTreeMap::new();
    for line in text.lines() {
        if let Some((name, value)) = line.split_once(":=")
            && let Some(path) = value.trim().strip_prefix("$(src)/")
        {
            variables.insert(name.trim().to_string(), path.to_string());
        }
    }
    let mut headers = Vec::new();
    for name in generated {
        let target = format!("$(uapi)/{name}:");
        let mut fields = BTreeMap::new();
        let mut table = None;
        for line in text.lines().filter_map(|line| line.trim().strip_prefix(&target)) {
            let line = line.trim();
            if let Some((key, value)) = line.split_once(":=") {
                if !matches!(key.trim(), "abis" | "offset" | "prefix") {
                    return Err(fail("unknown original syscall target variable"));
                }
                fields.insert(key.trim(), value.trim());
            } else {
                let source =
                    line.split_whitespace().next().ok_or_else(|| fail("empty original syscall prerequisite"))?;
                let variable = source
                    .strip_prefix("$(")
                    .and_then(|s| s.strip_suffix(')'))
                    .ok_or_else(|| fail("unexpanded original syscall source"))?;
                table = variables.get(variable).cloned();
            }
        }
        let table = table.ok_or_else(|| fail(format!("missing original syscall table for {name}")))?;
        if Path::new(&table).components().any(|c| !matches!(c, std::path::Component::Normal(_)))
            || !table.ends_with(".tbl")
        {
            return Err(fail("invalid original syscall table member"));
        }
        let abis = fields.get("abis").ok_or_else(|| fail("missing original syscall ABI selection"))?.to_string();
        if abis.is_empty() || !abis.bytes().all(|b| b.is_ascii_alphanumeric() || b == b',' || b == b'_') {
            return Err(fail("unexpanded original syscall ABI selection"));
        }
        let offset = fields.get("offset").copied().unwrap_or("").to_string();
        let prefix = fields.get("prefix").copied().unwrap_or("").to_string();
        if !offset.bytes().chain(prefix.bytes()).all(|b| b.is_ascii_alphanumeric() || b == b'_') {
            return Err(fail("unexpanded original syscall macro selection"));
        }
        headers.push(SyscallHeader {
            name: name.clone(),
            table,
            abis,
            offset,
            prefix,
        });
    }
    Ok(headers)
}
fn source_headers(root: &Path, directory: &str) -> Result<Vec<String>, RunError> {
    let mut pending = vec![root.join(directory)];
    let mut result = Vec::new();
    while let Some(path) = pending.pop() {
        for entry in std::fs::read_dir(&path).map_err(|e| fail(format!("reading original header directory: {e}")))? {
            let entry = entry.map_err(|e| fail(e.to_string()))?;
            let path = entry.path();
            let metadata = std::fs::metadata(&path).map_err(|e| fail(e.to_string()))?;
            if metadata.is_dir() {
                pending.push(path);
            } else if path.extension().is_some_and(|extension| extension == "h") {
                let relative = path
                    .strip_prefix(root)
                    .map_err(|e| fail(e.to_string()))?
                    .to_str()
                    .ok_or_else(|| fail("non-UTF8 original UAPI member"))?;
                result.push(relative.into());
            }
        }
    }
    result.sort();
    Ok(result)
}
fn kernel_member(
    h: &mut Host<'_, '_, '_>,
    helper: &ArtifactRef,
    source: &NativeSource,
    nested: &str,
    path: &str,
    suffix: &str,
) -> ArtifactRef {
    h.exec(
        suffix,
        helper.clone(),
        Files::from([("archive".into(), source.artifact.clone())]),
        vec![
            lit("copy"),
            input("archive", &relative_source(nested, path)),
            output("result"),
        ],
        ArtifactKind::Text,
        exact(),
    )
}
fn install_header(
    h: &mut Host<'_, '_, '_>,
    helper: &ArtifactRef,
    unifdef: &ArtifactRef,
    script: &ArtifactRef,
    original: &str,
    header: ArtifactRef,
    index: usize,
) -> ArtifactRef {
    let edited = h.exec(
        &format!("header-{index}.sed"),
        helper.clone(),
        Files::from([("header".into(), header), ("script".into(), script.clone())]),
        vec![
            lit("header-prepare"),
            input("header", ""),
            input("script", ""),
            output("result"),
        ],
        ArtifactKind::Text,
        exact(),
    );
    let mut inputs = h.tool_inputs.clone();
    inputs.insert("header".into(), edited);
    inputs.insert("tool".into(), unifdef.clone());
    let selected = h.add(&format!("header-{index}.unifdef"), inputs, ArtifactKind::Text, RecipeOperation::Exec {
        executable_input: "tool".into(),
        arguments: vec![
            lit("-U__KERNEL__"),
            lit("-D__EXPORTED_HEADERS__"),
            lit("-o"),
            output("result"),
            input("header", ""),
        ],
        environment: BTreeMap::from([("LC_ALL".into(), lit("C"))]),
        runtime_mounts: h.mounts.clone(),
        cwd_input: None,
        cwd_output: None,
        output_seeds: BTreeMap::new(),
        command_state: None,
        guard: None,
        stdin_input: None,
        merge_stderr: false,
        invocation_input: None,
        exit_policy: RecipeExitPolicy::OneOf { codes: vec![0, 1] },
    });
    h.exec(
        &format!("header-{index}.leaks"),
        helper.clone(),
        Files::from([("header".into(), selected), ("script".into(), script.clone())]),
        vec![
            lit("header-check"),
            input("header", ""),
            input("script", ""),
            lit(original),
            output("result"),
        ],
        ArtifactKind::Text,
        exact(),
    )
}
fn linux_headers(request: &NativeNodeRequest<'_, '_>) -> Result<NativeNodeProgram, RunError> {
    let src = archive(request, "-linux-6.6-src")?;
    let (root, nested) = archive_root(src, "linux-6.6", "Makefile")?;
    let mut h = Host::new(request)?;
    let helper = h.helper()?;
    let make = dependency(request, "-make-4.4.1-full-source-gcc10-v1.drv")?
        .members
        .get("out")
        .and_then(|files| files.get("bin/make"))
        .cloned()
        .ok_or_else(|| fail("original GNU Make executable member is missing"))?;
    let initial = h.exec(
        "make-version-request",
        make.clone(),
        Files::new(),
        vec![lit("--mantle-source-version-request"), output("result")],
        ArtifactKind::Tree,
        exact(),
    );
    let version_checkpoint = h.add(
        "original-make-version",
        Files::from([("tool".into(), make), ("initial".into(), initial)]),
        ArtifactKind::Tree,
        RecipeOperation::Exec {
            executable_input: "tool".into(),
            arguments: vec![
                lit("--mantle-source-control"),
                input("initial", "call"),
                lit("-"),
                lit("-"),
                output("result"),
                lit("make"),
                lit("--version"),
            ],
            environment: BTreeMap::from([
                ("LC_ALL".into(), lit("C")),
                ("SOURCE_DATE_EPOCH".into(), lit("1")),
                ("MANTLE_CONTROLLER_ROOTS".into(), input("initial", "controller-roots")),
            ]),
            runtime_mounts: BTreeMap::new(),
            cwd_input: None,
            cwd_output: None,
            output_seeds: BTreeMap::new(),
            command_state: None,
            guard: None,
            stdin_input: None,
            merge_stderr: false,
            invocation_input: None,
            exit_policy: RecipeExitPolicy::Exact { code: 1 },
        },
    );
    let make_version = h.exec(
        "original-make-version-stdout",
        helper.clone(),
        Files::from([("checkpoint".into(), version_checkpoint)]),
        vec![lit("make-version-output"), input("checkpoint", ""), output("result")],
        ArtifactKind::Text,
        exact(),
    );
    let unit_path = relative_source(&nested, "scripts/unifdef.c");
    let object = h.compile(
        "unifdef",
        src.artifact.clone(),
        &unit_path,
        &[
            "-Wall",
            "-Wmissing-prototypes",
            "-Wstrict-prototypes",
            "-O2",
            "-fomit-frame-pointer",
            "-std=gnu11",
        ],
        &[],
        &[],
    )?;
    let unifdef = h.link("unifdef", &[object])?;
    let script = kernel_member(&mut h, &helper, src, &nested, "scripts/headers_install.sh", "original-headers-install");
    let makefile = kernel_member(&mut h, &helper, src, &nested, "Makefile", "original-kernel-makefile");
    let version = h.exec(
        "version-header",
        helper.clone(),
        Files::from([("makefile".into(), makefile), ("version".into(), make_version)]),
        vec![
            lit("version"),
            input("makefile", ""),
            input("version", ""),
            output("result"),
        ],
        ArtifactKind::Text,
        exact(),
    );
    let mut pending =
        BTreeMap::from([("linux/version.h".into(), ("include/generated/uapi/linux/version.h".into(), version))]);
    let common = "include/uapi";
    let arch = "arch/x86/include/uapi";
    let arch_kbuild = read_source(&root.join(arch).join("asm/Kbuild"))?;
    let generic_kbuild = read_source(&root.join("include/uapi/asm-generic/Kbuild"))?;
    let generated = kbuild_list(&arch_kbuild, "generated-y")?;
    let mut generic = kbuild_list(&arch_kbuild, "generic-y")?;
    for name in kbuild_list(&generic_kbuild, "mandatory-y")? {
        if !generated.contains(&name) && !root.join(arch).join("asm").join(&name).is_file() {
            generic.push(name);
        }
    }
    generic.sort();
    generic.dedup();
    let wrapper_rule =
        kernel_member(&mut h, &helper, src, &nested, "scripts/Makefile.asm-generic", "original-wrapper-rule");
    let generated_paths = generic
        .iter()
        .chain(generated.iter())
        .map(|name| format!("arch/x86/include/generated/uapi/asm/{name}"))
        .collect();
    let excluded = excluded_uapi_headers(&root, &generated_paths)?;
    for directory in [common, arch] {
        for path in source_headers(&root, directory)? {
            let installed = path
                .strip_prefix(&format!("{directory}/"))
                .ok_or_else(|| fail("original header prefix"))?
                .to_string();
            if directory == common && excluded.contains(&installed) {
                continue;
            }
            let index = pending.len();
            let reference = kernel_member(&mut h, &helper, src, &nested, &path, &format!("header-{index}.source"));
            pending.insert(installed, (path, reference));
        }
    }
    for name in generic {
        let index = pending.len();
        let reference = h.exec(
            &format!("header-{index}.wrapper"),
            helper.clone(),
            Files::from([("rule".into(), wrapper_rule.clone())]),
            vec![lit("wrapper"), input("rule", ""), lit(&name), output("result")],
            ArtifactKind::Text,
            exact(),
        );
        pending.insert(format!("asm/{name}"), (format!("arch/x86/include/generated/uapi/asm/{name}"), reference));
    }
    let syscall_rule = kernel_member(&mut h, &helper, src, &nested, "scripts/syscallhdr.sh", "original-syscall-rule");
    for SyscallHeader {
        name,
        table,
        abis,
        offset,
        prefix,
    } in syscall_headers(&root, &generated)?
    {
        let index = pending.len();
        let reference = h.exec(
            &format!("header-{index}.syscall"),
            helper.clone(),
            Files::from([
                ("archive".into(), src.artifact.clone()),
                ("script".into(), syscall_rule.clone()),
            ]),
            vec![
                lit("syscall"),
                input("archive", &relative_source(&nested, &format!("arch/x86/entry/syscalls/{table}"))),
                input("script", ""),
                lit(abis),
                lit(offset),
                lit(prefix),
                lit(&name),
                output("result"),
            ],
            ArtifactKind::Text,
            exact(),
        );
        pending
            .entry(format!("asm/{name}"))
            .or_insert((format!("arch/x86/include/generated/uapi/asm/{name}"), reference));
    }
    let mut members = Files::new();
    for (index, (path, (original, reference))) in pending.into_iter().enumerate() {
        let header = install_header(&mut h, &helper, &unifdef, &script, &original, reference, index);
        members.insert(format!("include/{path}"), header);
    }
    let evidence = h.exec(
        "linux-header-metadata",
        helper.clone(),
        Files::from([("recipe".into(), source(LINUX_RECIPE_MEMBER))]),
        vec![
            lit("block"),
            input("recipe", ""),
            lit("full-source-rust-host-linux-headers.txt\" <<'EOF'\n"),
            lit("\nEOF"),
            output("result"),
        ],
        ArtifactKind::Text,
        exact(),
    );
    members.insert("share/mantle-bootstrap/full-source-rust-host-linux-headers.txt".into(), evidence);
    let mut files = helper_files();
    files.insert(LINUX_RECIPE_MEMBER.into(), include_str!("../bootstrap/linux-headers-6.6-gcc10.ncl").into());
    h.finish(members, files)
}

/// Keep original UAPI topology and source-evaluated original host recipe
/// configure/build/install/consumer continuations in one selected-root dispatch.
pub(crate) fn lower(request: &NativeNodeRequest<'_, '_>) -> Result<Option<NativeNodeProgram>, RunError> {
    let path = request.node.drv_path.to_string();
    if path.ends_with("-make-4.4.1-full-source-gcc10-v1.drv") {
        return make(request).map(Some);
    }
    if path.ends_with("-linux-headers-6.6-full-source-gcc10-v3.drv") {
        return linux_headers(request).map(Some);
    }
    packages::lower(request)
}
