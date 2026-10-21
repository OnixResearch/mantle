//! Original aws-lc-sys 0.45.0 CC source continuation and prefix bootstrap.
//! No symbol name, Cargo directive, generated header or assembly is imported
//! from the vendored generated output. Both native archives are real products.
use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::path::Path;

use super::compression::append_schedule_with;
use super::metadata::ScriptProcessBoundary;
use super::metadata::ScriptProcessKind;
use super::metadata::ScriptRun;
use super::native::NativeCommandPhase;
use super::native::NativeRequestTools;
use super::native::NativeToolchain;
use super::native::append_native_request;
use super::native::append_x86_64_perlasm;
use crate::errors::RunError;
use crate::source_built_recipe::ArtifactKind;
use crate::source_built_recipe::ArtifactRef;
use crate::source_built_recipe::RecipeExitPolicy;
use crate::source_built_recipe::RecipeOperation;
use crate::source_built_recipe::RecipeOutput;
use crate::source_built_recipe::RecipeStep;
use crate::source_built_recipe::RecipeWord;
use crate::source_built_recipe::SourceRecipe;
use crate::source_built_recipe::WordPart;
use crate::source_built_recipe::error;

pub(crate) struct AwsSchedule {
    files: Vec<String>,
    pub assembly: Vec<(String, String)>,
    pub prefix: String,
}
pub(crate) struct AwsTools {
    pub helper: ArtifactRef,
    pub original_aws: ArtifactRef,
    pub perl: ArtifactRef,
    pub nm: ArtifactRef,
}
pub(crate) fn source_files() -> BTreeMap<String, String> {
    BTreeMap::from([("aws-source-helper.rs".into(), include_str!("source_built_mantle_programs_aws_helper.rs").into())])
}

/// Materialize the complete original tree admitted in the Mantle source closure.
/// This revision is the aws-lc gitlink of aws-lc-sys 0.45.0's recorded
/// aws-lc-rs commit 7943223c99d909bc399bdf1b856821bb04f1f3c5. The crate archive
/// omits util generators, so its packaged/generated tree is not a substitute.
pub(super) fn append_original_source(
    program: &mut SourceRecipe,
    stage: &super::Stage<'_>,
    helper: ArtifactRef,
) -> Result<ArtifactRef, RunError> {
    let id = "aws-original-source";
    program.steps.push(RecipeStep {
        id: id.into(),
        inputs: BTreeMap::from([
            ("aws-helper".into(), helper),
            ("original".into(), ArtifactRef::Source {
                role: crate::source_built_fixed_point::SourceAuthorityRole::MantleSource,
                path: "bootstrap/source-inputs/aws-lc-original-02561621ffa4cf17c0c4f70bc11a82df36b42ae9".into(),
            }),
        ]),
        outputs: BTreeMap::from([
            ("source".into(), stage.bound(ArtifactKind::Tree)),
            ("stdout".into(), stage.bound(ArtifactKind::Text)),
            ("stderr".into(), stage.bound(ArtifactKind::Text)),
        ]),
        operation: RecipeOperation::Exec {
            command_state: None,
            executable_input: "aws-helper".into(),
            arguments: vec![lit("merge"), input("original", ""), output("source")],
            environment: BTreeMap::new(),
            runtime_mounts: stage.runtime_mounts()?,
            cwd_input: None,
            cwd_output: None,
            output_seeds: BTreeMap::new(),
            stdin_input: None,
            merge_stderr: false,
            guard: None,
            exit_policy: RecipeExitPolicy::Exact { code: 0 },
            invocation_input: None,
        },
    });
    Ok(product(program, id, "source"))
}
pub(crate) fn script_boundaries() -> Vec<ScriptProcessBoundary> {
    vec![ScriptProcessBoundary {
        receiver: "Command::new(executable).args(args)".into(),
        kind: ScriptProcessKind::Output,
    }]
}
pub(crate) fn bootstrap_overlay(original: ArtifactRef, compiler_path: &str) -> super::metadata::LibraryOverlaySource {
    super::metadata::LibraryOverlaySource { original,compiler_path:compiler_path.into(),
        replacements:vec![("fn main() {".into(),"fn mantle_original_main() {".into())],
        append:vec!["fn main() { initialize(); cc_builder::CcBuilder::new(canonicalized_manifest_dir(), out_dir(), None, OutputLibType::Static).build().unwrap(); }".into()] }
}
fn detector(phases: &mut Vec<NativeCommandPhase>) {
    phases.extend([NativeCommandPhase::Preprocess, NativeCommandPhase::DriverProbe]);
}
fn probe(phases: &mut Vec<NativeCommandPhase>) {
    detector(phases);
    phases.push(NativeCommandPhase::CapabilityCompile);
}
fn inventory(path: &Path) -> Result<Vec<String>, RunError> {
    let source = std::fs::read_to_string(path)
        .map_err(|e| error(format!("reading original AWS source inventory {}: {e}", path.display())))?;
    let source = source
        .split("pub(super) const CRYPTO_LIBRARY: &[&str] = &[")
        .nth(1)
        .and_then(|s| s.split("];").next())
        .ok_or_else(|| error("changed original AWS source inventory declaration"))?;
    source
        .lines()
        .filter(|line| !line.trim().is_empty())
        .map(|line| {
            line.trim()
                .strip_prefix('"')
                .and_then(|line| line.strip_suffix("\","))
                .map(str::to_owned)
                .ok_or_else(|| error("nonliteral original AWS source inventory"))
        })
        .collect()
}
fn selected_phases(files: &[String], release: bool, native: bool) -> Result<Vec<NativeCommandPhase>, RunError> {
    let mut phases = Vec::new();
    detector(&mut phases); // collect_cc_only_build_options fresh Build
    detector(&mut phases); // collect_universal_build_options main Build
    probe(&mut phases); // original C11 language check
    if release {
        probe(&mut phases);
        detector(&mut phases);
        probe(&mut phases);
    } // main and assembler path mapping
    for _ in ["stdalign_check", "builtin_swap_check", "linux_random_h"] {
        probe(&mut phases);
    }
    if native {
        detector(&mut phases);
        phases.push(NativeCommandPhase::DriverProbe);
    } // original memcmp GCC version gate
    // Jitter has its own Build and cache. Both probes run before source batches.
    detector(&mut phases);
    probe(&mut phases); // jitter -ffile-prefix-map
    detector(&mut phases);
    phases.push(NativeCommandPhase::DriverProbe); // GCC rejects Clang sanitizer flag without a child
    let mut regular = Vec::new();
    let mut s2n = Vec::new();
    let mut jitter = Vec::new();
    for file in files {
        if file.starts_with("third_party/s2n-bignum") {
            s2n.push(file);
        } else if file.starts_with("third_party/jitterentropy") {
            jitter.push(file);
        } else {
            regular.push(file);
        }
    }
    phases.extend(s2n.iter().chain(jitter.iter()).chain(regular.iter()).map(|_| NativeCommandPhase::Compile));
    // Original cc::assemble appends ordinary objects, then s2n, then jitter.
    let mut remaining = 4000usize;
    let mut occupied = false;
    for file in regular.iter().chain(s2n.iter()).chain(jitter.iter()) {
        let stem = Path::new(file)
            .file_stem()
            .and_then(|s| s.to_str())
            .ok_or_else(|| error("AWS object source without UTF-8 stem"))?;
        let length = "/outputs/out-dir/".len() + 16 + 1 + stem.len() + 2;
        if occupied && length > remaining {
            phases.push(NativeCommandPhase::ArchiveCreate);
            remaining = 4000;
        }
        remaining = remaining.saturating_sub(length);
        occupied = true;
    }
    if occupied {
        phases.push(NativeCommandPhase::ArchiveCreate);
    }
    phases.push(NativeCommandPhase::ArchiveIndex);
    Ok(phases)
}
/// `package_root` is the admitted original aws-lc-sys crate, not an OUT_DIR.
/// The selected Mantle branch is x86_64 Linux static CC with pregenerated Rust
/// bindings; original generators replace the packaged native generated files.
pub(crate) fn selected_schedule(
    package: &str,
    features: &BTreeSet<String>,
    target: &str,
    package_root: &Path,
) -> Result<Option<AwsSchedule>, RunError> {
    if package != "aws-lc-sys" {
        return Ok(None);
    }
    if !matches!(target, "x86_64-unknown-linux-gnu" | "x86_64-unknown-linux-musl") {
        return Err(error("AWS original selected schedule requires x86_64 Linux"));
    }
    if ["fips", "ssl", "bindgen"].iter().any(|f| features.contains(*f)) {
        return Err(error("AWS selected Mantle CC schedule cannot replace an original CMake/bindgen selected branch"));
    }
    let mut files = inventory(&package_root.join("builder/cc_builder/universal.rs"))?;
    files.extend(inventory(&package_root.join("builder/cc_builder/linux_x86_64.rs"))?);
    // Original perlasm directives select these exact ELF outputs. SHA256 and
    // SHA512 intentionally share a generator and retain distinct basenames.
    let assembly = vec![
        ("crypto/chacha/chacha-x86_64.S", "crypto/chacha/asm/chacha-x86_64.pl"),
        ("crypto/cipher_extra/aes128gcmsiv-x86_64.S", "crypto/cipher_extra/asm/aes128gcmsiv-x86_64.pl"),
        ("crypto/cipher_extra/aesni-sha1-x86_64.S", "crypto/cipher_extra/asm/aesni-sha1-x86_64.pl"),
        ("crypto/cipher_extra/aesni-sha256-x86_64.S", "crypto/cipher_extra/asm/aesni-sha256-x86_64.pl"),
        (
            "crypto/cipher_extra/chacha20_poly1305_x86_64.S",
            "crypto/cipher_extra/asm/chacha20_poly1305_x86_64.pl",
        ),
        ("crypto/fipsmodule/aesni-gcm-avx512.S", "crypto/fipsmodule/modes/asm/aesni-gcm-avx512.pl"),
        ("crypto/fipsmodule/aesni-gcm-x86_64.S", "crypto/fipsmodule/modes/asm/aesni-gcm-x86_64.pl"),
        ("crypto/fipsmodule/aesni-x86_64.S", "crypto/fipsmodule/aes/asm/aesni-x86_64.pl"),
        ("crypto/fipsmodule/aesni-xts-avx512.S", "crypto/fipsmodule/aes/asm/aesni-xts-avx512.pl"),
        ("crypto/fipsmodule/ghash-ssse3-x86_64.S", "crypto/fipsmodule/modes/asm/ghash-ssse3-x86_64.pl"),
        ("crypto/fipsmodule/ghash-x86_64.S", "crypto/fipsmodule/modes/asm/ghash-x86_64.pl"),
        ("crypto/fipsmodule/md5-x86_64.S", "crypto/fipsmodule/md5/asm/md5-x86_64.pl"),
        ("crypto/fipsmodule/p256-x86_64-asm.S", "crypto/fipsmodule/ec/asm/p256-x86_64-asm.pl"),
        ("crypto/fipsmodule/p256_beeu-x86_64-asm.S", "crypto/fipsmodule/ec/asm/p256_beeu-x86_64-asm.pl"),
        ("crypto/fipsmodule/rdrand-x86_64.S", "crypto/fipsmodule/rand/asm/rdrand-x86_64.pl"),
        ("crypto/fipsmodule/rsaz-2k-avx512.S", "crypto/fipsmodule/bn/asm/rsaz-2k-avx512.pl"),
        ("crypto/fipsmodule/rsaz-3k-avx512.S", "crypto/fipsmodule/bn/asm/rsaz-3k-avx512.pl"),
        ("crypto/fipsmodule/rsaz-4k-avx512.S", "crypto/fipsmodule/bn/asm/rsaz-4k-avx512.pl"),
        ("crypto/fipsmodule/rsaz-avx2.S", "crypto/fipsmodule/bn/asm/rsaz-avx2.pl"),
        ("crypto/fipsmodule/sha1-x86_64.S", "crypto/fipsmodule/sha/asm/sha1-x86_64.pl"),
        ("crypto/fipsmodule/sha256-x86_64.S", "crypto/fipsmodule/sha/asm/sha512-x86_64.pl"),
        ("crypto/fipsmodule/sha512-x86_64.S", "crypto/fipsmodule/sha/asm/sha512-x86_64.pl"),
        ("crypto/fipsmodule/vpaes-x86_64.S", "crypto/fipsmodule/aes/asm/vpaes-x86_64.pl"),
        ("crypto/fipsmodule/x86_64-mont.S", "crypto/fipsmodule/bn/asm/x86_64-mont.pl"),
        ("crypto/fipsmodule/x86_64-mont5.S", "crypto/fipsmodule/bn/asm/x86_64-mont5.pl"),
        ("crypto/test/trampoline-x86_64.S", "crypto/test/asm/trampoline-x86_64.pl"),
    ]
    .into_iter()
    .map(|(a, b)| (a.into(), b.into()))
    .collect();
    Ok(Some(AwsSchedule {
        files,
        assembly,
        prefix: "aws_lc_0_45_0".into(),
    }))
}
fn lit(s: impl Into<String>) -> RecipeWord {
    RecipeWord {
        parts: vec![WordPart::Literal { text: s.into() }],
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
fn output_member(name: &str, path: impl Into<String>) -> RecipeWord {
    RecipeWord {
        parts: vec![WordPart::Output {
            name: name.into(),
            path: path.into(),
        }],
    }
}
fn product(program: &SourceRecipe, id: &str, name: &str) -> ArtifactRef {
    ArtifactRef::Produced {
        action_id: format!("{}/{id}", program.stage_id),
        output_id: name.into(),
    }
}
fn outputs(name: &str, kind: ArtifactKind) -> BTreeMap<String, RecipeOutput> {
    BTreeMap::from([
        (name.into(), RecipeOutput {
            kind,
            bytes_max: 2 * 1024 * 1024 * 1024,
        }),
        ("stdout".into(), RecipeOutput {
            kind: ArtifactKind::Text,
            bytes_max: 16 * 1024 * 1024,
        }),
        ("stderr".into(), RecipeOutput {
            kind: ArtifactKind::Text,
            bytes_max: 16 * 1024 * 1024,
        }),
        ("exit-status".into(), RecipeOutput {
            kind: ArtifactKind::Text,
            bytes_max: 4,
        }),
    ])
}
fn helper(
    program: &mut SourceRecipe,
    tools: &AwsTools,
    id: &str,
    mut inputs: BTreeMap<String, ArtifactRef>,
    arguments: Vec<RecipeWord>,
    name: &str,
    mounts: &BTreeMap<String, String>,
) -> ArtifactRef {
    inputs.insert("aws-helper".into(), tools.helper.clone());
    inputs.insert("aws-original".into(), tools.original_aws.clone());
    program.steps.push(RecipeStep {
        id: id.into(),
        inputs,
        outputs: outputs(name, ArtifactKind::Tree),
        operation: RecipeOperation::Exec {
            command_state: None,
            executable_input: "aws-helper".into(),
            invocation_input: None,
            arguments,
            environment: BTreeMap::new(),
            runtime_mounts: mounts.clone(),
            cwd_input: None,
            cwd_output: None,
            output_seeds: BTreeMap::new(),
            stdin_input: None,
            merge_stderr: false,
            guard: None,
            exit_policy: RecipeExitPolicy::Exact { code: 0 },
        },
    });
    product(program, id, name)
}
fn with_package(tools: &NativeRequestTools, package: ArtifactRef) -> NativeRequestTools {
    let n = &tools.native;
    NativeRequestTools {
        native: NativeToolchain {
            driver: n.driver.clone(),
            splitter: n.splitter.clone(),
            cc1: n.cc1.clone(),
            cc1_image: n.cc1_image.clone(),
            assembler: n.assembler.clone(),
            archiver: n.archiver.clone(),
            inputs: n.inputs.clone(),
            arguments: n.arguments.clone(),
            environment: n.environment.clone(),
            runtime_mounts: n.runtime_mounts.clone(),
            assembler_arguments: n.assembler_arguments.clone(),
        },
        helper: tools.helper.clone(),
        phase_helper: tools.phase_helper.clone(),
        package,
        compiler_program: tools.compiler_program.clone(),
        archiver_program: tools.archiver_program.clone(),
        assembler_image: tools.assembler_image.clone(),
    }
}
/// Run the original native algorithm's unprefixed bootstrap entry, then the
/// unchanged original main. Only the final main owns consumer Cargo metadata.
pub(crate) fn append_schedule(
    program: &mut SourceRecipe,
    tools: &NativeRequestTools,
    schedule: &AwsSchedule,
    id: &str,
    script: ArtifactRef,
    environment: BTreeMap<String, RecipeWord>,
    extra_inputs: BTreeMap<String, ArtifactRef>,
    runtime_mounts: BTreeMap<String, String>,
    aws: &AwsTools,
) -> Result<ScriptRun, RunError> {
    for tool in [&aws.helper, &aws.original_aws, &aws.perl, &aws.nm] {
        if !matches!(tool, ArtifactRef::Produced { .. }) {
            return Err(error("AWS native generator inputs must be Produced"));
        }
    }
    let text = |key: &str| -> Result<String, RunError> {
        environment
            .get(key)
            .ok_or_else(|| error(format!("AWS original environment missing {key}")))?
            .parts
            .iter()
            .map(|p| match p {
                WordPart::Literal { text } => Ok(text.as_str()),
                _ => Err(error(format!("AWS source phase selector {key} is not literal"))),
            })
            .collect::<Result<Vec<_>, _>>()
            .map(|s| s.concat())
    };
    let phases = selected_phases(
        &schedule.files,
        !matches!(text("OPT_LEVEL")?.as_str(), "0" | "1" | "2"),
        text("HOST")? == text("TARGET")?,
    )?;
    let empty_id = format!("{id}.assembler-empty-source");
    program.steps.push(RecipeStep {
        id: empty_id.clone(),
        inputs: BTreeMap::new(),
        outputs: BTreeMap::from([("source".into(), RecipeOutput {
            kind: ArtifactKind::Text,
            bytes_max: 1,
        })]),
        operation: RecipeOperation::WriteText {
            output: "source".into(),
            text: String::new(),
        },
    });
    let empty_source = product(program, &empty_id, "source");
    let as_id = format!("{id}.assembler-version");
    program.steps.push(RecipeStep {
        id: as_id.clone(),
        inputs: BTreeMap::from([
            ("as".into(), tools.native.assembler.clone()),
            ("empty-source".into(), empty_source),
        ]),
        outputs: outputs("object", ArtifactKind::Object),
        operation: RecipeOperation::Exec {
            command_state: None,
            executable_input: "as".into(),
            invocation_input: None,
            arguments: vec![
                lit("-v"),
                lit("--64"),
                lit("-o"),
                output("object"),
                input("empty-source", ""),
            ],
            environment: tools.native.environment.clone(),
            runtime_mounts: runtime_mounts.clone(),
            cwd_input: None,
            cwd_output: None,
            output_seeds: BTreeMap::new(),
            stdin_input: None,
            merge_stderr: false,
            guard: None,
            exit_policy: RecipeExitPolicy::Exact { code: 0 },
        },
    });
    let perlasm = helper(
        program,
        aws,
        &format!("{id}.perlasm-source"),
        BTreeMap::new(),
        vec![lit("perlasm-tree"), input("aws-original", ""), output("source")],
        "source",
        &runtime_mounts,
    );
    let package = helper(
        program,
        aws,
        &format!("{id}.source-data"),
        BTreeMap::from([("original-package".into(), tools.package.clone())]),
        vec![
            lit("prepare"),
            input("original-package", ""),
            input("aws-original", ""),
            output("package"),
        ],
        "package",
        &runtime_mounts,
    );
    let mut generated = BTreeMap::from([("base".into(), package)]);
    let mut merge = vec![lit("merge"), input("base", ""), output("package")];
    for (ordinal, (destination, source)) in schedule.assembly.iter().enumerate() {
        let mut env = tools.native.environment.clone();
        env.insert("MANTLE_AWS_AS_PROBE".into(), input("aws-as-probe", ""));
        // The native helper's explicit source tree retains the original script
        // algorithms; only process boundaries consume actual separate effects.
        let basename = Path::new(destination)
            .file_name()
            .and_then(|s| s.to_str())
            .ok_or_else(|| error("invalid AWS generated assembly basename"))?;
        let assembly = append_x86_64_perlasm(
            program,
            &format!("{id}.perlasm-{ordinal}"),
            aws.perl.clone(),
            aws.original_aws.clone(),
            perlasm.clone(),
            source,
            basename,
            env,
            runtime_mounts.clone(),
        )?;
        // Bind the same actual assembler observation on each original generator.
        for step in program.steps.iter_mut().rev().take(3) {
            if matches!(&step.operation, RecipeOperation::Exec { .. }) {
                step.inputs.insert("aws-as-probe".into(), ArtifactRef::Produced {
                    action_id: format!("{}/{as_id}", program.stage_id),
                    output_id: "stderr".into(),
                });
            }
        }
        let alias = format!("assembly-{ordinal}");
        generated.insert(alias.clone(), assembly);
        merge.extend([
            input(&alias, ""),
            output_member("package", format!("aws-lc/generated-src/linux-x86_64/{destination}")),
        ]);
    }
    let unprefixed_package =
        helper(program, aws, &format!("{id}.generated-package"), generated, merge, "package", &runtime_mounts);
    let unprefixed_tools = with_package(tools, unprefixed_package.clone());
    let mut bootstrap_env = environment.clone();
    bootstrap_env.insert("CARGO_MANIFEST_DIR".into(), input("package", ""));
    let bootstrap = extra_inputs
        .get("aws-symbol-bootstrap")
        .cloned()
        .ok_or_else(|| error("AWS original unprefixed native bootstrap executable is missing"))?;
    let mut unprefixed = append_schedule_with(
        program,
        &unprefixed_tools,
        &phases,
        &format!("{id}.unprefixed"),
        bootstrap,
        bootstrap_env,
        extra_inputs.clone(),
        runtime_mounts.clone(),
        |program, ordinal, phase, request, partial| {
            append_native_request(
                program,
                &unprefixed_tools,
                *phase,
                &format!("{id}.unprefixed.native-{ordinal}"),
                request,
                partial,
            )
        },
    )?;
    let unprefixed_out = unprefixed.out_dir.clone();
    unprefixed.step.inputs.insert("aws-original".into(), aws.original_aws.clone());
    program.steps.push(unprefixed.step);
    let nm_id = format!("{id}.symbols");
    let mut nm_outputs = outputs("unused", ArtifactKind::Tree);
    nm_outputs.remove("unused");
    program.steps.push(RecipeStep {
        id: nm_id.clone(),
        inputs: BTreeMap::from([
            ("nm".into(), aws.nm.clone()),
            ("unprefixed".into(), unprefixed_out),
            ("aws-original".into(), aws.original_aws.clone()),
        ]),
        outputs: nm_outputs,
        operation: RecipeOperation::Exec {
            command_state: None,
            executable_input: "nm".into(),
            invocation_input: None,
            arguments: vec![
                lit("-g"),
                lit("--defined-only"),
                lit("-P"),
                input("unprefixed", "libcrypto.a"),
            ],
            environment: tools.native.environment.clone(),
            runtime_mounts: runtime_mounts.clone(),
            cwd_input: None,
            cwd_output: None,
            output_seeds: BTreeMap::new(),
            stdin_input: None,
            merge_stderr: false,
            guard: None,
            exit_policy: RecipeExitPolicy::Exact { code: 0 },
        },
    });
    let actual_symbols = product(program, &nm_id, "stdout");
    let headers = helper(
        program,
        aws,
        &format!("{id}.prefix-headers"),
        BTreeMap::from([("actual-symbols".into(), actual_symbols)]),
        vec![
            lit("prefix"),
            input("aws-original", ""),
            input("actual-symbols", ""),
            lit(&schedule.prefix),
            output("headers"),
        ],
        "headers",
        &runtime_mounts,
    );
    let package = helper(
        program,
        aws,
        &format!("{id}.prefixed-package"),
        BTreeMap::from([("base".into(), unprefixed_package), ("headers".into(), headers)]),
        vec![
            lit("merge"),
            input("base", ""),
            output("package"),
            input("headers", ""),
            output_member("package", "generated-include/openssl"),
        ],
        "package",
        &runtime_mounts,
    );
    let prefixed_tools = with_package(tools, package);
    let mut environment = environment;
    environment.insert("CARGO_MANIFEST_DIR".into(), input("package", ""));
    append_schedule_with(
        program,
        &prefixed_tools,
        &phases,
        id,
        script,
        environment,
        extra_inputs,
        runtime_mounts,
        |program, ordinal, phase, request, partial| {
            append_native_request(program, &prefixed_tools, *phase, &format!("{id}.native-{ordinal}"), request, partial)
        },
    )
}
