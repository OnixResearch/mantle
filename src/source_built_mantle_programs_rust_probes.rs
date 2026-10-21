//! Finite original Rust compiler-probe schedules for the admitted 1.94.1 stable
//! provider. This is not a Cargo metadata implementation: original generators
//! select their commands and write every generated source and final stdout.
use std::collections::BTreeMap;

use super::error;
use super::input;
use super::literal;
use super::metadata;
use super::native;
use super::output;
use super::prefixed;
use crate::errors::RunError;
use crate::rust_plan::RustUnitDerivationSummary;
use crate::source_built_recipe::ArtifactKind;
use crate::source_built_recipe::ArtifactRef;
use crate::source_built_recipe::RecipeExitPolicy;
use crate::source_built_recipe::RecipeGuard;
use crate::source_built_recipe::RecipeGuardCondition;
use crate::source_built_recipe::RecipeOperation;
use crate::source_built_recipe::RecipeOutput;
use crate::source_built_recipe::RecipeStep;
use crate::source_built_recipe::RecipeWord;
use crate::source_built_recipe::SourceRecipe;
use crate::source_built_recipe::WordPart;

#[path = "source_built_mantle_programs_rust_probes_selected.rs"]
mod selected;

const TEXT_MAX: u64 = 16 * 1024 * 1024;
const TREE_MAX: u64 = 512 * 1024 * 1024;
fn spec(kind: ArtifactKind, bytes_max: u64) -> RecipeOutput {
    RecipeOutput { kind, bytes_max }
}
fn produced(stage: &str, id: &str, name: &str) -> ArtifactRef {
    ArtifactRef::Produced {
        action_id: format!("{stage}/{id}"),
        output_id: name.into(),
    }
}

pub(super) struct RustProbe {
    pub arguments: Vec<RecipeWord>,
    /// Only source-proven fixed branches use Exact; feature detection consumes
    /// the genuine compiler's normal success or rejection.
    pub exit_policy: RecipeExitPolicy,
    pub stdin: bool,
    pub seed_out_dir: bool,
    pub autocfg_index: Option<u64>,
}

pub(super) struct ProbeSchedule {
    pub boundaries: Vec<metadata::ScriptProcessBoundary>,
    pub probes: Vec<RustProbe>,
    pub autocfg_state: bool,
    pub script_replacements: Vec<(String, String)>,
    pub script_append: Vec<String>,
    pub module_boundaries: BTreeMap<String, Vec<metadata::ScriptProcessBoundary>>,
}

fn literal_environment(environment: &BTreeMap<String, RecipeWord>, key: &str) -> Result<Option<String>, RunError> {
    environment
        .get(key)
        .map(|word| {
            let mut result = String::new();
            for part in &word.parts {
                match part {
                    WordPart::Literal { text } => result.push_str(text),
                    _ => return Err(error(format!("source-fixed probe policy requires literal {key}"))),
                }
            }
            Ok(result)
        })
        .transpose()
}

fn has_feature(unit: &RustUnitDerivationSummary, feature: &str) -> bool {
    let cfg = format!("feature={feature:?}");
    unit.derivation.args.windows(2).any(|pair| pair[0] == "--cfg" && pair[1] == cfg)
        || unit.derivation.args.iter().any(|argument| argument.strip_prefix("--cfg=") == Some(cfg.as_str()))
}

/// The selected finite policy excludes ambient stage/bootstrap/wrapper injection.
/// Unknown packages return None so the root can dispatch another complete author;
/// recognized packages never silently discard a command or selected feature.
pub(super) fn selected_schedule(
    package: &str,
    unit: &RustUnitDerivationSummary,
    environment: &BTreeMap<String, RecipeWord>,
) -> Result<Option<ProbeSchedule>, RunError> {
    let autocfg_probes = match package {
        "async-io" | "memoffset" | "memoffset-0.6.5" => Some(1),
        "bigdecimal" | "num-traits" => Some(2),
        "indexmap-1.9.3" => Some(if environment.contains_key("CARGO_FEATURE_STD") {
            0
        } else {
            2
        }),
        _ => None,
    };
    let version_queries = match package {
        "generic-array" | "proc-macro-error-attr" | "proc-macro2-diagnostics" => Some((1, false)),
        "uncased" => Some((2, false)),
        // Stable 1.94: is_max_version(1.38) is false, so the RHS runs too.
        "proc-macro-error" => Some((3, false)),
        "branches" | "crc32c" | "rustc_version_runtime" => Some((1, true)),
        "curve25519-dalek" | "rstest_macros" => Some((2, true)),
        _ => None,
    };
    if package != "proc-macro2"
        && autocfg_probes.is_none()
        && version_queries.is_none()
        && !selected::recognizes(package)
    {
        return Ok(None);
    }
    if unit.target_kind != "custom-build" {
        return Err(error("compiler probe schedule requires original custom-build unit"));
    }
    for key in [
        "RUSTC_STAGE",
        "RUSTC_BOOTSTRAP",
        "RUSTC_WRAPPER",
        "RUSTC_WORKSPACE_WRAPPER",
        "RUSTFLAGS",
    ] {
        if environment.contains_key(key) {
            return Err(error(format!("{package} finite stable probe policy forbids {key}")));
        }
    }
    let target =
        literal_environment(environment, "TARGET")?.ok_or_else(|| error("compiler probes require original TARGET"))?;
    if !matches!(target.as_str(), "x86_64-unknown-linux-gnu" | "x86_64-unknown-linux-musl") {
        return Err(error(format!("unselected compiler probe target {target}")));
    }
    let flags = literal_environment(environment, "CARGO_ENCODED_RUSTFLAGS")?.unwrap_or_default();
    let flags: Vec<_> = if flags.is_empty() {
        Vec::new()
    } else {
        flags.split('\x1f').map(literal).collect()
    };
    let mut schedule = ProbeSchedule {
        boundaries: Vec::new(),
        probes: Vec::new(),
        autocfg_state: false,
        script_replacements: Vec::new(),
        script_append: Vec::new(),
        module_boundaries: BTreeMap::new(),
    };
    if selected::recognizes(package) {
        selected::populate(&mut schedule, package, unit, environment, &target, &flags)?;
        return Ok(Some(schedule));
    }
    if package == "proc-macro2" {
        schedule.boundaries = vec![
            metadata::ScriptProcessBoundary {
                receiver: "Command::new(rustc).arg(\"--version\")".into(),
                kind: metadata::ScriptProcessKind::Output,
            },
            metadata::ScriptProcessBoundary {
                receiver: "cmd".into(),
                kind: metadata::ScriptProcessKind::Status,
            },
        ];
        schedule.probes.push(RustProbe {
            arguments: vec![literal("--version")],
            exit_policy: RecipeExitPolicy::Exact { code: 0 },
            stdin: false,
            seed_out_dir: false,
            autocfg_index: None,
        });
        if has_feature(unit, "proc-macro") {
            for (feature, exit_code) in [
                ("proc_macro_span", 1),
                ("proc_macro_span_location", 0),
                ("proc_macro_span_file", 0),
            ] {
                let mut arguments = [
                    "--cfg=procmacro2_build_probe",
                    "--edition=2021",
                    "--crate-name=proc_macro2",
                    "--crate-type=lib",
                    "--cap-lints=allow",
                    "--emit=dep-info,metadata",
                    "--out-dir",
                    "/outputs/out-dir/probe",
                ]
                .into_iter()
                .map(literal)
                .collect::<Vec<_>>();
                arguments.extend([
                    literal(format!("src/probe/{feature}.rs")),
                    literal("--target"),
                    literal(&target),
                ]);
                arguments.extend(flags.clone());
                let exit_policy = if exit_code == 1 {
                    RecipeExitPolicy::Exact { code: 1 }
                } else {
                    RecipeExitPolicy::OneOf { codes: vec![0, 1] }
                };
                schedule.probes.push(RustProbe {
                    arguments,
                    exit_policy,
                    stdin: false,
                    seed_out_dir: false,
                    autocfg_index: None,
                });
            }
        }
    } else if let Some(count) = autocfg_probes {
        schedule.autocfg_state = count != 0;
        if count != 0 {
            schedule.probes.push(RustProbe {
                arguments: vec![literal("--version"), literal("--verbose")],
                exit_policy: RecipeExitPolicy::Exact { code: 0 },
                stdin: false,
                seed_out_dir: false,
                autocfg_index: None,
            });
            for index in 0..count {
                let mut arguments = [
                    "--crate-name",
                    "@MANTLE_AUTOCFG_CRATE@",
                    "--crate-type=lib",
                    "--out-dir",
                    "/outputs/out-dir",
                    "--emit=llvm-ir",
                    "--target",
                ]
                .into_iter()
                .map(literal)
                .collect::<Vec<_>>();
                arguments.push(literal(&target));
                arguments.extend(flags.clone());
                arguments.push(literal("-"));
                let exit_policy = if index == 0 {
                    RecipeExitPolicy::Exact { code: 0 }
                } else {
                    RecipeExitPolicy::OneOf { codes: vec![0, 1] }
                };
                schedule.probes.push(RustProbe {
                    arguments,
                    exit_policy,
                    stdin: true,
                    seed_out_dir: false,
                    autocfg_index: Some(index),
                });
            }
        }
    } else if let Some((count, rustc_version)) = version_queries {
        for _ in 0..count {
            let arguments = if rustc_version {
                vec![literal("-vV")]
            } else {
                vec![literal("--verbose"), literal("--version")]
            };
            schedule.probes.push(RustProbe {
                arguments,
                exit_policy: RecipeExitPolicy::Exact { code: 0 },
                stdin: false,
                seed_out_dir: false,
                autocfg_index: None,
            });
        }
    }
    Ok(Some(schedule))
}

/// The executable is the single compilation of the original script with its
/// source-preserving overlay. Complete provider inputs include the exact Produced
/// rustc executable separately from the exported full-source sysroot tree.
pub(super) struct ProbeScript<'a> {
    pub id: &'a str,
    pub helper: ArtifactRef,
    pub compiler: ArtifactRef,
    pub script: ArtifactRef,
    pub package: ArtifactRef,
    pub environment: BTreeMap<String, RecipeWord>,
    pub runtime_inputs: BTreeMap<String, ArtifactRef>,
    pub runtime_mounts: BTreeMap<String, String>,
    pub generated_inputs: BTreeMap<String, ArtifactRef>,
}

fn assemble(
    program: &mut SourceRecipe,
    id: &str,
    name: &str,
    inputs: BTreeMap<String, ArtifactRef>,
    entries: BTreeMap<String, String>,
) -> ArtifactRef {
    program.steps.push(RecipeStep {
        id: id.into(),
        inputs,
        outputs: BTreeMap::from([(name.into(), spec(ArtifactKind::Tree, TREE_MAX))]),
        operation: RecipeOperation::AssembleTree {
            output: name.into(),
            entries,
        },
    });
    produced(&program.stage_id, id, name)
}

/// Each iteration is one source-enumerated command, not bounded discovery. The
/// next absent reply must suspend with exactly 75; the final original script must
/// exit 0. Extra/missing/changed commands fail the fixed schedule.
pub(super) fn append_probe_script(
    program: &mut SourceRecipe,
    script: ProbeScript<'_>,
    schedule: &ProbeSchedule,
) -> Result<metadata::ScriptRun, RunError> {
    let stage = program.stage_id.clone();
    let marker_id = format!("{}-empty-marker", script.id);
    program.steps.push(RecipeStep {
        id: marker_id.clone(),
        inputs: BTreeMap::new(),
        outputs: BTreeMap::from([("marker".into(), spec(ArtifactKind::Text, 64))]),
        operation: RecipeOperation::WriteText {
            output: "marker".into(),
            text: String::new(),
        },
    });
    let empty = assemble(
        program,
        &format!("{}-empty", script.id),
        "tree",
        BTreeMap::from([("marker".into(), produced(&stage, &marker_id, "marker"))]),
        BTreeMap::from([(".empty".into(), "marker".into())]),
    );
    let mut state = empty.clone();
    let mut replies = Vec::new();
    for (ordinal, probe) in schedule.probes.iter().enumerate() {
        let pending = continuation(program, &script, schedule, ordinal, &replies, &state, &empty, true)?;
        state = produced(&stage, &pending.step.id, "probe-state");
        let request = produced(&stage, &pending.step.id, "request");
        let source = produced(&stage, &pending.step.id, "probe-source");
        let pending_status = produced(&stage, &pending.step.id, "exit-status");
        program.steps.push(pending.step);
        let reply = append_probe(
            program,
            &script,
            ordinal,
            probe,
            request,
            source,
            state.clone(),
            pending.out_dir,
            pending_status,
        )?;
        replies.push(reply);
    }
    continuation(program, &script, schedule, schedule.probes.len(), &replies, &state, &empty, false)
}

fn continuation(
    program: &mut SourceRecipe,
    script: &ProbeScript<'_>,
    schedule: &ProbeSchedule,
    ordinal: usize,
    replies: &[ArtifactRef],
    state: &ArtifactRef,
    empty: &ArtifactRef,
    pending: bool,
) -> Result<metadata::ScriptRun, RunError> {
    let id = format!("{}-continuation-{ordinal}", script.id);
    let (inputs, entries) = if replies.is_empty() {
        (
            BTreeMap::from([("empty".into(), empty.clone())]),
            BTreeMap::from([(".empty".into(), "empty".into())]),
        )
    } else {
        (
            replies.iter().enumerate().map(|(i, reply)| (format!("reply-{i}"), reply.clone())).collect(),
            replies.iter().enumerate().map(|(i, _)| (i.to_string(), format!("reply-{i}"))).collect(),
        )
    };
    let tree = assemble(program, &format!("{id}-replies"), "replies", inputs, entries);
    let mut inputs = script.runtime_inputs.clone();
    inputs.insert("rustc".into(), script.compiler.clone());
    inputs.insert("process-replies".into(), tree);
    inputs.insert("probe-state".into(), state.clone());
    let mut environment = script.environment.clone();
    environment.insert("MANTLE_CC_REPLIES".into(), input("process-replies", ""));
    environment.insert(
        "MANTLE_CC_REQUEST".into(),
        if pending {
            output("request", "")
        } else {
            output("out-dir", "unexpected-request")
        },
    );
    if schedule.autocfg_state {
        environment.insert("MANTLE_AUTOCFG_STATE_INPUT".into(), input("probe-state", ""));
        environment.insert("MANTLE_AUTOCFG_STATE_OUTPUT".into(), output("probe-state", ""));
        // Kept identical even for the version phase, which does not write stdin.
        environment.insert("MANTLE_AUTOCFG_SOURCE".into(), output("probe-source", ""));
    }
    let mut run = metadata::original_script_run(
        &program.stage_id,
        &id,
        script.script.clone(),
        script.package.clone(),
        environment,
        inputs,
        script.runtime_mounts.clone(),
    )?;
    run.step.outputs.insert("probe-state".into(), spec(ArtifactKind::Tree, TREE_MAX));
    if schedule.autocfg_state {
        run.step.outputs.insert("probe-source".into(), spec(ArtifactKind::Text, TEXT_MAX));
        // A version-only phase never writes stdin. Initialize that workspace as
        // empty; every real stdin phase overwrites it with original source bytes.
        run.step.inputs.insert(
            "probe-source-seed".into(),
            produced(&program.stage_id, &format!("{}-empty-marker", script.id), "marker"),
        );
        if let RecipeOperation::Exec { output_seeds, .. } = &mut run.step.operation {
            output_seeds.insert("probe-source".into(), "probe-source-seed".into());
        }
    }
    if pending {
        run.step.outputs.insert("request".into(), spec(ArtifactKind::Object, TEXT_MAX));
        run.step.outputs.insert("exit-status".into(), spec(ArtifactKind::Text, 4));
        if let RecipeOperation::Exec { exit_policy, .. } = &mut run.step.operation {
            *exit_policy = RecipeExitPolicy::Exact { code: 75 };
        }
    }
    Ok(run)
}

fn append_probe(
    program: &mut SourceRecipe,
    script: &ProbeScript<'_>,
    ordinal: usize,
    probe: &RustProbe,
    request: ArtifactRef,
    source: ArtifactRef,
    state: ArtifactRef,
    generated: ArtifactRef,
    pending_status: ArtifactRef,
) -> Result<ArtifactRef, RunError> {
    let stage = program.stage_id.clone();
    let id = format!("{}-probe-{ordinal}", script.id);
    let mut inputs = script.runtime_inputs.clone();
    inputs.insert("package".into(), script.package.clone());
    // Preserve original script-generated source in the relocated writable output
    // tree. In particular heapless's actual probe.rs remains the compiler input;
    // it is neither reconstructed nor replaced by a stdin approximation.
    let mut output_seeds = BTreeMap::new();
    if probe.seed_out_dir {
        inputs.insert("probe-generated".into(), generated);
        output_seeds.insert("compiler-products".into(), "probe-generated".into());
    }
    let mappings = [(literal("/outputs/out-dir"), literal("/outputs/compiler-products"))];
    let mut response = metadata::rustc_request(
        &stage,
        &format!("{id}-arguments"),
        script.helper.clone(),
        request.clone(),
        script.compiler.clone(),
        input("package", ""),
        &probe.arguments,
        &mappings,
        inputs.clone(),
        script.runtime_mounts.clone(),
    )?;
    if let Some(index) = probe.autocfg_index {
        metadata::add_autocfg_name(&mut response, state, "uuid", index)?;
    }
    let actual_environment = response.metadata.clone();
    inputs.insert("rustc-request".into(), response.metadata);
    program.steps.extend(response.preparation);
    program.steps.push(response.step);
    inputs.insert("rustc".into(), script.compiler.clone());
    if probe.stdin {
        inputs.insert("probe-source".into(), source);
    }
    let mut arguments = vec![prefixed("@", input("rustc-request", "compiler.rsp"))];
    if !probe.arguments.iter().any(|word| {
        word.parts
            .iter()
            .any(|part| matches!(part, WordPart::Literal { text } if matches!(text.as_str(), "--version" | "-vV")))
    }) {
        if !inputs.contains_key("rust-provider") {
            return Err(error("probe compiler requires full source sysroot input rust-provider"));
        }
        arguments.extend([literal("--sysroot"), input("rust-provider", "")]);
    }
    let feature_probe = matches!(&probe.exit_policy, RecipeExitPolicy::OneOf { .. });
    let guard = if feature_probe {
        inputs.insert("continuation-status".into(), pending_status);
        Some(RecipeGuard {
            conditions: vec![RecipeGuardCondition {
                status_input: "continuation-status".into(),
                accepted_codes: vec![75],
            }],
        })
    } else {
        None
    };
    let mut compiler_step = RecipeStep {
        id: id.clone(),
        inputs,
        outputs: BTreeMap::from([
            ("compiler-products".into(), spec(ArtifactKind::Tree, TREE_MAX)),
            ("stdout".into(), spec(ArtifactKind::Text, TEXT_MAX)),
            ("stderr".into(), spec(ArtifactKind::Text, TEXT_MAX)),
            ("exit-status".into(), spec(ArtifactKind::Text, 4)),
        ]),
        operation: RecipeOperation::Exec {
            command_state: None,
            executable_input: "rustc".into(),
            invocation_input: None,
            arguments,
            environment: BTreeMap::from([("MANTLE_CAIRN_COMPILE_ENV".into(), input("rustc-request", "compiler.env"))]),
            runtime_mounts: script.runtime_mounts.clone(),
            cwd_input: Some("package".into()),
            cwd_output: None,
            stdin_input: probe.stdin.then(|| "probe-source".into()),
            exit_policy: probe.exit_policy.clone(),
            output_seeds,
            guard,
            merge_stderr: false,
        },
    };
    metadata::append_metadata_invocation(
        program,
        &mut compiler_step,
        script.helper.clone(),
        actual_environment,
        &script.generated_inputs,
    )?;
    program.steps.push(compiler_step);
    if feature_probe {
        // Rejected compiles have no ordinary-output authority. Observe the
        // actual guarded execution rather than mounting success-only products
        // or predicting its status. The helper copies the authenticated stdio
        // and, only on success, the genuine compiler output tree.
        let observed_id = format!("{id}-observed");
        program.steps.push(RecipeStep {
            id: observed_id.clone(),
            inputs: BTreeMap::new(),
            outputs: BTreeMap::from([("observed".into(), spec(ArtifactKind::Tree, TREE_MAX * 2))]),
            operation: RecipeOperation::ObservePhase {
                phase: id.clone(),
                output: "observed".into(),
                state: None,
            },
        });
        let reply_id = format!("{id}-reply");
        let mut inputs = script.runtime_inputs.clone();
        inputs.insert("metadata-helper".into(), script.helper.clone());
        inputs.insert("request".into(), request);
        inputs.insert("observed".into(), produced(&stage, &observed_id, "observed"));
        program.steps.push(RecipeStep {
            id: reply_id.clone(),
            inputs,
            outputs: BTreeMap::from([
                ("reply".into(), spec(ArtifactKind::Tree, TREE_MAX * 2)),
                ("stdout".into(), spec(ArtifactKind::Text, TEXT_MAX)),
                ("stderr".into(), spec(ArtifactKind::Text, TEXT_MAX)),
                ("exit-status".into(), spec(ArtifactKind::Text, 4)),
            ]),
            operation: RecipeOperation::Exec {
                command_state: None,
                executable_input: "metadata-helper".into(),
                invocation_input: None,
                arguments: vec![
                    literal("--rust-probe-reply"),
                    input("request", ""),
                    input("observed", ""),
                    output("reply", ""),
                ],
                environment: BTreeMap::new(),
                runtime_mounts: script.runtime_mounts.clone(),
                cwd_input: None,
                cwd_output: None,
                stdin_input: None,
                exit_policy: RecipeExitPolicy::Exact { code: 0 },
                output_seeds: BTreeMap::new(),
                guard: None,
                merge_stderr: false,
            },
        });
        return Ok(produced(&stage, &reply_id, "reply"));
    }
    // Bind the genuine complete output tree at files/ so autocfg's original
    // best-effort unlink and proc-macro2's original probe-directory removal see
    // precisely the original command's products, including dep-info and LLVM IR.
    Ok(assemble(
        program,
        &format!("{id}-reply"),
        "reply",
        BTreeMap::from([
            ("request".into(), request),
            ("stdout".into(), produced(&stage, &id, "stdout")),
            ("stderr".into(), produced(&stage, &id, "stderr")),
            ("status".into(), produced(&stage, &id, "exit-status")),
            ("files".into(), produced(&stage, &id, "compiler-products")),
        ]),
        ["request", "stdout", "stderr", "status", "files"]
            .into_iter()
            .map(|name| (name.into(), name.into()))
            .collect(),
    ))
}

/// Library overlays are required before compiling these original dependencies.
/// Every replacement is checked against admitted source at execution time by the
/// source-produced helper; source drift cannot silently choose a different plan.
pub(super) fn library_sources(
    package: &str,
    source_package: &ArtifactRef,
    compiler_package_path: &str,
) -> Result<Option<Vec<metadata::LibraryOverlaySource>>, RunError> {
    if !matches!(package, "autocfg" | "version_check" | "rustc_version") {
        return Ok(None);
    }
    let join = |root: &str, member: &str| {
        if root.is_empty() {
            member.to_owned()
        } else {
            format!("{root}/{member}")
        }
    };
    let original = |member: &str| match source_package {
        ArtifactRef::Source { role, path } => Ok(ArtifactRef::Source {
            role: *role,
            path: join(path, member),
        }),
        ArtifactRef::ManifestSource {
            role,
            record_identity,
            member: root,
        } => Ok(ArtifactRef::ManifestSource {
            role: *role,
            record_identity: record_identity.clone(),
            member: join(root, member),
        }),
        ArtifactRef::Produced { .. } => {
            Err(error("compiler probe library overlay requires exact original source package"))
        }
    };
    let source = |member: &str,
                  replacements: Vec<(String, String)>,
                  append: Vec<String>|
     -> Result<metadata::LibraryOverlaySource, RunError> {
        Ok(metadata::LibraryOverlaySource {
            original: original(member)?,
            compiler_path: format!("{compiler_package_path}/{member}"),
            replacements,
            append,
        })
    };
    let state = if package == "autocfg" { AUTOCFG_STATE_SOURCE } else { "" };
    let module = format!("\nmod mantle_cairn_script_process {{\n{}\n{}\n}}\n", native::CC_PROCESS_SOURCE, state);
    Ok(Some(match package {
        "autocfg" => vec![
            source(
                "src/lib.rs",
                vec![
                    (AUTOCFG_STDIN_BEFORE.into(), AUTOCFG_STDIN_AFTER.into()),
                    ("    hash\n}".into(), "    mantle_cairn_script_process::uuid(hash)\n}".into()),
                ],
                vec![module],
            )?,
            source(
                "src/version.rs",
                vec![(
                    "command\n            .args(&[\"--version\", \"--verbose\"])\n            .output()".into(),
                    "super::mantle_cairn_script_process::output(command.args(&[\"--version\", \"--verbose\"]))".into(),
                )],
                Vec::new(),
            )?,
        ],
        "version_check" => vec![source(
            "src/lib.rs",
            vec![(
                "Command::new(rustc).arg(\"--verbose\").arg(\"--version\").output()".into(),
                "mantle_cairn_script_process::output(Command::new(rustc).arg(\"--verbose\").arg(\"--version\"))".into(),
            )],
            vec![module],
        )?],
        "rustc_version" => vec![source(
            "src/lib.rs",
            vec![(
                "cmd\n            .arg(\"-vV\")\n            .output()".into(),
                "mantle_cairn_script_process::output(cmd.arg(\"-vV\"))".into(),
            )],
            vec![module],
        )?],
        _ => unreachable!("recognized original compiler probe library"),
    }))
}

const AUTOCFG_STDIN_BEFORE: &str = r#"        let mut child = try!(command.spawn().map_err(error::from_io));
        let mut stdin = child.stdin.take().expect("rustc stdin");

        try!(stdin.write_fmt(source).map_err(error::from_io));
        drop(stdin);

        match child.wait() {"#;
const AUTOCFG_STDIN_AFTER: &str = r#"        let mut stdin = Vec::new();
        try!(stdin.write_fmt(source).map_err(error::from_io));
        match mantle_cairn_script_process::with_stdin(&mut command, &stdin).map(|output| output.status) {"#;

/// Preserve actual random UUID and actual formatted source across split runs.
/// UUID is generated by original new_uuid, not chosen by the author. Replayed
/// stdin is byte-checked, while all original error/cleanup branches remain intact.
const AUTOCFG_STATE_SOURCE: &str = r#"
pub(crate) fn uuid(generated: u64) -> u64 {
    let previous = std::env::var_os("MANTLE_AUTOCFG_STATE_INPUT").expect("autocfg state input");
    let destination = std::env::var_os("MANTLE_AUTOCFG_STATE_OUTPUT").expect("autocfg state output");
    let previous = std::path::Path::new(&previous).join("uuid");
    let actual = match std::fs::read_to_string(previous) {
        Ok(value) => value.parse::<u64>().expect("actual original autocfg UUID"),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => generated,
        Err(error) => panic!("reading actual original autocfg UUID: {}", error),
    };
    std::fs::write(std::path::Path::new(&destination).join("uuid"), actual.to_string()).expect("retaining original autocfg UUID");
    actual
}
pub(crate) fn with_stdin(command: &mut std::process::Command, source: &[u8]) -> std::io::Result<std::process::Output> {
    let arguments: Vec<_> = command.get_args().collect();
    let name = arguments.windows(2).find(|pair| pair[0] == "--crate-name").expect("original autocfg crate name")[1];
    let name = name.to_str().expect("UTF8 original autocfg crate name");
    assert!(name.starts_with("autocfg_"), "original autocfg crate name prefix");
    let index = name.rsplit('_').next().unwrap().parse::<u64>().expect("original autocfg probe ordinal");
    let member = format!("stdin-{}.rs", index);
    let previous = std::env::var_os("MANTLE_AUTOCFG_STATE_INPUT").expect("autocfg state input");
    match std::fs::read(std::path::Path::new(&previous).join(&member)) {
        Ok(expected) if expected == source => (),
        Ok(_) => return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "original generated autocfg stdin changed")),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => (),
        Err(error) => return Err(error),
    }
    let destination = std::env::var_os("MANTLE_AUTOCFG_STATE_OUTPUT").expect("autocfg state output");
    std::fs::write(std::path::Path::new(&destination).join(member), source)?;
    std::fs::write(std::env::var_os("MANTLE_AUTOCFG_SOURCE").expect("autocfg generated source output"), source)?;
    let result = output(command)?;
    use std::io::Write;
    std::io::stdout().write_all(&result.stdout)?;
    std::io::stderr().write_all(&result.stderr)?;
    Ok(result)
}
"#;
