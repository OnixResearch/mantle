//! Whole-graph lowering of the original Cargo-free Mantle plan.
use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::path::PathBuf;

use super::Stage;
use super::UnitSource;
use super::aws;
use super::compression;
use super::error;
use super::input;
use super::literal;
use super::metadata;
use super::native;
use super::output;
use super::prefixed;
use super::rust_probes;
use crate::errors::RunError;
use crate::rust_plan::CargoFreeUnitTopology;
use crate::rust_plan::RustUnitDerivationSummary;
use crate::source_built_fixed_point::SourceAuthorityRole;
use crate::source_built_program_authoring::AuthoredPrograms;
use crate::source_built_program_authoring::AuthoringRequest;
use crate::source_built_recipe::ArtifactKind;
use crate::source_built_recipe::ArtifactRef;
use crate::source_built_recipe::RecipeExitPolicy;
use crate::source_built_recipe::RecipeOperation;
use crate::source_built_recipe::RecipeOutput;
use crate::source_built_recipe::RecipeStep;
use crate::source_built_recipe::SourceRecipe;

pub(super) fn source_files() -> BTreeMap<String, String> {
    let mut files = BTreeMap::from([(
        metadata::METADATA_HELPER_PATH.into(),
        metadata::metadata_helper_source_with_native(
            native::CC_PROCESS_SOURCE,
            compression::NATIVE_REQUEST_HELPER_SOURCE,
        ),
    )]);
    files.extend(native::source_files());
    files.extend(aws::source_files());
    files
}

fn mounted(mount: &str, path: &str) -> crate::source_built_recipe::RecipeWord {
    crate::source_built_recipe::RecipeWord {
        parts: vec![crate::source_built_recipe::WordPart::Runtime {
            mount: mount.into(),
            path: path.into(),
        }],
    }
}

fn mounted_directory(mount: &str, path: &str) -> crate::source_built_recipe::RecipeWord {
    let mut word = mounted(mount, path);
    word.parts.push(crate::source_built_recipe::WordPart::DirectorySeparator {});
    word
}

/// Input mounts and environment authority exported by one runtime stage.
type RuntimeAuthority = (BTreeMap<String, ArtifactRef>, BTreeMap<String, String>);

fn runtime(request: &AuthoringRequest<'_>) -> Result<RuntimeAuthority, RunError> {
    let prefixes = crate::source_built_native_programs::logical_prefixes(request)?;
    let mut inputs = BTreeMap::new();
    let mut mounts = BTreeMap::new();
    for (stage, export, alias, destination) in [
        ("full-source-native-provider", "native.provider", "native-provider-runtime", prefixes.provider),
        ("full-source-native-provider", "native.gcc-tree", "native-gcc-runtime", prefixes.gcc),
        ("full-source-native-provider", "native.musl-tree", "native-musl-runtime", prefixes.musl),
        ("full-source-native-provider", "native.binutils-tree", "native-binutils-runtime", prefixes.binutils),
        ("full-source-rust-provider", "rust-runtime", "rust-runtime", "/mantle-rust-runtime".into()),
    ] {
        let reference = request
            .predecessor_exports
            .get(&format!("{stage}:{export}"))
            .ok_or_else(|| error(format!("missing exact original runtime export {stage}:{export}")))?;
        inputs.insert(alias.into(), reference.clone());
        mounts.insert(destination, alias.into());
    }
    Ok((inputs, mounts))
}

impl Stage<'_> {
    pub(super) fn image_word(
        &mut self,
        inputs: &mut BTreeMap<String, ArtifactRef>,
        text: &str,
    ) -> Result<crate::source_built_recipe::RecipeWord, RunError> {
        if !text.contains(['/', '\\']) {
            return Ok(literal(text));
        }
        crate::source_built_recipe::validate_text(text)?;
        let id = format!("image-data-{:06}", self.steps.len());
        self.steps.push(RecipeStep {
            id: id.clone(),
            inputs: BTreeMap::new(),
            outputs: BTreeMap::from([("text".into(), RecipeOutput {
                kind: ArtifactKind::Text,
                bytes_max: (text.len() as u64).max(1),
            })]),
            operation: RecipeOperation::WriteText {
                output: "text".into(),
                text: text.into(),
            },
        });
        inputs.insert(id.clone(), self.produced(&id, "text"));
        Ok(crate::source_built_recipe::RecipeWord {
            parts: vec![crate::source_built_recipe::WordPart::TextInput { name: id }],
        })
    }
    pub(super) fn link_product(
        &mut self,
        id: &str,
        mut inputs: BTreeMap<String, ArtifactRef>,
        link_work: ArtifactRef,
        output_name: &str,
        mut readonly_trees: BTreeSet<String>,
    ) -> Result<ArtifactRef, RunError> {
        readonly_trees.extend(
            [
                "mantle-source",
                "vendor-inputs",
                "rust-provider",
                "native-sysroot",
                "package",
            ]
            .into_iter()
            .map(str::to_owned),
        );
        readonly_trees.extend(self.runtime_bindings.keys().cloned());
        // Only actual directory capabilities belong to the native read-only roots;
        // compiler images and source-data framing inputs are not native SDK Trees.
        inputs.retain(|name, _| readonly_trees.contains(name));
        let rust_runtime = self.predecessor("full-source-rust-provider", "rust-runtime")?;
        let native_controller = self.predecessor("full-source-rust-provider", "native-controller")?;
        let native_helper = self.predecessor("full-source-rust-provider", "native-source-boundary")?;
        let commands_max = crate::source_built_recipe::controller::reserve_native_commands(
            self.reservations,
            &format!("{id}.native"),
            1,
            crate::source_built_recipe::STEP_COUNT_MAX,
        )?;
        let runtime_mounts = self.runtime_mounts()?;
        let mut program = self.take_program();
        let product = crate::source_built_rust_programs::translation::append_link_response(
            self.request,
            &mut program,
            id,
            inputs,
            link_work,
            rust_runtime,
            runtime_mounts,
            output_name,
            ArtifactKind::Executable,
            &BTreeMap::new(),
            Some(input("mantle-source", "")),
            readonly_trees,
            native_controller,
            native_helper,
            &format!("{id}/{output_name}"),
            commands_max,
        )?;
        self.steps = program.steps;
        self.timeout_groups = program.timeout_groups;
        Ok(product)
    }
    pub fn runtime_mounts(&self) -> Result<BTreeMap<String, String>, RunError> {
        Ok(self.runtime_aliases.clone())
    }

    pub(super) fn take_program(&mut self) -> SourceRecipe {
        SourceRecipe {
            schema: crate::source_built_recipe::SCHEMA.into(),
            stage_id: self.id.clone(),
            preparation: crate::source_built_recipe::RecipePreparation {
                tool: "mantle-source-author".into(),
                provenance: "original source graph".into(),
            },
            steps: std::mem::take(&mut self.steps),
            roots: Vec::new(),
            exports: BTreeMap::new(),
            semantic_roots: BTreeMap::new(),
            timeout_groups: std::mem::take(&mut self.timeout_groups),
        }
    }

    fn native_tools(
        &self,
        helper: ArtifactRef,
        phase_helper: ArtifactRef,
        package: ArtifactRef,
        script_inputs: &BTreeMap<String, ArtifactRef>,
        script_environment: &BTreeMap<String, crate::source_built_recipe::RecipeWord>,
        metadata_bindings: &BTreeMap<String, ArtifactRef>,
    ) -> Result<native::NativeRequestTools, RunError> {
        let prefixes = crate::source_built_native_programs::logical_prefixes(self.request)?;
        let native = |name| self.predecessor("full-source-native-provider", name);
        let mut inputs = self.runtime_bindings.clone();
        for (name, export) in [("native-gcc", "native.gcc"), ("native-ar", "native.ar")] {
            inputs.insert(name.into(), native(export)?);
        }
        for (key, word) in script_environment {
            if matches!(key.as_str(), metadata::SCRIPT_ENVIRONMENT_VARIABLE | metadata::COMPILE_ENVIRONMENT_VARIABLE) {
                continue;
            }
            for part in &word.parts {
                let name = match part {
                    crate::source_built_recipe::WordPart::Input { name, .. }
                    | crate::source_built_recipe::WordPart::TextInput { name }
                    | crate::source_built_recipe::WordPart::TextSlice { name, .. } => name,
                    _ => continue,
                };
                if matches!(name.as_str(), "package" | "cc-work") {
                    continue;
                }
                if let Some(reference) = script_inputs.get(name) {
                    inputs.insert(name.clone(), reference.clone());
                } else if !inputs.contains_key(name) {
                    return Err(error(format!("native inherited environment lacks exact input {name}")));
                }
            }
        }
        inputs.extend(metadata_bindings.clone());
        inputs.insert(
            "rust-provider".into(),
            self.predecessor("full-source-rust-provider", "full-source-rust-provider")?,
        );
        let provider_lib = "x86_64-linux-musl/lib";
        let mut environment = BTreeMap::from([
            ("MANTLE_GCC_ASSEMBLER".into(), mounted(&prefixes.provider, "bin/as")),
            ("MANTLE_GCC_LINKER".into(), mounted(&prefixes.provider, "bin/ld")),
            ("MANTLE_GCC_LINK_LIBC".into(), mounted(&prefixes.provider, &format!("{provider_lib}/libc.so"))),
            ("MANTLE_GCC_LINK_LIBM".into(), mounted(&prefixes.provider, &format!("{provider_lib}/libc.so"))),
            ("MANTLE_GCC_LINK_LIBGCC".into(), mounted(&prefixes.provider, &format!("{provider_lib}/libgcc.a"))),
            (
                "MANTLE_GCC_LINK_LIBGCC_EH".into(),
                mounted(&prefixes.provider, &format!("{provider_lib}/libgcc_eh.a")),
            ),
            (
                "MANTLE_GCC_LINK_LIBGCC_S".into(),
                mounted(&prefixes.provider, &format!("{provider_lib}/libgcc_s.so.1")),
            ),
            ("MANTLE_GCC_LINK_CRT".into(), mounted(&prefixes.provider, provider_lib)),
        ]);
        let mut library_path = mounted("/mantle-rust-runtime", "");
        library_path.parts.push(crate::source_built_recipe::WordPart::Literal { text: ":".into() });
        library_path.parts.extend(mounted(&prefixes.provider, provider_lib).parts);
        let host = crate::source_built_rust_programs::source_route(self.request.source_roots)?.host_triple;
        for member in ["lib".to_owned(), format!("lib/rustlib/{host}/lib")] {
            library_path.parts.push(crate::source_built_recipe::WordPart::Literal { text: ":".into() });
            library_path.parts.extend(input("rust-provider", &member).parts);
        }
        environment.insert("LD_LIBRARY_PATH".into(), library_path);
        Ok(native::NativeRequestTools {
            native: native::NativeToolchain {
                driver: inputs["native-gcc"].clone(),
                splitter: native("native.gcc-splitter")?,
                cc1: native("native.cc1")?,
                cc1_image: mounted(&prefixes.gcc, "libexec/gcc/x86_64-unknown-linux-musl/10.5.0/cc1"),
                assembler: native("native.as")?,
                archiver: inputs["native-ar"].clone(),
                inputs,
                arguments: vec![
                    prefixed("-B", mounted_directory(&prefixes.provider, "bin")),
                    prefixed("-B", mounted_directory(&prefixes.provider, "lib/gcc/x86_64-unknown-linux-musl/10.5.0")),
                    prefixed("--sysroot=", mounted(&prefixes.provider, "x86_64-linux-musl")),
                    prefixed(
                        "-Wl,-dynamic-linker,",
                        mounted(&prefixes.provider, &format!("{provider_lib}/ld-musl-x86_64.so.1")),
                    ),
                    prefixed("-Wl,-rpath,", mounted(&prefixes.provider, provider_lib)),
                ],
                environment,
                runtime_mounts: self.runtime_mounts()?,
                assembler_arguments: Vec::new(),
            },
            helper,
            phase_helper,
            package,
            compiler_program: input("native-gcc", ""),
            archiver_program: input("native-ar", ""),
            assembler_image: mounted(&prefixes.provider, "bin/as"),
        })
    }

    fn source_path(&self, reference: &ArtifactRef) -> Result<PathBuf, RunError> {
        let ArtifactRef::Source { role, path } = reference else {
            return Err(error("preparation inspection requires original source tree"));
        };
        Ok(self.request.source_roots.get(role).ok_or_else(|| error("missing original source role"))?.join(path))
    }

    fn metadata_inputs(
        &self,
        unit: &RustUnitDerivationSummary,
        generators: &BTreeMap<String, Generator>,
    ) -> Result<(Vec<metadata::MetadataInput>, BTreeMap<String, ArtifactRef>), RunError> {
        let mut result = Vec::new();
        let mut resolvers = BTreeMap::new();
        let mut selected = BTreeSet::new();
        for host in &unit.consumed_host_artifacts {
            if host.target_kind != "custom-build" {
                continue;
            }
            let id =
                host.producer_unit_id.as_ref().ok_or_else(|| error("build-script metadata has no exact producer"))?;
            if !selected.insert((id.clone(), None)) {
                continue;
            }
            let generated =
                generators.get(id).ok_or_else(|| error(format!("metadata precedes original generator {id}")))?;
            resolvers.insert(format!("generated-{}", generated.alias), generated.out_dir.clone());
            result.push(generated.metadata(None));
        }
        for dependency in &unit.metadata_dependencies {
            let id =
                dependency.producer_unit_id.as_ref().ok_or_else(|| error("linked metadata has no exact producer"))?;
            if !selected.insert((id.clone(), Some(dependency.links.clone()))) {
                continue;
            }
            let generated = generators
                .get(id)
                .ok_or_else(|| error(format!("linked metadata precedes original generator {id}")))?;
            // These original native metadata algorithms publish OUT_DIR paths;
            // ring and mimalloc publish only rustc link directives, not DEP paths.
            if matches!(generated.package_name.as_str(), "aws-lc-sys" | "bzip2-sys" | "lzma-sys" | "zstd-sys") {
                resolvers.insert(format!("generated-{}", generated.alias), generated.out_dir.clone());
            }
            // Unlike AWS and bzip2, these original scripts export include paths
            // in the original source package, not copied OUT_DIR headers.
            if matches!(generated.package_name.as_str(), "lzma-sys" | "zstd-sys") {
                resolvers.insert(format!("source-{}", generated.alias), generated.package.clone());
            }
            result.push(generated.metadata(Some(dependency.links.clone())));
        }
        Ok((result, resolvers))
    }
}

struct Generator {
    stdout: ArtifactRef,
    out_dir: ArtifactRef,
    package: ArtifactRef,
    package_name: String,
    alias: String,
}
impl Generator {
    fn metadata(&self, links: Option<String>) -> metadata::MetadataInput {
        metadata::MetadataInput {
            stdout: self.stdout.clone(),
            out_dir: self.out_dir.clone(),
            consumer_out_dir: format!("generated-{}", self.alias),
            source_package: self.package.clone(),
            consumer_package: format!("source-{}", self.alias),
            links,
        }
    }
}

fn source_overlay(source: &UnitSource) -> metadata::LibraryOverlaySource {
    metadata::LibraryOverlaySource {
        original: source.original.clone(),
        compiler_path: source.compiler_path.clone(),
        replacements: Vec::new(),
        append: Vec::new(),
    }
}

pub(super) fn author(request: &AuthoringRequest<'_>) -> Result<AuthoredPrograms, RunError> {
    let mut files = BTreeMap::new();
    let mut exports = BTreeMap::new();
    let source_root = request
        .source_roots
        .get(&SourceAuthorityRole::MantleSource)
        .ok_or_else(|| error("missing continuation policy source"))?;
    let reservations = crate::source_built_program_authoring::read_continuation_policy(source_root)?;
    for (ordinal, id) in [("stage1", "mantle-stage1"), ("stage2", "mantle-stage2")] {
        let (runtime_bindings, runtime_aliases) = runtime(request)?;
        let mut stage = Stage {
            request,
            id: id.into(),
            runtime_root: request.scratch_root.join("cargo-free-fixed-point").join(ordinal).join("recipe"),
            steps: Vec::new(),
            products: BTreeMap::new(),
            semantic_roots: BTreeMap::new(),
            runtime_bindings,
            runtime_aliases,
            metadata_helper: None,
            metadata_resolvers: BTreeMap::new(),
            reservations: &reservations,
            timeout_groups: Vec::new(),
        };
        let topology = stage.topology()?;
        lower(&mut stage, &topology)?;
        let recipe = stage.finish(&topology)?;
        exports.extend(recipe.exports.iter().map(|(name, reference)| (format!("{id}:{name}"), reference.clone())));
        files.insert(
            format!("bootstrap/source-built/{id}.json"),
            serde_json::to_string_pretty(&recipe).map_err(|e| error(e.to_string()))?,
        );
    }
    Ok(AuthoredPrograms { files, exports })
}

fn lower(stage: &mut Stage<'_>, topology: &CargoFreeUnitTopology) -> Result<(), RunError> {
    let helper = stage.compile_helper("metadata", metadata::METADATA_HELPER_PATH)?;
    stage.metadata_helper = Some(helper.clone());
    let phase_helper = stage.compile_helper("native-phase", "source_built_mantle_native_helper.rs")?;
    let aws_helper = stage.compile_helper("aws-source", "aws-source-helper.rs")?;
    let mut original_aws = None;
    let mut generators = BTreeMap::new();
    let units = topology
        .graph
        .derivations
        .iter()
        .map(|unit| (unit.unit_id.as_str(), unit))
        .collect::<BTreeMap<_, _>>();
    for (index, effect) in topology.effects.iter().enumerate() {
        let unit = units
            .get(effect.unit_id.0.as_str())
            .ok_or_else(|| error("original effect has no exact derivation"))?;
        let source = stage.unit_source(unit)?;
        let package_key = match &source.package {
            ArtifactRef::Source { path, .. } => std::path::Path::new(path)
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or(&source.package_name),
            _ => &source.package_name,
        };
        let (metadata_inputs, metadata_bindings) = stage.metadata_inputs(unit, &generators)?;
        let mut compiler_inputs = BTreeMap::new();
        if unit.target_kind == "custom-build" {
            compiler_inputs.insert("source-package".into(), source.package.clone());
        }
        let mut compiler_arguments = Vec::new();
        let mut compiler_environment = BTreeMap::new();
        let mut script_inputs = stage.compiler_inputs()?;
        let host = crate::source_built_rust_programs::source_route(stage.request.source_roots)?.host_triple;
        let mut script_environment = metadata::build_script_environment(unit, &host, input("rustc", ""))?;
        if !metadata_inputs.is_empty() {
            let produced = metadata::metadata_response(
                &stage.id,
                &format!("unit-{index:05}-metadata"),
                helper.clone(),
                &metadata_inputs,
                stage.runtime_bindings.clone(),
                stage.runtime_mounts()?,
            )?;
            stage.metadata_resolvers.insert(produced.metadata.clone(), metadata_bindings.clone());
            compiler_inputs.extend(produced.generated_inputs.clone());
            script_inputs.extend(produced.generated_inputs);
            compiler_inputs.insert("metadata".into(), produced.metadata.clone());
            script_inputs.insert("metadata".into(), produced.metadata);
            compiler_arguments.push(prefixed("@", input("metadata", "compiler.rsp")));
            compiler_environment
                .insert(metadata::COMPILE_ENVIRONMENT_VARIABLE.into(), input("metadata", "compiler.env"));
            script_environment.insert(metadata::SCRIPT_ENVIRONMENT_VARIABLE.into(), input("metadata", "compiler.env"));
            stage.steps.push(produced.step);
        }
        let probe_schedule = if unit.target_kind == "custom-build" {
            rust_probes::selected_schedule(package_key, unit, &script_environment)?
        } else {
            None
        };
        let target = unit.derivation.env.get("TARGET").map(String::as_str).unwrap_or(&unit.selected_triple);
        let features = unit
            .derivation
            .args
            .iter()
            .enumerate()
            .filter_map(|(i, argument)| {
                let cfg = if argument == "--cfg" {
                    unit.derivation.args.get(i + 1).map(String::as_str)
                } else {
                    argument.strip_prefix("--cfg=")
                }?;
                serde_json::from_str::<String>(cfg.strip_prefix("feature=")?).ok()
            })
            .collect::<BTreeSet<_>>();
        let native_schedule = if unit.target_kind == "custom-build" {
            native::selected_schedule(package_key, &features, target)?
        } else {
            None
        };
        let compression_schedule = if unit.target_kind == "custom-build" && native_schedule.is_none() {
            compression::selected_schedule(package_key, &features, target)?
        } else {
            None
        };
        let aws_schedule = if unit.target_kind == "custom-build" {
            aws::selected_schedule(package_key, &features, target, &stage.source_path(&source.package)?)?
        } else {
            None
        };
        let mut overlays = rust_probes::library_sources(package_key, &source.package, &source.package_compiler_path)?
            .unwrap_or_default();
        if package_key == "pkg-config" {
            overlays.push(metadata::pkg_config_library_overlay(&source.package, &source.package_compiler_path)?);
        }
        overlays.extend(native::library_overlays(package_key, &source.package, &source.package_compiler_path)?);
        let mut boundaries = BTreeMap::new();
        if unit.target_kind == "custom-build" {
            let mut original = source_overlay(&source);
            if let Some(schedule) = &probe_schedule {
                original.replacements.extend(schedule.script_replacements.clone());
                original.append.extend(schedule.script_append.clone());
                boundaries.insert(source.compiler_path.clone(), schedule.boundaries.clone());
                for (member, module_boundaries) in &schedule.module_boundaries {
                    let compiler_path = format!("{}/{member}", source.package_compiler_path);
                    let original = match &source.package {
                        ArtifactRef::Source { role, path } => ArtifactRef::Source {
                            role: *role,
                            path: format!("{path}/{member}"),
                        },
                        _ => return Err(error("original script module is not admitted source")),
                    };
                    overlays.push(metadata::LibraryOverlaySource {
                        original,
                        compiler_path: compiler_path.clone(),
                        replacements: Vec::new(),
                        append: Vec::new(),
                    });
                    boundaries.insert(compiler_path, module_boundaries.clone());
                }
            } else {
                boundaries.insert(
                    source.compiler_path.clone(),
                    if aws_schedule.is_some() {
                        aws::script_boundaries()
                    } else if package_key == "cranelift-codegen" {
                        vec![metadata::ScriptProcessBoundary {
                            receiver: "cmd".into(),
                            kind: metadata::ScriptProcessKind::AbsentSpawn,
                        }]
                    } else {
                        native::script_boundaries(package_key)
                    },
                );
            }
            overlays.push(original);
        }
        let generation = if unit.target_kind == "custom-build" {
            native::generation_overlay(package_key, &source.package, &source.package_compiler_path)?
        } else {
            None
        };
        let generation = if aws_schedule.is_some() {
            Some(aws::bootstrap_overlay(source.original.clone(), &source.compiler_path))
        } else {
            generation
        };
        if let Some(generation) = generation {
            let generator_overlay = metadata::script_and_library_overlays(
                &stage.id,
                &format!("unit-{index:05}-generation-overlay"),
                helper.clone(),
                &[generation],
                Some(&source.compiler_path),
                &boundaries,
                stage.runtime_bindings.clone(),
                stage.runtime_mounts()?,
            )?;
            let mut generator_inputs = compiler_inputs.clone();
            generator_inputs.insert("source-overlay".into(), generator_overlay.tree);
            let mut generator_environment = compiler_environment.clone();
            generator_environment
                .insert(metadata::SOURCE_OVERLAY_VARIABLE.into(), input("source-overlay", "overlay.frame"));
            stage.steps.extend(generator_overlay.preparation);
            stage.steps.push(generator_overlay.step);
            let generator = stage.compile_unit(
                topology.effects.len() + index,
                unit,
                effect,
                generator_inputs,
                compiler_arguments.clone(),
                generator_environment,
            )?;
            script_inputs.insert(
                if aws_schedule.is_some() {
                    "aws-symbol-bootstrap"
                } else {
                    "ring-generator"
                }
                .into(),
                generator.reference,
            );
            script_inputs.insert("native-perl".into(), stage.predecessor("full-source-native-provider", "host.perl")?);
        }
        if !overlays.is_empty() {
            let overlay = metadata::script_and_library_overlays(
                &stage.id,
                &format!("unit-{index:05}-source-overlay"),
                helper.clone(),
                &overlays,
                (unit.target_kind == "custom-build").then_some(source.compiler_path.as_str()),
                &boundaries,
                stage.runtime_bindings.clone(),
                stage.runtime_mounts()?,
            )?;
            compiler_inputs.insert("source-overlay".into(), overlay.tree);
            compiler_environment
                .insert(metadata::SOURCE_OVERLAY_VARIABLE.into(), input("source-overlay", "overlay.frame"));
            stage.steps.extend(overlay.preparation);
            stage.steps.push(overlay.step);
        }
        let compiled =
            stage.compile_unit(index, unit, effect, compiler_inputs, compiler_arguments, compiler_environment)?;
        if unit.target_kind != "custom-build" {
            continue;
        }
        let id = format!("unit-{index:05}-script");
        let mut program = stage.take_program();
        let script_package = if package_key == "nickel-lang-parser" {
            let prepare = format!("{id}-original-package");
            program.steps.push(RecipeStep {
                id: prepare.clone(),
                inputs: BTreeMap::from([
                    ("metadata-helper".into(), helper.clone()),
                    ("original-package".into(), source.package.clone()),
                ]),
                outputs: BTreeMap::from([
                    ("package".into(), stage.bound(ArtifactKind::Tree)),
                    ("stdout".into(), stage.bound(ArtifactKind::Text)),
                    ("stderr".into(), stage.bound(ArtifactKind::Text)),
                ]),
                operation: RecipeOperation::Exec {
                    command_state: None,
                    executable_input: "metadata-helper".into(),
                    arguments: vec![
                        literal("--source-package"),
                        input("original-package", ""),
                        output("package", ""),
                        literal("src/grammar.rs"),
                    ],
                    environment: BTreeMap::new(),
                    runtime_mounts: stage.runtime_mounts()?,
                    cwd_input: None,
                    cwd_output: None,
                    output_seeds: BTreeMap::new(),
                    guard: None,
                    stdin_input: None,
                    exit_policy: RecipeExitPolicy::Exact { code: 0 },
                    merge_stderr: false,
                    invocation_input: None,
                },
            });
            stage.produced(&prepare, "package")
        } else {
            source.package.clone()
        };
        let run = if let Some(schedule) = &probe_schedule {
            rust_probes::append_probe_script(
                &mut program,
                rust_probes::ProbeScript {
                    id: &id,
                    helper: helper.clone(),
                    compiler: stage.predecessor("full-source-rust-provider", "rustc")?,
                    script: compiled.reference,
                    package: script_package.clone(),
                    environment: script_environment,
                    runtime_inputs: script_inputs,
                    runtime_mounts: stage.runtime_mounts()?,
                    generated_inputs: metadata_bindings.clone(),
                },
                schedule,
            )?
        } else if native_schedule.is_some() || compression_schedule.is_some() || aws_schedule.is_some() {
            script_environment.insert("CC".into(), input("native-gcc", ""));
            script_environment.insert("AR".into(), input("native-ar", ""));
            let tools = stage.native_tools(
                helper.clone(),
                phase_helper.clone(),
                script_package.clone(),
                &script_inputs,
                &script_environment,
                &metadata_bindings,
            )?;
            for (key, value) in &tools.native.environment {
                script_environment.entry(key.clone()).or_insert_with(|| value.clone());
            }
            if let Some(schedule) = aws_schedule {
                let original = match original_aws.clone() {
                    Some(original) => original,
                    None => {
                        let original = aws::append_original_source(&mut program, stage, aws_helper.clone())?;
                        original_aws = Some(original.clone());
                        original
                    }
                };
                aws::append_schedule(
                    &mut program,
                    &tools,
                    &schedule,
                    &id,
                    compiled.reference,
                    script_environment,
                    script_inputs,
                    stage.runtime_mounts()?,
                    &aws::AwsTools {
                        helper: aws_helper.clone(),
                        original_aws: original,
                        perl: stage.predecessor("full-source-native-provider", "host.perl")?,
                        nm: stage.predecessor("full-source-native-provider", "native.nm")?,
                    },
                )?
            } else if let Some(schedule) = native_schedule {
                native::append_schedule(
                    &mut program,
                    &tools,
                    &schedule,
                    &id,
                    compiled.reference,
                    script_environment,
                    script_inputs,
                    stage.runtime_mounts()?,
                )?
            } else {
                compression::append_schedule(
                    &mut program,
                    &tools,
                    compression_schedule.as_ref().expect("selected compression schedule"),
                    &id,
                    compiled.reference,
                    script_environment,
                    script_inputs,
                    stage.runtime_mounts()?,
                )?
            }
        } else {
            metadata::original_script_run(
                &stage.id,
                &id,
                compiled.reference,
                script_package,
                script_environment,
                script_inputs,
                stage.runtime_mounts()?,
            )?
        };
        let generator = Generator {
            stdout: run.stdout.clone(),
            out_dir: run.out_dir.clone(),
            package: source.package,
            package_name: source.package_name,
            alias: format!("{index:05}"),
        };
        program.steps.push(run.step);
        stage.steps = program.steps;
        stage.timeout_groups = program.timeout_groups;
        stage.semantic_roots.insert(format!("build-script:{}", unit.unit_id), vec![
            generator.stdout.clone(),
            generator.out_dir.clone(),
        ]);
        generators.insert(unit.unit_id.clone(), generator);
    }
    Ok(())
}
