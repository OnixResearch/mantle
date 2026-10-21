//! Individual native leaves for the authenticated mrustc 0.12 source route.
//!
//! Preparation reads only original archives. Makefiles select objects and flags;
//! make itself is never an executable action. GCC executes only its specs query,
//! with compilation, assembly, archives and post-link operations kept distinct.
use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::io::Read;

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
use crate::source_built_recipe::RecipeOperation;
use crate::source_built_recipe::RecipeOutput;
use crate::source_built_recipe::RecipePreparation;
use crate::source_built_recipe::RecipeStep;
use crate::source_built_recipe::RecipeWord;
use crate::source_built_recipe::SourceRecipe;
use crate::source_built_recipe::WordPart;
use crate::source_built_recipe::relative;

const STAGE: &str = "full-source-rust-provider";
const PREFIX: &str = "rust-mrustc";
const TOP: &str = "mrustc-0.12.0";
const GCC_VERSION: &str = "10.5.0";
const GCC_TARGET: &str = "x86_64-unknown-linux-musl";
const BYTES_MAX: u64 = 2 * 1024 * 1024 * 1024;
const MAKE_BYTES_MAX: u64 = 1024 * 1024;

pub(crate) struct Fragment {
    pub steps: Vec<RecipeStep>,
    pub files: BTreeMap<String, String>,
    pub exports: BTreeMap<String, ArtifactRef>,
}

fn error(message: impl Into<String>) -> RunError {
    RunError::Build(format!("mrustc source authoring: {}", message.into()))
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
fn attached(prefix: &str, name: &str, path: &str) -> RecipeWord {
    RecipeWord {
        parts: vec![WordPart::Literal { text: prefix.into() }, WordPart::Input {
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
fn produced(reference: &ArtifactRef) -> Result<(), RunError> {
    if matches!(reference, ArtifactRef::Produced { .. }) {
        Ok(())
    } else {
        Err(error("executable and native data references must have exact producers"))
    }
}
fn outputs(name: &str, kind: ArtifactKind) -> BTreeMap<String, RecipeOutput> {
    BTreeMap::from([
        (name.into(), RecipeOutput {
            kind,
            bytes_max: BYTES_MAX,
        }),
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
fn predecessor(request: &AuthoringRequest<'_>, role: &str) -> Result<ArtifactRef, RunError> {
    let reference = request
        .predecessor_exports
        .get(&format!("full-source-native-provider:{role}"))
        .ok_or_else(|| error(format!("missing exact native constituent {role}")))?
        .clone();
    produced(&reference)?;
    Ok(reference)
}

fn makefiles(request: &AuthoringRequest<'_>) -> Result<BTreeMap<String, String>, RunError> {
    let path = crate::source_built_rust_programs::source_archive(request.source_roots, "mrustc-0.12.0")?;
    let file = std::fs::File::open(path).map_err(|e| error(e.to_string()))?;
    let mut archive = tar::Archive::new(flate2::read::GzDecoder::new(file));
    let wanted = ["Makefile", "tools/common/Makefile", "tools/minicargo/Makefile"];
    let mut files = BTreeMap::new();
    let mut source_members = BTreeSet::new();
    for entry in archive.entries().map_err(|e| error(e.to_string()))? {
        let mut entry = entry.map_err(|e| error(e.to_string()))?;
        let path = entry.path().map_err(|e| error(e.to_string()))?.into_owned();
        let path = path.to_str().ok_or_else(|| error("non-UTF8 archive member"))?;
        let Some(member) = path.strip_prefix(&format!("{TOP}/")) else {
            continue;
        };
        if !entry.header().entry_type().is_file() {
            continue;
        }
        relative(member, false)?;
        if !source_members.insert(member.to_string()) {
            return Err(error("duplicate original archive member"));
        }
        if wanted.contains(&member) {
            if entry.size() > MAKE_BYTES_MAX {
                return Err(error("original Makefile exceeds read bound"));
            }
            let mut text = String::new();
            entry.read_to_string(&mut text).map_err(|e| error(e.to_string()))?;
            files.insert(member.to_string(), text);
        }
    }
    for member in wanted {
        if !files.contains_key(member) {
            return Err(error(format!("missing original {member}")));
        }
    }
    for (makefile, variable, directory) in [
        ("Makefile", "OBJ", "src"),
        ("tools/common/Makefile", "OBJS", "tools/common"),
        ("tools/minicargo/Makefile", "OBJS", "tools/minicargo"),
    ] {
        for object in object_inventory(&files[makefile], variable)? {
            let source =
                format!("{directory}/{}.cpp", object.strip_suffix(".o").ok_or_else(|| error("non-object member"))?);
            if !source_members.contains(&source) {
                return Err(error(format!("Makefile selects absent original source {source}")));
            }
        }
    }
    Ok(files)
}

/// These pinned Makefiles spell the complete inventory with simple assignments,
/// then perform exactly one directory-prefix transformation. Reject a changed
/// expression rather than treating an unevaluated make expression as a source.
fn object_inventory(text: &str, variable: &str) -> Result<Vec<String>, RunError> {
    let mut result = Vec::new();
    let mut prefixed = false;
    for line in text.lines() {
        let Some(rest) = line.strip_prefix(variable).and_then(|tail| tail.strip_prefix(' ')) else {
            continue;
        };
        let Some((operator, value)) = rest.split_once(' ') else {
            continue;
        };
        if !["=", ":=", "+="].contains(&operator) {
            continue;
        }
        if value.contains("$(") {
            let expected = if variable == "OBJ" {
                "$(addprefix $(OBJDIR),$(OBJ))"
            } else {
                "$(OBJS:%=$(OBJDIR)%)"
            };
            if value != expected || prefixed || result.is_empty() {
                return Err(error("unrecognized object-inventory expression"));
            }
            prefixed = true;
            continue;
        }
        if prefixed {
            return Err(error("object assignment after directory transformation"));
        }
        if operator != "+=" {
            result.clear();
        }
        for object in value.split_whitespace() {
            relative(object, false)?;
            if !object.ends_with(".o") || result.iter().any(|prior| prior == object) {
                return Err(error("invalid or repeated original object"));
            }
            result.push(object.to_string());
        }
    }
    if result.is_empty() || !prefixed {
        return Err(error("incomplete original object inventory"));
    }
    Ok(result)
}

/// Evaluate the selected ordinary assignment lines, excluding the optional
/// profiling branch. Main/common CXXFLAGS are command-line overrides in the
/// original source owner; minicargo's separate make uses its own assignments.
fn flags(text: &str, variable: &str) -> Result<Vec<String>, RunError> {
    let mut result = Vec::new();
    let mut profiling = false;
    for line in text.lines() {
        if line == "ifneq ($(GPROF),)" {
            profiling = true;
            continue;
        }
        if profiling {
            if line == "endif" {
                profiling = false;
            }
            continue;
        }
        let Some(rest) = line.strip_prefix(&format!("{variable} ")) else {
            continue;
        };
        let Some((operator, value)) = rest.split_once(' ') else {
            continue;
        };
        if !["=", ":=", "+="].contains(&operator) {
            continue;
        }
        if operator != "+=" {
            result.clear();
        }
        for word in value.split_whitespace() {
            if word == format!("$({variable}_EXTRA)") {
                continue;
            }
            if word.contains('$') {
                return Err(error(format!("unresolved original {variable} flag {word}")));
            }
            result.push(word.to_string());
        }
    }
    Ok(result)
}

struct Tools {
    inputs: BTreeMap<String, ArtifactRef>,
    original_prefixes: BTreeMap<String, String>,
    driver: ArtifactRef,
    splitter: ArtifactRef,
    cc1plus: ArtifactRef,
    assembler: ArtifactRef,
    linker: ArtifactRef,
    collect2: ArtifactRef,
    archiver: ArtifactRef,
    objcopy: ArtifactRef,
    strip: ArtifactRef,
}
impl Tools {
    fn new(request: &AuthoringRequest<'_>, program: &mut SourceRecipe) -> Result<Self, RunError> {
        let driver = predecessor(request, "native.gxx")?;
        let splitter = predecessor(request, "native.gcc-splitter")?;
        let cc1plus = predecessor(request, "native.cc1plus")?;
        let assembler = predecessor(request, "native.as")?;
        let linker = predecessor(request, "native.ld")?;
        let collect2 = predecessor(request, "native.collect2-query")?;
        let archiver = predecessor(request, "native.ar")?;
        let objcopy = predecessor(request, "native.objcopy")?;
        let strip = predecessor(request, "native.strip")?;
        let id = format!("{PREFIX}.phase-tools");
        program.steps.push(RecipeStep {
            id: id.clone(),
            inputs: BTreeMap::from([
                ("cc1plus".into(), cc1plus.clone()),
                ("as".into(), assembler.clone()),
                ("ld".into(), linker.clone()),
                ("collect2".into(), collect2.clone()),
            ]),
            outputs: BTreeMap::from([("tree".into(), RecipeOutput {
                kind: ArtifactKind::Tree,
                bytes_max: BYTES_MAX,
            })]),
            operation: RecipeOperation::AssembleTree {
                output: "tree".into(),
                entries: BTreeMap::from([
                    ("cc1plus".into(), "cc1plus".into()),
                    ("as".into(), "as".into()),
                    ("ld".into(), "ld".into()),
                    ("collect2".into(), "collect2".into()),
                ]),
            },
        });
        let mut inputs = BTreeMap::new();
        for role in [
            "native.provider",
            "native.sysroot",
            "linux.headers",
            "native.libc",
            "native.libgcc",
            "native.libgcc-eh",
            "native.libstdcxx",
            "host.cmake-root",
        ] {
            inputs.insert(role.replace('.', "-"), predecessor(request, role)?);
        }
        inputs.insert("phase-tools".into(), product(&id, "tree"));
        let prefixes = crate::source_built_native_programs::logical_prefixes(request)?;
        let original_prefixes = BTreeMap::from([
            (prefixes.provider, "/inputs/native-provider".into()),
            (prefixes.gcc, "/inputs/native-provider".into()),
            (prefixes.musl, "/inputs/native-sysroot/x86_64-linux-musl".into()),
            (prefixes.binutils, "/inputs/native-provider".into()),
        ]);
        Ok(Self {
            inputs,
            original_prefixes,
            driver,
            splitter,
            cc1plus,
            assembler,
            linker,
            collect2,
            archiver,
            objcopy,
            strip,
        })
    }
    fn arguments(&self) -> Vec<RecipeWord> {
        // Rebind the original GCC installation and its normalized musl sysroot.
        // These are search roots, not a replacement compiler or linker flag table.
        vec![
            attached("-B", "phase-tools", ""),
            attached("-B", "native-provider", &format!("lib/gcc/{GCC_TARGET}/{GCC_VERSION}")),
            attached("--sysroot=", "native-sysroot", "x86_64-linux-musl"),
            attached("-Wl,-dynamic-linker,", "native-sysroot", "x86_64-linux-musl/lib/ld-musl-x86_64.so.1"),
            attached("-Wl,-rpath,", "native-sysroot", "x86_64-linux-musl/lib"),
            lit("-isystem"),
            input("native-provider", &format!("include/c++/{GCC_VERSION}")),
            lit("-isystem"),
            input("native-provider", &format!("include/c++/{GCC_VERSION}/{GCC_TARGET}")),
            lit("-isystem"),
            input("native-provider", &format!("include/c++/{GCC_VERSION}/backward")),
            lit("-isystem"),
            input("native-provider", &format!("lib/gcc/{GCC_TARGET}/{GCC_VERSION}/include")),
            attached("-L", "native-provider", &format!("lib/gcc/{GCC_TARGET}/{GCC_VERSION}")),
        ]
    }
    fn environment(&self) -> BTreeMap<String, RecipeWord> {
        // Source semantics of seed-full.ncl write_compiler_wrapper, rebound
        // to declared inputs rather than running its shell/driver wrapper.
        BTreeMap::from([
            ("COMPILER_PATH".into(), input("phase-tools", "")),
            ("MANTLE_GCC_ASSEMBLER".into(), input("phase-tools", "as")),
            ("MANTLE_GCC_LINKER".into(), input("phase-tools", "ld")),
            ("MANTLE_GCC_LINK_LIBC".into(), input("native-sysroot", "x86_64-linux-musl/lib/libc.so")),
            ("MANTLE_GCC_LINK_LIBM".into(), input("native-sysroot", "x86_64-linux-musl/lib/libc.so")),
            ("MANTLE_GCC_LINK_LIBGCC".into(), input("native-sysroot", "x86_64-linux-musl/lib/libgcc.a")),
            ("MANTLE_GCC_LINK_LIBGCC_EH".into(), input("native-sysroot", "x86_64-linux-musl/lib/libgcc_eh.a")),
            ("MANTLE_GCC_LINK_LIBGCC_S".into(), input("native-sysroot", "x86_64-linux-musl/lib/libgcc_s.so.1")),
            ("MANTLE_GCC_LINK_CRT".into(), input("native-sysroot", "x86_64-linux-musl/lib")),
        ])
    }
    fn mappings(&self, inputs: &BTreeMap<String, ArtifactRef>) -> BTreeMap<String, String> {
        let mut map = self.original_prefixes.clone();
        map.extend(inputs.keys().map(|name| {
            let path = format!("/inputs/{name}");
            (path.clone(), path)
        }));
        map
    }
}

fn exec(
    program: &mut SourceRecipe,
    id: &str,
    tool: ArtifactRef,
    mut inputs: BTreeMap<String, ArtifactRef>,
    arguments: Vec<RecipeWord>,
    name: &str,
    kind: ArtifactKind,
) -> ArtifactRef {
    inputs.insert("tool".into(), tool);
    program.steps.push(RecipeStep {
        id: id.into(),
        inputs,
        outputs: outputs(name, kind),
        operation: RecipeOperation::Exec {
            invocation_input: None,
            merge_stderr: false,
            executable_input: "tool".into(),
            arguments,
            environment: BTreeMap::new(),
            guard: None,
            runtime_mounts: BTreeMap::new(),
            cwd_input: None,
            cwd_output: None,
            output_seeds: BTreeMap::new(),
            command_state: None,
            stdin_input: None,
            exit_policy: RecipeExitPolicy::Exact { code: 0 },
        },
    });
    product(id, name)
}

fn include_flags(words: &[String], directory: &str) -> Result<Vec<RecipeWord>, RunError> {
    let mut result = Vec::new();
    let mut words = words.iter();
    while let Some(word) = words.next() {
        if word == "-I" {
            let path = words.next().ok_or_else(|| error("missing original include operand"))?;
            let path = match path.trim_end_matches('/') {
                "../common" => "tools/common".to_string(),
                path if directory.is_empty() => path.to_string(),
                path => format!("{directory}/{path}"),
            };
            relative(&path, false)?;
            result.extend([lit("-I"), input("source", &format!("{TOP}/{path}"))]);
        } else if word.contains('/') || word.contains('$') {
            return Err(error("unlowered original flag path"));
        } else {
            result.push(lit(word));
        }
    }
    Ok(result)
}

fn compile(
    program: &mut SourceRecipe,
    tools: &Tools,
    source: ArtifactRef,
    id: &str,
    root: &str,
    member: &str,
    flags: Vec<RecipeWord>,
    pch: bool,
) -> Result<(ArtifactRef, ArtifactRef), RunError> {
    let mut inputs = tools.inputs.clone();
    inputs.insert("source".into(), source);
    let mut arguments = tools.arguments();
    arguments.extend(flags);
    // -S changes only the stopping point of the source's original -c rule.
    // A PCH invocation already has exactly one compiler phase and no assembler.
    if !pch {
        arguments.push(lit("-S"));
    }
    arguments.extend([
        lit("-o"),
        input("source", "mrustc-query-output"),
        input("source", &format!("{root}/{member}")),
        lit("-MMD"),
        lit("-MP"),
        lit("-MF"),
        input("source", "mrustc-query-dependency"),
    ]);
    let mut map = tools.mappings(&inputs);
    map.insert("/inputs/source/mrustc-query-output".into(), "/outputs/compiled".into());
    map.insert("/inputs/source/mrustc-query-dependency".into(), "/outputs/dependencies".into());
    let mut phase_outputs = outputs("compiled", if pch { ArtifactKind::Object } else { ArtifactKind::Text });
    phase_outputs.insert("dependencies".into(), RecipeOutput {
        kind: ArtifactKind::Text,
        bytes_max: MAKE_BYTES_MAX,
    });
    let expansion = append_gcc_invocation(program, GccInvocation {
        id: format!("{id}.compile"),
        driver: tools.driver.clone(),
        splitter: tools.splitter.clone(),
        inputs,
        arguments,
        environment: tools.environment(),
        runtime_mounts: BTreeMap::new(),
        cwd_input: None,
        phases: vec![GccPhase {
            invocation_input: None,
            merge_stderr: false,
            executable: tools.cc1plus.clone(),
            expected_image: input("phase-tools", "cc1plus"),
            inputs: BTreeMap::new(),
            environment: BTreeMap::new(),
            guard: None,
            output_seeds: BTreeMap::new(),
            cwd_output: None,
            outputs: phase_outputs,
            path_mappings: map,
            runtime_mounts: BTreeMap::new(),
            exit_policy: RecipeExitPolicy::Exact { code: 0 },
        }],
    })?;
    let compiled = expansion.phase_outputs[0]["compiled"].clone();
    let dependencies = expansion.phase_outputs[0]["dependencies"].clone();
    if pch {
        return Ok((compiled, dependencies));
    }
    // Ask the original driver for the assembler options too; target defaults
    // such as --64 are selected by its authenticated specs, never guessed.
    let mut inputs = tools.inputs.clone();
    inputs.insert("assembly".into(), compiled);
    let mut arguments = tools.arguments();
    arguments.extend([
        lit("-x"),
        lit("assembler"),
        lit("-c"),
        input("assembly", ""),
        lit("-o"),
        input("assembly", "mrustc-query-object"),
    ]);
    let mut map = tools.mappings(&inputs);
    map.insert("/inputs/assembly/mrustc-query-object".into(), "/outputs/object".into());
    let expansion = append_gcc_invocation(program, GccInvocation {
        id: format!("{id}.assemble"),
        driver: tools.driver.clone(),
        splitter: tools.splitter.clone(),
        inputs,
        arguments,
        environment: tools.environment(),
        runtime_mounts: BTreeMap::new(),
        cwd_input: None,
        phases: vec![GccPhase {
            invocation_input: None,
            merge_stderr: false,
            executable: tools.assembler.clone(),
            expected_image: input("phase-tools", "as"),
            inputs: BTreeMap::new(),
            environment: BTreeMap::new(),
            guard: None,
            output_seeds: BTreeMap::new(),
            cwd_output: None,
            outputs: outputs("object", ArtifactKind::Object),
            path_mappings: map,
            runtime_mounts: BTreeMap::new(),
            exit_policy: RecipeExitPolicy::Exact { code: 0 },
        }],
    })?;
    Ok((expansion.phase_outputs[0]["object"].clone(), dependencies))
}

fn archive(program: &mut SourceRecipe, tools: &Tools, id: &str, objects: &[(String, ArtifactRef)]) -> ArtifactRef {
    let mut inputs = BTreeMap::new();
    // Use original object basenames as archive member names (ar takes basename
    // from its operand). Separate individual input names preserve that contract.
    let mut arguments = vec![lit("crs"), output("archive")];
    for (ordinal, (member, object)) in objects.iter().enumerate() {
        let name = format!("object-{ordinal}");
        // Materializing an object beneath its original basename is required:
        // the direct object artifact itself has the generic output id object.
        let tree_id = format!("{id}.member-{ordinal}");
        let basename = member.rsplit('/').next().unwrap_or(member);
        program.steps.push(RecipeStep {
            id: tree_id.clone(),
            inputs: BTreeMap::from([("object".into(), object.clone())]),
            outputs: BTreeMap::from([("tree".into(), RecipeOutput {
                kind: ArtifactKind::Tree,
                bytes_max: BYTES_MAX,
            })]),
            operation: RecipeOperation::AssembleTree {
                output: "tree".into(),
                entries: BTreeMap::from([(basename.into(), "object".into())]),
            },
        });
        inputs.insert(name.clone(), product(&tree_id, "tree"));
        arguments.push(input(&name, basename));
    }
    exec(program, id, tools.archiver.clone(), inputs, arguments, "archive", ArtifactKind::Archive)
}

fn link(
    program: &mut SourceRecipe,
    tools: &Tools,
    id: &str,
    objects: &[(String, ArtifactRef)],
    archives: &[(&str, ArtifactRef)],
    flags: Vec<RecipeWord>,
    whole_archive: bool,
) -> Result<ArtifactRef, RunError> {
    let mut inputs = tools.inputs.clone();
    let mut arguments = tools.arguments();
    arguments.extend([lit("-o"), input("phase-tools", "mrustc-query-executable")]);
    // The original mrustc rule puts LINKFLAGS before main.o, while minicargo
    // puts them after its common archive. Preserve that ordering exactly.
    if whole_archive {
        arguments.extend(flags.clone());
    }
    for (ordinal, (_, object)) in objects.iter().enumerate() {
        let name = format!("object-{ordinal}");
        inputs.insert(name.clone(), object.clone());
        arguments.push(input(&name, ""));
    }
    for (ordinal, (name, reference)) in archives.iter().enumerate() {
        inputs.insert((*name).into(), reference.clone());
        if whole_archive && ordinal == 0 {
            arguments.push(lit("-Wl,--whole-archive"));
        }
        arguments.push(input(name, ""));
        if whole_archive && ordinal == 0 {
            arguments.push(lit("-Wl,--no-whole-archive"));
        }
    }
    if whole_archive {
        arguments.push(input("host-cmake-root", "lib/libz.a"));
    } else {
        arguments.extend(flags);
    }
    let query_id = format!("{id}.collect2");
    let query = append_gcc_invocation(program, GccInvocation {
        id: query_id.clone(),
        driver: tools.driver.clone(),
        splitter: tools.splitter.clone(),
        inputs: inputs.clone(),
        arguments,
        environment: tools.environment(),
        runtime_mounts: BTreeMap::new(),
        cwd_input: None,
        phases: vec![GccPhase {
            invocation_input: None,
            merge_stderr: false,
            executable: tools.collect2.clone(),
            expected_image: input("phase-tools", "collect2"),
            guard: None,
            output_seeds: BTreeMap::new(),
            cwd_output: None,
            inputs: BTreeMap::from([("query-driver-log".into(), product(&format!("{query_id}.specs"), "stderr"))]),
            environment: BTreeMap::from([
                ("MANTLE_COLLECT2_PHASE_QUERY".into(), lit("1")),
                ("MANTLE_COLLECT2_PHASE_LINKER".into(), input("phase-tools", "ld")),
                ("MANTLE_COLLECT2_PHASE_DRIVER_LOG".into(), input("query-driver-log", "")),
            ]),
            outputs: BTreeMap::from([
                ("stdout".into(), RecipeOutput {
                    kind: ArtifactKind::Text,
                    bytes_max: 16 * 1024 * 1024,
                }),
                ("stderr".into(), RecipeOutput {
                    kind: ArtifactKind::Text,
                    bytes_max: 16 * 1024 * 1024,
                }),
            ]),
            path_mappings: tools.mappings(&inputs),
            runtime_mounts: BTreeMap::new(),
            exit_policy: RecipeExitPolicy::Exact { code: 0 },
        }],
    })?;
    // Only the actual original collect2 source may select this ld argv. Its
    // guarded source seam emits ld1_argv before do_link and rejects routes
    // needing collection, LTO, plugins or any additional child phase.
    let mut map = tools.mappings(&inputs);
    map.insert("/inputs/phase-tools/mrustc-query-executable".into(), "/outputs/executable".into());
    let expansion = append_gcc_response_phases(program, GccResponseInvocation {
        id: format!("{id}.link"),
        splitter: tools.splitter.clone(),
        inputs,
        log: query.phase_outputs[0]["stdout"].clone(),
        environment: tools.environment(),
        cwd_input: None,
        phases: vec![GccPhase {
            invocation_input: None,
            merge_stderr: false,
            executable: tools.linker.clone(),
            expected_image: input("phase-tools", "ld"),
            inputs: BTreeMap::new(),
            environment: BTreeMap::new(),
            guard: None,
            output_seeds: BTreeMap::new(),
            cwd_output: None,
            outputs: outputs("executable", ArtifactKind::Executable),
            path_mappings: map,
            runtime_mounts: BTreeMap::new(),
            exit_policy: RecipeExitPolicy::Exact { code: 0 },
        }],
    })?;
    let unstripped = expansion.phase_outputs[0]["executable"].clone();
    program.exports.insert(format!("{id}.unstripped"), unstripped.clone());
    let basename = if whole_archive { "mrustc" } else { "minicargo" };
    let debug_name = format!("{basename}.debug");
    let debug = exec(
        program,
        &format!("{id}.keep-debug"),
        tools.objcopy.clone(),
        BTreeMap::from([("binary".into(), unstripped.clone())]),
        vec![lit("--only-keep-debug"), input("binary", ""), output(&debug_name)],
        &debug_name,
        ArtifactKind::Object,
    );
    // A named tree gives --add-gnu-debuglink its original filename, not a
    // generic artifact id. Objcopy's optional output operand avoids mutation.
    let debug_tree_id = format!("{id}.debug-file");
    program.steps.push(RecipeStep {
        id: debug_tree_id.clone(),
        inputs: BTreeMap::from([("debug".into(), debug.clone())]),
        outputs: BTreeMap::from([("tree".into(), RecipeOutput {
            kind: ArtifactKind::Tree,
            bytes_max: BYTES_MAX,
        })]),
        operation: RecipeOperation::AssembleTree {
            output: "tree".into(),
            entries: BTreeMap::from([(debug_name.clone(), "debug".into())]),
        },
    });
    let linked = exec(
        program,
        &format!("{id}.debuglink"),
        tools.objcopy.clone(),
        BTreeMap::from([
            ("binary".into(), unstripped),
            ("debug-files".into(), product(&debug_tree_id, "tree")),
        ]),
        vec![
            attached("--add-gnu-debuglink=", "debug-files", &debug_name),
            input("binary", ""),
            output("binary"),
        ],
        "binary",
        ArtifactKind::Executable,
    );
    program.exports.insert(format!("{basename}.debug"), debug);
    program.exports.insert(format!("{basename}.debuglinked"), linked.clone());
    Ok(exec(
        program,
        &format!("{id}.strip"),
        tools.strip.clone(),
        BTreeMap::from([("binary".into(), linked)]),
        vec![lit("-o"), output("executable"), input("binary", "")],
        "executable",
        ArtifactKind::Executable,
    ))
}

pub(crate) fn author(request: &AuthoringRequest<'_>, source_tree: ArtifactRef) -> Result<Fragment, RunError> {
    produced(&source_tree)?;
    let makefiles = makefiles(request)?;
    let main_make = &makefiles["Makefile"];
    let common_make = &makefiles["tools/common/Makefile"];
    let mini_make = &makefiles["tools/minicargo/Makefile"];
    let mut program = SourceRecipe {
        schema: "mantle-source-recipe-v1".into(),
        stage_id: STAGE.into(),
        preparation: RecipePreparation {
            tool: "original-mrustc-makefile-source-expansion".into(),
            provenance: "bootstrap/rust-source-musl-host-plan.ncl".into(),
        },
        steps: Vec::new(),
        roots: Vec::new(),
        exports: BTreeMap::new(),
        semantic_roots: BTreeMap::new(),
        timeout_groups: Vec::new(),
    };
    let tools = Tools::new(request, &mut program)?;
    let cpp = include_flags(&flags(main_make, "CPPFLAGS")?, "")?;
    let mut pch_flags = vec![lit("-std=c++14")];
    pch_flags.extend(cpp.clone());
    for word in &mut pch_flags {
        for part in &mut word.parts {
            if let WordPart::Input { name, path } = part
                && name == "source"
            {
                *path = format!("source/{path}");
            }
        }
    }
    // The PCH and its consumer must see identical virtual source paths. Keep
    // the original archive root under the same source/ prefix in both trees.
    let pch_original_id = format!("{PREFIX}.pch-original");
    program.steps.push(RecipeStep {
        id: pch_original_id.clone(),
        inputs: BTreeMap::from([("original".into(), source_tree.clone())]),
        outputs: BTreeMap::from([("tree".into(), RecipeOutput {
            kind: ArtifactKind::Tree,
            bytes_max: BYTES_MAX,
        })]),
        operation: RecipeOperation::AssembleTree {
            output: "tree".into(),
            entries: BTreeMap::from([("source".into(), "original".into())]),
        },
    });
    let pchs = main_make
        .lines()
        .find_map(|line| line.strip_prefix("PCHS := "))
        .ok_or_else(|| error("missing original PCH inventory"))?;
    let mut pch_inputs = BTreeMap::from([("original".into(), source_tree.clone())]);
    let mut pch_entries = BTreeMap::from([("source".into(), "original".into())]);
    for (ordinal, header) in pchs.split_whitespace().enumerate() {
        relative(header, false)?;
        let id = format!("{PREFIX}.pch-{ordinal}");
        let (pch, dependency) = compile(
            &mut program,
            &tools,
            product(&pch_original_id, "tree"),
            &id,
            &format!("source/{TOP}"),
            &format!("src/{header}"),
            pch_flags.clone(),
            true,
        )?;
        let name = format!("pch-{ordinal}");
        pch_inputs.insert(name.clone(), pch.clone());
        pch_entries.insert(format!("source/{TOP}/src/{header}.gch"), name);
        program.exports.insert(format!("pch.{header}"), pch);
        program.exports.insert(format!("pch.{header}.dependencies"), dependency);
    }
    let pch_tree_id = format!("{PREFIX}.pch-source");
    program.steps.push(RecipeStep {
        id: pch_tree_id.clone(),
        inputs: pch_inputs,
        outputs: BTreeMap::from([("tree".into(), RecipeOutput {
            kind: ArtifactKind::Tree,
            bytes_max: BYTES_MAX,
        })]),
        operation: RecipeOperation::AssembleTree {
            output: "tree".into(),
            entries: pch_entries,
        },
    });
    // Retain the PCH as a real preceding artifact without rewriting its source.
    // Its separate overlay is used below by the main translation unit.
    let original_override = vec![
        lit("-g0"),
        lit("-O2"),
        lit("-D_LIBCPP_HARDENING_MODE=_LIBCPP_HARDENING_MODE_NONE"),
        lit("-static-libstdc++"),
        lit("-static-libgcc"),
        lit("-I"),
        input("host-cmake-root", "include"),
        lit("-I"),
        input("linux-headers", "include"),
    ];
    let mut batches = BTreeMap::new();
    for (batch, makefile, variable, directory) in [
        ("common", common_make, "OBJS", "tools/common"),
        ("compiler", main_make, "OBJ", "src"),
        ("minicargo", mini_make, "OBJS", "tools/minicargo"),
    ] {
        let mut objects = Vec::new();
        for (ordinal, member) in object_inventory(makefile, variable)?.into_iter().enumerate() {
            let id = format!("{PREFIX}.{batch}.unit-{ordinal}");
            let mut unit_flags = if batch == "minicargo" {
                include_flags(&flags(makefile, "CXXFLAGS")?, directory)?
            } else {
                original_override.clone()
            };
            if batch == "compiler" {
                unit_flags.extend(cpp.clone());
            }
            if batch == "compiler" && member == "version.o" {
                // Exact normalization owned by rust_source_provider.rs's
                // push_first_stage_authenticated_version_metadata; no Git/date.
                unit_flags.extend([
                    lit("-D"),
                    lit("VERSION_GIT_FULLHASH=\"authenticated-mrustc-0.12.0\""),
                    lit("-D"),
                    lit("VERSION_GIT_BRANCH=\"v0.12.0\""),
                    lit("-D"),
                    lit("VERSION_GIT_SHORTHASH=\"v0.12.0\""),
                    lit("-D"),
                    lit("VERSION_BUILDTIME=\"1970-01-01T00:00:00Z\""),
                    lit("-D"),
                    lit("VERSION_GIT_ISDIRTY=0"),
                ]);
            }
            let source_member =
                format!("{directory}/{}.cpp", member.strip_suffix(".o").ok_or_else(|| error("invalid object suffix"))?);
            let (unit_source, unit_root) = if batch == "compiler" && member == "main.o" {
                for word in &mut unit_flags {
                    for part in &mut word.parts {
                        if let WordPart::Input { name, path } = part
                            && name == "source"
                        {
                            *path = format!("source/{path}");
                        }
                    }
                }
                (product(&pch_tree_id, "tree"), format!("source/{TOP}"))
            } else {
                (source_tree.clone(), TOP.to_string())
            };
            let (object, dependency) =
                compile(&mut program, &tools, unit_source, &id, &unit_root, &source_member, unit_flags, false)?;
            program.exports.insert(format!("object.{batch}.{member}"), object.clone());
            program.exports.insert(format!("dependencies.{batch}.{member}"), dependency);
            objects.push((member, object));
        }
        batches.insert(batch, objects);
    }
    let common = archive(&mut program, &tools, &format!("{PREFIX}.common.archive"), &batches["common"]);
    let compiler_members = batches["compiler"].iter().filter(|(name, _)| name != "main.o").cloned().collect::<Vec<_>>();
    let compiler = archive(&mut program, &tools, &format!("{PREFIX}.compiler.archive"), &compiler_members);
    let main = batches["compiler"].iter().filter(|(name, _)| name == "main.o").cloned().collect::<Vec<_>>();
    if main.len() != 1 {
        return Err(error("original compiler inventory must select exactly one main.o"));
    }
    let mrustc = link(
        &mut program,
        &tools,
        &format!("{PREFIX}.compiler"),
        &main,
        &[("mrustc-archive", compiler.clone()), ("common-archive", common.clone())],
        flags(main_make, "LINKFLAGS")?.into_iter().map(lit).collect(),
        true,
    )?;
    let minicargo = link(
        &mut program,
        &tools,
        &format!("{PREFIX}.minicargo"),
        &batches["minicargo"],
        &[("common-archive", common.clone())],
        flags(mini_make, "LINKFLAGS")?.into_iter().map(lit).collect(),
        false,
    )?;
    program.exports.insert("mrustc".into(), mrustc);
    program.exports.insert("minicargo".into(), minicargo);
    program.exports.insert("mrustc.archive".into(), compiler);
    program.exports.insert("common.archive".into(), common);
    program.exports.insert("mrustc.pch-source".into(), product(&pch_tree_id, "tree"));
    Ok(Fragment {
        steps: program.steps,
        files: BTreeMap::new(),
        exports: program.exports,
    })
}
