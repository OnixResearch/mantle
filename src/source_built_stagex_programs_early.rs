//! Expansion of the original early StageX programs into child-free commands.
use std::collections::BTreeMap;
use std::path::Component;
use std::path::Path;

use crate::errors::RunError;
use crate::source_built_program_authoring::AuthoringRequest;
use crate::source_built_recipe::ArtifactKind;
use crate::source_built_recipe::ArtifactRef;
use crate::source_built_recipe::RecipeExitPolicy;
use crate::source_built_recipe::RecipeOperation;
use crate::source_built_recipe::RecipeOutput;
use crate::source_built_recipe::RecipeStep;
use crate::source_built_recipe::RecipeWord;
use crate::source_built_recipe::error;
use crate::source_built_stagex_programs::TransitionBuilder;
use crate::source_built_stagex_programs::input;
use crate::source_built_stagex_programs::literal;
use crate::source_built_stagex_programs::output;

const MINI: &str = "stage0-mini-kaem-exec";
const FULL: &str = "stage0-full-kaem-exec";
const MES: &str = "mes-m2-kaem-exec";
const RUNTIME: &str = "mes-runtime-exec";
const LIMIT: u64 = 256 * 1024 * 1024;

type Files = BTreeMap<String, ArtifactRef>;

fn spec(kind: ArtifactKind) -> RecipeOutput {
    RecipeOutput { kind, bytes_max: LIMIT }
}

fn definition_header(
    b: &mut TransitionBuilder,
    tool: &str,
    id: &str,
    arguments: &mut Vec<RecipeWord>,
) -> Result<Option<ArtifactRef>, RunError> {
    let mut definitions = String::new();
    let mut index = 0;
    while index + 1 < arguments.len() {
        let is_define = matches!(arguments[index].parts.as_slice(), [crate::source_built_recipe::WordPart::Literal { text }] if text == "-D");
        if is_define
            && let [crate::source_built_recipe::WordPart::Literal { text }] = arguments[index + 1].parts.as_slice()
            && (text.contains('/') || text.contains('\\'))
        {
            definitions.push_str(text);
            definitions.push('\n');
            arguments.drain(index..index + 2);
            continue;
        }
        index += 1;
    }
    if !definitions.is_empty() {
        let instructions = b.authored(&format!("instructions/{id}-defines.txt"), definitions)?;
        let helper = b.artifact("early-executable-copy")?;
        let header = exec(
            b,
            tool,
            &format!("{id}-defines"),
            helper,
            BTreeMap::from([("instructions".into(), instructions)]),
            vec![literal("defines"), input("instructions", ""), output("result", "")],
            ArtifactKind::Text,
            BTreeMap::new(),
        )?;
        return Ok(Some(header));
    }
    Ok(None)
}

fn exec(
    b: &mut TransitionBuilder,
    tool: &str,
    id: &str,
    executable: ArtifactRef,
    mut inputs: Files,
    mut arguments: Vec<RecipeWord>,
    kind: ArtifactKind,
    environment: BTreeMap<String, RecipeWord>,
) -> Result<ArtifactRef, RunError> {
    if let Some(header) = definition_header(b, tool, id, &mut arguments)? {
        inputs.insert("defines.h".into(), header);
        arguments.splice(0..0, [literal("-include"), input("defines.h", "")]);
    }
    inputs.insert("exe".into(), executable);
    let outputs = b.add(tool, RecipeStep {
        id: id.into(),
        inputs,
        outputs: BTreeMap::from([
            ("result".into(), spec(kind)),
            ("stdout".into(), spec(ArtifactKind::Text)),
            ("stderr".into(), spec(ArtifactKind::Text)),
        ]),
        operation: RecipeOperation::Exec {
            invocation_input: None,
            command_state: None,
            executable_input: "exe".into(),
            arguments,
            environment,
            runtime_mounts: BTreeMap::new(),
            output_seeds: BTreeMap::new(),
            guard: None,
            merge_stderr: false,
            cwd_input: None,
            cwd_output: None,
            stdin_input: None,
            exit_policy: RecipeExitPolicy::Exact { code: 0 },
        },
    })?;
    Ok(outputs["result"].clone())
}

fn text(b: &mut TransitionBuilder, tool: &str, id: &str, value: &str) -> Result<ArtifactRef, RunError> {
    Ok(b.add(tool, RecipeStep {
        id: id.into(),
        inputs: Files::new(),
        outputs: BTreeMap::from([("result".into(), spec(ArtifactKind::Text))]),
        operation: RecipeOperation::WriteText {
            output: "result".into(),
            text: value.into(),
        },
    })?["result"]
        .clone())
}

fn tree(b: &mut TransitionBuilder, tool: &str, id: &str, files: &Files) -> Result<ArtifactRef, RunError> {
    let mut inputs = Files::new();
    let mut entries = BTreeMap::new();
    for (index, (path, reference)) in files.iter().enumerate() {
        let name = format!("file-{index}");
        inputs.insert(name.clone(), reference.clone());
        entries.insert(path.clone(), name);
    }
    Ok(b.add(tool, RecipeStep {
        id: id.into(),
        inputs,
        outputs: BTreeMap::from([("result".into(), spec(ArtifactKind::Tree))]),
        operation: RecipeOperation::AssembleTree {
            output: "result".into(),
            entries,
        },
    })?["result"]
        .clone())
}

fn original_files(b: &mut TransitionBuilder, source: &str) -> Result<Files, RunError> {
    let root = b.source_path(source)?;
    let canonical_root = std::fs::canonicalize(&root).map_err(|e| error(e.to_string()))?;
    let mut files = Files::new();
    for member in b.members(source)? {
        let path = root.join(&member);
        let target = std::fs::canonicalize(&path)
            .map_err(|e| error(format!("resolving original member {source}:{member}: {e}")))?;
        let target = target
            .strip_prefix(&canonical_root)
            .map_err(|_| error("original source symlink escapes its authenticated tree"))?;
        if !path.is_file() {
            continue;
        }
        let target = target.to_str().ok_or_else(|| error("non-UTF8 original source path"))?;
        files.insert(member, b.member(source, target)?);
    }
    Ok(files)
}

fn normalize(cwd: &str, value: &str) -> Result<String, RunError> {
    let joined = Path::new(cwd).join(value);
    let mut parts = Vec::new();
    for part in joined.components() {
        match part {
            Component::Normal(p) => parts.push(p.to_str().ok_or_else(|| error("non-UTF8 kaem path"))?),
            Component::CurDir => (),
            Component::ParentDir => {
                parts.pop().ok_or_else(|| error("kaem path escapes original closure"))?;
            }
            _ => return Err(error("absolute path in original kaem program")),
        }
    }
    Ok(parts.join("/"))
}

fn words(line: &str, vars: &BTreeMap<String, String>) -> Result<Vec<String>, RunError> {
    let mut result = Vec::new();
    let mut word = String::new();
    let mut quote = None;
    let mut active = false;
    let mut chars = line.chars().peekable();
    while let Some(c) = chars.next() {
        match c {
            '#' if quote.is_none() && !active => break,
            '\'' | '"' if quote.is_none() => {
                quote = Some(c);
                active = true;
            }
            c if Some(c) == quote => quote = None,
            c if c.is_whitespace() && quote.is_none() => {
                if active {
                    result.push(std::mem::take(&mut word));
                    active = false;
                }
            }
            '$' if quote != Some('\'') => {
                let mut key = String::new();
                if chars.peek() == Some(&'{') {
                    chars.next();
                    loop {
                        match chars.next() {
                            Some('}') => break,
                            Some(c) => key.push(c),
                            None => return Err(error("unclosed kaem variable")),
                        }
                    }
                } else {
                    while chars.peek().is_some_and(|c| c.is_ascii_alphanumeric() || *c == '_') {
                        key.push(chars.next().unwrap());
                    }
                }
                word.push_str(vars.get(&key).ok_or_else(|| error(format!("unbound original kaem variable {key}")))?);
                active = true;
            }
            '|' | ';' | '`' | '>' | '<' if quote.is_none() => {
                return Err(error("unexpanded shell syntax in original kaem"));
            }
            _ => {
                word.push(c);
                active = true;
            }
        }
    }
    if quote.is_some() {
        return Err(error("unclosed original kaem quote"));
    }
    if active {
        result.push(word);
    }
    Ok(result)
}

/// Interpret only authoring-time assignments and paths. Every executable line
/// becomes its own action; neither kaem nor an implicit compiler driver runs.
fn kaem(
    b: &mut TransitionBuilder,
    tool: &str,
    source: &str,
    files: &mut Files,
    cwd: &mut String,
    vars: &mut BTreeMap<String, String>,
    index: &mut usize,
) -> Result<(), RunError> {
    for line in source.replace("\\\n", " ").lines() {
        let tokens = words(line, vars)?;
        if tokens.is_empty() {
            continue;
        }
        if tokens.len() == 1
            && let Some((key, value)) = tokens[0].split_once('=')
        {
            vars.insert(key.into(), value.into());
            continue;
        }
        if tokens[0] == "cd" {
            *cwd = normalize(cwd, &tokens[1])?;
            continue;
        }
        let executable_path = normalize(cwd, &tokens[0])?;
        let executable = files
            .get(&executable_path)
            .cloned()
            .ok_or_else(|| error(format!("missing original kaem executable {executable_path}")))?;
        let name = Path::new(&executable_path).file_name().and_then(|p| p.to_str()).unwrap_or("");
        let output_index = if let Some(position) = tokens.iter().position(|t| matches!(t.as_str(), "-o" | "--output")) {
            position + 1
        } else if name == "catm" {
            1
        } else {
            tokens.len() - 1
        };
        let destination = normalize(cwd, tokens.get(output_index).ok_or_else(|| error("missing kaem output"))?)?;
        let mut input_files = Files::new();
        let mut arguments = Vec::new();
        for (position, token) in tokens.iter().enumerate().skip(1) {
            if position == output_index {
                arguments.push(output("result", ""));
                continue;
            }
            let path = normalize(cwd, token)?;
            if let Some(reference) = files.get(&path) {
                input_files.insert(path.clone(), reference.clone());
                arguments.push(input("files", &path));
            } else if token.starts_with("./")
                || token.starts_with("../")
                || token.ends_with(".c")
                || token.ends_with(".M1")
                || token.ends_with(".hex2")
            {
                return Err(error(format!("missing original kaem input {path}")));
            } else {
                arguments.push(literal(token));
            }
        }
        let kind = if name == "mkdir" {
            ArtifactKind::Tree
        } else if name == "cp" {
            ArtifactKind::Object
        } else if destination.ends_with(".M1")
            || destination.ends_with(".hex2")
            || destination.ends_with(".c")
            || destination.ends_with("blood-elf-M1")
        {
            ArtifactKind::Text
        } else {
            ArtifactKind::Executable
        };
        let id = format!("{tool}-command-{:04}", *index);
        *index += 1;
        let inputs = if input_files.is_empty() {
            Files::new()
        } else {
            let reference = tree(b, tool, &format!("{id}-inputs"), &input_files)?;
            BTreeMap::from([("files".into(), reference)])
        };
        let reference = exec(b, tool, &id, executable, inputs, arguments, kind, BTreeMap::new())?;
        let reference = if name == "cp" {
            executable_copy(b, tool, &format!("{id}-executable-mode"), reference)?
        } else {
            reference
        };
        files.insert(destination, reference);
    }
    Ok(())
}

fn stage0(b: &mut TransitionBuilder) -> Result<(), RunError> {
    let mut files = Files::new();
    for (id, prefix) in [
        ("stage0-posix-amd64-source", "AMD64"),
        ("bootstrap-seeds-source", "bootstrap-seeds"),
        ("m2-planet-source", "M2-Planet"),
        ("m2libc-source", "M2libc"),
        ("mescc-tools-source", "mescc-tools"),
        ("mescc-tools-extra-source", "mescc-tools-extra"),
        ("m2-mesoplanet-source", "M2-Mesoplanet"),
    ] {
        for (path, reference) in original_files(b, id)? {
            files.insert(format!("{prefix}/{path}"), reference);
        }
    }
    files.insert("AMD64/artifact/hex0".into(), b.artifact("hex0-reproduced")?);
    let mini = b.read_source("stage0-posix-amd64-source", "mescc-tools-mini-kaem.kaem")?;
    let mut index = 0;
    kaem(b, MINI, &mini, &mut files, &mut String::new(), &mut BTreeMap::new(), &mut index)?;
    for expected in crate::stagex_stage0::STAGE0_EXPECTED_EXECUTABLES {
        if expected.artifact_id != "stage0-hex0-copy" {
            b.publish(
                MINI,
                expected.artifact_id,
                files
                    .get(expected.relative_path)
                    .cloned()
                    .ok_or_else(|| error(format!("missing mini output {}", expected.artifact_id)))?,
            )?;
        }
    }
    let mut vars = BTreeMap::from([
        ("ARCH".into(), "amd64".into()),
        ("ARCH_DIR".into(), "AMD64".into()),
        ("ENDIAN_FLAG".into(), "--little-endian".into()),
        ("BLOOD_FLAG".into(), "--64".into()),
        ("BASE_ADDRESS".into(), "0x00600000".into()),
    ]);
    let full = b.read_source("stage0-posix-amd64-source", "mescc-tools-full-kaem.kaem")?;
    kaem(b, FULL, &full, &mut files, &mut String::new(), &mut vars, &mut index)?;
    for expected in crate::stagex_stage0_full::STAGE0_FULL_EXPECTED_EXECUTABLES {
        if expected.artifact_id.starts_with("stage0-full-")
            && let Some(reference) = files.get(expected.relative_path)
        {
            b.publish(FULL, expected.artifact_id, reference.clone())?;
        }
    }
    for name in [
        "sha256sum",
        "match",
        "mkdir",
        "untar",
        "ungz",
        "unbz2",
        "unxz",
        "catm",
        "cp",
        "chmod",
        "rm",
        "replace",
        "wrap",
    ] {
        let source = b.member("mescc-tools-extra-source", &format!("{name}.c"))?;
        let executable = m2_program(b, FULL, &format!("stage0-extra-{name}"), source, &format!("{name}.c"))?;
        files.insert(format!("AMD64/bin/{name}"), executable);
    }
    for expected in crate::stagex_stage0_full::STAGE0_FULL_EXPECTED_EXECUTABLES {
        if expected.artifact_id.starts_with("stage0-full-") && !b.artifacts.contains_key(expected.artifact_id) {
            b.publish(
                FULL,
                expected.artifact_id,
                files
                    .get(expected.relative_path)
                    .cloned()
                    .ok_or_else(|| error(format!("missing full output {}", expected.artifact_id)))?,
            )?;
        }
    }
    let binaries = files
        .iter()
        .filter(|(path, _)| path.starts_with("AMD64/bin/"))
        .map(|(path, reference)| (path.clone(), reference.clone()))
        .collect();
    let binaries = tree(b, FULL, "stage0-full-answer-inputs", &binaries)?;
    let executable = b.artifact("stage0-full-sha256sum")?;
    let answers = b.source("stage0-amd64-answers")?;
    let after = b.source("stage0-after-script")?;
    b.add(FULL, RecipeStep {
        id: "stage0-full-check-answers".into(),
        inputs: BTreeMap::from([
            ("exe".into(), executable),
            ("answers".into(), answers),
            ("binaries".into(), binaries),
            ("after".into(), after),
        ]),
        outputs: BTreeMap::from([
            ("stdout".into(), spec(ArtifactKind::Text)),
            ("stderr".into(), spec(ArtifactKind::Text)),
        ]),
        operation: RecipeOperation::Exec {
            invocation_input: None,
            command_state: None,
            executable_input: "exe".into(),
            arguments: vec![literal("-c"), input("answers", "")],
            environment: BTreeMap::new(),
            runtime_mounts: BTreeMap::new(),
            output_seeds: BTreeMap::new(),
            guard: None,
            merge_stderr: false,
            cwd_input: Some("binaries".into()),
            cwd_output: None,
            stdin_input: None,
            exit_policy: RecipeExitPolicy::Exact { code: 0 },
        },
    })?;
    Ok(())
}

fn m2_program(
    b: &mut TransitionBuilder,
    tool: &str,
    id: &str,
    source: ArtifactRef,
    source_name: &str,
) -> Result<ArtifactRef, RunError> {
    let libc = b.fetcher(tool, "m2libc-source")?;
    let inputs = BTreeMap::from([(source_name.into(), source), ("libc".into(), libc)]);
    let preprocessor = b.artifact("stage0-full-m2-mesoplanet")?;
    let preprocessed = exec(
        b,
        tool,
        &format!("{id}-preprocess"),
        preprocessor,
        inputs,
        vec![
            literal("--architecture"),
            literal("amd64"),
            literal("--operating-system"),
            literal("Linux"),
            literal("-E"),
            literal("-f"),
            input(source_name, ""),
            literal("-o"),
            output("result", ""),
        ],
        ArtifactKind::Text,
        BTreeMap::from([("M2LIBC_PATH".into(), input("libc", ""))]),
    )?;
    let inputs = BTreeMap::from([("source".into(), preprocessed)]);
    let m2 = b.artifact("stage0-full-m2-planet")?;
    let assembly = exec(
        b,
        tool,
        &format!("{id}-compile"),
        m2,
        inputs,
        vec![
            literal("--architecture"),
            literal("amd64"),
            literal("--debug"),
            literal("-f"),
            input("source", ""),
            literal("-o"),
            output("result", ""),
        ],
        ArtifactKind::Text,
        BTreeMap::new(),
    )?;
    let blood = b.artifact("stage0-full-blood-elf")?;
    let footer = exec(
        b,
        tool,
        &format!("{id}-debug"),
        blood,
        BTreeMap::from([("source".into(), assembly.clone())]),
        vec![
            literal("--64"),
            literal("--little-endian"),
            literal("-f"),
            input("source", ""),
            literal("-o"),
            output("result", ""),
        ],
        ArtifactKind::Text,
        BTreeMap::new(),
    )?;
    let definitions = b.member("m2libc-source", "amd64/amd64_defs.M1")?;
    let libc = b.member("m2libc-source", "amd64/libc-full.M1")?;
    let m1 = b.artifact("stage0-m1")?;
    let hex = exec(
        b,
        tool,
        &format!("{id}-assemble"),
        m1,
        BTreeMap::from([
            ("definitions".into(), definitions),
            ("libc".into(), libc),
            ("source".into(), assembly),
            ("footer".into(), footer),
        ]),
        vec![
            literal("--architecture"),
            literal("amd64"),
            literal("--little-endian"),
            literal("-f"),
            input("definitions", ""),
            literal("-f"),
            input("libc", ""),
            literal("-f"),
            input("source", ""),
            literal("-f"),
            input("footer", ""),
            literal("-o"),
            output("result", ""),
        ],
        ArtifactKind::Text,
        BTreeMap::new(),
    )?;
    let header = b.member("m2libc-source", "amd64/ELF-amd64-debug.hex2")?;
    let hex2 = b.artifact("stage0-hex2")?;
    exec(
        b,
        tool,
        &format!("{id}-link"),
        hex2,
        BTreeMap::from([("header".into(), header), ("source".into(), hex)]),
        vec![
            literal("--architecture"),
            literal("amd64"),
            literal("--little-endian"),
            literal("--base-address"),
            literal("0x00600000"),
            literal("-f"),
            input("header", ""),
            literal("-f"),
            input("source", ""),
            literal("-o"),
            output("result", ""),
        ],
        ArtifactKind::Executable,
        BTreeMap::new(),
    )
}

fn replace(
    b: &mut TransitionBuilder,
    tool: &str,
    id: &str,
    source: ArtifactRef,
    old: &str,
    new: &str,
) -> Result<ArtifactRef, RunError> {
    let executable = b.artifact("early-executable-copy")?;
    let old = b.authored(&format!("instructions/{id}-match.txt"), old.into())?;
    let new = b.authored(&format!("instructions/{id}-replacement.txt"), new.into())?;
    exec(
        b,
        tool,
        id,
        executable,
        BTreeMap::from([
            ("source".into(), source),
            ("match".into(), old),
            ("replacement".into(), new),
        ]),
        vec![
            literal("replace"),
            input("source", ""),
            output("result", ""),
            input("match", ""),
            input("replacement", ""),
        ],
        ArtifactKind::Text,
        BTreeMap::new(),
    )
}

fn concat(
    b: &mut TransitionBuilder,
    tool: &str,
    id: &str,
    references: &[ArtifactRef],
    kind: ArtifactKind,
) -> Result<ArtifactRef, RunError> {
    let executable = b.artifact("stage0-full-catm")?;
    let mut inputs = Files::new();
    let mut args = vec![output("result", "")];
    for (index, reference) in references.iter().enumerate() {
        let key = format!("part-{index}");
        inputs.insert(key.clone(), reference.clone());
        args.push(input(&key, ""));
    }
    exec(b, tool, id, executable, inputs, args, kind, BTreeMap::new())
}

fn mes_sources(b: &mut TransitionBuilder, tool: &str, source: &str, prefix: &str) -> Result<Files, RunError> {
    let mut files = original_files(b, source)?;
    let config = text(b, tool, &format!("{prefix}-config"), "#undef SYSTEM_LIBC\n#define MES_VERSION \"0.27.1\"\n")?;
    files.insert("include/mes/config.h".into(), config);
    for name in ["kernel-stat.h", "signal.h", "syscall.h"] {
        let reference = files
            .get(&format!("include/linux/x86_64/{name}"))
            .cloned()
            .ok_or_else(|| error("missing Mes architecture header"))?;
        files.insert(format!("include/arch/{name}"), reference);
    }
    files.remove("mes/module/mes/psyntax.pp");
    files.remove("mes/module/mes/psyntax.pp.header");
    for (old, new) in [
        ("mes/module/srfi/srfi-9-struct.mes", "mes/module/srfi/srfi-9.mes"),
        ("mes/module/srfi/srfi-9/gnu-struct.mes", "mes/module/srfi/srfi-9/gnu.mes"),
    ] {
        let reference = files.get(old).cloned().ok_or_else(|| error(format!("missing Mes bootstrap module {old}")))?;
        files.insert(new.into(), reference);
    }
    Ok(files)
}

fn mes_bootstrap(b: &mut TransitionBuilder) -> Result<Files, RunError> {
    let helper = b.authored("helpers/stagex-early-copy.c", EARLY_COPY_SOURCE.into())?;
    let helper = m2_program(b, MES, "early-executable-copy-build", helper, "stagex-early-copy.c")?;
    b.artifacts.insert("early-executable-copy".into(), helper);
    let mut files = mes_sources(b, MES, "mes-0.27.1-source", "mes-bootstrap")?;
    let wait = files.get("lib/linux/wait4.c").cloned().ok_or_else(|| error("missing original wait4.c"))?;
    let wait = replace(
        b,
        MES,
        "mes-wait4-status",
        wait,
        "  long long_rusage = cast_voidp_to_long (rusage);",
        "  long long_rusage = cast_voidp_to_long (rusage);\n  *long_status_ptr = 0;",
    )?;
    files.insert("lib/linux/wait4.c".into(), wait);
    for (name, id) in [
        ("M2-Planet", "stage0-full-m2-planet"),
        ("blood-elf", "stage0-full-blood-elf"),
        ("M1", "stage0-m1"),
        ("hex2", "stage0-hex2"),
        ("mkdir", "stage0-full-mkdir"),
        ("cp", "stage0-full-cp"),
    ] {
        files.insert(name.into(), b.artifact(id)?);
    }
    let script = b.read_source("mes-0.27.1-source", "kaem.run")?;
    let start = script.find("mkdir -p m2").ok_or_else(|| error("original Mes script lacks bootstrap boundary"))?;
    let script = script[start..].replace("0x1000000", "0x8048000");
    let script = script
        .lines()
        .filter(|line| {
            let line = line.trim();
            !line.starts_with("echo Running")
                && !line.starts_with("GUILE_LOAD_PATH=")
                && !line.starts_with("./bin/mes-m2 -c ")
        })
        .collect::<Vec<_>>()
        .join("\n");
    let mut vars = BTreeMap::from([
        ("cc_cpu".into(), "x86_64".into()),
        ("mes_cpu".into(), "x86_64".into()),
        ("stage0_cpu".into(), "amd64".into()),
        ("blood_elf_flag".into(), "--64".into()),
        ("srcdest".into(), String::new()),
    ]);
    kaem(b, MES, &script, &mut files, &mut String::new(), &mut vars, &mut 0)?;
    for (id, path) in [("mes-m2", "bin/mes-m2"), ("mes", "bin/mes")] {
        b.publish(MES, id, files.get(path).cloned().ok_or_else(|| error(format!("missing Mes output {path}")))?)?;
    }
    let includes = files
        .iter()
        .filter_map(|(path, reference)| path.strip_prefix("include/").map(|p| (p.to_string(), reference.clone())))
        .collect();
    let include = tree(b, MES, "mes-bootstrap-includes", &includes)?;
    b.artifacts.insert("mes-include-tree".into(), include);
    let modules = tree(b, MES, "mes-bootstrap-smoke-modules", &files)?;
    let mut environment =
        mes_env("modules", path_list(&[input("modules", "mes/module"), input("modules", "module")]), "20000000");
    environment.insert("MES_STACK".into(), literal("6000000"));
    b.add(MES, RecipeStep {
        id: "mes-bootstrap-smoke".into(),
        inputs: BTreeMap::from([("exe".into(), b.artifact("mes-m2")?), ("modules".into(), modules)]),
        outputs: BTreeMap::from([
            ("stdout".into(), spec(ArtifactKind::Text)),
            ("stderr".into(), spec(ArtifactKind::Text)),
        ]),
        operation: RecipeOperation::Exec {
            invocation_input: None,
            command_state: None,
            executable_input: "exe".into(),
            arguments: vec![literal("-c"), literal("(display 'Hello,M2-mes!) (newline)")],
            environment,
            runtime_mounts: BTreeMap::new(),
            output_seeds: BTreeMap::new(),
            guard: None,
            merge_stderr: false,
            cwd_input: None,
            cwd_output: None,
            stdin_input: None,
            exit_policy: RecipeExitPolicy::Exact { code: 0 },
        },
    })?;
    Ok(files)
}

fn copy_member(
    b: &mut TransitionBuilder,
    tool: &str,
    id: &str,
    reference: ArtifactRef,
    path: &str,
    kind: ArtifactKind,
) -> Result<ArtifactRef, RunError> {
    let executable = b.artifact("stage0-full-catm")?;
    exec(
        b,
        tool,
        id,
        executable,
        BTreeMap::from([("source".into(), reference)]),
        vec![output("result", ""), input("source", path)],
        kind,
        BTreeMap::new(),
    )
}

fn mes_env(prefix: &str, load: RecipeWord, arena: &str) -> BTreeMap<String, RecipeWord> {
    BTreeMap::from([
        ("MES_PREFIX".into(), input(prefix, "")),
        ("GUILE_LOAD_PATH".into(), load),
        ("MES_ARENA".into(), literal(arena)),
        ("MES_MAX_ARENA".into(), literal(arena)),
        ("MES_STACK".into(), literal("15000000")),
    ])
}

fn path_list(parts: &[RecipeWord]) -> RecipeWord {
    let mut word = RecipeWord { parts: Vec::new() };
    for (index, part) in parts.iter().enumerate() {
        if index > 0 {
            word.parts.extend(literal(":").parts);
        }
        word.parts.extend(part.parts.clone());
    }
    word
}

fn mes_runtime(b: &mut TransitionBuilder, mut files: Files) -> Result<ArtifactRef, RunError> {
    let mut nyacc = original_files(b, "nyacc-1.00.2-source")?;
    let generated = [
        "c99-act.scm",
        "c99-tab.scm",
        "c99cx-act.scm",
        "c99cx-tab.scm",
        "c99x-act.scm",
        "c99x-tab.scm",
        "cpp-act.scm",
        "cpp-tab.scm",
    ];
    for file in generated {
        nyacc.remove(&format!("module/nyacc/lang/c99/mach.d/{file}"));
    }
    let util = nyacc
        .get("module/nyacc/lang/util.scm")
        .cloned()
        .ok_or_else(|| error("missing NYACC generator utility"))?;
    let util = replace(b, RUNTIME, "nyacc-output-directory", util, "\"/mach.d/\"", "\"/\"")?;
    nyacc.insert("module/nyacc/lang/util.scm".into(), util);
    // Mes' compatibility modules precede NYACC on the original load path.
    // Redirect an original utility there as well if it defines mach-dir.
    for path in ["mes/module/nyacc/lang/util.scm", "module/nyacc/lang/util.scm"] {
        if let Some(reference) = files.get(path).cloned() {
            let source = b.read_source("mes-0.27.1-source", path)?;
            if source.contains("\"/mach.d/\"") {
                let reference = replace(
                    b,
                    RUNTIME,
                    &format!("nyacc-{}-output-directory", path.replace(['/', '.'], "-")),
                    reference,
                    "\"/mach.d/\"",
                    "\"/\"",
                )?;
                files.insert(path.into(), reference);
            }
        }
    }
    let mes_source_tree = tree(b, RUNTIME, "mes-nyacc-bootstrap-modules", &files)?;
    for (name, arena, outputs) in [
        ("cpp", "20000000", vec!["cpp-act.scm", "cpp-tab.scm"]),
        ("c99", "32000000", vec!["c99-act.scm", "c99-tab.scm", "c99x-act.scm", "c99x-tab.scm"]),
        ("c99cx", "20000000", vec!["c99cx-act.scm", "c99cx-tab.scm"]),
    ] {
        let source = b.member("nyacc-1.00.2-source", &format!("gen-{name}-files.scm"))?;
        let script = replace(
            b,
            RUNTIME,
            &format!("nyacc-{name}-output-path"),
            source,
            "\"module/nyacc/lang/c99\"",
            "\"/outputs/result\"",
        )?;
        let modules = tree(b, RUNTIME, &format!("nyacc-{name}-modules"), &nyacc)?;
        let mes = b.artifact("mes-m2")?;
        let mut environment = mes_env(
            "mes-source",
            path_list(&[
                input("mes-source", "mes/module"),
                input("mes-source", "module"),
                input("nyacc", "module"),
            ]),
            arena,
        );
        environment.insert("MES_MAX_ARENA".into(), literal("20000000"));
        environment.insert("MES_STACK".into(), literal("6000000"));
        let produced = b.add(RUNTIME, RecipeStep {
            id: format!("nyacc-{name}-generate"),
            inputs: BTreeMap::from([
                ("exe".into(), mes),
                ("script".into(), script),
                ("mes-source".into(), mes_source_tree.clone()),
                ("nyacc".into(), modules),
            ]),
            outputs: BTreeMap::from([
                ("result".into(), spec(ArtifactKind::Tree)),
                ("stdout".into(), spec(ArtifactKind::Text)),
                ("stderr".into(), spec(ArtifactKind::Text)),
            ]),
            operation: RecipeOperation::Exec {
                invocation_input: None,
                command_state: None,
                executable_input: "exe".into(),
                arguments: vec![input("script", "")],
                environment,
                runtime_mounts: BTreeMap::new(),
                output_seeds: BTreeMap::new(),
                guard: None,
                merge_stderr: false,
                cwd_input: None,
                cwd_output: Some("result".into()),
                stdin_input: None,
                exit_policy: RecipeExitPolicy::Exact { code: 0 },
            },
        })?["result"]
            .clone();
        for output_name in outputs {
            let reference = copy_member(
                b,
                RUNTIME,
                &format!("nyacc-{name}-table-{}", output_name.replace('.', "-")),
                produced.clone(),
                output_name,
                ArtifactKind::Text,
            )?;
            nyacc.insert(format!("module/nyacc/lang/c99/mach.d/{output_name}"), reference);
        }
    }
    let mut installed = Files::new();
    for (path, reference) in &files {
        if path.starts_with("include/") {
            installed.insert(path.replacen("include/", "include/mes-include/", 1), reference.clone());
        } else if path.starts_with("mes/module/") {
            installed.insert(path.clone(), reference.clone());
        } else if let Some(path) = path.strip_prefix("module/mescc") {
            installed.insert(format!("mes/module/mescc{path}"), reference.clone());
        } else if [
            "module/mes/getopt-long.scm",
            "module/mes/misc.scm",
            "module/mes/test.scm",
        ]
        .contains(&path.as_str())
        {
            installed.insert(format!("mes/{path}"), reference.clone());
        }
    }
    for (path, reference) in nyacc {
        if let Some(path) = path.strip_prefix("module/") {
            installed.insert(format!("mes/module/{path}"), reference);
        }
    }
    for path in [
        "nyacc/compat18.scm",
        "nyacc/lang/c99/pprint.scm",
        "rnrs/arithmetic/bitwise.scm",
        "system/foreign.scm",
    ] {
        installed.insert(
            format!("mes/module/{path}"),
            files
                .get(&format!("mes/module/{path}"))
                .cloned()
                .ok_or_else(|| error("missing original Mes-safe module"))?,
        );
    }
    let compile = installed
        .get("mes/module/mescc/compile.scm")
        .cloned()
        .ok_or_else(|| error("missing Mes compiler module"))?;
    let compile = replace(
        b,
        RUNTIME,
        "mescc-array-type",
        compile,
        "(make-c-array (rank+= type rank) count)",
        "(make-c-array (rank+= (ast->type type info) rank) count)",
    )?;
    installed.insert("mes/module/mescc/compile.scm".into(), compile);
    installed.insert(
        "mes/module/ice-9/syncase.scm".into(),
        text(b, RUNTIME, "mes-syncase-module", "(define-module (ice-9 syncase))\n")?,
    );
    let mut entrypoint = b.member("mes-0.27.1-source", "scripts/mescc.scm.in")?;
    for (index, (old, new)) in [
        ("@prefix@", "/stagex/mes-runtime"),
        ("@VERSION@", "0.27.1"),
        ("@mes_cpu@", "x86_64"),
        ("@mes_kernel@", "linux"),
        ("@guile_site_dir@", "/stagex/mes-runtime/mes/module"),
        ("@guile_site_ccache_dir@", "/stagex/mes-runtime/mes/module"),
        ("@GUILE@", "/stagex/mes-runtime/bin/mes-m2"),
    ]
    .iter()
    .enumerate()
    {
        entrypoint = replace(b, RUNTIME, &format!("mescc-template-{index}"), entrypoint, old, new)?;
    }
    b.publish(RUNTIME, "mescc-entrypoint", entrypoint.clone())?;
    installed.insert("bin/mescc.scm".into(), entrypoint);
    installed.insert("bin/mes-m2".into(), b.artifact("mes-m2")?);
    for (id, path) in [
        ("mes-x86_64-m1", "lib/x86_64-mes/x86_64.M1"),
        ("mes-elf-header", "lib/linux/x86_64-mes/elf64-header.hex2"),
        ("mes-elf-footer", "lib/linux/x86_64-mes/elf64-footer-single-main.hex2"),
    ] {
        let source = files.get(path).cloned().ok_or_else(|| error(format!("missing original Mes runtime {path}")))?;
        let reference = concat(b, RUNTIME, id, &[source], ArtifactKind::Text)?;
        b.publish(RUNTIME, id, reference.clone())?;
        installed.insert(path.into(), reference);
    }
    let modules = tree(b, RUNTIME, "mes-installed-compiler-modules", &installed)?;
    let source_tree = tree(b, RUNTIME, "mes-runtime-original-sources", &files)?;
    let configure = b.read_source("mes-0.27.1-source", "build-aux/configure-lib.sh")?;
    let libraries = crate::stagex_mes_lib::derive_mes_library_sources(&configure).map_err(|e| error(e.to_string()))?;
    let mut compiled = BTreeMap::<String, (ArtifactRef, ArtifactRef)>::new();
    for library in libraries {
        let mut objects = Vec::new();
        let mut assemblies = Vec::new();
        for source in library.source_paths {
            let pair = if let Some(pair) = compiled.get(&source) {
                pair.clone()
            } else {
                let id = format!("mes-runtime-{}", source.replace(['/', '.'], "-"));
                let flags = vec![
                    literal("-D"),
                    literal("HAVE_CONFIG_H=1"),
                    literal("-I"),
                    input("source", "include"),
                    literal("-I"),
                    input("source", "include/linux/x86_64"),
                ];
                let assembly = mes_compile(b, RUNTIME, &id, modules.clone(), source_tree.clone(), &source, flags)?;
                let object = mes_assemble(b, RUNTIME, &format!("{id}-assemble"), assembly.clone())?;
                compiled.insert(source.clone(), (object.clone(), assembly.clone()));
                (object, assembly)
            };
            objects.push(pair.0);
            assemblies.push(pair.1);
        }
        let id = match library.name {
            "libc-mini.a" => "mes-libc-mini",
            "libmes.a" => "mes-libmes",
            "libmescc.a" => "mes-libmescc",
            "libc.a" => "mes-libc",
            "libc+tcc.a" => "mes-libc-tcc",
            "libtcc1.a" => "mes-libtcc1",
            "libgetopt.a" => "mes-libgetopt",
            "crt1.o" => "mes-crt1",
            _ => return Err(error("unrecognized original Mes library")),
        };
        let object = concat(
            b,
            RUNTIME,
            id,
            &objects,
            if library.name == "crt1.o" {
                ArtifactKind::Object
            } else {
                ArtifactKind::Archive
            },
        )?;
        b.publish(RUNTIME, id, object.clone())?;
        installed.insert(format!("lib/x86_64-mes/{}", library.name), object);
        let assembly = concat(b, RUNTIME, &format!("{id}-assembly"), &assemblies, ArtifactKind::Text)?;
        installed.insert(format!("lib/x86_64-mes/{}", Path::new(library.name).with_extension("s").display()), assembly);
    }
    let result = tree(b, RUNTIME, "mes-runtime-installed-tree", &installed)?;
    b.artifacts.insert("mes-runtime-tree".into(), result.clone());
    Ok(result)
}

fn mes_compile(
    b: &mut TransitionBuilder,
    tool: &str,
    id: &str,
    modules: ArtifactRef,
    source_tree: ArtifactRef,
    path: &str,
    mut extra: Vec<RecipeWord>,
) -> Result<ArtifactRef, RunError> {
    let executable = b.artifact("mes-m2")?;
    let mut inputs = BTreeMap::from([("modules".into(), modules), ("source".into(), source_tree.clone())]);
    let unit = if let Some(header) = definition_header(b, tool, id, &mut extra)? {
        // MesCC has no -include option. Preserve its original include search
        // tree while prepending the exact logical path definitions to the unit.
        let source = copy_member(b, tool, &format!("{id}-original-unit"), source_tree, path, ArtifactKind::Text)?;
        let unit = concat(b, tool, &format!("{id}-configured-unit"), &[header, source], ArtifactKind::Text)?;
        inputs.insert("unit.c".into(), unit);
        input("unit.c", "")
    } else {
        input("source", path)
    };
    let mut arguments = vec![
        literal("--no-auto-compile"),
        literal("-e"),
        literal("main"),
        input("modules", "bin/mescc.scm"),
        literal("--"),
        literal("-S"),
        literal("-o"),
        output("result", ""),
    ];
    arguments.extend(extra);
    arguments.push(unit);
    exec(
        b,
        tool,
        &format!("{id}-compile"),
        executable,
        inputs,
        arguments,
        ArtifactKind::Text,
        mes_env("modules", input("modules", "mes/module"), "30000000"),
    )
}

fn mes_assemble(
    b: &mut TransitionBuilder,
    tool: &str,
    id: &str,
    assembly: ArtifactRef,
) -> Result<ArtifactRef, RunError> {
    let executable = b.artifact("stage0-m1")?;
    let definitions = b.artifact("mes-x86_64-m1")?;
    exec(
        b,
        tool,
        id,
        executable,
        BTreeMap::from([("definitions".into(), definitions), ("source".into(), assembly)]),
        vec![
            literal("--little-endian"),
            literal("--architecture"),
            literal("amd64"),
            literal("-f"),
            input("definitions", ""),
            literal("-f"),
            input("source", ""),
            literal("-o"),
            output("result", ""),
        ],
        ArtifactKind::Object,
        BTreeMap::new(),
    )
}

fn tcc26_sources(b: &mut TransitionBuilder) -> Result<Files, RunError> {
    let tool = "tinycc-mes-exec";
    let mut files = original_files(b, "tinycc-0.9.26-source")?;
    let patches = [
        (
            "tcctools.c",
            "    if ((fh = fopen(argv[i_lib], \"wb\")) == NULL)\n    {\n        fprintf(stderr, \"tcc: ar: can't open file %s \\n\", argv[i_lib]);\n        goto the_end;\n    }\n\n",
            "",
        ),
        (
            "tcctools.c",
            "    // write header",
            "    if ((fh = fopen(argv[i_lib], \"wb\")) == NULL)\n    {\n        fprintf(stderr, \"tcc: ar: cannot open file %s \\n\", argv[i_lib]);\n        goto the_end;\n    }\n\n    // write header",
        ),
        ("tccelf.c", "const char filename[]", "const char *filename"),
        (
            "x86_64-gen.c",
            "SValue tmp = vtop[0];",
            "SValue tmp;\n            memcpy(&tmp, &vtop[0], sizeof(SValue));",
        ),
        ("x86_64-gen.c", "tmp = vtop[0];", "memcpy(&tmp, &vtop[0], sizeof(SValue));"),
        ("x86_64-gen.c", "vtop[0] = vtop[-i];", "memcpy(&vtop[0], &vtop[-i], sizeof(SValue));"),
        ("x86_64-gen.c", "vtop[-i] = tmp;", "memcpy(&vtop[-i], &tmp, sizeof(SValue));"),
        ("tccpp.c", "size = size * 2;", "size = size + size;"),
        ("tccelf.c", "size = size * 2;", "size = size + size;"),
        ("libtcc.c", "nb_alloc = nb * 2;", "nb_alloc = nb + nb;"),
        ("tccpp.c", "next->size * 2", "next->size + next->size"),
        ("tccelf.c", "rebuild_hash(s, 2 * nbuckets);", "rebuild_hash(s, nbuckets + nbuckets);"),
        ("x86_64-gen.c", "loc = 0;", "loc = 0;\n    ind = cur_text_section->data_offset;"),
        ("x86_64-gen.c", "g(vtop->c.i & (ll ? 63 : 31));", "g(vtop->c.i);"),
        (
            "libtcc.c",
            "        snprintf(buf, sizeof(buf), fmt, paths[i], filename);",
            "        if (fmt[3] == 'l') {\n            strcpy(buf, paths[i]); strcat(buf, \"/lib\"); strcat(buf, filename); strcat(buf, \".a\");\n        } else {\n            strcpy(buf, paths[i]); strcat(buf, \"/\"); strcat(buf, filename);\n        }",
        ),
        (
            "tccpp.c",
            "    sprintf(buf, \"\\\"%s\\\"\", file->filename);",
            "    buf[0] = '\"'; strcpy(buf + 1, file->filename); strcat(buf, \"\\\"\");",
        ),
        (
            "libtcc.c",
            "ST_FUNC int tcc_add_crt(TCCState *s, const char *filename)\n{\n    if (-1 == tcc_add_library_internal(s, \"%s/%s\",\n        filename, 0, s->crt_paths, s->nb_crt_paths))\n        tcc_error_noabort(\"file '%s' not found\", filename);\n    return 0;\n}",
            "ST_FUNC int tcc_add_crt(TCCState *s, const char *filename)\n{\n    char buf[1024];\n    int i;\n    if (!strcmp(filename, \"crti.o\") || !strcmp(filename, \"crtn.o\"))\n        return 0;\n    for (i = 0; i < s->nb_crt_paths; i++) {\n        strcpy(buf, s->crt_paths[i]); strcat(buf, \"/\"); strcat(buf, filename);\n        if (tcc_add_file_internal(s, buf, AFF_TYPE_BIN) == 0)\n            return 0;\n    }\n    tcc_error_noabort(\"file '%s' not found\", filename);\n    return 0;\n}",
        ),
        (
            "tccgen.c",
            "if (!is_compatible_types(&sym->type, type))",
            "if (0 && !is_compatible_types(&sym->type, type))",
        ),
        (
            "tccelf.c",
            "    if (file_type == TCC_OUTPUT_EXE && s1->static_link)\n        fill_got(s1);",
            "    if (file_type == TCC_OUTPUT_EXE && s1->static_link) {\n        if (s1->plt)\n            relocate_plt(s1);\n        fill_got(s1);\n    }",
        ),
    ];
    for (index, (path, old, new)) in patches.iter().enumerate() {
        let original = files.get(*path).cloned().ok_or_else(|| error("missing original TinyCC patch target"))?;
        let reference = replace(b, tool, &format!("tinycc26-source-patch-{index}"), original, old, new)?;
        files.insert(path.to_string(), reference);
    }
    files.insert("config.h".into(), text(b, tool, "tinycc26-config", "")?);
    Ok(files)
}

fn unified_libc(b: &mut TransitionBuilder, tool: &str, id: &str, files: &Files) -> Result<ArtifactRef, RunError> {
    let configure = b.read_source("tinycc-mes-0.27.1-source", "build-aux/configure-lib.sh")?;
    let sources =
        crate::stagex_mes_lib::derive_mes_unified_libc_sources(&configure).map_err(|e| error(e.to_string()))?;
    let parts = sources
        .iter()
        .map(|path| files.get(path).cloned().ok_or_else(|| error(format!("missing unified libc source {path}"))))
        .collect::<Result<Vec<_>, _>>()?;
    concat(b, tool, id, &parts, ArtifactKind::Text)
}

fn tcc_defines(version: &str, boot: bool) -> Vec<RecipeWord> {
    let prefix = format!("/stagex/tinycc-{version}");
    let mut args = Vec::new();
    for value in [
        "BOOTSTRAP=1".into(),
        "HAVE_LONG_LONG=1".into(),
        "TCC_TARGET_X86_64=1".into(),
        format!("CONFIG_TCCDIR=\"{prefix}/lib/mes/tcc\""),
        format!("CONFIG_TCC_CRTPREFIX=\"{prefix}/lib/mes\""),
        "CONFIG_TCC_ELFINTERP=\"/mes/loader\"".into(),
        format!("CONFIG_TCC_SYSINCLUDEPATHS=\"{prefix}/include/mes\""),
        format!("TCC_LIBGCC=\"{prefix}/lib/mes/libc.a\""),
        "CONFIG_TCCBOOT=1".into(),
        "CONFIG_TCC_STATIC=1".into(),
        "CONFIG_USE_LIBGCC=1".into(),
        format!("TCC_VERSION=\"{version}\""),
        "ONE_SOURCE=1".into(),
    ] {
        args.push(literal("-D"));
        args.push(literal(&value));
    }
    if boot {
        for value in [
            "HAVE_FLOAT=1".to_string(),
            "HAVE_BITFIELD=1".into(),
            "HAVE_SETJMP=1".into(),
            "TCC_LIBTCC1=\"libtcc1.a\"".into(),
            format!("CONFIG_TCC_LIBPATHS=\"{prefix}/lib/mes:{prefix}/lib/mes/tcc\""),
        ] {
            args.push(literal("-D"));
            args.push(literal(&value));
        }
    } else {
        for value in ["inline=", "CONFIG_SYSROOT=\"/\"", "CONFIG_TCC_LIBTCC1_MES=0"] {
            args.push(literal("-D"));
            args.push(literal(value));
        }
    }
    args
}

fn mes_tcc(
    b: &mut TransitionBuilder,
    tool: &str,
    id: &str,
    modules: ArtifactRef,
    sources: ArtifactRef,
    libc_tcc: ArtifactRef,
) -> Result<ArtifactRef, RunError> {
    let mut extra = tcc_defines("0.9.26", false);
    extra.extend([
        literal("-I"),
        input("source", "include"),
        literal("-I"),
        input("source", ""),
    ]);
    let assembly = mes_compile(b, tool, id, modules, sources, "tcc.c", extra)?;
    let object = mes_assemble(b, tool, &format!("{id}-assemble"), assembly)?;
    let mut inputs = BTreeMap::from([("object".into(), object), ("libc-tcc".into(), libc_tcc)]);
    for (name, artifact) in [
        ("crt1", "mes-crt1"),
        ("libmescc", "mes-libmescc"),
        ("libc", "mes-libc"),
        ("header", "mes-elf-header"),
        ("footer", "mes-elf-footer"),
    ] {
        inputs.insert(name.into(), b.artifact(artifact)?);
    }
    let executable = b.artifact("stage0-hex2")?;
    exec(
        b,
        tool,
        &format!("{id}-link"),
        executable,
        inputs,
        vec![
            literal("--little-endian"),
            literal("--architecture"),
            literal("amd64"),
            literal("--base-address"),
            literal("0x08048000"),
            literal("-f"),
            input("header", ""),
            literal("-f"),
            input("crt1", ""),
            literal("-f"),
            input("object", ""),
            literal("-f"),
            input("libc-tcc", ""),
            literal("-f"),
            input("libmescc", ""),
            literal("-f"),
            input("libc", ""),
            literal("-f"),
            input("footer", ""),
            literal("-o"),
            output("result", ""),
        ],
        ArtifactKind::Executable,
        BTreeMap::new(),
    )
}

fn c_compile(
    b: &mut TransitionBuilder,
    tool: &str,
    id: &str,
    compiler: ArtifactRef,
    source: ArtifactRef,
    path: &str,
    mut extra: Vec<RecipeWord>,
) -> Result<ArtifactRef, RunError> {
    let mut args = vec![
        literal("-c"),
        literal("-D"),
        literal("HAVE_CONFIG_H=1"),
        literal("-I"),
        input("source", "include"),
        literal("-I"),
        input("source", "include/linux/x86_64"),
    ];
    args.append(&mut extra);
    args.extend([literal("-o"), output("result", ""), input("source", path)]);
    exec(
        b,
        tool,
        id,
        compiler,
        BTreeMap::from([("source".into(), source)]),
        args,
        ArtifactKind::Object,
        BTreeMap::new(),
    )
}

fn c_archive(
    b: &mut TransitionBuilder,
    tool: &str,
    id: &str,
    compiler: ArtifactRef,
    objects: &[(&str, ArtifactRef)],
) -> Result<ArtifactRef, RunError> {
    let mut inputs = Files::new();
    let mut args = vec![literal("-ar"), literal("cr"), output("result", "")];
    for (name, object) in objects {
        inputs.insert((*name).into(), object.clone());
        args.push(input(name, ""));
    }
    exec(b, tool, id, compiler, inputs, args, ArtifactKind::Archive, BTreeMap::new())
}

fn c_tcc(
    b: &mut TransitionBuilder,
    tool: &str,
    id: &str,
    compiler: ArtifactRef,
    sources: ArtifactRef,
    version: &str,
) -> Result<ArtifactRef, RunError> {
    let mut inputs = BTreeMap::from([("source".into(), sources)]);
    let mut args = vec![
        literal("-static"),
        literal("-nostdlib"),
        literal("-I"),
        input("source", ""),
        literal("-I"),
        input("source", "include"),
        literal("-o"),
        output("result", ""),
    ];
    args.extend(tcc_defines(version, true));
    if version == "0.9.27" {
        args.extend([literal("-D"), literal("CONFIG_SYSROOT=\"/\"")]);
    }
    args.extend([literal("-B"), input("source", "runtime"), input("source", "tcc.c")]);
    if version == "0.9.27" {
        inputs.insert("va-list.o".into(), b.artifact("tinycc27-va-list")?);
        args.push(input("va-list.o", ""));
    }
    for (name, artifact) in [
        ("crt1.o", "tinycc-crt1"),
        (
            "libc.a",
            if version == "0.9.27" {
                "tinycc27-libc"
            } else {
                "tinycc-libc"
            },
        ),
        ("libtcc1.a", "tinycc-libtcc1"),
    ] {
        inputs.insert(name.into(), b.artifact(artifact)?);
        args.push(input(name, ""));
    }
    exec(b, tool, id, compiler, inputs, args, ArtifactKind::Executable, BTreeMap::new())
}

fn tinycc26(b: &mut TransitionBuilder, modules: ArtifactRef) -> Result<(), RunError> {
    let mut mes = mes_sources(b, "tinycc-mes-exec", "tinycc-mes-0.27.1-source", "tinycc26-mes")?;
    let lib = mes.get("include/mes/lib.h").cloned().ok_or_else(|| error("missing original Mes lib.h"))?;
    mes.insert(
        "include/mes/lib.h".into(),
        replace(b, "tinycc-mes-exec", "tinycc26-oputs-prototype", lib, "int oputs (char const *s);\n", "")?,
    );
    let unified = unified_libc(b, "tinycc-runtime-exec", "tinycc26-unified-libc", &mes)?;
    mes.insert("unified-libc.c".into(), unified);
    let mes_tree = tree(b, "tinycc-mes-exec", "tinycc26-mes-source-tree", &mes)?;
    let abort_flags = vec![
        literal("-D"),
        literal("HAVE_CONFIG_H=1"),
        literal("-I"),
        input("source", "include"),
        literal("-I"),
        input("source", "include/linux/x86_64"),
    ];
    let abort = mes_compile(
        b,
        "tinycc-mes-exec",
        "tinycc26-abort",
        modules.clone(),
        mes_tree.clone(),
        "lib/stdlib/abort.c",
        abort_flags,
    )?;
    let abort = mes_assemble(b, "tinycc-mes-exec", "tinycc26-abort-assemble", abort)?;
    let libc_tcc = b.artifact("mes-libc-tcc")?;
    let libc_tcc = concat(b, "tinycc-mes-exec", "tinycc26-libc-tcc-abort", &[libc_tcc, abort], ArtifactKind::Archive)?;
    let mut tcc = tcc26_sources(b)?;
    for (path, reference) in &mes {
        if path.starts_with("include/") {
            tcc.insert(path.clone(), reference.clone());
        }
    }
    let tcc_tree = tree(b, "tinycc-mes-exec", "tinycc26-source-tree", &tcc)?;
    let compiler = mes_tcc(b, "tinycc-mes-exec", "tcc-mes", modules.clone(), tcc_tree, libc_tcc.clone())?;
    b.publish("tinycc-mes-exec", "tcc-mes", compiler.clone())?;
    observe(
        b,
        "tinycc-mes-exec",
        "tcc-mes-version",
        compiler.clone(),
        Files::new(),
        vec![literal("-version")],
        0,
    )?;
    for (id, path, extra, archive) in [
        ("tinycc-crt1", "lib/linux/x86_64-mes-gcc/crt1.c", vec![], false),
        (
            "tinycc-libtcc1",
            "lib/libtcc1.c",
            vec![
                literal("-D"),
                literal("HAVE_LONG_LONG=1"),
                literal("-D"),
                literal("HAVE_FLOAT=1"),
            ],
            true,
        ),
        ("tinycc-libc", "unified-libc.c", vec![], true),
        ("tinycc-libgetopt", "lib/posix/getopt.c", vec![], true),
    ] {
        let object = c_compile(
            b,
            "tinycc-runtime-exec",
            &format!("{id}-compile"),
            compiler.clone(),
            mes_tree.clone(),
            path,
            extra,
        )?;
        if id == "tinycc-libtcc1" {
            b.artifacts.insert("tinycc-libtcc1-object".into(), object.clone());
            let members = tree(
                b,
                "tinycc-runtime-exec",
                "tinycc-libtcc1-objects",
                &BTreeMap::from([("libtcc1.o".into(), object.clone())]),
            )?;
            b.artifacts.insert("tinycc-libtcc1-objects".into(), members);
        }
        let object_name = Path::new(path).file_name().ok_or_else(|| error("missing runtime source filename"))?;
        let object_name = Path::new(object_name)
            .with_extension("o")
            .to_str()
            .ok_or_else(|| error("non-UTF8 runtime object filename"))?
            .to_string();
        let result = if archive {
            c_archive(b, "tinycc-runtime-exec", &format!("{id}-archive"), compiler.clone(), &[(&object_name, object)])?
        } else {
            object
        };
        b.publish("tinycc-runtime-exec", id, result)?;
    }
    for id in ["tinycc-crti", "tinycc-crtn"] {
        let empty = text(b, "tinycc-runtime-exec", id, "")?;
        b.artifacts.insert(id.into(), empty);
    }
    for (path, id) in [
        ("runtime/crt1.o", "tinycc-crt1"),
        ("runtime/crti.o", "tinycc-crti"),
        ("runtime/crtn.o", "tinycc-crtn"),
        ("runtime/libc.a", "tinycc-libc"),
        ("runtime/libtcc1.a", "tinycc-libtcc1"),
    ] {
        tcc.insert(path.into(), b.artifact(id)?);
    }
    let tcc_tree = tree(b, "tinycc-boot0-exec", "tinycc26-boot-source-tree", &tcc)?;
    let boot = c_tcc(b, "tinycc-boot0-exec", "tcc-boot0-build", compiler, tcc_tree, "0.9.26")?;
    observe(
        b,
        "tinycc-boot0-exec",
        "tcc-boot0-version",
        boot.clone(),
        Files::new(),
        vec![literal("-version")],
        0,
    )?;
    b.publish("tinycc-boot0-exec", "tcc-boot0", boot)?;
    let tcc_tree = tree(b, "tinycc-final-exec", "tinycc26-final-source-tree", &tcc)?;
    let final_tcc = mes_tcc(b, "tinycc-final-exec", "tinycc26-final", modules, tcc_tree, libc_tcc)?;
    observe(
        b,
        "tinycc-final-exec",
        "tinycc26-final-version",
        final_tcc.clone(),
        Files::new(),
        vec![literal("-version")],
        0,
    )?;
    b.publish("tinycc-final-exec", "tinycc-0.9.26", final_tcc)?;
    let include_files = mes
        .iter()
        .filter_map(|(path, reference)| path.strip_prefix("include/").map(|path| (path.to_string(), reference.clone())))
        .collect();
    let includes = tree(b, "tinycc-final-exec", "tinycc26-includes", &include_files)?;
    b.artifacts.insert("tinycc26-include-tree".into(), includes);
    Ok(())
}

fn patch27(b: &mut TransitionBuilder) -> Result<Files, RunError> {
    let mut files = original_files(b, "tinycc-0.9.27-source")?;
    let patch = b.read_source("tinycc-0.9.27-stagex-patch", "")?;
    let lines = patch.lines().collect::<Vec<_>>();
    let mut index = 0;
    let mut hunk = 0;
    let mut path = String::new();
    while index < lines.len() {
        let line = lines[index];
        if let Some(file) = line.strip_prefix("+++ b/patched/") {
            path = file.into();
        }
        if line.starts_with("@@ ") {
            let mut old = String::new();
            let mut new = String::new();
            index += 1;
            while index < lines.len() && !lines[index].starts_with("@@ ") && !lines[index].starts_with("diff --git ") {
                let line = lines[index];
                match line.as_bytes().first() {
                    Some(b' ') => {
                        old.push_str(&line[1..]);
                        old.push('\n');
                        new.push_str(&line[1..]);
                        new.push('\n');
                    }
                    Some(b'-') => {
                        old.push_str(&line[1..]);
                        old.push('\n');
                    }
                    Some(b'+') => {
                        new.push_str(&line[1..]);
                        new.push('\n');
                    }
                    Some(b'\\') => {
                        return Err(error("non-newline TinyCC patch hunks require explicit source handling"));
                    }
                    _ => return Err(error("malformed original TinyCC patch hunk")),
                }
                index += 1;
            }
            let source =
                files.get(&path).cloned().ok_or_else(|| error(format!("missing original patch target {path}")))?;
            let executable = b.artifact("early-executable-copy")?;
            let patch_source = b.source("tinycc-0.9.27-stagex-patch")?;
            let old = b.authored(&format!("instructions/tinycc27-hunk-{hunk}-old.txt"), old)?;
            let new = b.authored(&format!("instructions/tinycc27-hunk-{hunk}-new.txt"), new)?;
            let patched = exec(
                b,
                "tinycc27-build-exec",
                &format!("tinycc27-source-patch-{hunk}"),
                executable,
                BTreeMap::from([
                    ("source".into(), source),
                    ("patch".into(), patch_source),
                    ("match".into(), old),
                    ("replacement".into(), new),
                ]),
                vec![
                    literal("replace"),
                    input("source", ""),
                    output("result", ""),
                    input("match", ""),
                    input("replacement", ""),
                ],
                ArtifactKind::Text,
                BTreeMap::new(),
            )?;
            files.insert(path.clone(), patched);
            hunk += 1;
            continue;
        }
        index += 1;
    }
    if hunk == 0 {
        return Err(error("original TinyCC patch contains no transformations"));
    }
    files.insert("config.h".into(), text(b, "tinycc27-build-exec", "tinycc27-config", "")?);
    Ok(files)
}

fn observe(
    b: &mut TransitionBuilder,
    tool: &str,
    id: &str,
    executable: ArtifactRef,
    mut inputs: Files,
    arguments: Vec<RecipeWord>,
    expected_exit_code: i32,
) -> Result<ArtifactRef, RunError> {
    inputs.insert("exe".into(), executable);
    Ok(b.add(tool, RecipeStep {
        id: id.into(),
        inputs,
        outputs: BTreeMap::from([
            ("stdout".into(), spec(ArtifactKind::Text)),
            ("stderr".into(), spec(ArtifactKind::Text)),
        ]),
        operation: RecipeOperation::Exec {
            invocation_input: None,
            command_state: None,
            executable_input: "exe".into(),
            arguments,
            environment: BTreeMap::new(),
            runtime_mounts: BTreeMap::new(),
            output_seeds: BTreeMap::new(),
            guard: None,
            merge_stderr: false,
            cwd_input: None,
            cwd_output: None,
            stdin_input: None,
            exit_policy: RecipeExitPolicy::Exact {
                code: expected_exit_code,
            },
        },
    })?["stdout"]
        .clone())
}

fn tinycc27(b: &mut TransitionBuilder) -> Result<(), RunError> {
    let tool = "tinycc27-runtime-exec";
    let mut mes = mes_sources(b, tool, "tinycc-mes-0.27.1-source", "tinycc27-mes")?;
    let header = b.authored(
        "helpers/tinycc27-stdarg.h",
        std::str::from_utf8(crate::stagex_tinycc27::TINYCC27_STDARG_HEADER)
            .map_err(|e| error(e.to_string()))?
            .into(),
    )?;
    let va_list = b.authored(
        "helpers/tinycc27-va-list.c",
        std::str::from_utf8(crate::stagex_tinycc27::TINYCC27_VA_LIST_SOURCE)
            .map_err(|e| error(e.to_string()))?
            .into(),
    )?;
    mes.insert("include/stdarg.h".into(), header);
    mes.insert("va_list.c".into(), va_list);
    let exit = mes
        .get("lib/linux/x86_64-mes-gcc/_exit.c")
        .cloned()
        .ok_or_else(|| error("missing Mes exit source"))?;
    let exit = replace(b, tool, "tinycc27-exit-outputs", exit, ": // no outputs \"=\" (r)", ": // no outputs")?;
    let exit = replace(
        b,
        tool,
        "tinycc27-exit-clobbers",
        exit,
        ": \"rm\" (code)",
        ": \"r\" (code)\n       : \"rax\", \"rdi\"",
    )?;
    mes.insert("lib/linux/x86_64-mes-gcc/_exit.c".into(), exit);
    let unified = unified_libc(b, tool, "tinycc27-unified-libc", &mes)?;
    mes.insert("unified-libc.c".into(), unified);
    let sources = tree(b, tool, "tinycc27-mes-source-tree", &mes)?;
    let compiler = b.artifact("tinycc-0.9.26")?;
    let va_list =
        c_compile(b, tool, "tinycc27-va-list-compile", compiler.clone(), sources.clone(), "va_list.c", Vec::new())?;
    b.publish(tool, "tinycc27-va-list", va_list.clone())?;
    let libc =
        c_compile(b, tool, "tinycc27-libc-compile", compiler.clone(), sources, "unified-libc.c", vec![literal("-w")])?;
    let archive = c_archive(b, tool, "tinycc27-libc-archive", compiler.clone(), &[
        ("va_list.o", va_list),
        ("unified-libc.o", libc),
    ])?;
    b.publish(tool, "tinycc27-libc", archive)?;
    let mut tcc = patch27(b)?;
    for (path, reference) in &mes {
        if path.starts_with("include/") {
            tcc.insert(path.clone(), reference.clone());
        }
    }
    for (path, id) in [
        ("runtime/crt1.o", "tinycc-crt1"),
        ("runtime/crti.o", "tinycc-crti"),
        ("runtime/crtn.o", "tinycc-crtn"),
        ("runtime/libc.a", "tinycc27-libc"),
        ("runtime/libtcc1.a", "tinycc-libtcc1"),
    ] {
        tcc.insert(path.into(), b.artifact(id)?);
    }
    let sources = tree(b, "tinycc27-build-exec", "tinycc27-source-tree", &tcc)?;
    let tcc = c_tcc(b, "tinycc27-build-exec", "tinycc27-compiler-build", compiler, sources, "0.9.27")?;
    b.publish("tinycc27-build-exec", "tinycc-0.9.27", tcc.clone())?;
    let alias = executable_copy(b, "tinycc27-build-exec", "tinycc27-compiler-alias", tcc.clone())?;
    b.publish("tinycc27-build-exec", "tinycc27-alias", alias)?;
    let includes = mes
        .iter()
        .filter_map(|(path, reference)| path.strip_prefix("include/").map(|path| (path.to_string(), reference.clone())))
        .collect();
    let includes = tree(b, "tinycc27-build-exec", "tinycc27-includes", &includes)?;
    b.artifacts.insert("tinycc27-include-tree".into(), includes);
    observe(
        b,
        "tinycc27-smoke-exec",
        "tinycc27-version",
        tcc.clone(),
        Files::new(),
        vec![literal("-version")],
        0,
    )?;
    let positive = b.authored(
        "helpers/tinycc27-positive.c",
        std::str::from_utf8(crate::stagex_tinycc27::TINYCC27_POSITIVE_SMOKE_SOURCE)
            .map_err(|e| error(e.to_string()))?
            .into(),
    )?;
    let negative = b.authored(
        "helpers/tinycc27-malformed.c",
        std::str::from_utf8(crate::stagex_tinycc27::TINYCC27_MALFORMED_SMOKE_SOURCE)
            .map_err(|e| error(e.to_string()))?
            .into(),
    )?;
    let object = exec(
        b,
        "tinycc27-smoke-exec",
        "tinycc27-positive-compile",
        tcc.clone(),
        BTreeMap::from([("source.c".into(), positive)]),
        vec![
            literal("-c"),
            literal("-o"),
            output("result", ""),
            input("source.c", ""),
        ],
        ArtifactKind::Object,
        BTreeMap::new(),
    )?;
    b.publish("tinycc27-smoke-exec", "tinycc27-positive-object", object)?;
    observe(
        b,
        "tinycc27-smoke-exec",
        "tinycc27-negative-compile",
        tcc,
        BTreeMap::from([("source.c".into(), negative)]),
        vec![
            literal("-c"),
            input("source.c", ""),
            literal("-o"),
            literal("malformed.o"),
        ],
        1,
    )?;
    Ok(())
}

pub(super) fn extend(builder: &mut TransitionBuilder, _request: &AuthoringRequest<'_>) -> Result<(), RunError> {
    stage0(builder)?;
    let mes = mes_bootstrap(builder)?;
    let modules = mes_runtime(builder, mes)?;
    tinycc26(builder, modules)?;
    tinycc27(builder)
}

const EARLY_COPY_SOURCE: &str = r###"/* M2-compatible copying and source transformations; never launches a child. */
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>
#include <sys/stat.h>
#include "M2libc/bootstrappable.h"
char* read_text(char* path)
{
    FILE* file = fopen(path, "r");
    require(file != NULL, "cannot open text operand\n");
    require(fseek(file, 0, SEEK_END) == 0, "cannot seek text operand\n");
    int length = ftell(file);
    require(length >= 0, "invalid text operand size\n");
    char* value = calloc(length + 1, 1);
    require(value != NULL, "cannot allocate text operand\n");
    rewind(file);
    require(fread(value, 1, length, file) == length, "cannot read text operand\n");
    fclose(file);
    return value;
}
int main(int argc, char** argv)
{
    if(argc == 6)
    {
        require(match(argv[1], "replace"), "unknown source operation\n");
        char* source_text = read_text(argv[2]);
        char* pattern = read_text(argv[4]);
        char* replacement = read_text(argv[5]);
        int size = strlen(source_text);
        int pattern_size = strlen(pattern);
        require(pattern_size > 0, "empty source pattern\n");
        FILE* result = fopen(argv[3], "w");
        require(result != NULL, "cannot open transformed source\n");
        int at = 0;
        int found = 0;
        while(at < size)
        {
            if((at + pattern_size <= size) && (strncmp(source_text + at, pattern, pattern_size) == 0))
            {
                fputs(replacement, result);
                at = at + pattern_size;
                found = 1;
            }
            else
            {
                fputc(source_text[at], result);
                at = at + 1;
            }
        }
        require(found, "original source pattern missing\n");
        require(fclose(result) == 0, "cannot close transformed source\n");
        return 0;
    }
    if(argc == 4)
    {
        require(match(argv[1], "defines"), "unknown source operation\n");
        char* definitions = read_text(argv[2]);
        FILE* result = fopen(argv[3], "w");
        require(result != NULL, "cannot open definitions header\n");
        int at = 0;
        while(definitions[at] != 0)
        {
            fputs("#define ", result);
            while((definitions[at] != 0) && (definitions[at] != '='))
            {
                require(definitions[at] != '\n', "definition lacks value\n");
                fputc(definitions[at], result);
                at = at + 1;
            }
            require(definitions[at] == '=', "definition lacks value\n");
            fputc(' ', result);
            at = at + 1;
            while((definitions[at] != 0) && (definitions[at] != '\n'))
            {
                fputc(definitions[at], result);
                at = at + 1;
            }
            fputc('\n', result);
            if(definitions[at] == '\n') at = at + 1;
        }
        require(fclose(result) == 0, "cannot close definitions header\n");
        return 0;
    }
    require(argc == 3, "copy-executable INPUT OUTPUT\n");
    FILE* source = fopen(argv[1], "r");
    require(source != NULL, "cannot open copy source\n");
    FILE* destination = fopen(argv[2], "w");
    require(destination != NULL, "cannot open copy destination\n");
    int c = fgetc(source);
    while(c != EOF)
    {
        fputc(c, destination);
        c = fgetc(source);
    }
    fclose(source);
    fclose(destination);
    require(chmod(argv[2], 0700) == 0, "cannot set executable mode\n");
    return 0;
}
"###;

fn executable_copy(
    b: &mut TransitionBuilder,
    tool: &str,
    id: &str,
    source: ArtifactRef,
) -> Result<ArtifactRef, RunError> {
    let executable = b.artifact("early-executable-copy")?;
    exec(
        b,
        tool,
        id,
        executable,
        BTreeMap::from([("source".into(), source)]),
        vec![input("source", ""), output("result", "")],
        ArtifactKind::Executable,
        BTreeMap::new(),
    )
}
