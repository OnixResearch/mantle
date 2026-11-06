//! Opt-in real protected macro controller smoke. The fixture compiler/plugins
//! implement the original request protocol; they are not a Rust bootstrap.
use std::fs;
use std::path::Path;
use std::path::PathBuf;

use super::*;
use crate::source_built_fixed_point::ProofHermeticityMode;
use crate::source_built_fixed_point::SourceAuthorityInput;
use crate::source_built_fixed_point::SourceAuthorityRole;
use crate::source_built_fixed_point::SourceBuiltFixedPointPlan;
use crate::source_built_fixed_point::SourceBuiltFixedPointPolicies;
use crate::source_built_fixed_point::SourceBuiltFixedPointResourceBounds;
use crate::source_built_fixed_point::SourceContentKind;
use crate::source_built_fixed_point_action_trust::ActionObservationState;
use crate::source_built_fixed_point_action_trust::ExecutionEventKind;
use crate::source_built_fixed_point_planning::InitialOutputAuthorityState;
use crate::source_built_fixed_point_planning::SourceBuiltFixedPointPlanInput;
use crate::source_built_fixed_point_planning::plan_source_built_fixed_point;
use crate::source_built_recipe::RecipePaths;
use crate::source_built_recipe::digest;
use crate::source_built_recipe::prepare;
use crate::source_built_recipe::run;

const FIXTURE_SOURCE: &str = r#"
fn smoke_payload(n:usize)->Vec<u8>{let mut b=vec![0,255,17];b.extend_from_slice(format!("macro-{n}\n").as_bytes());b}
fn smoke_reply(input:&[u8])->Vec<u8>{let mut b=b"plugin\0\xfe".to_vec();b.extend(input.iter().rev().map(|v|v^90));b.extend_from_slice("value β".as_bytes());b}
fn smoke_bootstrap(args:&[String])->io::Result<()>{
    use std::os::unix::fs::PermissionsExt;
    let image=std::fs::read(std::env::current_exe()?)?;
    for path in &args[1..4]{std::fs::write(path,&image)?;std::fs::set_permissions(path,std::fs::Permissions::from_mode(0o755))?;}
    let package=Path::new(&args[4]);std::fs::create_dir_all(package)?;
    let mut plugin=image;if args[6]=="tamper"{plugin.push(0);}
    std::fs::write(package.join("plugin"),plugin)?;
    std::fs::set_permissions(package.join("plugin"),std::fs::Permissions::from_mode(0o755))?;
    std::fs::create_dir_all(&args[5])?;Ok(())
}
fn smoke_compiler(args:&[String])->io::Result<()>{
    assert_eq!(std::env::args().next().unwrap(),"/inputs/mrustc");
    let history=PathBuf::from(std::env::var_os("MRUSTC_CAIRN_MACRO_HISTORY").unwrap());
    let count=std::fs::read_dir(&history)?.count();
    for n in 0..count{
        assert_eq!(std::fs::read(history.join(n.to_string()).join("stdin"))?,smoke_payload(n));
        assert_eq!(std::fs::read(history.join(n.to_string()).join("stdout"))?,smoke_reply(&smoke_payload(n)));
        assert_eq!(std::fs::read(format!("/tmp/macro-side-{n}"))?,smoke_payload(n));
    }
    assert_eq!(std::env::current_dir()?,Path::new("/inputs/package"));
    let requested=std::env::var("SMOKE_COUNT").unwrap().parse::<usize>().unwrap();
    if count==requested{std::fs::write(&args[1],b"actual-compiler-complete\0\xfe")?;return Ok(());}
    let work=PathBuf::from(std::env::var_os("MRUSTC_CAIRN_MACRO_WORK").unwrap()).join("macro-request");
    std::fs::create_dir_all(&work)?;
    for (name,value) in [("executable","/inputs/package/plugin"),("cwd","/inputs/package"),("macro-name","--smoke-plugin")]{std::fs::write(work.join(name),value)?;}
    let mut environment=Vec::new();
    for (name,value) in [("SMOKE_ACTUAL".to_string(),"value β".to_string()),("SMOKE_TURN".to_string(),count.to_string())]{
        environment.extend_from_slice(&(name.len()as u32).to_le_bytes());environment.extend_from_slice(&(value.len()as u32).to_le_bytes());environment.extend_from_slice(name.as_bytes());environment.extend_from_slice(value.as_bytes());
    }
    std::fs::write(work.join("environment"),environment)?;std::fs::write(work.join("stdin"),smoke_payload(count))?;
    std::process::exit(75)
}
fn smoke_plugin()->io::Result<()>{
    assert_eq!(std::env::var("SMOKE_ACTUAL").unwrap(),"value β");
    assert_eq!(std::env::current_dir()?,Path::new("/inputs/package"));
    let mut input=Vec::new();std::io::stdin().read_to_end(&mut input)?;
    let n=std::env::var("SMOKE_TURN").unwrap().parse::<usize>().unwrap();assert_eq!(input,smoke_payload(n));
    std::fs::write(format!("/tmp/macro-side-{n}"),&input)?;
    std::io::stdout().write_all(&smoke_reply(&input))?;std::io::stderr().write_all(b"plugin diagnostic\n")?;Ok(())
}
fn main()->io::Result<()>{
    let args=std::env::args().skip(1).collect::<Vec<_>>();
    match args.first().map(String::as_str){
        Some("--smoke-bootstrap")=>smoke_bootstrap(&args),
        Some("--smoke-compiler")=>smoke_compiler(&args),
        Some("--smoke-plugin")=>smoke_plugin(),
        _=>production_main(),
    }
}
"#;

fn fixture_binary(root: &Path) -> PathBuf {
    let rustc = std::env::var_os("MANTLE_MACRO_SMOKE_RUSTC").expect("explicit pinned rustc with musl std");
    let linker = std::env::var("MANTLE_MACRO_SMOKE_LINKER").expect("explicit musl linker");
    let source = root.join("macro-fixture.rs");
    let helper = crate::source_built_rust_programs::successors::helper_source();
    let helper = helper.replacen("fn main()->io::Result<()> {", "fn production_main()->io::Result<()> {", 1);
    fs::write(&source, format!("{helper}\n{FIXTURE_SOURCE}")).unwrap();
    let binary = root.join("macro-fixture");
    for tests in [false, true] {
        let output_path = if tests {
            root.join("macro-protocol-tests")
        } else {
            binary.clone()
        };
        let mut command = std::process::Command::new(&rustc);
        command
            .args([
                "--edition=2021",
                "--target",
                "x86_64-unknown-linux-musl",
                "--crate-name",
                "macro_fixture",
                "-C",
                &format!("linker={linker}"),
            ])
            .arg(&source)
            .arg("-o")
            .arg(&output_path);
        if tests {
            command.arg("--test");
        }
        let output = command.output().unwrap();
        assert!(output.status.success(), "fixture compilation failed: {}", String::from_utf8_lossy(&output.stderr));
        if tests {
            let output = std::process::Command::new(&output_path)
                .args(["macro_protocol_tests", "--nocapture"])
                .output()
                .unwrap();
            fs::write(root.join("macro-protocol-tests.log"), [&output.stdout[..], &output.stderr[..]].concat())
                .unwrap();
            assert!(
                output.status.success(),
                "helper protocol tests failed: {}{}",
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr)
            );
        }
    }
    binary
}

fn recipe(stage: &str) -> SourceRecipe {
    SourceRecipe {
        schema: crate::source_built_recipe::SCHEMA.into(),
        stage_id: stage.into(),
        preparation: RecipePreparation {
            tool: "compact-macro-protected-smoke".into(),
            provenance: "parent-compiled static request fixture; not a Rust bootstrap or provider proof".into(),
        },
        steps: Vec::new(),
        timeout_groups: Vec::new(),
        roots: Vec::new(),
        exports: BTreeMap::new(),
        semantic_roots: BTreeMap::new(),
    }
}

fn reference(stage: &str, step: &str, output: &str) -> ArtifactRef {
    ArtifactRef::Produced {
        action_id: format!("{stage}/{step}"),
        output_id: output.into(),
    }
}

fn bootstrap(tamper: bool) -> SourceRecipe {
    let mut recipe = recipe("stagex-transition");
    let mut declared = outputs("helper", ArtifactKind::Executable);
    for (name, kind) in [
        ("compiler", ArtifactKind::Executable),
        ("plugin", ArtifactKind::Executable),
        ("package", ArtifactKind::Tree),
        ("runtime", ArtifactKind::Tree),
    ] {
        declared.insert(name.into(), RecipeOutput {
            kind,
            bytes_max: 64 * 1024 * 1024,
        });
    }
    recipe.steps.push(RecipeStep {
        id: "fixture".into(),
        inputs: BTreeMap::from([("seed".into(), ArtifactRef::Source {
            role: SourceAuthorityRole::StagexSeed,
            path: String::new(),
        })]),
        outputs: declared,
        operation: RecipeOperation::Exec {
            executable_input: "seed".into(),
            invocation_input: None,
            arguments: vec![
                lit("--smoke-bootstrap"),
                output("helper", ""),
                output("compiler", ""),
                output("plugin", ""),
                output("package", ""),
                output("runtime", ""),
                lit(if tamper { "tamper" } else { "exact" }),
            ],
            environment: BTreeMap::new(),
            runtime_mounts: BTreeMap::new(),
            cwd_input: None,
            cwd_output: None,
            output_seeds: BTreeMap::new(),
            command_state: None,
            guard: None,
            stdin_input: None,
            merge_stderr: false,
            exit_policy: RecipeExitPolicy::Exact { code: 0 },
        },
    });
    recipe.roots.extend(
        ["helper", "compiler", "plugin", "package", "runtime"]
            .map(|name| reference("stagex-transition", "fixture", name)),
    );
    recipe
}

fn macro_recipe(count: u32, slots: usize) -> (SourceRecipe, ArtifactRef, ArtifactRef) {
    let mut recipe = recipe("full-source-rust-provider");
    let compiler = RecipeStep {
        id: "fixture-compiler".into(),
        inputs: BTreeMap::from([
            ("mrustc".into(), reference("stagex-transition", "fixture", "compiler")),
            ("package".into(), reference("stagex-transition", "fixture", "package")),
            ("rust-runtime".into(), reference("stagex-transition", "fixture", "runtime")),
        ]),
        outputs: outputs("translation", ArtifactKind::Tree),
        operation: RecipeOperation::Exec {
            executable_input: "mrustc".into(),
            invocation_input: None,
            arguments: vec![lit("--smoke-compiler"), output("translation", "result.bin")],
            environment: BTreeMap::from([("SMOKE_COUNT".into(), lit(count.to_string()))]),
            runtime_mounts: BTreeMap::from([("/mantle-rust-runtime".into(), "rust-runtime".into())]),
            cwd_input: Some("package".into()),
            cwd_output: None,
            output_seeds: BTreeMap::new(),
            command_state: None,
            guard: None,
            stdin_input: None,
            merge_stderr: false,
            exit_policy: RecipeExitPolicy::Exact { code: 0 },
        },
    };
    let (translation, tmp) = macros::append(
        &mut recipe,
        "smoke",
        compiler,
        reference("stagex-transition", "fixture", "helper"),
        &BTreeMap::from([("plugin".into(), reference("stagex-transition", "fixture", "plugin"))]),
        slots,
        None,
        1024 * 1024 * 1024,
    )
    .unwrap();
    recipe.roots.extend([translation.clone(), tmp.clone()]);
    (recipe, translation, tmp)
}

fn proof(root: &Path, roots: &BTreeMap<SourceAuthorityRole, PathBuf>) -> SourceBuiltFixedPointPlan {
    let source_inputs = roots
        .iter()
        .map(|(role, path)| {
            let (size_bytes, digest_blake3) = digest(path).unwrap();
            SourceAuthorityInput {
                id: format!("macro-smoke-{role:?}"),
                role: *role,
                kind: if path.is_dir() {
                    SourceContentKind::Directory
                } else {
                    SourceContentKind::RegularFile
                },
                digest_blake3,
                size_bytes,
            }
        })
        .collect();
    let policy = |text: &str| blake3::hash(text.as_bytes()).to_hex().to_string();
    let proof = plan_source_built_fixed_point(SourceBuiltFixedPointPlanInput {
        proof_id: format!("macro-smoke-{}", root.file_name().unwrap().to_str().unwrap()),
        logical_store_prefix: "/mantle/store".into(),
        source_inputs,
        initial_output_authority: InitialOutputAuthorityState {
            stagex_transition_entries: 0,
            native_provider_entries: 0,
            rust_provider_entries: 0,
            mantle_output_entries: 0,
        },
        policies: SourceBuiltFixedPointPolicies {
            expected_native_provider_digest_blake3: policy("no provider from smoke"),
            closure_policy_digest_blake3: policy("static declared fixture only"),
            hermeticity_policy_digest_blake3: policy("strict namespace macro smoke"),
            protected_execution_policy_digest_blake3: policy("exact fixture and sandbox images"),
            effect_policy_digest_blake3: policy("original binary protocol and tmp files"),
            normalization_policy_digest_blake3: policy("exact original output bytes"),
            hermeticity_mode: ProofHermeticityMode::Strict,
            live_fetch_allowed: false,
            cargo_invocation_allowed: false,
            ambient_discovery_allowed: false,
            fallback_allowed: false,
            provider_cache_completion_allowed: false,
        },
        resource_bounds: SourceBuiltFixedPointResourceBounds {
            elapsed_seconds_max: 120,
            disk_bytes_max: 1024 * 1024 * 1024,
            open_file_descriptors_max: 1024,
            protected_exec_events_max: 524_288,
            source_records_max: 64,
        },
    })
    .unwrap();
    fs::write(root.join("proof.json"), serde_json::to_vec_pretty(&proof).unwrap()).unwrap();
    proof
}

fn sources(
    root: &Path,
    seed: &Path,
    bootstrap: &SourceRecipe,
    macros: &SourceRecipe,
) -> BTreeMap<SourceAuthorityRole, PathBuf> {
    let lineage = root.join("lineage.json");
    fs::write(&lineage, serde_json::to_vec(bootstrap).unwrap()).unwrap();
    let mut roots = BTreeMap::from([
        (SourceAuthorityRole::StagexSeed, seed.to_path_buf()),
        (SourceAuthorityRole::StagexLineage, lineage),
    ]);
    for role in [
        SourceAuthorityRole::StagexSourceBundle,
        SourceAuthorityRole::NativeSourceBundle,
    ] {
        let file = root.join(format!("{role:?}.source"));
        fs::write(&file, b"smoke-only inert source\n").unwrap();
        roots.insert(role, file);
    }
    for role in [
        SourceAuthorityRole::RustSourceArchiveSet,
        SourceAuthorityRole::MantleSource,
        SourceAuthorityRole::VendorInputs,
    ] {
        let directory = root.join(format!("{role:?}.sources"));
        fs::create_dir(&directory).unwrap();
        fs::write(directory.join("README"), b"smoke-only inert source\n").unwrap();
        roots.insert(role, directory);
    }
    fs::write(
        roots[&SourceAuthorityRole::RustSourceArchiveSet].join("macro.json"),
        serde_json::to_vec(macros).unwrap(),
    )
    .unwrap();
    roots
}

// Exercise the real author and generic runtime in the existing seed-stage smoke
// lane. Only typed producer identities are restaged; no fictitious native/Rust
// provider outputs are installed to satisfy production stage prerequisites.
fn seed_stage(
    mut bootstrap: SourceRecipe,
    macros: SourceRecipe,
    translation: ArtifactRef,
    tmp: ArtifactRef,
) -> (SourceRecipe, ArtifactRef, ArtifactRef) {
    fn retarget(value: &mut serde_json::Value) {
        match value {
            serde_json::Value::Object(fields) => {
                if let Some(serde_json::Value::String(id)) = fields.get_mut("action_id") {
                    if let Some(rest) = id.strip_prefix("full-source-rust-provider/") {
                        *id = format!("stagex-transition/{rest}");
                    }
                }
                for value in fields.values_mut() {
                    retarget(value);
                }
            }
            serde_json::Value::Array(values) => {
                for value in values {
                    retarget(value);
                }
            }
            _ => {}
        }
    }
    bootstrap.steps.extend(macros.steps);
    bootstrap.roots = macros.roots;
    let mut value = serde_json::to_value((bootstrap, translation, tmp)).unwrap();
    retarget(&mut value);
    let (mut result, translation, tmp): (SourceRecipe, ArtifactRef, ArtifactRef) =
        serde_json::from_value(value).unwrap();
    // Compact input aliases are sorted from producer identities. Rebuild them
    // after moving the fixture into the seed stage rather than retaining stale
    // source-stage enumeration indices.
    for step in &mut result.steps {
        if let RecipeOperation::Controller {
            specification,
            guard: None,
        } = &step.operation
        {
            let mut declaration = recipe("stagex-transition");
            crate::source_built_recipe::controller::append(&mut declaration, specification.clone()).unwrap();
            *step = declaration.steps.pop().unwrap();
        }
    }
    (result, translation, tmp)
}

fn exercise(root: &Path, seed: &Path, sandbox: &Path, count: u32, slots: usize, tamper: bool, accepted: bool) {
    fs::create_dir(root).unwrap();
    let bootstrap = bootstrap(tamper);
    let (macros, translation, tmp) = macro_recipe(count, slots);
    let (recipe, translation, tmp) = seed_stage(bootstrap, macros, translation, tmp);
    let roots = sources(root, seed, &recipe, &recipe);
    let proof = proof(root, &roots);
    let host = digest(&std::env::current_exe().unwrap()).unwrap().1;
    let stage = prepare(
        &proof,
        "stagex-transition",
        SourceAuthorityRole::StagexLineage,
        "",
        &RecipePaths {
            source_roots: &roots,
            runtime_root: &root.join("macros"),
            sandbox_binary: sandbox,
        },
        &[],
        &host,
    )
    .unwrap();
    let mut state = ActionObservationState::default();
    let mut events = Vec::new();
    let adapters = [stage.adapter().clone()];
    let result = run(&stage, &mut |event| {
        state.record_adapters(&proof, &adapters, &event).map_err(error)?;
        events.push(event);
        Ok(())
    });
    fs::write(root.join("events.json"), serde_json::to_vec_pretty(&events).unwrap()).unwrap();
    let executed = |slot: &str| {
        events
            .iter()
            .filter(|event| {
                event.kind == ExecutionEventKind::BuildExecution
                    && event.normal_exit_code.is_some()
                    && crate::source_built_fixed_point_action_trust::controller_action_position(
                        &stage.adapter().controllers[0],
                        &event.action_id,
                    )
                    .unwrap()
                    .is_some_and(|position| {
                        !position.sandbox
                            && position.slot.as_deref() == Some(slot)
                            && position.role
                                == crate::source_built_fixed_point_action_trust::ActionControllerRole::Command
                    })
            })
            .count()
    };
    if accepted {
        let run =
            result.unwrap_or_else(|failure| panic!("protected macro smoke failed at {}: {failure}", root.display()));
        assert_eq!(
            fs::read(run.outputs[&translation].0.join("result.bin")).unwrap(),
            b"actual-compiler-complete\0\xfe"
        );
        assert_eq!(executed("compiler"), count as usize + 1);
        assert_eq!(executed("macro"), count as usize);
        for n in 0..count {
            let mut expected = vec![0, 255, 17];
            expected.extend_from_slice(format!("macro-{n}\n").as_bytes());
            assert_eq!(fs::read(run.outputs[&tmp].0.join(format!("macro-side-{n}"))).unwrap(), expected);
        }
        assert_eq!(
            state.controller_outcome("stagex-transition/smoke.macro-controller").unwrap().completed_turns,
            count * 2 + 1
        );
    } else {
        let failure = result.err().expect("invalid macro continuation admitted");
        fs::write(root.join("failure.txt"), failure.to_string()).unwrap();
        assert!(state.controller_outcome("stagex-transition/smoke.macro-controller").is_err());
        assert!(!root.join("macros/smoke.macro-translation/outputs/tree").exists());
        if tamper {
            assert_eq!(executed("macro"), 0);
        } else {
            assert_eq!(executed("macro"), slots);
        }
    }
    println!(
        "protected macro case {}: accepted={accepted}, compiler={}, macro={}",
        root.display(),
        executed("compiler"),
        executed("macro")
    );
}

fn deferred_namespace_case(root: &Path, decoder: &Path, emitted: &str, declared: &str, accepted: bool) {
    fs::create_dir(root).unwrap();
    let translation = root.join("translation");
    fs::create_dir(&translation).unwrap();
    fs::create_dir(root.join("prior-tmp")).unwrap();
    let command = format!("\"/inputs/package/wrapper\" \"@{emitted}/probe_cmd.txt\"\n");
    fs::write(translation.join("probe-codegen.sh"), &command).unwrap();
    fs::write(
        translation.join("probe_cmd.txt"),
        format!("\"-c\"\n\"{emitted}/probe.c\"\n\"-o\"\n\"{emitted}/probe\"\n"),
    )
    .unwrap();
    fs::write(translation.join("probe.c"), "int probe(void) { return 37; }\n").unwrap();
    let temporary = format!("{emitted}/tmp");
    fs::write(translation.join("probe-environment"), format!("6:TMPDIR,{}:{temporary},", temporary.len())).unwrap();
    let result = std::process::Command::new(decoder)
        .arg("prepare")
        .arg(&translation)
        .args(["probe", "bin", "/inputs/package/wrapper", "compiler/probe"])
        .arg(root.join("prior-tmp"))
        .arg(root.join("workspace"))
        .arg(root.join("environment"))
        .arg(root.join("generated.c"))
        .arg(root.join("command"))
        .arg(root.join("native-command"))
        .arg(declared)
        .output()
        .unwrap();
    fs::write(root.join("decoder.log"), &result.stderr).unwrap();
    assert_eq!(result.status.success(), accepted, "{}", String::from_utf8_lossy(&result.stderr));
    if accepted {
        assert_eq!(
            fs::read_to_string(root.join("workspace/tmp/compiler/probe_cmd.txt")).unwrap(),
            "\"-c\"\n\"/tmp/compiler/probe.c\"\n\"-o\"\n\"/tmp/compiler/probe\"\n"
        );
        assert_eq!(fs::read_to_string(root.join("environment")).unwrap(), "6:TMPDIR,17:/tmp/compiler/tmp,");
        assert_eq!(fs::read_to_string(root.join("command")).unwrap(), command);
    } else {
        let expected = if declared == "/outputs/translation" {
            "original exact deferred compiler/response"
        } else {
            "original translation namespace"
        };
        assert!(String::from_utf8_lossy(&result.stderr).contains(expected));
        assert!(!root.join("command").exists());
    }
    println!("deferred namespace case {}: accepted={accepted}", root.display());
}

fn deferred_namespace_smoke(root: &Path) {
    let source = root.join("deferred.c");
    fs::write(&source, TRANSLATION_HELPER_SOURCE).unwrap();
    let decoder = root.join("deferred");
    let result = std::process::Command::new(std::env::var_os("MANTLE_MACRO_SMOKE_LINKER").unwrap())
        .arg("-std=c99")
        .arg(&source)
        .arg("-o")
        .arg(&decoder)
        .output()
        .unwrap();
    assert!(result.status.success(), "{}", String::from_utf8_lossy(&result.stderr));
    for (name, emitted, declared, accepted) in [
        ("deferred-plain", "/outputs/translation", "/outputs/translation", true),
        ("deferred-macro", "/outputs/workspace/translation", "/outputs/workspace/translation", true),
        ("deferred-mismatch", "/outputs/workspace/translation", "/outputs/translation", false),
        ("deferred-unknown", "/outputs/unknown", "/outputs/unknown", false),
    ] {
        deferred_namespace_case(&root.join(name), &decoder, emitted, declared, accepted);
    }
}

#[test]
#[ignore = "requires explicit pinned musl compiler/linker, reviewed bwrap and fresh evidence root"]
fn compact_macro_protected_smoke() {
    let root = PathBuf::from(std::env::var_os("MANTLE_MACRO_SMOKE_ROOT").expect("fresh absolute smoke evidence root"));
    let sandbox = PathBuf::from(std::env::var_os("MANTLE_PROTECTED_BWRAP").expect("explicit reviewed bwrap"));
    assert!(root.is_absolute() && sandbox.is_absolute());
    fs::create_dir(&root).expect("smoke root must be fresh");
    let seed = fixture_binary(&root);
    deferred_namespace_smoke(&root);
    exercise(&root.join("early"), &seed, &sandbox, 0, 5612, false, true);
    exercise(&root.join("binary-replay"), &seed, &sandbox, 2, 5612, false, true);
    exercise(&root.join("exhausted"), &seed, &sandbox, 2, 1, false, false);
    exercise(&root.join("wrong-image"), &seed, &sandbox, 1, 5612, true, false);
}
