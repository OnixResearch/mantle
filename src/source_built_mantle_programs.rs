//! Preparation-time lowering of the source-only Cargo-free Mantle unit graph.
//! No compiler, Cargo executable, or previously generated build output is read.
use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::path::Path;
use std::path::PathBuf;

use crate::errors::RunError;
use crate::rust_plan::CargoFreeUnitTopology;
use crate::rust_plan::RustPlanOptions;
use crate::rust_plan::RustUnitDerivationSummary;
use crate::source_built_fixed_point::SourceAuthorityRole;
use crate::source_built_program_authoring::AuthoringRequest;
use crate::source_built_recipe::ArtifactKind;
use crate::source_built_recipe::ArtifactRef;
use crate::source_built_recipe::RecipeExitPolicy;
use crate::source_built_recipe::RecipeOperation;
use crate::source_built_recipe::RecipeOutput;
use crate::source_built_recipe::RecipePreparation;
use crate::source_built_recipe::RecipeStep;
use crate::source_built_recipe::RecipeWord;
use crate::source_built_recipe::SourceRecipe;
use crate::source_built_recipe::WordPart;

#[path = "source_built_mantle_programs_aws.rs"]
pub(super) mod aws;
#[path = "source_built_mantle_programs_compression.rs"]
pub(super) mod compression;
#[path = "source_built_mantle_programs_graph.rs"]
mod graph;
#[path = "source_built_mantle_programs_metadata.rs"]
pub(super) mod metadata;
#[path = "source_built_mantle_programs_native.rs"]
pub(super) mod native;
#[path = "source_built_mantle_programs_rust_probes.rs"]
pub(super) mod rust_probes;

pub(crate) const PROGRAM_RECORD: &str = "source-built-mantle-programs";

pub(crate) fn source_files() -> BTreeMap<String, String> {
    graph::source_files()
}

pub(crate) fn author(
    request: &AuthoringRequest<'_>,
) -> Result<crate::source_built_program_authoring::AuthoredPrograms, RunError> {
    graph::author(request)
}

pub(super) fn error(message: impl Into<String>) -> RunError {
    RunError::Build(format!("Mantle source authoring: {}", message.into()))
}

pub(super) fn literal(text: impl Into<String>) -> RecipeWord {
    RecipeWord {
        parts: vec![WordPart::Literal { text: text.into() }],
    }
}

pub(super) fn input(name: impl Into<String>, path: impl Into<String>) -> RecipeWord {
    RecipeWord {
        parts: vec![WordPart::Input {
            name: name.into(),
            path: path.into(),
        }],
    }
}

pub(super) fn output(name: impl Into<String>, path: impl Into<String>) -> RecipeWord {
    RecipeWord {
        parts: vec![WordPart::Output {
            name: name.into(),
            path: path.into(),
        }],
    }
}

pub(super) fn prefixed(prefix: &str, mut word: RecipeWord) -> RecipeWord {
    word.parts.insert(0, WordPart::Literal { text: prefix.into() });
    word
}

#[derive(Clone)]
pub(super) struct UnitProduct {
    pub reference: ArtifactRef,
    pub member: String,
    pub kind: ArtifactKind,
    /// Preserve the compiler's crate identity in the dependency input basename.
    pub filename: String,
    pub dependencies: BTreeMap<String, ArtifactRef>,
    pub dependency_directories: BTreeSet<String>,
}

pub(super) struct UnitSource {
    pub original: ArtifactRef,
    pub package: ArtifactRef,
    pub compiler_path: String,
    pub package_compiler_path: String,
    pub package_name: String,
}

pub(super) struct Stage<'a> {
    pub request: &'a AuthoringRequest<'a>,
    pub id: String,
    pub runtime_root: PathBuf,
    pub steps: Vec<RecipeStep>,
    pub timeout_groups: Vec<crate::source_built_recipe::RecipeTimeoutGroup>,
    pub products: BTreeMap<String, UnitProduct>,
    pub semantic_roots: BTreeMap<String, Vec<ArtifactRef>>,
    pub runtime_bindings: BTreeMap<String, ArtifactRef>,
    pub runtime_aliases: BTreeMap<String, String>,
    pub metadata_helper: Option<ArtifactRef>,
    pub metadata_resolvers: BTreeMap<ArtifactRef, BTreeMap<String, ArtifactRef>>,
    pub reservations: &'a crate::source_built_program_authoring::ContinuationReservationPolicy,
}

impl Stage<'_> {
    pub fn produced(&self, step: &str, output_id: &str) -> ArtifactRef {
        ArtifactRef::Produced {
            action_id: format!("{}/{step}", self.id),
            output_id: output_id.into(),
        }
    }

    pub fn predecessor(&self, stage: &str, export: &str) -> Result<ArtifactRef, RunError> {
        self.request
            .predecessor_exports
            .get(&format!("{stage}:{export}"))
            .cloned()
            .ok_or_else(|| error(format!("missing exact predecessor export {stage}:{export}")))
    }

    pub fn bound(&self, kind: ArtifactKind) -> RecipeOutput {
        RecipeOutput {
            kind,
            bytes_max: self.request.proof.resource_bounds.disk_bytes_max,
        }
    }

    /// Bind complete, separate admitted source trees; no vendor overlay is made.
    pub fn source_inputs(&self) -> BTreeMap<String, ArtifactRef> {
        BTreeMap::from([
            ("mantle-source".into(), ArtifactRef::Source {
                role: SourceAuthorityRole::MantleSource,
                path: String::new(),
            }),
            ("vendor-inputs".into(), ArtifactRef::Source {
                role: SourceAuthorityRole::VendorInputs,
                path: String::new(),
            }),
        ])
    }

    pub fn source_word(&self, value: &str) -> Result<RecipeWord, RunError> {
        let path = Path::new(value);
        let absolute;
        let path = if let Ok(member) = path.strip_prefix("/mantle/release/source") {
            absolute = self
                .request
                .source_roots
                .get(&SourceAuthorityRole::MantleSource)
                .ok_or_else(|| error("missing admitted Mantle source"))?
                .join(member);
            &absolute
        } else if path.is_absolute() {
            path
        } else {
            absolute = self
                .request
                .source_roots
                .get(&SourceAuthorityRole::MantleSource)
                .ok_or_else(|| error("missing admitted Mantle source"))?
                .join(path);
            &absolute
        };
        // A separately admitted vendor tree can itself be nested physically in
        // the source directory during preparation. Its role still takes priority.
        for (role, name) in [
            (SourceAuthorityRole::VendorInputs, "vendor-inputs"),
            (SourceAuthorityRole::MantleSource, "mantle-source"),
        ] {
            let root =
                self.request.source_roots.get(&role).ok_or_else(|| error(format!("missing {role:?} source root")))?;
            if let Ok(member) = path.strip_prefix(root) {
                let member = member.to_str().ok_or_else(|| error("non-UTF-8 source operand"))?;
                crate::source_built_recipe::relative(member, true)?;
                return Ok(input(name, member));
            }
        }
        Err(error(format!("source operand is outside both admitted source roles: {value}")))
    }

    pub fn unit_source(&self, unit: &RustUnitDerivationSummary) -> Result<UnitSource, RunError> {
        let primary = unit
            .derivation
            .args
            .iter()
            .find(|argument| !argument.starts_with('-') && argument.ends_with(".rs"))
            .ok_or_else(|| error("unit has no original primary Rust source"))?;
        let primary = self.source_word(primary)?;
        let manifest = unit
            .derivation
            .env
            .get("CARGO_MANIFEST_DIR")
            .ok_or_else(|| error("unit has no source-owned manifest directory"))?;
        let original_manifest;
        let manifest = if let Ok(member) = Path::new(manifest).strip_prefix("/mantle/release/source") {
            original_manifest = self
                .request
                .source_roots
                .get(&SourceAuthorityRole::MantleSource)
                .ok_or_else(|| error("missing admitted Mantle source"))?
                .join(member);
            original_manifest.to_str().ok_or_else(|| error("non-UTF-8 manifest source"))?
        } else {
            manifest
        };
        let manifest = self.source_word(manifest)?;
        let binding = |word: &RecipeWord| -> Result<(ArtifactRef, String), RunError> {
            let [WordPart::Input { name, path }] = word.parts.as_slice() else {
                return Err(error("primary source is not an exact typed source operand"));
            };
            let role = match name.as_str() {
                "mantle-source" => SourceAuthorityRole::MantleSource,
                "vendor-inputs" => SourceAuthorityRole::VendorInputs,
                _ => return Err(error("primary source uses another authority")),
            };
            let sandbox = if path.is_empty() {
                format!("/inputs/{name}")
            } else {
                format!("/inputs/{name}/{path}")
            };
            Ok((
                ArtifactRef::Source {
                    role,
                    path: path.clone(),
                },
                sandbox,
            ))
        };
        let (original, compiler_path) = binding(&primary)?;
        let (package, package_compiler_path) = binding(&manifest)?;
        let package_name = unit
            .derivation
            .env
            .get("CARGO_PKG_NAME")
            .ok_or_else(|| error("unit has no original Cargo package name"))?
            .clone();
        Ok(UnitSource {
            original,
            package,
            compiler_path,
            package_compiler_path,
            package_name,
        })
    }

    pub fn topology(&self) -> Result<CargoFreeUnitTopology, RunError> {
        let root = self
            .request
            .source_roots
            .get(&SourceAuthorityRole::MantleSource)
            .ok_or_else(|| error("missing admitted Mantle source"))?;
        let vendor = self
            .request
            .source_roots
            .get(&SourceAuthorityRole::VendorInputs)
            .ok_or_else(|| error("missing independently admitted vendor inputs"))?;
        let options = RustPlanOptions {
            root: root.clone(),
            cargo: PathBuf::from("cargo-forbidden"),
            // Planning does not inspect this executable. The real compiler
            // capability is bound separately for every compiler action.
            rustc: PathBuf::from("/inputs/rustc"),
            targets: self.request.targets.to_vec(),
            host_triple: crate::source_built_rust_programs::source_route(self.request.source_roots)?.host_triple,
            profile: crate::rust_plan::default_profile(),
            profile_table: crate::rust_plan::load_root_profile_table(root)?,
            workspace_members: BTreeSet::new(),
            features: Vec::new(),
            all_features: false,
            no_default_features: false,
            no_cargo_oracle: true,
            deterministic_release_paths: true,
            path_remaps: crate::rust_plan::deterministic_release_path_remaps(root, Some(&self.runtime_root)),
        };
        crate::rust_plan::plan_cargo_free_unit_topology(&options, std::slice::from_ref(vendor))
    }

    pub fn compiler_inputs(&self) -> Result<BTreeMap<String, ArtifactRef>, RunError> {
        let mut inputs = self.source_inputs();
        inputs.insert("rustc".into(), self.predecessor("full-source-rust-provider", "rustc")?);
        inputs.insert(
            "rust-provider".into(),
            self.predecessor("full-source-rust-provider", "full-source-rust-provider")?,
        );
        inputs.insert("package".into(), self.predecessor("full-source-rust-provider", "native-package")?);
        for role in [
            "native.gcc",
            "native.ld",
            "native.sysroot",
            "native.libc",
            "native.libgcc",
            "native.libgcc-eh",
            "native.libstdcxx",
        ] {
            inputs.insert(role.replace('.', "-"), self.predecessor("full-source-native-provider", role)?);
        }
        inputs.extend(self.runtime_bindings.clone());
        Ok(inputs)
    }

    pub fn compile_unit(
        &mut self,
        index: usize,
        unit: &RustUnitDerivationSummary,
        effect: &mantle_rust_plan_core::ExistingUnitEffect,
        mut extra_tree_inputs: BTreeMap<String, ArtifactRef>,
        extra_arguments: Vec<RecipeWord>,
        extra_environment: BTreeMap<String, RecipeWord>,
    ) -> Result<UnitProduct, RunError> {
        if unit.unit_id != effect.unit_id.0 || unit.derivation.args != effect.arguments {
            return Err(error("unit/effect identity or native rustc arguments changed"));
        }
        let links = matches!(unit.target_kind.as_str(), "bin" | "proc-macro" | "custom-build")
            || unit.crate_types.iter().any(|kind| matches!(kind.as_str(), "dylib" | "cdylib"));
        // Extra bindings are source packages, overlays, and generated metadata Trees.
        let mut readonly_trees = if links {
            extra_tree_inputs.keys().cloned().collect::<BTreeSet<_>>()
        } else {
            BTreeSet::new()
        };
        let mut inputs = self.compiler_inputs()?;
        inputs.append(&mut extra_tree_inputs);
        let mut dependencies = BTreeMap::new();
        let mut dependency_directories = BTreeSet::new();
        for dependency in &effect.dependency_unit_ids {
            let product = self
                .products
                .get(&dependency.0)
                .ok_or_else(|| error(format!("unit {} precedes exact dependency {}", unit.unit_id, dependency.0)))?;
            dependency_directories.extend(product.dependency_directories.iter().cloned());
            if product.kind == ArtifactKind::Tree {
                dependency_directories.insert(product.filename.clone());
            }
            for (name, reference) in
                product.dependencies.iter().chain(std::iter::once((&product.filename, &product.reference)))
            {
                if let Some(previous) = dependencies.insert(name.clone(), reference.clone())
                    && previous != *reference
                {
                    return Err(error(format!("compiler dependency basename collision: {name}")));
                }
            }
        }
        inputs.extend(dependencies.clone());
        if links {
            readonly_trees.extend(dependency_directories.iter().cloned());
        }
        let mut arguments = Vec::new();
        let mut cursor = 0;
        while cursor < effect.arguments.len() {
            let argument = &effect.arguments[cursor];
            if argument == "--extern" {
                let operand = effect.arguments.get(cursor + 1).ok_or_else(|| error("missing --extern operand"))?;
                arguments.push(literal(argument));
                if operand == "proc_macro" {
                    arguments.push(literal(operand));
                } else {
                    let (name, artifact) =
                        operand.split_once('=').ok_or_else(|| error("malformed --extern operand"))?;
                    let dependency_id = unit
                        .dependency_artifacts
                        .iter()
                        .filter(|dependency| dependency.name == name && dependency.artifact == artifact)
                        .filter_map(|dependency| dependency.producer_unit_id.as_ref())
                        .chain(
                            unit.consumed_host_artifacts
                                .iter()
                                .filter(|dependency| dependency.artifact == artifact)
                                .filter_map(|dependency| dependency.producer_unit_id.as_ref()),
                        )
                        .collect::<BTreeSet<_>>();
                    if dependency_id.len() != 1 {
                        return Err(error(format!("ambiguous exact extern producer {operand}")));
                    }
                    let product = self
                        .products
                        .get(*dependency_id.first().expect("one producer"))
                        .ok_or_else(|| error(format!("unlowered exact extern producer {operand}")))?;
                    arguments.push(prefixed(&format!("{name}="), input(&product.filename, &product.member)));
                }
                cursor += 2;
                continue;
            }
            if argument == "--remap-path-prefix" {
                let remap = effect.arguments.get(cursor + 1).ok_or_else(|| error("missing path remap operand"))?;
                let (source, destination) = remap.rsplit_once('=').ok_or_else(|| error("malformed path remap"))?;
                arguments.push(literal(argument));
                let word = if Path::new(source) == self.runtime_root {
                    self.image_word(&mut inputs, &format!("/outputs={destination}"))?
                } else {
                    let mut word = self.source_word(source)?;
                    word.parts.push(WordPart::Literal { text: "=".into() });
                    word.parts.extend(self.image_word(&mut inputs, destination)?.parts);
                    word
                };
                arguments.push(word);
                cursor += 2;
                continue;
            }
            if argument == "-C" && effect.arguments.get(cursor + 1).is_some_and(|value| value.starts_with("linker=")) {
                cursor += 2;
                continue;
            }
            if argument.starts_with("-Clinker=") {
                cursor += 1;
                continue;
            }
            if !argument.starts_with('-') && argument.ends_with(".rs") {
                arguments.push(self.source_word(argument)?);
            } else if let Some(remap) = argument.strip_prefix("--remap-path-prefix=") {
                let (source, destination) = remap.rsplit_once('=').ok_or_else(|| error("malformed path remap"))?;
                if Path::new(source) == self.runtime_root {
                    arguments
                        .push(self.image_word(&mut inputs, &format!("--remap-path-prefix=/outputs={destination}"))?);
                } else {
                    let mut word = prefixed("--remap-path-prefix=", self.source_word(source)?);
                    word.parts.push(WordPart::Literal { text: "=".into() });
                    word.parts.extend(self.image_word(&mut inputs, destination)?.parts);
                    arguments.push(word);
                }
            } else {
                arguments.push(literal(argument));
            }
            cursor += 1;
        }
        arguments.extend([literal("--sysroot"), input("rust-provider", "")]);
        for name in &dependency_directories {
            arguments.push(prefixed("-Ldependency=", input(name, "")));
        }
        if unit.execution_kind == "target" {
            arguments.extend([literal("--target"), literal(&unit.selected_triple)]);
        }
        arguments.extend(extra_arguments);
        let mut environment = BTreeMap::new();
        for (name, value) in &effect.environment {
            if name == "SOURCE_CLOSURE_DIGEST" {
                continue;
            }
            let word = if name == "CARGO_MANIFEST_DIR" {
                if unit.target_kind == "custom-build" {
                    input("source-package", "")
                } else {
                    self.source_word(value)?
                }
            } else {
                self.image_word(&mut inputs, value)?
            };
            environment.insert(name.clone(), word);
        }
        environment.extend(extra_environment);
        let step_id = format!("unit-{index:05}-compile");
        let crate_name = effect
            .arguments
            .windows(2)
            .find(|pair| pair[0] == "--crate-name")
            .map(|pair| pair[1].clone())
            .ok_or_else(|| error("unit has no original crate name"))?;
        let filename = if unit.target_kind == "proc-macro" {
            format!("lib{crate_name}-{}.so", unit.rustc_metadata_hash)
        } else if links {
            format!("{crate_name}-{}", unit.rustc_metadata_hash)
        } else {
            format!("lib{crate_name}-{}.rlib", unit.rustc_metadata_hash)
        };
        if links {
            arguments.push(literal("-Ctarget-feature=-crt-static"));
        }
        let product = if links {
            arguments.extend([
                literal("-Csave-temps"),
                literal("-Clinker-flavor=gcc"),
                prefixed("-Clinker=", input("package", "target-wrappers/cc")),
                literal("--out-dir"),
                output("link-work", ""),
                literal("-o"),
                output("link-work", "product"),
            ]);
            environment.insert("MANTLE_CAIRN_LINK_WORK".into(), output("link-work", ""));
            environment.insert("MANTLE_CAIRN_LINK_RESPONSE".into(), output("link-work", "native-link.argv"));
            environment.insert("MANTLE_CAIRN_TARGET_CC".into(), input("package", "target-wrappers/cc"));
            environment.insert("TMPDIR".into(), output("link-work", ""));
            self.steps.push(RecipeStep {
                id: step_id.clone(),
                inputs: inputs.clone(),
                outputs: BTreeMap::from([
                    ("link-work".into(), self.bound(ArtifactKind::Tree)),
                    ("stdout".into(), self.bound(ArtifactKind::Text)),
                    ("stderr".into(), self.bound(ArtifactKind::Text)),
                ]),
                operation: RecipeOperation::Exec {
                    command_state: None,
                    executable_input: "rustc".into(),
                    invocation_input: None,
                    merge_stderr: false,
                    arguments,
                    environment,
                    runtime_mounts: self.runtime_mounts()?,
                    cwd_input: Some("mantle-source".into()),
                    cwd_output: None,
                    stdin_input: None,
                    output_seeds: BTreeMap::new(),
                    guard: None,
                    exit_policy: RecipeExitPolicy::Exact { code: 0 },
                },
            });
            let link_id = format!("unit-{index:05}-link");
            let output_id = if unit.target_name == "mantle" && unit.target_kind == "bin" {
                self.id.clone()
            } else {
                "product".into()
            };
            let reference =
                self.link_product(&link_id, inputs, self.produced(&step_id, "link-work"), &output_id, readonly_trees)?;
            UnitProduct {
                reference,
                member: String::new(),
                kind: ArtifactKind::Executable,
                filename,
                dependencies,
                dependency_directories,
            }
        } else {
            arguments.extend([literal("-o"), output("product", &filename)]);
            self.steps.push(RecipeStep {
                id: step_id.clone(),
                inputs,
                outputs: BTreeMap::from([
                    ("product".into(), self.bound(ArtifactKind::Tree)),
                    ("stdout".into(), self.bound(ArtifactKind::Text)),
                    ("stderr".into(), self.bound(ArtifactKind::Text)),
                ]),
                operation: RecipeOperation::Exec {
                    command_state: None,
                    executable_input: "rustc".into(),
                    invocation_input: None,
                    merge_stderr: false,
                    arguments,
                    environment,
                    runtime_mounts: self.runtime_mounts()?,
                    cwd_input: Some("mantle-source".into()),
                    cwd_output: None,
                    stdin_input: None,
                    output_seeds: BTreeMap::new(),
                    guard: None,
                    exit_policy: RecipeExitPolicy::Exact { code: 0 },
                },
            });
            UnitProduct {
                reference: self.produced(&step_id, "product"),
                member: filename.clone(),
                kind: ArtifactKind::Tree,
                filename,
                dependencies,
                dependency_directories,
            }
        };
        self.semantic_roots
            .insert(format!("rust-unit:{}", effect.unit_id.0), vec![product.reference.clone()]);
        self.products.insert(effect.unit_id.0.clone(), product.clone());
        Ok(product)
    }
}

impl Stage<'_> {
    /// Helpers are ordinary admitted source programs compiled by the exact same
    /// produced Rust compiler. They do not stand in for Cargo semantic units.
    pub fn compile_helper(&mut self, id: &str, source_path: &str) -> Result<ArtifactRef, RunError> {
        crate::source_built_recipe::relative(id, false)?;
        crate::source_built_recipe::relative(source_path, false)?;
        let mut inputs = self.compiler_inputs()?;
        inputs.insert("helper-source".into(), ArtifactRef::ManifestSource {
            role: SourceAuthorityRole::NativeSourceBundle,
            record_identity: PROGRAM_RECORD.into(),
            member: source_path.into(),
        });
        let compile_id = format!("helper-{id}-compile");
        let arguments = vec![
            literal("--crate-name"),
            literal(id.replace('-', "_")),
            literal("--edition=2024"),
            input("helper-source", ""),
            literal("--crate-type=bin"),
            literal("--emit=link"),
            literal("-Copt-level=2"),
            literal("-Cdebuginfo=0"),
            literal("-Csave-temps"),
            literal("-Ctarget-feature=-crt-static"),
            literal("-Clinker-flavor=gcc"),
            prefixed("-Clinker=", input("package", "target-wrappers/cc")),
            literal("--sysroot"),
            input("rust-provider", ""),
            literal("--out-dir"),
            output("link-work", ""),
            literal("-o"),
            output("link-work", "product"),
            self.image_word(&mut inputs, "--remap-path-prefix=/inputs=/mantle/release/source")?,
            self.image_word(&mut inputs, "--remap-path-prefix=/outputs=/mantle/release/execution")?,
        ];
        let environment = BTreeMap::from([
            ("MANTLE_CAIRN_LINK_WORK".into(), output("link-work", "")),
            ("MANTLE_CAIRN_LINK_RESPONSE".into(), output("link-work", "native-link.argv")),
            ("MANTLE_CAIRN_TARGET_CC".into(), input("package", "target-wrappers/cc")),
            ("TMPDIR".into(), output("link-work", "")),
        ]);
        self.steps.push(RecipeStep {
            id: compile_id.clone(),
            inputs: inputs.clone(),
            outputs: BTreeMap::from([
                ("link-work".into(), self.bound(ArtifactKind::Tree)),
                ("stdout".into(), self.bound(ArtifactKind::Text)),
                ("stderr".into(), self.bound(ArtifactKind::Text)),
            ]),
            operation: RecipeOperation::Exec {
                command_state: None,
                executable_input: "rustc".into(),
                invocation_input: None,
                merge_stderr: false,
                arguments,
                environment,
                runtime_mounts: self.runtime_mounts()?,
                cwd_input: Some("mantle-source".into()),
                cwd_output: None,
                stdin_input: None,
                output_seeds: BTreeMap::new(),
                guard: None,
                exit_policy: RecipeExitPolicy::Exact { code: 0 },
            },
        });
        self.link_product(
            &format!("helper-{id}-link"),
            inputs,
            self.produced(&compile_id, "link-work"),
            "helper",
            BTreeSet::new(),
        )
    }

    pub fn finish(mut self, topology: &CargoFreeUnitTopology) -> Result<SourceRecipe, RunError> {
        let expected = topology
            .effects
            .iter()
            .flat_map(|effect| {
                std::iter::once(format!("rust-unit:{}", effect.unit_id.0))
                    .chain((effect.target_kind == "custom-build").then(|| format!("build-script:{}", effect.unit_id.0)))
            })
            .collect::<BTreeSet<_>>();
        if self.semantic_roots.keys().cloned().collect::<BTreeSet<_>>() != expected {
            return Err(error("incomplete exact Cargo-free unit/build-script inventory"));
        }
        let main = topology
            .effects
            .iter()
            .filter(|effect| effect.target_name == "mantle" && effect.target_kind == "bin")
            .collect::<Vec<_>>();
        if main.len() != 1 {
            return Err(error("Cargo-free inventory needs one exact Mantle binary"));
        }
        let binary = self
            .products
            .get(&main[0].unit_id.0)
            .ok_or_else(|| error("Mantle binary has no unit producer"))?
            .reference
            .clone();
        let smoke = self.produced("mantle-help", "stdout");
        self.steps.push(RecipeStep {
            id: "mantle-help".into(),
            inputs: self
                .runtime_bindings
                .iter()
                .map(|(name, reference)| (name.clone(), reference.clone()))
                .chain(std::iter::once(("mantle".into(), binary.clone())))
                .collect(),
            outputs: BTreeMap::from([
                ("stdout".into(), self.bound(ArtifactKind::Text)),
                ("stderr".into(), self.bound(ArtifactKind::Text)),
            ]),
            operation: RecipeOperation::Exec {
                command_state: None,
                executable_input: "mantle".into(),
                invocation_input: None,
                merge_stderr: false,
                arguments: vec![literal("--help")],
                environment: BTreeMap::new(),
                runtime_mounts: self.runtime_mounts()?,
                cwd_input: None,
                cwd_output: None,
                stdin_input: None,
                output_seeds: BTreeMap::new(),
                guard: None,
                exit_policy: RecipeExitPolicy::Exact { code: 0 },
            },
        });
        let host = crate::source_built_rust_programs::source_route(self.request.source_roots)?.host_triple;
        let mut program = self.take_program();
        let steps = std::mem::take(&mut program.steps);
        for mut step in steps {
            if let RecipeOperation::Exec {
                environment,
                runtime_mounts,
                invocation_input,
                ..
            } = &mut step.operation
            {
                step.inputs.extend(self.runtime_bindings.clone());
                for (path, alias) in &self.runtime_aliases {
                    if let Some(previous) = runtime_mounts.insert(path.clone(), alias.clone())
                        && previous != *alias
                    {
                        return Err(error("inconsistent original runtime mount"));
                    }
                }
                if invocation_input.is_none() && !environment.contains_key("LD_LIBRARY_PATH") {
                    let mut path = RecipeWord {
                        parts: vec![WordPart::Runtime {
                            mount: "/mantle-rust-runtime".into(),
                            path: String::new(),
                        }],
                    };
                    if step.inputs.contains_key("rust-provider") {
                        for member in ["lib".to_owned(), format!("lib/rustlib/{host}/lib")] {
                            path.parts.push(WordPart::Literal { text: ":".into() });
                            path.parts.extend(input("rust-provider", &member).parts);
                        }
                    }
                    environment.insert("LD_LIBRARY_PATH".into(), path);
                }
                for (key, word) in environment.iter_mut().filter(|(key, _)| key.starts_with("CARGO_PKG_")) {
                    program.steps.extend(metadata::lower_source_data(
                        &self.id,
                        &format!("{}-source-env-{key}", step.id),
                        &mut step.inputs,
                        std::slice::from_mut(word),
                    )?);
                }
            }
            let metadata = if let RecipeOperation::Exec { environment, .. } = &step.operation {
                environment
                    .get(metadata::COMPILE_ENVIRONMENT_VARIABLE)
                    .or_else(|| environment.get(metadata::SCRIPT_ENVIRONMENT_VARIABLE))
                    .map(|word| {
                        let [WordPart::Input { name, path }] = word.parts.as_slice() else {
                            return Err(error("actual metadata environment is not an exact typed input"));
                        };
                        if path != "compiler.env" {
                            return Err(error("actual metadata environment names another member"));
                        }
                        step.inputs
                            .get(name)
                            .cloned()
                            .ok_or_else(|| error("actual metadata environment binding is absent"))
                    })
                    .transpose()?
            } else {
                None
            };
            if let Some(actual) = metadata {
                let helper = self
                    .metadata_helper
                    .clone()
                    .ok_or_else(|| error("actual metadata has no source-built invocation helper"))?;
                let resolvers = self
                    .metadata_resolvers
                    .get(&actual)
                    .ok_or_else(|| error("actual metadata has no original source resolver policy"))?;
                metadata::append_metadata_invocation(&mut program, &mut step, helper, actual, resolvers)?;
            }
            let used = crate::source_built_fixed_point_stagex_actions::used_inputs(&step);
            let unused = step
                .inputs
                .iter()
                .filter(|(_, reference)| !used.contains(reference))
                .map(|(name, _)| name.clone())
                .collect::<Vec<_>>();
            for name in unused {
                step.inputs.remove(&name);
            }
            program.steps.push(step);
        }
        self.steps = program.steps;
        Ok(SourceRecipe {
            schema: crate::source_built_recipe::SCHEMA.into(),
            stage_id: self.id.clone(),
            preparation: RecipePreparation {
                tool: "mantle-source-author".into(),
                provenance: "source-only Cargo-free topology and original generator phases".into(),
            },
            steps: self.steps,
            roots: vec![binary.clone(), smoke.clone()],
            timeout_groups: program.timeout_groups,
            exports: BTreeMap::from([(self.id, binary), ("mantle-smoke".into(), smoke)]),
            semantic_roots: self.semantic_roots,
        })
    }
}
