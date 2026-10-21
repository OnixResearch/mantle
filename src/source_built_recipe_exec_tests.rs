//! Real Linux executor regressions. The ignored recipe cases require an
//! explicitly parent-built static fixture and reviewed bwrap, never a mock
//! runner. These cases establish protected-leaf behavior, not cold bootstrap.
//! Deadline probes intercept the declared sandbox shell before it executes;
//! hashing the large test harness would consume the short notification budget.
use std::os::fd::AsRawFd;
use std::os::fd::FromRawFd;
use std::os::fd::OwnedFd;

use super::*;
use crate::source_built_fixed_point::ProofHermeticityMode;
use crate::source_built_fixed_point::SourceAuthorityInput;
use crate::source_built_fixed_point::SourceAuthorityRole;
use crate::source_built_fixed_point::SourceBuiltFixedPointPolicies;
use crate::source_built_fixed_point::SourceBuiltFixedPointResourceBounds;
use crate::source_built_fixed_point::SourceContentKind;
use crate::source_built_fixed_point_action_trust::ActionObservationState;
use crate::source_built_fixed_point_planning::InitialOutputAuthorityState;
use crate::source_built_fixed_point_planning::SourceBuiltFixedPointPlanInput;
use crate::source_built_fixed_point_planning::plan_source_built_fixed_point;

static PROTECTED_TEST_GATE: parking_lot::Mutex<()> = parking_lot::Mutex::new(());
static HOST_EXECUTABLE_DIGEST: std::sync::LazyLock<String> =
    std::sync::LazyLock::new(|| digest(&std::env::current_exe().unwrap()).unwrap().1);

#[test]
fn protected_listener_teardown_cancels_already_blocked_receive() {
    let _test_gate = PROTECTED_TEST_GATE.lock();
    std::thread::spawn(|| {
        let mut pump = NotificationPump::start().unwrap();
        let pid = pump.pid;
        let listener = unsafe { OwnedFd::from_raw_fd(install_filter(&filter_instructions().unwrap()).unwrap()) };
        send_listener(pump.socket.as_raw_fd(), listener.as_raw_fd()).unwrap();
        let deadline = Instant::now() + Duration::from_secs(2);
        loop {
            let syscall = fs::read_to_string(format!("/proc/{pid}/syscall")).unwrap();
            if syscall.starts_with(&format!("{} ", libc::SYS_ioctl)) {
                break;
            }
            assert!(Instant::now() < deadline, "pump never entered NOTIF_RECV: {syscall}");
            std::thread::sleep(Duration::from_millis(2));
        }
        // No tracee notification exists, so the ioctl is already asleep. Closing
        // the other listener reference must not be the mechanism that wakes it.
        drop(listener);
        let start = Instant::now();
        pump.finish().unwrap();
        assert!(start.elapsed() < Duration::from_secs(3));
        assert_eq!(unsafe { libc::waitpid(pid, std::ptr::null_mut(), libc::WNOHANG) }, -1);
        assert_eq!(std::io::Error::last_os_error().raw_os_error(), Some(libc::ECHILD));
    })
    .join()
    .unwrap();
}

#[test]
fn protected_initial_exec_decision_has_a_deadline() {
    let _test_gate = PROTECTED_TEST_GATE.lock();
    let executable = fs::canonicalize(env!("SNIX_BUILD_SANDBOX_SHELL")).unwrap();
    let expected = vec![(executable.clone(), digest(&executable).unwrap().1)];
    let (send, receive) = std::sync::mpsc::sync_channel(1);
    let start = Instant::now();
    let worker = std::thread::spawn(move || {
        protected_launch(Command::new(executable), expected, Duration::from_millis(250), 4096, send, None)
    });
    let pending = loop {
        let pending = receive.recv_timeout(Duration::from_secs(2)).unwrap();
        assert!(pending.row.allowed);
        if pending.row.is_exec {
            break pending;
        }
        pending.decision.send(true).unwrap();
    };
    // The child has forked, but exec has not closed its CLOEXEC descriptors.
    // Keep the decision channel alive to exercise listener-owned cancellation.
    assert!(worker.join().unwrap().is_err());
    assert!(start.elapsed() < Duration::from_secs(6));
    drop(pending);
}

#[test]
fn clone3_never_grants_authority_from_mutable_flags() {
    let thread_flags = (libc::CLONE_THREAD | libc::CLONE_VM | libc::CLONE_SIGHAND) as u64;
    let mut notification: libc::seccomp_notif = unsafe { std::mem::zeroed() };
    notification.pid = unsafe { libc::getpid() } as u32;
    notification.data.nr = libc::SYS_clone3 as i32;
    notification.data.args[0] = (&thread_flags as *const u64) as u64;
    notification.data.args[1] = 88;
    assert!(!thread_clone(&notification));
    notification.data.nr = libc::SYS_clone as i32;
    notification.data.args[0] = thread_flags;
    assert!(thread_clone(&notification));
    notification.data.args[0] &= !(libc::CLONE_THREAD as u64);
    assert!(!thread_clone(&notification));
}

#[test]
fn protected_parallel_launch_timeout_does_not_cancel_another_listener() {
    let _test_gate = PROTECTED_TEST_GATE.lock();
    let executable = fs::canonicalize(env!("SNIX_BUILD_SANDBOX_SHELL")).unwrap();
    let expected = vec![(executable.clone(), digest(&executable).unwrap().1)];
    let (first_send, first_receive) = std::sync::mpsc::sync_channel(1);
    let first_executable = executable.clone();
    let first_expected = expected.clone();
    let first = std::thread::spawn(move || {
        protected_launch(Command::new(first_executable), first_expected, Duration::from_secs(8), 4096, first_send, None)
    });
    let pending = loop {
        let pending = first_receive.recv_timeout(Duration::from_secs(3)).unwrap();
        if pending.row.is_exec {
            break pending;
        }
        pending.decision.send(true).unwrap();
    };
    let (second_send, second_receive) = std::sync::mpsc::sync_channel(1);
    let started = Instant::now();
    let second = std::thread::spawn(move || {
        protected_launch(Command::new(executable), expected, Duration::from_secs(1), 4096, second_send, None)
    });
    let second_pending = second_receive.recv_timeout(Duration::from_secs(2)).unwrap();
    let second_result = second.join().unwrap();
    drop(second_pending);
    pending.decision.send(false).unwrap();
    let first_result = first.join().unwrap();
    // Each listener belongs to its tracee. Timing out the second launch must
    // leave the first live until its own durable decision denies execution.
    assert!(second_result.is_err());
    assert!(started.elapsed() < Duration::from_secs(4));
    assert!(pending.row.is_exec);
    assert!(first_result.unwrap().violation);
}

#[test]
fn protected_filter_preserves_legacy_threads_without_clone3() {
    let _test_gate = PROTECTED_TEST_GATE.lock();
    let mut pump = NotificationPump::start().unwrap();
    let socket = std::sync::Arc::clone(&pump.socket);
    let worker = std::thread::spawn(move || {
        let listener = unsafe { OwnedFd::from_raw_fd(install_filter(&filter_instructions().unwrap()).unwrap()) };
        send_listener(socket.as_raw_fd(), listener.as_raw_fd()).unwrap();
        drop(listener);
        for flags in [
            (libc::CLONE_THREAD | libc::CLONE_VM | libc::CLONE_SIGHAND) as u64,
            libc::SIGCHLD as u64,
        ] {
            let mut arguments = [0u64; 11];
            arguments[0] = flags;
            assert_eq!(
                unsafe { libc::syscall(libc::SYS_clone3, arguments.as_ptr(), std::mem::size_of_val(&arguments)) },
                -1
            );
            assert_eq!(std::io::Error::last_os_error().raw_os_error(), Some(libc::ENOSYS));
        }
        assert_eq!(std::thread::spawn(|| 42).join().unwrap(), 42);
    });
    let deadline = Instant::now() + Duration::from_secs(3);
    let mut admitted_threads = 0;
    while !worker.is_finished() {
        assert!(Instant::now() < deadline);
        let mut notification: libc::seccomp_notif = unsafe { std::mem::zeroed() };
        if pump.receive(&mut notification).unwrap() {
            assert_eq!(notification.data.nr as i64, libc::SYS_clone);
            assert!(thread_clone(&notification));
            let response = libc::seccomp_notif_resp {
                id: notification.id,
                val: 0,
                error: 0,
                flags: libc::SECCOMP_USER_NOTIF_FLAG_CONTINUE as u32,
            };
            pump.respond(&response, deadline, None).unwrap();
            admitted_threads += 1;
        }
    }
    worker.join().unwrap();
    pump.finish().unwrap();
    assert_eq!(admitted_threads, 1);
}

fn recipe_case(mode: &str, seconds: u64) -> (PreparedRecipe, PathBuf) {
    let leaf = PathBuf::from(std::env::var_os("MANTLE_PROTECTED_LEAF").expect("absolute parent-built static ELF path"));
    let sandbox = PathBuf::from(std::env::var_os("MANTLE_PROTECTED_BWRAP").expect("absolute reviewed bwrap path"));
    let evidence =
        PathBuf::from(std::env::var_os("MANTLE_PROTECTED_SMOKE_ROOT").expect("fresh absolute evidence directory"));
    assert!(leaf.is_absolute() && sandbox.is_absolute() && evidence.is_absolute());
    fs::create_dir_all(&evidence).unwrap();
    let root = evidence.join(mode);
    fs::create_dir(&root).expect("case evidence directory must be fresh");
    let stdout_ref = ArtifactRef::Produced {
        action_id: "stagex-transition/leaf".into(),
        output_id: "stdout".into(),
    };
    let stderr_ref = ArtifactRef::Produced {
        action_id: "stagex-transition/leaf".into(),
        output_id: "stderr".into(),
    };
    let mut recipe = SourceRecipe {
        schema: SCHEMA.into(),
        stage_id: "stagex-transition".into(),
        timeout_groups: Vec::new(),
        preparation: RecipePreparation {
            tool: "mantle-protected-leaf-regression".into(),
            provenance: "parent-built libc-free test seed; protected-leaf smoke only, not bootstrap evidence".into(),
        },
        steps: vec![RecipeStep {
            id: "leaf".into(),
            inputs: BTreeMap::from([("seed".into(), ArtifactRef::Source {
                role: SourceAuthorityRole::StagexSeed,
                path: String::new(),
            })]),
            outputs: {
                let mut outputs = BTreeMap::from([
                    ("stdout".into(), RecipeOutput {
                        kind: ArtifactKind::Text,
                        bytes_max: 4096,
                    }),
                    ("stderr".into(), RecipeOutput {
                        kind: ArtifactKind::Text,
                        bytes_max: 4096,
                    }),
                ]);
                if mode == "workspace" {
                    outputs.insert("generated".into(), RecipeOutput {
                        kind: ArtifactKind::Tree,
                        bytes_max: 4096,
                    });
                    outputs.insert("other".into(), RecipeOutput {
                        kind: ArtifactKind::Tree,
                        bytes_max: 4096,
                    });
                }
                if mode.starts_with("probe") || mode.starts_with("guard") {
                    outputs.insert("exit-status".into(), RecipeOutput {
                        kind: ArtifactKind::Text,
                        bytes_max: 4,
                    });
                }
                if mode == "guard1" {
                    outputs.insert("absent-object".into(), RecipeOutput {
                        kind: ArtifactKind::Object,
                        bytes_max: 4096,
                    });
                }
                if mode.starts_with("invocation")
                    || mode == "operands"
                    || mode.starts_with("cwd")
                    || mode.starts_with("temporary")
                    || mode.starts_with("image")
                {
                    outputs.insert("executable".into(), RecipeOutput {
                        kind: ArtifactKind::Executable,
                        bytes_max: 4 * 1024 * 1024,
                    });
                }
                outputs
            },
            operation: RecipeOperation::Exec {
                executable_input: "seed".into(),
                arguments: vec![RecipeWord {
                    parts: vec![WordPart::Literal {
                        text: if mode == "guard0" {
                            "probe0".into()
                        } else if mode.starts_with("guard") {
                            "probe1".into()
                        } else if mode.starts_with("invocation")
                            || mode == "operands"
                            || mode.starts_with("cwd")
                            || mode.starts_with("temporary")
                            || mode.starts_with("image")
                        {
                            "request".into()
                        } else {
                            mode.into()
                        },
                    }],
                }],
                environment: BTreeMap::new(),
                runtime_mounts: BTreeMap::new(),
                cwd_input: None,
                cwd_output: (mode == "workspace").then(|| "generated".into()),
                stdin_input: None,
                merge_stderr: mode == "merged",
                invocation_input: None,
                exit_policy: if mode.starts_with("probe") || mode.starts_with("guard") {
                    RecipeExitPolicy::OneOf { codes: vec![0, 1] }
                } else {
                    RecipeExitPolicy::Exact { code: 0 }
                },
                guard: None,
                output_seeds: BTreeMap::new(),
                command_state: None,
            },
        }],
        roots: vec![stdout_ref.clone(), stderr_ref],
        exports: BTreeMap::from([("smoke-output".into(), stdout_ref)]),
        semantic_roots: BTreeMap::new(),
    };
    if mode.starts_with("guard") {
        let prior = ArtifactRef::Produced {
            action_id: "stagex-transition/initial-state".into(),
            output_id: "state".into(),
        };
        let seed = recipe.steps[0].inputs["seed"].clone();
        recipe.steps.splice(0..0, [
            RecipeStep {
                id: "initial-text".into(),
                inputs: BTreeMap::new(),
                outputs: BTreeMap::from([("text".into(), RecipeOutput {
                    kind: ArtifactKind::Text,
                    bytes_max: 64,
                })]),
                operation: RecipeOperation::WriteText {
                    output: "text".into(),
                    text: "immutable prior state\n".into(),
                },
            },
            RecipeStep {
                id: "initial-state".into(),
                inputs: BTreeMap::from([("text".into(), ArtifactRef::Produced {
                    action_id: "stagex-transition/initial-text".into(),
                    output_id: "text".into(),
                })]),
                outputs: BTreeMap::from([("state".into(), RecipeOutput {
                    kind: ArtifactKind::Tree,
                    bytes_max: 4096,
                })]),
                operation: RecipeOperation::AssembleTree {
                    output: "state".into(),
                    entries: BTreeMap::from([("prior.txt".into(), "text".into())]),
                },
            },
        ]);
        recipe.steps.push(RecipeStep {
            id: "conditional".into(),
            inputs: BTreeMap::from([
                ("seed".into(), seed),
                ("prior".into(), prior.clone()),
                ("status".into(), ArtifactRef::Produced {
                    action_id: "stagex-transition/leaf".into(),
                    output_id: "exit-status".into(),
                }),
            ]),
            outputs: BTreeMap::from([
                ("stdout".into(), RecipeOutput {
                    kind: ArtifactKind::Text,
                    bytes_max: 4096,
                }),
                ("stderr".into(), RecipeOutput {
                    kind: ArtifactKind::Text,
                    bytes_max: 4096,
                }),
                ("exit-status".into(), RecipeOutput {
                    kind: ArtifactKind::Text,
                    bytes_max: 4,
                }),
                ("generated".into(), RecipeOutput {
                    kind: ArtifactKind::Tree,
                    bytes_max: 4096,
                }),
                ("other".into(), RecipeOutput {
                    kind: ArtifactKind::Tree,
                    bytes_max: 4096,
                }),
            ]),
            operation: RecipeOperation::Exec {
                command_state: None,
                executable_input: "seed".into(),
                arguments: vec![RecipeWord {
                    parts: vec![WordPart::Literal {
                        text: "workspace".into(),
                    }],
                }],
                environment: BTreeMap::new(),
                runtime_mounts: BTreeMap::new(),
                cwd_input: None,
                cwd_output: Some("generated".into()),
                stdin_input: None,
                merge_stderr: false,
                invocation_input: None,
                exit_policy: RecipeExitPolicy::Exact { code: 0 },
                guard: Some(RecipeGuard {
                    conditions: vec![RecipeGuardCondition {
                        status_input: "status".into(),
                        accepted_codes: vec![0],
                    }],
                }),
                output_seeds: BTreeMap::from([("generated".into(), "prior".into())]),
            },
        });
        recipe.steps.push(RecipeStep {
            id: "conditional-next".into(),
            inputs: BTreeMap::from([
                ("seed".into(), ArtifactRef::Source {
                    role: SourceAuthorityRole::StagexSeed,
                    path: String::new(),
                }),
                ("status".into(), ArtifactRef::Produced {
                    action_id: "stagex-transition/leaf".into(),
                    output_id: "exit-status".into(),
                }),
                ("prior-status".into(), ArtifactRef::Produced {
                    action_id: "stagex-transition/conditional".into(),
                    output_id: "exit-status".into(),
                }),
            ]),
            outputs: BTreeMap::from([
                ("stdout".into(), RecipeOutput {
                    kind: ArtifactKind::Text,
                    bytes_max: 4096,
                }),
                ("stderr".into(), RecipeOutput {
                    kind: ArtifactKind::Text,
                    bytes_max: 4096,
                }),
                ("exit-status".into(), RecipeOutput {
                    kind: ArtifactKind::Text,
                    bytes_max: 4,
                }),
            ]),
            operation: RecipeOperation::Exec {
                command_state: None,
                executable_input: "seed".into(),
                arguments: vec![RecipeWord {
                    parts: vec![WordPart::Literal { text: "probe0".into() }],
                }],
                environment: BTreeMap::new(),
                runtime_mounts: BTreeMap::new(),
                cwd_input: None,
                cwd_output: None,
                stdin_input: None,
                merge_stderr: false,
                invocation_input: None,
                exit_policy: RecipeExitPolicy::Exact { code: 0 },
                guard: Some(RecipeGuard {
                    conditions: vec![
                        RecipeGuardCondition {
                            status_input: "status".into(),
                            accepted_codes: vec![0],
                        },
                        RecipeGuardCondition {
                            status_input: "prior-status".into(),
                            accepted_codes: vec![0],
                        },
                    ],
                }),
                output_seeds: BTreeMap::new(),
            },
        });
        recipe.steps.push(RecipeStep {
            id: "observe-next".into(),
            inputs: BTreeMap::new(),
            outputs: BTreeMap::from([("envelope".into(), RecipeOutput {
                kind: ArtifactKind::Tree,
                bytes_max: 32768,
            })]),
            operation: RecipeOperation::ObservePhase {
                phase: "conditional-next".into(),
                output: "envelope".into(),
                state: None,
            },
        });
        recipe.roots.push(ArtifactRef::Produced {
            action_id: "stagex-transition/observe-next".into(),
            output_id: "envelope".into(),
        });
        recipe.steps.push(RecipeStep {
            id: "observe".into(),
            inputs: BTreeMap::from([("prior".into(), prior)]),
            outputs: BTreeMap::from([("envelope".into(), RecipeOutput {
                kind: ArtifactKind::Tree,
                bytes_max: 32768,
            })]),
            operation: RecipeOperation::ObservePhase {
                phase: "conditional".into(),
                output: "envelope".into(),
                state: Some(RecipePhaseState {
                    output: "generated".into(),
                    skipped_input: "prior".into(),
                    skipped_path: String::new(),
                }),
            },
        });
        recipe.roots.push(ArtifactRef::Produced {
            action_id: "stagex-transition/observe".into(),
            output_id: "envelope".into(),
        });
    }
    if mode.starts_with("invocation") {
        recipe.steps.push(RecipeStep {
            id: "invoked".into(),
            inputs: BTreeMap::from([
                ("program".into(), ArtifactRef::Produced {
                    action_id: "stagex-transition/leaf".into(),
                    output_id: "executable".into(),
                }),
                ("request".into(), ArtifactRef::Produced {
                    action_id: "stagex-transition/leaf".into(),
                    output_id: "stdout".into(),
                }),
            ]),
            outputs: BTreeMap::from([
                ("stdout".into(), RecipeOutput {
                    kind: ArtifactKind::Text,
                    bytes_max: 4096,
                }),
                ("stderr".into(), RecipeOutput {
                    kind: ArtifactKind::Text,
                    bytes_max: 4096,
                }),
                ("exit-status".into(), RecipeOutput {
                    kind: ArtifactKind::Text,
                    bytes_max: 4,
                }),
            ]),
            operation: RecipeOperation::Exec {
                command_state: None,
                executable_input: "program".into(),
                arguments: Vec::new(),
                environment: BTreeMap::new(),
                runtime_mounts: BTreeMap::new(),
                cwd_input: None,
                cwd_output: None,
                stdin_input: None,
                merge_stderr: false,
                invocation_input: Some("request".into()),
                exit_policy: RecipeExitPolicy::Exact { code: 0 },
                guard: None,
                output_seeds: BTreeMap::new(),
            },
        });
        recipe.roots.push(ArtifactRef::Produced {
            action_id: "stagex-transition/invoked".into(),
            output_id: "stdout".into(),
        });
    }
    if mode == "operands" {
        let repository = "https://example.invalid/pkg";
        let license = "MIT/Apache-2.0";
        let frame = format!("repo={repository}\nlicense={license}\n");
        let metadata = ArtifactRef::Produced {
            action_id: "stagex-transition/metadata".into(),
            output_id: "text".into(),
        };
        let license_ref = ArtifactRef::Produced {
            action_id: "stagex-transition/license".into(),
            output_id: "text".into(),
        };
        let runtime = ArtifactRef::Produced {
            action_id: "stagex-transition/runtime-tree".into(),
            output_id: "tree".into(),
        };
        let repository_word = RecipeWord {
            parts: vec![WordPart::TextSlice {
                name: "metadata".into(),
                offset_bytes: 5,
                length_bytes: repository.len() as u64,
            }],
        };
        let invocation = RecipeInvocation {
            readonly_input_roots: Vec::new(),
            arguments: vec![
                RecipeWord {
                    parts: vec![WordPart::Literal { text: "data".into() }],
                },
                repository_word.clone(),
                RecipeWord {
                    parts: vec![WordPart::Runtime {
                        mount: "/mantle-rust-runtime".into(),
                        path: "metadata.txt".into(),
                    }],
                },
            ],
            environment: BTreeMap::from([
                ("CARGO_PKG_REPOSITORY".into(), repository_word),
                ("CARGO_PKG_LICENSE".into(), RecipeWord {
                    parts: vec![WordPart::TextInput { name: "license".into() }],
                }),
                ("LD_LIBRARY_PATH".into(), RecipeWord {
                    parts: vec![WordPart::Runtime {
                        mount: "/mantle-rust-runtime".into(),
                        path: String::new(),
                    }],
                }),
            ]),
            cwd: None,
            argv0: None,
            executable_path: None,
            workspace_mounts: BTreeMap::new(),
            process_attributes: None,
            descriptors: None,
        };
        for (id, text) in [
            ("metadata", frame),
            ("license", license.into()),
            ("operand-request", serde_json::to_string(&invocation).unwrap()),
        ] {
            recipe.steps.push(RecipeStep {
                id: id.into(),
                inputs: BTreeMap::new(),
                outputs: BTreeMap::from([("text".into(), RecipeOutput {
                    kind: ArtifactKind::Text,
                    bytes_max: 4096,
                })]),
                operation: RecipeOperation::WriteText {
                    output: "text".into(),
                    text,
                },
            });
        }
        recipe.steps.push(RecipeStep {
            id: "runtime-tree".into(),
            inputs: BTreeMap::from([("metadata".into(), metadata.clone())]),
            outputs: BTreeMap::from([("tree".into(), RecipeOutput {
                kind: ArtifactKind::Tree,
                bytes_max: 4096,
            })]),
            operation: RecipeOperation::AssembleTree {
                output: "tree".into(),
                entries: BTreeMap::from([("metadata.txt".into(), "metadata".into())]),
            },
        });
        recipe.steps.push(RecipeStep {
            id: "operands".into(),
            inputs: BTreeMap::from([
                ("program".into(), ArtifactRef::Produced {
                    action_id: "stagex-transition/leaf".into(),
                    output_id: "executable".into(),
                }),
                ("request".into(), ArtifactRef::Produced {
                    action_id: "stagex-transition/operand-request".into(),
                    output_id: "text".into(),
                }),
                ("metadata".into(), metadata),
                ("license".into(), license_ref),
                ("runtime".into(), runtime),
            ]),
            outputs: BTreeMap::from([
                ("stdout".into(), RecipeOutput {
                    kind: ArtifactKind::Text,
                    bytes_max: 4096,
                }),
                ("stderr".into(), RecipeOutput {
                    kind: ArtifactKind::Text,
                    bytes_max: 4096,
                }),
            ]),
            operation: RecipeOperation::Exec {
                command_state: None,
                executable_input: "program".into(),
                arguments: Vec::new(),
                environment: BTreeMap::new(),
                runtime_mounts: BTreeMap::from([("/mantle-rust-runtime".into(), "runtime".into())]),
                cwd_input: None,
                cwd_output: None,
                stdin_input: None,
                merge_stderr: false,
                invocation_input: Some("request".into()),
                exit_policy: RecipeExitPolicy::Exact { code: 0 },
                guard: None,
                output_seeds: BTreeMap::new(),
            },
        });
        recipe.roots.push(ArtifactRef::Produced {
            action_id: "stagex-transition/operands".into(),
            output_id: "stdout".into(),
        });
    }
    if mode.starts_with("cwd") || mode.starts_with("image") {
        let metadata = ArtifactRef::Produced {
            action_id: "stagex-transition/cwd-metadata".into(),
            output_id: "text".into(),
        };
        let workspace = ArtifactRef::Produced {
            action_id: "stagex-transition/cwd-workspace".into(),
            output_id: "tree".into(),
        };
        let image_case = mode.starts_with("image");
        let invocation = RecipeInvocation {
            readonly_input_roots: Vec::new(),
            arguments: vec![RecipeWord {
                parts: vec![WordPart::Literal {
                    text: if image_case {
                        "image-context".into()
                    } else if mode == "cwd-output" {
                        "local-output".into()
                    } else {
                        "local".into()
                    },
                }],
            }],
            environment: BTreeMap::new(),
            cwd: Some(RecipeWord {
                parts: vec![if mode == "cwd-output" {
                    WordPart::Output {
                        name: "generated".into(),
                        path: "package".into(),
                    }
                } else {
                    WordPart::Input {
                        name: if mode == "cwd-unknown" {
                            "missing".into()
                        } else {
                            "workspace".into()
                        },
                        path: if mode == "cwd-escape" {
                            "../outside".into()
                        } else {
                            "package".into()
                        },
                    }
                }],
            }),
            argv0: None,
            executable_path: image_case.then(|| RecipeWord {
                parts: vec![WordPart::Input {
                    name: "installation".into(),
                    path: "bin/fixture-image".into(),
                }],
            }),
            workspace_mounts: BTreeMap::new(),
            process_attributes: None,
            descriptors: None,
        };
        let mut frame = serde_json::to_string(&invocation).unwrap();
        if mode == "cwd-noncanonical" {
            frame.insert(0, ' ');
        }
        for (id, text) in [("cwd-metadata", "actual-cwd-source\n".into()), ("cwd-request", frame)] {
            recipe.steps.push(RecipeStep {
                id: id.into(),
                inputs: BTreeMap::new(),
                outputs: BTreeMap::from([("text".into(), RecipeOutput {
                    kind: ArtifactKind::Text,
                    bytes_max: 4096,
                })]),
                operation: RecipeOperation::WriteText {
                    output: "text".into(),
                    text,
                },
            });
        }
        recipe.steps.push(RecipeStep {
            id: "cwd-workspace".into(),
            inputs: BTreeMap::from([("metadata".into(), metadata)]),
            outputs: BTreeMap::from([("tree".into(), RecipeOutput {
                kind: ArtifactKind::Tree,
                bytes_max: 4096,
            })]),
            operation: RecipeOperation::AssembleTree {
                output: "tree".into(),
                entries: BTreeMap::from([("package/metadata.txt".into(), "metadata".into())]),
            },
        });
        let program = ArtifactRef::Produced {
            action_id: "stagex-transition/leaf".into(),
            output_id: "executable".into(),
        };
        let mut consumer_inputs = BTreeMap::from([
            ("program".into(), program.clone()),
            ("request".into(), ArtifactRef::Produced {
                action_id: "stagex-transition/cwd-request".into(),
                output_id: "text".into(),
            }),
            ("workspace".into(), workspace.clone()),
        ]);
        if image_case {
            let installation_source = if mode == "image-mismatch" {
                ArtifactRef::Produced {
                    action_id: "stagex-transition/cwd-metadata".into(),
                    output_id: "text".into(),
                }
            } else {
                program
            };
            recipe.steps.push(RecipeStep {
                id: "image-installation".into(),
                inputs: BTreeMap::from([("image".into(), installation_source)]),
                outputs: BTreeMap::from([("tree".into(), RecipeOutput {
                    kind: ArtifactKind::Tree,
                    bytes_max: 4 * 1024 * 1024,
                })]),
                operation: RecipeOperation::AssembleTree {
                    output: "tree".into(),
                    entries: BTreeMap::from([("bin/fixture-image".into(), "image".into())]),
                },
            });
            consumer_inputs.insert("installation".into(), ArtifactRef::Produced {
                action_id: "stagex-transition/image-installation".into(),
                output_id: "tree".into(),
            });
        }
        recipe.steps.push(RecipeStep {
            id: "cwd-consumer".into(),
            inputs: consumer_inputs,
            outputs: BTreeMap::from([
                ("stdout".into(), RecipeOutput {
                    kind: ArtifactKind::Text,
                    bytes_max: 4096,
                }),
                ("stderr".into(), RecipeOutput {
                    kind: ArtifactKind::Text,
                    bytes_max: 4096,
                }),
                ("generated".into(), RecipeOutput {
                    kind: ArtifactKind::Tree,
                    bytes_max: 4096,
                }),
            ]),
            operation: RecipeOperation::Exec {
                command_state: None,
                executable_input: "program".into(),
                arguments: Vec::new(),
                environment: BTreeMap::new(),
                runtime_mounts: BTreeMap::new(),
                cwd_input: (mode == "cwd-conflict").then(|| "workspace".into()),
                cwd_output: None,
                stdin_input: None,
                merge_stderr: false,
                invocation_input: Some("request".into()),
                exit_policy: RecipeExitPolicy::Exact { code: 0 },
                guard: None,
                output_seeds: BTreeMap::from([("generated".into(), "workspace".into())]),
            },
        });
        recipe.roots.push(ArtifactRef::Produced {
            action_id: "stagex-transition/cwd-consumer".into(),
            output_id: "stdout".into(),
        });
    }
    if mode.starts_with("temporary") {
        let invocation = RecipeInvocation {
            readonly_input_roots: Vec::new(),
            arguments: vec![RecipeWord {
                parts: vec![WordPart::Literal {
                    text: "temporary".into(),
                }],
            }],
            environment: BTreeMap::new(),
            cwd: None,
            argv0: None,
            executable_path: None,
            workspace_mounts: BTreeMap::from([("/tmp".into(), RecipeWorkspaceMount {
                output: "generated".into(),
                path: if mode == "temporary-missing" {
                    "removed".into()
                } else {
                    String::new()
                },
            })]),
            process_attributes: None,
            descriptors: None,
        };
        recipe.steps.push(RecipeStep {
            id: "temporary-request".into(),
            inputs: BTreeMap::new(),
            outputs: BTreeMap::from([("text".into(), RecipeOutput {
                kind: ArtifactKind::Text,
                bytes_max: 4096,
            })]),
            operation: RecipeOperation::WriteText {
                output: "text".into(),
                text: serde_json::to_string(&invocation).unwrap(),
            },
        });
        recipe.steps.push(RecipeStep {
            id: "temporary-consumer".into(),
            inputs: BTreeMap::from([
                ("program".into(), ArtifactRef::Produced {
                    action_id: "stagex-transition/leaf".into(),
                    output_id: "executable".into(),
                }),
                ("request".into(), ArtifactRef::Produced {
                    action_id: "stagex-transition/temporary-request".into(),
                    output_id: "text".into(),
                }),
            ]),
            outputs: BTreeMap::from([
                ("stdout".into(), RecipeOutput {
                    kind: ArtifactKind::Text,
                    bytes_max: 4096,
                }),
                ("stderr".into(), RecipeOutput {
                    kind: ArtifactKind::Text,
                    bytes_max: 4096,
                }),
                ("generated".into(), RecipeOutput {
                    kind: ArtifactKind::Tree,
                    bytes_max: 4096,
                }),
            ]),
            operation: RecipeOperation::Exec {
                command_state: None,
                executable_input: "program".into(),
                arguments: Vec::new(),
                environment: BTreeMap::new(),
                runtime_mounts: BTreeMap::new(),
                cwd_input: None,
                cwd_output: None,
                stdin_input: None,
                merge_stderr: false,
                invocation_input: Some("request".into()),
                exit_policy: RecipeExitPolicy::Exact { code: 0 },
                guard: None,
                output_seeds: BTreeMap::new(),
            },
        });
        recipe.roots.push(ArtifactRef::Produced {
            action_id: "stagex-transition/temporary-consumer".into(),
            output_id: "stdout".into(),
        });
    }
    configure_observation_case(&mut recipe, mode);
    if mode == "fdtable" {
        append_descriptor_case(&mut recipe);
    }
    if mode == "argv0" {
        append_argv0_case(&mut recipe);
    }
    if mode.starts_with("umask-") {
        append_umask_case(&mut recipe, if mode == "umask-owner" { 0o077 } else { 0o027 });
    }
    let lineage = root.join("lineage.json");
    fs::write(&lineage, serde_json::to_vec(&recipe).unwrap()).unwrap();
    let mut roots = BTreeMap::from([
        (SourceAuthorityRole::StagexSeed, leaf),
        (SourceAuthorityRole::StagexLineage, lineage),
    ]);
    // Other roles are authenticated inert data, never executable authority or
    // claimed full recipes. The production planner still requires all roles.
    for role in [
        SourceAuthorityRole::StagexSourceBundle,
        SourceAuthorityRole::NativeSourceBundle,
    ] {
        let path = root.join(format!("{role:?}.source"));
        fs::write(&path, b"inert smoke-only source data\n").unwrap();
        roots.insert(role, path);
    }
    for role in [
        SourceAuthorityRole::RustSourceArchiveSet,
        SourceAuthorityRole::MantleSource,
        SourceAuthorityRole::VendorInputs,
    ] {
        let path = root.join(format!("{role:?}.sources"));
        fs::create_dir(&path).unwrap();
        fs::write(path.join("README"), b"inert smoke-only source data\n").unwrap();
        roots.insert(role, path);
    }
    let sources = roots
        .iter()
        .map(|(role, path)| {
            let (size_bytes, digest_blake3) = digest(path).unwrap();
            SourceAuthorityInput {
                id: format!("smoke-{role:?}"),
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
    // Policy identities bind the explicit smoke-only policy labels; no host
    // discovery, runtime loaders, fallback, provider cache or live fetch.
    let policy = |label: &str| blake3::hash(label.as_bytes()).to_hex().to_string();
    let proof = plan_source_built_fixed_point(SourceBuiltFixedPointPlanInput {
        proof_id: format!("protected-leaf-{mode}"),
        logical_store_prefix: "/mantle/store".into(),
        source_inputs: sources,
        initial_output_authority: InitialOutputAuthorityState {
            stagex_transition_entries: 0,
            native_provider_entries: 0,
            rust_provider_entries: 0,
            mantle_output_entries: 0,
        },
        policies: SourceBuiltFixedPointPolicies {
            expected_native_provider_digest_blake3: policy("no native provider claimed by this smoke"),
            closure_policy_digest_blake3: policy("smoke exact declared closure"),
            hermeticity_policy_digest_blake3: policy("smoke strict namespaces and explicit mounts"),
            protected_execution_policy_digest_blake3: policy("smoke exact bwrap and StagexSeed only"),
            effect_policy_digest_blake3: policy("smoke stdout and stderr only"),
            normalization_policy_digest_blake3: policy("smoke exact output bytes"),
            hermeticity_mode: ProofHermeticityMode::Strict,
            live_fetch_allowed: false,
            cargo_invocation_allowed: false,
            ambient_discovery_allowed: false,
            fallback_allowed: false,
            provider_cache_completion_allowed: false,
        },
        resource_bounds: SourceBuiltFixedPointResourceBounds {
            elapsed_seconds_max: seconds,
            disk_bytes_max: 16 * 1024 * 1024,
            open_file_descriptors_max: 1024,
            protected_exec_events_max: 64,
            source_records_max: 64,
        },
    })
    .unwrap();
    let runtime = root.join("runtime");
    let prepared = crate::source_built_recipe::prepare(
        &proof,
        "stagex-transition",
        SourceAuthorityRole::StagexLineage,
        "",
        &RecipePaths {
            source_roots: &roots,
            runtime_root: &runtime,
            sandbox_binary: &sandbox,
        },
        &[],
        HOST_EXECUTABLE_DIGEST.as_str(),
    )
    .unwrap();
    fs::write(root.join("proof.json"), serde_json::to_vec_pretty(&proof).unwrap()).unwrap();
    (prepared, root)
}

fn smoke_text(recipe: &mut SourceRecipe, id: &str, text: &str) -> ArtifactRef {
    recipe.steps.push(RecipeStep {
        id: id.into(),
        inputs: BTreeMap::new(),
        outputs: BTreeMap::from([("text".into(), RecipeOutput {
            kind: ArtifactKind::Text,
            bytes_max: 4096,
        })]),
        operation: RecipeOperation::WriteText {
            output: "text".into(),
            text: text.into(),
        },
    });
    ArtifactRef::Produced {
        action_id: format!("stagex-transition/{id}"),
        output_id: "text".into(),
    }
}

fn configure_observation_case(recipe: &mut SourceRecipe, mode: &str) {
    if mode == "normal" {
        let leaf = &mut recipe.steps[0];
        leaf.outputs.insert("exit-status".into(), RecipeOutput {
            kind: ArtifactKind::Text,
            bytes_max: 4,
        });
        let RecipeOperation::Exec { exit_policy, .. } = &mut leaf.operation else {
            unreachable!()
        };
        *exit_policy = RecipeExitPolicy::Normal;
        recipe.roots.push(ArtifactRef::Produced {
            action_id: "stagex-transition/leaf".into(),
            output_id: "exit-status".into(),
        });
    }
    if mode != "observed" {
        return;
    }
    let input = smoke_text(recipe, "observed-input", "abcdef");
    let text = recipe.steps.pop().unwrap();
    recipe.steps.insert(0, text);
    let leaf = &mut recipe.steps[1];
    leaf.inputs.insert("stdin".into(), input);
    for (name, kind) in [
        ("workspace", ArtifactKind::Tree),
        ("times", ArtifactKind::Text),
        ("descriptors", ArtifactKind::Text),
    ] {
        leaf.outputs.insert(name.into(), RecipeOutput { kind, bytes_max: 4096 });
        recipe.roots.push(ArtifactRef::Produced {
            action_id: "stagex-transition/leaf".into(),
            output_id: name.into(),
        });
    }
    let RecipeOperation::Exec {
        stdin_input,
        cwd_output,
        command_state,
        ..
    } = &mut leaf.operation
    else {
        unreachable!()
    };
    *stdin_input = Some("stdin".into());
    *cwd_output = Some("workspace".into());
    *command_state = Some(RecipeCommandState {
        workspace_output: "workspace".into(),
        prior_times_input: None,
        times_output: "times".into(),
        descriptors_output: "descriptors".into(),
        descriptor_files_output: None,
    });
}

fn append_descriptor_case(recipe: &mut SourceRecipe) {
    let leaf = &mut recipe.steps[0];
    leaf.outputs.insert("executable".into(), RecipeOutput {
        kind: ArtifactKind::Executable,
        bytes_max: 4 * 1024 * 1024,
    });
    let RecipeOperation::Exec { arguments, .. } = &mut leaf.operation else {
        unreachable!()
    };
    *arguments = vec![RecipeWord {
        parts: vec![WordPart::Literal { text: "request".into() }],
    }];
    let mut entries = BTreeMap::new();
    let mut inputs = BTreeMap::new();
    for (id, bytes) in [("0", "abcdef"), ("1", "seed"), ("5", "prior")] {
        let reference = smoke_text(recipe, &format!("fd-{id}"), bytes);
        inputs.insert(format!("fd-{id}"), reference);
        entries.insert(id.into(), format!("fd-{id}"));
    }
    recipe.steps.push(RecipeStep {
        id: "fd-seed".into(),
        inputs,
        outputs: BTreeMap::from([("tree".into(), RecipeOutput {
            kind: ArtifactKind::Tree,
            bytes_max: 4096,
        })]),
        operation: RecipeOperation::AssembleTree {
            output: "tree".into(),
            entries,
        },
    });
    let mut slots = vec![None; 64];
    for (index, access, flags, offset) in [
        (0, RecipeDescriptorAccess::Read, 0, 0),
        (1, RecipeDescriptorAccess::ReadWrite, 0, 4),
        (5, RecipeDescriptorAccess::ReadWrite, libc::O_APPEND as u32, 1),
    ] {
        slots[index] = Some(RecipeDescriptor {
            target: RecipeDescriptorTarget::File {
                path: RecipeWord {
                    parts: vec![WordPart::Output {
                        name: "descriptor-files".into(),
                        path: index.to_string(),
                    }],
                },
            },
            access,
            flags,
            offset,
            alias: index as u8,
        });
    }
    slots[2] = slots[1].clone();
    slots[9] = slots[0].clone();
    slots[63] = slots[5].clone();
    slots[3] = Some(RecipeDescriptor {
        target: RecipeDescriptorTarget::NullDevice,
        access: RecipeDescriptorAccess::ReadWrite,
        flags: 0,
        offset: 0,
        alias: 3,
    });
    let invocation = RecipeInvocation {
        readonly_input_roots: Vec::new(),
        arguments: vec![RecipeWord {
            parts: vec![WordPart::Literal { text: "backing".into() }],
        }],
        environment: BTreeMap::new(),
        cwd: None,
        argv0: None,
        executable_path: None,
        workspace_mounts: BTreeMap::new(),
        process_attributes: None,
        descriptors: Some(RecipeDescriptorTable { slots }),
    };
    let request = smoke_text(recipe, "fd-request", &serde_json::to_string(&invocation).unwrap());
    recipe.steps.push(RecipeStep {
        id: "fd-command".into(),
        inputs: BTreeMap::from([
            ("program".into(), ArtifactRef::Produced {
                action_id: "stagex-transition/leaf".into(),
                output_id: "executable".into(),
            }),
            ("snapshots".into(), ArtifactRef::Produced {
                action_id: "stagex-transition/fd-seed".into(),
                output_id: "tree".into(),
            }),
            ("request".into(), request),
        ]),
        outputs: [
            ("workspace", ArtifactKind::Tree),
            ("descriptor-files", ArtifactKind::Tree),
            ("stdout", ArtifactKind::Text),
            ("stderr", ArtifactKind::Text),
            ("times", ArtifactKind::Text),
            ("descriptors", ArtifactKind::Text),
        ]
        .into_iter()
        .map(|(name, kind)| (name.into(), RecipeOutput { kind, bytes_max: 4096 }))
        .collect(),
        operation: RecipeOperation::Exec {
            executable_input: "program".into(),
            arguments: Vec::new(),
            environment: BTreeMap::new(),
            runtime_mounts: BTreeMap::new(),
            cwd_input: None,
            cwd_output: None,
            stdin_input: None,
            merge_stderr: false,
            invocation_input: Some("request".into()),
            exit_policy: RecipeExitPolicy::Exact { code: 0 },
            guard: None,
            output_seeds: BTreeMap::from([("descriptor-files".into(), "snapshots".into())]),
            command_state: Some(RecipeCommandState {
                workspace_output: "workspace".into(),
                prior_times_input: None,
                times_output: "times".into(),
                descriptors_output: "descriptors".into(),
                descriptor_files_output: Some("descriptor-files".into()),
            }),
        },
    });
    for output in ["descriptor-files", "descriptors", "times"] {
        recipe.roots.push(ArtifactRef::Produced {
            action_id: "stagex-transition/fd-command".into(),
            output_id: output.into(),
        });
    }
}

fn append_argv0_case(recipe: &mut SourceRecipe) {
    let leaf = &mut recipe.steps[0];
    leaf.outputs.insert("executable".into(), RecipeOutput {
        kind: ArtifactKind::Executable,
        bytes_max: 4 * 1024 * 1024,
    });
    let RecipeOperation::Exec { arguments, .. } = &mut leaf.operation else {
        unreachable!()
    };
    *arguments = vec![RecipeWord {
        parts: vec![WordPart::Literal { text: "request".into() }],
    }];
    let argv0 = smoke_text(recipe, "argv0-data", "/logical/source-program");
    let invocation = RecipeInvocation {
        readonly_input_roots: Vec::new(),
        arguments: vec![RecipeWord {
            parts: vec![WordPart::Literal { text: "verify".into() }],
        }],
        environment: BTreeMap::new(),
        cwd: None,
        argv0: Some(RecipeWord {
            parts: vec![WordPart::TextInput {
                name: "original-argv0".into(),
            }],
        }),
        executable_path: None,
        workspace_mounts: BTreeMap::new(),
        process_attributes: None,
        descriptors: None,
    };
    let request = smoke_text(recipe, "argv0-request", &serde_json::to_string(&invocation).unwrap());
    recipe.steps.push(RecipeStep {
        id: "argv0-command".into(),
        inputs: BTreeMap::from([
            ("program".into(), ArtifactRef::Produced {
                action_id: "stagex-transition/leaf".into(),
                output_id: "executable".into(),
            }),
            ("original-argv0".into(), argv0),
            ("request".into(), request),
        ]),
        outputs: ["stdout", "stderr"]
            .into_iter()
            .map(|name| {
                (name.into(), RecipeOutput {
                    kind: ArtifactKind::Text,
                    bytes_max: 4096,
                })
            })
            .collect(),
        operation: RecipeOperation::Exec {
            executable_input: "program".into(),
            arguments: Vec::new(),
            environment: BTreeMap::new(),
            runtime_mounts: BTreeMap::new(),
            cwd_input: None,
            cwd_output: None,
            stdin_input: None,
            merge_stderr: false,
            invocation_input: Some("request".into()),
            exit_policy: RecipeExitPolicy::Exact { code: 0 },
            guard: None,
            output_seeds: BTreeMap::new(),
            command_state: None,
        },
    });
    recipe.roots.push(ArtifactRef::Produced {
        action_id: "stagex-transition/argv0-command".into(),
        output_id: "stdout".into(),
    });
}

fn append_umask_case(recipe: &mut SourceRecipe, mask: u16) {
    let leaf = &mut recipe.steps[0];
    leaf.outputs.insert("executable".into(), RecipeOutput {
        kind: ArtifactKind::Executable,
        bytes_max: 4 * 1024 * 1024,
    });
    let RecipeOperation::Exec { arguments, .. } = &mut leaf.operation else {
        unreachable!()
    };
    *arguments = vec![RecipeWord {
        parts: vec![WordPart::Literal { text: "request".into() }],
    }];
    let invocation = RecipeInvocation {
        readonly_input_roots: Vec::new(),
        arguments: vec![RecipeWord {
            parts: vec![WordPart::Literal { text: "umask".into() }],
        }],
        environment: BTreeMap::new(),
        cwd: None,
        argv0: None,
        executable_path: None,
        workspace_mounts: BTreeMap::new(),
        process_attributes: Some(crate::source_built_recipe::RecipeProcessAttributes {
            umask: Some(mask),
            ..crate::source_built_recipe::RecipeProcessAttributes::default()
        }),
        descriptors: None,
    };
    let request = smoke_text(recipe, "umask-request", &serde_json::to_string(&invocation).unwrap());
    recipe.steps.push(RecipeStep {
        id: "umask-command".into(),
        inputs: BTreeMap::from([
            ("program".into(), ArtifactRef::Produced {
                action_id: "stagex-transition/leaf".into(),
                output_id: "executable".into(),
            }),
            ("request".into(), request),
        ]),
        outputs: [
            ("stdout", ArtifactKind::Text),
            ("stderr", ArtifactKind::Text),
            ("generated", ArtifactKind::Tree),
        ]
        .into_iter()
        .map(|(name, kind)| (name.into(), RecipeOutput { kind, bytes_max: 4096 }))
        .collect(),
        operation: RecipeOperation::Exec {
            executable_input: "program".into(),
            arguments: Vec::new(),
            environment: BTreeMap::new(),
            runtime_mounts: BTreeMap::new(),
            cwd_input: None,
            cwd_output: None,
            stdin_input: None,
            merge_stderr: false,
            invocation_input: Some("request".into()),
            exit_policy: RecipeExitPolicy::Exact { code: 0 },
            guard: None,
            output_seeds: BTreeMap::new(),
            command_state: None,
        },
    });
    recipe.roots.push(ArtifactRef::Produced {
        action_id: "stagex-transition/umask-command".into(),
        output_id: "generated".into(),
    });
}

fn observation_records(path: &Path) -> Vec<String> {
    let bytes = fs::read_to_string(path).unwrap();
    let mut input = bytes.as_str();
    let mut records = Vec::new();
    while !input.is_empty() {
        let (length, body) = input.split_once(':').unwrap();
        let length = length.parse::<usize>().unwrap();
        let (value, tail) = body.split_at(length);
        records.push(value.to_owned());
        input = tail.strip_prefix(',').unwrap();
    }
    records
}

fn smoke(mode: &str, stdout: &[u8], rejected: bool, cancellation: bool) {
    let _test_gate = PROTECTED_TEST_GATE.lock();
    let (prepared, root) = recipe_case(mode, if cancellation { 1 } else { 20 });
    let mut state = ActionObservationState::default();
    let mut events = Vec::new();
    let mut event_file = create(&root.join("observed-events.jsonl")).unwrap();
    let started = Instant::now();
    let result = crate::source_built_recipe::run(&prepared, &mut |event| {
        journal(&mut event_file, &event)?;
        events.push(event.clone());
        let action = prepared.adapter().actions.iter().find(|action| action.id == event.action_id).unwrap();
        state.record(&prepared.proof, action, &event).map_err(error)
    });
    assert_eq!(fs::read(prepared.runtime_root.join("leaf/outputs/stdout")).unwrap(), stdout);
    assert_eq!(fs::read(prepared.runtime_root.join("leaf/outputs/stderr")).unwrap(), b"leaf-stderr\n");
    let raw = fs::read_to_string(prepared.runtime_root.join("execution.jsonl")).unwrap();
    let rows = raw.lines().map(|line| serde_json::from_str::<serde_json::Value>(line).unwrap()).collect::<Vec<_>>();
    let admitted_execs = rows.iter().filter(|row| row["is_exec"] == true && row["allowed"] == true).count();
    assert_eq!(admitted_execs, 2, "actual bwrap and exact StagexSeed must both execute: {raw}");
    if rejected {
        assert!(result.is_err(), "handled EPERM/exit0 must not promote outputs");
        assert!(rows.iter().all(|row| row.get("actual_outputs").is_none() && row["complete"] != true));
        assert!(events.iter().all(|event| event.produced_outputs.is_empty()));
        let denied = rows.iter().find(|row| row["allowed"] == false).expect("durable denied RawExec");
        assert_eq!(denied["is_exec"], mode == "exec");
        assert!(events.iter().any(|event| event.protected_exec_violation));
        // Re-applying an actual earlier accepted event after the denial must
        // fail: the live observation state cannot recover from rejection.
        let earlier = events
            .iter()
            .find(|event| event.kind == ExecutionEventKind::ProtectedExec && !event.protected_exec_violation)
            .unwrap();
        let action = prepared.adapter().actions.iter().find(|action| action.id == earlier.action_id).unwrap();
        assert!(state.record(&prepared.proof, action, earlier).unwrap_err().contains("already failed"));
    } else if cancellation {
        assert!(result.is_err());
        assert!(started.elapsed() < Duration::from_secs(7), "cancellation must bound all teardown waits");
        assert!(rows.iter().all(|row| row.get("actual_outputs").is_none() && row["complete"] != true));
    } else {
        let completed = result.unwrap_or_else(|failure| panic!("real protected recipe failed: {failure}"));
        assert_eq!(completed.completions[0].exit_code, 0);
        assert_eq!(
            completed.outputs[&ArtifactRef::Produced {
                action_id: "stagex-transition/leaf".into(),
                output_id: "stdout".into()
            }]
                .1,
            blake3::hash(stdout).to_hex().to_string()
        );
        assert_eq!(events.iter().filter(|event| event.kind == ExecutionEventKind::ProtectedExec).count(), 2);
        assert!(rows.iter().any(|row| row["complete"] == true));
    }
    if mode == "workspace" {
        assert_eq!(
            fs::read(prepared.runtime_root.join("leaf/outputs/generated/generated.c")).unwrap(),
            b"int generated(void) { return 42; }\n"
        );
        assert!(prepared.runtime_root.join("leaf/outputs/other").is_dir());
        assert_eq!(fs::read_dir(prepared.runtime_root.join("leaf/outputs/other")).unwrap().count(), 0);
    }
}

#[test]
#[ignore = "requires parent-built libc-free ELF, reviewed bwrap and fresh evidence root"]
fn protected_recipe_smoke_success() {
    smoke("success", b"leaf-ok\n", false, false);
}

#[test]
#[ignore = "requires parent-built libc-free ELF, reviewed bwrap and fresh evidence root"]
fn protected_recipe_smoke_handled_second_exec() {
    smoke("exec", b"exec-denied\n", true, false);
}

#[test]
#[ignore = "requires parent-built libc-free ELF, reviewed bwrap and fresh evidence root"]
fn protected_recipe_smoke_handled_process_clone() {
    smoke("fork", b"clone-denied\n", true, false);
}

#[test]
#[ignore = "requires parent-built libc-free ELF, reviewed bwrap and fresh evidence root"]
fn protected_recipe_smoke_cancellation() {
    smoke("cancel", b"cancel-ready\n", false, true);
}

#[test]
#[ignore = "requires parent-built libc-free ELF with workspace mode, reviewed bwrap and fresh evidence root"]
fn protected_recipe_smoke_generated_workspace() {
    smoke("workspace", b"workspace-ok\n", false, false);
}

fn fetcher_record_fixture(path: &Path, metadata: BTreeMap<String, String>) -> crate::source_bundle::SourceRecord {
    use crate::source_bundle::SourceRecordKind;
    use crate::source_bundle::SourceSpec;
    let mut records = crate::source_bundle::plan_source_bundle(
        &[SourceSpec {
            kind: SourceRecordKind::LocalPath,
            identity: "original-source".into(),
            path: path.to_path_buf(),
            adapter: None,
        }],
        "/mantle/store",
    )
    .unwrap()
    .records;
    let mut record = records.remove(0);
    record.kind = SourceRecordKind::FixedUrl;
    record.metadata = metadata;
    record.content_blake3 =
        crate::source_bundle::digest_source_record_content(&record.kind, &record.metadata, &record.files).unwrap();
    record
}

fn prepare_fetcher_fixture(
    root: &Path,
    record: crate::source_bundle::SourceRecord,
    operation: RecipeOperation,
    kind: ArtifactKind,
    bytes_max: u64,
) -> Result<PreparedRecipe, RunError> {
    let source = ArtifactRef::Produced {
        action_id: "stagex-transition/materialize".into(),
        output_id: "source".into(),
    };
    let recipe = SourceRecipe {
        schema: SCHEMA.into(),
        stage_id: "stagex-transition".into(),
        preparation: RecipePreparation {
            tool: "original-fetcher-regression".into(),
            provenance: "source materialization only; no bootstrap claim".into(),
        },
        timeout_groups: Vec::new(),
        steps: vec![RecipeStep {
            id: "materialize".into(),
            inputs: BTreeMap::new(),
            outputs: BTreeMap::from([("source".into(), RecipeOutput { kind, bytes_max })]),
            operation,
        }],
        roots: vec![source.clone()],
        exports: BTreeMap::from([("source".into(), source)]),
        semantic_roots: BTreeMap::new(),
    };
    let bundle = crate::source_bundle::assemble_source_bundle(vec![record], "/mantle/store")?;
    let mut original = crate::source_built_fixed_point_action_trust::tests::fixture().0;
    let mut roots = BTreeMap::new();
    for input in &mut original.source_inputs {
        let path = root.join(format!("{:?}", input.role));
        match input.role {
            SourceAuthorityRole::StagexLineage => fs::write(&path, serde_json::to_vec(&recipe).unwrap()).unwrap(),
            SourceAuthorityRole::StagexSourceBundle => crate::source_bundle::write_source_bundle(&path, &bundle)?,
            _ if input.kind == SourceContentKind::Directory => {
                fs::create_dir(&path).unwrap();
                fs::write(path.join("source.txt"), b"inert source fixture\n").unwrap();
            }
            _ => fs::write(&path, b"inert source fixture\n").unwrap(),
        }
        (input.size_bytes, input.digest_blake3) = digest(&path)?;
        roots.insert(input.role, path);
    }
    let proof = plan_source_built_fixed_point(SourceBuiltFixedPointPlanInput {
        proof_id: original.proof_id,
        logical_store_prefix: original.logical_store_prefix,
        source_inputs: original.source_inputs,
        policies: original.policies,
        resource_bounds: original.resource_bounds,
        initial_output_authority: InitialOutputAuthorityState {
            stagex_transition_entries: 0,
            native_provider_entries: 0,
            rust_provider_entries: 0,
            mantle_output_entries: 0,
        },
    })
    .map_err(|failure| error(failure.to_string()))?;
    // No sandbox launch occurs for source materialization. Its declared identity
    // still comes from a real executable; this is not a mock execution path.
    let sandbox = fs::canonicalize(env!("SNIX_BUILD_SANDBOX_SHELL")).unwrap();
    crate::source_built_recipe::prepare(
        &proof,
        "stagex-transition",
        SourceAuthorityRole::StagexLineage,
        "",
        &RecipePaths {
            source_roots: &roots,
            runtime_root: &root.join("runtime"),
            sandbox_binary: &sandbox,
        },
        &[],
        HOST_EXECUTABLE_DIGEST.as_str(),
    )
}

fn fetcher_operation() -> RecipeOperation {
    RecipeOperation::MaterializeFetcher {
        role: SourceAuthorityRole::StagexSourceBundle,
        record_identity: "original-source".into(),
        output: "source".into(),
    }
}

#[test]
fn materialize_fetcher_preserves_flat_bytes_and_rejects_wrong_pin_and_bound() {
    use sha2::Digest;
    let payload = b"original source bytes\n";
    let fixtures = tempfile::tempdir().unwrap();
    let path = fixtures.path().join("payload");
    fs::write(&path, payload).unwrap();
    let record = fetcher_record_fixture(
        &path,
        BTreeMap::from([
            ("url".into(), "https://example.invalid/original.txt".into()),
            ("hash_algo".into(), "sha256".into()),
            ("hash_mode".into(), "flat".into()),
            ("hash".into(), data_encoding::HEXLOWER.encode(&sha2::Sha256::digest(payload))),
        ]),
    );
    let good = tempfile::tempdir().unwrap();
    let prepared =
        prepare_fetcher_fixture(good.path(), record.clone(), fetcher_operation(), ArtifactKind::Text, 4096).unwrap();
    let run = crate::source_built_recipe::run(&prepared, &mut |_| Ok(())).unwrap();
    assert_eq!(fs::read(&run.outputs.values().next().unwrap().0).unwrap(), payload);
    let small = tempfile::tempdir().unwrap();
    let prepared =
        prepare_fetcher_fixture(small.path(), record.clone(), fetcher_operation(), ArtifactKind::Text, 1).unwrap();
    assert!(
        crate::source_built_recipe::run(&prepared, &mut |_| Ok(()))
            .err()
            .unwrap()
            .to_string()
            .contains("byte bound")
    );
    let mut wrong = record;
    wrong.metadata.insert("hash".into(), "00".repeat(32));
    wrong.content_blake3 =
        crate::source_bundle::digest_source_record_content(&wrong.kind, &wrong.metadata, &wrong.files).unwrap();
    let bad = tempfile::tempdir().unwrap();
    let prepared = prepare_fetcher_fixture(bad.path(), wrong, fetcher_operation(), ArtifactKind::Text, 4096).unwrap();
    assert!(crate::source_built_recipe::run(&prepared, &mut |_| Ok(())).is_err());
}

#[test]
fn materialize_fetcher_decodes_original_archive_and_verifies_recursive_pin() {
    let fixtures = tempfile::tempdir().unwrap();
    let tree = fixtures.path().join("tree");
    fs::create_dir(&tree).unwrap();
    fs::write(tree.join("source.c"), b"int source(void) { return 7; }\n").unwrap();
    let pin = crunch_nar::observe_path(
        &tree,
        crunch_nar::FilesystemNarRequest::new(
            nix_compat::nixhash::HashAlgo::Sha256,
            crunch_nar::CaseHackPolicy::native(),
            1_048_576,
        ),
    )
    .unwrap()
    .digest
    .to_sri_string();
    let path = fixtures.path().join("original.tar.gz");
    let encoder = flate2::write::GzEncoder::new(File::create(&path).unwrap(), flate2::Compression::default());
    let mut archive = tar::Builder::new(encoder);
    archive.append_dir_all("original", &tree).unwrap();
    archive.into_inner().unwrap().finish().unwrap();
    let record = fetcher_record_fixture(
        &path,
        BTreeMap::from([
            ("url".into(), "https://example.invalid/original.tar.gz".into()),
            ("unpack".into(), "1".into()),
            ("payload_encoding".into(), "tarball-archive-v1".into()),
            ("hash_algo".into(), "sha256".into()),
            ("hash_mode".into(), "recursive".into()),
            ("hash".into(), pin),
        ]),
    );
    let good = tempfile::tempdir().unwrap();
    let prepared =
        prepare_fetcher_fixture(good.path(), record.clone(), fetcher_operation(), ArtifactKind::Tree, 4096).unwrap();
    let run = crate::source_built_recipe::run(&prepared, &mut |_| Ok(())).unwrap();
    assert_eq!(
        fs::read(run.outputs.values().next().unwrap().0.join("source.c")).unwrap(),
        b"int source(void) { return 7; }\n"
    );
    let mut wrong = record;
    wrong.metadata.insert("hash".into(), "00".repeat(32));
    wrong.content_blake3 =
        crate::source_bundle::digest_source_record_content(&wrong.kind, &wrong.metadata, &wrong.files).unwrap();
    let bad = tempfile::tempdir().unwrap();
    let prepared = prepare_fetcher_fixture(bad.path(), wrong, fetcher_operation(), ArtifactKind::Tree, 4096).unwrap();
    assert!(crate::source_built_recipe::run(&prepared, &mut |_| Ok(())).is_err());
}

#[test]
fn materialize_fetcher_rejects_nonfetchers_wrong_kinds_unknown_records_and_undeclared_roles() {
    use crate::source_bundle::SourceRecordKind;
    let fixtures = tempfile::tempdir().unwrap();
    let path = fixtures.path().join("payload");
    fs::write(&path, b"original source\n").unwrap();
    let record = fetcher_record_fixture(&path, BTreeMap::new());
    for kind in [
        SourceRecordKind::ToolchainSourceRoot,
        SourceRecordKind::ProviderManifest,
        SourceRecordKind::BootstrapArchive,
    ] {
        let mut nonfetcher = record.clone();
        nonfetcher.kind = kind;
        assert!(fetcher_output_kind(&nonfetcher).is_err());
    }
    for kind in [ArtifactKind::Tree, ArtifactKind::Executable] {
        let root = tempfile::tempdir().unwrap();
        assert!(prepare_fetcher_fixture(root.path(), record.clone(), fetcher_operation(), kind, 4096).is_err());
    }
    let root = tempfile::tempdir().unwrap();
    let operation = RecipeOperation::MaterializeFetcher {
        role: SourceAuthorityRole::StagexSourceBundle,
        record_identity: "absent".into(),
        output: "source".into(),
    };
    assert!(prepare_fetcher_fixture(root.path(), record.clone(), operation, ArtifactKind::Text, 4096).is_err());
    let root = tempfile::tempdir().unwrap();
    let operation = RecipeOperation::MaterializeFetcher {
        role: SourceAuthorityRole::MantleSource,
        record_identity: "original-source".into(),
        output: "source".into(),
    };
    assert!(prepare_fetcher_fixture(root.path(), record, operation, ArtifactKind::Text, 4096).is_err());
}

#[test]
fn materialize_fetcher_rejects_manifest_drift_after_preparation() {
    let fixtures = tempfile::tempdir().unwrap();
    let path = fixtures.path().join("payload");
    fs::write(&path, b"original source\n").unwrap();
    let record = fetcher_record_fixture(&path, BTreeMap::new());
    let root = tempfile::tempdir().unwrap();
    let prepared = prepare_fetcher_fixture(root.path(), record, fetcher_operation(), ArtifactKind::Text, 4096).unwrap();
    fs::write(&prepared.sources[&SourceAuthorityRole::StagexSourceBundle], b"changed manifest\n").unwrap();
    assert!(
        crate::source_built_recipe::run(&prepared, &mut |_| Ok(()))
            .err()
            .unwrap()
            .to_string()
            .contains("changed after planning")
    );
    assert!(!prepared.runtime_root.join("materialize/outputs/source").exists());
}

fn probe_smoke(mode: &str, code: i32, accepted: bool) {
    let _test_gate = PROTECTED_TEST_GATE.lock();
    let (prepared, root) = recipe_case(mode, 20);
    let mut events = Vec::new();
    let mut event_file = create(&root.join("observed-events.jsonl")).unwrap();
    let result = crate::source_built_recipe::run(&prepared, &mut |event| {
        journal(&mut event_file, &event)?;
        events.push(event);
        Ok(())
    });
    assert_eq!(
        fs::read(prepared.runtime_root.join("leaf/outputs/stdout")).unwrap(),
        if mode == "normal" {
            b"normal-probe\n".as_slice()
        } else {
            b"probe-ok\n".as_slice()
        }
    );
    assert_eq!(
        fs::read(prepared.runtime_root.join("leaf/outputs/exit-status")).unwrap(),
        format!("{code}\n").as_bytes()
    );
    let raw = fs::read_to_string(prepared.runtime_root.join("execution.jsonl")).unwrap();
    let rows = raw.lines().map(|line| serde_json::from_str::<serde_json::Value>(line).unwrap()).collect::<Vec<_>>();
    assert!(rows.iter().any(|row| row["normal_exit_code"] == code));
    assert_eq!(rows.iter().filter(|row| row["is_exec"] == true && row["allowed"] == true).count(), 2);
    if accepted {
        let run = result.unwrap_or_else(|failure| panic!("source-owned probe policy rejected normal exit: {failure}"));
        assert_eq!(run.completions[0].exit_code, code);
        let reference = ArtifactRef::Produced {
            action_id: "stagex-transition/leaf".into(),
            output_id: "exit-status".into(),
        };
        assert_eq!(run.outputs[&reference].1, blake3::hash(format!("{code}\n").as_bytes()).to_hex().to_string());
        assert!(rows.iter().any(|row| row["complete"] == true));
    } else {
        assert!(result.is_err());
        assert!(rows.iter().any(|row| row.get("failure").is_some()));
        assert!(rows.iter().all(|row| row.get("actual_outputs").is_none() && row["complete"] != true));
        assert!(
            events
                .iter()
                .filter(|event| event.action_id == "stagex-transition/leaf")
                .all(|event| event.produced_outputs.is_empty())
        );
    }
}

#[test]
#[ignore = "requires parent-built libc-free ELF with probe0/probe1 modes, reviewed bwrap and fresh evidence root"]
fn protected_recipe_smoke_probe_exit_policy_preserves_actual_zero_and_one() {
    probe_smoke("probe0", 0, true);
    probe_smoke("probe1", 1, true);
}

#[test]
#[ignore = "requires parent-built libc-free ELF with probe2 mode, reviewed bwrap and fresh evidence root"]
fn protected_recipe_smoke_probe_exit_policy_rejects_unlisted_exit() {
    probe_smoke("probe2", 2, false);
}

#[test]
#[ignore = "requires parent-built libc-free ELF with probe0/probe1/workspace modes, reviewed bwrap and fresh evidence root"]
fn protected_recipe_smoke_guarded_phase_and_immutable_output_seeds() {
    let _test_gate = PROTECTED_TEST_GATE.lock();
    for (mode, active) in [("guard0", true), ("guard1", false)] {
        let (prepared, _) = recipe_case(mode, 30);
        let mut state = ActionObservationState::default();
        let mut events = Vec::new();
        let result = crate::source_built_recipe::run(&prepared, &mut |event| {
            let action = prepared.adapter().actions.iter().find(|action| action.id == event.action_id).unwrap();
            state.record(&prepared.proof, action, &event).map_err(error)?;
            events.push(event);
            Ok(())
        })
        .unwrap();
        let prior = prepared.runtime_root.join("initial-state/outputs/state");
        assert_eq!(fs::read(prior.join("prior.txt")).unwrap(), b"immutable prior state\n");
        assert!(!prior.join("generated.c").exists());
        let envelope = prepared.runtime_root.join("observe/outputs/envelope");
        assert_eq!(fs::read(envelope.join("state/prior.txt")).unwrap(), b"immutable prior state\n");
        let outcome: PhaseOutcome = serde_json::from_slice(&fs::read(envelope.join("outcome.json")).unwrap()).unwrap();
        let conditional_events =
            events.iter().filter(|event| event.action_id == "stagex-transition/conditional").collect::<Vec<_>>();
        assert_eq!(
            conditional_events
                .iter()
                .filter(|event| matches!(event.kind, ExecutionEventKind::GuardDecision { .. }))
                .count(),
            1
        );
        let next_decision = events
            .iter()
            .find(|event| {
                event.action_id == "stagex-transition/conditional-next"
                    && matches!(event.kind, ExecutionEventKind::GuardDecision { .. })
            })
            .unwrap();
        assert_eq!(next_decision.kind, ExecutionEventKind::GuardDecision {
            outcome: if active {
                GuardOutcome::Active
            } else {
                GuardOutcome::Skipped
            },
            exit_codes: if active { vec![0, 0] } else { vec![1] }
        });
        if active {
            assert!(matches!(outcome, PhaseOutcome::Executed { .. }));
            assert_eq!(fs::read(envelope.join("state/generated.c")).unwrap(), b"int generated(void) { return 42; }\n");
            assert_eq!(result.completions.len(), 3);
            assert_eq!(conditional_events.len(), 3);
        } else {
            assert_eq!(outcome, PhaseOutcome::Skipped { guard_codes: vec![1] });
            assert!(!prepared.runtime_root.join("conditional").exists());
            assert!(!prepared.runtime_root.join("conditional-next").exists());
            assert!(!envelope.join("outputs").exists());
            assert_eq!(result.completions.len(), 1);
            assert!(result.outputs.keys().all(|reference| !matches!(reference, ArtifactRef::Produced { action_id, output_id } if action_id.starts_with("stagex-transition/conditional") || output_id == "absent-object")));
            assert_eq!(conditional_events.len(), 1);
            assert!(!conditional_events[0].completed_locally && conditional_events[0].produced_outputs.is_empty());
        }
    }
}

#[test]
#[ignore = "requires parent-built libc-free ELF with probe1 mode, reviewed bwrap and fresh evidence root"]
fn protected_recipe_smoke_rejects_forged_status_before_conditional_launch() {
    let _test_gate = PROTECTED_TEST_GATE.lock();
    let (prepared, _) = recipe_case("guard-forged", 30);
    let mut state = ActionObservationState::default();
    let mut events = Vec::new();
    let result = crate::source_built_recipe::run(&prepared, &mut |event| {
        let action = prepared.adapter().actions.iter().find(|action| action.id == event.action_id).unwrap();
        state.record(&prepared.proof, action, &event).map_err(error)?;
        if event.action_id == "stagex-transition/leaf" && event.kind == ExecutionEventKind::BuildExecution {
            fs::write(prepared.runtime_root.join("leaf/outputs/exit-status"), b"0\n").unwrap();
        }
        events.push(event);
        Ok(())
    });
    assert!(result.err().unwrap().to_string().contains("changed after actual completion"));
    assert!(!prepared.runtime_root.join("conditional").exists());
    assert!(!prepared.runtime_root.join("observe").exists());
    assert!(
        events
            .iter()
            .all(|event| !event.action_id.ends_with("/conditional")
                && !event.action_id.ends_with("/conditional.sandbox"))
    );
}

#[test]
#[ignore = "requires parent-built libc-free ELF merged mode writing alternating descriptors, reviewed bwrap and fresh evidence root"]
fn protected_recipe_smoke_merge_preserves_actual_descriptor_write_order() {
    let _test_gate = PROTECTED_TEST_GATE.lock();
    let (prepared, _) = recipe_case("merged", 30);
    let mut state = ActionObservationState::default();
    let result = crate::source_built_recipe::run(&prepared, &mut |event| {
        let action = prepared.adapter().actions.iter().find(|action| action.id == event.action_id).unwrap();
        state.record(&prepared.proof, action, &event).map_err(error)
    })
    .unwrap();
    assert_eq!(
        fs::read(prepared.runtime_root.join("leaf/outputs/stdout")).unwrap(),
        b"out-a\nerr-a\nout-b\nerr-b\n"
    );
    assert_eq!(fs::read(prepared.runtime_root.join("leaf/outputs/stderr")).unwrap(), b"");
    assert_eq!(result.completions[0].exit_code, 0);
}

#[test]
#[ignore = "requires parent-built libc-free ELF request/self-copy and probe0 modes, reviewed bwrap and fresh evidence root"]
fn protected_recipe_smoke_generated_invocation_uses_exact_produced_executable() {
    let _test_gate = PROTECTED_TEST_GATE.lock();
    let (prepared, _) = recipe_case("invocation", 30);
    let mut state = ActionObservationState::default();
    let mut events = Vec::new();
    let result = crate::source_built_recipe::run(&prepared, &mut |event| {
        let action = prepared.adapter().actions.iter().find(|action| action.id == event.action_id).unwrap();
        state.record(&prepared.proof, action, &event).map_err(error)?;
        events.push(event);
        Ok(())
    })
    .unwrap();
    assert_eq!(fs::read(prepared.runtime_root.join("invoked/outputs/stdout")).unwrap(), b"probe-ok\n");
    assert_eq!(result.completions.iter().map(|item| item.exit_code).collect::<Vec<_>>(), [0, 0]);
    let execution = events
        .iter()
        .find(|event| event.action_id.ends_with("/invoked") && event.kind == ExecutionEventKind::ProtectedExec)
        .unwrap();
    assert_eq!(execution.executable_producer, Some(("stagex-transition/leaf".into(), "executable".into())));
    assert_eq!(
        execution.executable_digest_blake3,
        digest(&prepared.runtime_root.join("leaf/outputs/executable")).unwrap().1
    );
}

#[test]
#[ignore = "requires parent-built libc-free ELF request/self-copy mode, reviewed bwrap and fresh evidence root"]
fn protected_recipe_smoke_rejects_forged_invocation_before_launch() {
    let _test_gate = PROTECTED_TEST_GATE.lock();
    let (prepared, _) = recipe_case("invocation-forged", 30);
    let mut state = ActionObservationState::default();
    let mut events = Vec::new();
    let result = crate::source_built_recipe::run(&prepared, &mut |event| {
        let action = prepared.adapter().actions.iter().find(|action| action.id == event.action_id).unwrap();
        state.record(&prepared.proof, action, &event).map_err(error)?;
        if event.action_id == "stagex-transition/leaf" && event.kind == ExecutionEventKind::BuildExecution {
            let forged = RecipeInvocation {
                readonly_input_roots: Vec::new(),
                arguments: vec![RecipeWord {
                    parts: vec![WordPart::Literal { text: "probe1".into() }],
                }],
                environment: BTreeMap::new(),
                cwd: None,
                argv0: None,
                executable_path: None,
                workspace_mounts: BTreeMap::new(),
                process_attributes: None,
                descriptors: None,
            };
            fs::write(prepared.runtime_root.join("leaf/outputs/stdout"), serde_json::to_vec(&forged).unwrap()).unwrap();
        }
        events.push(event);
        Ok(())
    });
    assert!(result.err().unwrap().to_string().contains("authenticated producer bytes"));
    assert!(!prepared.runtime_root.join("invoked/outputs/stdout").exists());
    assert!(
        events
            .iter()
            .all(|event| !event.action_id.ends_with("/invoked") && !event.action_id.ends_with("/invoked.sandbox"))
    );
}

fn materialized_operand_text(
    payload: &[u8],
) -> (tempfile::TempDir, PreparedRecipe, RecipeRun, RecipeStep, ActionSpec, Vec<String>) {
    use sha2::Digest;
    let root = tempfile::tempdir().unwrap();
    let original = root.path().join("original-source.txt");
    fs::write(&original, payload).unwrap();
    let record = fetcher_record_fixture(
        &original,
        BTreeMap::from([
            ("url".into(), "https://example.invalid/original.txt".into()),
            ("hash_algo".into(), "sha256".into()),
            ("hash_mode".into(), "flat".into()),
            ("hash".into(), data_encoding::HEXLOWER.encode(&sha2::Sha256::digest(payload))),
        ]),
    );
    let prepared = prepare_fetcher_fixture(root.path(), record, fetcher_operation(), ArtifactKind::Text, 4096).unwrap();
    let run = crate::source_built_recipe::run(&prepared, &mut |_| Ok(())).unwrap();
    let reference = run.outputs.keys().next().unwrap().clone();
    let mut step = prepared.recipe.steps[0].clone();
    step.inputs = BTreeMap::from([("data".into(), reference.clone()), ("alias".into(), reference.clone())]);
    // Exercise the actual operand resolver over genuinely materialized source,
    // not a mock leaf or a claimed executable/proof completion.
    let mut consumer = prepared.adapter().actions[0].clone();
    let ArtifactRef::Produced { action_id, output_id } = &reference else {
        unreachable!()
    };
    consumer.inputs = vec![ActionInput::Produced {
        action_id: action_id.clone(),
        output_id: output_id.clone(),
    }];
    let measured = vec![run.outputs[&reference].1.clone()];
    (root, prepared, run, step, consumer, measured)
}

#[test]
fn text_operands_authenticate_whole_source_and_share_utf8_safe_snapshot() {
    let original = "prefix:https://example.invalid/π";
    let (_root, prepared, run, step, action, measured) = materialized_operand_text(original.as_bytes());
    let mut operands = CommandOperands {
        prepared: &prepared,
        step: &step,
        action: &action,
        measured_inputs: &measured,
        known_outputs: &run.outputs,
        texts: BTreeMap::new(),
        dynamic_bindings: None,
    };
    let start = original.find("https:").unwrap();
    let word = RecipeWord {
        parts: vec![WordPart::TextSlice {
            name: "data".into(),
            offset_bytes: start as u64,
            length_bytes: (original.len() - start) as u64,
        }],
    };
    assert_eq!(operands.word(&word).unwrap(), "https://example.invalid/π");
    // A second label/slice of the same producer cannot reread changed bytes.
    fs::remove_file(&run.outputs.values().next().unwrap().0).unwrap();
    assert_eq!(
        operands
            .word(&RecipeWord {
                parts: vec![WordPart::TextInput { name: "alias".into() }]
            })
            .unwrap(),
        original
    );
    let unicode = original.find('π').unwrap();
    assert_eq!(
        operands
            .word(&RecipeWord {
                parts: vec![WordPart::TextSlice {
                    name: "alias".into(),
                    offset_bytes: unicode as u64,
                    length_bytes: 2
                }]
            })
            .unwrap(),
        "π"
    );
    for (offset_bytes, length_bytes) in [(unicode as u64 + 1, 1), (original.len() as u64, 1), (u64::MAX, 1)] {
        assert!(
            operands
                .word(&RecipeWord {
                    parts: vec![WordPart::TextSlice {
                        name: "data".into(),
                        offset_bytes,
                        length_bytes
                    }]
                })
                .is_err()
        );
    }
}

#[test]
fn text_slice_rejects_forged_whole_frame_even_when_selected_bytes_match() {
    let original = b"prefix:accepted\ntrailer:original\n";
    let (_root, prepared, run, step, action, measured) = materialized_operand_text(original);
    fs::write(&run.outputs.values().next().unwrap().0, b"prefix:accepted\ntrailer:forged\n").unwrap();
    let mut operands = CommandOperands {
        prepared: &prepared,
        step: &step,
        action: &action,
        measured_inputs: &measured,
        known_outputs: &run.outputs,
        texts: BTreeMap::new(),
        dynamic_bindings: None,
    };
    let word = RecipeWord {
        parts: vec![WordPart::TextSlice {
            name: "data".into(),
            offset_bytes: 7,
            length_bytes: 8,
        }],
    };
    assert!(operands.word(&word).err().unwrap().to_string().contains("authenticated producer bytes"));
}

#[test]
#[ignore = "requires parent-built libc-free ELF request/self-copy and data modes, reviewed bwrap and fresh evidence root"]
fn protected_recipe_smoke_typed_text_slices_and_runtime_tree_reach_actual_leaf() {
    let _test_gate = PROTECTED_TEST_GATE.lock();
    let (prepared, _) = recipe_case("operands", 30);
    let mut state = ActionObservationState::default();
    let result = crate::source_built_recipe::run(&prepared, &mut |event| {
        let action = prepared.adapter().actions.iter().find(|action| action.id == event.action_id).unwrap();
        state.record(&prepared.proof, action, &event).map_err(error)
    })
    .unwrap();
    // The actual ELF checks exact argv and the three declared environment
    // entries plus bubblewrap's exact PWD=/work; it reads/compares the full
    // file through the read-only Produced runtime Tree.
    assert_eq!(fs::read(prepared.runtime_root.join("operands/outputs/stdout")).unwrap(), b"operands-ok\n");
    assert_eq!(result.completions.iter().map(|item| item.exit_code).collect::<Vec<_>>(), [0, 0]);
}

#[test]
fn archive_extraction_preserves_original_root_and_enforces_member_output_bounds() {
    let (_root, mut prepared, _run, mut step, _action, _measured) = materialized_operand_text(b"fixture");
    let archives = tempfile::tempdir().unwrap();
    let original = archives.path().join("source");
    fs::create_dir(&original).unwrap();
    fs::write(original.join("source.c"), b"int original(void) { return 7; }\n").unwrap();
    let mut tar = tar::Builder::new(Vec::new());
    tar.append_dir_all("original-release", &original).unwrap();
    let raw = tar.into_inner().unwrap();
    let mut encoder = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::default());
    encoder.write_all(&raw).unwrap();
    let gzip = encoder.finish().unwrap();
    let reference = step.inputs["data"].clone();
    step.inputs = BTreeMap::from([("archive".into(), reference.clone())]);
    step.outputs = BTreeMap::from([("source".into(), RecipeOutput {
        kind: ArtifactKind::Tree,
        bytes_max: 4096,
    })]);
    for (index, (format, payload)) in
        [(RecipeArchiveFormat::Tar, raw), (RecipeArchiveFormat::TarGz, gzip)].into_iter().enumerate()
    {
        let path = archives.path().join(format!("archive-{index}"));
        fs::write(&path, payload).unwrap();
        prepared.bindings.get_mut(&reference).unwrap().path = path;
        step.operation = RecipeOperation::ExtractArchive {
            input: "archive".into(),
            output: "source".into(),
            format,
        };
        let output = archives.path().join(format!("output-{index}"));
        fs::create_dir(&output).unwrap();
        builtin(&prepared, &step, &output).unwrap();
        assert_eq!(
            fs::read(output.join("source/original-release/source.c")).unwrap(),
            b"int original(void) { return 7; }\n"
        );
        assert!(!output.join("source/source.c").exists());
        step.outputs.get_mut("source").unwrap().bytes_max = 1;
        let small = archives.path().join(format!("small-{index}"));
        fs::create_dir(&small).unwrap();
        assert!(builtin(&prepared, &step, &small).err().unwrap().to_string().contains("archive byte bound"));
        step.outputs.get_mut("source").unwrap().bytes_max = 4096;
    }
    for (index, entry_type) in [tar::EntryType::Regular, tar::EntryType::Symlink].into_iter().enumerate() {
        let mut header = tar::Header::new_gnu();
        header.set_size(0);
        header.set_mode(0o644);
        header.set_entry_type(entry_type);
        header.set_path("original-release/member").unwrap();
        if entry_type.is_symlink() {
            header.set_link_name("../../outside").unwrap();
        } else {
            header.as_mut_bytes()[..10].copy_from_slice(b"../outside");
            header.as_mut_bytes()[10] = 0;
        }
        header.set_cksum();
        let mut archive = tar::Builder::new(Vec::new());
        archive.append(&header, &b""[..]).unwrap();
        let path = archives.path().join(format!("unsafe-{index}.tar"));
        fs::write(&path, archive.into_inner().unwrap()).unwrap();
        prepared.bindings.get_mut(&reference).unwrap().path = path;
        step.operation = RecipeOperation::ExtractArchive {
            input: "archive".into(),
            output: "source".into(),
            format: RecipeArchiveFormat::Tar,
        };
        let output = archives.path().join(format!("unsafe-output-{index}"));
        fs::create_dir(&output).unwrap();
        assert!(builtin(&prepared, &step, &output).is_err());
        assert!(!archives.path().join("outside").exists());
    }
}

#[test]
fn shared_timeout_arena_never_restarts_for_resumed_leaves_or_observation() {
    let (_root, prepared, _, _, _, _) = materialized_operand_text(b"fixture");
    let mut recipe = prepared.recipe.clone();
    for id in ["leaf", "observe", "after"] {
        let mut step = recipe.steps[0].clone();
        step.id = id.into();
        recipe.steps.push(step);
    }
    recipe.timeout_groups = vec![RecipeTimeoutGroup {
        first_step: recipe.steps[0].id.clone(),
        last_step: "observe".into(),
        seconds_max: 1,
    }];
    let start = Instant::now();
    let mut deadlines = RecipeDeadlines::new(&recipe, 30, start).unwrap();
    deadlines.begin(0, start).unwrap();
    deadlines.begin(1, start + Duration::from_millis(800)).unwrap();
    assert_eq!(deadlines.deadline().duration_since(start + Duration::from_millis(800)), Duration::from_millis(200));
    assert!(deadlines.begin(2, start + Duration::from_secs(1)).is_err());
    let mut fresh = RecipeDeadlines::new(&recipe, 30, start).unwrap();
    fresh.begin(0, start).unwrap();
    fresh.begin(2, start + Duration::from_millis(900)).unwrap();
    fresh.check(start + Duration::from_millis(950)).unwrap();
    fresh.begin(3, start + Duration::from_millis(950)).unwrap();
    assert_eq!(fresh.deadline(), start + Duration::from_secs(30));
}

#[test]
fn nested_source_timeout_uses_tightest_original_deadline_not_a_per_leaf_cap() {
    let (_root, prepared, _, _, _, _) = materialized_operand_text(b"fixture");
    let mut recipe = prepared.recipe.clone();
    for id in ["inner", "outer-last"] {
        let mut step = recipe.steps[0].clone();
        step.id = id.into();
        recipe.steps.push(step);
    }
    recipe.timeout_groups = vec![
        RecipeTimeoutGroup {
            first_step: recipe.steps[0].id.clone(),
            last_step: "outer-last".into(),
            seconds_max: 10,
        },
        RecipeTimeoutGroup {
            first_step: "inner".into(),
            last_step: "inner".into(),
            seconds_max: 2,
        },
    ];
    let start = Instant::now();
    let mut deadlines = RecipeDeadlines::new(&recipe, 30, start).unwrap();
    deadlines.begin(0, start).unwrap();
    deadlines.begin(1, start + Duration::from_secs(1)).unwrap();
    assert_eq!(deadlines.deadline(), start + Duration::from_secs(3));
    assert!(deadlines.check(start + Duration::from_secs(3)).is_err());
    let mut within = RecipeDeadlines::new(&recipe, 30, start).unwrap();
    within.begin(0, start).unwrap();
    within.begin(1, start + Duration::from_secs(1)).unwrap();
    within.begin(2, start + Duration::from_secs(2)).unwrap();
    assert_eq!(within.deadline(), start + Duration::from_secs(10));
    assert!(within.check(start + Duration::from_secs(10)).is_err());
}

#[test]
#[ignore = "requires parent-built libc-free ELF request/self-copy and local modes, reviewed bwrap and fresh evidence root"]
fn protected_recipe_smoke_typed_cwd_reaches_nested_input_and_output_tree_consumers() {
    let _test_gate = PROTECTED_TEST_GATE.lock();
    for mode in ["cwd", "cwd-output"] {
        let (prepared, _) = recipe_case(mode, 30);
        let mut state = ActionObservationState::default();
        let result = crate::source_built_recipe::run(&prepared, &mut |event| {
            let action = prepared.adapter().actions.iter().find(|action| action.id == event.action_id).unwrap();
            state.record(&prepared.proof, action, &event).map_err(error)
        })
        .unwrap();
        // Actual ELF verifies the exact selected Input/Output nested cwd and reads
        // source bytes using relative open("metadata.txt"), not argv echo.
        assert_eq!(fs::read(prepared.runtime_root.join("cwd-consumer/outputs/stdout")).unwrap(), b"cwd-operands-ok\n");
        assert_eq!(result.completions.iter().map(|item| item.exit_code).collect::<Vec<_>>(), [0, 0]);
    }
}

#[test]
#[ignore = "requires parent-built libc-free ELF request/self-copy mode, reviewed bwrap and fresh evidence root"]
fn protected_recipe_smoke_typed_cwd_rejects_forged_noncanonical_unknown_conflicting_and_escape_requests() {
    let _test_gate = PROTECTED_TEST_GATE.lock();
    for (mode, expected) in [
        ("cwd-forged", "authenticated producer bytes"),
        ("cwd-noncanonical", "noncanonical"),
        ("cwd-unknown", "unknown input"),
        ("cwd-conflict", "conflicts"),
        ("cwd-escape", "non-normalized relative path"),
    ] {
        let (prepared, _) = recipe_case(mode, 30);
        let mut state = ActionObservationState::default();
        let mut events = Vec::new();
        let result = crate::source_built_recipe::run(&prepared, &mut |event| {
            let action = prepared.adapter().actions.iter().find(|action| action.id == event.action_id).unwrap();
            state.record(&prepared.proof, action, &event).map_err(error)?;
            if mode == "cwd-forged"
                && event.action_id == "stagex-transition/cwd-request"
                && event.kind == ExecutionEventKind::BuildExecution
            {
                let path = prepared.runtime_root.join("cwd-request/outputs/text");
                let mut request: RecipeInvocation = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
                request.cwd = Some(RecipeWord {
                    parts: vec![WordPart::Input {
                        name: "workspace".into(),
                        path: "another-package".into(),
                    }],
                });
                fs::write(path, serde_json::to_vec(&request).unwrap()).unwrap();
            }
            events.push(event);
            Ok(())
        });
        assert!(result.err().unwrap().to_string().contains(expected), "{mode}");
        assert!(!prepared.runtime_root.join("cwd-consumer/outputs/stdout").exists());
        assert!(events.iter().all(|event| !event.action_id.starts_with("stagex-transition/cwd-consumer")));
    }
}

#[test]
#[ignore = "requires parent-built libc-free ELF request/self-copy and temporary modes, reviewed bwrap and fresh evidence root"]
fn protected_recipe_smoke_workspace_mount_captures_actual_original_tmp_write() {
    let _test_gate = PROTECTED_TEST_GATE.lock();
    let (prepared, _) = recipe_case("temporary", 30);
    let mut state = ActionObservationState::default();
    let result = crate::source_built_recipe::run(&prepared, &mut |event| {
        let action = prepared.adapter().actions.iter().find(|action| action.id == event.action_id).unwrap();
        state.record(&prepared.proof, action, &event).map_err(error)
    })
    .unwrap();
    assert_eq!(
        fs::read(prepared.runtime_root.join("temporary-consumer/outputs/generated/perl-io-positive")).unwrap(),
        b"actual-tmp-output\n"
    );
    assert_eq!(
        fs::read(prepared.runtime_root.join("temporary-consumer/outputs/stdout")).unwrap(),
        b"tmp-write-ok\n"
    );
    assert_eq!(result.completions.iter().map(|item| item.exit_code).collect::<Vec<_>>(), [0, 0]);
}

#[test]
#[ignore = "requires parent-built libc-free ELF request/self-copy mode, reviewed bwrap and fresh evidence root"]
fn protected_recipe_smoke_workspace_mount_does_not_recreate_missing_subtree() {
    let _test_gate = PROTECTED_TEST_GATE.lock();
    let (prepared, _) = recipe_case("temporary-missing", 30);
    let mut events = Vec::new();
    assert!(
        crate::source_built_recipe::run(&prepared, &mut |event| {
            events.push(event);
            Ok(())
        })
        .is_err()
    );
    assert!(!prepared.runtime_root.join("temporary-consumer/outputs/generated/removed").exists());
    assert!(!prepared.runtime_root.join("temporary-consumer/outputs/generated/perl-io-positive").exists());
    assert!(events.iter().all(|event| !event.action_id.starts_with("stagex-transition/temporary-consumer")));
}

#[test]
#[ignore = "requires parent-built libc-free ELF request/self-copy and image-context modes, reviewed bwrap and fresh evidence root"]
fn protected_recipe_smoke_installed_image_path_preserves_real_image_cwd_and_data_context() {
    let _test_gate = PROTECTED_TEST_GATE.lock();
    let (prepared, _) = recipe_case("image", 30);
    let mut state = ActionObservationState::default();
    let mut events = Vec::new();
    let result = crate::source_built_recipe::run(&prepared, &mut |event| {
        let action = prepared.adapter().actions.iter().find(|action| action.id == event.action_id).unwrap();
        state.record(&prepared.proof, action, &event).map_err(error)?;
        events.push(event);
        Ok(())
    })
    .unwrap();
    assert_eq!(fs::read(prepared.runtime_root.join("cwd-consumer/outputs/stdout")).unwrap(), b"image-context-ok\n");
    assert_eq!(result.completions.iter().map(|item| item.exit_code).collect::<Vec<_>>(), [0, 0]);
    let action = prepared
        .adapter()
        .actions
        .iter()
        .find(|action| action.id == "stagex-transition/cwd-consumer")
        .unwrap();
    let observed = events
        .iter()
        .find(|event| event.action_id == action.id && event.kind == ExecutionEventKind::ProtectedExec)
        .unwrap();
    assert_eq!(observed.executable_producer, Some(("stagex-transition/leaf".into(), "executable".into())));
}

#[test]
#[ignore = "requires parent-built libc-free ELF request/self-copy mode, reviewed bwrap and fresh evidence root"]
fn protected_recipe_smoke_installed_image_rejects_wrong_member_and_forged_whole_tree() {
    let _test_gate = PROTECTED_TEST_GATE.lock();
    for mode in ["image-mismatch", "image-forged"] {
        let (prepared, _) = recipe_case(mode, 30);
        let mut events = Vec::new();
        let result = crate::source_built_recipe::run(&prepared, &mut |event| {
            if mode == "image-forged"
                && event.action_id == "stagex-transition/image-installation"
                && event.kind == ExecutionEventKind::BuildExecution
            {
                let path = prepared.runtime_root.join("image-installation/outputs/tree/bin/fixture-image");
                OpenOptions::new().append(true).open(path).unwrap().write_all(b"forged").unwrap();
            }
            events.push(event);
            Ok(())
        });
        assert!(result.is_err());
        assert!(events.iter().all(|event| !event.action_id.starts_with("stagex-transition/cwd-consumer")));
    }
}

#[test]
fn phase_state_projection_forwards_exact_prior_envelope_checkpoint_and_rejects_invalid_members() {
    for case in [
        "valid",
        "escape",
        "absolute",
        "symlink",
        "missing",
        "file",
        "wrong-kind",
        "forged",
        "bound",
    ] {
        let (root, mut prepared, _run, _, mut action, _) = materialized_operand_text(b"checkpoint source");
        let prior = root.path().join("prior-envelope");
        fs::create_dir_all(prior.join("state")).unwrap();
        fs::write(prior.join("state/checkpoint"), b"actual-checkpoint\n").unwrap();
        fs::write(prior.join("outcome.json"), b"prior observed envelope\n").unwrap();
        #[cfg(unix)]
        std::os::unix::fs::symlink("state", prior.join("alias")).unwrap();
        let reference = ArtifactRef::Produced {
            action_id: "stagex-transition/prior-envelope".into(),
            output_id: "tree".into(),
        };
        let hash = digest(&prior).unwrap().1;
        let known = BTreeMap::from([(reference.clone(), (prior.clone(), hash.clone()))]);
        prepared.bindings.insert(reference.clone(), ArtifactBinding {
            reference: reference.clone(),
            path: prior.clone(),
            kind: if case == "wrong-kind" {
                ArtifactKind::Text
            } else {
                ArtifactKind::Tree
            },
            bytes_max: 4096,
        });
        action.inputs = vec![ActionInput::Produced {
            action_id: "stagex-transition/prior-envelope".into(),
            output_id: "tree".into(),
        }];
        let selected = match case {
            "escape" => "../state",
            "absolute" => "/state",
            "symlink" => "alias",
            "missing" => "absent",
            "file" => "outcome.json",
            _ => "state",
        };
        let step = RecipeStep {
            id: "projection".into(),
            inputs: BTreeMap::from([("prior".into(), reference)]),
            outputs: BTreeMap::from([("envelope".into(), RecipeOutput {
                kind: ArtifactKind::Tree,
                bytes_max: if case == "bound" { 1 } else { 4096 },
            })]),
            operation: RecipeOperation::ObservePhase {
                phase: "paused".into(),
                output: "envelope".into(),
                state: Some(RecipePhaseState {
                    output: "checkpoint".into(),
                    skipped_input: "prior".into(),
                    skipped_path: selected.into(),
                }),
            },
        };
        let outcome = PhaseOutcome::Skipped { guard_codes: vec![1] };
        let phases = BTreeMap::from([("stagex-transition/paused".into(), outcome.clone())]);
        let target = root.path().join("projection");
        fs::create_dir(&target).unwrap();
        if case == "forged" {
            fs::write(prior.join("state/checkpoint"), b"forged-checkpoint\n").unwrap();
        }
        let result = observe_phase(&prepared, &step, &target, &phases, &action, &[hash], &known);
        if case == "valid" {
            result.unwrap();
            assert_eq!(fs::read(target.join("envelope/state/checkpoint")).unwrap(), b"actual-checkpoint\n");
            assert!(!target.join("envelope/state/state").exists());
            assert!(!target.join("envelope/state/outcome.json").exists());
            assert_eq!(
                serde_json::from_slice::<PhaseOutcome>(&fs::read(target.join("envelope/outcome.json")).unwrap())
                    .unwrap(),
                outcome
            );
        } else {
            assert!(result.is_err(), "{case}");
        }
    }
}

#[test]
fn rooted_directory_selection_does_not_follow_symlinks_create_missing_members_or_accept_files() {
    let root = tempfile::tempdir().unwrap();
    fs::create_dir_all(root.path().join("package/nested")).unwrap();
    fs::write(root.path().join("package/data"), b"real source\n").unwrap();
    assert_eq!(rooted_directory(root.path(), "package/nested").unwrap(), root.path().join("package/nested"));
    for member in [
        "missing",
        "package/data",
        "../outside",
        "/outside",
        "package/../package",
    ] {
        assert!(rooted_directory(root.path(), member).is_err());
    }
    assert!(!root.path().join("missing").exists());
    #[cfg(unix)]
    {
        std::os::unix::fs::symlink("package", root.path().join("alias")).unwrap();
        assert!(rooted_directory(root.path(), "alias/nested").is_err());
    }
}

#[test]
fn installed_image_member_requires_same_real_regular_executable_without_symlink_projection() {
    let root = tempfile::tempdir().unwrap();
    let original = fs::canonicalize(env!("SNIX_BUILD_SANDBOX_SHELL")).unwrap();
    let expected = digest(&original).unwrap().1;
    fs::create_dir(root.path().join("bin")).unwrap();
    let installed = root.path().join("bin/image");
    fs::copy(&original, &installed).unwrap();
    assert_eq!(installed_image_member(root.path(), "bin/image", &expected).unwrap(), installed);
    for member in ["missing", "bin", "../image", "/image", ""] {
        assert!(installed_image_member(root.path(), member, &expected).is_err());
    }
    #[cfg(unix)]
    {
        std::os::unix::fs::symlink("image", root.path().join("bin/alias")).unwrap();
        assert!(installed_image_member(root.path(), "bin/alias", &expected).is_err());
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&installed, fs::Permissions::from_mode(0o644)).unwrap();
        assert!(installed_image_member(root.path(), "bin/image", &expected).is_err());
        fs::set_permissions(&installed, fs::Permissions::from_mode(0o755)).unwrap();
    }
    OpenOptions::new().append(true).open(&installed).unwrap().write_all(b"different bytes").unwrap();
    assert!(
        installed_image_member(root.path(), "bin/image", &expected)
            .err()
            .unwrap()
            .to_string()
            .contains("differs from fixed Produced executable")
    );
}

#[test]
#[ignore = "requires parent-built libc-free ELF normal-probe mode, reviewed bwrap and fresh evidence root"]
fn protected_recipe_smoke_normal_probe_records_actual_non_whitelisted_exit() {
    probe_smoke("normal", 123, true);
}

#[test]
#[ignore = "requires parent-built libc-free ELF observation mode, reviewed bwrap and fresh evidence root"]
fn protected_recipe_smoke_observes_actual_workspace_mtimes_and_seeked_stream_offsets() {
    use std::os::unix::fs::MetadataExt;
    let _test_gate = PROTECTED_TEST_GATE.lock();
    let (prepared, _) = recipe_case("observed", 30);
    let mut state = ActionObservationState::default();
    let result = crate::source_built_recipe::run(&prepared, &mut |event| {
        state
            .record_adapters(&prepared.proof, std::slice::from_ref(prepared.adapter()), &event)
            .map_err(error)
    })
    .unwrap();
    let outputs = prepared.runtime_root.join("leaf/outputs");
    assert_eq!(fs::read(outputs.join("workspace/observed.c")).unwrap(), b"int observed(void) { return 17; }\n");
    let descriptors = observation_records(&outputs.join("descriptors"));
    assert_eq!(descriptors[0], "MANTLE-COMMAND-DESCRIPTORS-2");
    for (index, expected) in [(0, 3), (1, 2), (2, 4)] {
        assert_eq!(descriptors[1 + index * 3 + 1], expected.to_string());
    }
    let times = observation_records(&outputs.join("times"));
    assert_eq!(times[0], "MANTLE-WORKSPACE-TIMES-1");
    let actual = fs::metadata(outputs.join("workspace/observed.c")).unwrap();
    let record = times[1..].chunks_exact(3).find(|entry| entry[0] == "observed.c").unwrap();
    assert_eq!(
        (record[1].parse::<i64>().unwrap(), record[2].parse::<i64>().unwrap()),
        (actual.mtime(), actual.mtime_nsec())
    );
    assert_eq!(result.completions[0].exit_code, 0);
}

#[test]
#[ignore = "requires parent-built libc-free ELF backing mode, reviewed bwrap/ADDFD and fresh evidence root"]
fn protected_recipe_smoke_preserves_sparse_64_fd_aliases_null_and_actual_status_flags() {
    let _test_gate = PROTECTED_TEST_GATE.lock();
    let (prepared, _) = recipe_case("fdtable", 30);
    let mut state = ActionObservationState::default();
    let result = crate::source_built_recipe::run(&prepared, &mut |event| {
        state
            .record_adapters(&prepared.proof, std::slice::from_ref(prepared.adapter()), &event)
            .map_err(error)
    })
    .unwrap();
    let outputs = prepared.runtime_root.join("fd-command/outputs");
    assert_eq!(fs::read(outputs.join("descriptor-files/1")).unwrap(), b"seedouterr");
    assert_eq!(fs::read(outputs.join("descriptor-files/5")).unwrap(), b"priorfd-five\n");
    assert_eq!(fs::read(outputs.join("descriptor-files/0")).unwrap(), b"abcdef");
    for absent in ["2", "3", "4", "9", "63"] {
        assert!(!outputs.join("descriptor-files").join(absent).exists());
    }
    let records = observation_records(&outputs.join("descriptors"));
    assert_eq!(records[0], "MANTLE-COMMAND-DESCRIPTORS-2");
    for (index, position, flags) in [
        (0, 4, 0),
        (1, 2, 0),
        (2, 2, 0),
        (3, 0, 0),
        (4, -1, -1),
        (5, 13, libc::O_APPEND | libc::O_NONBLOCK),
        (6, -1, -1),
        (9, 4, 0),
        (63, 13, libc::O_APPEND | libc::O_NONBLOCK),
    ] {
        assert_eq!(records[1 + index * 3], index.to_string());
        assert_eq!(records[1 + index * 3 + 1], position.to_string());
        assert_eq!(records[1 + index * 3 + 2], flags.to_string());
    }
    assert_eq!(result.completions.iter().map(|completion| completion.exit_code).collect::<Vec<_>>(), [0, 0]);
}

#[test]
#[ignore = "requires parent-built libc-free ELF verify mode, reviewed bwrap --argv0 and fresh evidence root"]
fn protected_recipe_smoke_actual_argv0_does_not_change_exact_image_authority() {
    let _test_gate = PROTECTED_TEST_GATE.lock();
    let (prepared, _) = recipe_case("argv0", 30);
    let mut state = ActionObservationState::default();
    let result = crate::source_built_recipe::run(&prepared, &mut |event| {
        state
            .record_adapters(&prepared.proof, std::slice::from_ref(prepared.adapter()), &event)
            .map_err(error)
    })
    .unwrap();
    assert_eq!(fs::read(prepared.runtime_root.join("argv0-command/outputs/stdout")).unwrap(), b"actual-argv0-ok\n");
    assert_eq!(result.completions.iter().map(|completion| completion.exit_code).collect::<Vec<_>>(), [0, 0]);
}

#[test]
#[ignore = "requires parent-built libc-free ELF umask mode, reviewed bwrap and fresh evidence root"]
fn protected_recipe_smoke_original_creation_masks_do_not_mutate_parent() {
    use std::os::unix::fs::MetadataExt;
    let _test_gate = PROTECTED_TEST_GATE.lock();
    let parent_status = fs::read_to_string("/proc/self/status").unwrap();
    let parent_mask = parent_status.lines().find(|line| line.starts_with("Umask:")).unwrap();
    for (mode, expected) in [("umask-owner", 0o600), ("umask-group", 0o640)] {
        let (prepared, _) = recipe_case(mode, 30);
        let mut state = ActionObservationState::default();
        let result = crate::source_built_recipe::run(&prepared, &mut |event| {
            state
                .record_adapters(&prepared.proof, std::slice::from_ref(prepared.adapter()), &event)
                .map_err(error)
        })
        .unwrap();
        let created = prepared.runtime_root.join("umask-command/outputs/generated/created");
        assert_eq!(fs::read(&created).unwrap(), b"original-mask\n");
        assert_eq!(fs::metadata(&created).unwrap().mode() & 0o777, expected);
        assert_eq!(result.completions.iter().map(|completion| completion.exit_code).collect::<Vec<_>>(), [0, 0]);
        let after = fs::read_to_string("/proc/self/status").unwrap();
        assert_eq!(after.lines().find(|line| line.starts_with("Umask:")).unwrap(), parent_mask);
    }
}
