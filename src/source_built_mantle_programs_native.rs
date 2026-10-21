//! Direct native archive leaves and original-cc process continuation support.
//! This module does not synthesize Cargo metadata. The original build script
//! must complete with these products before its stdout can authorize metadata.
#![allow(dead_code)] // Pending wiring in the source-only preparation path (Cairn prove-source-built-mantle-fixed-point I3).
use std::collections::BTreeMap;
use std::collections::BTreeSet;

use crate::errors::RunError;
use crate::source_built_native_programs::phases::GccInvocation;
use crate::source_built_native_programs::phases::GccPhase;
use crate::source_built_native_programs::phases::append_gcc_invocation;
use crate::source_built_native_programs::phases::append_gcc_request_invocation;
use crate::source_built_native_programs::phases::gcc_phase_output;
use crate::source_built_recipe::ArtifactKind;
use crate::source_built_recipe::ArtifactRef;
use crate::source_built_recipe::RecipeExitPolicy;
use crate::source_built_recipe::RecipeGuard;
use crate::source_built_recipe::RecipeGuardCondition;
use crate::source_built_recipe::RecipeOperation;
use crate::source_built_recipe::RecipeOutput;
use crate::source_built_recipe::RecipePhaseState;
use crate::source_built_recipe::RecipeStep;
use crate::source_built_recipe::RecipeWord;
use crate::source_built_recipe::SourceRecipe;
use crate::source_built_recipe::WordPart;
use crate::source_built_recipe::error;
use crate::source_built_recipe::relative;

const BYTES_MAX: u64 = 512 * 1024 * 1024;

/// All compiler defaults, include directories and runtime mounts are bound by
/// the caller from the original source-owned compiler and cc policies.
/// In particular, there is no ambient GCC search, version query or flag table.
#[derive(Clone)]
pub(crate) struct NativeToolchain {
    pub driver: ArtifactRef,
    pub splitter: ArtifactRef,
    pub cc1: ArtifactRef,
    pub cc1_image: RecipeWord,
    pub assembler: ArtifactRef,
    pub archiver: ArtifactRef,
    pub inputs: BTreeMap<String, ArtifactRef>,
    pub arguments: Vec<RecipeWord>,
    pub environment: BTreeMap<String, RecipeWord>,
    pub runtime_mounts: BTreeMap<String, String>,
    pub assembler_arguments: Vec<RecipeWord>,
}

#[derive(Clone, Copy)]
pub(crate) enum NativeLanguage {
    C,
    AssemblyWithCpp,
    Assembly,
}

pub(crate) struct NativeTranslationUnit {
    /// Exact original source or an original generator's Produced source tree.
    pub source: ArtifactRef,
    pub path: String,
    /// Exact original cc object basename, including its source-directory hash.
    pub object_name: String,
    pub language: NativeLanguage,
    /// Resolved original cc::Build arguments, with filesystem operands typed.
    pub arguments: Vec<RecipeWord>,
    /// Original per-file assembler options (for example --noexecstack).
    pub assembler_arguments: Vec<RecipeWord>,
    pub inputs: BTreeMap<String, ArtifactRef>,
}

pub(crate) struct NativeArchive {
    pub id: String,
    /// A compiled original script or its emitted source phase. This is not a
    /// substitute for executing that script to obtain its final metadata.
    pub generator: ArtifactRef,
    pub units: Vec<NativeTranslationUnit>,
    /// Original archive member basename and its exact Produced object.
    pub additional_objects: Vec<(String, ArtifactRef)>,
}

pub(crate) struct NativeExpansion {
    pub step_ids: BTreeSet<String>,
    pub objects: Vec<ArtifactRef>,
    pub archive: ArtifactRef,
}

fn lit(text: &str) -> RecipeWord {
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
fn product(stage: &str, step: &str, name: &str) -> ArtifactRef {
    ArtifactRef::Produced {
        action_id: format!("{stage}/{step}"),
        output_id: name.into(),
    }
}
fn produced(reference: &ArtifactRef) -> Result<(), RunError> {
    if matches!(reference, ArtifactRef::Produced { .. }) {
        Ok(())
    } else {
        Err(error("native generator, compiler and object authority must be Produced"))
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
            bytes_max: BYTES_MAX,
        }),
        ("stderr".into(), RecipeOutput {
            kind: ArtifactKind::Text,
            bytes_max: BYTES_MAX,
        }),
        ("exit-status".into(), RecipeOutput {
            kind: ArtifactKind::Text,
            bytes_max: 4,
        }),
    ])
}
fn merge(inputs: &mut BTreeMap<String, ArtifactRef>, name: &str, value: ArtifactRef) -> Result<(), RunError> {
    if inputs.insert(name.into(), value).is_some() {
        return Err(error(format!("duplicate native input {name}")));
    }
    Ok(())
}

/// Lower an original source-selected archive batch. The compiler driver only
/// interprets its genuine specs with -###; exactly one cc1 phase then executes.
/// Assembling and archive indexing are individual no-child native leaves.
///
/// `-S`/`-E` changes the stopping point, not the selected original language or
/// compiler defaults. Its query-only output path is under a declared read-only
/// input, because -### does not write it; the strict splitter remaps that path
/// into the direct cc1 leaf's declared assembly output.
pub(crate) fn append_native_archive(
    program: &mut SourceRecipe,
    tools: &NativeToolchain,
    archive: NativeArchive,
) -> Result<NativeExpansion, RunError> {
    for tool in [
        &tools.driver,
        &tools.splitter,
        &tools.cc1,
        &tools.assembler,
        &tools.archiver,
        &archive.generator,
    ] {
        produced(tool)?;
    }
    relative(&archive.id, false)?;
    if archive.id.contains('/') || archive.units.is_empty() && archive.additional_objects.is_empty() {
        return Err(error("native archive needs a label and original source/object members"));
    }
    let mut member_names = BTreeSet::new();
    for unit in &archive.units {
        relative(&unit.path, false)?;
        relative(&unit.object_name, false)?;
        if unit.object_name.contains('/') || !member_names.insert(unit.object_name.clone()) {
            return Err(error("native archive has duplicate or non-basename object members"));
        }
    }
    for (name, object) in &archive.additional_objects {
        relative(name, false)?;
        if name.contains('/') || !member_names.insert(name.clone()) {
            return Err(error("native archive has duplicate or non-basename object members"));
        }
        produced(object)?;
    }
    // Build off to the side: a failed expansion must not leave half a program.
    let mut fragment = SourceRecipe {
        schema: program.schema.clone(),
        stage_id: program.stage_id.clone(),
        preparation: program.preparation.clone(),
        steps: Vec::new(),
        roots: Vec::new(),
        exports: BTreeMap::new(),
        semantic_roots: BTreeMap::new(),
        timeout_groups: Vec::new(),
    };
    let mut objects = Vec::with_capacity(archive.units.len() + archive.additional_objects.len());
    let mut object_names = Vec::with_capacity(objects.capacity());
    for (ordinal, unit) in archive.units.into_iter().enumerate() {
        let id = format!("{}.unit-{ordinal}", archive.id);
        let mut inputs = tools.inputs.clone();
        for (name, reference) in unit.inputs {
            merge(&mut inputs, &name, reference)?;
        }
        merge(&mut inputs, "native-source", unit.source)?;
        merge(&mut inputs, "native-generator", archive.generator.clone())?;
        let assembly = match unit.language {
            NativeLanguage::Assembly => None,
            NativeLanguage::C | NativeLanguage::AssemblyWithCpp => {
                let virtual_output = "mantle-native-query.s";
                let mut arguments = tools.arguments.clone();
                arguments.extend(unit.arguments);
                arguments.extend([
                    lit("-x"),
                    lit(match unit.language {
                        NativeLanguage::C => "c",
                        _ => "assembler-with-cpp",
                    }),
                    lit(match unit.language {
                        NativeLanguage::C => "-S",
                        _ => "-E",
                    }),
                    input("native-source", &unit.path),
                    lit("-o"),
                    input("native-source", virtual_output),
                ]);
                let mut mappings = inputs
                    .keys()
                    .map(|name| {
                        let path = format!("/inputs/{name}");
                        (path.clone(), path)
                    })
                    .collect::<BTreeMap<_, _>>();
                mappings.insert(format!("/inputs/native-source/{virtual_output}"), "/outputs/assembly".into());
                let expansion = append_gcc_invocation(&mut fragment, GccInvocation {
                    id: format!("{id}.compile"),
                    driver: tools.driver.clone(),
                    splitter: tools.splitter.clone(),
                    inputs: inputs.clone(),
                    arguments,
                    environment: tools.environment.clone(),
                    runtime_mounts: tools.runtime_mounts.clone(),
                    cwd_input: Some("native-source".into()),
                    phases: vec![GccPhase {
                        invocation_input: None,
                        merge_stderr: false,
                        executable: tools.cc1.clone(),
                        expected_image: tools.cc1_image.clone(),
                        inputs: BTreeMap::new(),
                        outputs: outputs("assembly", ArtifactKind::Text),
                        path_mappings: mappings,
                        runtime_mounts: tools.runtime_mounts.clone(),
                        output_seeds: BTreeMap::new(),
                        cwd_output: None,
                        guard: None,
                        environment: BTreeMap::new(),
                        exit_policy: RecipeExitPolicy::Exact { code: 0 },
                    }],
                })?;
                Some(expansion.phase_outputs[0]["assembly"].clone())
            }
        };
        let mut arguments = tools.assembler_arguments.clone();
        arguments.extend(unit.assembler_arguments);
        if let Some(assembly) = assembly {
            merge(&mut inputs, "native-assembly", assembly)?;
            arguments.push(input("native-assembly", ""));
        } else {
            arguments.push(input("native-source", &unit.path));
        }
        arguments.extend([lit("-o"), output("object")]);
        merge(&mut inputs, "native-assembler", tools.assembler.clone())?;
        let assemble_id = format!("{id}.assemble");
        fragment.steps.push(RecipeStep {
            id: assemble_id.clone(),
            inputs,
            outputs: outputs("object", ArtifactKind::Object),
            operation: RecipeOperation::Exec {
                command_state: None,
                merge_stderr: false,
                executable_input: "native-assembler".into(),
                invocation_input: None,
                arguments,
                environment: tools.environment.clone(),
                runtime_mounts: tools.runtime_mounts.clone(),
                cwd_input: Some("native-source".into()),
                cwd_output: None,
                output_seeds: BTreeMap::new(),
                stdin_input: None,
                exit_policy: RecipeExitPolicy::Exact { code: 0 },
                guard: None,
            },
        });
        objects.push(product(&program.stage_id, &assemble_id, "object"));
        object_names.push(unit.object_name);
    }
    for (name, object) in archive.additional_objects {
        object_names.push(name);
        objects.push(object);
    }
    let archive_id = format!("{}.archive", archive.id);
    let mut inputs = BTreeMap::from([
        ("native-archiver".into(), tools.archiver.clone()),
        ("native-generator".into(), archive.generator),
    ]);
    let mut arguments = vec![lit("crsD"), output("archive")];
    for (name, object) in object_names.iter().zip(&objects) {
        merge(&mut inputs, name, object.clone())?;
        arguments.push(input(name, ""));
    }
    fragment.steps.push(RecipeStep {
        id: archive_id.clone(),
        inputs,
        outputs: outputs("archive", ArtifactKind::Archive),
        operation: RecipeOperation::Exec {
            command_state: None,
            merge_stderr: false,
            executable_input: "native-archiver".into(),
            invocation_input: None,
            arguments,
            environment: tools.environment.clone(),
            runtime_mounts: tools.runtime_mounts.clone(),
            cwd_input: None,
            cwd_output: None,
            output_seeds: BTreeMap::new(),
            stdin_input: None,
            exit_policy: RecipeExitPolicy::Exact { code: 0 },
            guard: None,
        },
    });
    let step_ids = fragment.steps.iter().map(|step| step.id.clone()).collect::<BTreeSet<_>>();
    if step_ids.len() != fragment.steps.len() || program.steps.iter().any(|step| step_ids.contains(&step.id)) {
        return Err(error("native expansion collides with existing action IDs"));
    }
    program.steps.extend(fragment.steps);
    program.timeout_groups.extend(fragment.timeout_groups);
    Ok(NativeExpansion {
        step_ids,
        objects,
        archive: product(&program.stage_id, &archive_id, "archive"),
    })
}

/// Pack one actual completed original-script command into the continuation ABI.
/// `request`, stdout, stderr, actual exit status and every installed file must
/// be Produced products of the original command's direct recipe action. No
/// bytes are copied from an earlier build or inferred from a filename.
pub(crate) fn append_cc_reply(
    program: &mut SourceRecipe,
    id: &str,
    request: ArtifactRef,
    stdout: ArtifactRef,
    stderr: ArtifactRef,
    status: ArtifactRef,
    files: BTreeMap<String, ArtifactRef>,
) -> Result<ArtifactRef, RunError> {
    for reference in [&request, &stdout, &stderr, &status].into_iter().chain(files.values()) {
        produced(reference)?;
    }
    let reply_id = format!("{id}.reply");
    if program.steps.iter().any(|step| step.id == reply_id) {
        return Err(error("cc reply collides with an existing action ID"));
    }
    let mut inputs = BTreeMap::from([
        ("request".into(), request),
        ("stdout".into(), stdout),
        ("stderr".into(), stderr),
        ("status".into(), status),
    ]);
    let mut entries = BTreeMap::from([
        ("request".into(), "request".into()),
        ("stdout".into(), "stdout".into()),
        ("stderr".into(), "stderr".into()),
        ("status".into(), "status".into()),
    ]);
    for (ordinal, (path, reference)) in files.into_iter().enumerate() {
        relative(&path, false)?;
        let name = format!("file-{ordinal}");
        inputs.insert(name.clone(), reference);
        entries.insert(format!("files/{path}"), name);
    }
    program.steps.push(RecipeStep {
        id: reply_id.clone(),
        inputs,
        outputs: BTreeMap::from([("reply".into(), RecipeOutput {
            kind: ArtifactKind::Tree,
            bytes_max: BYTES_MAX,
        })]),
        operation: RecipeOperation::AssembleTree {
            output: "reply".into(),
            entries,
        },
    });
    Ok(product(&program.stage_id, &reply_id, "reply"))
}

/// A source-owned, pure-I/O boundary for cc's original Command values. The
/// compiler request is emitted by original cc code; replies contain exclusively
/// actual direct-leaf products. Every replay checks exact argv/environment/cwd.
/// An absent next reply is a phase boundary (75), not successful compilation.
/// This ABI intentionally does not choose a recipe's finite command inventory.
pub(crate) const CC_PROCESS_SOURCE: &str = r###"
use std::{collections::BTreeMap, ffi::OsStr, fs, io, path::Path, process::{Command, Output}, sync::atomic::{AtomicUsize, Ordering}};
use std::os::unix::{ffi::OsStrExt, process::ExitStatusExt};
static RANDOM_NEXT: AtomicUsize = AtomicUsize::new(0);
pub(crate) fn random(original: impl FnOnce() -> u64) -> u64 {
    fn value(ordinal: usize, original: impl FnOnce() -> u64) -> io::Result<u64> {
        let name = ordinal.to_string();
        let previous = std::env::var_os("MANTLE_CC_PHASE_STATE");
        let saved = previous.map(|root| fs::read_to_string(Path::new(&root).join(".mantle-cc-random").join(&name)));
        let value = match saved {
            Some(Ok(text)) => text.trim().parse().map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "invalid original cc random state"))?,
            Some(Err(error)) if error.kind() != io::ErrorKind::NotFound => return Err(error),
            _ => original(),
        };
        let out = std::env::var_os("OUT_DIR").ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "missing original cc OUT_DIR"))?;
        let state = Path::new(&out).join(".mantle-cc-random");
        fs::create_dir_all(&state)?;
        fs::write(state.join(name), format!("{value}\n"))?;
        Ok(value)
    }
    value(RANDOM_NEXT.fetch_add(1, Ordering::SeqCst), original).unwrap_or_else(|error| panic!("original cc phase random state: {}", error))
}
fn next_command() -> io::Result<usize> {
    let out = std::env::var_os("OUT_DIR").ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "missing original process OUT_DIR"))?;
    let directory = Path::new(&out).join(".mantle-cc-random");
    fs::create_dir_all(&directory)?;
    let path = directory.join(".process-index");
    let ordinal: usize = match fs::read_to_string(&path) {
        Ok(value) => value.trim().parse().map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "invalid original process ordinal"))?,
        Err(error) if error.kind() == io::ErrorKind::NotFound => 0,
        Err(error) => return Err(error),
    };
    let next = ordinal.checked_add(1).ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "original process ordinal overflow"))?;
    fs::write(path, format!("{next}\n"))?;
    Ok(ordinal)
}
fn bytes(out: &mut Vec<u8>, word: &OsStr) {
    let word = word.as_bytes(); out.extend_from_slice(&(word.len() as u64).to_le_bytes()); out.extend_from_slice(word);
}
fn request(command: &Command) -> io::Result<Vec<u8>> {
    let mut out = b"mantle-cc-process-v1\0".to_vec();
    bytes(&mut out, command.get_program());
    let args: Vec<_> = command.get_args().collect(); out.extend_from_slice(&(args.len() as u64).to_le_bytes());
    for arg in args { bytes(&mut out, arg); }
    let cwd = match command.get_current_dir() { Some(path) => path.to_path_buf(), None => std::env::current_dir()? };
    bytes(&mut out, cwd.as_os_str());
    let mut env = std::env::vars_os().collect::<BTreeMap<_, _>>();
    for (key, value) in command.get_envs() { if let Some(value) = value { env.insert(key.into(), value.into()); } else { env.remove(key); } }
    env.remove(OsStr::new("MANTLE_CC_REPLIES")); env.remove(OsStr::new("MANTLE_CC_REQUEST"));
    env.remove(OsStr::new("MANTLE_CC_PHASE_STATE"));
    env.remove(OsStr::new("MANTLE_CAIRN_SCRIPT_ENV")); env.remove(OsStr::new("MANTLE_CAIRN_COMPILE_ENV"));
    out.extend_from_slice(&(env.len() as u64).to_le_bytes());
    for (key, value) in env { bytes(&mut out, &key); bytes(&mut out, &value); }
    Ok(out)
}
fn install(source: &Path, destination: &Path, complete: bool) -> io::Result<()> {
    let metadata = fs::symlink_metadata(source)?;
    if complete {
        match fs::symlink_metadata(destination) {
            Ok(prior) if prior.is_dir() != metadata.is_dir() || prior.file_type().is_symlink() => {
                if prior.is_dir() { fs::remove_dir_all(destination)?; } else { fs::remove_file(destination)?; }
            }
            Ok(_) => (),
            Err(error) if error.kind() == io::ErrorKind::NotFound => (),
            Err(error) => return Err(error),
        }
    }
    if metadata.is_dir() {
        fs::create_dir_all(destination)?;
        if complete {
            for entry in fs::read_dir(destination)? {
                let entry = entry?;
                if !source.join(entry.file_name()).try_exists()? {
                    let prior = fs::symlink_metadata(entry.path())?;
                    if prior.is_dir() { fs::remove_dir_all(entry.path())?; } else { fs::remove_file(entry.path())?; }
                }
            }
        }
        let mut entries = fs::read_dir(source)?.collect::<Result<Vec<_>, _>>()?; entries.sort_by_key(|entry| entry.file_name());
        for entry in entries { install(&entry.path(), &destination.join(entry.file_name()), complete)?; }
    } else if metadata.is_file() {
        if let Some(parent) = destination.parent() { fs::create_dir_all(parent)?; }
        fs::copy(source, destination)?;
    } else { return Err(io::Error::new(io::ErrorKind::InvalidData, "non-regular cc replay product")); }
    Ok(())
}
pub(crate) fn output(command: &mut Command) -> io::Result<Output> {
    let ordinal = next_command()?;
    let root = std::env::var_os("MANTLE_CC_REPLIES").ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "missing cc replies input"))?;
    let reply = Path::new(&root).join(ordinal.to_string());
    let actual = request(command)?;
    match fs::read(reply.join("request")) {
        Ok(expected) if expected == actual => (),
        Ok(_) => return Err(io::Error::new(io::ErrorKind::InvalidData, "original cc command does not match source-authored reply")),
        Err(error) if error.kind() == io::ErrorKind::NotFound => {
            let destination = std::env::var_os("MANTLE_CC_REQUEST").ok_or(error)?;
            fs::write(destination, actual)?; std::process::exit(75);
        },
        Err(error) => return Err(error),
    }
    let code: i32 = fs::read_to_string(reply.join("status"))?.trim().parse().map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "invalid cc reply status"))?;
    if !(0..=255).contains(&code) { return Err(io::Error::new(io::ErrorKind::InvalidData, "invalid cc reply status")); }
    let state = reply.join("state");
    let files = reply.join("files");
    let complete = state.try_exists()?;
    if complete || files.try_exists()? {
        let out = std::env::var_os("OUT_DIR").ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "missing original OUT_DIR"))?;
        install(if complete { &state } else { &files }, Path::new(&out), complete)?;
    }
    Ok(Output { status: std::process::ExitStatus::from_raw(code << 8), stdout: fs::read(reply.join("stdout"))?, stderr: fs::read(reply.join("stderr"))? })
}
"###;

/// Cut only original process I/O and parallel scheduling boundaries. Every
/// selected feature module, compiler policy and serial compile loop is retained.
/// The loader applies these unique replacements against admitted original files.
pub(crate) fn cc_library_overlays(
    source: &ArtifactRef,
    compiler_package_path: &str,
) -> Result<Vec<super::metadata::LibraryOverlaySource>, RunError> {
    let original = |member: &str| -> Result<ArtifactRef, RunError> {
        let join = |root: &str| {
            if root.is_empty() {
                member.to_string()
            } else {
                format!("{}/{member}", root.trim_end_matches('/'))
            }
        };
        match source {
            ArtifactRef::Source { role, path } => Ok(ArtifactRef::Source {
                role: *role,
                path: join(path),
            }),
            ArtifactRef::ManifestSource {
                role,
                record_identity,
                member,
            } => Ok(ArtifactRef::ManifestSource {
                role: *role,
                record_identity: record_identity.clone(),
                member: join(member),
            }),
            _ => Err(error("cc process overlay requires the original source package")),
        }
    };
    let overlay = |member: &str,
                   replacements: Vec<(String, String)>,
                   append: Vec<String>|
     -> Result<super::metadata::LibraryOverlaySource, RunError> {
        Ok(super::metadata::LibraryOverlaySource {
            original: original(member)?,
            compiler_path: format!("{}/{member}", compiler_package_path.trim_end_matches('/')),
            replacements,
            append,
        })
    };
    Ok(vec![
        overlay("src/lib.rs", vec![
            ("let output = cmd.current_dir(out_dir).output()?;".into(),
                "let output = crate::mantle_process::output(cmd.current_dir(out_dir))?;".into()),
            ("if objs.len() > 1 {".into(),
                "if objs.len() > 1 && std::env::var_os(\"MANTLE_CC_REPLIES\").is_none() {".into()),
        ], vec![format!("\nmod mantle_process {{\n{CC_PROCESS_SOURCE}\n}}\n")])?,
        overlay("src/command_helpers.rs", vec![
            ("//! Miscellaneous helpers for running commands".into(),
                "//! Miscellaneous helpers for running commands\n#![allow(dead_code)]".into()),
            ("    let mut child = spawn(cmd, cargo_output)?;\n    wait_on_child(cmd, &mut child, cargo_output)".into(),
                r#"    cargo_output.print_debug(&format_args!("running: {cmd:?}"));
    let result = crate::mantle_process::output(cmd)
        .map_err(|e| Error::new(ErrorKind::ToolExecError, e.to_string()))?;
    if cargo_output.warnings {
        for line in result.stderr.split_inclusive(|byte| *byte == b'\n') {
            write_warning(line.strip_suffix(b"\n").unwrap_or(line));
        }
    }
    if matches!(cargo_output.output, OutputKind::Forward) {
        io::stdout().write_all(&result.stdout)
            .map_err(|e| Error::new(ErrorKind::ToolExecError, e.to_string()))?;
    }
    let status = result.status;
    cargo_output.print_debug(&status);
    if status.success() { Ok(()) } else {
        Err(Error::new(ErrorKind::ToolExecError,
            format!("command did not execute successfully (status code {status}): {cmd:?}")))
    }"#.into()),
            ("    // We specifically need the output to be captured, so override default\n    let mut captured_cargo_output = cargo_output.clone();\n    captured_cargo_output.output = OutputKind::Capture;\n    spawn(cmd, &captured_cargo_output)?\n        .wait_with_output()\n        .map_err(|e| {\n            Error::new(\n                ErrorKind::ToolExecError,\n                format!(\"failed to wait on spawned child process `{cmd:?}`: {e}\"),\n            )\n        })".into(),
                r#"    cargo_output.print_debug(&format_args!("running: {cmd:?}"));
    let mut result = crate::mantle_process::output(cmd)
        .map_err(|e| Error::new(ErrorKind::ToolExecError, e.to_string()))?;
    if !cargo_output.warnings { result.stderr.clear(); }
    Ok(result)"#.into()),
        ], Vec::new())?,
        overlay("src/tempfile.rs", vec![(
            "    RandomState::new().build_hasher().finish()".into(),
            "    crate::mantle_process::random(|| RandomState::new().build_hasher().finish())".into(),
        )], Vec::new())?,
    ])
}

/// Split the original x86_64 PerlAsm generator/translator pipe into two
/// genuine Perl leaves. Neither phase runs an assembler, shell or interpreter
/// child. The original filename remains the generator's output argument, so
/// SHA-256/SHA-512 basename-dependent selection remains original behavior.
pub(crate) fn append_x86_64_perlasm(
    program: &mut SourceRecipe,
    id: &str,
    perl: ArtifactRef,
    original_script: ArtifactRef,
    source_tree: ArtifactRef,
    source_path: &str,
    output_filename: &str,
    environment: BTreeMap<String, RecipeWord>,
    runtime_mounts: BTreeMap<String, String>,
) -> Result<ArtifactRef, RunError> {
    produced(&perl)?;
    produced(&original_script)?;
    relative(source_path, false)?;
    relative(output_filename, false)?;
    if output_filename.contains('/') || !output_filename.ends_with(".S") {
        return Err(error("PerlAsm output must be the original ELF assembly basename"));
    }
    let helper_id = format!("{id}.pipe-source");
    let generate_id = format!("{id}.generate");
    let translate_id = format!("{id}.translate");
    if program.steps.iter().any(|step| [&helper_id, &generate_id, &translate_id].contains(&&step.id)) {
        return Err(error("PerlAsm phases collide with existing source steps"));
    }
    // This wrapper performs only the source-selected pipe cut. It evaluates
    // the admitted original program, not a translation of its algorithm.
    const PIPE_SOURCE: &str = r###"
use strict;
use warnings;
my ($source, $raw, $filename) = @ARGV;
open my $input, '<', $source or die "original PerlAsm source: $!";
my $text; { local $/; $text = <$input>; } close $input or die $!;
my $count = 0;
my @lines = split /\n/, $text, -1;
for my $line (@lines) {
    my $normalized = $line;
    $normalized =~ s/^open OUT,\s*/open OUT,/;
    $normalized =~ s/\s+$//;
    if ($normalized eq q{open OUT,"| \"$^X\" \"$xlate\" $flavour \"$output\"";}) {
        $line = q{open OUT, '>', $ENV{MANTLE_PERLASM_RAW} or die "raw PerlAsm output: $!";};
        $count++;
    }
}
$text = join "\n", @lines;
die "changed original PerlAsm pipe boundary" unless $count == 1;
$ENV{MANTLE_PERLASM_RAW} = $raw;
@ARGV = ('elf', $filename);
$0 = $source;
my $result = eval "no strict; no warnings;\n" . $text;
die $@ if $@;
"###;
    let mut steps = vec![RecipeStep {
        id: helper_id.clone(),
        inputs: BTreeMap::new(),
        outputs: BTreeMap::from([("source".into(), RecipeOutput {
            kind: ArtifactKind::Text,
            bytes_max: BYTES_MAX,
        })]),
        operation: RecipeOperation::WriteText {
            output: "source".into(),
            text: PIPE_SOURCE.into(),
        },
    }];
    let inputs = BTreeMap::from([
        ("perl".into(), perl.clone()),
        ("original-script".into(), original_script.clone()),
        ("source".into(), source_tree.clone()),
        ("pipe-source".into(), product(&program.stage_id, &helper_id, "source")),
    ]);
    steps.push(RecipeStep {
        id: generate_id.clone(),
        inputs,
        outputs: outputs("raw", ArtifactKind::Text),
        operation: RecipeOperation::Exec {
            command_state: None,
            merge_stderr: false,
            executable_input: "perl".into(),
            invocation_input: None,
            arguments: vec![
                input("pipe-source", ""),
                input("source", source_path),
                output("raw"),
                lit(output_filename),
            ],
            environment: environment.clone(),
            runtime_mounts: runtime_mounts.clone(),
            cwd_input: Some("source".into()),
            cwd_output: None,
            output_seeds: BTreeMap::new(),
            stdin_input: None,
            exit_policy: RecipeExitPolicy::Exact { code: 0 },
            guard: None,
        },
    });
    steps.push(RecipeStep {
        id: translate_id.clone(),
        inputs: BTreeMap::from([
            ("perl".into(), perl),
            ("original-script".into(), original_script),
            ("source".into(), source_tree),
            ("raw".into(), product(&program.stage_id, &generate_id, "raw")),
        ]),
        outputs: outputs("assembly", ArtifactKind::Text),
        operation: RecipeOperation::Exec {
            command_state: None,
            merge_stderr: false,
            executable_input: "perl".into(),
            invocation_input: None,
            arguments: vec![
                input("source", "crypto/perlasm/x86_64-xlate.pl"),
                lit("elf"),
                output("assembly"),
            ],
            environment,
            runtime_mounts,
            cwd_input: Some("source".into()),
            cwd_output: None,
            output_seeds: BTreeMap::new(),
            stdin_input: Some("raw".into()),
            exit_policy: RecipeExitPolicy::Exact { code: 0 },
            guard: None,
        },
    });
    program.steps.extend(steps);
    Ok(product(&program.stage_id, &translate_id, "assembly"))
}

/// Execute an original compiler runtime guard (not a precomputed guard result).
/// The linker response must be emitted by a genuine source-owned linker-plan
/// phase, and all response paths must address the supplied immutable inputs or
/// the declared `/outputs/executable`. No GCC/cc child-launching linker driver
/// is run here and no collect2-to-ld substitution is performed.
pub(crate) fn append_native_link_and_run(
    program: &mut SourceRecipe,
    id: &str,
    linker: ArtifactRef,
    response: ArtifactRef,
    mut inputs: BTreeMap<String, ArtifactRef>,
    environment: BTreeMap<String, RecipeWord>,
    runtime_mounts: BTreeMap<String, String>,
) -> Result<(ArtifactRef, ArtifactRef), RunError> {
    produced(&linker)?;
    produced(&response)?;
    let link_id = format!("{id}.link");
    let run_id = format!("{id}.run");
    if program.steps.iter().any(|step| step.id == link_id || step.id == run_id) {
        return Err(error("native guard collides with existing source steps"));
    }
    merge(&mut inputs, "native-linker", linker)?;
    merge(&mut inputs, "native-link-response", response)?;
    let executable = product(&program.stage_id, &link_id, "executable");
    let mut run_inputs = inputs.clone();
    merge(&mut run_inputs, "native-guard", executable.clone())?;
    program.steps.extend([
        RecipeStep {
            id: link_id,
            inputs,
            outputs: outputs("executable", ArtifactKind::Executable),
            operation: RecipeOperation::Exec {
                command_state: None,
                merge_stderr: false,
                executable_input: "native-linker".into(),
                invocation_input: None,
                arguments: vec![RecipeWord {
                    parts: vec![WordPart::Literal { text: "@".into() }, WordPart::Input {
                        name: "native-link-response".into(),
                        path: String::new(),
                    }],
                }],
                environment: environment.clone(),
                runtime_mounts: runtime_mounts.clone(),
                cwd_input: None,
                cwd_output: None,
                output_seeds: BTreeMap::new(),
                stdin_input: None,
                exit_policy: RecipeExitPolicy::Exact { code: 0 },
                guard: None,
            },
        },
        RecipeStep {
            id: run_id.clone(),
            inputs: run_inputs,
            outputs: BTreeMap::from([
                ("stdout".into(), RecipeOutput {
                    kind: ArtifactKind::Text,
                    bytes_max: BYTES_MAX,
                }),
                ("stderr".into(), RecipeOutput {
                    kind: ArtifactKind::Text,
                    bytes_max: BYTES_MAX,
                }),
                ("exit-status".into(), RecipeOutput {
                    kind: ArtifactKind::Text,
                    bytes_max: 4,
                }),
            ]),
            operation: RecipeOperation::Exec {
                command_state: None,
                merge_stderr: false,
                executable_input: "native-guard".into(),
                invocation_input: None,
                arguments: Vec::new(),
                environment,
                runtime_mounts,
                cwd_input: None,
                cwd_output: None,
                output_seeds: BTreeMap::new(),
                stdin_input: None,
                exit_policy: RecipeExitPolicy::Exact { code: 0 },
                guard: None,
            },
        },
    ]);
    Ok((executable, product(&program.stage_id, &run_id, "stdout")))
}

#[derive(Clone, Copy, Debug)]
pub(crate) enum NativeCommandPhase {
    DriverProbe,
    Preprocess,
    Compile,
    /// An original compiler capability check: preserve its actual failure.
    CapabilityCompile,
    ArchiveCreate,
    ArchiveIndex,
}

/// Bindings for lowering requests emitted by the original compiled cc crate.
/// `native.inputs` must include every immutable /inputs alias in those original
/// requests. The package and prior actual OUT_DIR are separately versioned.
#[derive(Clone)]
pub(crate) struct NativeRequestTools {
    pub native: NativeToolchain,
    pub helper: ArtifactRef,
    pub phase_helper: ArtifactRef,
    pub package: ArtifactRef,
    pub compiler_program: RecipeWord,
    pub archiver_program: RecipeWord,
    pub assembler_image: RecipeWord,
}

#[derive(Clone)]
pub(crate) struct NativeCommandProducts {
    pub stdout: ArtifactRef,
    pub stderr: ArtifactRef,
    pub status: ArtifactRef,
    pub files: ArtifactRef,
    pub complete_state: bool,
}

fn command_outputs() -> BTreeMap<String, RecipeOutput> {
    outputs("native-files", ArtifactKind::Tree)
}

/// Lower one authenticated original cc process request. The source helper
/// validates its program/cwd/argv, emits a response without launching anything,
/// and prepares only actual previous archive bytes or required parent dirs.
/// GCC -### retains the ORIGINAL -c arguments and selects both original cc1
/// and as phases; no caller-generated compiler/assembler flag table replaces
/// those specs. Original ar q/s runs with its actual immutable archive seed.
///
/// Compile/preprocess phases require their source-selected success branch.
/// A failed capability probe requiring conditional compiler/as effects must be
/// represented by the owning source schedule, not passed off as a successful
/// Compile phase. Driver -? and --version probes retain their real normal exit.
pub(crate) fn append_native_request(
    program: &mut SourceRecipe,
    tools: &NativeRequestTools,
    phase: NativeCommandPhase,
    id: &str,
    request: ArtifactRef,
    partial_out_dir: ArtifactRef,
) -> Result<NativeCommandProducts, RunError> {
    produced(&tools.helper)?;
    produced(&request)?;
    produced(&partial_out_dir)?;
    let mut fragment = SourceRecipe {
        schema: program.schema.clone(),
        stage_id: program.stage_id.clone(),
        preparation: program.preparation.clone(),
        steps: Vec::new(),
        roots: Vec::new(),
        exports: BTreeMap::new(),
        semantic_roots: BTreeMap::new(),
        timeout_groups: Vec::new(),
    };
    let decode_id = format!("{id}.request");
    let invoke_id = format!("{id}.invoke");
    let result_id = format!("{id}.result");
    let archive = matches!(phase, NativeCommandPhase::ArchiveCreate | NativeCommandPhase::ArchiveIndex);
    let capability = matches!(phase, NativeCommandPhase::CapabilityCompile);
    let phase_name = match phase {
        NativeCommandPhase::DriverProbe => "driver",
        NativeCommandPhase::Preprocess => "preprocess",
        NativeCommandPhase::Compile => "compile",
        NativeCommandPhase::CapabilityCompile => "compile",
        NativeCommandPhase::ArchiveCreate => "ar-create",
        NativeCommandPhase::ArchiveIndex => "ar-index",
    };
    let mut inputs = tools.native.inputs.clone();
    merge(&mut inputs, "package", tools.package.clone())?;
    merge(&mut inputs, "cc-work", partial_out_dir)?;
    let mut decode_inputs = inputs.clone();
    merge(&mut decode_inputs, "native-request", request)?;
    merge(&mut decode_inputs, "native-request-helper", tools.helper.clone())?;
    let mut decode_arguments = vec![
        lit("--native-request"),
        lit(phase_name),
        input("native-request", ""),
        if archive {
            tools.archiver_program.clone()
        } else {
            tools.compiler_program.clone()
        },
        output("invocation"),
        input("cc-work", ""),
        input("package", ""),
        output("environment-data"),
    ];
    if !archive {
        for argument in &tools.native.arguments {
            decode_arguments.extend([lit("--driver-argument"), argument.clone()]);
        }
    }
    for name in inputs.keys() {
        decode_arguments.extend([lit("--binding"), lit(name), input(name, "")]);
    }
    for (mount, name) in &tools.native.runtime_mounts {
        decode_arguments.extend([
            lit("--runtime"),
            RecipeWord {
                parts: vec![WordPart::Runtime {
                    mount: mount.clone(),
                    path: String::new(),
                }],
            },
            lit(name),
        ]);
    }
    fragment.steps.push(RecipeStep {
        id: decode_id.clone(),
        inputs: decode_inputs,
        outputs: {
            let mut result = outputs("invocation", ArtifactKind::Text);
            result.insert("environment-data".into(), RecipeOutput {
                kind: ArtifactKind::Text,
                bytes_max: BYTES_MAX,
            });
            result
        },
        operation: RecipeOperation::Exec {
            command_state: None,
            merge_stderr: false,
            executable_input: "native-request-helper".into(),
            invocation_input: None,
            arguments: decode_arguments,
            environment: tools.native.environment.clone(),
            runtime_mounts: tools.native.runtime_mounts.clone(),
            cwd_input: Some("package".into()),
            cwd_output: None,
            output_seeds: BTreeMap::new(),
            guard: None,
            stdin_input: None,
            exit_policy: RecipeExitPolicy::Exact { code: 0 },
        },
    });
    merge(&mut inputs, "native-request-invocation", product(&program.stage_id, &decode_id, "invocation"))?;
    merge(&mut inputs, "native-environment-data", product(&program.stage_id, &decode_id, "environment-data"))?;
    let seeds = BTreeMap::from([("native-files".into(), "cc-work".into())]);
    let results: Vec<NativeCommandProducts> = match phase {
        NativeCommandPhase::DriverProbe | NativeCommandPhase::ArchiveCreate | NativeCommandPhase::ArchiveIndex => {
            let executable = if archive {
                tools.native.archiver.clone()
            } else {
                tools.native.driver.clone()
            };
            produced(&executable)?;
            let mut direct_inputs = inputs.clone();
            merge(&mut direct_inputs, "native-request-tool", executable)?;
            fragment.steps.push(RecipeStep {
                id: invoke_id.clone(),
                inputs: direct_inputs,
                outputs: command_outputs(),
                operation: RecipeOperation::Exec {
                    command_state: None,
                    merge_stderr: false,
                    executable_input: "native-request-tool".into(),
                    invocation_input: Some("native-request-invocation".into()),
                    arguments: Vec::new(),
                    environment: BTreeMap::new(),
                    runtime_mounts: tools.native.runtime_mounts.clone(),
                    cwd_input: None,
                    cwd_output: None,
                    output_seeds: seeds.clone(),
                    stdin_input: None,
                    guard: None,
                    exit_policy: if archive {
                        RecipeExitPolicy::Exact { code: 0 }
                    } else {
                        RecipeExitPolicy::OneOf { codes: vec![0, 1] }
                    },
                },
            });
            vec![NativeCommandProducts {
                stdout: product(&program.stage_id, &invoke_id, "stdout"),
                stderr: product(&program.stage_id, &invoke_id, "stderr"),
                status: product(&program.stage_id, &invoke_id, "exit-status"),
                files: product(&program.stage_id, &invoke_id, "native-files"),
                complete_state: true,
            }]
        }
        NativeCommandPhase::Preprocess | NativeCommandPhase::Compile | NativeCommandPhase::CapabilityCompile => {
            let mut mappings = inputs
                .keys()
                .filter(|name| name.as_str() != "native-request-invocation")
                .map(|name| {
                    let path = format!("/inputs/{name}");
                    (path.clone(), path)
                })
                .chain([(String::from("/outputs/native-files"), String::from("/outputs/native-files"))])
                .collect::<BTreeMap<_, _>>();
            mappings.insert("/inputs/cc-work".into(), "/outputs/native-files".into());
            for mount in tools.native.runtime_mounts.keys() {
                mappings.insert(mount.clone(), mount.clone());
            }
            let mut compiler_maps = mappings.clone();
            compiler_maps.insert("/tmp".into(), "/outputs/native-temporary".into());
            let mut compiler_outputs = command_outputs();
            compiler_outputs.insert("native-temporary".into(), RecipeOutput {
                kind: ArtifactKind::Tree,
                bytes_max: BYTES_MAX,
            });
            let query_inputs = inputs.clone();
            let mut phases = vec![GccPhase {
                invocation_input: None,
                merge_stderr: false,
                executable: tools.native.cc1.clone(),
                expected_image: tools.native.cc1_image.clone(),
                inputs: BTreeMap::new(),
                outputs: compiler_outputs,
                path_mappings: compiler_maps,
                environment: BTreeMap::new(),
                runtime_mounts: tools.native.runtime_mounts.clone(),
                output_seeds: seeds.clone(),
                cwd_output: None,
                guard: None,
                exit_policy: if capability {
                    RecipeExitPolicy::OneOf { codes: vec![0, 1] }
                } else {
                    RecipeExitPolicy::Exact { code: 0 }
                },
            }];
            if !matches!(phase, NativeCommandPhase::Preprocess) {
                let mut assembler_maps = mappings;
                assembler_maps.insert("/tmp".into(), "/inputs/native-temporary".into());
                let mut assembler_inputs = BTreeMap::from([
                    ("native-temporary".into(), gcc_phase_output(&program.stage_id, &invoke_id, 0, "native-temporary")),
                    (
                        "native-compiler-files".into(),
                        gcc_phase_output(&program.stage_id, &invoke_id, 0, "native-files"),
                    ),
                ]);
                if capability {
                    assembler_inputs.insert(
                        "native-compiler-status".into(),
                        gcc_phase_output(&program.stage_id, &invoke_id, 0, "exit-status"),
                    );
                }
                phases.push(GccPhase {
                    invocation_input: None,
                    merge_stderr: false,
                    executable: tools.native.assembler.clone(),
                    expected_image: tools.assembler_image.clone(),
                    inputs: assembler_inputs,
                    outputs: command_outputs(),
                    path_mappings: assembler_maps,
                    environment: BTreeMap::new(),
                    runtime_mounts: tools.native.runtime_mounts.clone(),
                    output_seeds: BTreeMap::from([("native-files".into(), "native-compiler-files".into())]),
                    cwd_output: None,
                    guard: capability.then(|| RecipeGuard {
                        conditions: vec![RecipeGuardCondition {
                            status_input: "native-compiler-status".into(),
                            accepted_codes: vec![0],
                        }],
                    }),
                    exit_policy: if capability {
                        RecipeExitPolicy::OneOf { codes: vec![0, 1] }
                    } else {
                        RecipeExitPolicy::Exact { code: 0 }
                    },
                });
            }
            let expansion = append_gcc_request_invocation(
                &mut fragment,
                GccInvocation {
                    id: invoke_id.clone(),
                    driver: tools.native.driver.clone(),
                    splitter: tools.native.splitter.clone(),
                    inputs: query_inputs,
                    arguments: Vec::new(),
                    environment: BTreeMap::new(),
                    runtime_mounts: tools.native.runtime_mounts.clone(),
                    cwd_input: None,
                    phases,
                },
                "native-request-invocation".into(),
            )?;
            if capability {
                let observation_id = format!("{id}.assembler-observation");
                fragment.steps.push(RecipeStep {
                    id: observation_id.clone(),
                    inputs: BTreeMap::from([(
                        "prior-files".into(),
                        expansion.phase_outputs[0]["native-files"].clone(),
                    )]),
                    outputs: BTreeMap::from([("phase".into(), RecipeOutput {
                        kind: ArtifactKind::Tree,
                        bytes_max: BYTES_MAX * 2,
                    })]),
                    operation: RecipeOperation::ObservePhase {
                        phase: format!("{invoke_id}.phase-1"),
                        output: "phase".into(),
                        state: Some(RecipePhaseState {
                            output: "native-files".into(),
                            skipped_input: "prior-files".into(),
                            skipped_path: String::new(),
                        }),
                    },
                });
                let compiler = &expansion.phase_outputs[0];
                fragment.steps.push(RecipeStep {
                    id: result_id.clone(),
                    inputs: BTreeMap::from([
                        ("native-phase-helper".into(), tools.phase_helper.clone()),
                        ("compiler-stdout".into(), compiler["stdout"].clone()),
                        ("compiler-stderr".into(), compiler["stderr"].clone()),
                        ("compiler-status".into(), compiler["exit-status"].clone()),
                        ("assembler-phase".into(), product(&program.stage_id, &observation_id, "phase")),
                    ]),
                    outputs: {
                        let mut values = outputs("files", ArtifactKind::Tree);
                        for name in ["command-stdout", "command-stderr", "command-status"] {
                            values.insert(name.into(), RecipeOutput {
                                kind: ArtifactKind::Text,
                                bytes_max: BYTES_MAX,
                            });
                        }
                        values
                    },
                    operation: RecipeOperation::Exec {
                        command_state: None,
                        merge_stderr: false,
                        executable_input: "native-phase-helper".into(),
                        invocation_input: None,
                        arguments: vec![
                            lit("--capability-result"),
                            input("compiler-stdout", ""),
                            input("compiler-stderr", ""),
                            input("compiler-status", ""),
                            input("assembler-phase", ""),
                            output("command-stdout"),
                            output("command-stderr"),
                            output("command-status"),
                            output("files"),
                        ],
                        environment: tools.native.environment.clone(),
                        runtime_mounts: tools.native.runtime_mounts.clone(),
                        cwd_input: None,
                        cwd_output: None,
                        output_seeds: BTreeMap::new(),
                        stdin_input: None,
                        guard: None,
                        exit_policy: RecipeExitPolicy::Exact { code: 0 },
                    },
                });
                let step_ids = fragment.steps.iter().map(|step| step.id.clone()).collect::<BTreeSet<_>>();
                if step_ids.len() != fragment.steps.len()
                    || program.steps.iter().any(|step| step_ids.contains(&step.id))
                {
                    return Err(error("native capability request collides with existing recipe steps"));
                }
                program.steps.extend(fragment.steps);
                program.timeout_groups.extend(fragment.timeout_groups);
                return Ok(NativeCommandProducts {
                    stdout: product(&program.stage_id, &result_id, "command-stdout"),
                    stderr: product(&program.stage_id, &result_id, "command-stderr"),
                    status: product(&program.stage_id, &result_id, "command-status"),
                    files: product(&program.stage_id, &result_id, "files"),
                    complete_state: true,
                });
            }
            expansion
                .phase_outputs
                .into_iter()
                .map(|products| NativeCommandProducts {
                    stdout: products["stdout"].clone(),
                    stderr: products["stderr"].clone(),
                    status: products["exit-status"].clone(),
                    files: products["native-files"].clone(),
                    complete_state: true,
                })
                .collect()
        }
    };
    let mut result_inputs = BTreeMap::from([("native-request-helper".into(), tools.helper.clone())]);
    let mut arguments = vec![
        lit("--native-result"),
        output("command-stdout"),
        output("command-stderr"),
        output("command-status"),
        output("files"),
    ];
    let mut final_files = None;
    for (ordinal, result) in results.into_iter().enumerate() {
        final_files = Some(result.files);
        for (suffix, reference) in [
            ("stdout", result.stdout),
            ("stderr", result.stderr),
            ("status", result.status),
        ] {
            let name = format!("phase-{ordinal}-{suffix}");
            result_inputs.insert(name.clone(), reference);
            arguments.push(input(&name, ""));
        }
    }
    result_inputs
        .insert("final-files".into(), final_files.ok_or_else(|| error("native result has no actual phase state"))?);
    arguments.push(input("final-files", ""));
    let mut result_outputs = outputs("files", ArtifactKind::Tree);
    for name in ["command-stdout", "command-stderr", "command-status"] {
        result_outputs.insert(name.into(), RecipeOutput {
            kind: ArtifactKind::Text,
            bytes_max: BYTES_MAX,
        });
    }
    fragment.steps.push(RecipeStep {
        id: result_id.clone(),
        inputs: result_inputs,
        outputs: result_outputs,
        operation: RecipeOperation::Exec {
            command_state: None,
            merge_stderr: false,
            executable_input: "native-request-helper".into(),
            invocation_input: None,
            arguments,
            environment: tools.native.environment.clone(),
            runtime_mounts: tools.native.runtime_mounts.clone(),
            cwd_input: None,
            cwd_output: None,
            output_seeds: BTreeMap::new(),
            stdin_input: None,
            guard: None,
            exit_policy: RecipeExitPolicy::Exact { code: 0 },
        },
    });
    let step_ids = fragment.steps.iter().map(|step| step.id.clone()).collect::<BTreeSet<_>>();
    if step_ids.len() != fragment.steps.len() || program.steps.iter().any(|step| step_ids.contains(&step.id)) {
        return Err(error("native command request collides with existing recipe steps"));
    }
    program.steps.extend(fragment.steps);
    program.timeout_groups.extend(fragment.timeout_groups);
    Ok(NativeCommandProducts {
        stdout: product(&program.stage_id, &result_id, "command-stdout"),
        stderr: product(&program.stage_id, &result_id, "command-stderr"),
        status: product(&program.stage_id, &result_id, "command-status"),
        files: product(&program.stage_id, &result_id, "files"),
        complete_state: true,
    })
}

pub(crate) fn append_cc_reply_files_tree(
    program: &mut SourceRecipe,
    id: &str,
    request: ArtifactRef,
    products: NativeCommandProducts,
) -> Result<ArtifactRef, RunError> {
    for reference in [
        &request,
        &products.stdout,
        &products.stderr,
        &products.status,
        &products.files,
    ] {
        produced(reference)?;
    }
    let reply_id = format!("{id}.reply");
    if program.steps.iter().any(|step| step.id == reply_id) {
        return Err(error("duplicate native reply tree"));
    }
    program.steps.push(RecipeStep {
        id: reply_id.clone(),
        inputs: BTreeMap::from([
            ("request".into(), request),
            ("stdout".into(), products.stdout),
            ("stderr".into(), products.stderr),
            ("status".into(), products.status),
            ("files".into(), products.files),
        ]),
        outputs: BTreeMap::from([("reply".into(), RecipeOutput {
            kind: ArtifactKind::Tree,
            bytes_max: BYTES_MAX,
        })]),
        operation: RecipeOperation::AssembleTree {
            output: "reply".into(),
            entries: BTreeMap::from([
                ("request".into(), "request".into()),
                ("stdout".into(), "stdout".into()),
                ("stderr".into(), "stderr".into()),
                ("status".into(), "status".into()),
                (if products.complete_state { "state" } else { "files" }.into(), "files".into()),
            ]),
        },
    });
    Ok(product(&program.stage_id, &reply_id, "reply"))
}

#[path = "source_built_mantle_programs_native_schedules.rs"]
mod schedules;
pub(crate) use schedules::append_schedule;
pub(crate) use schedules::generation_overlay;
pub(crate) use schedules::library_overlays;
pub(crate) use schedules::script_boundaries;
pub(crate) use schedules::selected_schedule;
pub(crate) use schedules::source_files;
