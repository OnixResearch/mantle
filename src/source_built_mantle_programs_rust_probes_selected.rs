//! Source-enumerated schedules for the remaining selected original scripts.
//! Probe stdout, stdin, generated files and status always come from their original
//! producers. Source overlays change only process boundaries, never generators.
use super::*;

pub(super) fn recognizes(package: &str) -> bool {
    matches!(
        package,
        "libc"
            | "anyhow"
            | "rustix"
            | "rustix-0.38.44"
            | "crc32fast"
            | "httparse"
            | "erased-serde"
            | "linkme"
            | "linkme-impl"
            | "object"
            | "rayon-core"
            | "rustversion"
            | "thiserror"
            | "thiserror-1.0.69"
            | "owo-colors"
            | "paste"
            | "proc-macro-hack"
            | "quote"
            | "ref-cast"
            | "serde"
            | "serde_core"
            | "syn-1.0.109"
            | "target-lexicon"
            | "typeid"
            | "zerocopy"
            | "zmij"
            | "cap-primitives"
            | "cap-std"
            | "cap-fs-ext"
            | "io-extras"
            | "io-lifetimes"
            | "io-lifetimes-2.0.4"
            | "getrandom-0.3.4"
            | "portable-atomic"
            | "heapless"
            | "portable-atomic-util"
    )
}

fn probe(schedule: &mut ProbeSchedule, arguments: Vec<RecipeWord>, stdin: bool) {
    schedule.probes.push(RustProbe {
        arguments,
        stdin,
        seed_out_dir: false,
        autocfg_index: None,
        exit_policy: if stdin {
            RecipeExitPolicy::OneOf { codes: vec![0, 1] }
        } else {
            RecipeExitPolicy::Exact { code: 0 }
        },
    });
}
fn boundary(schedule: &mut ProbeSchedule, receiver: &str, kind: metadata::ScriptProcessKind) {
    schedule.boundaries.push(metadata::ScriptProcessBoundary {
        receiver: receiver.into(),
        kind,
    });
}
fn version(schedule: &mut ProbeSchedule, receiver: &str) {
    boundary(schedule, receiver, metadata::ScriptProcessKind::Output);
    probe(schedule, vec![literal("--version")], false);
}

pub(super) fn populate(
    schedule: &mut ProbeSchedule,
    package: &str,
    unit: &RustUnitDerivationSummary,
    environment: &BTreeMap<String, RecipeWord>,
    target: &str,
    flags: &[RecipeWord],
) -> Result<(), RunError> {
    match package {
        // These original scripts have no child processes. They must still run:
        // linkme writes private.rs, and rayon-core publishes its Cargo directives.
        "linkme" | "rayon-core" => (),
        "libc" => {
            if environment.contains_key("LIBC_CI") {
                return Err(error("selected libc script excludes its host FreeBSD CI query"));
            }
            version(schedule, "cmd");
        }
        "rustversion" => version(schedule, "command"),
        "crc32fast" => version(schedule, "Command::new(env::var_os(\"RUSTC\")?)\n        .arg(\"--version\")"),
        "httparse" => version(schedule, "Command::new(rustc)\n        .arg(\"--version\")"),
        "zerocopy" => version(schedule, "Command::new(rustc_cmd_name).arg(\"--version\")"),
        "getrandom-0.3.4" => {
            // The sole version call is inside the original Windows-only branch.
            // Both admitted GNU and musl targets are Linux.
            boundary(schedule, "cmd.arg(\"-vV\")", metadata::ScriptProcessKind::Output);
        }
        "portable-atomic" => {
            schedule.module_boundaries.insert("version.rs".into(), vec![metadata::ScriptProcessBoundary {
                receiver: "cmd.arg(\"-vV\")".into(),
                kind: metadata::ScriptProcessKind::Output,
            }]);
            probe(schedule, vec![literal("-vV")], false);
        }
        "heapless" => {
            boundary(schedule, "cmd", metadata::ScriptProcessKind::Status);
            let mut arguments = [
                "--edition=2018",
                "--crate-name=probe",
                "--crate-type=lib",
                "--out-dir",
                "/outputs/out-dir",
                "/outputs/out-dir/probe.rs",
                "--target",
            ]
            .into_iter()
            .map(literal)
            .collect::<Vec<_>>();
            arguments.push(literal(target));
            arguments.extend_from_slice(flags);
            probe(schedule, arguments, false);
            let probe = schedule.probes.last_mut().unwrap();
            probe.seed_out_dir = true;
            probe.exit_policy = RecipeExitPolicy::OneOf { codes: vec![0, 1] };
        }
        "portable-atomic-util" => {
            // The process is in an inline submodule, so name the root replay
            // module explicitly rather than adding a new version implementation.
            schedule.script_replacements.push((
                "cmd.arg(\"-vV\").output()".into(),
                "crate::mantle_cairn_script_process::output(cmd.arg(\"-vV\"))".into(),
            ));
            schedule.script_append.push(process_module(""));
            probe(schedule, vec![literal("-vV")], false);
        }
        "anyhow" | "thiserror" | "thiserror-1.0.69" => {
            boundary(schedule, "cmd", metadata::ScriptProcessKind::Status);
            let compile = package != "anyhow" || has_feature(unit, "std");
            if compile {
                let crate_name = if package == "anyhow" { "anyhow" } else { "thiserror" };
                let mut arguments = Vec::new();
                if package == "anyhow" {
                    arguments.push(literal("--cfg=anyhow_build_probe"));
                }
                arguments.extend([
                    literal("--edition=2018"),
                    literal(format!("--crate-name={crate_name}")),
                    literal("--crate-type=lib"),
                    literal("--cap-lints=allow"),
                    literal("--emit=dep-info,metadata"),
                    literal("--out-dir"),
                    literal("/outputs/out-dir/probe"),
                    literal(if package == "anyhow" {
                        "src/nightly.rs"
                    } else {
                        "build/probe.rs"
                    }),
                    literal("--target"),
                    literal(target),
                ]);
                arguments.extend_from_slice(flags);
                probe(schedule, arguments, false);
                schedule.probes.last_mut().unwrap().exit_policy = RecipeExitPolicy::OneOf { codes: vec![0, 1] };
            }
            if package != "thiserror-1.0.69" {
                version(schedule, "Command::new(rustc).arg(\"--version\")");
            }
        }
        "rustix" | "rustix-0.38.44" | "cap-primitives" | "cap-std" | "cap-fs-ext" | "io-extras" | "io-lifetimes"
        | "io-lifetimes-2.0.4" => {
            let count = match package {
                "cap-primitives" => 5,
                "cap-std" => 3,
                "cap-fs-ext" => 1,
                "io-extras" => 2,
                "io-lifetimes" | "io-lifetimes-2.0.4" => 0, // original WASI-only probe
                _ => {
                    let experimental = environment.contains_key("CARGO_CFG_RUSTIX_USE_EXPERIMENTAL_FEATURES")
                        && !environment.contains_key("CARGO_FEATURE_RUSTC_DEP_OF_STD");
                    let no_std = !has_feature(unit, "std");
                    usize::from(experimental) * 2
                        + usize::from(no_std) * if package == "rustix" { 5 } else { 4 }
                        + if package == "rustix" { 3 } else { 1 }
                }
            };
            schedule.script_replacements.push((STDIN_BEFORE.into(), STDIN_AFTER.into()));
            schedule.script_append.push(process_module(STDIN_SOURCE));
            schedule.autocfg_state = count != 0;
            for _ in 0..count {
                let mut arguments = vec![
                    literal("--crate-type=rlib"),
                    literal("--emit=metadata"),
                    literal("--target"),
                    literal(target),
                ];
                if matches!(package, "io-extras" | "io-lifetimes" | "io-lifetimes-2.0.4") {
                    arguments.extend([literal("--out-dir"), literal("/outputs/out-dir")]);
                } else {
                    arguments.extend([
                        literal("-o"),
                        literal(if package == "rustix" {
                            "/outputs/out-dir/rustix_test_can_compile"
                        } else {
                            "-"
                        }),
                    ]);
                }
                arguments.extend_from_slice(flags);
                arguments.push(literal("-"));
                probe(schedule, arguments, true);
            }
        }
        _ => version(schedule, "Command::new(rustc).arg(\"--version\")"),
    }
    Ok(())
}

fn process_module(extra: &str) -> String {
    format!(
        "\n#[allow(dead_code)]\nmod mantle_cairn_script_process {{\n{}\n{}\n}}\n",
        native::CC_PROCESS_SOURCE,
        extra
    )
}

// This is the exact common boundary in the original rustix, cap-* and io-*
// scripts. Formatting the original source into a byte buffer preserves writeln!'s
// trailing newline; the actual compiler consumes these bytes as its stdin.
const STDIN_BEFORE: &str = r#"    let mut child = cmd
        .arg("-") // Read from stdin.
        .stdin(Stdio::piped()) // Stdin is a pipe.
        .stderr(Stdio::null()) // Errors from feature detection aren't interesting and can be confusing.
        .spawn()
        .unwrap();

    writeln!(child.stdin.take().unwrap(), "{}", test.as_ref()).unwrap();

    child.wait().unwrap().success()"#;
const STDIN_AFTER: &str = r#"    cmd.arg("-").stdin(Stdio::piped()).stderr(Stdio::null());
    let mut source = Vec::new();
    writeln!(&mut source, "{}", test.as_ref()).unwrap();
    mantle_cairn_script_process::with_stdin(&mut cmd, &source).unwrap().status.success()"#;
const STDIN_SOURCE: &str = r#"
static STDIN_NEXT: AtomicUsize = AtomicUsize::new(0);
pub(crate) fn with_stdin(command: &mut Command, source: &[u8]) -> io::Result<Output> {
    let index = STDIN_NEXT.fetch_add(1, Ordering::SeqCst);
    let member = format!("stdin-{}.rs", index);
    let previous = std::env::var_os("MANTLE_AUTOCFG_STATE_INPUT").expect("original probe state input");
    match fs::read(Path::new(&previous).join(&member)) {
        Ok(expected) if expected == source => (),
        Ok(_) => return Err(io::Error::new(io::ErrorKind::InvalidData, "original generated probe stdin changed")),
        Err(error) if error.kind() == io::ErrorKind::NotFound => (),
        Err(error) => return Err(error),
    }
    let destination = std::env::var_os("MANTLE_AUTOCFG_STATE_OUTPUT").expect("original probe state output");
    fs::write(Path::new(&destination).join(member), source)?;
    fs::write(std::env::var_os("MANTLE_AUTOCFG_SOURCE").expect("original probe source output"), source)?;
    output(command)
}
"#;
