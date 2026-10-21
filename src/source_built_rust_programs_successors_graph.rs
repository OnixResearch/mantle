//! Original release crate/build-script topology and the run_rustc/std/compiler
//! cutover. No Cargo or bootstrap executable is an executor in this graph.
use std::collections::BTreeMap;
use std::collections::BTreeSet;

use super::Compiled;
use super::Context;
use super::Fragment;
use super::HELPER_PATH;
use super::Package;
use super::STAGE;
use super::ScriptProducts;
use super::TRIPLE;
use super::error;
use super::input;
use super::library;
use super::lit;
use super::output;
use super::outputs;
use super::package_environment;
use super::product;
use super::require;
use super::resolve;
use crate::errors::RunError;
use crate::source_built_program_authoring::AuthoringRequest;
use crate::source_built_recipe::ArtifactKind;
use crate::source_built_recipe::ArtifactRef;
use crate::source_built_recipe::RecipeOperation;
use crate::source_built_recipe::RecipeStep;
use crate::source_built_recipe::RecipeWord;
use crate::source_built_recipe::WordPart;

/// Declared environment words and their producer inputs for one package.
type PackageEnvironment = (BTreeMap<String, RecipeWord>, BTreeMap<String, ArtifactRef>);
/// Direct dependency aliases, compiled closure, and generated inputs.
type PackageDependencies =
    (Vec<(String, ArtifactRef, String)>, BTreeMap<String, Compiled>, BTreeMap<String, ArtifactRef>);

#[derive(Clone)]
struct Unit {
    primary: Compiled,
    variants: Vec<Compiled>,
    closure: BTreeMap<String, Compiled>,
    generated: BTreeMap<String, ArtifactRef>,
}
struct Group {
    programs: BTreeMap<String, Compiled>,
    libraries: BTreeMap<String, ArtifactRef>,
}
impl Context<'_, '_> {
    fn tree(&mut self, id: &str, entries: BTreeMap<String, ArtifactRef>) -> ArtifactRef {
        if entries.is_empty() {
            self.exec(
                id,
                self.helper.clone(),
                BTreeMap::new(),
                vec![lit("--empty"), output("tree")],
                BTreeMap::new(),
                outputs("tree", ArtifactKind::Tree),
            );
            return product(id, "tree");
        }
        let mut inputs = BTreeMap::new();
        let mut paths = BTreeMap::new();
        for (index, (path, artifact)) in entries.into_iter().enumerate() {
            let alias = format!("member-{index}");
            inputs.insert(alias.clone(), artifact);
            paths.insert(path, alias);
        }
        self.steps.push(RecipeStep {
            id: id.into(),
            inputs,
            outputs: BTreeMap::from([("tree".into(), super::spec(ArtifactKind::Tree))]),
            operation: RecipeOperation::AssembleTree {
                output: "tree".into(),
                entries: paths,
            },
        });
        product(id, "tree")
    }
    fn world(&mut self, source: ArtifactRef) -> Result<(), RunError> {
        self.world_generation += 1;
        let id = self.id(&format!("world-{}", self.world_generation));
        let tools = self.tree(
            &format!("{id}.tools"),
            [
                ("cc", "native.gcc"),
                ("c++", "native.gxx"),
                ("ar", "native.ar"),
                ("cc1", "native.cc1"),
                ("cc1plus", "native.cc1plus"),
                ("as", "native.as"),
                ("ld", "native.ld"),
                ("collect2", "native.collect2-query"),
            ]
            .into_iter()
            .map(|(name, role)| Ok((name.into(), self.native(role)?)))
            .collect::<Result<_, RunError>>()?,
        );
        let mut entries = BTreeMap::from([
            ("source".into(), source),
            ("rustc".into(), self.compiler.clone()),
            ("sysroot".into(), self.provider.clone()),
            ("runtime".into(), self.runtime["tree"].clone()),
            ("native-provider".into(), self.native("native.provider")?),
            ("native-sysroot".into(), self.native("native.sysroot")?),
            ("phase-tools".into(), tools),
        ]);
        for (key, path) in [("llvm-root", "llvm"), ("llvm-config", "llvm-config")] {
            let reference =
                self.llvm.get(key).ok_or_else(|| error(format!("missing same-release LLVM product {key}")))?.clone();
            require(&reference)?;
            entries.insert(path.into(), reference);
        }
        entries.insert("linux-headers".into(), self.native("linux.headers")?);
        super::native::extend_world(self.request, &mut entries)?;
        if self.sdk_cache.is_empty() {
            let base = self.tree(&format!("{id}.sdk-world"), entries.clone());
            for (cxx, name) in [(false, "native-sdk-cc"), (true, "native-sdk-cxx")] {
                let mut flags = super::super::translation::sdk_arguments(cxx, true);
                for word in &mut flags {
                    for part in &mut word.parts {
                        if let WordPart::Input { name, path } = part {
                            let prefix = match name.as_str() {
                                "phase-tools" => "phase-tools",
                                "native-provider" => "native-provider",
                                "native-sysroot" => "native-sysroot",
                                "rust-runtime" => "runtime",
                                _ => continue,
                            };
                            *path = if path.is_empty() {
                                prefix.into()
                            } else {
                                format!("{prefix}/{path}")
                            };
                            *name = "package".into();
                        }
                    }
                }
                let producer = format!("{id}.{name}");
                let mut arguments = vec![lit("--native-sdk"), output("response")];
                arguments.extend(flags);
                self.exec(
                    &producer,
                    self.helper.clone(),
                    BTreeMap::from([("package".into(), base.clone())]),
                    arguments,
                    BTreeMap::new(),
                    outputs("response", ArtifactKind::Text),
                );
                self.sdk_cache.insert(name.into(), product(&producer, "response"));
            }
        }
        entries.extend(self.sdk_cache.clone());
        if !self.sdk_cache.contains_key("target-wrappers") {
            let package = self.tree(&format!("{id}.wrapper-world"), entries.clone());
            let empty = format!("{id}.wrapper-empty");
            self.exec(
                &empty,
                self.helper.clone(),
                BTreeMap::new(),
                vec![lit("--empty"), output("tree")],
                BTreeMap::new(),
                outputs("tree", ArtifactKind::Tree),
            );
            let initial =
                self.tree(&format!("{id}.wrapper-initial"), BTreeMap::from([("tmp".into(), product(&empty, "tree"))]));
            let mut recipe=crate::source_built_recipe::SourceRecipe{schema:crate::source_built_recipe::SCHEMA.into(),stage_id:STAGE.into(),preparation:crate::source_built_recipe::RecipePreparation{tool:"original-rust-target-wrapper-generator".into(),provenance:"Unchanged admitted Rust source-provider target CC generator and original CXX companion algorithm; original SDK response remains a source-produced input.".into()},steps:Vec::new(),timeout_groups:Vec::new(),roots:Vec::new(),exports:BTreeMap::new(),semantic_roots:BTreeMap::new()};
            let controller = self
                .llvm
                .get("llvm-native-controller")
                .ok_or_else(|| error("original wrapper source-shell controller absent"))?
                .clone();
            let workspace = super::append_target_wrappers(
                self.request,
                &mut recipe,
                &format!("{id}.target-wrappers"),
                package,
                controller,
                self.runtime["tree"].clone(),
                false,
                "phase-tools/cc",
                "phase-tools/c++",
                initial,
            )?;
            self.steps.extend(recipe.steps);
            let wrappers =
                self.select(&format!("{id}.wrapper-directory"), workspace, "tmp/target-wrappers", ArtifactKind::Tree);
            self.sdk_cache.insert("target-wrappers".into(), wrappers.clone());
            entries.insert("target-wrappers".into(), wrappers);
        }
        self.source = self.tree(&id, entries);
        Ok(())
    }
    fn package_env(
        &mut self,
        package: &Package,
        sources: &resolve::Sources,
        unit: &str,
    ) -> Result<PackageEnvironment, RunError> {
        let mut environment = package_environment(package, &self.version);
        let manifest = format!("{}/{}", self.top, package.key.manifest);
        let workspace = sources.workspace(&package.key.manifest).map(|(directory, _)| {
            if directory.is_empty() {
                format!("{}/Cargo.toml", self.top)
            } else {
                format!("{}/{directory}/Cargo.toml", self.top)
            }
        });
        let id = format!("{unit}.package-fields");
        let runtime = BTreeMap::from([("rust-runtime".into(), self.runtime["tree"].clone())]);
        let (step, fields) = super::super::minicargo::package_fields(
            &id,
            self.source.clone(),
            &manifest,
            workspace.as_deref(),
            self.source_utility.clone(),
            runtime,
            BTreeMap::from([("/mantle-rust-runtime".into(), "rust-runtime".into())]),
        )?;
        self.steps.push(step);
        let mut inputs = BTreeMap::new();
        for (key, reference) in fields {
            let alias = format!("package-field-{}", key.to_ascii_lowercase().replace('_', "-"));
            inputs.insert(alias.clone(), reference);
            environment.insert(key, RecipeWord {
                parts: vec![WordPart::TextInput { name: alias }],
            });
        }
        environment
            .insert("CARGO_MANIFEST_DIR".into(), input("package", &format!("{}/{}", self.top, package.directory)));
        environment.insert("RUSTC".into(), input("package", "target-wrappers/rustc"));
        environment.insert("CC".into(), input("package", "target-wrappers/cc"));
        environment.insert("CXX".into(), input("package", "target-wrappers/c++"));
        environment.insert("AR".into(), input("package", "phase-tools/ar"));
        environment.insert("LLVM_CONFIG".into(), input("package", "llvm/bin/llvm-config"));
        for (key, path) in [
            ("COMPILER_PATH", "phase-tools"),
            ("MANTLE_GCC_ASSEMBLER", "phase-tools/as"),
            ("MANTLE_GCC_LINKER", "phase-tools/ld"),
            ("MANTLE_GCC_LINK_LIBC", "runtime/libc.so"),
            ("MANTLE_GCC_LINK_LIBM", "runtime/libc.so"),
            ("MANTLE_GCC_LINK_LIBGCC", "runtime/libgcc.a"),
            ("MANTLE_GCC_LINK_LIBGCC_EH", "runtime/libgcc_eh.a"),
            ("MANTLE_GCC_LINK_LIBGCC_S", "runtime/libgcc_s.so.1"),
            ("MANTLE_GCC_LINK_CRT", "runtime"),
        ] {
            environment.insert(key.into(), input("package", path));
        }
        environment.insert("MANTLE_TARGET_TOOLCHAIN_ROOT".into(), input("package", "native-provider"));
        let headers = super::attached("-I", "package", "linux-headers/include");
        for key in ["MANTLE_LINUX_HEADERS_CFLAGS", "CFLAGS", "CPPFLAGS", "CXXFLAGS"] {
            environment.insert(key.into(), headers.clone());
        }
        environment.insert("MANTLE_CAIRN_SYSROOT".into(), input("package", "sysroot"));
        environment.insert("LD_LIBRARY_PATH".into(), RecipeWord {
            parts: vec![
                WordPart::Input {
                    name: "package".into(),
                    path: "sysroot/lib".into(),
                },
                WordPart::Literal { text: ":".into() },
                WordPart::Runtime {
                    mount: "/mantle-rust-runtime".into(),
                    path: String::new(),
                },
            ],
        });
        super::native::extend_environment(self.request, &mut environment)?;
        Ok((environment, inputs))
    }
    fn dependencies(
        &self,
        edges: &[resolve::Edge],
        units: &BTreeMap<resolve::Key, Unit>,
    ) -> Result<PackageDependencies, RunError> {
        let mut direct = Vec::new();
        let mut closure = BTreeMap::new();
        let mut generated = BTreeMap::new();
        for edge in edges {
            let unit = units
                .get(&edge.key)
                .ok_or_else(|| error(format!("original dependency compiled out of order: {}", edge.key.manifest)))?;
            direct.push((edge.name.clone(), unit.primary.directory.clone(), unit.primary.filename.clone()));
            closure.extend(unit.closure.clone());
            generated.extend(unit.generated.clone());
            for variant in &unit.variants {
                closure.insert(variant.filename.clone(), variant.clone());
            }
        }
        Ok((direct, closure, generated))
    }
    fn search_inputs(
        &self,
        closure: &BTreeMap<String, Compiled>,
        inputs: &mut BTreeMap<String, ArtifactRef>,
        arguments: &mut Vec<RecipeWord>,
    ) {
        for (index, unit) in closure.values().enumerate() {
            let alias = format!("search-{index}");
            inputs.insert(alias.clone(), unit.directory.clone());
            arguments.extend([lit("-L"), super::attached("dependency=", &alias, "")]);
        }
    }
    fn script_metadata(
        &mut self,
        id: &str,
        package_world: ArtifactRef,
        own: Option<&ScriptProducts>,
        linked: &[(String, &ScriptProducts)],
        inputs: &mut BTreeMap<String, ArtifactRef>,
        environment: &mut BTreeMap<String, RecipeWord>,
        arguments: &mut Vec<RecipeWord>,
    ) {
        let (cfg, cfg_environment) = self.cfg_cache[&self.compiler].clone();
        if own.is_none() && linked.is_empty() {
            inputs.insert("metadata".into(), cfg_environment);
            environment.insert("MANTLE_CAIRN_COMPILE_ENV".into(), input("metadata", "compiler.env"));
            return;
        }
        let mut helper_inputs = BTreeMap::from([("package".into(), package_world)]);
        let mut words = vec![output("metadata")];
        for (index, (links, script)) in own
            .map(|script| (None, script))
            .into_iter()
            .chain(linked.iter().map(|(links, script)| (Some(links.as_str()), *script)))
            .enumerate()
        {
            let stdout = format!("script-stdout-{index}");
            helper_inputs.insert(stdout.clone(), script.stdout.clone());
            let logical = format!("{id}.metadata-root-{index}");
            self.steps.push(RecipeStep {
                id: logical.clone(),
                inputs: BTreeMap::new(),
                outputs: BTreeMap::from([("root".into(), super::spec(ArtifactKind::Text))]),
                operation: RecipeOperation::WriteText {
                    output: "root".into(),
                    text: script.logical_out.clone(),
                },
            });
            let root = format!("script-root-{index}");
            helper_inputs.insert(root.clone(), product(&logical, "root"));
            if let Some(links) = links {
                words.extend([lit("--dependency"), lit(links)]);
            } else {
                words.push(lit("--own"));
            }
            words.extend([
                input(&stdout, ""),
                RecipeWord {
                    parts: vec![WordPart::TextInput { name: root }],
                },
                input("package", ""),
            ]);
        }
        let metadata = format!("{id}.metadata");
        self.exec(
            &metadata,
            self.metadata_helper.clone(),
            helper_inputs,
            words,
            BTreeMap::new(),
            outputs("metadata", ArtifactKind::Tree),
        );
        let combined = format!("{id}.cargo-environment");
        self.exec(
            &combined,
            self.helper.clone(),
            BTreeMap::from([("cfg".into(), cfg), ("base".into(), product(&metadata, "metadata"))]),
            vec![
                lit("--cargo-metadata"),
                input("cfg", ""),
                output("metadata"),
                input("base", ""),
            ],
            BTreeMap::new(),
            outputs("metadata", ArtifactKind::Tree),
        );
        inputs.insert("metadata".into(), product(&combined, "metadata"));
        environment.insert("MANTLE_CAIRN_COMPILE_ENV".into(), input("metadata", "compiler.env"));
        arguments.push(super::attached("@", "metadata", "compiler.rsp"));
    }
    fn overlay(&mut self, id: &str, package: &Package, script: bool) -> ArtifactRef {
        let mut words = if script {
            let source = format!("{}/{}", self.top, package.script.as_deref().expect("original build script"));
            vec![
                lit("--original-script-overlay"),
                input("package", &source),
                input("package", &source),
                output("source-overlay"),
            ]
        } else {
            vec![
                lit("--process-library"),
                input("package", ""),
                input("package", &format!("{}/{}", self.top, package.directory)),
                lit(&package.name),
                output("source-overlay"),
            ]
        };
        let _ = &mut words;
        self.exec(
            id,
            self.helper.clone(),
            BTreeMap::from([("package".into(), self.source.clone())]),
            words,
            BTreeMap::new(),
            outputs("source-overlay", ArtifactKind::Tree),
        );
        product(id, "source-overlay")
    }
    fn group(
        &mut self,
        sources: &resolve::Sources,
        name: &str,
        goals: &[(&str, &[&str])],
        binaries: &[(&str, &str)],
    ) -> Result<Group, RunError> {
        if !self.cfg_cache.contains_key(&self.compiler) {
            let query = self.id("target-cfg");
            self.exec(
                &query,
                self.compiler.clone(),
                BTreeMap::from([("provider".into(), self.provider.clone())]),
                vec![
                    lit("--target"),
                    lit(TRIPLE),
                    lit("--print"),
                    lit("cfg"),
                    lit("-Ctarget-feature=-crt-static"),
                    lit("--sysroot"),
                    input("provider", ""),
                ],
                BTreeMap::new(),
                outputs("cfg", ArtifactKind::Text),
            );
            let cfg = product(&query, "stdout");
            let metadata = self.id("target-environment");
            self.exec(
                &metadata,
                self.helper.clone(),
                BTreeMap::from([("cfg".into(), cfg.clone())]),
                vec![lit("--cargo-metadata"), input("cfg", ""), output("metadata")],
                BTreeMap::new(),
                outputs("metadata", ArtifactKind::Tree),
            );
            self.cfg_cache.insert(self.compiler.clone(), (cfg, product(&metadata, "metadata")));
        }
        let profile = super::profile::select(sources, &self.version, name, self.stage_zero)?;
        self.profile_arguments = profile.arguments;
        self.incremental = profile.incremental;
        let seed_overrides = self.version == "1.90.0" && name == "std-initial";
        let topology = resolve::resolve_with_scripts(sources, goals, !seed_overrides)?;
        let mut units = BTreeMap::new();
        let mut scripts: BTreeMap<resolve::Key, ScriptProducts> = BTreeMap::new();
        let mut programs = BTreeMap::new();
        let mut libraries = BTreeMap::new();
        for key in &topology.order {
            let package = &topology.packages[key];
            let unit_id = self.id(&format!(
                "{name}.{}.{}.{}",
                package.name.replace('_', "-"),
                package.version,
                if key.host { "host" } else { "target" }
            ));
            let (mut environment, mut extra) = self.package_env(package, sources, &unit_id)?;
            environment.insert("DEBUG".into(), lit(if profile.debug { "true" } else { "false" }));
            let linked = package
                .dependencies
                .iter()
                .filter_map(|edge| {
                    topology.packages[&edge.key]
                        .links
                        .as_ref()
                        .and_then(|links| scripts.get(&edge.key).map(|script| (links.clone(), script)))
                })
                .collect::<Vec<_>>();
            let (dependencies, closure, mut generated) = self.dependencies(&package.dependencies, &units)?;
            if let Some(script_path) = &package.script {
                if seed_overrides {
                    let stdout = self.select(
                        &format!("{unit_id}.original-script-override"),
                        self.mrustc_source.clone(),
                        &format!("mrustc-0.12.0/script-overrides/stable-1.90.0-linux/build_{}.txt", package.name),
                        ArtifactKind::Text,
                    );
                    let out_dir = self.tree(&format!("{unit_id}.initial-out-dir"), BTreeMap::new());
                    scripts.insert(key.clone(), ScriptProducts {
                        stdout,
                        out_dir,
                        logical_out: super::native::out_directory(&format!("{unit_id}.build-script.run")),
                    });
                } else {
                    let script_id = format!("{unit_id}.build-script");
                    let package_source = self.source.clone();
                    let (script_dependencies, script_closure, build_generated) =
                        self.dependencies(&package.build_dependencies, &units)?;
                    let mut script_generated = generated.clone();
                    script_generated.extend(build_generated);
                    let mut script_inputs = extra.clone();
                    let mut script_arguments = Vec::new();
                    let mut script_environment = environment.clone();
                    self.search_inputs(&script_closure, &mut script_inputs, &mut script_arguments);
                    let overlay = self.overlay(&format!("{script_id}.source-overlay"), package, true);
                    script_inputs.insert("source-overlay".into(), overlay);
                    script_environment
                        .insert("MANTLE_CAIRN_SOURCE_OVERLAY".into(), input("source-overlay", "overlay.frame"));
                    let old = self.provider.clone();
                    self.provider = self.host_provider.clone();
                    let previous_override = self.build_override;
                    self.build_override = true;
                    let executable = self.compile_program(
                        &script_id,
                        package_source.clone(),
                        &format!("{}/{script_path}", self.top),
                        "build_script_build",
                        "bin",
                        &package.edition,
                        &script_dependencies,
                        script_environment,
                        script_inputs,
                        script_arguments,
                        &script_generated,
                    )?;
                    self.build_override = previous_override;
                    self.provider = old;
                    let script_generated = executable.generated.clone();
                    let mut runtime_inputs = extra.clone();
                    runtime_inputs.insert("rust-runtime".into(), self.runtime["tree"].clone());
                    for name in ["llvm-config", "llvm-native-controller", "llvm-native-helper"] {
                        runtime_inputs.insert(
                            name.into(),
                            self.llvm
                                .get(name)
                                .ok_or_else(|| error(format!("missing exact LLVM export {name}")))?
                                .clone(),
                        );
                    }
                    let mut runtime_arguments = Vec::new();
                    self.script_metadata(
                        &format!("{script_id}.runtime"),
                        package_source.clone(),
                        None,
                        &linked,
                        &mut runtime_inputs,
                        &mut environment,
                        &mut runtime_arguments,
                    );
                    environment.remove("MANTLE_CAIRN_COMPILE_ENV");
                    environment.insert("MANTLE_CAIRN_SCRIPT_ENV".into(), input("metadata", "compiler.env"));
                    let slots =
                        source_schedule_bound(self.reservations, &format!("{script_id}.run"), sources, package)?;
                    let run = super::native::schedule(
                        self.request,
                        &format!("{script_id}.run"),
                        executable.artifact,
                        package_source,
                        self.compiler.clone(),
                        self.helper.clone(),
                        environment.clone(),
                        runtime_inputs,
                        slots,
                        &script_generated,
                    )?;
                    self.steps.extend(run.steps);
                    let mut roots = script_generated;
                    roots.insert(run.logical_out.clone(), run.out_dir.clone());
                    self.project_generated(&format!("{script_id}.run"), run.generated_tmp, &roots)?;
                    scripts.insert(key.clone(), ScriptProducts {
                        stdout: run.stdout,
                        out_dir: run.out_dir,
                        logical_out: run.logical_out,
                    });
                }
            }
            if let Some(script) = scripts.get(key) {
                generated.insert(script.logical_out.clone(), script.out_dir.clone());
            }
            let mut arguments = Vec::new();
            self.search_inputs(&closure, &mut extra, &mut arguments);
            environment.remove("MANTLE_CAIRN_SCRIPT_ENV");
            let package_source = self.source.clone();
            self.script_metadata(
                &unit_id,
                package_source.clone(),
                scripts.get(key),
                &[],
                &mut extra,
                &mut environment,
                &mut arguments,
            );
            if package.name == "autocfg" {
                let overlay = self.overlay(&format!("{unit_id}.source-overlay"), package, false);
                extra.insert("source-overlay".into(), overlay);
                environment.insert("MANTLE_CAIRN_SOURCE_OVERLAY".into(), input("source-overlay", "overlay.frame"));
            }
            for feature in &package.features {
                arguments.extend([lit("--cfg"), lit(format!("feature={feature:?}"))]);
            }
            if let Some((crate_name, path, default_kind)) = library(package, sources) {
                let mut kinds = if seed_overrides {
                    vec!["rlib".into()]
                } else if default_kind == "proc-macro" {
                    vec![default_kind]
                } else {
                    let kinds = resolve::strings(package.manifest.get("lib").and_then(|lib| lib.get("crate-type")));
                    if kinds.is_empty() { vec![default_kind] } else { kinds }
                };
                if kinds.len() == 2
                    && kinds.iter().any(|kind| kind == "rlib")
                    && kinds.iter().any(|kind| kind == "dylib")
                {
                    kinds = vec!["rlib,dylib".into()];
                }
                let mut variants = Vec::new();
                for kind in kinds {
                    let old = self.provider.clone();
                    if key.host {
                        self.provider = self.host_provider.clone();
                    }
                    let previous_override = self.build_override;
                    self.build_override = key.host;
                    let compiled = self.compile_program(
                        &format!("{unit_id}.lib.{kind}"),
                        package_source.clone(),
                        &format!("{}/{}/{path}", self.top, package.directory),
                        &crate_name,
                        &kind,
                        &package.edition,
                        &dependencies,
                        environment.clone(),
                        extra.clone(),
                        arguments.clone(),
                        &generated,
                    )?;
                    self.build_override = previous_override;
                    self.provider = old;
                    if !key.host {
                        libraries.insert(compiled.filename.clone(), compiled.artifact.clone());
                        libraries.extend(compiled.companions.clone());
                    }
                    generated = compiled.generated.clone();
                    for (filename, artifact) in &compiled.companions {
                        variants.push(Compiled {
                            artifact: artifact.clone(),
                            filename: filename.clone(),
                            directory: compiled.directory.clone(),
                            companions: BTreeMap::new(),
                            generated: compiled.generated.clone(),
                        });
                    }
                    variants.push(compiled);
                }
                let primary =
                    variants.iter().find(|unit| unit.filename.ends_with(".rlib")).unwrap_or(&variants[0]).clone();
                units.insert(key.clone(), Unit {
                    primary,
                    variants,
                    closure,
                    generated: generated.clone(),
                });
            }
            for (manifest, binary) in binaries.iter().filter(|(manifest, _)| *manifest == key.manifest && !key.host) {
                let target = package.manifest.get("bin").and_then(toml::Value::as_array).and_then(|bins| {
                    bins.iter().find(|target| target.get("name").and_then(toml::Value::as_str) == Some(*binary))
                });
                let default_path = if *binary == package.name {
                    "src/main.rs".to_owned()
                } else {
                    format!("src/bin/{binary}.rs")
                };
                let path =
                    target.and_then(|target| target.get("path")).and_then(toml::Value::as_str).unwrap_or(&default_path);
                let mut direct = dependencies.clone();
                let mut inputs = extra.clone();
                let mut args = arguments.clone();
                if let Some(unit) = units.get(key) {
                    direct.push((
                        library(package, sources).expect("own library").0,
                        unit.primary.directory.clone(),
                        unit.primary.filename.clone(),
                    ));
                    let mut own = unit.closure.clone();
                    for variant in &unit.variants {
                        own.insert(variant.filename.clone(), variant.clone());
                    }
                    self.search_inputs(&own, &mut inputs, &mut args);
                }
                let compiled = self.compile_program(
                    &format!("{unit_id}.bin.{binary}"),
                    package_source.clone(),
                    &format!("{}/{}/{path}", self.top, package.directory),
                    &binary.replace('-', "_"),
                    "bin",
                    &package.edition,
                    &direct,
                    environment.clone(),
                    inputs,
                    args,
                    &generated,
                )?;
                programs.insert(format!("{manifest}:{binary}"), compiled);
            }
        }
        Ok(Group { programs, libraries })
    }
}

/// Source-authored resource reservation, not a claimed generator command count.
/// Every slot is guarded by actual suspended source execution; exhaustion fails.
fn source_schedule_bound(
    policy: &crate::source_built_program_authoring::ContinuationReservationPolicy,
    phase: &str,
    sources: &resolve::Sources,
    package: &Package,
) -> Result<usize, RunError> {
    let prefix = format!("{}/", package.directory);
    let files = sources
        .members
        .iter()
        .filter(|member| {
            member.starts_with(&prefix)
                && matches!(member.rsplit('.').next(), Some("c" | "cc" | "cpp" | "cxx" | "S" | "s"))
        })
        .count();
    let files = u32::try_from(files).map_err(|_| error("selected source file reservation hint exceeds u32"))?;
    policy.native_slots(phase, files, crate::source_built_recipe::STEP_COUNT_MAX as u32)
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn author_version(
    request: &AuthoringRequest<'_>,
    version: &str,
    source_tree: ArtifactRef,
    bootstrap_compiler: ArtifactRef,
    bootstrap_provider: ArtifactRef,
    llvm_exports: &BTreeMap<String, ArtifactRef>,
    runtime_members: &BTreeMap<String, ArtifactRef>,
    source_utility: ArtifactRef,
    bootstrap_helper: ArtifactRef,
    mrustc_source: ArtifactRef,
    reservations: &crate::source_built_program_authoring::ContinuationReservationPolicy,
    translator: ArtifactRef,
) -> Result<Fragment, RunError> {
    let route = crate::source_built_rust_programs::source_route(request.source_roots)?;
    if !route.stages.iter().any(|stage| stage.rust_version == version) {
        return Err(error(format!("release {version} is outside admitted original route")));
    }
    for reference in [
        &source_tree,
        &bootstrap_compiler,
        &bootstrap_provider,
        &source_utility,
        &bootstrap_helper,
        &mrustc_source,
        &translator,
    ] {
        require(reference)?;
    }
    for reference in runtime_members.values() {
        require(reference)?;
    }
    if !runtime_members.contains_key("tree") {
        return Err(error("original Rust runtime tree is absent"));
    }
    let sources = resolve::load(request, version)?;
    let original_top = sources.top.clone();
    let prefix = format!("successor-{version}");
    let helper_source = super::super::helper_source_ref(HELPER_PATH);
    let mut context = Context {
        request,
        source: source_tree.clone(),
        compiler: bootstrap_compiler.clone(),
        provider: bootstrap_provider.clone(),
        host_provider: bootstrap_provider.clone(),
        source_utility,
        mrustc_source,
        helper: bootstrap_helper.clone(),
        metadata_helper: bootstrap_helper,
        llvm: llvm_exports.clone(),
        translator,
        version: version.into(),
        prefix: prefix.clone(),
        top: format!("source/{original_top}"),
        steps: Vec::new(),
        files: BTreeMap::from([(HELPER_PATH.into(), super::helper_source())]),
        runtime: runtime_members.clone(),
        bootstrap: false,
        build_override: false,
        force_unstable: false,
        reservations,
        world_generation: 0,
        profile_arguments: Vec::new(),
        incremental: false,
        stage_zero: true,
        generated_current: BTreeMap::new(),
        cfg_cache: BTreeMap::new(),
        sdk_cache: BTreeMap::new(),
    };
    let compiler_goals: [(&str, &[&str]); 1] = [("compiler/rustc/Cargo.toml", &["llvm"])];
    let standard_goals: [(&str, &[&str]); 1] = [("library/sysroot/Cargo.toml", &["panic-unwind"])];
    let cargo_goals: [(&str, &[&str]); 1] = [("src/tools/cargo/Cargo.toml", &["vendored-openssl", "all-static"])];
    let mut cc = BTreeSet::new();
    for goals in [&compiler_goals[..], &standard_goals[..], &cargo_goals[..]] {
        for package in resolve::resolve(&sources, goals)?.packages.values() {
            if package.name == "cc" {
                cc.insert(package.directory.clone());
            }
        }
    }
    let overlay = format!("{prefix}.native-source-overlay");
    let mut args = vec![lit("--cc-overlay"), input("source", ""), output("tree")];
    for package in cc {
        args.push(input("source", &format!("{original_top}/{package}")));
    }
    context.exec(
        &overlay,
        context.helper.clone(),
        BTreeMap::from([("source".into(), source_tree)]),
        args,
        BTreeMap::new(),
        outputs("tree", ArtifactKind::Tree),
    );
    let std_overlay = format!("{prefix}.std-process-overlay");
    context.exec(
        &std_overlay,
        context.helper.clone(),
        BTreeMap::from([
            ("source".into(), product(&overlay, "tree")),
            (
                "descriptor".into(),
                super::super::helper_source_ref(super::super::minicargo::STD_PROCESS_DESCRIPTOR_PATH),
            ),
            ("module".into(), super::super::helper_source_ref(super::super::minicargo::STD_PROCESS_MODULE_PATH)),
        ]),
        vec![
            lit("--std-process-overlay"),
            input("source", ""),
            output("tree"),
            input("descriptor", ""),
            input("module", ""),
        ],
        BTreeMap::new(),
        outputs("tree", ArtifactKind::Tree),
    );
    let original = product(&std_overlay, "tree");
    context.world(original.clone())?;
    context.bootstrap = true;
    context.force_unstable = true;
    let std_initial = context.group(&sources, "std-initial", &standard_goals, &[])?;
    let initial_libs = context.tree(&format!("{prefix}.std-initial.rustlib"), std_initial.libraries);
    let old_bin =
        context.tree(&format!("{prefix}.bootstrap-bin"), BTreeMap::from([("rustc".into(), context.compiler.clone())]));
    context.provider = install_provider(
        &mut context,
        &format!("{prefix}.std-initial.provider"),
        old_bin.clone(),
        initial_libs.clone(),
        BTreeMap::new(),
    );
    context.host_provider = context.provider.clone();
    let initial_libs = if version == "1.90.0" {
        context.world(original.clone())?;
        let proper = context.group(&sources, "std-bootstrap", &standard_goals, &[])?;
        let libs = context.tree(&format!("{prefix}.std-bootstrap.rustlib"), proper.libraries);
        context.provider = install_provider(
            &mut context,
            &format!("{prefix}.std-bootstrap.provider"),
            old_bin,
            libs.clone(),
            BTreeMap::new(),
        );
        context.host_provider = context.provider.clone();
        libs
    } else {
        initial_libs
    };
    let helper_tree =
        context.tree(&format!("{prefix}.source-helper.file"), BTreeMap::from([("helper.rs".into(), helper_source)]));
    let helper_world = format!("{prefix}.source-helper.package");
    context.exec(
        &helper_world,
        context.helper.clone(),
        BTreeMap::from([
            ("native-world".into(), context.source.clone()),
            ("helper-source".into(), helper_tree),
        ]),
        vec![
            lit("--merge-trees"),
            output("tree"),
            input("native-world", ""),
            input("helper-source", ""),
        ],
        BTreeMap::new(),
        outputs("tree", ArtifactKind::Tree),
    );
    // Cairn's pure-I/O helper is not a standard-library or compiler Cargo
    // product: do not inherit std's prefer-dynamic/incremental profile.
    let previous_profile = std::mem::take(&mut context.profile_arguments);
    let previous_incremental = context.incremental;
    context.incremental = false;
    let helper = context.compile_program(
        &format!("{prefix}.source-helper"),
        product(&helper_world, "tree"),
        "helper.rs",
        "mantle_rust_source_helper",
        "bin",
        "2021",
        &[],
        BTreeMap::from([("CARGO_MANIFEST_DIR".into(), input("package", ""))]),
        BTreeMap::new(),
        Vec::new(),
        &BTreeMap::new(),
    )?;
    context.profile_arguments = previous_profile;
    context.incremental = previous_incremental;
    context.helper = helper.artifact.clone();
    context.metadata_helper = helper.artifact;
    context.prefix = format!("{prefix}.compiler");
    context.world(original.clone())?;
    let compiler =
        context.group(&sources, "compiler", &compiler_goals, &[("compiler/rustc/Cargo.toml", "rustc-main")])?;
    let compiler_image = compiler
        .programs
        .get("compiler/rustc/Cargo.toml:rustc-main")
        .ok_or_else(|| error("original compiler binary target absent"))?
        .artifact
        .clone();
    let compiler_libraries = compiler.libraries;
    let compiler_bin =
        context.tree(&format!("{prefix}.intermediate-bin"), BTreeMap::from([("rustc".into(), compiler_image.clone())]));
    context.provider = install_provider(
        &mut context,
        &format!("{prefix}.intermediate-provider"),
        compiler_bin,
        initial_libs,
        compiler_libraries.clone(),
    );
    context.compiler = compiler_image.clone();
    context.host_provider = context.provider.clone();
    context.bootstrap = false;
    context.prefix = format!("{prefix}.final-std");
    context.world(original.clone())?;
    context.stage_zero = false;
    let std_final = context.group(&sources, "std-final", &standard_goals, &[])?;
    let rustlib = context.tree(&format!("{prefix}.rustlib"), std_final.libraries);
    let compiler_bin =
        context.tree(&format!("{prefix}.compiler-bin"), BTreeMap::from([("rustc".into(), compiler_image.clone())]));
    context.provider = install_provider(
        &mut context,
        &format!("{prefix}.compiler-provider"),
        compiler_bin,
        rustlib.clone(),
        compiler_libraries.clone(),
    );
    context.host_provider = context.provider.clone();
    context.force_unstable = version == "1.90.0";
    context.prefix = format!("{prefix}.tools");
    context.world(original)?;
    let cargo = context.group(&sources, "cargo", &cargo_goals, &[("src/tools/cargo/Cargo.toml", "cargo")])?;
    let cargo = cargo.programs["src/tools/cargo/Cargo.toml:cargo"].artifact.clone();
    let mut binaries = BTreeMap::from([
        ("rustc".into(), compiler_image.clone()),
        ("cargo".into(), cargo.clone()),
    ]);
    let mut exports = BTreeMap::from([
        ("rustc".into(), compiler_image),
        ("cargo".into(), cargo),
        ("host-rustlib".into(), rustlib.clone()),
        ("target-rustlib".into(), rustlib.clone()),
        ("source-utility".into(), context.source_utility.clone()),
    ]);
    if version == route.final_version {
        let rustdoc = context.group(&sources, "rustdoc", &[("src/tools/rustdoc/Cargo.toml", &[])], &[(
            "src/tools/rustdoc/Cargo.toml",
            "rustdoc",
        )])?;
        let image = rustdoc.programs["src/tools/rustdoc/Cargo.toml:rustdoc"].artifact.clone();
        binaries.insert("rustdoc".into(), image.clone());
        exports.insert("rustdoc".into(), image);
    }
    let bin = context.tree(&format!("{prefix}.bin"), binaries);
    let provider = install_provider(
        &mut context,
        &format!("{prefix}.provider-unreceipted"),
        bin.clone(),
        rustlib.clone(),
        compiler_libraries,
    );
    let probe = format!("{prefix}.version");
    context.exec(
        &probe,
        exports["rustc"].clone(),
        BTreeMap::from([("provider".into(), provider.clone())]),
        vec![lit("-vV"), lit("--sysroot"), input("provider", "")],
        BTreeMap::new(),
        outputs("observed", ArtifactKind::Text),
    );
    // stdout is the actual compiler observation. The receipt helper also reads
    // the original version source, never a frozen successful build summary.
    let receipt = format!("{prefix}.receipt");
    context.exec(
        &receipt,
        context.helper.clone(),
        BTreeMap::from([
            ("version".into(), product(&probe, "stdout")),
            ("source".into(), context.source.clone()),
        ]),
        vec![
            lit("--provider-receipt"),
            input("version", ""),
            input("source", &format!("{}/src/version", context.top)),
            lit(version),
            output("receipt"),
        ],
        BTreeMap::new(),
        outputs("receipt", ArtifactKind::Text),
    );
    let receipt = product(&receipt, "receipt");
    exports.insert("provider-receipt".into(), receipt.clone());
    let installed_receipt =
        context.tree(&format!("{prefix}.receipt-tree"), BTreeMap::from([("build.json".into(), receipt)]));
    let share = context.tree(
        &format!("{prefix}.receipt-share"),
        BTreeMap::from([("mantle-rust-provider/receipts".into(), installed_receipt)]),
    );
    let provider_libs = context.select(&format!("{prefix}.provider-libs"), provider, "lib", ArtifactKind::Tree);
    let final_provider = context.tree(
        &format!("{prefix}.provider"),
        BTreeMap::from([
            ("bin".into(), bin),
            ("lib".into(), provider_libs),
            ("share".into(), share),
        ]),
    );
    exports.insert("provider".into(), final_provider);
    exports.insert(
        "native-controller".into(),
        context
            .llvm
            .get("llvm-native-controller")
            .ok_or_else(|| error("final source world lacks its actual native controller"))?
            .clone(),
    );
    exports.insert("native-source-boundary".into(), context.translator.clone());
    exports.insert(
        "native-target-wrappers".into(),
        context
            .sdk_cache
            .get("target-wrappers")
            .ok_or_else(|| error("final source world lacks its consumed target wrappers"))?
            .clone(),
    );
    exports.insert("native-package".into(), context.source.clone());
    Ok(Fragment {
        steps: context.steps,
        files: context.files,
        exports,
    })
}
fn install_provider(
    context: &mut Context<'_, '_>,
    id: &str,
    bin: ArtifactRef,
    rustlib: ArtifactRef,
    compiler_libraries: BTreeMap<String, ArtifactRef>,
) -> ArtifactRef {
    let compiler = context.tree(&format!("{id}.compiler-crates"), compiler_libraries);
    let merged = format!("{id}.combined-rustlib");
    context.exec(
        &merged,
        context.helper.clone(),
        BTreeMap::from([("standard".into(), rustlib), ("compiler".into(), compiler)]),
        vec![
            lit("--merge-trees"),
            output("tree"),
            input("standard", ""),
            input("compiler", ""),
        ],
        BTreeMap::new(),
        outputs("tree", ArtifactKind::Tree),
    );
    let combined = product(&merged, "tree");
    let runtime = format!("{id}.runtime");
    context.exec(
        &runtime,
        context.helper.clone(),
        BTreeMap::from([("rustlib".into(), combined.clone())]),
        vec![lit("--runtime-libraries"), input("rustlib", ""), output("tree")],
        BTreeMap::new(),
        outputs("tree", ArtifactKind::Tree),
    );
    let triple = context.tree(&format!("{id}.triple"), BTreeMap::from([("lib".into(), combined)]));
    let rustlibs = context.tree(&format!("{id}.rustlib"), BTreeMap::from([(TRIPLE.into(), triple)]));
    let skeleton = context.tree(&format!("{id}.lib-skeleton"), BTreeMap::from([("rustlib".into(), rustlibs)]));
    let lib = format!("{id}.lib");
    context.exec(
        &lib,
        context.helper.clone(),
        BTreeMap::from([
            ("skeleton".into(), skeleton),
            ("runtime".into(), product(&runtime, "tree")),
        ]),
        vec![
            lit("--merge-trees"),
            output("tree"),
            input("skeleton", ""),
            input("runtime", ""),
        ],
        BTreeMap::new(),
        outputs("tree", ArtifactKind::Tree),
    );
    context.tree(id, BTreeMap::from([("bin".into(), bin), ("lib".into(), product(&lib, "tree"))]))
}
