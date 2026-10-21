//! Source-only release-to-release Rust bootstrap. Cargo is a product, never an
//! oracle or executor. Compiler/native phases remain separate producer actions.
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
#[path = "source_built_rust_programs_successors_graph.rs"]
mod graph;
#[path = "source_built_rust_programs_successors_helper.rs"]
mod helper;
#[path = "source_built_rust_programs_successors_native.rs"]
pub(crate) mod native;
#[path = "source_built_rust_programs_successors_profile.rs"]
mod profile;
pub(crate) use crate::source_built_rust_programs::resolve;
#[path = "source_built_rust_programs_successors_std_process.rs"]
mod std_process;
#[path = "source_built_rust_programs_successors_wrappers.rs"]
mod wrappers;
pub(crate) use graph::author_version;
pub(super) use wrappers::append_target_wrappers;
pub(crate) fn helper_source() -> String {
    helper::source()
}
// Pending wiring in the source-only preparation path (Cairn prove-source-built-mantle-fixed-point
// I3).
#[allow(dead_code)]
pub(crate) fn process_replay_source() -> String {
    helper::process_source()
}
pub(crate) fn std_process_module_source() -> String {
    std_process::module_source()
}
// Pending wiring in the source-only preparation path (Cairn prove-source-built-mantle-fixed-point
// I3).
#[allow(dead_code)]
pub(crate) fn std_process_replacements() -> [(String, String); 2] {
    std_process::replacements()
}
pub(crate) fn std_process_patch_descriptor() -> String {
    let mut words = vec![
        "MANTLE-STD-PROCESS-PATCH-1".to_string(),
        "library/std/src/process.rs".into(),
        "library/std/src/mantle_cairn_std_process.rs".into(),
        "2".into(),
    ];
    for (before, after) in std_process::replacements() {
        words.extend([before, after]);
    }
    words.push("\ninclude!(\"mantle_cairn_std_process.rs\");\n".into());
    words.into_iter().map(|word| format!("{}:{word},", word.len())).collect()
}
pub(super) use resolve::Package;

const STAGE: &str = "full-source-rust-provider";
const TRIPLE: &str = "x86_64-unknown-linux-musl";
const HELPER_PATH: &str = "cairn/rust/successors/helper.rs";
// Pending wiring in the source-only preparation path (Cairn prove-source-built-mantle-fixed-point
// I3).
#[allow(dead_code)]
const METADATA_PATH: &str = "cairn/rust/successors/metadata.rs";
pub(crate) struct Fragment {
    pub steps: Vec<RecipeStep>,
    pub files: BTreeMap<String, String>,
    pub exports: BTreeMap<String, ArtifactRef>,
}
pub(super) fn error(message: impl Into<String>) -> RunError {
    RunError::Build(format!("Rust successor source authoring: {}", message.into()))
}
pub(super) fn lit(text: impl Into<String>) -> RecipeWord {
    RecipeWord {
        parts: vec![WordPart::Literal { text: text.into() }],
    }
}
pub(super) fn input(name: &str, path: &str) -> RecipeWord {
    RecipeWord {
        parts: vec![WordPart::Input {
            name: name.into(),
            path: path.into(),
        }],
    }
}
pub(super) fn output(name: &str) -> RecipeWord {
    RecipeWord {
        parts: vec![WordPart::Output {
            name: name.into(),
            path: String::new(),
        }],
    }
}
fn attached(prefix: &str, name: &str, path: &str) -> RecipeWord {
    RecipeWord {
        parts: vec![WordPart::Literal { text: prefix.into() }, WordPart::Input {
            name: name.into(),
            path: path.into(),
        }],
    }
}
pub(super) fn product(id: &str, name: &str) -> ArtifactRef {
    ArtifactRef::Produced {
        action_id: format!("{STAGE}/{id}"),
        output_id: name.into(),
    }
}
pub(super) fn spec(kind: ArtifactKind) -> RecipeOutput {
    RecipeOutput {
        kind,
        bytes_max: 8 * 1024 * 1024 * 1024,
    }
}
pub(super) fn outputs(name: &str, kind: ArtifactKind) -> BTreeMap<String, RecipeOutput> {
    BTreeMap::from([
        (name.into(), spec(kind)),
        ("stdout".into(), RecipeOutput {
            kind: ArtifactKind::Text,
            bytes_max: 64 * 1024 * 1024,
        }),
        ("stderr".into(), RecipeOutput {
            kind: ArtifactKind::Text,
            bytes_max: 64 * 1024 * 1024,
        }),
        ("exit-status".into(), RecipeOutput {
            kind: ArtifactKind::Text,
            bytes_max: 4,
        }),
    ])
}
fn require(reference: &ArtifactRef) -> Result<(), RunError> {
    if matches!(reference, ArtifactRef::Produced { .. }) {
        Ok(())
    } else {
        Err(error("compiler, sysroot and LLVM inputs must have individual producers"))
    }
}
#[derive(Clone)]
pub(super) struct Compiled {
    pub artifact: ArtifactRef,
    pub filename: String,
    pub directory: ArtifactRef,
    pub companions: BTreeMap<String, ArtifactRef>,
    pub generated: BTreeMap<String, ArtifactRef>,
}
pub(super) struct ScriptProducts {
    pub stdout: ArtifactRef,
    pub out_dir: ArtifactRef,
    pub logical_out: String,
}
pub(super) struct Context<'a, 'b> {
    pub request: &'a AuthoringRequest<'b>,
    pub source: ArtifactRef,
    pub compiler: ArtifactRef,
    pub provider: ArtifactRef,
    pub helper: ArtifactRef,
    pub metadata_helper: ArtifactRef,
    pub llvm: BTreeMap<String, ArtifactRef>,
    pub translator: ArtifactRef,
    pub version: String,
    pub prefix: String,
    pub top: String,
    pub steps: Vec<RecipeStep>,
    pub files: BTreeMap<String, String>,
    pub runtime: BTreeMap<String, ArtifactRef>,
    pub bootstrap: bool,
    pub host_provider: ArtifactRef,
    pub source_utility: ArtifactRef,
    pub mrustc_source: ArtifactRef,
    pub build_override: bool,
    pub force_unstable: bool,
    pub reservations: &'a crate::source_built_program_authoring::ContinuationReservationPolicy,
    pub world_generation: usize,
    pub profile_arguments: Vec<RecipeWord>,
    pub incremental: bool,
    pub stage_zero: bool,
    pub generated_current: BTreeMap<String, ArtifactRef>,
    pub cfg_cache: BTreeMap<ArtifactRef, (ArtifactRef, ArtifactRef)>,
    pub sdk_cache: BTreeMap<String, ArtifactRef>,
}
impl Context<'_, '_> {
    pub(super) fn id(&self, suffix: &str) -> String {
        format!("{}.{suffix}", self.prefix)
    }
    pub(super) fn native(&self, role: &str) -> Result<ArtifactRef, RunError> {
        let value = self
            .request
            .predecessor_exports
            .get(&format!("full-source-native-provider:{role}"))
            .ok_or_else(|| error(format!("missing exact native dependency {role}")))?
            .clone();
        require(&value)?;
        Ok(value)
    }
    pub(super) fn exec(
        &mut self,
        id: &str,
        executable: ArtifactRef,
        mut inputs: BTreeMap<String, ArtifactRef>,
        arguments: Vec<RecipeWord>,
        mut environment: BTreeMap<String, RecipeWord>,
        declared: BTreeMap<String, RecipeOutput>,
    ) {
        inputs.insert("executable".into(), executable);
        inputs.insert("rust-runtime".into(), self.runtime["tree"].clone());
        environment.entry("LD_LIBRARY_PATH".into()).or_insert_with(|| RecipeWord {
            parts: vec![WordPart::Runtime {
                mount: "/mantle-rust-runtime".into(),
                path: String::new(),
            }],
        });
        self.steps.push(RecipeStep {
            id: id.into(),
            inputs,
            outputs: declared,
            operation: RecipeOperation::Exec {
                executable_input: "executable".into(),
                arguments,
                environment,
                runtime_mounts: BTreeMap::from([("/mantle-rust-runtime".into(), "rust-runtime".into())]),
                cwd_input: None,
                cwd_output: None,
                output_seeds: BTreeMap::new(),
                command_state: None,
                guard: None,
                stdin_input: None,
                exit_policy: RecipeExitPolicy::Exact { code: 0 },
                merge_stderr: false,
                invocation_input: None,
            },
        });
    }
    pub(super) fn select(&mut self, id: &str, tree: ArtifactRef, path: &str, kind: ArtifactKind) -> ArtifactRef {
        self.exec(
            id,
            self.helper.clone(),
            BTreeMap::from([("source".into(), tree)]),
            vec![lit("--select"), input("source", path), output("selected")],
            BTreeMap::new(),
            outputs("selected", kind),
        );
        product(id, "selected")
    }
    pub(super) fn current_generated(&self, roots: &BTreeMap<String, ArtifactRef>) -> BTreeMap<String, ArtifactRef> {
        roots
            .iter()
            .map(|(root, reference)| (root.clone(), self.generated_current.get(root).unwrap_or(reference).clone()))
            .collect()
    }
    pub(super) fn project_generated(
        &mut self,
        id: &str,
        tmp: ArtifactRef,
        roots: &BTreeMap<String, ArtifactRef>,
    ) -> Result<BTreeMap<String, ArtifactRef>, RunError> {
        let mut result = BTreeMap::new();
        for (index, root) in roots.keys().enumerate() {
            let path = root
                .strip_prefix("/tmp/")
                .ok_or_else(|| error("captured compiler root is outside original tmp workspace"))?;
            crate::source_built_recipe::relative(path, false)?;
            let selected = self.select(&format!("{id}.captured-root-{index}"), tmp.clone(), path, ArtifactKind::Tree);
            self.generated_current.insert(root.clone(), selected.clone());
            result.insert(root.clone(), selected);
        }
        Ok(result)
    }
    #[allow(clippy::too_many_arguments)]
    pub(super) fn compile_program(
        &mut self,
        id: &str,
        tree: ArtifactRef,
        source_path: &str,
        crate_name: &str,
        crate_type: &str,
        edition: &str,
        dependencies: &[(String, ArtifactRef, String)],
        mut environment: BTreeMap<String, RecipeWord>,
        mut extra_inputs: BTreeMap<String, ArtifactRef>,
        extra_args: Vec<RecipeWord>,
        generated: &BTreeMap<String, ArtifactRef>,
    ) -> Result<Compiled, RunError> {
        let generated = self.current_generated(generated);
        let generated = &generated;
        let metadata_id = id.rsplit_once(".lib.").map_or(id, |(unit, _)| unit);
        let hash = blake3::hash(metadata_id.as_bytes()).to_hex()[..16].to_owned();
        let filename = match crate_type {
            "rlib" | "lib" => format!("lib{crate_name}-{hash}.rlib"),
            "proc-macro" | "dylib" | "rlib,dylib" => format!("lib{crate_name}-{hash}.so"),
            "bin" => crate_name.into(),
            _ => return Err(error(format!("unlowered source crate type {crate_type}"))),
        };
        extra_inputs.insert("sysroot".into(), self.provider.clone());
        extra_inputs.insert("package".into(), tree);
        extra_inputs.insert("native-sysroot".into(), self.native("native.sysroot")?);
        let mut args = vec![
            input("package", source_path),
            lit("--crate-name"),
            lit(crate_name),
            lit("--crate-type"),
            lit(crate_type),
            lit(format!("--edition={edition}")),
            lit("--target"),
            lit(TRIPLE),
            lit("--sysroot"),
            input("sysroot", ""),
            lit("--cap-lints"),
            lit("allow"),
            lit(if self.build_override {
                "-Copt-level=0"
            } else {
                "-Copt-level=3"
            }),
            lit(format!("-Cmetadata={hash}")),
            lit("-Ctarget-feature=-crt-static"),
            lit("-L"),
            input("native-sysroot", "x86_64-linux-musl/lib"),
        ];
        if self.bootstrap {
            args.extend([lit("--cfg"), lit("bootstrap")]);
        }
        if self.force_unstable {
            args.extend([lit("-Z"), lit("force-unstable-if-unmarked")]);
        }
        if self.build_override {
            args.extend([
                lit("-Cdebug-assertions=off"),
                lit("-Coverflow-checks=off"),
                lit("-Cdebuginfo=0"),
                lit("-Ccodegen-units=256"),
            ]);
        } else {
            args.extend(self.profile_arguments.clone());
        }
        let incremental = self.incremental && !self.build_override;
        if incremental {
            args.extend([lit("-C"), RecipeWord {
                parts: vec![
                    WordPart::Literal {
                        text: "incremental=".into(),
                    },
                    WordPart::Output {
                        name: "incremental".into(),
                        path: String::new(),
                    },
                ],
            }]);
        }
        if extra_inputs.contains_key("dependencies") {
            args.extend([lit("-L"), attached("dependency=", "dependencies", "")]);
        }
        environment.insert("RUSTC_BOOTSTRAP".into(), lit("1"));
        environment.insert("LD_LIBRARY_PATH".into(), RecipeWord {
            parts: vec![
                WordPart::Input {
                    name: "sysroot".into(),
                    path: "lib".into(),
                },
                WordPart::Literal { text: ":".into() },
                WordPart::Runtime {
                    mount: "/mantle-rust-runtime".into(),
                    path: String::new(),
                },
            ],
        });
        for (index, (name, reference, path)) in dependencies.iter().enumerate() {
            let alias = format!("dependency-{index}");
            extra_inputs.insert(alias.clone(), reference.clone());
            args.push(lit("--extern"));
            args.push(attached(&format!("{name}="), &alias, path));
        }
        args.extend(extra_args);
        // Crate and Cairn-helper sources are all genuine package Trees.
        let readonly_input_roots = extra_inputs
            .keys()
            .filter(|name| {
                matches!(
                    name.as_str(),
                    "package" | "sysroot" | "native-sysroot" | "metadata" | "source-overlay" | "dependencies"
                ) || name.starts_with("dependency-")
                    || name.starts_with("search-")
            })
            .cloned()
            .collect::<Vec<_>>();
        let direct_link = matches!(crate_type, "bin" | "proc-macro" | "dylib" | "rlib,dylib");
        if direct_link {
            args.extend([
                lit("-Csave-temps"),
                lit("-Clinker-flavor=gcc"),
                attached("-Clinker=", "package", "target-wrappers/cc"),
            ]);
            if crate_type == "rlib,dylib" {
                args.extend([
                    lit("--out-dir"),
                    output("link-work"),
                    lit(format!("-Cextra-filename=-{hash}")),
                ]);
            } else {
                args.extend([lit("-o"), RecipeWord {
                    parts: vec![WordPart::Output {
                        name: "link-work".into(),
                        path: filename.clone(),
                    }],
                }]);
            }
            environment.insert("MANTLE_CAIRN_LINK_RESPONSE".into(), RecipeWord {
                parts: vec![WordPart::Output {
                    name: "link-work".into(),
                    path: "native-link.argv".into(),
                }],
            });
            environment.insert("MANTLE_CAIRN_LINK_WORK".into(), output("link-work"));
            environment.insert("MANTLE_CAIRN_TARGET_CC".into(), input("package", "target-wrappers/cc"));
            environment.insert("TMPDIR".into(), output("link-work"));
            let compile = format!("{id}.rustc");
            let mut declared = outputs("link-work", ArtifactKind::Tree);
            if incremental {
                declared.insert("incremental".into(), spec(ArtifactKind::Tree));
            }
            let native_cwd = environment.get("CARGO_MANIFEST_DIR").cloned();
            self.compiler_exec(
                &compile,
                extra_inputs.clone(),
                args,
                environment,
                declared,
                generated,
                &readonly_input_roots,
            )?;
            let compiled_generated =
                self.project_generated(&format!("{id}.compiled"), product(&compile, "generated-tmp"), generated)?;
            extra_inputs.remove("rustc");
            extra_inputs.retain(|name, _| !name.starts_with("package-field-"));
            let mut recipe = crate::source_built_recipe::SourceRecipe {
                schema: crate::source_built_recipe::SCHEMA.into(),
                stage_id: STAGE.into(),
                preparation: crate::source_built_recipe::RecipePreparation {
                    tool: "original-rust-successor".into(),
                    provenance: "original release crate graph".into(),
                },
                steps: Vec::new(),
                timeout_groups: Vec::new(),
                roots: Vec::new(),
                exports: BTreeMap::new(),
                semantic_roots: BTreeMap::new(),
            };
            extra_inputs.insert("rust-runtime".into(), self.runtime["tree"].clone());
            let link_id = format!("{id}.link");
            let controller = self
                .llvm
                .get("llvm-native-controller")
                .ok_or_else(|| error("source linker lacks original native controller"))?
                .clone();
            let commands_max = crate::source_built_recipe::controller::reserve_native_commands(
                self.reservations,
                &format!("{link_id}.native"),
                1,
                crate::source_built_recipe::STEP_COUNT_MAX,
            )?;
            // Original compile::Assemble installs the selected stage-1 compiler
            // in sysroot(compiler)/bin/rustc. The genuine target wrapper uses
            // that logical path to choose its dynamic-compiler branch.
            let logical_output = if crate_type == "bin" && crate_name == "rustc_main" {
                "stage1/bin/rustc".to_owned()
            } else {
                filename.clone()
            };
            let artifact = super::translation::append_link_response(
                self.request,
                &mut recipe,
                &link_id,
                extra_inputs,
                product(&compile, "link-work"),
                self.runtime["tree"].clone(),
                BTreeMap::from([("/mantle-rust-runtime".into(), "rust-runtime".into())]),
                "artifact",
                if crate_type == "bin" {
                    ArtifactKind::Executable
                } else {
                    ArtifactKind::Object
                },
                &compiled_generated,
                native_cwd,
                readonly_input_roots.into_iter().collect(),
                controller,
                self.translator.clone(),
                &logical_output,
                commands_max,
            )?;
            self.steps.extend(recipe.steps);
            let generated = if compiled_generated.is_empty() {
                BTreeMap::new()
            } else {
                let tmp = recipe
                    .exports
                    .get(&format!("{id}.link.generated-tmp"))
                    .ok_or_else(|| error("original linker omitted its actual temporary workspace"))?
                    .clone();
                self.project_generated(&format!("{id}.linked"), tmp, &compiled_generated)?
            };
            if crate_type == "rlib,dylib" {
                let rlib_name = format!("lib{crate_name}-{hash}.rlib");
                let rlib = self.select(
                    &format!("{id}.rlib"),
                    product(&compile, "link-work"),
                    &rlib_name,
                    ArtifactKind::Archive,
                );
                let directory_id = format!("{id}.crate-directory");
                self.steps.push(RecipeStep {
                    id: directory_id.clone(),
                    inputs: BTreeMap::from([("rlib".into(), rlib.clone()), ("dylib".into(), artifact.clone())]),
                    outputs: BTreeMap::from([("tree".into(), spec(ArtifactKind::Tree))]),
                    operation: RecipeOperation::AssembleTree {
                        output: "tree".into(),
                        entries: BTreeMap::from([
                            (rlib_name.clone(), "rlib".into()),
                            (filename.clone(), "dylib".into()),
                        ]),
                    },
                });
                Ok(Compiled {
                    artifact: rlib,
                    filename: rlib_name,
                    directory: product(&directory_id, "tree"),
                    companions: BTreeMap::from([(filename, artifact)]),
                    generated,
                })
            } else {
                let directory = self.artifact_directory(id, &filename, artifact.clone());
                Ok(Compiled {
                    artifact,
                    filename,
                    directory,
                    companions: BTreeMap::new(),
                    generated,
                })
            }
        } else {
            args.extend([lit("-o"), output("artifact")]);
            let mut declared = outputs("artifact", ArtifactKind::Archive);
            if incremental {
                declared.insert("incremental".into(), spec(ArtifactKind::Tree));
            }
            self.compiler_exec(id, extra_inputs, args, environment, declared, generated, &readonly_input_roots)?;
            let artifact = product(id, "artifact");
            let directory = self.artifact_directory(id, &filename, artifact.clone());
            let generated = self.project_generated(id, product(id, "generated-tmp"), generated)?;
            Ok(Compiled {
                artifact,
                filename,
                directory,
                companions: BTreeMap::new(),
                generated,
            })
        }
    }
    fn artifact_directory(&mut self, id: &str, filename: &str, artifact: ArtifactRef) -> ArtifactRef {
        let id = format!("{id}.crate-directory");
        self.steps.push(RecipeStep {
            id: id.clone(),
            inputs: BTreeMap::from([("artifact".into(), artifact)]),
            outputs: BTreeMap::from([("tree".into(), spec(ArtifactKind::Tree))]),
            operation: RecipeOperation::AssembleTree {
                output: "tree".into(),
                entries: BTreeMap::from([(filename.into(), "artifact".into())]),
            },
        });
        product(&id, "tree")
    }
    fn compiler_exec(
        &mut self,
        id: &str,
        mut inputs: BTreeMap<String, ArtifactRef>,
        arguments: Vec<RecipeWord>,
        environment: BTreeMap<String, RecipeWord>,
        mut declared: BTreeMap<String, RecipeOutput>,
        generated: &BTreeMap<String, ArtifactRef>,
        readonly_input_roots: &[String],
    ) -> Result<(), RunError> {
        let cwd = environment.get("CARGO_MANIFEST_DIR").cloned();
        let mut workspace_mounts = BTreeMap::new();
        let mut output_seeds = BTreeMap::new();
        if !generated.is_empty() {
            let mut seed_inputs = BTreeMap::new();
            let mut entries = BTreeMap::new();
            for (index, (root, tree)) in generated.iter().enumerate() {
                require(tree)?;
                let path = root
                    .strip_prefix("/tmp/")
                    .ok_or_else(|| error("captured source output is outside original temporary workspace"))?;
                crate::source_built_recipe::relative(path, false)?;
                let alias = format!("root-{index}");
                seed_inputs.insert(alias.clone(), tree.clone());
                entries.insert(path.into(), alias);
            }
            let seed = format!("{id}.generated-tmp");
            self.steps.push(RecipeStep {
                id: seed.clone(),
                inputs: seed_inputs,
                outputs: BTreeMap::from([("tree".into(), spec(ArtifactKind::Tree))]),
                operation: RecipeOperation::AssembleTree {
                    output: "tree".into(),
                    entries,
                },
            });
            inputs.insert("generated-tmp-seed".into(), product(&seed, "tree"));
            declared.insert("generated-tmp".into(), spec(ArtifactKind::Tree));
            output_seeds.insert("generated-tmp".into(), "generated-tmp-seed".into());
            workspace_mounts.insert("/tmp".into(), crate::source_built_recipe::RecipeWorkspaceMount {
                output: "generated-tmp".into(),
                path: String::new(),
            });
        }
        let invocation = crate::source_built_recipe::RecipeInvocation {
            arguments,
            environment,
            cwd,
            argv0: None,
            executable_path: None,
            workspace_mounts,
            readonly_input_roots: readonly_input_roots.to_vec(),
            process_attributes: None,
            descriptors: None,
        };
        let text = serde_json::to_string(&invocation)
            .map_err(|reason| error(format!("source compiler invocation: {reason}")))?;
        let frame = format!("{id}.invocation");
        let bytes_max = text.len() as u64;
        self.steps.push(RecipeStep {
            id: frame.clone(),
            inputs: BTreeMap::new(),
            outputs: BTreeMap::from([("invocation".into(), RecipeOutput {
                kind: ArtifactKind::Text,
                bytes_max,
            })]),
            operation: RecipeOperation::WriteText {
                output: "invocation".into(),
                text,
            },
        });
        inputs.insert("invocation".into(), product(&frame, "invocation"));
        inputs.insert("executable".into(), self.compiler.clone());
        inputs.insert("rust-runtime".into(), self.runtime["tree"].clone());
        self.steps.push(RecipeStep {
            id: id.into(),
            inputs,
            outputs: declared,
            operation: RecipeOperation::Exec {
                executable_input: "executable".into(),
                invocation_input: Some("invocation".into()),
                arguments: Vec::new(),
                environment: BTreeMap::new(),
                runtime_mounts: BTreeMap::from([("/mantle-rust-runtime".into(), "rust-runtime".into())]),
                cwd_input: None,
                cwd_output: None,
                stdin_input: None,
                merge_stderr: false,
                output_seeds,
                command_state: None,
                guard: None,
                exit_policy: RecipeExitPolicy::Exact { code: 0 },
            },
        });
        Ok(())
    }
}

fn package_environment(package: &Package, version: &str) -> BTreeMap<String, RecipeWord> {
    let mut env = BTreeMap::new();
    for (key, value) in [
        ("CARGO_PKG_NAME", package.name.as_str()),
        ("CARGO_PKG_VERSION", package.version.as_str()),
        ("HOST", TRIPLE),
        ("TARGET", TRIPLE),
        ("PROFILE", "release"),
        ("OPT_LEVEL", "3"),
        ("DEBUG", "false"),
        ("NUM_JOBS", "1"),
        ("RUSTC_BOOTSTRAP", "1"),
        ("CFG_RELEASE", version),
        ("CFG_RELEASE_CHANNEL", "stable"),
        ("CFG_VERSION", version),
        ("CFG_RELEASE_NUM", version),
        ("CFG_LIBDIR_RELATIVE", "lib"),
        ("CFG_DEFAULT_CODEGEN_BACKEND", "llvm"),
        ("CFG_DISABLE_UNSTABLE_FEATURES", "1"),
        ("CFG_COMPILER_HOST_TRIPLE", TRIPLE),
        ("DOC_RUST_LANG_ORG_CHANNEL", "stable"),
        ("LZMA_API_STATIC", "1"),
        ("LIBZ_SYS_STATIC", "1"),
    ] {
        env.insert(key.into(), lit(value));
    }
    let mut parts = package.version.splitn(2, '-');
    let numbers = parts.next().unwrap_or("").split('.').collect::<Vec<_>>();
    for (i, key) in ["MAJOR", "MINOR", "PATCH"].into_iter().enumerate() {
        env.insert(format!("CARGO_PKG_VERSION_{key}"), lit(numbers.get(i).copied().unwrap_or("0")));
    }
    env.insert("CARGO_PKG_VERSION_PRE".into(), lit(parts.next().unwrap_or("")));
    env.insert("CARGO_MANIFEST_DIR".into(), input("package", ""));
    for feature in &package.features {
        env.insert(format!("CARGO_FEATURE_{}", feature.replace('-', "_").to_uppercase()), lit("1"));
    }
    if let Some(links) = &package.links {
        env.insert("CARGO_MANIFEST_LINKS".into(), lit(links));
    }
    env
}
fn library(package: &Package, sources: &resolve::Sources) -> Option<(String, String, String)> {
    let lib = package.manifest.get("lib");
    let path = lib.and_then(|v| v.get("path")).and_then(toml::Value::as_str).unwrap_or("src/lib.rs");
    if lib.is_none() && !sources.members.contains(&format!("{}/{path}", package.directory)) {
        return None;
    }
    let name = lib
        .and_then(|v| v.get("name"))
        .and_then(toml::Value::as_str)
        .unwrap_or(&package.name)
        .replace('-', "_");
    let kind = if lib.and_then(|v| v.get("proc-macro")).and_then(toml::Value::as_bool) == Some(true) {
        "proc-macro"
    } else {
        "rlib"
    };
    Some((name, path.into(), kind.into()))
}
