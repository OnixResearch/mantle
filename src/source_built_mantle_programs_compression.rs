//! Finite continuations of the pinned compression packages' original build scripts.
//! Every boundary resumes the original executable with only genuine native replies;
//! the final successful original process, not this schedule, owns Cargo stdout.
use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::path::Path;

use super::metadata::ScriptRun;
use super::metadata::original_script_run;
use super::native::NativeCommandPhase;
use super::native::NativeRequestTools;
use super::native::append_cc_reply_files_tree;
use super::native::append_native_request;
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

/// The source operand documents the original phase; the actual request is emitted
/// by original cc, including its selected flags and hashed object destination.
// Pending wiring in the source-only preparation path (Cairn prove-source-built-mantle-fixed-point I3).
#[allow(dead_code)]
pub(crate) struct CompressionPhase {
    pub kind: NativeCommandPhase,
    pub source: Option<String>,
    pub flag: Option<String>,
}
// Pending wiring in the source-only preparation path (Cairn prove-source-built-mantle-fixed-point
// I3).
#[allow(dead_code)]
pub(crate) struct CompressionSchedule {
    pub package: String,
    pub phases: Vec<CompressionPhase>,
    /// An original pkg-config attempt precedes the native fallback when true.
    /// Its process boundary must be lowered by the package's source dependency
    /// overlay; this is not permission to replace the original static feature.
    pub probes_pkg_config: bool,
}
fn phase(kind: NativeCommandPhase) -> CompressionPhase {
    CompressionPhase {
        kind,
        source: None,
        flag: None,
    }
}
fn detector(phases: &mut Vec<CompressionPhase>) {
    // cc::Tool::detect_family_inner, then guess_family_from_stdout's -?.
    phases.push(phase(NativeCommandPhase::Preprocess));
    phases.push(phase(NativeCommandPhase::DriverProbe));
}
fn flag_probe(phases: &mut Vec<CompressionPhase>, flag: &str, driver_rejects: bool) {
    // Each is_flag_supported_inner creates a fresh Build with its own family
    // cache. The enclosing Build's detector has already run.
    detector(phases);
    phases.push(CompressionPhase {
        kind: if driver_rejects {
            NativeCommandPhase::DriverProbe
        } else {
            NativeCommandPhase::CapabilityCompile
        },
        source: Some("flag_check.c".into()),
        flag: Some(flag.into()),
    });
}
fn names(dir: &str, values: &str) -> Vec<String> {
    values.split_whitespace().map(|name| format!("{dir}/{name}")).collect()
}
fn compile_batch(phases: &mut Vec<CompressionPhase>, files: &[String], nested_lib: bool) {
    for file in files {
        phases.push(CompressionPhase {
            kind: NativeCommandPhase::Compile,
            source: Some(file.clone()),
            flag: None,
        });
    }
    // cc::Build::assemble uses a 4000-byte budget for the *actual object paths*,
    // not a fixed object count. The hash is always 16 hexadecimal digits; only
    // its width matters for this source-derived finite batch partition.
    let out = if nested_lib {
        "/outputs/out-dir/lib/"
    } else {
        "/outputs/out-dir/"
    };
    let mut remaining = 4000usize;
    let mut occupied = false;
    for file in files {
        let stem = Path::new(file)
            .file_stem()
            .expect("source inventory has stems")
            .to_str()
            .expect("source inventory is UTF-8");
        let length = out.len() + 16 + 1 + stem.len() + 2;
        if occupied && length > remaining {
            phases.push(phase(NativeCommandPhase::ArchiveCreate));
            remaining = 4000;
        }
        remaining = remaining.saturating_sub(length);
        occupied = true;
    }
    if occupied {
        phases.push(phase(NativeCommandPhase::ArchiveCreate));
    }
    phases.push(phase(NativeCommandPhase::ArchiveIndex));
}

/// Pinned Linux/GNU source policy. Feature selection is the original compiler's
/// --cfg inventory, never an alternative pure-Rust package substitution.
/// Unrecognized packages return None so the root retains its ordinary scripts.
pub(crate) fn selected_schedule(
    package: &str,
    features: &BTreeSet<String>,
    target: &str,
) -> Result<Option<CompressionSchedule>, RunError> {
    if !matches!(package, "zstd-sys" | "bzip2-sys" | "lzma-sys") {
        return Ok(None);
    }
    if target != "x86_64-unknown-linux-musl" && target != "x86_64-unknown-linux-gnu" {
        return Err(error(format!(
            "{package} original native schedule requires the selected x86_64 Linux GNU compiler, not {target}"
        )));
    }
    let has = |feature: &str| features.contains(feature);
    let mut phases = Vec::new();
    let mut probes_pkg_config = false;
    match package {
        "bzip2-sys" => {
            if !has("__disabled") {
                probes_pkg_config = !has("static");
                detector(&mut phases);
                let files = names(
                    "bzip2-1.0.8",
                    "blocksort.c huffman.c crctable.c randtable.c compress.c decompress.c bzlib.c",
                );
                compile_batch(&mut phases, &files, true);
            }
        }
        "lzma-sys" => {
            probes_pkg_config = !has("static");
            detector(&mut phases);
            let mut files = names(
                "xz-5.2/src/liblzma/common",
                "alone_decoder.c alone_encoder.c auto_decoder.c block_buffer_decoder.c block_buffer_encoder.c block_decoder.c block_encoder.c block_header_decoder.c block_header_encoder.c block_util.c common.c easy_buffer_encoder.c easy_decoder_memusage.c easy_encoder_memusage.c easy_encoder.c easy_preset.c filter_buffer_decoder.c filter_buffer_encoder.c filter_common.c filter_decoder.c filter_encoder.c filter_flags_decoder.c filter_flags_encoder.c hardware_cputhreads.c hardware_physmem.c index_decoder.c index_encoder.c index_hash.c index.c outqueue.c stream_buffer_decoder.c stream_buffer_encoder.c stream_decoder.c stream_encoder_mt.c stream_encoder.c stream_flags_common.c stream_flags_decoder.c stream_flags_encoder.c vli_decoder.c vli_encoder.c vli_size.c",
            );
            files.extend(names("xz-5.2/src/liblzma/lzma", "fastpos_table.c lzma_decoder.c lzma_encoder_optimum_fast.c lzma_encoder_optimum_normal.c lzma_encoder_presets.c lzma_encoder.c lzma2_decoder.c lzma2_encoder.c"));
            files.extend(names("xz-5.2/src/liblzma/lz", "lz_decoder.c lz_encoder_mf.c lz_encoder.c"));
            files.extend(names(
                "xz-5.2/src/liblzma/check",
                "check.c crc32_fast.c crc32_table.c crc64_fast.c crc64_table.c sha256.c",
            ));
            files.extend(names("xz-5.2/src/liblzma/delta", "delta_common.c delta_decoder.c delta_encoder.c"));
            files.extend(names("xz-5.2/src/liblzma/rangecoder", "price_table.c"));
            files.extend(names(
                "xz-5.2/src/liblzma/simple",
                "arm.c armthumb.c ia64.c powerpc.c simple_coder.c simple_decoder.c simple_encoder.c sparc.c x86.c",
            ));
            files.extend(names("xz-5.2/src/common", "tuklib_cpucores.c tuklib_physmem.c"));
            files.sort();
            compile_batch(&mut phases, &files, false);
        }
        "zstd-sys" => {
            if has("pkg-config") {
                return Err(error(
                    "zstd pkg-config-selected source branch needs its original pkg-config generator schedule",
                ));
            }
            if has("bindgen") {
                return Err(error(
                    "zstd bindgen-selected source branch additionally needs its original libclang binding generator phases",
                ));
            }
            detector(&mut phases);
            // thin-lto probes are immediate; the three section/merge probes are
            // deferred until compile's first get_compiler call.
            if !has("fat-lto") && has("thin-lto") {
                flag_probe(&mut phases, "-flto=thin", true);
                flag_probe(&mut phases, "-flto", false);
            }
            for flag in ["-ffunction-sections", "-fdata-sections", "-fmerge-all-constants"] {
                flag_probe(&mut phases, flag, false);
            }
            if has("fat-lto") {
                flag_probe(&mut phases, "-flto", false);
            }
            let mut files = names(
                "zstd/lib/common",
                "debug.c entropy_common.c error_private.c fse_decompress.c pool.c threading.c zstd_common.c",
            );
            let mut compress = names(
                "zstd/lib/compress",
                "fse_compress.c hist.c huf_compress.c zstd_compress_literals.c zstd_compress_sequences.c zstd_compress_superblock.c zstd_compress.c zstd_double_fast.c zstd_fast.c zstd_lazy.c zstd_ldm.c zstd_opt.c zstd_preSplit.c zstdmt_compress.c",
            );
            compress.sort();
            files.extend(compress);
            let mut decompress =
                names("zstd/lib/decompress", "huf_decompress.c zstd_ddict.c zstd_decompress_block.c zstd_decompress.c");
            decompress.sort();
            files.extend(decompress);
            if has("seekable") {
                files.extend(names("zstd/contrib/seekable_format", "zstdseek_compress.c zstdseek_decompress.c"));
            }
            if has("zdict_builder") {
                files.extend(names("zstd/lib/dictBuilder", "cover.c divsufsort.c fastcover.c zdict.c"));
            }
            if has("legacy") {
                files.extend(names(
                    "zstd/lib/legacy",
                    "zstd_v01.c zstd_v02.c zstd_v03.c zstd_v04.c zstd_v05.c zstd_v06.c zstd_v07.c",
                ));
            }
            if !has("no_asm") {
                files.push("zstd/lib/decompress/huf_decompress_amd64.S".into());
            }
            compile_batch(&mut phases, &files, false);
        }
        _ => unreachable!(),
    }
    Ok(Some(CompressionSchedule {
        package: package.into(),
        phases,
        probes_pkg_config,
    }))
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
fn spec(kind: ArtifactKind, bytes_max: u64) -> RecipeOutput {
    RecipeOutput { kind, bytes_max }
}

/// Execute exactly one original process for every source-fixed continuation,
/// followed by one successful original process. No probing loop, guessed upper
/// bound or metadata synthesis is permitted. Missing or extra requests fail.
pub(crate) fn append_schedule(
    program: &mut SourceRecipe,
    tools: &NativeRequestTools,
    schedule: &CompressionSchedule,
    id: &str,
    script: ArtifactRef,
    environment: BTreeMap<String, RecipeWord>,
    extra_inputs: BTreeMap<String, ArtifactRef>,
    runtime_mounts: BTreeMap<String, String>,
) -> Result<ScriptRun, RunError> {
    append_schedule_with(
        program,
        tools,
        &schedule.phases,
        id,
        script,
        environment,
        extra_inputs,
        runtime_mounts,
        |program, ordinal, selected, request, partial| {
            append_native_request(program, tools, selected.kind, &format!("{id}.native-{ordinal}"), request, partial)
        },
    )
}

/// Reuse the same source-fixed continuation machinery for a package whose
/// finite original phases include non-cc native generators (for example Perl).
pub(crate) fn append_schedule_with<P, F>(
    program: &mut SourceRecipe,
    tools: &NativeRequestTools,
    phases: &[P],
    id: &str,
    script: ArtifactRef,
    environment: BTreeMap<String, RecipeWord>,
    extra_inputs: BTreeMap<String, ArtifactRef>,
    runtime_mounts: BTreeMap<String, String>,
    mut produce: F,
) -> Result<ScriptRun, RunError>
where
    F: FnMut(
        &mut SourceRecipe,
        usize,
        &P,
        ArtifactRef,
        ArtifactRef,
    ) -> Result<super::native::NativeCommandProducts, RunError>,
{
    let mut replies = Vec::new();
    let mut previous_state = None;
    for (ordinal, selected) in phases.iter().enumerate() {
        let step_id = format!("{id}.continuation-{ordinal}");
        let mut run = continuation(
            program,
            tools,
            &step_id,
            script.clone(),
            &environment,
            &extra_inputs,
            &runtime_mounts,
            &replies,
            previous_state.as_ref(),
        )?;
        run.step.outputs.insert("request".into(), spec(ArtifactKind::Object, 16 * 1024 * 1024));
        let RecipeOperation::Exec {
            environment,
            exit_policy,
            ..
        } = &mut run.step.operation
        else {
            unreachable!()
        };
        environment.insert("MANTLE_CC_REQUEST".into(), output("request"));
        *exit_policy = RecipeExitPolicy::Exact { code: 75 };
        let request = product(&program.stage_id, &step_id, "request");
        previous_state = Some(run.out_dir.clone());
        program.steps.push(run.step);
        let products = produce(program, ordinal, selected, request.clone(), previous_state.clone().unwrap())?;
        let reply_id = format!("{id}.reply-{ordinal}");
        replies.push(append_cc_reply_files_tree(program, &reply_id, request, products)?);
    }
    let final_id = format!("{id}.complete");
    let mut run = continuation(
        program,
        tools,
        &final_id,
        script,
        &environment,
        &extra_inputs,
        &runtime_mounts,
        &replies,
        previous_state.as_ref(),
    )?;
    // An unexpected additional request exits75 and fails the Exact0 action.
    // A successful original process need not create a request artifact.
    let RecipeOperation::Exec { environment, .. } = &mut run.step.operation else {
        unreachable!()
    };
    environment.insert("MANTLE_CC_REQUEST".into(), RecipeWord {
        parts: vec![WordPart::Output {
            name: "out-dir".into(),
            path: ".unexpected-native-request".into(),
        }],
    });
    Ok(run)
}
pub(crate) fn continuation(
    program: &mut SourceRecipe,
    tools: &NativeRequestTools,
    id: &str,
    script: ArtifactRef,
    environment: &BTreeMap<String, RecipeWord>,
    extra_inputs: &BTreeMap<String, ArtifactRef>,
    runtime_mounts: &BTreeMap<String, String>,
    replies: &[ArtifactRef],
    previous_state: Option<&ArtifactRef>,
) -> Result<ScriptRun, RunError> {
    let replies_id = format!("{id}.replies");
    // AssembleTree intentionally rejects an empty tree. A nonnumeric marker is
    // inert to the original cc ordinal lookup and also binds the real script.
    let mut inputs = BTreeMap::from([("original-script".into(), script.clone())]);
    let mut entries = BTreeMap::from([("original-script".into(), "original-script".into())]);
    for (ordinal, reply) in replies.iter().enumerate() {
        let alias = format!("reply-{ordinal}");
        inputs.insert(alias.clone(), reply.clone());
        entries.insert(ordinal.to_string(), alias);
    }
    program.steps.push(RecipeStep {
        id: replies_id.clone(),
        inputs,
        outputs: BTreeMap::from([("replies".into(), spec(ArtifactKind::Tree, 2 * 1024 * 1024 * 1024))]),
        operation: RecipeOperation::AssembleTree {
            output: "replies".into(),
            entries,
        },
    });
    let mut inputs = extra_inputs.clone();
    let mut environment = environment.clone();
    inputs.insert("cc-replies".into(), product(&program.stage_id, &replies_id, "replies"));
    environment.insert("MANTLE_CC_REPLIES".into(), input("cc-replies", ""));
    if let Some(state) = previous_state {
        inputs.insert("cc-phase-state".into(), state.clone());
        environment.insert("MANTLE_CC_PHASE_STATE".into(), input("cc-phase-state", ""));
    }
    // These are the same source-produced executables the original cc chooses.
    // They are never invoked as opaque child-launching compilers in the script.
    for (alias, reference) in &tools.native.inputs {
        if inputs.insert(alias.clone(), reference.clone()).is_some_and(|old| old != *reference) {
            return Err(error("compression script native input alias collision"));
        }
    }
    original_script_run(
        &program.stage_id,
        id,
        script,
        tools.package.clone(),
        environment,
        inputs,
        runtime_mounts.clone(),
    )
}

/// Pure request decoding, canonical typed invocations and actual-result merging.
/// Compiled by the same source-owned compiler as the original build scripts.
pub(crate) const NATIVE_REQUEST_HELPER_SOURCE: &str = include_str!("source_built_mantle_programs_native_request.rs");
