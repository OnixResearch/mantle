//! Child-free lowering of the original binutils preparation and native helpers.
use std::collections::BTreeMap;
use std::collections::BTreeSet;

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
use crate::source_built_stagex_programs::error;
use crate::source_built_stagex_programs::input;
use crate::source_built_stagex_programs::literal;
use crate::source_built_stagex_programs::output;
use crate::stagex_transition::BINUTILS_CANONICALIZER_BUILD_STAGE_ID as CANONICALIZER;
use crate::stagex_transition::BINUTILS_GENERATOR_BUILD_STAGE_ID as GENERATORS;
use crate::stagex_transition::BINUTILS_HELPER_BUILD_STAGE_ID as HELPERS;

#[path = "source_built_stagex_programs_binutils_writer.rs"]
mod writer;
use writer::ld_writer_variant;
#[path = "source_built_stagex_programs_binutils_dispatch.rs"]
mod dispatch;
#[path = "source_built_stagex_programs_binutils_runtime.rs"]
mod runtime;

type Files = BTreeMap<String, ArtifactRef>;
const FILE_BYTES_MAX: u64 = 64 * 1024 * 1024;
const TREE_BYTES_MAX: u64 = 1024 * 1024 * 1024;
const SOURCE: &str = "binutils-2.30-source";
const RECIPE: &str = "binutils-2.30-recipe-source";

struct Author<'a> {
    builder: &'a mut TransitionBuilder,
    // Pending wiring in the source-only preparation path (Cairn prove-source-built-mantle-fixed-point I3).
    #[allow(dead_code)]
    ready: BTreeSet<String>,
}

impl Author<'_> {
    fn add(&mut self, tool: &str, step: RecipeStep) -> Result<Files, RunError> {
        self.builder.add(tool, step)
    }

    fn exec(
        &mut self,
        tool: &str,
        id: &str,
        executable: ArtifactRef,
        mut inputs: Files,
        arguments: Vec<RecipeWord>,
        kind: ArtifactKind,
    ) -> Result<ArtifactRef, RunError> {
        inputs.insert("executable".into(), executable);
        let outputs = self.add(tool, RecipeStep {
            id: id.into(),
            inputs,
            outputs: BTreeMap::from([
                ("result".into(), spec(kind)),
                ("stdout".into(), spec(ArtifactKind::Text)),
                ("stderr".into(), spec(ArtifactKind::Text)),
                ("exit-status".into(), RecipeOutput {
                    kind: ArtifactKind::Text,
                    bytes_max: 4,
                }),
            ]),
            operation: RecipeOperation::Exec {
                invocation_input: None,
                command_state: None,
                merge_stderr: false,
                executable_input: "executable".into(),
                arguments,
                environment: BTreeMap::new(),
                runtime_mounts: BTreeMap::new(),
                cwd_input: None,
                cwd_output: None,
                output_seeds: BTreeMap::new(),
                stdin_input: None,
                guard: None,
                exit_policy: RecipeExitPolicy::Exact { code: 0 },
            },
        })?;
        Ok(outputs["result"].clone())
    }

    fn capture(
        &mut self,
        tool: &str,
        id: &str,
        executable: ArtifactRef,
        mut inputs: Files,
        arguments: Vec<RecipeWord>,
        stdin: Option<ArtifactRef>,
    ) -> Result<ArtifactRef, RunError> {
        inputs.insert("executable".into(), executable);
        let stdin_input = stdin.map(|reference| {
            inputs.insert("stdin".into(), reference);
            "stdin".into()
        });
        Ok(self.add(tool, RecipeStep {
            id: id.into(),
            inputs,
            outputs: BTreeMap::from([
                ("stdout".into(), spec(ArtifactKind::Text)),
                ("stderr".into(), spec(ArtifactKind::Text)),
                ("exit-status".into(), RecipeOutput {
                    kind: ArtifactKind::Text,
                    bytes_max: 4,
                }),
            ]),
            operation: RecipeOperation::Exec {
                invocation_input: None,
                command_state: None,
                merge_stderr: false,
                executable_input: "executable".into(),
                arguments,
                environment: BTreeMap::new(),
                runtime_mounts: BTreeMap::new(),
                cwd_input: None,
                cwd_output: None,
                output_seeds: BTreeMap::new(),
                stdin_input,
                guard: None,
                exit_policy: RecipeExitPolicy::Exact { code: 0 },
            },
        })?["stdout"]
            .clone())
    }

    fn text(&mut self, tool: &str, id: &str, source: ArtifactRef, text: String) -> Result<ArtifactRef, RunError> {
        Ok(self.add(tool, RecipeStep {
            id: id.into(),
            inputs: BTreeMap::from([("recipe-source".into(), source)]),
            outputs: BTreeMap::from([("result".into(), spec(ArtifactKind::Text))]),
            operation: RecipeOperation::WriteText {
                output: "result".into(),
                text,
            },
        })?["result"]
            .clone())
    }

    fn concat(
        &mut self,
        tool: &str,
        id: &str,
        executable: &ArtifactRef,
        parts: Vec<ArtifactRef>,
    ) -> Result<ArtifactRef, RunError> {
        let mut inputs = Files::new();
        let mut arguments = vec![literal("concat"), output("result", "")];
        for (index, reference) in parts.into_iter().enumerate() {
            let key = format!("part-{index}");
            inputs.insert(key.clone(), reference);
            arguments.push(input(&key, ""));
        }
        self.exec(tool, id, executable.clone(), inputs, arguments, ArtifactKind::Text)
    }

    fn tree(&mut self, tool: &str, id: &str, files: Files) -> Result<ArtifactRef, RunError> {
        let mut inputs = Files::new();
        let mut entries = BTreeMap::new();
        for (index, (path, reference)) in files.into_iter().enumerate() {
            let name = format!("file-{index}");
            inputs.insert(name.clone(), reference);
            entries.insert(path, name);
        }
        Ok(self.add(tool, RecipeStep {
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

    // Pending wiring in the source-only preparation path (Cairn prove-source-built-mantle-fixed-point
    // I3).
    #[allow(dead_code)]
    fn workspace(
        &mut self,
        tool: &str,
        id: &str,
        executable: ArtifactRef,
        mut inputs: Files,
        arguments: Vec<RecipeWord>,
        stdin: Option<ArtifactRef>,
        code: i32,
    ) -> Result<Files, RunError> {
        inputs.insert("executable".into(), executable);
        let stdin_input = stdin.map(|reference| {
            inputs.insert("stdin".into(), reference);
            "stdin".into()
        });
        self.add(tool, RecipeStep {
            id: id.into(),
            inputs,
            outputs: BTreeMap::from([
                ("workspace".into(), spec(ArtifactKind::Tree)),
                ("stdout".into(), spec(ArtifactKind::Text)),
                ("stderr".into(), spec(ArtifactKind::Text)),
                ("exit-status".into(), spec(ArtifactKind::Text)),
            ]),
            operation: RecipeOperation::Exec {
                invocation_input: None,
                command_state: None,
                merge_stderr: false,
                executable_input: "executable".into(),
                arguments,
                environment: BTreeMap::new(),
                runtime_mounts: BTreeMap::new(),
                cwd_input: None,
                cwd_output: Some("workspace".into()),
                output_seeds: BTreeMap::new(),
                stdin_input,
                guard: None,
                exit_policy: RecipeExitPolicy::Exact { code },
            },
        })
    }

    // Pending wiring in the source-only preparation path (Cairn prove-source-built-mantle-fixed-point
    // I3).
    #[allow(dead_code)]
    fn observe(
        &mut self,
        tool: &str,
        id: &str,
        executable: ArtifactRef,
        mut inputs: Files,
        arguments: Vec<RecipeWord>,
        code: i32,
    ) -> Result<Files, RunError> {
        inputs.insert("executable".into(), executable);
        self.add(tool, RecipeStep {
            id: id.into(),
            inputs,
            outputs: BTreeMap::from([
                ("stdout".into(), spec(ArtifactKind::Text)),
                ("stderr".into(), spec(ArtifactKind::Text)),
                ("exit-status".into(), spec(ArtifactKind::Text)),
            ]),
            operation: RecipeOperation::Exec {
                invocation_input: None,
                command_state: None,
                merge_stderr: false,
                executable_input: "executable".into(),
                arguments,
                environment: BTreeMap::new(),
                runtime_mounts: BTreeMap::new(),
                cwd_input: None,
                cwd_output: None,
                output_seeds: BTreeMap::new(),
                stdin_input: None,
                guard: None,
                exit_policy: RecipeExitPolicy::Exact { code },
            },
        })
    }

    fn compile(&mut self, tool: &str, id: &str, source: ArtifactRef, path: &str) -> Result<ArtifactRef, RunError> {
        let inputs = BTreeMap::from([
            ("source".into(), source),
            ("headers".into(), self.builder.artifact("musl-native-headers")?),
        ]);
        self.exec(
            tool,
            id,
            self.builder.artifact("tcc-musl-v2")?,
            inputs,
            vec![
                literal("-c"),
                flag("-I", input("headers", "")),
                input("source", path),
                literal("-o"),
                output("result", ""),
            ],
            ArtifactKind::Object,
        )
    }

    fn link(&mut self, tool: &str, id: &str, objects: &[ArtifactRef], strip: bool) -> Result<ArtifactRef, RunError> {
        let mut inputs = BTreeMap::from([
            ("crt1".into(), self.builder.artifact("musl-native-crt1")?),
            ("libc".into(), self.builder.artifact("musl-native-libc")?),
        ]);
        let mut arguments = vec![literal("-nostdlib"), literal("-static")];
        if strip {
            arguments.push(literal("-s"));
        }
        arguments.push(input("crt1", ""));
        for (index, reference) in objects.iter().enumerate() {
            let name = format!("object-{index}");
            inputs.insert(name.clone(), reference.clone());
            arguments.push(input(&name, ""));
        }
        arguments.extend([input("libc", ""), literal("-o"), output("result", "")]);
        self.exec(tool, id, self.builder.artifact("tcc-musl-v2")?, inputs, arguments, ArtifactKind::Executable)
    }

    fn canonicalize(
        &mut self,
        tool: &str,
        id: &str,
        raw: ArtifactRef,
        adapter: &ArtifactRef,
        kind: ArtifactKind,
    ) -> Result<ArtifactRef, RunError> {
        self.exec(
            tool,
            id,
            adapter.clone(),
            BTreeMap::from([("raw".into(), raw)]),
            vec![input("raw", ""), output("result", "")],
            kind,
        )
    }
}

fn spec(kind: ArtifactKind) -> RecipeOutput {
    RecipeOutput {
        kind,
        bytes_max: if kind == ArtifactKind::Tree {
            TREE_BYTES_MAX
        } else {
            FILE_BYTES_MAX
        },
    }
}

fn flag(prefix: &str, mut word: RecipeWord) -> RecipeWord {
    word.parts.insert(0, WordPart::Literal { text: prefix.into() });
    word
}

struct NativeHelpers {
    transform: ArtifactRef,
    runtime: ArtifactRef,
    assembly: ArtifactRef,
    canonicalizer_copy: ArtifactRef,
}

fn native_helpers(author: &mut Author<'_>) -> Result<NativeHelpers, RunError> {
    let recipe = author.builder.read_source(RECIPE, "")?;
    let runtime_source =
        recipe_block(&recipe, "      $BB cat > \"$WORK/binutils-runtime.c\" <<'RUNTIME_EOF'\n", "\nRUNTIME_EOF")?;
    let assembly_source = recipe_block(
        &recipe,
        "      $BB cat > \"$WORK/binutils-runtime.s\" <<'RUNTIME_ASM_EOF'\n",
        "\nRUNTIME_ASM_EOF",
    )?;
    let runtime_source = author.builder.authored("helpers/binutils-runtime.c", runtime_source)?;
    let assembly_source = author.builder.authored("helpers/binutils-runtime.s", assembly_source)?;
    let runtime = author.compile(CANONICALIZER, "binutils-runtime-compile", runtime_source, "")?;
    let assembly = author.compile(CANONICALIZER, "binutils-runtime-assembly-compile", assembly_source, "")?;
    author.builder.artifacts.insert("binutils-runtime-object".into(), runtime.clone());
    author.builder.artifacts.insert("binutils-runtime-assembly-object".into(), assembly.clone());
    let canonicalizer_source = author.builder.source("stagex-elf-local-symbol-canonicalizer-source")?;
    let canonicalizer_object =
        author.compile(CANONICALIZER, "binutils-canonicalizer-compile", canonicalizer_source.clone(), "")?;
    let canonicalizer = author.link(
        CANONICALIZER,
        "binutils-canonicalizer-link",
        &[canonicalizer_object, runtime.clone(), assembly.clone()],
        true,
    )?;
    author.builder.publish(CANONICALIZER, "binutils-elf-symbol-canonicalizer", canonicalizer.clone())?;
    // Canonicalization changes the actual compiled image bytes. Its completed
    // original rewrite, not the earlier compiler snapshot, owns the final
    // executable product consumed by a later original ./conftest request.
    let controlled_object = author.exec(
        CANONICALIZER,
        "binutils-canonicalizer-command-compile",
        author.builder.artifact("tcc-musl-v2")?,
        BTreeMap::from([
            ("source".into(), canonicalizer_source.clone()),
            ("headers".into(), author.builder.artifact("musl-native-headers")?),
        ]),
        vec![
            literal("-c"),
            literal("-Dmain=mantle_original_main"),
            literal("-I"),
            input("headers", ""),
            input("source", ""),
            literal("-o"),
            output("result", ""),
        ],
        ArtifactKind::Object,
    )?;
    let entry_source = author.builder.authored("helpers/binutils-canonicalizer-command-entry.c",
        "extern int mantle_original_main(int,char**);\nextern void mantle_native_generated_created(const char*);\nint main(int argc,char **argv){int status=mantle_original_main(argc,argv);if(!status&&argc==2)mantle_native_generated_created(argv[1]);return status;}\n".into())?;
    let entry = author.compile(CANONICALIZER, "binutils-canonicalizer-command-entry-compile", entry_source, "")?;
    let publisher_source = author.builder.authored(
        "helpers/binutils-canonicalizer-products.c",
        include_str!("source_built_native_programs_generated.c").into(),
    )?;
    let publisher = author.compile(CANONICALIZER, "binutils-canonicalizer-products-compile", publisher_source, "")?;
    let controlled = author.link(
        CANONICALIZER,
        "binutils-canonicalizer-command-link",
        &[controlled_object, entry, publisher, runtime.clone(), assembly.clone()],
        false,
    )?;
    author.builder.artifacts.insert("phase-tool:binutils-elf-symbol-canonicalizer".into(), controlled);

    // Keep the original in-place canonicalizer unchanged. Only this adapter owns
    // the writable copy; recipe inputs remain immutable and no child is run.
    let adapter_source = author.builder.authored("helpers/binutils-canonicalizer-copy.c", CANONICALIZER_COPY.into())?;
    let adapter_tree = author.tree(
        CANONICALIZER,
        "binutils-canonicalizer-copy-source",
        BTreeMap::from([
            ("canonicalizer.c".into(), canonicalizer_source),
            ("copy.c".into(), adapter_source),
        ]),
    )?;
    let adapter_object =
        author.compile(CANONICALIZER, "binutils-canonicalizer-copy-compile", adapter_tree, "copy.c")?;
    let canonicalizer_copy = author.link(
        CANONICALIZER,
        "binutils-canonicalizer-copy-link",
        &[adapter_object, runtime.clone(), assembly.clone()],
        false,
    )?;
    author
        .builder
        .artifacts
        .insert("elf-local-symbol-canonicalizer-copy".into(), canonicalizer_copy.clone());

    let transform_source = author.builder.helper_source();
    let transform_object = author.compile(CANONICALIZER, "binutils-transform-compile", transform_source, "")?;
    let transform = author.link(
        CANONICALIZER,
        "binutils-transform-link",
        &[transform_object, runtime.clone(), assembly.clone()],
        false,
    )?;
    author.builder.artifacts.insert("binutils-source-transform".into(), transform.clone());

    let semaphore_source = author.builder.source("stagex-single-thread-semaphore-compat-source")?;
    let semaphore = author.compile(HELPERS, "binutils-semaphore-compile", semaphore_source, "")?;
    for (name, source_id, artifact_id) in [
        ("sed-bridge", "stagex-sed-bridge-launcher-source", "binutils-sed-bridge-launcher"),
        ("configure-utility", "stagex-configure-utility-source", "binutils-configure-utility"),
        ("ylwrap", "stagex-ylwrap-sed-runner-source", "binutils-ylwrap-runner"),
    ] {
        let source = author.builder.source(source_id)?;
        let object = author.compile(HELPERS, &format!("binutils-{name}-compile"), source, "")?;
        let raw = author.link(
            HELPERS,
            &format!("binutils-{name}-link"),
            &[object, semaphore.clone(), runtime.clone(), assembly.clone()],
            false,
        )?;
        let canonical = author.canonicalize(
            HELPERS,
            &format!("binutils-{name}-canonicalize"),
            raw,
            &canonicalizer_copy,
            ArtifactKind::Executable,
        )?;
        author.builder.publish(HELPERS, artifact_id, canonical)?;
    }
    Ok(NativeHelpers {
        transform,
        runtime,
        assembly,
        canonicalizer_copy,
    })
}

fn recipe_block(recipe: &str, start: &str, end: &str) -> Result<String, RunError> {
    if recipe.matches(start).count() != 1 {
        return Err(error("original binutils recipe source block opener drift"));
    }
    let (_, after) = recipe.split_once(start).ok_or_else(|| error("missing original binutils source block"))?;
    let (block, _) = after.split_once(end).ok_or_else(|| error("unterminated original binutils source block"))?;
    Ok(format!("{block}\n"))
}

fn normalize_source(author: &mut Author<'_>, native: &NativeHelpers) -> Result<Files, RunError> {
    let mut files = Files::new();
    for path in author.builder.members(SOURCE)? {
        files.insert(path.clone(), author.builder.member(SOURCE, &path)?);
    }
    for path in crate::stagex_binutils::source_owned_generated_removals() {
        if files.remove(*path).is_none() {
            return Err(error(format!("original binutils generated-removal inventory lacks {path}")));
        }
    }
    for (index, (path, old, new, count)) in
        crate::stagex_binutils::source_owned_normalization_rewrites().into_iter().enumerate()
    {
        let original = files
            .get(path)
            .cloned()
            .ok_or_else(|| error(format!("missing original binutils normalization source {path}")))?;
        if path == "bfd/configure" && old == "bfd-in3.h:bfd-in2.h" {
            author.builder.artifacts.insert("binutils-bfd-configure-authenticated".into(), original.clone());
        }
        let rewritten = author.exec(
            GENERATORS,
            &format!("binutils-normalize-{index:03}"),
            native.transform.clone(),
            BTreeMap::from([("source".into(), original)]),
            vec![
                literal("replace"),
                input("source", ""),
                output("result", ""),
                literal(old),
                literal(new),
                literal(&count.to_string()),
            ],
            ArtifactKind::Text,
        )?;
        files.insert(path.into(), rewritten);
    }
    let tree = author.tree(GENERATORS, "binutils-normalized-source-tree", files.clone())?;
    author.builder.artifacts.insert("binutils-normalized-source".into(), tree);
    Ok(files)
}

fn make_variable(source: &str, name: &str) -> Result<Vec<String>, RunError> {
    let joined = source.replace("\\\n", " ");
    let prefix = format!("{name} =");
    let mut matches = joined.lines().filter_map(|line| line.strip_prefix(&prefix));
    let value = matches.next().ok_or_else(|| error(format!("missing original Makefile variable {name}")))?;
    if matches.next().is_some() {
        return Err(error(format!("ambiguous original Makefile variable {name}")));
    }
    Ok(value.split_whitespace().map(str::to_owned).collect())
}

fn bfd_headers(author: &mut Author<'_>, native: &NativeHelpers, sources: &mut Files) -> Result<(), RunError> {
    let source_tree = author.builder.fetcher(GENERATORS, SOURCE)?;
    let headers = author.builder.artifact("musl-native-headers")?;
    let chew_object = author.exec(
        GENERATORS,
        "binutils-bfd-chew-compile",
        author.builder.artifact("tcc-musl-v2")?,
        BTreeMap::from([("source".into(), source_tree), ("headers".into(), headers)]),
        vec![
            literal("-c"),
            flag("-I", input("headers", "")),
            flag("-I", input("source", "include")),
            literal("-static"),
            literal("-D_GNU_SOURCE"),
            literal("-DBUILDFIXED=1"),
            literal("-DDYNAMIC_CRC_TABLE=1"),
            input("source", "bfd/doc/chew.c"),
            literal("-o"),
            output("result", ""),
        ],
        ArtifactKind::Object,
    )?;
    let chew_raw = author.link(
        GENERATORS,
        "binutils-bfd-chew-link",
        &[chew_object, native.runtime.clone(), native.assembly.clone()],
        false,
    )?;
    let chew = author.canonicalize(
        GENERATORS,
        "binutils-bfd-chew-canonicalize",
        chew_raw,
        &native.canonicalizer_copy,
        ArtifactKind::Executable,
    )?;
    author.builder.publish(GENERATORS, "binutils-bfd-chew", chew.clone())?;
    let makefile = author.builder.read_source(SOURCE, "bfd/doc/Makefile.am")?;
    let makefile_ref = author.builder.member(SOURCE, "bfd/doc/Makefile.am")?;
    for (name, path, internal) in [
        ("BFD_H_DEP", "bfd/bfd-in2.h", false),
        ("LIBBFD_H_DEP", "bfd/libbfd.h", true),
        ("LIBCOFF_H_DEP", "bfd/libcoff.h", true),
    ] {
        let dependencies = make_variable(&makefile, name)?;
        let dependency_echo = dependencies
            .iter()
            .map(|word| word.replace("$(srcdir)", ".").replace("$(MKDOC)", "chew"))
            .collect::<Vec<_>>()
            .join(" ");
        let dependency_line = author.text(
            GENERATORS,
            &format!("binutils-{name}-dependency-echo"),
            makefile_ref.clone(),
            format!("{dependency_echo}\n"),
        )?;
        let header_sed = author.builder.member(SOURCE, "bfd/doc/header.sed")?;
        let prologue = author.capture(
            GENERATORS,
            &format!("binutils-{name}-header-sed"),
            author.builder.artifact("sed-4.0.9")?,
            BTreeMap::from([("program".into(), header_sed)]),
            vec![literal("-f"), input("program", "")],
            Some(dependency_line),
        )?;
        let mut parts = vec![prologue];
        let mut reached_header_sed = false;
        for (index, dependency) in dependencies.iter().enumerate() {
            if dependency.ends_with("/header.sed") {
                reached_header_sed = true;
                break;
            }
            let basename = dependency.rsplit('/').next().ok_or_else(|| error("empty BFD header source dependency"))?;
            let input_path = format!("bfd/{basename}");
            let source = sources
                .get(&input_path)
                .cloned()
                .ok_or_else(|| error(format!("missing BFD header input {input_path}")))?;
            if dependency.ends_with("-in.h") {
                parts.push(source);
                continue;
            }
            parts.push(author.text(
                GENERATORS,
                &format!("binutils-{name}-extracted-{index}"),
                makefile_ref.clone(),
                format!("/* Extracted from {basename}.  */\n"),
            )?);
            let mut arguments = Vec::new();
            if internal {
                arguments.push(literal("-i"));
            }
            arguments.extend([literal("-f"), input("program", "")]);
            let proto = author.builder.member(SOURCE, "bfd/doc/proto.str")?;
            parts.push(author.capture(
                GENERATORS,
                &format!("binutils-{name}-chew-{index}"),
                chew.clone(),
                BTreeMap::from([("program".into(), proto)]),
                arguments,
                Some(source),
            )?);
        }
        if !reached_header_sed {
            return Err(error("BFD header dependency terminator changed"));
        }
        let tail = if internal {
            "#ifdef __cplusplus\n}\n#endif\n"
        } else {
            "#ifdef __cplusplus\n}\n#endif\n#endif\n"
        };
        parts.push(author.text(
            GENERATORS,
            &format!("binutils-{name}-header-close"),
            makefile_ref.clone(),
            tail.into(),
        )?);
        let header = author.concat(GENERATORS, &format!("binutils-{name}-header-concat"), &native.transform, parts)?;
        sources.insert(path.into(), header);
    }
    Ok(())
}

fn parser_sources(author: &mut Author<'_>, native: &NativeHelpers, sources: &mut Files) -> Result<(), RunError> {
    for (directory, name) in [
        ("binutils", "arparse"),
        ("binutils", "defparse"),
        ("binutils", "mcparse"),
        ("binutils", "rcparse"),
        ("binutils", "sysinfo"),
        ("ld", "deffilep"),
        ("ld", "ldgram"),
        ("intl", "plural"),
    ] {
        let grammar_path = format!("{directory}/{name}.y");
        let output_path = format!("{name}.c");
        let args = if name == "plural" {
            vec!["-y", "-d", "--name-prefix=__gettext", "--output", output_path.as_str()]
        } else {
            vec!["-y", "-d", "-o", output_path.as_str()]
        };
        let grammar = sources
            .get(&grammar_path)
            .cloned()
            .ok_or_else(|| error(format!("missing original grammar {grammar_path}")))?;
        let generated = crate::source_built_stagex_programs::generators::bison_generate(
            author.builder,
            GENERATORS,
            &format!("binutils-{name}-bison"),
            grammar,
            "",
            &args,
        )?;
        for extension in if name == "plural" { &["c"][..] } else { &["c", "h"][..] } {
            let member = format!("{name}.{extension}");
            let reference = author.exec(
                GENERATORS,
                &format!("binutils-{name}-{extension}-extract"),
                native.transform.clone(),
                BTreeMap::from([("generated".into(), generated.clone())]),
                vec![literal("copy"), input("generated", &member), output("result", "")],
                ArtifactKind::Text,
            )?;
            sources.insert(format!("{directory}/{member}"), reference);
        }
    }
    for (directory, name) in [
        ("binutils", "arlex"),
        ("binutils", "deflex"),
        ("binutils", "syslex"),
        ("ld", "ldlex"),
    ] {
        let grammar_path = format!("{directory}/{name}.l");
        let output_path = format!("{name}.c");
        let grammar = sources
            .get(&grammar_path)
            .cloned()
            .ok_or_else(|| error(format!("missing original scanner {grammar_path}")))?;
        let generated = crate::source_built_stagex_programs::generators::flex_generate(
            author.builder,
            GENERATORS,
            &format!("binutils-{name}-flex"),
            grammar,
            "",
            &["-o", output_path.as_str()],
        )?;
        let reference = author.exec(
            GENERATORS,
            &format!("binutils-{name}-c-extract"),
            native.transform.clone(),
            BTreeMap::from([("generated".into(), generated)]),
            vec![literal("copy"), input("generated", &output_path), output("result", "")],
            ArtifactKind::Text,
        )?;
        sources.insert(format!("{directory}/{output_path}"), reference);
    }
    Ok(())
}

pub(super) fn extend(builder: &mut TransitionBuilder, request: &AuthoringRequest<'_>) -> Result<(), RunError> {
    let mut author = Author {
        builder,
        ready: BTreeSet::new(),
    };
    let native = native_helpers(&mut author)?;
    let mut sources = normalize_source(&mut author, &native)?;
    bfd_headers(&mut author, &native, &mut sources)?;
    parser_sources(&mut author, &native, &mut sources)?;
    runtime::extend(&mut author, &native, &mut sources, request)
}

const CANONICALIZER_COPY: &str = r#"#define main mantle_original_canonicalizer_main
#include "canonicalizer.c"
#undef main

int main(int argc, char **argv)
{
    struct stat metadata;
    unsigned char *bytes;
    int source;
    int destination;
    int status;
    if (argc != 3 || argv == NULL || argv[1] == NULL || argv[2] == NULL) return EXIT_USAGE;
    source = open(argv[1], O_RDONLY | O_NOFOLLOW);
    if (source < 0) return EXIT_INPUT;
    if (fstat(source, &metadata) != 0 || !S_ISREG(metadata.st_mode) || metadata.st_size <= 0 ||
        (uint64_t)metadata.st_size > FILE_BYTES_MAX) { close(source); return EXIT_INPUT; }
    bytes = malloc((size_t)metadata.st_size);
    if (bytes == NULL) { close(source); return EXIT_INPUT; }
    status = read_exact(source, bytes, (uint64_t)metadata.st_size);
    if (close(source) != 0) status = 0;
    if (!status) { free(bytes); return EXIT_INPUT; }
    destination = open(argv[2], O_WRONLY | O_CREAT | O_EXCL | O_NOFOLLOW, metadata.st_mode & FILE_MODE_PERMISSION_MASK);
    if (destination < 0) { free(bytes); return EXIT_OUTPUT; }
    status = write_exact(destination, bytes, (uint64_t)metadata.st_size);
    free(bytes);
    if (fchmod(destination, metadata.st_mode & FILE_MODE_PERMISSION_MASK) != 0) status = 0;
    if (close(destination) != 0) status = 0;
    if (!status) { unlink(argv[2]); return EXIT_OUTPUT; }
    return canonicalize_file(argv[2]);
}
"#;
