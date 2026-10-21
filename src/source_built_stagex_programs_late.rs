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
use crate::source_built_stagex_programs::TransitionBuilder;
use crate::source_built_stagex_programs::input;
use crate::source_built_stagex_programs::literal;
use crate::source_built_stagex_programs::output;

const FILE_BYTES: u64 = 64 * 1024 * 1024;
const TREE_BYTES: u64 = 512 * 1024 * 1024;

pub(super) fn extend(builder: &mut TransitionBuilder, _request: &AuthoringRequest<'_>) -> Result<(), RunError> {
    m4(builder)?;
    diffutils(builder)?;
    grep(builder)?;
    gawk(builder)?;
    bash_full(builder)?;
    Ok(())
}

fn bounded(kind: ArtifactKind) -> RecipeOutput {
    RecipeOutput {
        kind,
        bytes_max: if kind == ArtifactKind::Tree {
            TREE_BYTES
        } else {
            FILE_BYTES
        },
    }
}

pub(super) fn command(
    builder: &mut TransitionBuilder,
    tool: &str,
    id: &str,
    executable: ArtifactRef,
    mut inputs: BTreeMap<String, ArtifactRef>,
    arguments: Vec<RecipeWord>,
    outputs: &[(&str, ArtifactKind)],
    stdin: Option<&str>,
    exit_code: i32,
) -> Result<BTreeMap<String, ArtifactRef>, RunError> {
    inputs.insert("executable".into(), executable);
    let mut outputs: BTreeMap<String, RecipeOutput> =
        outputs.iter().map(|(id, kind)| ((*id).into(), bounded(*kind))).collect();
    outputs.entry("stdout".into()).or_insert_with(|| bounded(ArtifactKind::Text));
    outputs.entry("stderr".into()).or_insert_with(|| bounded(ArtifactKind::Text));
    outputs.entry("exit-status".into()).or_insert_with(|| RecipeOutput {
        kind: ArtifactKind::Text,
        bytes_max: 4,
    });
    let result = builder.add(tool, RecipeStep {
        id: id.into(),
        inputs,
        outputs,
        operation: RecipeOperation::Exec {
            command_state: None,
            executable_input: "executable".into(),
            invocation_input: None,
            arguments,
            environment: BTreeMap::from([("LC_ALL".into(), literal("C")), ("LANG".into(), literal("C"))]),
            runtime_mounts: BTreeMap::new(),
            output_seeds: BTreeMap::new(),
            guard: None,
            cwd_input: None,
            cwd_output: None,
            stdin_input: stdin.map(str::to_owned),
            merge_stderr: false,
            exit_policy: RecipeExitPolicy::Exact { code: exit_code },
        },
    })?;
    for (name, reference) in &result {
        builder.artifacts.insert(format!("late-leaf:{id}:{name}"), reference.clone());
    }
    Ok(result)
}

pub(super) fn prefixed(prefix: &str, mut word: RecipeWord) -> RecipeWord {
    word.parts.insert(0, WordPart::Literal { text: prefix.into() });
    word
}

pub(super) fn compile(
    builder: &mut TransitionBuilder,
    tool: &str,
    id: &str,
    source: ArtifactRef,
    path: &str,
    includes: &[&str],
    flags: &[&str],
) -> Result<ArtifactRef, RunError> {
    let compiler = builder.artifact("tcc-musl-v2")?;
    let headers = builder.artifact("musl-native-headers")?;
    let mut args = vec![literal("-c")];
    if path.is_empty() {
        args.extend([literal("-x"), literal("c")]);
    }
    args.extend(includes.iter().map(|include| {
        if path.is_empty() {
            prefixed("-I", input("source", include))
        } else {
            literal(&format!("-I{}", if include.is_empty() { "." } else { include }))
        }
    }));
    args.push(prefixed("-I", input("headers", "")));
    args.extend(flags.iter().map(|flag| literal(flag)));
    args.extend([
        if path.is_empty() {
            input("source", "")
        } else {
            literal(path)
        },
        literal("-o"),
        output("object", ""),
    ]);
    let outputs = command(
        builder,
        tool,
        id,
        compiler,
        BTreeMap::from([("source".into(), source), ("headers".into(), headers)]),
        args,
        &[("object", ArtifactKind::Object)],
        None,
        0,
    )?;
    if !path.is_empty()
        && let Some(RecipeStep {
            operation: RecipeOperation::Exec { cwd_input, .. },
            ..
        }) = builder.recipe.steps.last_mut()
    {
        *cwd_input = Some("source".into());
    }
    Ok(outputs["object"].clone())
}

pub(super) fn link(
    builder: &mut TransitionBuilder,
    tool: &str,
    id: &str,
    objects: Vec<ArtifactRef>,
) -> Result<ArtifactRef, RunError> {
    let compiler = builder.artifact("tcc-musl-v2")?;
    let mut inputs = BTreeMap::from([
        ("crt1".into(), builder.artifact("musl-native-crt1")?),
        ("libc".into(), builder.artifact("musl-native-libc")?),
    ]);
    let mut args = vec![
        literal("-nostdlib"),
        literal("-static"),
        literal("-o"),
        output("program", ""),
        input("crt1", ""),
    ];
    for (index, object) in objects.into_iter().enumerate() {
        let name = format!("object-{index:03}");
        args.push(input(&name, ""));
        inputs.insert(name, object);
    }
    args.push(input("libc", ""));
    let outputs =
        command(builder, tool, id, compiler, inputs, args, &[("program", ArtifactKind::Executable)], None, 0)?;
    Ok(outputs["program"].clone())
}

pub(super) fn helper(builder: &mut TransitionBuilder, tool: &str) -> Result<ArtifactRef, RunError> {
    if let Some(reference) = builder.artifacts.get("late-source-transform") {
        return Ok(reference.clone());
    }
    let source = builder.authored("helpers/stagex-late-source-transform.c", TRANSFORM_C.to_string())?;
    let object = compile(builder, tool, "late-transform-compile", source, "", &[], &[])?;
    let program = link(builder, tool, "late-transform-link", vec![object])?;
    builder.artifacts.insert("late-source-transform".into(), program.clone());
    Ok(program)
}

pub(super) fn transform(
    builder: &mut TransitionBuilder,
    tool: &str,
    id: &str,
    source: ArtifactRef,
    path: &str,
    operation: &str,
    parameters: &[&str],
) -> Result<ArtifactRef, RunError> {
    let executable = helper(builder, tool)?;
    let mut arguments = vec![literal(operation), input("source", path), output("text", "")];
    arguments.extend(parameters.iter().map(|value| literal(value)));
    let outputs = command(
        builder,
        tool,
        id,
        executable,
        BTreeMap::from([("source".into(), source)]),
        arguments,
        &[("text", ArtifactKind::Text)],
        None,
        0,
    )?;
    Ok(outputs["text"].clone())
}

pub(super) fn emit(builder: &mut TransitionBuilder, tool: &str, id: &str, text: &str) -> Result<ArtifactRef, RunError> {
    let executable = helper(builder, tool)?;
    let result = command(
        builder,
        tool,
        id,
        executable,
        BTreeMap::new(),
        vec![literal("emit"), output("text", ""), literal(text)],
        &[("text", ArtifactKind::Text)],
        None,
        0,
    )?;
    Ok(result["text"].clone())
}

pub(super) fn overlay(
    builder: &mut TransitionBuilder,
    tool: &str,
    id: &str,
    source: ArtifactRef,
    entries: BTreeMap<String, ArtifactRef>,
) -> Result<ArtifactRef, RunError> {
    let executable = helper(builder, tool)?;
    let mut inputs = BTreeMap::from([("source".into(), source)]);
    let mut arguments = vec![literal("overlay"), input("source", ""), output("tree", "")];
    for (index, (path, reference)) in entries.into_iter().enumerate() {
        let name = format!("entry-{index:03}");
        arguments.extend([literal(&path), input(&name, "")]);
        inputs.insert(name, reference);
    }
    let result = command(builder, tool, id, executable, inputs, arguments, &[("tree", ArtifactKind::Tree)], None, 0)?;
    Ok(result["tree"].clone())
}

pub(super) fn observation(
    builder: &mut TransitionBuilder,
    tool: &str,
    id: &str,
    executable: ArtifactRef,
    args: &[&str],
    stdin_text: &str,
    code: i32,
    published: Option<&str>,
) -> Result<ArtifactRef, RunError> {
    let stdin = emit(builder, tool, &format!("{id}-input"), stdin_text)?;
    let result = command(
        builder,
        tool,
        id,
        executable,
        BTreeMap::from([("stdin".into(), stdin)]),
        args.iter().map(|arg| literal(arg)).collect(),
        &[("stdout", ArtifactKind::Text), ("stderr", ArtifactKind::Text)],
        Some("stdin"),
        code,
    )?;
    if let Some(name) = published {
        let observed = verify_observation(builder, tool, name, result["stdout"].clone())?;
        builder.publish(tool, name, observed)?;
    }
    if id == "m4-prefix-negative" {
        transform(builder, tool, "m4-prefix-negative-assert", result["stdout"].clone(), "", "not-equal", &[
            "must-not-expand\n",
        ])?;
    }
    if id == "diffutils-malformed" {
        transform(builder, tool, "diffutils-malformed-stdout-assert", result["stdout"].clone(), "", "equal", &[""])?;
    }
    if code == 0 {
        Ok(result["stderr"].clone())
    } else {
        transform(builder, tool, &format!("{id}-diagnostic-assert"), result["stderr"].clone(), "", "nonempty", &[])
    }
}

pub(super) fn observation_checks(name: &str) -> &'static [(&'static str, &'static str)] {
    match name {
        "musl-native-validation-receipt" => &[("equal", "native-musl-validation-ok\n")],
        "m4-malformed-rejection" => {
            &[("equal", "{\"format\":\"mantle-stagex-m4-rejection-v1\",\"malformed_input_rejected\":true}\n")]
        }
        "tcc-musl-v2-positive-execution" => &[("equal", "")],
        "m4-version-observation" => &[("contains", "GNU M4 1.4.7")],
        "bison-version-observation" => &[("contains", "bison")],
        "flex-version-observation" => &[("contains", "flex 2.6.4")],
        "m4-macro-observation" => &[("equal", "42\nambient-disabled\n")],
        "m4-prefix-observation" => &[("equal", "prefixed-ok\n")],
        "diffutils-version-observation" => &[("contains", "GNU diffutils"), ("contains", "2.7")],
        "grep-literal-observation" => &[("equal", "hello\n")],
        "grep-quiet-observation" => &[("equal", "")],
        "grep-anchored-observation" => &[("equal", "abc\n")],
        "gawk-behavior-observation" => &[("equal", "7:2:AWK\n")],
        "gawk-numeric-observation" => &[("equal", "1:x\n")],
        "gawk-decimal-observation" => &[("equal", "3\n")],
        "gawk-length-observation" => &[("equal", "2\n")],
        "gawk-hex-observation" => &[("equal", "00000044\n")],
        "gawk-installed-observation" => &[("equal", "awk-ok\n")],
        "bash-full-version-observation" => &[("contains", "GNU bash")],
        "bash-full-functional-observation" => &[("equal", "first\nsecond\nmantle\neval-ok\n")],
        "bash-full-external-child-observation" => &[("contains", "GNU Make 3.82")],
        "bash-full-make-shell-observation" => &[("equal", "make-shell-ok\n")],
        _ => &[],
    }
}

pub(super) fn verify_observation(
    builder: &mut TransitionBuilder,
    tool: &str,
    name: &str,
    mut reference: ArtifactRef,
) -> Result<ArtifactRef, RunError> {
    for (index, (operation, expected)) in observation_checks(name).iter().enumerate() {
        reference =
            transform(builder, tool, &format!("{name}-assert-{index}"), reference, "", operation, &[*expected])?;
    }
    Ok(reference)
}

fn input_tree(
    builder: &mut TransitionBuilder,
    tool: &str,
    id: &str,
    files: BTreeMap<String, ArtifactRef>,
) -> Result<ArtifactRef, RunError> {
    let inputs = files
        .values()
        .enumerate()
        .map(|(index, reference)| (format!("file-{index}"), reference.clone()))
        .collect();
    let entries = files.keys().enumerate().map(|(index, name)| (name.clone(), format!("file-{index}"))).collect();
    let result = builder.add(tool, RecipeStep {
        id: id.into(),
        inputs,
        outputs: BTreeMap::from([("tree".into(), bounded(ArtifactKind::Tree))]),
        operation: RecipeOperation::AssembleTree {
            output: "tree".into(),
            entries,
        },
    })?;
    Ok(result["tree"].clone())
}

fn rejection_receipt(
    builder: &mut TransitionBuilder,
    tool: &str,
    id: &str,
    diagnostic: ArtifactRef,
    text: &str,
) -> Result<(), RunError> {
    let executable = helper(builder, tool)?;
    let result = command(
        builder,
        tool,
        &format!("{id}-receipt"),
        executable,
        BTreeMap::from([("diagnostic".into(), diagnostic)]),
        vec![
            literal("receipt"),
            input("diagnostic", ""),
            output("text", ""),
            literal(text),
        ],
        &[("text", ArtifactKind::Text)],
        None,
        0,
    )?;
    builder.publish(tool, id, result["text"].clone())
}

fn m4(builder: &mut TransitionBuilder) -> Result<(), RunError> {
    let tool = "m4-build-exec";
    let source = builder.fetcher(tool, "m4-1.4.7-source")?;
    let recipe = builder.source("m4-1.4.7-recipe-source")?;
    let config = transform(builder, tool, "m4-extract-config", recipe, "", "extract", &[
        "      $BB cat > lib/config.h <<'EOF'\n",
        "\nEOF",
    ])?;
    let config = transform(builder, tool, "m4-disable-shell-path", config, "", "replace", &[
        "#define SYSCMD_SHELL \"/bin/sh\"",
        "#define SYSCMD_SHELL \"/__mantle_stagex_ambient_shell_disabled__\"",
        "1",
    ])?;
    let builtin =
        transform(builder, tool, "m4-disable-ambient-builtins", source.clone(), "src/builtin.c", "m4-disable", &[])?;
    let tree = overlay(
        builder,
        tool,
        "m4-configure",
        source,
        BTreeMap::from([("lib/config.h".into(), config), ("src/builtin.c".into(), builtin)]),
    )?;
    builder.publish(tool, "m4-configured-source", tree.clone())?;
    let mut objects = Vec::new();
    for (directory, names) in [
        (
            "lib",
            "cloexec close-stream dup-safer error exitfail fd-safer fopen-safer getopt getopt1 mkstemp-safer regex obstack tmpfile-safer verror xalloc-die xasprintf xmalloc xvasprintf",
        ),
        ("src", "m4 builtin debug eval format freeze input macro output path symtab"),
    ] {
        for name in names.split_whitespace() {
            let object = compile(
                builder,
                tool,
                &format!("m4-{directory}-{name}-compile"),
                tree.clone(),
                &format!("{directory}/{name}.c"),
                &["lib", "src"],
                &["-DHAVE_CONFIG_H=1", "-DSIZE_MAX=~(size_t)0"],
            )?;
            objects.push(object);
        }
    }
    let program = link(builder, tool, "m4-link", objects)?;
    builder.publish(tool, "m4-1.4.7", program.clone())?;
    builder
        .artifacts
        .insert("member:m4-stage/runtime/output/bin/m4".into(), builder.artifact("m4-1.4.7")?);
    builder.artifacts.insert("phase-tool:m4-1.4.7".into(), program.clone());
    let smoke = "m4-smoke-exec";
    observation(builder, smoke, "m4-version", program.clone(), &["--version"], "", 0, Some("m4-version-observation"))?;
    observation(
        builder,
        smoke,
        "m4-macro",
        program.clone(),
        &[],
        "define(`mantle_value', `42')mantle_value\nifdef(`syscmd',`ambient-enabled',`ambient-disabled')\n",
        0,
        Some("m4-macro-observation"),
    )?;
    observation(
        builder,
        smoke,
        "m4-prefix",
        program.clone(),
        &["-P"],
        "m4_define(`mantle_prefixed_value', `prefixed-ok')mantle_prefixed_value\n",
        0,
        Some("m4-prefix-observation"),
    )?;
    observation(
        builder,
        smoke,
        "m4-prefix-negative",
        program.clone(),
        &["-P"],
        "define(`mantle_unprefixed_value', `must-not-expand')mantle_unprefixed_value\n",
        0,
        None,
    )?;
    let diagnostic =
        observation(builder, smoke, "m4-malformed", program, &[], "define(`unterminated, `value')\n", 1, None)?;
    rejection_receipt(
        builder,
        smoke,
        "m4-malformed-rejection",
        diagnostic,
        "{\"format\":\"mantle-stagex-m4-rejection-v1\",\"malformed_input_rejected\":true}\n",
    )
}

fn diffutils(builder: &mut TransitionBuilder) -> Result<(), RunError> {
    let tool = "diffutils-build-exec";
    let source = builder.fetcher(tool, "diffutils-2.7-source")?;
    let recipe = builder.source("diffutils-2.7-recipe-source")?;
    let config =
        transform(builder, tool, "diffutils-extract-config", recipe, "", "extract", &["      : > config.h\n", ""])?;
    let tree = overlay(builder, tool, "diffutils-configure", source, BTreeMap::from([("config.h".into(), config)]))?;
    builder.publish(tool, "diffutils-configured-source", tree.clone())?;
    let flags = [
        "-DNULL_DEVICE=\"/dev/null\"",
        "-DHAVE_STRERROR=1",
        "-DREGEX_MALLOC=1",
        "-DHAVE_DIRENT_H=1",
        "-DHAVE_DUP2=1",
        "-DHAVE_FORK=1",
        "-DHAVE_UNISTD_H=1",
        "-DHAVE_STRING_H=1",
        "-DHAVE_STDLIB_H=1",
    ];
    for (target, names) in [
        ("cmp", "cmp cmpbuf error getopt getopt1 xmalloc version"),
        (
            "diff",
            "diff alloca analyze cmpbuf dir io util context ed ifdef normal side fnmatch getopt getopt1 regex version",
        ),
    ] {
        let mut objects = Vec::new();
        for name in names.split_whitespace() {
            let object = compile(
                builder,
                tool,
                &format!("diffutils-{target}-{name}-compile"),
                tree.clone(),
                &format!("{name}.c"),
                &[""],
                &flags,
            )?;
            objects.push(object);
        }
        let program = link(builder, tool, &format!("diffutils-{target}-link"), objects)?;
        builder.publish(tool, &format!("diffutils-{target}-2.7"), program.clone())?;
        let installed = builder.artifact(&format!("diffutils-{target}-2.7"))?;
        builder.artifacts.insert(format!("member:diffutils-stage/runtime/output/bin/{target}"), installed);
        builder.artifacts.insert(format!("phase-tool:diffutils-{target}-2.7"), program);
    }
    let smoke = "diffutils-smoke-exec";
    let first = emit(builder, smoke, "diffutils-first-input", "same\n")?;
    let second = emit(builder, smoke, "diffutils-changed-input", "changed\n")?;
    let equal = input_tree(
        builder,
        smoke,
        "diffutils-equal-files",
        BTreeMap::from([
            ("first.txt".into(), first.clone()),
            ("second.txt".into(), first.clone()),
        ]),
    )?;
    let changed = input_tree(
        builder,
        smoke,
        "diffutils-changed-files",
        BTreeMap::from([("first.txt".into(), first), ("second.txt".into(), second)]),
    )?;
    for target in ["diff", "cmp"] {
        let program = builder.artifact(&format!("diffutils-{target}-2.7"))?;
        command(
            builder,
            smoke,
            &format!("diffutils-{target}-equal"),
            program.clone(),
            BTreeMap::from([("files".into(), equal.clone())]),
            vec![literal("first.txt"), literal("second.txt")],
            &[("stdout", ArtifactKind::Text)],
            None,
            0,
        )?;
        if let Some(RecipeStep {
            operation: RecipeOperation::Exec { cwd_input, .. },
            ..
        }) = builder.recipe.steps.last_mut()
        {
            *cwd_input = Some("files".into());
        }
        let result = command(
            builder,
            smoke,
            &format!("diffutils-{target}-changed"),
            program,
            BTreeMap::from([("files".into(), changed.clone())]),
            vec![literal("first.txt"), literal("second.txt")],
            &[("stdout", ArtifactKind::Text)],
            None,
            1,
        )?;
        if let Some(RecipeStep {
            operation: RecipeOperation::Exec { cwd_input, .. },
            ..
        }) = builder.recipe.steps.last_mut()
        {
            *cwd_input = Some("files".into());
        }
        builder.publish(smoke, &format!("diffutils-{target}-observation"), result["stdout"].clone())?;
    }
    let diff = builder.artifact("diffutils-diff-2.7")?;
    observation(
        builder,
        smoke,
        "diffutils-version",
        diff.clone(),
        &["--version"],
        "",
        0,
        Some("diffutils-version-observation"),
    )?;
    let diagnostic =
        observation(builder, smoke, "diffutils-malformed", diff, &["--mantle-invalid-option"], "", 2, None)?;
    rejection_receipt(
        builder,
        smoke,
        "diffutils-malformed-rejection",
        diagnostic,
        "{\"format\":\"mantle-stagex-diffutils-rejection-v1\",\"malformed_input_rejected\":true}\n",
    )
}

fn grep(builder: &mut TransitionBuilder) -> Result<(), RunError> {
    let tool = "grep-build-exec";
    let source = builder.fetcher(tool, "grep-2.4-source")?;
    let runner_source = builder.source("grep-2.4-bridge-runner-source")?;
    let recipe = builder.source("grep-2.4-recipe-source")?;
    let tree = overlay(
        builder,
        tool,
        "grep-configure",
        source,
        BTreeMap::from([("stagex-grep-bridge-runner.c".into(), runner_source)]),
    )?;
    builder.publish(tool, "grep-configured-source", tree.clone())?;
    let object = compile(builder, tool, "grep-runner-compile", tree.clone(), "stagex-grep-bridge-runner.c", &[], &[])?;
    let program = link(builder, tool, "grep-runner-link", vec![object])?;
    for id in ["grep-2.4-runner", "egrep-2.4-runner", "fgrep-2.4-runner"] {
        builder.publish(tool, id, program.clone())?;
    }
    builder
        .artifacts
        .insert("member:grep-stage/runtime/output/bin/grep".into(), builder.artifact("grep-2.4-runner")?);
    for id in ["grep-2.4-runner", "egrep-2.4-runner", "fgrep-2.4-runner"] {
        builder.artifacts.insert(format!("phase-tool:{id}"), program.clone());
    }
    let bridge = transform(builder, tool, "grep-bridge-extract", recipe, "", "extract", &[
        "      $BB cat > grep <<'EOF'\n",
        "\nEOF\n",
    ])?;
    builder.publish(tool, "grep-2.4-bridge-script", bridge)?;
    let smoke = "grep-smoke-exec";
    for (name, args, text) in [
        ("literal", vec!["hello"], "hello\n"),
        ("quiet", vec!["-q", "HELLO"], "HELLO\n"),
        ("anchored", vec!["-E", "^ab"], "abc\n"),
    ] {
        observation(
            builder,
            smoke,
            &format!("grep-{name}"),
            program.clone(),
            &args,
            text,
            0,
            Some(&format!("grep-{name}-observation")),
        )?;
    }
    Ok(())
}

fn gawk(builder: &mut TransitionBuilder) -> Result<(), RunError> {
    let tool = "gawk-build-exec";
    let source = builder.fetcher(tool, "gawk-3.0.4-source")?;
    let recipe = builder.source("gawk-3.0.4-recipe-source")?;
    let mut entries = BTreeMap::new();
    for (name, start, end) in [
        ("config.h", "      $BB cat > config.h << 'CONFIG'\n", "\nCONFIG\n"),
        ("mantle_decimal.c", "      $BB cat > mantle_decimal.c <<'DECIMAL_EOF'\n", "\nDECIMAL_EOF\n"),
    ] {
        let reference = transform(
            builder,
            tool,
            &format!("gawk-extract-{}", name.replace('.', "-")),
            recipe.clone(),
            "",
            "extract",
            &[start, end],
        )?;
        entries.insert(name.into(), reference);
    }
    for (path, rules) in gawk_rules() {
        let mut reference = source.clone();
        let mut relative = path;
        for (index, (old, new, count)) in rules.into_iter().enumerate() {
            reference = transform(
                builder,
                tool,
                &format!("gawk-{}-rewrite-{index:02}", path.replace('.', "-")),
                reference,
                relative,
                "replace",
                &[old, new, &count.to_string()],
            )?;
            relative = "";
        }
        entries.insert(path.into(), reference);
    }
    let tree = overlay(builder, tool, "gawk-configure", source, entries)?;
    builder.publish(tool, "gawk-configured-source", tree.clone())?;
    let mut objects = Vec::new();
    let members = builder.members("gawk-3.0.4-source")?;
    let parser = if members.iter().any(|path| path == "awkgram.c") {
        "awkgram"
    } else {
        "awktab"
    };
    for name in [
        "mantle_decimal",
        "array",
        "builtin",
        "dfa",
        "eval",
        "field",
        "io",
        "main",
        "msg",
        "node",
        "random",
        "re",
        "regex",
        "version",
        "gawkmisc",
    ]
    .into_iter()
    .chain(std::iter::once(parser))
    {
        let mut flags = vec!["-DHAVE_CONFIG_H", "-Dstrtod=mantle_strtod", "-Datof=mantle_atof"];
        if name == "awktab" {
            flags.push("-Dalloca=malloc");
        }
        let object =
            compile(builder, tool, &format!("gawk-{name}-compile"), tree.clone(), &format!("{name}.c"), &[""], &flags)?;
        objects.push(object);
    }
    let program = link(builder, tool, "gawk-link", objects)?;
    builder.publish(tool, "gawk-3.0.4", program.clone())?;
    builder.publish(tool, "awk-3.0.4-alias", program.clone())?;
    builder.artifacts.insert("phase-tool:gawk-3.0.4".into(), program.clone());
    builder.artifacts.insert("phase-tool:awk-3.0.4-alias".into(), program.clone());
    let smoke = "gawk-smoke-exec";
    for (name, args, text, status) in [
        (
            "behavior",
            vec![
                "BEGIN { label = toupper(\"awk\"); total = 2 + 5 } /a/ { matches++ } END { print total \":\" matches \":\" label }",
            ],
            "alpha\nbeta\n",
            0,
        ),
        ("numeric", vec!["BEGIN { n = 0; values[++n] = \"x\"; print n \":\" values[1] }"], "", 0),
        ("decimal", vec!["BEGIN { product = 1.5 * 2.0; print product }"], "", 0),
        ("length", vec!["BEGIN { print length(\"S?\") }"], "", 0),
        (
            "hex",
            vec![
                "-v",
                "value=68",
                "-v",
                "format=%08x",
                "BEGIN { printf format \"\\n\", value }",
            ],
            "",
            0,
        ),
        ("negative", vec!["BEGIN {"], "", 2),
        ("installed", vec!["{print \"awk-ok\"}"], "test\n", 0),
    ] {
        let published = format!("gawk-{name}-observation");
        let diagnostic = observation(
            builder,
            smoke,
            &format!("gawk-{name}"),
            program.clone(),
            &args,
            text,
            status,
            if name == "negative" {
                None
            } else {
                Some(published.as_str())
            },
        )?;
        if name == "negative" {
            builder.publish(smoke, "gawk-negative-observation", diagnostic)?;
        }
    }
    Ok(())
}

// psize.sh's sleeping reader only supplies backpressure: every 128-byte write
// is atomic (PIPE_BUF >= 128), and the next write after a full pipe encounters
// SIGPIPE when that reader closes. A nonblocking pipe observes the same full
// boundary without a child or a timer. Do not add the EAGAIN attempt to nw.
fn bash_pipe_probe(builder: &mut TransitionBuilder, tool: &str, source: ArtifactRef) -> Result<ArtifactRef, RunError> {
    let probe =
        transform(builder, tool, "bash-full-psize-reader-state", source.clone(), "builtins/psize.c", "replace", &[
            "int nw;",
            "int nw;\nstatic int mantle_pipe_reader;",
            "1",
        ])?;
    let probe = transform(builder, tool, "bash-full-psize-backpressure", probe, "", "replace", &[
        "n = write (1, buf, 128);",
        "n = write (1, buf, 128);\n      if (n < 0 && errno == EAGAIN)\n        { close (mantle_pipe_reader); continue; }\n      if (n != 128) exit (1);",
        "1",
    ])?;
    let probe = transform(builder, tool, "bash-full-psize-transport", probe, "", "append", &[PIPE_SIZE_TRANSPORT_C])?;
    let configured = overlay(
        builder,
        tool,
        "bash-full-psize-source",
        source.clone(),
        BTreeMap::from([("builtins/psize.c".into(), probe)]),
    )?;
    let object = compile(
        builder,
        tool,
        "bash-full-psize-compile",
        configured,
        "builtins/psize.c",
        &["", "include", "lib", "builtins"],
        &["-DHAVE_CONFIG_H", "-Dmain=mantle_pipe_original_main"],
    )?;
    link(builder, tool, "bash-full-psize-link", vec![object])
}

fn bash_pipe_size(
    builder: &mut TransitionBuilder,
    tool: &str,
    source: ArtifactRef,
    probe: ArtifactRef,
) -> Result<ArtifactRef, RunError> {
    let measured = command(builder, tool, "bash-full-psize-measure", probe, BTreeMap::new(), vec![], &[], None, 0)?;
    transform(builder, tool, "bash-full-psize-stdout-assert", measured["stdout"].clone(), "", "equal", &[""])?;
    let executable = helper(builder, tool)?;
    let script = command(
        builder,
        tool,
        "bash-full-psize-script",
        executable,
        BTreeMap::from([
            ("source".into(), source),
            ("measurement".into(), measured["stderr"].clone()),
        ]),
        vec![
            literal("pipe-header-script"),
            input("source", "builtins/psize.sh"),
            output("text", ""),
            input("measurement", ""),
        ],
        &[("text", ArtifactKind::Text)],
        None,
        0,
    )?;
    let shell = builder.artifact("bash-2.05b")?;
    let generated = command(
        builder,
        tool,
        "bash-full-psize-generate-header",
        shell,
        BTreeMap::from([("script".into(), script["text"].clone())]),
        vec![],
        &[],
        Some("script"),
        0,
    )?;
    Ok(generated["stdout"].clone())
}

const PIPE_SIZE_TRANSPORT_C: &str = r#"
#undef main
#include <fcntl.h>
int main(int argc, char **argv)
{
  int descriptors[2], flags;
  if (pipe(descriptors) || fpathconf(descriptors[1], _PC_PIPE_BUF) < 128) return 1;
  flags = fcntl(descriptors[1], F_GETFL);
  if (flags < 0 || fcntl(descriptors[1], F_SETFL, flags | O_NONBLOCK) < 0 || dup2(descriptors[1], 1) != 1) return 1;
  if (close(descriptors[1])) return 1;
  mantle_pipe_reader = descriptors[0];
  return mantle_pipe_original_main(argc, argv);
}
"#;

fn bash_full(builder: &mut TransitionBuilder) -> Result<(), RunError> {
    let build_generator = "bash-full-generator-build-exec";
    let source = builder.artifact("bash-configured-source")?;
    let config =
        transform(builder, build_generator, "bash-full-config-normalize", source.clone(), "config.h", "append", &[
            "#define HAVE_SYS_TIMES_H 1\n#undef HAVE_SYS_SOCKET_H\n",
        ])?;
    let generator_source = overlay(
        builder,
        build_generator,
        "bash-full-generator-configure",
        source.clone(),
        BTreeMap::from([("config.h".into(), config.clone())]),
    )?;
    let object = compile(
        builder,
        build_generator,
        "bash-full-mkbuiltins-compile",
        generator_source.clone(),
        "builtins/mkbuiltins.c",
        &["", "include", "lib", "builtins"],
        &["-DHAVE_CONFIG_H"],
    )?;
    let generator = link(builder, build_generator, "bash-full-mkbuiltins-link", vec![object])?;
    builder.publish(build_generator, "bash-full-mkbuiltins", generator.clone())?;
    builder.artifacts.insert("phase-tool:bash-full-mkbuiltins".into(), generator.clone());
    let pipe_probe = bash_pipe_probe(builder, build_generator, source.clone())?;

    let run_generator = "bash-full-generator-run-exec";
    let mut generated = BTreeMap::new();
    let pipesize = bash_pipe_size(builder, run_generator, source.clone(), pipe_probe)?;
    generated.insert("builtins/pipesize.h".into(), pipesize);
    let executable = helper(builder, run_generator)?;
    let builtins = command(
        builder,
        run_generator,
        "bash-full-builtin-definition-tree",
        executable,
        BTreeMap::from([("source".into(), source.clone())]),
        vec![literal("overlay"), input("source", "builtins"), output("tree", "")],
        &[("tree", ArtifactKind::Tree)],
        None,
        0,
    )?;
    let builtins = builtins["tree"].clone();
    let definitions = crate::stagex_bash_full::BASH_FULL_BUILTIN_DEFINITIONS;
    for name in definitions
        .iter()
        .filter(|name| !matches!(**name, "common" | "evalstring" | "evalfile" | "reserved"))
    {
        let result = command(
            builder,
            run_generator,
            &format!("bash-full-generate-{name}"),
            generator.clone(),
            BTreeMap::from([("definitions".into(), builtins.clone())]),
            vec![literal("-D"), literal("."), literal(&format!("{name}.def"))],
            &[("generated", ArtifactKind::Tree)],
            None,
            0,
        )?;
        let step = builder
            .recipe
            .steps
            .last_mut()
            .ok_or_else(|| crate::source_built_recipe::error("missing generator leaf"))?;
        if let RecipeOperation::Exec {
            cwd_output,
            output_seeds,
            ..
        } = &mut step.operation
        {
            *cwd_output = Some("generated".into());
            output_seeds.insert("generated".into(), "definitions".into());
        }
        let text = transform(
            builder,
            run_generator,
            &format!("bash-full-project-{name}"),
            result["generated"].clone(),
            &format!("{name}.c"),
            "copy",
            &[],
        )?;
        generated.insert(format!("builtins/{name}.c"), text);
    }
    let mut args = vec![
        literal("-externfile"),
        literal("builtext.h"),
        literal("-structfile"),
        literal("builtins.c"),
        literal("-noproduction"),
        literal("-D"),
        literal("."),
    ];
    args.extend(
        definitions
            .iter()
            .filter(|name| !matches!(**name, "common" | "evalstring" | "evalfile"))
            .map(|name| literal(&format!("{name}.def"))),
    );
    let table = command(
        builder,
        run_generator,
        "bash-full-generate-table",
        generator,
        BTreeMap::from([("definitions".into(), builtins)]),
        args,
        &[("generated", ArtifactKind::Tree)],
        None,
        0,
    )?;
    let step = builder
        .recipe
        .steps
        .last_mut()
        .ok_or_else(|| crate::source_built_recipe::error("missing generator table leaf"))?;
    if let RecipeOperation::Exec {
        cwd_output,
        output_seeds,
        ..
    } = &mut step.operation
    {
        *cwd_output = Some("generated".into());
        output_seeds.insert("generated".into(), "definitions".into());
    }
    for name in ["builtext.h", "builtins.c"] {
        let text = transform(
            builder,
            run_generator,
            &format!("bash-full-project-{}", name.replace('.', "-")),
            table["generated"].clone(),
            name,
            "copy",
            &[],
        )?;
        generated.insert(format!("builtins/{name}"), text);
    }
    let inputs = generated
        .iter()
        .enumerate()
        .map(|(i, (_, reference))| (format!("file-{i:03}"), reference.clone()))
        .collect();
    let entries = generated.keys().enumerate().map(|(i, path)| (path.clone(), format!("file-{i:03}"))).collect();
    let outputs = builder.add(run_generator, RecipeStep {
        id: "bash-full-assemble-generated-builtins".into(),
        inputs,
        outputs: BTreeMap::from([("generated".into(), bounded(ArtifactKind::Tree))]),
        operation: RecipeOperation::AssembleTree {
            output: "generated".into(),
            entries,
        },
    })?;
    builder.publish(run_generator, "bash-full-generated-builtins", outputs["generated"].clone())?;

    let build = "bash-full-build-exec";
    let executable = helper(builder, build)?;
    let merged = command(
        builder,
        build,
        "bash-full-merge-generated-builtins",
        executable,
        BTreeMap::from([
            ("source".into(), source),
            ("generated".into(), outputs["generated"].clone()),
        ]),
        vec![
            literal("merge"),
            input("source", ""),
            input("generated", ""),
            output("tree", ""),
        ],
        &[("tree", ArtifactKind::Tree)],
        None,
        0,
    )?;
    let merged = merged["tree"].clone();
    let executable = helper(builder, build)?;
    let inlined = command(
        builder,
        build,
        "bash-full-fmtumax-inline",
        executable,
        BTreeMap::from([("source".into(), merged.clone())]),
        vec![
            literal("inline"),
            input("source", "lib/sh/fmtumax.c"),
            output("text", ""),
            literal("#include \"fmtulong.c\"\n"),
            input("source", "lib/sh/fmtulong.c"),
        ],
        &[("text", ArtifactKind::Text)],
        None,
        0,
    )?;
    let fmtumax = transform(builder, build, "bash-full-fmtumax-types", inlined["text"].clone(), "", "replace", &[
        "#define LONG\tintmax_t",
        "#include <stdint.h>\n#define LONG\tintmax_t",
        "1",
    ])?;
    let runtime = emit(builder, build, "bash-full-runtime-source", crate::stagex_bash_full::RUNTIME_COMPAT_SOURCE)?;
    let boundary =
        format!("{BASH_EXTERNAL_PHASE}  /* We have to make the child before we check for the non-existence\n");
    let execute =
        transform(builder, build, "bash-full-external-transport", merged.clone(), "execute_cmd.c", "replace", &[
            "  /* We have to make the child before we check for the non-existence\n",
            &boundary,
            "1",
        ])?;
    let tree = overlay(
        builder,
        build,
        "bash-full-configure",
        merged,
        BTreeMap::from([
            ("config.h".into(), config),
            ("lib/sh/fmtumax.c".into(), fmtumax),
            ("stagex-full-runtime-compat.c".into(), runtime),
            ("execute_cmd.c".into(), execute),
        ]),
    )?;
    builder.publish(build, "bash-full-configured-source", tree.clone())?;
    let mut objects = Vec::new();
    for (index, path) in crate::stagex_bash_full::full_shell_sources().into_iter().enumerate() {
        let object = compile(
            builder,
            build,
            &format!("bash-full-compile-{index:03}"),
            tree.clone(),
            &path,
            &["", "include", "lib", "builtins"],
            &["-DHAVE_CONFIG_H"],
        )?;
        objects.push(object);
    }
    let bash = link(builder, build, "bash-full-link", objects)?;
    builder.publish(build, "bash-2.05b-full", bash.clone())?;
    // Musl's startup supplies envp directly to Bash's original three-argument main.
    builder.artifacts.insert("phase-tool:bash-2.05b-full".into(), bash.clone());
    builder.artifacts.insert("member:bash-full-stage/runtime/output/bin/bash-full".into(), bash.clone());
    builder.artifacts.insert("member:bash-full-stage/runtime/output/bin/sh".into(), bash.clone());

    let smoke = "bash-full-smoke-exec";
    observation(
        builder,
        smoke,
        "bash-full-version",
        bash.clone(),
        &["--version"],
        "",
        0,
        Some("bash-full-version-observation"),
    )?;
    observation(
        builder,
        smoke,
        "bash-full-functional",
        bash.clone(),
        &[
            "-c",
            "set -eu\nvalue=mantle\nfor item in one two; do case \"$item\" in one) printf '%s\\n' first ;; two) printf '%s\\n' second ;; esac; done\nprintf '%s\\n' \"$value\"\nread line\ntest \"$line\" = input\neval 'printf \"%s\\n\" eval-ok'\n",
        ],
        "input\n",
        0,
        Some("bash-full-functional-observation"),
    )?;
    let negative = observation(builder, smoke, "bash-full-negative", bash.clone(), &["-c", "if then"], "", 2, None)?;
    let normalized = transform(builder, smoke, "bash-full-negative-normalize", negative, "", "replace", &[
        "/inputs/executable",
        "bash-full",
        "2",
    ])?;
    builder.publish(smoke, "bash-full-negative-observation", normalized)?;

    // Capture the original shell's expanded disk-command operands before its
    // fork boundary, then launch the admitted child as its own actual Exec.
    let make = builder.artifact("make-3.82")?;
    let planned = command(
        builder,
        smoke,
        "bash-full-external-command",
        bash.clone(),
        BTreeMap::from([("child".into(), make.clone())]),
        vec![
            literal("-c"),
            literal("\"$1\" --version"),
            literal("bash-full"),
            input("child", ""),
        ],
        &[("plan", ArtifactKind::Text)],
        None,
        0,
    )?;
    if let Some(RecipeStep {
        operation: RecipeOperation::Exec { environment, .. },
        ..
    }) = builder.recipe.steps.last_mut()
    {
        environment.insert("MANTLE_BASH_COMMAND_PLAN".into(), output("plan", ""));
    }
    transform(
        builder,
        smoke,
        "bash-full-external-command-stdout-assert",
        planned["stdout"].clone(),
        "",
        "equal",
        &[""],
    )?;
    let executable = helper(builder, smoke)?;
    let admitted = command(
        builder,
        smoke,
        "bash-full-external-command-gate",
        executable,
        BTreeMap::from([("child".into(), make.clone()), ("plan".into(), planned["plan"].clone())]),
        vec![
            literal("external-gate"),
            input("child", ""),
            output("program", ""),
            input("plan", ""),
            input("child", ""),
            literal("--version"),
        ],
        &[("program", ArtifactKind::Executable)],
        None,
        0,
    )?;
    let external = command(
        builder,
        smoke,
        "bash-full-external-make-version",
        admitted["program"].clone(),
        BTreeMap::new(),
        vec![literal("--version")],
        &[("stdout", ArtifactKind::Text)],
        None,
        0,
    )?;
    let external =
        verify_observation(builder, smoke, "bash-full-external-child-observation", external["stdout"].clone())?;
    builder.publish(smoke, "bash-full-external-child-observation", external)?;
    let makefile = emit(
        builder,
        smoke,
        "bash-full-makefile-source",
        "SHELL := /inputs/shell\n.SHELLFLAGS := -c\nall:\n\t@printf '%s\\n' make-shell-ok\n",
    )?;
    let commands = command(
        builder,
        smoke,
        "bash-full-make-command-generation",
        make,
        BTreeMap::from([("makefile".into(), makefile), ("shell".into(), bash.clone())]),
        vec![
            literal("-n"),
            literal("-f"),
            input("makefile", ""),
            prefixed("SHELL=", input("shell", "")),
            literal("all"),
        ],
        &[("stdout", ArtifactKind::Text)],
        None,
        0,
    )?;
    let make_shell = command(
        builder,
        smoke,
        "bash-full-make-shell",
        bash,
        BTreeMap::from([("script".into(), commands["stdout"].clone())]),
        vec![],
        &[("stdout", ArtifactKind::Text)],
        Some("script"),
        0,
    )?;
    let make_shell =
        verify_observation(builder, smoke, "bash-full-make-shell-observation", make_shell["stdout"].clone())?;
    builder.publish(smoke, "bash-full-make-shell-observation", make_shell)
}

const BASH_EXTERNAL_PHASE: &str = r#"  {
    const char *plan_path = getenv ("MANTLE_BASH_COMMAND_PLAN");
    if (plan_path)
      {
        FILE *plan;
        size_t i, total = 0;
        if (!command || redirects || pipe_in != NO_PIPE || pipe_out != NO_PIPE || async)
          { internal_error ("unsupported disk-command transport"); exit (EXECUTION_FAILURE); }
        args = strvec_from_word_list (words, 0, 0, (int *)NULL);
        plan = fopen (plan_path, "wb");
        if (!plan) exit (EXECUTION_FAILURE);
        for (i = 0; args[i]; i++)
          {
            size_t length = strlen (args[i]), j;
            if (i >= 256 || length > 1048576 || total > 8388608 - length) exit (EXECUTION_FAILURE);
            for (j = 0; j < length; j++)
              if (fprintf (plan, "%02x", (unsigned char)args[i][j]) != 2) exit (EXECUTION_FAILURE);
            if (fputc ('\n', plan) == EOF) exit (EXECUTION_FAILURE);
            total += length;
          }
        if (!i || fclose (plan)) exit (EXECUTION_FAILURE);
        /* This completes only argument transport, not the external command.
           Its actual exit status belongs to the successor child Exec. */
        exit (EXECUTION_SUCCESS);
      }
  }
"#;

/// File, source replacements, and expected application count for gawk rules.
type GawkRules = Vec<(&'static str, Vec<(&'static str, &'static str, usize)>)>;

fn gawk_rules() -> GawkRules {
    vec![
        ("builtin.c", vec![
            ("#define Floor(n) floor((n) * (1.0 + DBL_EPSILON))", "#define Floor(n) floor(n)", 1),
            ("#define Ceil(n) ceil((n) * (1.0 + DBL_EPSILON))", "#define Ceil(n) ceil(n)", 1),
            ("arg < 0.0", "arg < (AWKNUM)0", 2),
            ("d_index < 1.0", "d_index < (AWKNUM)1", 1),
            ("d_length <= 0.0", "d_length <= (AWKNUM)0", 1),
            ("rlength = -1.0", "rlength = (AWKNUM)-1", 1),
            ("tmp_number((AWKNUM) 0.0)", "tmp_number((AWKNUM)0)", 1),
            ("g == 0.0", "g == (AWKNUM)0", 1),
            ("AWKNUM retval = 0.0", "AWKNUM retval = (AWKNUM)0", 1),
            ("int sgn;", "int sgn; int mantle_conversion_ok;", 1),
            ("if (! (tmpval <= (unsigned long) ULONG_MAX))", "if (0)", 2),
            (
                "uval = - (unsigned long) (long) tmpval;",
                "uval = mantle_double_to_ulong_bounded(-tmpval, &mantle_conversion_ok); if (! mantle_conversion_ok) goto out_of_range;",
                1,
            ),
            (
                "uval = (unsigned long) (long) tmpval;",
                "uval = mantle_double_to_ulong_bounded(tmpval, &mantle_conversion_ok); if (! mantle_conversion_ok) goto out_of_range;",
                1,
            ),
            (
                "uval = (unsigned long) tmpval;",
                "uval = mantle_double_to_ulong_bounded(tmpval, &mantle_conversion_ok); if (! mantle_conversion_ok) goto out_of_range;",
                2,
            ),
        ]),
        ("eval.c", vec![
            ("? 1.0 : -1.0", "? (AWKNUM)1 : (AWKNUM)-1", 2),
            (" != 0.0", " != (AWKNUM)0", 3),
        ]),
        ("main.c", vec![
            ("Nnull_string->numbr = 0.0;", "Nnull_string->numbr = (AWKNUM)0;", 1),
            ("tmp_number(0.0)", "tmp_number((AWKNUM)0)", 1),
        ]),
        ("node.c", vec![
            (
                "if ((val = double_to_int(s->numbr)) != s->numbr\n\t    || val < LONG_MIN || val > LONG_MAX) {",
                "if ((val = double_to_int(s->numbr)) != s->numbr) {",
                1,
            ),
            (
                "\t\tif (num < NVAL && num >= 0) {\n\t\t\tsp = (char *) values[num];\n\t\t\ts->stlen = 1;\n\t\t} else {\n\t\t\t(void) sprintf(sp, \"%ld\", num);\n\t\t\ts->stlen = strlen(sp);\n\t\t}",
                "\t\t(void) sprintf(sp, \"%ld\", num);\n\t\ts->stlen = strlen(sp);",
                1,
            ),
            ("n->numbr = 0.0;", "n->numbr = (AWKNUM)0;", 1),
            ("return 0.0;", "return (AWKNUM)0;", 3),
        ]),
        ("io.c", vec![
            ("tmp_number((AWKNUM) 0.0)", "tmp_number((AWKNUM)0)", 4),
            ("tmp_number((AWKNUM) -1.0)", "tmp_number((AWKNUM)-1)", 2),
            ("tmp_number((AWKNUM) 1.0)", "tmp_number((AWKNUM)1)", 1),
        ]),
        ("awktab.c", vec![("make_number(0.0)", "make_number((AWKNUM)0)", 7)]),
    ]
}

const TRANSFORM_C: &str = r##"#define _GNU_SOURCE 1
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <errno.h>
#include <sys/stat.h>
#include <unistd.h>
#include <dirent.h>
#include <limits.h>
static unsigned long total;
static void die(const char *why) { fprintf(stderr,"stagex source transform: %s: %s\n",why,strerror(errno)); exit(1); }
static void parent(char *p) { char *s; for(s=p+1;*s;s++) if(*s=='/') { *s=0; if(mkdir(p,0755)&&errno!=EEXIST)die(p); *s='/'; } }
static FILE *out(const char *p) { char *q=strdup(p); FILE *f; if(!q)die("allocation"); parent(q);free(q); f=fopen(p,"wb");if(!f)die(p);return f; }
static char *load(const char *p,size_t *n) { FILE *f=fopen(p,"rb");long z;char *b;if(!f)die(p);if(fseek(f,0,SEEK_END)||(z=ftell(f))<0||z>67108864||fseek(f,0,SEEK_SET))die("source bound");b=malloc((size_t)z+1);if(!b)die("allocation");if(fread(b,1,(size_t)z,f)!=(size_t)z)die("read");if(fclose(f))die("close");b[z]=0;*n=(size_t)z;return b; }
static void save(const char *p,const char *b,size_t n) { FILE *f=out(p);if(fwrite(b,1,n,f)!=n||fclose(f))die("write"); }
static void copy(const char *a,const char *b,unsigned depth) { struct stat st;FILE *i,*o;char block[8192];size_t n;DIR *d;struct dirent *e;char x[PATH_MAX],y[PATH_MAX];if(depth>64||lstat(a,&st))die("tree depth or stat");if(S_ISDIR(st.st_mode)){if(mkdir(b,0755)&&errno!=EEXIST)die(b);d=opendir(a);if(!d)die(a);while((e=readdir(d))) { if(!strcmp(e->d_name,".")||!strcmp(e->d_name,".."))continue;if(snprintf(x,sizeof x,"%s/%s",a,e->d_name)>=(int)sizeof x||snprintf(y,sizeof y,"%s/%s",b,e->d_name)>=(int)sizeof y)die("path bound");copy(x,y,depth+1);}closedir(d);return;}if(S_ISLNK(st.st_mode)){ssize_t z=readlink(a,x,sizeof x-1);if(z<0||z==(ssize_t)sizeof x-1)die("symlink");x[z]=0;if(symlink(x,b))die(b);return;}if(!S_ISREG(st.st_mode)||st.st_size<0||(unsigned long)st.st_size>67108864||total+(unsigned long)st.st_size>536870912)die("tree bound");total+=(unsigned long)st.st_size;i=fopen(a,"rb");if(!i)die(a);o=out(b);while((n=fread(block,1,sizeof block,i)))if(fwrite(block,1,n,o)!=n)die("copy write");if(ferror(i)||fclose(i)||fclose(o)||chmod(b,st.st_mode&0777))die("copy close"); }
static int safe(const char *p){const char *s=p;if(!*p||*p=='/')return 0;while(*s){const char *e=strchr(s,'/');size_t n=e?(size_t)(e-s):strlen(s);if(!n||(n==1&&s[0]=='.')||(n==2&&s[0]=='.'&&s[1]=='.'))return 0;if(!e)break;s=e+1;}return 1;}
int main(int c,char **v) {
 size_t n;char *b,*p,*q;FILE *f;int i;
 if(c==4&&!strcmp(v[1],"emit")){save(v[2],v[3],strlen(v[3]));return 0;}
 if(c==5&&!strcmp(v[1],"receipt")){b=load(v[2],&n);if(!n)die("missing rejection diagnostic");free(b);save(v[3],v[4],strlen(v[4]));return 0;}
 if(c>=4&&!strcmp(v[1],"overlay")&&!(c%2)){char path[PATH_MAX];copy(v[2],v[3],0);for(i=4;i<c;i+=2){if(!safe(v[i])||snprintf(path,sizeof path,"%s/%s",v[3],v[i])>=(int)sizeof path)die("overlay path");if(unlink(path)&&errno!=ENOENT)die("overlay removal");copy(v[i+1],path,0);}return 0;}
 if(c==5&&!strcmp(v[1],"merge")){copy(v[2],v[4],0);copy(v[3],v[4],0);return 0;}
 if(c<4)die("arguments");b=load(v[2],&n);
 if(c==5&&!strcmp(v[1],"pipe-header-script")){size_t z,j;char *measurement=load(v[4],&z),*start,*end,*line,*sub,*tail;if(z<2||z>16||measurement[z-1]!='\n')die("pipe probe decimal");for(j=0;j+1<z;j++)if(measurement[j]<'0'||measurement[j]>'9')die("pipe probe decimal");if(measurement[0]=='0')die("empty pipe capacity");start=strstr(b,"echo \"/*\"\n");end=start?strstr(start,"\n#\n# Try to avoid tempfile races."):NULL;line=strstr(b,"\techo \"#define PIPESIZE `cat \"$TMPFILE\"`\"\n");sub=line?strstr(line,"`cat \"$TMPFILE\"`"):NULL;tail=sub?sub+strlen("`cat \"$TMPFILE\"`"):NULL;if(!start||!end||!line||!sub||!tail)die("psize.sh source anchors");f=out(v[3]);if(fwrite(start,1,(size_t)(end-start),f)!=(size_t)(end-start)||fputc('\n',f)==EOF||fwrite(line+1,1,(size_t)(sub-line-1),f)!=(size_t)(sub-line-1)||fwrite(measurement,1,z-1,f)!=z-1||fwrite(tail,1,2,f)!=2||fclose(f))die("pipe header script");free(measurement);free(b);return 0;}
 if(c>=6&&!strcmp(v[1],"external-gate")){size_t z,at=0;char *plan=load(v[4],&z);const char *hex="0123456789abcdef";for(i=5;i<c;i++){size_t j,length=strlen(v[i]);if(length>(z-at)/2)die("shell external operands");for(j=0;j<length;j++){unsigned char byte=(unsigned char)v[i][j];if(plan[at++]!=hex[byte>>4]||plan[at++]!=hex[byte&15])die("shell external operands");}if(at>=z||plan[at++]!='\n')die("shell external operand separator");}if(at!=z)die("shell external operand count");save(v[3],b,n);if(chmod(v[3],0755))die("external executable mode");free(plan);free(b);return 0;}
 if(c==6&&!strcmp(v[1],"extract")){p=strstr(b,v[4]);if(!p||strstr(p+strlen(v[4]),v[4]))die("extract start count");p+=strlen(v[4]);if(!*v[5]){save(v[3],"",0);free(b);return 0;}q=strstr(p,v[5]);if(!q)die("extract end");f=out(v[3]);if(fwrite(p,1,(size_t)(q-p),f)!=(size_t)(q-p)||fputc('\n',f)==EOF||fclose(f))die("extract write");}
 else if(c==4&&!strcmp(v[1],"copy")){save(v[3],b,n);}
 else if(c==5&&!strcmp(v[1],"equal")){if(n!=strlen(v[4])||memcmp(b,v[4],n))die("observation differs");save(v[3],b,n);}
 else if(c==5&&!strcmp(v[1],"contains")){if(!strstr(b,v[4]))die("observation marker missing");save(v[3],b,n);}
 else if(c==5&&!strcmp(v[1],"not-equal")){if(n==strlen(v[4])&&!memcmp(b,v[4],n))die("forbidden observation");save(v[3],b,n);}
 else if(c==4&&!strcmp(v[1],"nonempty")){if(!n)die("missing diagnostic");save(v[3],b,n);}
 else if(c==5&&!strcmp(v[1],"append")){f=out(v[3]);if(fwrite(b,1,n,f)!=n||fwrite(v[4],1,strlen(v[4]),f)!=strlen(v[4])||fclose(f))die("append");}
 else if(c==6&&!strcmp(v[1],"inline")){size_t z;char *a=load(v[5],&z);p=strstr(b,v[4]);if(!p||strstr(p+strlen(v[4]),v[4]))die("inline count");f=out(v[3]);if(fwrite(b,1,(size_t)(p-b),f)!=(size_t)(p-b)||fwrite(a,1,z,f)!=z||fwrite(p+strlen(v[4]),1,strlen(p+strlen(v[4])),f)!=strlen(p+strlen(v[4]))||fclose(f))die("inline write");free(a);}
 else if(c==7&&!strcmp(v[1],"replace")){size_t old=strlen(v[4]),z=strlen(v[5]),count=0;char *s=b;if(!old)die("empty replacement");while((p=strstr(s,v[4]))){count++;s=p+old;}if(count!=(size_t)strtoul(v[6],0,10))die("replacement count");f=out(v[3]);s=b;while((p=strstr(s,v[4]))){if(fwrite(s,1,(size_t)(p-s),f)!=(size_t)(p-s)||fwrite(v[5],1,z,f)!=z)die("replacement write");s=p+old;}if(fwrite(s,1,strlen(s),f)!=strlen(s)||fclose(f))die("replacement close");}
 else if(c==4&&!strcmp(v[1],"m4-disable")){size_t a=0,z=0;f=out(v[3]);p=b;while(*p){q=strchr(p,'\n');if(!q)q=p+strlen(p);{char keep=*q;*q=0;if(strstr(p,"{ \"esyscmd\","))a++;else if(strstr(p,"{ \"syscmd\","))z++;else if(fwrite(p,1,(size_t)(q-p),f)!=(size_t)(q-p)||fputc('\n',f)==EOF)die("builtin write");*q=keep;}p=*q?q+1:q;}if(a!=1||z!=1||fclose(f))die("builtin shape");}
 else die("operation");free(b);return 0;
}
"##;
