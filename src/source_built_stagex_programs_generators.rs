use std::collections::BTreeMap;

use super::TransitionBuilder;
use super::bounded;
use super::error;
use super::input;
use super::late::command;
use super::late::compile;
use super::late::emit;
use super::late::link;
use super::late::observation;
use super::late::overlay;
use super::late::transform;
use super::literal;
use super::output;
use crate::errors::RunError;
use crate::source_built_program_authoring::AuthoringRequest;
use crate::source_built_recipe::ArtifactKind;
use crate::source_built_recipe::ArtifactRef;
use crate::source_built_recipe::RecipeOperation;
use crate::source_built_recipe::RecipeStep;
use crate::source_built_recipe::RecipeWord;

// These adapters split the original generators at their existing pipe boundary.
// The grammar algorithms, skeletons, muscle serialization, skeleton scanner and
// Flex tee remain the original sources. Only process transport is replaced.
// The owner's normalized Flex line filter is the byte identity; capturing M4's
// actual stdout preserves it without a redundant process or copied snapshot.
pub(super) fn extend(b: &mut TransitionBuilder, _: &AuthoringRequest<'_>) -> Result<(), RunError> {
    bison(b)?;
    flex(b)
}

fn editor(b: &mut TransitionBuilder, tool: &str) -> Result<ArtifactRef, RunError> {
    if let Some(program) = b.artifacts.get("parser-source-editor") {
        return Ok(program.clone());
    }
    let source = b.authored("helpers/stagex-parser-source-editor.c", EDITOR_C.into())?;
    let object = compile(b, tool, "parser-source-editor-compile", source, "", &[], &[])?;
    let program = link(b, tool, "parser-source-editor-link", vec![object])?;
    b.artifacts.insert("parser-source-editor".into(), program.clone());
    Ok(program)
}

fn edit_source(
    b: &mut TransitionBuilder,
    tool: &str,
    id: &str,
    source: ArtifactRef,
    path: &str,
    operation: &str,
    parameters: &[&str],
) -> Result<ArtifactRef, RunError> {
    let executable = editor(b, tool)?;
    let mut args = vec![literal(operation), input("source", path), output("text", "")];
    args.extend(parameters.iter().map(|p| literal(p)));
    let result = command(
        b,
        tool,
        id,
        executable,
        BTreeMap::from([("source".into(), source)]),
        args,
        &[("text", ArtifactKind::Text)],
        None,
        0,
    )?;
    Ok(result["text"].clone())
}

fn replace(
    b: &mut TransitionBuilder,
    tool: &str,
    id: &str,
    source: ArtifactRef,
    path: &str,
    old: &str,
    new: &str,
) -> Result<ArtifactRef, RunError> {
    transform(b, tool, id, source, path, "replace", &[old, new, "1"])
}

fn fresh_command(
    b: &mut TransitionBuilder,
    tool: &str,
    id: &str,
    executable: ArtifactRef,
    inputs: BTreeMap<String, ArtifactRef>,
    arguments: Vec<RecipeWord>,
    environment: BTreeMap<String, RecipeWord>,
    stdin: Option<&str>,
    code: i32,
) -> Result<BTreeMap<String, ArtifactRef>, RunError> {
    let result =
        command(b, tool, id, executable, inputs, arguments, &[("generated", ArtifactKind::Tree)], stdin, code)?;
    let step = b.recipe.steps.last_mut().ok_or_else(|| error("missing parser phase"))?;
    if let RecipeOperation::Exec {
        cwd_output,
        environment: env,
        ..
    } = &mut step.operation
    {
        *cwd_output = Some("generated".into());
        env.extend(environment);
    }
    Ok(result)
}

fn assemble(
    b: &mut TransitionBuilder,
    tool: &str,
    id: &str,
    files: BTreeMap<String, ArtifactRef>,
) -> Result<ArtifactRef, RunError> {
    let inputs = files.iter().enumerate().map(|(i, (_, r))| (format!("file-{i:03}"), r.clone())).collect();
    let entries = files.keys().enumerate().map(|(i, name)| (name.clone(), format!("file-{i:03}"))).collect();
    let result = b.add(tool, RecipeStep {
        id: id.into(),
        inputs,
        outputs: BTreeMap::from([("generated".into(), bounded(ArtifactKind::Tree, 512 * 1024 * 1024))]),
        operation: RecipeOperation::AssembleTree {
            output: "generated".into(),
            entries,
        },
    })?;
    Ok(result["generated"].clone())
}

fn grammar_filename(path: &str, args: &[&str], extension: &str) -> String {
    if !path.is_empty() {
        return path.rsplit('/').next().unwrap_or(path).to_string();
    }
    let mut name = "input";
    for (i, arg) in args.iter().enumerate() {
        if matches!(*arg, "-o" | "--output" | "--outfile") {
            if let Some(value) = args.get(i + 1) {
                name = value;
            }
        } else if let Some(value) = arg.strip_prefix("--output=").or_else(|| arg.strip_prefix("--outfile=")) {
            name = value;
        }
    }
    let name = name.rsplit('/').next().unwrap_or(name);
    format!("{}.{extension}", name.rsplit_once('.').map_or(name, |(stem, _)| stem))
}

fn m4_ready_config(
    b: &mut TransitionBuilder,
    tool: &str,
    id: &str,
    config: ArtifactRef,
) -> Result<ArtifactRef, RunError> {
    let m4 = b.artifact("m4-1.4.7")?;
    let ready = command(
        b,
        tool,
        &format!("{id}-m4-readiness"),
        m4,
        BTreeMap::new(),
        vec![literal("--version")],
        &[],
        None,
        0,
    )?;
    let helper = editor(b, tool)?;
    let gated = command(
        b,
        tool,
        &format!("{id}-m4-config-gate"),
        helper,
        BTreeMap::from([("config".into(), config), ("version".into(), ready["stdout"].clone())]),
        vec![
            literal("readiness"),
            input("config", ""),
            output("text", ""),
            input("version", ""),
            literal("GNU M4 1.4.7"),
        ],
        &[("text", ArtifactKind::Text)],
        None,
        0,
    )?;
    Ok(gated["text"].clone())
}

/// Validate concrete original inventory/graph prerequisites before releasing
/// source to a successor. Observation predicates remain the original owners'
/// source policies; object, archive and tree gates inspect the actual products.
pub(super) fn readiness(
    b: &mut TransitionBuilder,
    tool: &str,
    id: &str,
    source: ArtifactRef,
    path: &str,
) -> Result<ArtifactRef, RunError> {
    let mut predecessors = b.original_inputs(tool)?;
    predecessors.extend(b.predecessors(tool)?);
    let mut inputs = BTreeMap::from([("source".into(), source)]);
    let mut arguments = vec![literal("gate"), input("source", path), output("text", "")];
    for (index, (name, mut reference)) in predecessors.into_iter().enumerate() {
        let kind = match &reference {
            ArtifactRef::Produced { action_id, output_id } => b
                .recipe
                .steps
                .iter()
                .find(|step| action_id == &format!("{}/{}", b.recipe.stage_id, step.id))
                .and_then(|step| step.outputs.get(output_id))
                .map(|output| output.kind),
            _ => None,
        };
        if kind == Some(ArtifactKind::Text) {
            for (check, (operation, expected)) in super::late::observation_checks(&name).iter().enumerate() {
                reference =
                    transform(b, tool, &format!("{id}-ready-{index:03}-{check:02}"), reference, "", operation, &[
                        *expected,
                    ])?;
            }
        }
        let format = match kind {
            Some(ArtifactKind::Executable) => "elf",
            Some(ArtifactKind::Object) => "object",
            Some(ArtifactKind::Archive) => "archive",
            Some(ArtifactKind::Tree) => "tree",
            Some(ArtifactKind::Text)
                if matches!(name.as_str(), "tcc-musl-v2-positive-execution" | "grep-quiet-observation") =>
            {
                "empty"
            }
            Some(ArtifactKind::Text) => "text",
            None => "source",
        };
        let label = format!("predecessor-{index:03}");
        arguments.extend([literal(format), input(&label, "")]);
        inputs.insert(label, reference);
    }
    let helper = editor(b, tool)?;
    let result = command(b, tool, id, helper, inputs, arguments, &[("text", ArtifactKind::Text)], None, 0)?;
    Ok(result["text"].clone())
}

fn rejection(
    b: &mut TransitionBuilder,
    tool: &str,
    id: &str,
    result: &BTreeMap<String, ArtifactRef>,
    name: &str,
    absent: bool,
) -> Result<ArtifactRef, RunError> {
    let helper = editor(b, tool)?;
    let checked = command(
        b,
        tool,
        &format!("{id}-rejection-assert"),
        helper,
        BTreeMap::from([
            ("diagnostic".into(), result["stderr"].clone()),
            ("stdout".into(), result["stdout"].clone()),
            ("generated".into(), result["generated"].clone()),
        ]),
        vec![
            literal("rejection"),
            input("diagnostic", ""),
            output("text", ""),
            input("stdout", ""),
            input("generated", name),
            literal(if absent { "absent" } else { "empty" }),
        ],
        &[("text", ArtifactKind::Text)],
        None,
        0,
    )?;
    Ok(checked["text"].clone())
}

fn bison(b: &mut TransitionBuilder) -> Result<(), RunError> {
    use crate::stagex_bison as policy;
    let tool = "bison-build-exec";
    let source = b.fetcher(tool, "bison-2.3-source")?;
    let recipe = b.source("bison-2.3-recipe-source")?;
    let config = emit(b, tool, "bison-config-header", &policy::bison_config_header())?;
    let config = readiness(b, tool, "bison-original-readiness", config, "")?;
    let config = m4_ready_config(b, tool, "bison", config)?;
    let runtime = transform(b, tool, "bison-runtime-source", recipe, "", "extract", &[
        "      $BB cat > mantle_runtime_compat.c <<'RUNTIME_EOF'\n",
        "\nRUNTIME_EOF\n",
    ])?;
    let mut hash = replace(
        b,
        tool,
        "bison-hash-growth-constants",
        source.clone(),
        "lib/hash.c",
        "#define DEFAULT_GROWTH_THRESHOLD 0.8\n#define DEFAULT_GROWTH_FACTOR 1.414",
        "#define MANTLE_HASH_RATIO_DENOMINATOR 1000\n#define MANTLE_HASH_GROWTH_THRESHOLD_NUMERATOR 800\n#define MANTLE_HASH_GROWTH_FACTOR_NUMERATOR 1414\n#define MANTLE_HASH_COMBINED_GROWTH_NUMERATOR 11312\n#define MANTLE_HASH_COMBINED_GROWTH_DENOMINATOR 10000\n#define DEFAULT_GROWTH_THRESHOLD ((float) 8 / (float) 10)\n#define DEFAULT_GROWTH_FACTOR ((float) 1414 / (float) 1000)",
    )?;
    for (id, old, new) in [
        (
            "shrink",
            "#define DEFAULT_SHRINK_THRESHOLD 0.0\n#define DEFAULT_SHRINK_FACTOR 1.0",
            "#define DEFAULT_SHRINK_THRESHOLD ((float) 0)\n#define DEFAULT_SHRINK_FACTOR ((float) 1)",
        ),
        ("tuning", policy::CHECK_TUNING_OLD, policy::CHECK_TUNING_NEW),
        ("candidate", policy::INITIAL_CANDIDATE_OLD, policy::INITIAL_CANDIDATE_NEW),
        ("rehash", policy::GROWTH_REHASH_OLD, policy::GROWTH_REHASH_NEW),
    ] {
        hash = replace(b, tool, &format!("bison-hash-{id}"), hash, "", old, new)?;
    }
    let configured = overlay(
        b,
        tool,
        "bison-configure",
        source.clone(),
        BTreeMap::from([
            ("config.h".into(), config),
            ("mantle_runtime_compat.c".into(), runtime),
            ("lib/hash.c".into(), hash),
        ]),
    )?;
    b.publish(tool, "bison-configured-source", configured.clone())?;
    let phase = replace(
        b,
        tool,
        "bison-output-transport-state",
        configured.clone(),
        "src/output.c",
        "  FILE *in;\n  FILE *out;\n  int filter_fd[2];\n  char const *argv[6];\n  pid_t pid;",
        "  FILE *out;\n  FILE *state;\n  const char *state_path = getenv (\"MANTLE_BISON_STATE\");",
    )?;
    let phase = edit_source(b, tool, "bison-split-output-pipe", phase, "", "region", &[
        "  argv[0] = m4;\n",
        "  /* Output the definitions of all the muscles.  */\n",
        BISON_PIPE_OPEN,
    ])?;
    let phase = edit_source(b, tool, "bison-split-skeleton-pipe", phase, "", "region", &[
        "  xfclose (out);\n\n  /* Read and process m4's output.  */\n",
        "  timevar_pop (TV_M4);\n",
        BISON_PIPE_CLOSE,
    ])?;
    let main_phase = format!("{GRAMMAR_TRANSPORT_C}{BISON_MAIN_PHASE}");
    let main =
        transform(b, tool, "bison-append-continuation", configured.clone(), "src/main.c", "append", &[&main_phase])?;
    let adapted = overlay(
        b,
        tool,
        "bison-phase-source",
        configured,
        BTreeMap::from([("src/output.c".into(), phase), ("src/main.c".into(), main)]),
    )?;
    let mut objects = Vec::new();
    for (directory, names) in [
        ("lib", policy::BISON_LIBRARY_SOURCES),
        ("src", policy::BISON_PROGRAM_SOURCES),
    ] {
        for name in names {
            let flags: &[&str] = if directory == "src" && *name == "main" {
                &["-DHAVE_CONFIG_H", "-Dmain=mantle_bison_original_main"]
            } else {
                &["-DHAVE_CONFIG_H"]
            };
            let object = compile(
                b,
                tool,
                &format!("bison-{directory}-{name}-compile"),
                adapted.clone(),
                &format!("{directory}/{name}.c"),
                &["", "lib", "src"],
                flags,
            )?;
            objects.push(object);
        }
    }
    let runtime = compile(b, tool, "bison-runtime-compile", adapted, "mantle_runtime_compat.c", &["", "lib"], &[])?;
    objects.push(runtime);
    let program = link(b, tool, "bison-link", objects)?;
    b.publish(tool, "bison-2.3", program.clone())?;
    b.artifacts.insert("phase-tool:bison-2.3".into(), program.clone());
    let helper = super::late::helper(b, tool)?;
    let data = command(
        b,
        tool,
        "bison-install-runtime-data",
        helper,
        BTreeMap::from([("source".into(), source)]),
        vec![literal("overlay"), input("source", "data"), output("tree", "")],
        &[("tree", ArtifactKind::Tree)],
        None,
        0,
    )?;
    let data = data["tree"].clone();
    b.publish(tool, "bison-2.3-runtime-data", data)?;
    let smoke = "bison-smoke-exec";
    observation(b, smoke, "bison-version", program.clone(), &["--version"], "", 0, Some("bison-version-observation"))?;
    let positive = emit(b, smoke, "bison-positive-grammar", "%token WORD\n%%\ninput: WORD ;\n%%\n")?;
    let positive = readiness(b, smoke, "bison-smoke-original-readiness", positive, "")?;
    let generated = bison_generate(b, smoke, "bison-positive", positive, "", &["-d", "-o", "positive.c"])?;
    for (name, published) in [
        ("positive.c", "bison-positive-parser-source"),
        ("positive.h", "bison-positive-parser-header"),
    ] {
        let text = transform(
            b,
            smoke,
            &format!("bison-positive-project-{}", name.replace('.', "-")),
            generated.clone(),
            name,
            "nonempty",
            &[],
        )?;
        b.publish(smoke, published, text)?;
    }
    let negative = emit(b, smoke, "bison-negative-grammar", "%token WORD\n%%\ninput WORD ;\n%%\n")?;
    let result = bison_prepare(b, smoke, "bison-negative", negative, "", &["-o", "negative.c"], 1)?;
    let diagnostic = rejection(b, smoke, "bison-negative", &result, "negative.c", true)?;
    b.publish(smoke, "bison-negative-observation", diagnostic)
}

fn bison_prepare(
    b: &mut TransitionBuilder,
    tool: &str,
    id: &str,
    grammar: ArtifactRef,
    path: &str,
    args: &[&str],
    code: i32,
) -> Result<BTreeMap<String, ArtifactRef>, RunError> {
    let program = b.artifact("bison-2.3")?;
    let data = b.artifact("bison-2.3-runtime-data")?;
    let m4 = b.artifact("m4-1.4.7")?;
    let mut arguments: Vec<_> = args.iter().map(|s| literal(s)).collect();
    arguments.push(input("grammar", path));
    let name = grammar_filename(path, args, "y");
    let result = fresh_command(
        b,
        tool,
        &format!("{id}-prepare"),
        program,
        BTreeMap::from([("grammar".into(), grammar), ("data".into(), data), ("m4".into(), m4)]),
        arguments,
        BTreeMap::from([
            ("BISON_PKGDATADIR".into(), literal("/mantle/stagex/bison-data")),
            ("M4".into(), input("m4", "")),
            ("MANTLE_GRAMMAR_NAME".into(), literal(&name)),
            ("MANTLE_BISON_STATE".into(), output("generated", "state")),
        ]),
        None,
        code,
    )?;
    if let Some(step) = b.recipe.steps.last_mut()
        && let RecipeOperation::Exec { runtime_mounts, .. } = &mut step.operation
    {
        runtime_mounts.insert("/mantle/stagex/bison-data".into(), "data".into());
    }
    Ok(result)
}

/// Serialize original muscles, expand the selected original skeleton with the
/// immutable M4 producer, then run the original skeleton scanner in a fresh tree.
/// `args` excludes the grammar operand and names outputs relative to that tree.
pub(super) fn bison_generate(
    b: &mut TransitionBuilder,
    tool: &str,
    id: &str,
    grammar: ArtifactRef,
    grammar_path: &str,
    args: &[&str],
) -> Result<ArtifactRef, RunError> {
    let prepared = bison_prepare(b, tool, id, grammar, grammar_path, args, 0)?;
    let m4 = b.artifact("m4-1.4.7")?;
    let data = b.artifact("bison-2.3-runtime-data")?;
    let expanded = command(
        b,
        tool,
        &format!("{id}-m4"),
        m4,
        BTreeMap::from([("data".into(), data), ("muscles".into(), prepared["stdout"].clone())]),
        vec![input("data", "m4sugar/m4sugar.m4"), literal("-")],
        &[],
        Some("muscles"),
        0,
    )?;
    if let Some(step) = b.recipe.steps.last_mut()
        && let RecipeOperation::Exec { runtime_mounts, .. } = &mut step.operation
    {
        runtime_mounts.insert("/mantle/stagex/bison-data".into(), "data".into());
    }
    let program = b.artifact("bison-2.3")?;
    let resumed = fresh_command(
        b,
        tool,
        &format!("{id}-resume"),
        program,
        BTreeMap::from([
            ("prepared".into(), prepared["generated"].clone()),
            ("expanded".into(), expanded["stdout"].clone()),
        ]),
        vec![literal("--mantle-resume"), input("prepared", "state")],
        BTreeMap::new(),
        Some("expanded"),
        0,
    )?;
    transform(b, tool, &format!("{id}-terminal-stdout-assert"), resumed["stdout"].clone(), "", "equal", &[""])?;
    Ok(resumed["generated"].clone())
}

fn flex(b: &mut TransitionBuilder) -> Result<(), RunError> {
    use crate::stagex_flex as policy;
    let tool = "flex-build-exec";
    let source = b.fetcher(tool, "flex-2.6.4-source")?;
    let recipe = b.source("flex-2.6.4-recipe-source")?;
    let config = emit(b, tool, "flex-config-header", &policy::flex_config_header())?;
    let config = readiness(b, tool, "flex-original-readiness", config, "")?;
    let config = m4_ready_config(b, tool, "flex", config)?;
    let runtime = transform(b, tool, "flex-runtime-source", recipe, "", "extract", &[
        "      $BB cat > /tmp/flex-compat.c <<'COMPAT'\n",
        "\nCOMPAT\n",
    ])?;
    let filter = edit_source(b, tool, "flex-external-filter-policy", source.clone(), "src/filter.c", "function", &[
        "struct filter *filter_create_ext (struct filter *chain, const char *cmd,",
        policy::FILTER_CREATE_EXT_NEW,
    ])?;
    let filter = edit_source(b, tool, "flex-linedir-policy", filter, "", "region", &[
        "int filter_fix_linedirs (struct filter *chain)\n",
        "/* vim:set expandtab cindent tabstop=4 softtabstop=4 shiftwidth=4 textwidth=0: */\n",
        policy::FILTER_FIX_LINEDIRS_NEW,
    ])?;
    let mut filter = replace(
        b,
        tool,
        "flex-header-fopen-policy",
        filter,
        "",
        "if (freopen ((char *) chain->extra, \"w\", stdout) == NULL)",
        "fclose(stdout); if ((stdout = fopen ((char *) chain->extra, \"w\")) == NULL)",
    )?;
    filter = replace(
        b,
        tool,
        "flex-header-diagnostic-policy",
        filter,
        "",
        "flexfatal (_(\"freopen(headerfilename) failed\"));",
        "flexfatal (_(\"fopen(headerfilename) failed\"));",
    )?;
    filter = replace(
        b,
        tool,
        "flex-declared-exec-policy",
        filter,
        "",
        "execvp (chain->argv[0],",
        "flex_exec_declared (chain->argv[0],",
    )?;
    let mut misc = edit_source(b, tool, "flex-error-policy", source.clone(), "src/misc.c", "function", &[
        "void lerr (const char *msg, ...)",
        policy::LERR_NEW,
    ])?;
    for (id, signature, replacement) in [
        ("fatal", "void lerr_fatal (const char *msg, ...)", policy::LERR_FATAL_NEW),
        ("allocate", "void   *allocate_array (int size, size_t element_size)", policy::ALLOCATE_ARRAY_NEW),
        (
            "reallocate",
            "void   *reallocate_array (void *array, int size, size_t element_size)",
            policy::REALLOCATE_ARRAY_NEW,
        ),
    ] {
        misc = edit_source(b, tool, &format!("flex-{id}-policy"), misc, "", "function", &[signature, replacement])?;
    }
    misc = replace(
        b,
        tool,
        "flex-decimal-policy",
        misc,
        "",
        "(void) sscanf (array, \"%d\", &val);",
        "val = flex_parse_decimal(array);",
    )?;
    let mut main = replace(
        b,
        tool,
        "flex-main-wrapper-policy",
        source.clone(),
        "src/main.c",
        "int main (int argc, char *argv[])\n{",
        policy::FLEX_MAIN_WRAPPER_NEW,
    )?;
    for (id, old, new) in [
        ("decimal", "(size_t)(1 + ceil (log10(i))) + 2", "FLEX_DECIMAL_RENDER_CAPACITY"),
        (
            "stdout",
            "prev_stdout = freopen (outfilename, \"w+\", stdout);",
            "fclose(stdout); prev_stdout = stdout = fopen (outfilename, \"w+\");",
        ),
        (
            "allocation",
            "action_array = allocate_character_array (action_size);",
            "action_array = (char *)malloc((size_t)action_size); if (action_array == NULL) flexfatal(\"action allocation failed\");",
        ),
    ] {
        main = replace(b, tool, &format!("flex-main-{id}-policy"), main, "", old, new)?;
    }
    main = edit_source(b, tool, "flex-initialization-policy", main, "", "region", &[
        "\tprintstats = syntaxerror = trace = spprdflt = false;\n",
        "\ttablesfilename = tablesname = NULL;\n",
        policy::FLEX_GLOBAL_INITIALIZATION_NEW,
    ])?;
    main = edit_source(b, tool, "flex-version-policy", main, "", "region", &[
        "\t\tcase OPT_VERSION:\n",
        "\t\t\tFLEX_EXIT (0);\n",
        policy::FLEX_VERSION_CASE_NEW,
    ])?;
    let buf = replace(
        b,
        tool,
        "flex-buffer-decimal-policy",
        source.clone(),
        "src/buf.c",
        "(size_t) (1 + ceil (log10 (abs (lineno))))",
        "FLEX_DECIMAL_RENDER_CAPACITY",
    )?;
    let configured = overlay(
        b,
        tool,
        "flex-configure",
        source,
        BTreeMap::from([
            ("config.h".into(), config),
            ("mantle_flex_compat.c".into(), runtime),
            ("src/filter.c".into(), filter.clone()),
            ("src/misc.c".into(), misc),
            ("src/main.c".into(), main.clone()),
            ("src/buf.c".into(), buf),
        ]),
    )?;
    b.publish(tool, "flex-configured-source", configured.clone())?;
    filter = edit_source(b, tool, "flex-split-filter-pipe", filter, "", "function", &[
        "bool filter_apply_chain (struct filter * chain)",
        &flex_prepare_phase_source(),
    ])?;
    filter = edit_source(b, tool, "flex-sequential-tee-streams", filter, "", "region", &[
        "\t/* Store a copy of the stdout pipe,",
        "\t/* Now to_c is a pipe to the C branch, and to_h is a pipe to the H branch.\n\t */",
        FLEX_TEE_STREAMS,
    ])?;
    filter = replace(
        b,
        tool,
        "flex-sequential-tee-return",
        filter,
        "",
        "\twhile (wait (0) > 0) ;\n\n\tFLEX_EXIT (0);\n\treturn 0;",
        "\treturn 0;",
    )?;
    filter = replace(b, tool, "flex-remove-pipe-descriptor", filter, "", "\tint     to_cfd = -1;\n", "")?;
    main = replace(
        b,
        tool,
        "flex-remove-child-status",
        main,
        "",
        "int     i, exit_status, child_status;",
        "int     i, exit_status;",
    )?;
    main = edit_source(b, tool, "flex-remove-child-reaping", main, "", "region", &[
        "        while (wait(&child_status) > 0){\n",
        "        return exit_status - 1;\n",
        "        return exit_status - 1;\n",
    ])?;
    let main_phase = flex_native_main_source();
    main = transform(b, tool, "flex-append-continuation", main, "", "append", &[&main_phase])?;
    let adapted = overlay(
        b,
        tool,
        "flex-phase-source",
        configured,
        BTreeMap::from([("src/filter.c".into(), filter), ("src/main.c".into(), main)]),
    )?;
    let mut objects = Vec::new();
    for name in policy::FLEX_PROGRAM_SOURCES {
        let flags: &[&str] = if *name == "main" {
            &["-DHAVE_CONFIG_H", "-Dmain=mantle_flex_original_main"]
        } else {
            &["-DHAVE_CONFIG_H"]
        };
        let object = compile(
            b,
            tool,
            &format!("flex-{name}-compile"),
            adapted.clone(),
            &format!("src/{name}.c"),
            &["", "src"],
            flags,
        )?;
        objects.push(object);
    }
    let runtime = compile(b, tool, "flex-runtime-compile", adapted.clone(), "mantle_flex_compat.c", &["", "src"], &[
        "-DHAVE_CONFIG_H",
    ])?;
    objects.push(runtime);
    let program = link(b, tool, "flex-link", objects)?;
    let helper = editor(b, tool)?;
    let runtime_bytes: String = policy::FLEX_RUNTIME_SECTION_BYTES.iter().map(|byte| format!("{byte:02x}")).collect();
    let canonical = command(
        b,
        tool,
        "flex-canonical-runtime-section",
        helper,
        BTreeMap::from([("program".into(), program)]),
        vec![
            literal("flex-elf"),
            input("program", ""),
            output("program", ""),
            literal(&runtime_bytes),
        ],
        &[("program", ArtifactKind::Executable)],
        None,
        0,
    )?;
    let program = canonical["program"].clone();
    b.publish(tool, "flex-2.6.4", program.clone())?;
    b.artifacts.insert("phase-tool:flex-2.6.4".into(), program.clone());
    let header = transform(b, tool, "flex-install-header", adapted, "src/FlexLexer.h", "copy", &[])?;
    b.publish(tool, "flex-cxx-header", header)?;
    let lib_source = emit(b, tool, "flex-libfl-source", "int yywrap(void) { return 1; }\n")?;
    let lib_object = compile(b, tool, "flex-libfl-compile", lib_source, "", &[], &[])?;
    let helper = editor(b, tool)?;
    let archive = command(
        b,
        tool,
        "flex-libfl-archive",
        helper,
        BTreeMap::from([("object".into(), lib_object)]),
        vec![literal("archive"), input("object", ""), output("archive", "")],
        &[("archive", ArtifactKind::Archive)],
        None,
        0,
    )?;
    b.publish(tool, "flex-libfl", archive["archive"].clone())?;
    let consumer_source = emit(
        b,
        tool,
        "flex-libfl-consumer-source",
        "int yywrap(void);\nint main(void) { return yywrap() == 1 ? 0 : 1; }\n",
    )?;
    let consumer_object = compile(b, tool, "flex-libfl-consumer-compile", consumer_source, "", &[], &[])?;
    let consumer = link(b, tool, "flex-libfl-consumer-link", vec![consumer_object, archive["archive"].clone()])?;
    b.publish(tool, "flex-libfl-consumer", consumer.clone())?;
    let smoke = "flex-smoke-exec";
    let empty = emit(b, smoke, "flex-libfl-consumer-smoke-input", "")?;
    let consumer_result = command(
        b,
        smoke,
        "flex-libfl-consumer-smoke",
        consumer,
        BTreeMap::from([("stdin".into(), empty)]),
        vec![],
        &[],
        Some("stdin"),
        0,
    )?;
    transform(b, smoke, "flex-libfl-consumer-stdout-assert", consumer_result["stdout"].clone(), "", "equal", &[""])?;
    observation(b, smoke, "flex-version", program.clone(), &["--version"], "", 0, Some("flex-version-observation"))?;
    let positive = emit(b, smoke, "flex-positive-scanner", "%%\nhello  ECHO;\n%%\nint yywrap(void) { return 1; }\n")?;
    let positive = readiness(b, smoke, "flex-smoke-original-readiness", positive, "")?;
    let generated = flex_generate(b, smoke, "flex-positive", positive, "", &["-o", "positive.c"])?;
    let scanner = transform(b, smoke, "flex-positive-project-source", generated, "positive.c", "nonempty", &[])?;
    b.publish(smoke, "flex-positive-scanner-source", scanner)?;
    let negative = emit(b, smoke, "flex-negative-scanner", "%%\n[unterminated  ECHO;\n%%\n")?;
    let result = flex_prepare(b, smoke, "flex-negative", negative, "", &["-o", "negative.c"], 1)?;
    let diagnostic = rejection(b, smoke, "flex-negative", &result, "negative.c", false)?;
    b.publish(smoke, "flex-negative-observation", diagnostic)
}

fn flex_prepare(
    b: &mut TransitionBuilder,
    tool: &str,
    id: &str,
    grammar: ArtifactRef,
    path: &str,
    args: &[&str],
    code: i32,
) -> Result<BTreeMap<String, ArtifactRef>, RunError> {
    let program = b.artifact("flex-2.6.4")?;
    let m4 = b.artifact("m4-1.4.7")?;
    let mut arguments: Vec<_> = args.iter().map(|s| literal(s)).collect();
    arguments.push(input("grammar", path));
    let result = fresh_command(
        b,
        tool,
        &format!("{id}-prepare"),
        program,
        BTreeMap::from([("grammar".into(), grammar), ("m4".into(), m4)]),
        arguments,
        BTreeMap::from([
            ("M4".into(), input("m4", "")),
            ("MANTLE_GRAMMAR_NAME".into(), literal(&grammar_filename(path, args, "l"))),
            ("MANTLE_FLEX_STATE".into(), output("generated", "state")),
        ]),
        None,
        code,
    )?;
    transform(b, tool, &format!("{id}-stdout-assert"), result["stdout"].clone(), "", "equal", &[""])?;
    Ok(result)
}

/// Run original scanner generation and the source-derived sequential tee before
/// standalone M4 expansion. `args` excludes the grammar operand and names the
/// scanner (and optional header) relative to the returned product tree.
pub(super) fn flex_generate(
    b: &mut TransitionBuilder,
    tool: &str,
    id: &str,
    grammar: ArtifactRef,
    grammar_path: &str,
    args: &[&str],
) -> Result<ArtifactRef, RunError> {
    let mut outname = "lex.yy.c";
    let mut header = None;
    for (i, arg) in args.iter().enumerate() {
        if *arg == "-o" || *arg == "--outfile" {
            outname = args.get(i + 1).ok_or_else(|| error("Flex output option lacks value"))?;
        } else if let Some(name) = arg.strip_prefix("--outfile=") {
            outname = name;
        } else if let Some(name) = arg.strip_prefix("-o").filter(|s| !s.is_empty()) {
            outname = name;
        } else if let Some(name) = arg.strip_prefix("--header-file=") {
            header = Some(name);
        }
    }
    crate::source_built_recipe::relative(outname, false)?;
    if let Some(name) = header {
        crate::source_built_recipe::relative(name, false)?;
    }
    let prepared = flex_prepare(b, tool, id, grammar, grammar_path, args, 0)?;
    let program = b.artifact("flex-2.6.4")?;
    let helper = super::late::helper(b, tool)?;
    let raw = command(
        b,
        tool,
        &format!("{id}-raw-stream"),
        helper,
        BTreeMap::from([("prepared".into(), prepared["generated"].clone())]),
        vec![literal("copy"), input("prepared", outname), output("text", "")],
        &[("text", ArtifactKind::Text)],
        None,
        0,
    )?;
    let tee = fresh_command(
        b,
        tool,
        &format!("{id}-tee"),
        program,
        BTreeMap::from([
            ("prepared".into(), prepared["generated"].clone()),
            ("raw".into(), raw["text"].clone()),
        ]),
        vec![literal("--mantle-tee"), input("prepared", "state")],
        BTreeMap::from([("MANTLE_FLEX_HEADER".into(), output("generated", "header.m4"))]),
        Some("raw"),
        0,
    )?;
    let m4 = b.artifact("m4-1.4.7")?;
    let expanded = command(
        b,
        tool,
        &format!("{id}-m4"),
        m4.clone(),
        BTreeMap::from([("script".into(), tee["stdout"].clone())]),
        vec![literal("-P")],
        &[],
        Some("script"),
        0,
    )?;
    let mut products = BTreeMap::from([(outname.to_string(), expanded["stdout"].clone())]);
    if let Some(name) = header {
        let script =
            transform(b, tool, &format!("{id}-header-script"), tee["generated"].clone(), "header.m4", "copy", &[])?;
        let expanded = command(
            b,
            tool,
            &format!("{id}-header-m4"),
            m4,
            BTreeMap::from([("script".into(), script)]),
            vec![literal("-P")],
            &[],
            Some("script"),
            0,
        )?;
        products.insert(name.into(), expanded["stdout"].clone());
    }
    assemble(b, tool, &format!("{id}-products"), products)
}

/// Source-only Flex 2.6.4 transport hooks. Native successor builds apply these
/// to their authenticated original sources, leaving filter_fix_linedirs intact.
/// Only the selected StageX build replaces that algorithm with its owner policy.
pub(crate) fn flex_prepare_phase_source() -> String {
    format!("bool filter_apply_chain (struct filter * chain)\n{{\n{FLEX_CAPTURE_BODY}\n}}")
}

/// Inject this branch at the beginning of the original filter_apply_chain body;
/// without the explicit state output, its original pipe engine remains selected.
pub(crate) fn flex_capture_branch_source() -> String {
    format!("  if (getenv(\"MANTLE_FLEX_STATE\")) {{\n{FLEX_CAPTURE_BODY}\n  }}\n")
}

/// Replace only the original tee's pipe setup region in the selected phase.
pub(crate) fn flex_tee_streams_source() -> &'static str {
    FLEX_TEE_STREAMS
}

/// Append in main.c, compiled with -Dmain=mantle_flex_original_main. The native
/// default branch calls that original main unchanged. MANTLE_GRAMMAR_NAME can
/// restore the original relative grammar spelling from a typed file operand;
/// an already shared workspace inode is never copied over itself.
pub(crate) fn flex_native_main_source() -> String {
    format!("{GRAMMAR_TRANSPORT_C}{FLEX_MAIN_PHASE}")
}

pub(crate) const FLEX_TEE_PIPE_START: &str = "\t/* Store a copy of the stdout pipe,";
pub(crate) const FLEX_TEE_PIPE_END: &str =
    "\t/* Now to_c is a pipe to the C branch, and to_h is a pipe to the H branch.\n\t */";
pub(crate) const FLEX_TEE_WAIT: &str = "\twhile (wait (0) > 0) ;\n\n\tFLEX_EXIT (0);\n\treturn 0;";

// Only pipe ownership changes. The authenticated output.c retains its own
// package-data lookup, installation check and muscle serialization, and the
// resumed executable retains the original skeleton scanner.
const BISON_PIPE_OPEN: &str = r#"  if (!state_path) error (EXIT_FAILURE, 0, "missing source-owned Bison phase state");
  state = xfopen (state_path, "w");
  fprintf (state, "%s\n%s\n%s\n", parser_file_name ? parser_file_name : "",
           spec_defines_file ? spec_defines_file : "", dir_prefix ? dir_prefix : "");
  xfclose (state);
  out = stdout;

  /* Output the definitions of all the muscles.  */
"#;

const BISON_PIPE_CLOSE: &str = r#"  fprintf (out, "m4_include([%s])\n", full_skeleton);
  if (fflush (out) || ferror (out)) error (EXIT_FAILURE, 0, "Bison muscle stream failed");
  free (full_m4sugar);
  free (full_skeleton);
"#;

const BISON_MAIN_PHASE: &str = r#"
#undef main
#include "scan-skel.h"
static void mantle_bison_state_line(FILE *f, char *line, size_t capacity)
{
  size_t n;
  if (!fgets(line, capacity, f)) exit(1);
  n = strlen(line);
  if (!n || line[n-1] != '\n') exit(1);
  line[n-1] = 0;
}
int main(int argc, char **argv)
{
  FILE *state; char parser[4096], header[4096], directory[4096];
  if (argc != 3 || strcmp(argv[1], "--mantle-resume")) {
    mantle_source_grammar(argc, argv);
    return mantle_bison_original_main(argc, argv);
  }
  program_name = argv[0];
  state = xfopen(argv[2], "r");
  mantle_bison_state_line(state, parser, sizeof parser);
  mantle_bison_state_line(state, header, sizeof header);
  mantle_bison_state_line(state, directory, sizeof directory);
  parser_file_name = parser; spec_defines_file = header; dir_prefix = directory;
  if (fgetc(state) != EOF) exit(1);
  xfclose(state);
  scan_skel(stdin);
  return complaint_issued ? EXIT_FAILURE : EXIT_SUCCESS;
}
"#;

const FLEX_CAPTURE_BODY: &str = r#"
  const char *path = getenv("MANTLE_FLEX_STATE");
  FILE *state;
  if (!chain) return true;
  if (!path) flexfatal("missing source-owned Flex phase state");
  state = fopen(path, "w");
  if (!state) flexfatal("cannot write Flex phase state");
  if (fprintf(state, "%s\n%s\n%s\n%d\n", prefix ? prefix : "yy", outfilename ? outfilename : "",
      headerfilename ? headerfilename : "", gen_line_dirs) < 0 || fclose(state))
    flexfatal("cannot flush Flex phase state");
  return true;
"#;

const FLEX_TEE_STREAMS: &str = r#"	to_c = stdout;
	if (write_header) {
        const char *path = getenv("MANTLE_FLEX_HEADER");
        if (!path || !(to_h = fopen(path, "w"))) flexfatal("cannot open Flex header phase stream");
	}"#;

const FLEX_MAIN_PHASE: &str = r#"
#undef main
static void mantle_flex_state_line(FILE *f, char *line, size_t capacity)
{
  size_t n;
  if (!fgets(line, capacity, f)) exit(1);
  n = strlen(line);
  if (!n || line[n-1] != '\n') exit(1);
  line[n-1] = 0;
}
int main(int argc, char **argv)
{
  FILE *state; struct filter chain; int status;
  char pbuf[4096], obuf[4096], hbuf[4096], flag[32];
  const char *cwd = getenv("MANTLE_PHASE_CWD");
  int linedirs = argc == 3 && !strcmp(argv[1], "--mantle-linedirs");
  if (cwd && getenv("MANTLE_FLEX_STATE")) {
    size_t length = strlen(cwd); int i;
    if (chdir(cwd)) return 1;
    /* Typed workspace paths resolve to these same files from the original
       working directory; retain the owner's relative #line spelling. */
    for (i = 1; i < argc; i++)
      if (!strncmp(argv[i], cwd, length) && argv[i][length] == '/')
        argv[i] += length + 1;
  }
  if (!linedirs && (argc != 3 || strcmp(argv[1], "--mantle-tee"))) {
    mantle_source_grammar(argc, argv);
    return mantle_flex_original_main(argc, argv);
  }
  status = setjmp(flex_main_jmp_buf);
  if (status) return status - 1;
  state = fopen(argv[2], "r"); if (!state) return 1;
  mantle_flex_state_line(state, pbuf, sizeof pbuf);
  mantle_flex_state_line(state, obuf, sizeof obuf);
  mantle_flex_state_line(state, hbuf, sizeof hbuf);
  mantle_flex_state_line(state, flag, sizeof flag); gen_line_dirs = atoi(flag);
  prefix = pbuf; outfilename = obuf; headerfilename = hbuf;
  if (fgetc(state) != EOF || fclose(state)) return 1;
  memset(&chain, 0, sizeof chain);
  chain.extra = *headerfilename ? headerfilename : NULL;
  if (linedirs) {
    flex_init_regex();
    return filter_fix_linedirs(&chain);
  }
  if (setenv("MANTLE_FLEX_TEE", "1", 1)) return 1;
  return filter_tee_header(&chain);
}
"#;

const GRAMMAR_TRANSPORT_C: &str = r#"
#include <sys/stat.h>
static void mantle_source_grammar(int argc, char **argv)
{
  const char *name = getenv("MANTLE_GRAMMAR_NAME");
  FILE *in, *out; char block[8192]; size_t n, total = 0;
  struct stat source_stat, target_stat;
  if (!name) return;
  if (argc < 2) exit(1);
  in = fopen(argv[argc-1], "rb"); if (!in) exit(1);
  if (fstat(fileno(in), &source_stat)) exit(1);
  /* A typed workspace operand may already name the original relative file.
     Preserve its inode and bytes while restoring the original argv spelling. */
  if (!stat(name, &target_stat) && source_stat.st_dev == target_stat.st_dev
      && source_stat.st_ino == target_stat.st_ino) {
    if (fclose(in)) exit(1);
    argv[argc-1] = (char *)name;
    return;
  }
  out = fopen(name, "wb"); if (!out) exit(1);
  while ((n = fread(block, 1, sizeof block, in))) {
    total += n;
    if (total > 8388608 || fwrite(block, 1, n, out) != n) exit(1);
  }
  if (ferror(in) || fclose(in) || fclose(out)) exit(1);
  argv[argc-1] = (char *)name;
}
"#;

const EDITOR_C: &str = r#"#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <stdint.h>
#include <sys/stat.h>
#include <errno.h>
#include <dirent.h>
#include <unistd.h>
#include <limits.h>
static void die(const char *s) { fprintf(stderr,"parser source policy: %s\n",s); exit(1); }
static char *load(const char *path,size_t *n) { FILE *f=fopen(path,"rb");long z;char *p;if(!f||fseek(f,0,SEEK_END)||(z=ftell(f))<0||z>67108864||fseek(f,0,SEEK_SET))die("input bound");p=malloc((size_t)z+1);if(!p||fread(p,1,(size_t)z,f)!=(size_t)z||fclose(f))die("read");p[z]=0;*n=(size_t)z;return p; }
static char *unique(char *p,const char *s) { char *q;if(!*s||!(q=strstr(p,s))||strstr(q+strlen(s),s))die("source anchor count");return q; }
static char *function_end(char *p) { unsigned depth=0;int mode=0,escape=0;for(;*p;p++){if(mode==3){if(*p=='\n')mode=0;continue;}if(mode==4){if(*p=='*'&&p[1]=='/'){p++;mode=0;}continue;}if(escape){escape=0;continue;}if(mode){if(*p=='\\')escape=1;else if((mode==1&&*p=='\"')||(mode==2&&*p=='\''))mode=0;continue;}if(*p=='/'&&p[1]=='/'){p++;mode=3;}else if(*p=='/'&&p[1]=='*'){p++;mode=4;}else if(*p=='\"')mode=1;else if(*p=='\'')mode=2;else if(*p=='{')depth++;else if(*p=='}'){if(!depth)die("brace depth");if(!--depth)return p+1;}}die("function end");return 0; }
static void bytes(FILE *f,const char *p,size_t n) { if(fwrite(p,1,n,f)!=n)die("write"); }
static uint64_t integer(const char *b,size_t n,size_t at,unsigned width) { uint64_t v=0;unsigned i;if(at>n||width>n-at)die("ELF integer bound");for(i=0;i<width;i++)v|=(uint64_t)(unsigned char)b[at+i]<<(i*8);return v; }
static void put_integer(char *b,size_t at,uint64_t value,unsigned width) { unsigned i;for(i=0;i<width;i++)b[at+i]=(char)(value>>(i*8)); }
static size_t ready_entries,ready_bytes;
static void ready_tree(const char *path,unsigned depth) {
 struct stat st;DIR *d;struct dirent *e;char child[PATH_MAX];FILE *f;
 if(depth>64||lstat(path,&st)||++ready_entries>32768)die("predecessor tree bound");
 if(S_ISDIR(st.st_mode)) { d=opendir(path);if(!d)die("predecessor directory");while((e=readdir(d))){if(!strcmp(e->d_name,".")||!strcmp(e->d_name,".."))continue;if(snprintf(child,sizeof child,"%s/%s",path,e->d_name)>=(int)sizeof child)die("predecessor path bound");ready_tree(child,depth+1);}if(closedir(d))die("predecessor directory close"); }
 else if(S_ISLNK(st.st_mode)) { ssize_t z=readlink(path,child,sizeof child);if(z<=0||z==(ssize_t)sizeof child)die("predecessor symlink"); }
 else if(S_ISREG(st.st_mode)) { if(st.st_size<0||(uint64_t)st.st_size>67108864||ready_bytes>1073741824-(size_t)st.st_size)die("predecessor source bound");ready_bytes+=(size_t)st.st_size;f=fopen(path,"rb");if(!f)die("predecessor source readability");if(st.st_size&&fgetc(f)==EOF)die("predecessor source truncation");if(fclose(f))die("predecessor source close"); }
 else die("predecessor source type");
}
static void ready_product(const char *kind,const char *path) {
 size_t n;char *b;struct stat st;
 if(!strcmp(kind,"tree")||!strcmp(kind,"source")) { if(lstat(path,&st))die("missing original prerequisite");if(S_ISDIR(st.st_mode)){ready_entries=ready_bytes=0;ready_tree(path,0);if(ready_entries<2)die("empty predecessor tree");return;}if(!strcmp(kind,"tree"))die("predecessor is not a tree"); }
 b=load(path,&n);
 if(!strcmp(kind,"empty")) { if(n)die("original empty observation changed"); }
 else if(!strcmp(kind,"elf")||!strcmp(kind,"object")) { uint64_t type=!strcmp(kind,"object")?1:2,machine;if(n<20||memcmp(b,"\177ELF",4)||(b[4]!=1&&b[4]!=2)||b[5]!=1||integer(b,n,16,2)!=type)die("original ELF prerequisite format");machine=integer(b,n,18,2);if(machine!=3&&machine!=62)die("original ELF prerequisite architecture"); }
 else if(!strcmp(kind,"archive")) { size_t at=8;if(n<8||memcmp(b,"!<arch>\n",8))die("original archive prerequisite format");while(at<n){size_t i,z=0;int seen=0;if(n-at<60||memcmp(b+at+58,"`\n",2))die("original archive member header");for(i=0;i<10;i++){char c=b[at+48+i];if(c==' '){if(seen)break;continue;}if(c<'0'||c>'9')die("original archive member size");seen=1;z=z*10+(c-'0');if(z>67108864)die("original archive member bound");}if(!seen||z>n-at-60)die("original archive member range");at+=60+z;if(z%2){if(at==n||b[at]!='\n')die("original archive member alignment");at++;}} }
 else if(!n)die("empty original prerequisite");
 free(b);
}
static unsigned hex(char c) { if(c>='0'&&c<='9')return c-'0';if(c>='a'&&c<='f')return c-'a'+10;die("runtime hex policy");return 0; }
static void canonical_flex(char *b,size_t n,const char *policy) {
 size_t table,count,index,nh,no,ns,i,found=0,start=0,len=0;unsigned char runtime[49];
 if(n<64||memcmp(b,"\177ELF",4)||b[4]!=2||b[5]!=1||integer(b,n,16,2)!=2||integer(b,n,18,2)!=62)die("Flex ELF header");
 if(strlen(policy)!=98)die("Flex runtime policy length");for(i=0;i<49;i++)runtime[i]=(hex(policy[i*2])<<4)|hex(policy[i*2+1]);
 table=integer(b,n,40,8);count=integer(b,n,60,2);index=integer(b,n,62,2);
 if(integer(b,n,58,2)!=64||!count||index>=count||table>n||count>(n-table)/64)die("Flex ELF section table");
 nh=table+index*64;if(integer(b,n,nh+4,4)!=3)die("Flex ELF names type");no=integer(b,n,nh+24,8);ns=integer(b,n,nh+32,8);
 if(!ns||no>table||ns>table-no||b[no])die("Flex ELF name range");
 for(i=0;i<count;i++) { size_t s=table+i*64,off=integer(b,n,s,4),data=integer(b,n,s+24,8),size=integer(b,n,s+32,8);char *end;
  if(off>=ns||(off&&b[no+off-1])||!(end=memchr(b+no+off,0,ns-off)))die("Flex ELF section name");
  if(integer(b,n,s+4,4)==1&&integer(b,n,s+8,8)==2&&integer(b,n,s+48,8)==8&&size==49&&data<=n&&size<=n-data&&!memcmp(b+data,runtime,49)) {
   found++;start=no+off;len=(size_t)(end-(b+start));if(len<3||len>4)die("Flex runtime section name length");
  }
 }
 if(found!=1)die("Flex runtime section count");memcpy(b+start,"stx",3);b[start+3]=0;
 if(len>3) { size_t removed=len-3,oldend=no+ns,newend=oldend-removed,cut=start+3-no;
  memmove(b+start+4,b+start+len+1,oldend-(start+len+1));memset(b+newend,0,removed);
  for(i=0;i<count;i++) { size_t s=table+i*64,off=integer(b,n,s,4);if(off>cut)put_integer(b,s,off-removed,4); }
  put_integer(b,nh+32,ns-removed,8);
 }
}
int main(int c,char **v) {
 size_t n;char *b,*p,*q;FILE *f;
 if(c<4)die("arguments");b=load(v[2],&n);f=fopen(v[3],"wb");if(!f)die("output");
 if(c==4&&!strcmp(v[1],"archive")) { char header[61];int z;if(!n)die("empty archive member");bytes(f,"!<arch>\n",8);z=snprintf(header,sizeof header,"%-16s%-12s%-6s%-6s%-8s%-10lu`\n","libfl_main.o/","0","0","0","100644",(unsigned long)n);if(z!=60)die("archive header");bytes(f,header,60);bytes(f,b,n);if(n%2)bytes(f,"\n",1); }
 else if(c==6&&!strcmp(v[1],"function")) { p=unique(b,v[4]);q=strchr(p,'{');if(!q)die("function opening");q=function_end(q);bytes(f,b,(size_t)(p-b));bytes(f,v[5],strlen(v[5]));bytes(f,q,n-(size_t)(q-b)); }
 else if(c==7&&!strcmp(v[1],"region")) { p=unique(b,v[4]);q=strstr(p,v[5]);if(!q)die("region end");q+=strlen(v[5]);bytes(f,b,(size_t)(p-b));bytes(f,v[6],strlen(v[6]));bytes(f,q,n-(size_t)(q-b)); }
 else if(c==6&&!strcmp(v[1],"readiness")) { size_t z;char *ready=load(v[4],&z);if(!strstr(ready,v[5]))die("required GNU M4 provider unavailable");free(ready);bytes(f,b,n); }
 else if(c==7&&!strcmp(v[1],"rejection")) { size_t z;char *out=load(v[4],&z);struct stat st;if(!n||z)die("negative generator stream contract");free(out);if(!stat(v[5],&st)){if(!strcmp(v[6],"absent")||st.st_size)die("negative generator retained source");}else if(errno!=ENOENT)die("negative generator output stat");bytes(f,b,n); }
 else if(c==5&&!strcmp(v[1],"flex-elf")) { canonical_flex(b,n,v[4]);bytes(f,b,n);if(chmod(v[3],0755))die("Flex executable mode"); }
 else if(c>=4&&!(c%2)&&!strcmp(v[1],"gate")) { int i;for(i=4;i<c;i+=2)ready_product(v[i],v[i+1]);bytes(f,b,n); }
 else die("operation");
 free(b);if(fclose(f))die("flush");return 0;
}
"#;
