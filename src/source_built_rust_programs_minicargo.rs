//! Source-selected mrustc 0.12 minicargo graph. minicargo is a product and an
//! ancestry input, never a process. Every crate goes through deferred mrustc C
//! translation; every genuine generator is separately compiled and executed.
use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::io::Read;

use super::translation::TranslationInvocation;
use super::translation::TranslationKind;
use super::translation::{self};
use crate::errors::RunError;
use crate::source_built_fixed_point::SourceAuthorityRole;
use crate::source_built_program_authoring::AuthoringRequest;
use crate::source_built_program_authoring::ContinuationReservationPolicy;
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
#[path = "source_built_rust_programs_minicargo_resolve.rs"]
mod resolve;
use resolve::Key;
use resolve::Package;
use resolve::Sources;
use resolve::Topology;
const STAGE: &str = "full-source-rust-provider";
const TRIPLE: &str = "x86_64-unknown-linux-musl";
pub(crate) const HELPER_PATH: &str = "mantle-source-authoring/rust-minicargo.c";
pub(crate) const HELPER_SOURCE: &str = include_str!("source_built_rust_programs_minicargo_helper.c");
pub(crate) const STD_PROCESS_DESCRIPTOR_PATH: &str = "mantle-source-authoring/rust-minicargo-std-process.patch";
pub(crate) const STD_PROCESS_MODULE_PATH: &str = "mantle-source-authoring/rust-minicargo-std-process.rs";
pub(crate) struct Fragment {
    pub steps: Vec<RecipeStep>,
    pub files: BTreeMap<String, String>,
    pub exports: BTreeMap<String, ArtifactRef>,
}
fn error(s: impl Into<String>) -> RunError {
    RunError::Build(format!("original minicargo author: {}", s.into()))
}
fn lit(s: impl Into<String>) -> RecipeWord {
    RecipeWord {
        parts: vec![WordPart::Literal { text: s.into() }],
    }
}
fn input(n: &str, p: &str) -> RecipeWord {
    RecipeWord {
        parts: vec![WordPart::Input {
            name: n.into(),
            path: p.into(),
        }],
    }
}
fn output(n: &str) -> RecipeWord {
    RecipeWord {
        parts: vec![WordPart::Output {
            name: n.into(),
            path: String::new(),
        }],
    }
}
fn attached(prefix: &str, n: &str, p: &str) -> RecipeWord {
    RecipeWord {
        parts: vec![WordPart::Literal { text: prefix.into() }, WordPart::Input {
            name: n.into(),
            path: p.into(),
        }],
    }
}
fn product(id: &str, n: &str) -> ArtifactRef {
    ArtifactRef::Produced {
        action_id: format!("{STAGE}/{id}"),
        output_id: n.into(),
    }
}
fn spec(kind: ArtifactKind) -> RecipeOutput {
    RecipeOutput {
        kind,
        bytes_max: 16 * 1024 * 1024 * 1024,
    }
}
fn outputs(name: &str, kind: ArtifactKind) -> BTreeMap<String, RecipeOutput> {
    BTreeMap::from([
        (name.into(), spec(kind)),
        ("stdout".into(), spec(ArtifactKind::Text)),
        ("stderr".into(), spec(ArtifactKind::Text)),
        ("exit-status".into(), RecipeOutput {
            kind: ArtifactKind::Text,
            bytes_max: 4,
        }),
    ])
}
fn require(r: &ArtifactRef) -> Result<(), RunError> {
    if matches!(r, ArtifactRef::Produced { .. }) {
        Ok(())
    } else {
        Err(error("executable/source-stage inputs require Produced ancestry"))
    }
}

/// Read package metadata in a cold source leaf. URLs, authors and licenses are
/// text products, never preparation-generated source or untyped path operands.
pub(crate) fn package_fields(
    id: &str,
    source: ArtifactRef,
    manifest: &str,
    workspace: Option<&str>,
    helper: ArtifactRef,
    mut runtime_inputs: BTreeMap<String, ArtifactRef>,
    runtime_mounts: BTreeMap<String, String>,
) -> Result<(RecipeStep, BTreeMap<String, ArtifactRef>), RunError> {
    require(&source)?;
    require(&helper)?;
    crate::source_built_recipe::relative(manifest, false)?;
    if let Some(path) = workspace {
        crate::source_built_recipe::relative(path, false)?;
    }
    runtime_inputs.insert("source".into(), source);
    runtime_inputs.insert("executable".into(), helper);
    let fields = [
        ("AUTHORS", "authors"),
        ("DESCRIPTION", "description"),
        ("HOMEPAGE", "homepage"),
        ("LICENSE", "license"),
        ("LICENSE_FILE", "license-file"),
        ("REPOSITORY", "repository"),
        ("README", "readme"),
        ("RUST_VERSION", "rust-version"),
        ("VERSION", "version"),
    ];
    let mut arguments = vec![
        lit("package-fields"),
        input("source", manifest),
        workspace.map_or_else(|| lit("-"), |p| input("source", p)),
    ];
    let mut declared = BTreeMap::new();
    let mut exports = BTreeMap::new();
    for (env, name) in fields {
        declared.insert(name.into(), RecipeOutput {
            kind: ArtifactKind::Text,
            bytes_max: 65536,
        });
        arguments.push(output(name));
        exports.insert(format!("CARGO_PKG_{env}"), product(id, name));
    }
    Ok((
        RecipeStep {
            id: id.into(),
            inputs: runtime_inputs,
            outputs: declared,
            operation: RecipeOperation::Exec {
                executable_input: "executable".into(),
                arguments,
                environment: BTreeMap::new(),
                invocation_input: None,
                runtime_mounts,
                merge_stderr: false,
                cwd_input: None,
                cwd_output: None,
                stdin_input: None,
                exit_policy: RecipeExitPolicy::Exact { code: 0 },
                guard: None,
                output_seeds: BTreeMap::new(),
                command_state: None,
            },
        },
        exports,
    ))
}
#[derive(Clone)]
struct Compiled {
    bundle: ArtifactRef,
    artifact: ArtifactRef,
    filename: String,
    name: String,
    searches: BTreeSet<ArtifactRef>,
    proc_macros: BTreeMap<String, ArtifactRef>,
    generated: BTreeMap<String, ArtifactRef>,
}
#[derive(Clone)]
struct Script {
    stdout: ArtifactRef,
    out_dir: ArtifactRef,
    package: ArtifactRef,
    logical_out: String,
    generated: BTreeMap<String, ArtifactRef>,
}
struct Metadata {
    world: ArtifactRef,
    arguments: ArtifactRef,
    environment: ArtifactRef,
    generated: BTreeMap<String, ArtifactRef>,
}
fn merge_generated(target: &mut BTreeMap<String, ArtifactRef>, source: &BTreeMap<String, ArtifactRef>) {
    for (path, r) in source {
        if !target.contains_key(path) {
            target.insert(path.clone(), r.clone());
        }
    }
}
fn logical_output(path: &str) -> RecipeWord {
    RecipeWord {
        parts: vec![WordPart::Workspace {
            mount: path.into(),
            path: String::new(),
        }],
    }
}
fn target_tool_path() -> RecipeWord {
    RecipeWord {
        parts: vec![
            WordPart::Input {
                name: "package".into(),
                path: "target-wrappers".into(),
            },
            WordPart::Literal { text: ":".into() },
            WordPart::Input {
                name: "package".into(),
                path: "phase-tools".into(),
            },
        ],
    }
}
struct Context<'a, 'b> {
    request: &'a AuthoringRequest<'b>,
    source: ArtifactRef,
    source_root: Option<(ArtifactRef, ArtifactRef)>,
    world_members: BTreeMap<ArtifactRef, BTreeMap<String, ArtifactRef>>,
    standard_generated: BTreeMap<ArtifactRef, BTreeMap<String, ArtifactRef>>,
    generated_current: BTreeMap<String, ArtifactRef>,
    mrustc_source: ArtifactRef,
    mrustc: ArtifactRef,
    translator: ArtifactRef,
    helper: ArtifactRef,
    native_helper: Option<ArtifactRef>,
    native_members: Option<BTreeMap<String, ArtifactRef>>,
    cfg: Option<ArtifactRef>,
    reservation_policy: &'a ContinuationReservationPolicy,
    reservation_hints: BTreeMap<String, resolve::ReservationHints>,
    llvm: BTreeMap<String, ArtifactRef>,
    steps: Vec<RecipeStep>,
    files: BTreeMap<String, String>,
    exports: BTreeMap<String, ArtifactRef>,
    package_metadata: BTreeMap<String, BTreeMap<String, ArtifactRef>>,
    runtime: BTreeMap<String, ArtifactRef>,
    native_runtime: ArtifactRef,
    mounts: BTreeMap<String, String>,
}
/// Declared environment words and their producer inputs for one package.
type PackageEnvironment = (BTreeMap<String, RecipeWord>, BTreeMap<String, ArtifactRef>);
/// Compiled objects and generated scripts keyed by resolved package.
type CompiledScripts = (BTreeMap<Key, Compiled>, BTreeMap<Key, Script>);

impl Context<'_, '_> {
    fn remaining_steps(&self) -> Result<usize, RunError> {
        crate::source_built_recipe::STEP_COUNT_MAX
            .checked_sub(self.steps.len())
            .ok_or_else(|| error("recipe action capacity exhausted before continuation reservation"))
    }
    // Logical roots identify original directories; snapshots advance in recipe
    // execution order and may differ even when no source path identity changes.
    fn refresh_generated(&self, generated: &mut BTreeMap<String, ArtifactRef>) {
        for (root, snapshot) in generated {
            if let Some(current) = self.generated_current.get(root)
                && *snapshot != *current
            {
                *snapshot = current.clone();
            }
        }
    }
    fn capture_generated(
        &mut self,
        id: &str,
        tmp: ArtifactRef,
        generated: &mut BTreeMap<String, ArtifactRef>,
    ) -> Result<(), RunError> {
        for (index, (root, snapshot)) in generated.iter_mut().enumerate() {
            let path = root.strip_prefix("/tmp/").ok_or_else(|| error("captured generated root is outside /tmp"))?;
            crate::source_built_recipe::relative(path, false)?;
            let actual = self.select(&format!("{id}.generated-{index}"), tmp.clone(), path, ArtifactKind::Tree);
            *snapshot = actual.clone();
            self.generated_current.insert(root.clone(), actual);
        }
        Ok(())
    }
    fn macro_source_sites(&self, path: &str) -> Result<u32, RunError> {
        let Some(path) = path.strip_prefix("rustc-1.90.0-src/") else {
            return Ok(0);
        };
        let mut dir = resolve::directory(path);
        loop {
            let key = if dir.is_empty() {
                "Cargo.toml".to_string()
            } else {
                format!("{dir}/Cargo.toml")
            };
            if let Some(hint) = self.reservation_hints.get(&key) {
                return u32::try_from(hint.macro_sites)
                    .map_err(|_| error("macro source-site reservation hint exceeds u32"));
            }
            if dir.is_empty() {
                return Ok(0);
            }
            dir = resolve::directory(dir);
        }
    }
    fn environment(&mut self, p: &Package, sources: &Sources) -> Result<PackageEnvironment, RunError> {
        if !self.package_metadata.contains_key(&p.key.manifest) {
            let id = format!("minicargo.package-metadata.{}", self.package_metadata.len());
            let manifest = format!("rustc-1.90.0-src/{}", p.key.manifest);
            let workspace = sources.workspace(&p.key.manifest).map(|(dir, _)| {
                if dir.is_empty() {
                    "rustc-1.90.0-src/Cargo.toml".into()
                } else {
                    format!("rustc-1.90.0-src/{dir}/Cargo.toml")
                }
            });
            let (step, fields) = package_fields(
                &id,
                self.source.clone(),
                &manifest,
                workspace.as_deref(),
                self.helper.clone(),
                self.runtime.clone(),
                self.mounts.clone(),
            )?;
            self.steps.push(step);
            self.package_metadata.insert(p.key.manifest.clone(), fields);
        }
        let mut env = environment(p, &format!("rustc-1.90.0-src/{}", p.directory));
        let mut inputs = BTreeMap::new();
        for (name, r) in &self.package_metadata[&p.key.manifest] {
            let alias = format!("package-field-{}", name.to_lowercase().replace('_', "-"));
            inputs.insert(alias.clone(), r.clone());
            env.insert(name.clone(), RecipeWord {
                parts: vec![WordPart::TextInput { name: alias }],
            });
        }
        Ok((env, inputs))
    }
    fn exec(
        &mut self,
        id: &str,
        exe: ArtifactRef,
        mut inputs: BTreeMap<String, ArtifactRef>,
        args: Vec<RecipeWord>,
        env: BTreeMap<String, RecipeWord>,
        declared: BTreeMap<String, RecipeOutput>,
    ) {
        inputs.extend(self.runtime.clone());
        inputs.insert("executable".into(), exe);
        self.steps.push(RecipeStep {
            id: id.into(),
            inputs,
            outputs: declared,
            operation: RecipeOperation::Exec {
                executable_input: "executable".into(),
                arguments: args,
                environment: env,
                invocation_input: None,
                runtime_mounts: self.mounts.clone(),
                merge_stderr: false,
                cwd_input: None,
                cwd_output: None,
                stdin_input: None,
                exit_policy: RecipeExitPolicy::Exact { code: 0 },
                guard: None,
                output_seeds: BTreeMap::new(),
                command_state: None,
            },
        });
    }
    fn select(&mut self, id: &str, tree: ArtifactRef, path: &str, kind: ArtifactKind) -> ArtifactRef {
        self.exec(
            id,
            self.helper.clone(),
            BTreeMap::from([("source".into(), tree)]),
            vec![lit("select"), input("source", path), output("selected")],
            BTreeMap::new(),
            outputs("selected", kind),
        );
        product(id, "selected")
    }
    fn merge(&mut self, id: &str, bundles: impl IntoIterator<Item = ArtifactRef>) -> ArtifactRef {
        let mut inputs = BTreeMap::new();
        let mut args = vec![lit("merge"), output("tree")];
        for (i, r) in bundles.into_iter().enumerate() {
            let n = format!("member-{i}");
            inputs.insert(n.clone(), r);
            args.push(input(&n, ""));
        }
        self.exec(id, self.helper.clone(), inputs, args, BTreeMap::new(), outputs("tree", ArtifactKind::Tree));
        product(id, "tree")
    }
    // Source/SDK package paths keep the original input namespace. Captured script
    // outputs are carried separately as logical workspace roots; stdout, metadata
    // and generated source contents never undergo producer-path replacement.
    fn source_members(&mut self) -> BTreeMap<String, ArtifactRef> {
        if self.source_root.as_ref().map(|(source, _)| source) != Some(&self.source) {
            let id = format!("minicargo.package-world.source-{}", self.steps.len());
            let root = self.select(&id, self.source.clone(), "rustc-1.90.0-src", ArtifactKind::Tree);
            self.source_root = Some((self.source.clone(), root));
        }
        BTreeMap::from([("rustc-1.90.0-src".into(), self.source_root.as_ref().unwrap().1.clone())])
    }
    fn emit_world(&mut self, id: &str, members: BTreeMap<String, ArtifactRef>) -> ArtifactRef {
        let mut inputs = BTreeMap::new();
        let mut entries = BTreeMap::new();
        for (index, (path, r)) in members.iter().enumerate() {
            let name = format!("member-{index}");
            inputs.insert(name.clone(), r.clone());
            entries.insert(path.clone(), name);
        }
        self.steps.push(RecipeStep {
            id: id.into(),
            inputs,
            outputs: BTreeMap::from([("tree".into(), spec(ArtifactKind::Tree))]),
            operation: RecipeOperation::AssembleTree {
                output: "tree".into(),
                entries,
            },
        });
        let world = product(id, "tree");
        self.world_members.insert(world.clone(), members);
        world
    }
    fn extend_world(
        &mut self,
        id: &str,
        base: ArtifactRef,
        additions: BTreeMap<String, ArtifactRef>,
    ) -> Result<ArtifactRef, RunError> {
        if additions.is_empty() {
            return Ok(base);
        }
        let members = if base == self.source {
            Some(self.source_members())
        } else {
            self.world_members.get(&base).cloned()
        };
        if let Some(mut members) = members {
            for (path, r) in additions {
                if let Some(previous) = members.insert(path.clone(), r.clone())
                    && previous != r
                {
                    return Err(error(format!("compiler world member authority collision at {path}")));
                }
            }
            Ok(self.emit_world(id, members))
        } else {
            let extra = self.emit_world(&format!("{id}.members"), additions);
            Ok(self.merge(id, [base, extra]))
        }
    }
    // SDK responses and original wrapper scripts have no Rust-std dependency.
    // Cache only the invariant native namespace, never source/package state.
    fn native_members(&mut self) -> Result<&BTreeMap<String, ArtifactRef>, RunError> {
        if self.native_members.is_none() {
            let mut members = BTreeMap::from([
                ("phase-tools/rustc".into(), self.mrustc.clone()),
                ("runtime".into(), self.native_runtime.clone()),
            ]);
            for (path, role) in [
                ("phase-tools/cc", "native.gcc"),
                ("phase-tools/c++", "native.gxx"),
                ("phase-tools/ar", "native.ar"),
                ("phase-tools/ranlib", "native.ranlib"),
                ("phase-tools/cc1", "native.cc1"),
                ("phase-tools/cc1plus", "native.cc1plus"),
                ("phase-tools/as", "native.as"),
                ("phase-tools/ld", "native.ld"),
                ("phase-tools/collect2", "native.collect2-query"),
                ("native-provider", "native.provider"),
                ("native-sysroot", "native.sysroot"),
                ("linux-headers", "linux.headers"),
            ] {
                let r = self
                    .request
                    .predecessor_exports
                    .get(&format!("full-source-native-provider:{role}"))
                    .ok_or_else(|| error(format!("missing native package-world tool {role}")))?
                    .clone();
                require(&r)?;
                members.insert(path.into(), r);
            }
            if let Some(root) = self.llvm.get("llvm-root") {
                members.insert("llvm".into(), root.clone());
            }
            super::successors::native::extend_world(self.request, &mut members)?;
            let base = self.emit_world("minicargo.native-sdk.world", members.clone());
            for (cxx, name) in [(false, "cc"), (true, "cxx")] {
                let mut flags = translation::sdk_arguments(cxx, true);
                for word in &mut flags {
                    for part in &mut word.parts {
                        if let WordPart::Input { name, path } = part {
                            let prefix = match name.as_str() {
                                "phase-tools" => "phase-tools",
                                "native-provider" => "native-provider",
                                "native-sysroot" => "native-sysroot",
                                "rust-runtime" => "runtime",
                                _ => return Err(error(format!("unbound original native SDK input {name}"))),
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
                let id = format!("minicargo.native-sdk.{name}");
                let mut arguments = vec![lit("--native-sdk"), output("response")];
                arguments.extend(flags);
                self.exec(
                    &id,
                    self.helper.clone(),
                    BTreeMap::from([("package".into(), base.clone())]),
                    arguments,
                    BTreeMap::new(),
                    outputs("response", ArtifactKind::Text),
                );
                members.insert(format!("native-sdk-{name}"), product(&id, "response"));
            }
            let package = self.emit_world("minicargo.target-wrappers.world", members.clone());
            let mut recipe = SourceRecipe {
                schema: crate::source_built_recipe::SCHEMA.into(),
                stage_id: STAGE.into(),
                preparation: RecipePreparation {
                    tool: "original first-stage target wrappers".into(),
                    provenance: "authenticated original source-provider wrapper generators".into(),
                },
                steps: Vec::new(),
                roots: Vec::new(),
                exports: BTreeMap::new(),
                semantic_roots: BTreeMap::new(),
                timeout_groups: Vec::new(),
            };
            let empty = self.merge("minicargo.target-wrappers.empty", std::iter::empty::<ArtifactRef>());
            let initial = "minicargo.target-wrappers.initial";
            self.steps.push(RecipeStep {
                id: initial.into(),
                inputs: BTreeMap::from([("empty".into(), empty)]),
                outputs: BTreeMap::from([("tree".into(), spec(ArtifactKind::Tree))]),
                operation: RecipeOperation::AssembleTree {
                    output: "tree".into(),
                    entries: BTreeMap::from([("tmp".into(), "empty".into())]),
                },
            });
            let controller = self
                .llvm
                .get("llvm-native-controller")
                .ok_or_else(|| error("missing original wrapper source controller"))?
                .clone();
            let workspace = super::successors::append_target_wrappers(
                self.request,
                &mut recipe,
                "minicargo.target-wrappers",
                package,
                controller,
                self.native_runtime.clone(),
                true,
                "phase-tools/cc",
                "phase-tools/c++",
                product(initial, "tree"),
            )?;
            self.steps.extend(recipe.steps);
            self.exports.extend(recipe.exports);
            let wrappers =
                self.select("minicargo.target-wrappers.select", workspace, "tmp/target-wrappers", ArtifactKind::Tree);
            members.insert("target-wrappers".into(), wrappers);
            self.native_members = Some(members);
        }
        Ok(self.native_members.as_ref().expect("native world initialized"))
    }
    fn script_world(
        &mut self,
        id: &str,
        stdlib: Option<&ArtifactRef>,
        metadata: Option<ArtifactRef>,
    ) -> Result<ArtifactRef, RunError> {
        let mut members = self.source_members();
        members.extend(self.native_members()?.iter().map(|(path, r)| (path.clone(), r.clone())));
        if let Some(stdlib) = stdlib {
            members.insert("stdlib".into(), stdlib.clone());
        }
        if let Some(metadata) = metadata {
            members.insert("metadata/script-environment".into(), metadata);
        }
        Ok(self.emit_world(id, members))
    }
    fn script_metadata(&mut self, id: &str, deps: &[(String, Script)]) -> Result<ArtifactRef, RunError> {
        let cfg = self.cfg.clone().ok_or_else(|| error("native script metadata lacks actual mrustc cfg query"))?;
        let mut inputs = BTreeMap::from([("cfg".into(), cfg)]);
        let mut args = vec![
            lit("metadata"),
            output("arguments"),
            output("environment"),
            input("cfg", ""),
        ];
        for (index, (links, script)) in deps.iter().enumerate() {
            let stdout = format!("dep-stdout-{index}");
            inputs.insert(stdout.clone(), script.stdout.clone());
            args.extend([
                lit(links),
                input(&stdout, ""),
                lit(format!("DEP_{}", links.replace('-', "_"))),
            ]);
        }
        let mut declared = outputs("arguments", ArtifactKind::Object);
        declared.insert("environment".into(), spec(ArtifactKind::Object));
        self.exec(id, self.helper.clone(), inputs, args, BTreeMap::new(), declared);
        Ok(product(id, "environment"))
    }
    fn metadata(
        &mut self,
        id: &str,
        base: ArtifactRef,
        own: Option<&Script>,
        deps: &[(&str, &Script)],
    ) -> Result<Metadata, RunError> {
        let mut additions = BTreeMap::new();
        let mut generated = BTreeMap::new();
        for script in own.into_iter().chain(deps.iter().map(|(_, script)| *script)) {
            merge_generated(&mut generated, &script.generated);
            if let Some(members) = self.world_members.get(&script.package) {
                for (path, r) in members {
                    if path == "metadata/script-environment" {
                        continue;
                    }
                    if let Some(previous) = additions.insert(path.clone(), r.clone())
                        && previous != *r
                    {
                        return Err(error(format!("metadata source-path authority collision {path}")));
                    }
                }
            }
        }
        let base = own.map(|script| script.package.clone()).unwrap_or(base);
        let world = self.extend_world(&format!("{id}.world"), base, additions)?;
        let mut inputs = BTreeMap::new();
        let mut args = vec![lit("metadata"), output("arguments"), output("environment"), lit("-")];
        if let Some(script) = own {
            inputs.insert("own-stdout".into(), script.stdout.clone());
            args.extend([lit("-"), input("own-stdout", ""), lit("-")]);
        }
        for (index, (links, script)) in deps.iter().enumerate() {
            let stdout = format!("dep-stdout-{index}");
            inputs.insert(stdout.clone(), script.stdout.clone());
            args.extend([
                lit(*links),
                input(&stdout, ""),
                lit(format!("DEP_{}", links.replace('-', "_"))),
            ]);
        }
        let mut declared = outputs("arguments", ArtifactKind::Object);
        declared.insert("environment".into(), spec(ArtifactKind::Object));
        self.exec(id, self.helper.clone(), inputs, args, BTreeMap::new(), declared);
        Ok(Metadata {
            world,
            arguments: product(id, "arguments"),
            environment: product(id, "environment"),
            generated,
        })
    }
    #[allow(clippy::too_many_arguments)]
    fn compile(
        &mut self,
        id: &str,
        package: ArtifactRef,
        path: &str,
        name: &str,
        filename: String,
        kind: TranslationKind,
        edition: &str,
        tag: Option<&str>,
        features: &BTreeSet<String>,
        dependencies: &[(String, Compiled)],
        search: Option<&ArtifactRef>,
        env: BTreeMap<String, RecipeWord>,
        metadata: Option<(ArtifactRef, ArtifactRef)>,
        extra: BTreeMap<String, ArtifactRef>,
        mut generated: BTreeMap<String, ArtifactRef>,
        output_directory: &str,
    ) -> Result<Compiled, RunError> {
        let mut inputs = self.runtime.clone();
        inputs.extend(extra);
        let mut env = env;
        env.insert("CARGO_CRATE_NAME".into(), lit(name));
        env.insert("PATH".into(), target_tool_path());
        super::successors::native::extend_environment(self.request, &mut env)?;
        for (_, dependency) in dependencies {
            merge_generated(&mut generated, &dependency.generated);
        }
        if let Some(standard) = search.and_then(|search| self.standard_generated.get(search)) {
            merge_generated(&mut generated, standard);
        }
        self.refresh_generated(&mut generated);
        let crate_type = match kind {
            TranslationKind::Rlib => "rlib",
            TranslationKind::Bin | TranslationKind::BuildScript => "bin",
            TranslationKind::ProcMacro => "proc-macro",
            TranslationKind::Dylib => "dylib",
        };
        let mut arguments = vec![
            input("package", path),
            lit("--crate-name"),
            lit(name),
            lit("--crate-type"),
            lit(crate_type),
            lit("--edition"),
            lit(edition),
            lit("--cfg"),
            lit("debug_assertions"),
            lit("-O"),
        ];
        if kind != TranslationKind::BuildScript {
            arguments.extend([lit("--target"), lit(TRIPLE)]);
            if let Some(tag) = tag {
                arguments.extend([lit("--crate-tag"), lit(tag)]);
            }
        }
        for feature in features {
            arguments.extend([lit("--cfg"), lit(format!("feature=\"{feature}\""))]);
        }
        let mut used = BTreeSet::new();
        let mut proc_macros = BTreeMap::new();
        for (_, c) in dependencies {
            used.insert(c.bundle.clone());
            used.extend(c.searches.iter().cloned());
            for (name, exe) in &c.proc_macros {
                if let Some(previous) = proc_macros.insert(name.clone(), exe.clone())
                    && previous != *exe
                {
                    return Err(error(format!("proc macro basename authority collision: {name}")));
                }
            }
        }
        let mut additions = self.native_members()?.clone();
        let mut paths = self
            .world_members
            .get(&package)
            .into_iter()
            .flat_map(|members| members.iter().map(|(path, r)| (r.clone(), path.clone())))
            .collect::<BTreeMap<_, _>>();
        for bundle in used.iter().chain(search) {
            if !paths.contains_key(bundle) {
                let path = format!("compiler-dependencies/bundle-{}", paths.len());
                paths.insert(bundle.clone(), path.clone());
                additions.insert(path, bundle.clone());
            }
        }
        for (alias, c) in dependencies {
            let path = format!(
                "{}/{}",
                paths.get(&c.bundle).ok_or_else(|| error("missing unique dependency bundle path"))?,
                c.filename
            );
            arguments.extend([lit("--extern"), attached(&format!("{alias}="), "package", &path)]);
        }
        for bundle in &used {
            arguments.extend([
                lit("-L"),
                input("package", paths.get(bundle).ok_or_else(|| error("missing transitive dependency bundle path"))?),
            ]);
        }
        if let Some(search) = search {
            arguments.extend([
                lit("-L"),
                input("package", paths.get(search).ok_or_else(|| error("missing standard library bundle path"))?),
            ]);
        }
        if let Some((a, e)) = &metadata {
            additions.insert("metadata/compiler-arguments".into(), a.clone());
            additions.insert("metadata/compiler-environment".into(), e.clone());
        }
        let world = self.extend_world(&format!("{id}.world"), package, additions)?;
        inputs.insert("package".into(), world);
        let (metadata_arguments, metadata_environment) = metadata.map_or((None, None), |_| {
            (
                Some(input("package", "metadata/compiler-arguments")),
                Some(input("package", "metadata/compiler-environment")),
            )
        });
        let proc_macro_slots = if proc_macros.is_empty() {
            0
        } else {
            let slots_max = translation::macro_reservation_slots_max(self.remaining_steps()?)?;
            self.reservation_policy
                .macro_slots(&format!("{id}.macro"), self.macro_source_sites(path)?, slots_max)?
        };
        let native_commands_max = crate::source_built_recipe::controller::reserve_native_commands(
            self.reservation_policy,
            &format!("{id}.native"),
            1,
            self.remaining_steps()?,
        )?;
        let native_controller = self
            .llvm
            .get("llvm-native-controller")
            .ok_or_else(|| error("missing deferred native source controller"))?
            .clone();
        let logical_output = format!("{output_directory}/{filename}");
        crate::source_built_recipe::relative(&logical_output, false)?;
        let f = translation::author(self.request, TranslationInvocation {
            id: id.into(),
            mrustc: self.mrustc.clone(),
            helper: self.translator.clone(),
            native_runtime: self.native_runtime.clone(),
            native_controller,
            native_wrapper_path: "/inputs/package/target-wrappers/cc".into(),
            logical_output,
            native_commands_max,
            macro_helper: self.native_helper.clone(),
            proc_macro_executables: proc_macros.clone(),
            proc_macro_slots,
            generated: generated.clone(),
            inputs,
            arguments,
            environment: env,
            runtime_mounts: self.mounts.clone(),
            cwd_input: Some("package".into()),
            filename: filename.clone(),
            kind,
            metadata_arguments,
            metadata_environment,
        })?;
        let bundle = f
            .exports
            .get(&format!("{id}.bundle"))
            .ok_or_else(|| error("translation omitted exact bundle"))?
            .clone();
        let artifact = f
            .exports
            .get(&format!("{id}.artifact"))
            .ok_or_else(|| error("translation omitted artifact"))?
            .clone();
        let generated_tmp = if generated.is_empty() {
            None
        } else {
            Some(
                f.exports
                    .get(&format!("{id}.generated-tmp"))
                    .ok_or_else(|| error("translation omitted actual generated workspace"))?
                    .clone(),
            )
        };
        self.steps.extend(f.steps);
        self.files.extend(f.files);
        self.exports.extend(f.exports);
        if let Some(tmp) = generated_tmp {
            self.capture_generated(id, tmp, &mut generated)?;
        }
        if kind == TranslationKind::ProcMacro {
            proc_macros.insert(filename.clone(), artifact.clone());
        }
        Ok(Compiled {
            bundle,
            artifact,
            filename,
            name: name.into(),
            searches: used,
            proc_macros,
            generated,
        })
    }
}
fn environment(package: &Package, manifest_path: &str) -> BTreeMap<String, RecipeWord> {
    let mut e = BTreeMap::new();
    for (k, v) in [
        ("CARGO_PKG_NAME", package.name.as_str()),
        ("CARGO_PKG_VERSION", package.version.as_str()),
        ("HOST", TRIPLE),
        ("TARGET", TRIPLE),
        ("NUM_JOBS", "1"),
        ("OPT_LEVEL", "2"),
        ("DEBUG", "0"),
        ("PROFILE", "release"),
        ("RUSTC_BOOTSTRAP", "1"),
        ("CFG_COMPILER_HOST_TRIPLE", TRIPLE),
        ("CFG_RELEASE", "1.90.0"),
        ("CFG_RELEASE_CHANNEL", "stable"),
        ("CFG_VERSION", "1.90.0-stable-mrustc"),
        ("CFG_PREFIX", "mrustc"),
        ("CFG_LIBDIR_RELATIVE", "lib"),
        ("REAL_LIBRARY_PATH_VAR", "LD_LIBRARY_PATH"),
        ("RUSTC_INSTALL_BINDIR", "bin"),
    ] {
        e.insert(k.into(), lit(v));
    }
    let numbers = package.version.split(['.', '-', '+']).collect::<Vec<_>>();
    for (i, n) in ["MAJOR", "MINOR", "PATCH"].into_iter().enumerate() {
        e.insert(format!("CARGO_PKG_VERSION_{n}"), lit(numbers.get(i).copied().unwrap_or("0")));
    }
    e.insert("CARGO_MANIFEST_DIR".into(), input("package", manifest_path));
    for feature in &package.features {
        e.insert(format!("CARGO_FEATURE_{}", feature.replace('-', "_").to_uppercase()), lit("1"));
    }
    e
}
fn library(p: &Package, s: &Sources) -> Option<(String, String, TranslationKind)> {
    let lib = p.manifest.get("lib");
    let path = lib.and_then(|l| l.get("path")).and_then(toml::Value::as_str).unwrap_or("src/lib.rs");
    if lib.is_none() && !s.members.contains(&format!("{}/{path}", p.directory)) {
        return None;
    }
    let name = lib.and_then(|v| v.get("name")).and_then(toml::Value::as_str).unwrap_or(&p.name).replace('-', "_");
    let kind = if lib.and_then(|v| v.get("proc-macro")).and_then(toml::Value::as_bool) == Some(true) {
        TranslationKind::ProcMacro
    } else {
        TranslationKind::Rlib
    };
    Some((name, path.into(), kind))
}
fn transitive(
    key: &Key,
    topology: &Topology,
    compiled: &BTreeMap<Key, Compiled>,
    build: bool,
) -> Vec<(String, Compiled)> {
    let edges = if build {
        &topology.packages[key].build_dependencies
    } else {
        &topology.packages[key].dependencies
    };
    edges
        .iter()
        .filter_map(|edge| {
            compiled.get(&edge.key).cloned().map(|c| {
                let alias = if build || edge.name == topology.packages[&edge.key].name.replace('-', "_") {
                    c.name.clone()
                } else {
                    edge.name.clone()
                };
                (alias, c)
            })
        })
        .collect()
}
fn graph(
    c: &mut Context<'_, '_>,
    sources: &Sources,
    topology: &Topology,
    prefix: &str,
    output_directory: &str,
    stdlib: Option<&ArtifactRef>,
    seed: bool,
) -> Result<CompiledScripts, RunError> {
    let mut compiled = BTreeMap::new();
    let mut scripts = BTreeMap::new();
    for (key_index, key) in topology.order.iter().enumerate() {
        let p = &topology.packages[key];
        let id = format!("minicargo.{prefix}.{key_index}.{}", p.name);
        let package = c.source.clone();
        let (package_env, package_inputs) = c.environment(p, sources)?;
        let linked = p
            .dependencies
            .iter()
            .filter_map(|edge| {
                sources
                    .package(&edge.key)
                    .ok()
                    .and_then(|dependency| dependency.links.zip(scripts.get(&edge.key).cloned()))
            })
            .collect::<Vec<(String, Script)>>();
        let mut own = None;
        let original_script = sources.package(key)?.script;
        if let Some(path) = original_script {
            if seed {
                let override_path = format!("mrustc-0.12.0/script-overrides/stable-1.90.0-linux/build_{}.txt", p.name);
                let stdout = c.select(
                    &format!("{id}.original-override"),
                    c.mrustc_source.clone(),
                    &override_path,
                    ArtifactKind::Text,
                );
                let empty = c.merge(&format!("{id}.empty-out-dir"), Vec::<ArtifactRef>::new());
                let logical_out = super::successors::native::out_directory(&format!("{id}.original-override"));
                own = Some(Script {
                    stdout,
                    out_dir: empty.clone(),
                    package: package.clone(),
                    logical_out: logical_out.clone(),
                    generated: BTreeMap::from([(logical_out, empty)]),
                });
            } else {
                let script_tree = package.clone();
                let source_path = format!("rustc-1.90.0-src/{path}");
                let deps = transitive(key, topology, &compiled, true);
                let exe = c.compile(
                    &format!("{id}.build-script"),
                    script_tree.clone(),
                    &source_path,
                    "build",
                    format!("build_{}_run", p.name),
                    TranslationKind::BuildScript,
                    &p.edition,
                    None,
                    &p.features,
                    &deps,
                    stdlib,
                    package_env.clone(),
                    None,
                    package_inputs.clone(),
                    BTreeMap::new(),
                    output_directory,
                )?;
                let mut generated = exe.generated;
                for edge in &p.dependencies {
                    if let Some(dependency) = compiled.get(&edge.key) {
                        merge_generated(&mut generated, &dependency.generated);
                    }
                }
                for (_, script) in &linked {
                    merge_generated(&mut generated, &script.generated);
                }
                c.refresh_generated(&mut generated);
                let dep_env = c.script_metadata(&format!("{id}.script-metadata"), &linked)?;
                let script_tree = c.script_world(&format!("{id}.script-world"), stdlib, Some(dep_env))?;
                let mut inputs = package_inputs.clone();
                inputs.extend(c.runtime.clone());
                for name in ["llvm-config", "llvm-native-controller", "llvm-native-helper"] {
                    inputs.insert(
                        name.into(),
                        c.llvm.get(name).ok_or_else(|| error(format!("missing exact LLVM export {name}")))?.clone(),
                    );
                }
                let mut env = package_env.clone();
                env.insert("MRUSTC_CAIRN_SCRIPT_ENV".into(), input("package", "metadata/script-environment"));
                env.insert(
                    "MRUSTC_CAIRN_MANIFEST_DIR".into(),
                    input("package", &format!("rustc-1.90.0-src/{}", p.directory)),
                );
                for (key, path) in [
                    ("CC", "target-wrappers/cc"),
                    ("CXX", "target-wrappers/c++"),
                    ("AR", "phase-tools/ar"),
                    ("RANLIB", "phase-tools/ranlib"),
                    ("RUSTC", "phase-tools/rustc"),
                ] {
                    env.insert(key.into(), input("package", path));
                }
                if stdlib.is_some() {
                    env.insert("MRUSTC_LIBDIR".into(), input("package", "stdlib"));
                }
                if c.llvm.contains_key("llvm-config") {
                    env.insert("LLVM_CONFIG".into(), input("package", "llvm/bin/llvm-config"));
                }
                env.insert("COMPILER_PATH".into(), input("package", "phase-tools"));
                env.insert("MANTLE_GCC_ASSEMBLER".into(), input("package", "phase-tools/as"));
                env.insert("MANTLE_GCC_LINKER".into(), input("package", "phase-tools/ld"));
                for (key, path) in [
                    ("MANTLE_GCC_LINK_LIBC", "runtime/libc.so"),
                    ("MANTLE_GCC_LINK_LIBM", "runtime/libc.so"),
                    ("MANTLE_GCC_LINK_LIBGCC", "runtime/libgcc.a"),
                    ("MANTLE_GCC_LINK_LIBGCC_EH", "runtime/libgcc_eh.a"),
                    ("MANTLE_GCC_LINK_LIBGCC_S", "runtime/libgcc_s.so.1"),
                    ("MANTLE_GCC_LINK_CRT", "runtime"),
                ] {
                    env.insert(key.into(), input("package", path));
                }
                env.insert("PATH".into(), target_tool_path());
                super::successors::native::extend_environment(c.request, &mut env)?;
                let source_files =
                    u32::try_from(c.reservation_hints.get(&p.key.manifest).map(|h| h.native_files).unwrap_or(0))
                        .map_err(|_| error("native source-file reservation hint exceeds u32"))?;
                let slots_max = super::successors::native::reservation_slots_max(c.remaining_steps()?)?;
                let slots = c.reservation_policy.native_slots(&format!("{id}.run-script"), source_files, slots_max)?;
                let run = super::successors::native::schedule(
                    c.request,
                    &format!("{id}.run-script"),
                    exe.artifact,
                    script_tree.clone(),
                    c.mrustc.clone(),
                    c.native_helper.clone().ok_or_else(|| error("native helper not compiled after seed std"))?,
                    env,
                    inputs,
                    slots,
                    &generated,
                )?;
                c.steps.extend(run.steps);
                c.capture_generated(&format!("{id}.run-script"), run.generated_tmp, &mut generated)?;
                if generated.insert(run.logical_out.clone(), run.out_dir.clone()).is_some() {
                    return Err(error(format!(
                        "native script overwrites captured output authority at {}",
                        run.logical_out
                    )));
                }
                c.generated_current.insert(run.logical_out.clone(), run.out_dir.clone());
                own = Some(Script {
                    stdout: run.stdout,
                    out_dir: run.out_dir,
                    package: script_tree,
                    logical_out: run.logical_out,
                    generated,
                });
            }
        }
        if let Some((name, path, kind)) = library(p, sources) {
            let deps = transitive(key, topology, &compiled, false);
            let linked_refs = linked.iter().map(|(a, b)| (a.as_str(), b)).collect::<Vec<_>>();
            let metadata =
                c.metadata(&format!("{id}.compile-metadata"), package.clone(), own.as_ref(), &linked_refs)?;
            let mut env = package_env;
            if let Some(script) = &own {
                env.insert("OUT_DIR".into(), logical_output(&script.logical_out));
            }
            let suffix = if p.version == "0.0.0" {
                String::new()
            } else {
                resolve::suffix(p)
            };
            let filename = if kind == TranslationKind::ProcMacro {
                format!("lib{name}{suffix}-plugin")
            } else {
                format!("lib{name}{suffix}.rlib")
            };
            let source_path = format!("rustc-1.90.0-src/{}/{path}", p.directory);
            let result = c.compile(
                &format!("{id}.library"),
                metadata.world,
                &source_path,
                &name,
                filename,
                kind,
                &p.edition,
                Some(resolve::suffix(p).trim_start_matches('-')),
                &p.features,
                &deps,
                stdlib,
                env,
                Some((metadata.arguments, metadata.environment)),
                package_inputs,
                metadata.generated,
                output_directory,
            )?;
            compiled.insert(key.clone(), result);
        }
        if let Some(script) = own {
            let authority = if seed {
                "original-source-override"
            } else {
                "observed-script"
            };
            c.exports.insert(format!("{id}.{authority}.stdout"), script.stdout.clone());
            c.exports.insert(format!("{id}.{authority}.out-dir"), script.out_dir.clone());
            scripts.insert(key.clone(), script);
        }
    }
    Ok((compiled, scripts))
}
fn mrustc_proc_manifest(request: &AuthoringRequest<'_>) -> Result<(toml::Value, BTreeSet<String>), RunError> {
    use sha2::Digest;
    use sha2::Sha256;
    let root = request
        .source_roots
        .get(&SourceAuthorityRole::RustSourceArchiveSet)
        .ok_or_else(|| error("missing original archive authority"))?;
    let mut pending = vec![root.clone()];
    let mut found = None;
    let mut buffer = [0; 65536];
    while let Some(path) = pending.pop() {
        let m = std::fs::symlink_metadata(&path).map_err(|e| error(e.to_string()))?;
        if m.is_dir() {
            for e in std::fs::read_dir(path).map_err(|e| error(e.to_string()))? {
                pending.push(e.map_err(|e| error(e.to_string()))?.path());
            }
        } else if m.is_file() {
            let mut file = std::fs::File::open(&path).map_err(|e| error(e.to_string()))?;
            let mut hash = Sha256::new();
            loop {
                let n = file.read(&mut buffer).map_err(|e| error(e.to_string()))?;
                if n == 0 {
                    break;
                }
                hash.update(&buffer[..n]);
            }
            if format!("{:x}", hash.finalize()) == "c1ba35f5fc5c4ca2952d9f5526e900dcb6632ea7fd4d71fa58029b3bb563ae56"
                && found.replace(path).is_some()
            {
                return Err(error("duplicate original mrustc archive"));
            }
        }
    }
    let mut archive = tar::Archive::new(flate2::read::GzDecoder::new(
        std::fs::File::open(found.ok_or_else(|| error("original mrustc archive missing"))?)
            .map_err(|e| error(e.to_string()))?,
    ));
    let mut manifest = None;
    let mut members = BTreeSet::new();
    for entry in archive.entries().map_err(|e| error(e.to_string()))? {
        let mut entry = entry.map_err(|e| error(e.to_string()))?;
        let path = entry.path().map_err(|e| error(e.to_string()))?.to_string_lossy().into_owned();
        if let Some(path) = path.strip_prefix("mrustc-0.12.0/lib/libproc_macro/") {
            members.insert(path.into());
            if path == "Cargo.toml" {
                let mut text = String::new();
                entry.read_to_string(&mut text).map_err(|e| error(e.to_string()))?;
                manifest = Some(text.parse().map_err(|e| error(format!("original proc_macro manifest: {e}")))?);
            }
        }
    }
    Ok((manifest.ok_or_else(|| error("missing original proc_macro manifest"))?, members))
}
/// `source_tree` is the cold patched full-top Rust tree. `mrustc_source` is the
/// cold patched full-top mrustc tree, preserving original overrides/proc_macro.
/// LLVM exports: llvm-config, llvm-native-controller and llvm-native-helper
/// executables; llvm-root is the installed bin/include/lib prefix.
/// The authenticated policy reserves resources, not claimed command counts or
/// expansion bounds. Exact overrides use `{translation_id}.macro` and
/// `{package_phase_id}.run-script`; exhaustion remains an explicit failure.
pub(crate) fn author(
    request: &AuthoringRequest<'_>,
    source_tree: ArtifactRef,
    mrustc_source: ArtifactRef,
    mrustc: ArtifactRef,
    minicargo: ArtifactRef,
    translation_helper: ArtifactRef,
    graph_helper: ArtifactRef,
    llvm_exports: &BTreeMap<String, ArtifactRef>,
    runtime_members: &BTreeMap<String, ArtifactRef>,
    reservation_policy: &ContinuationReservationPolicy,
) -> Result<Fragment, RunError> {
    for r in [
        &source_tree,
        &mrustc_source,
        &mrustc,
        &minicargo,
        &translation_helper,
        &graph_helper,
    ] {
        require(r)?;
    }
    for name in [
        "llvm-config",
        "llvm-root",
        "llvm-native-controller",
        "llvm-native-helper",
    ] {
        require(llvm_exports.get(name).ok_or_else(|| error(format!("missing exact LLVM export {name}")))?)?;
    }
    let native_runtime = runtime_members.get("tree").ok_or_else(|| error("missing source runtime tree"))?.clone();
    require(&native_runtime)?;
    let runtime = BTreeMap::from([("rust-runtime".into(), native_runtime.clone())]);
    let mounts = BTreeMap::from([("/mantle-rust-runtime".into(), "rust-runtime".into())]);
    let mut c = Context {
        request,
        source: source_tree,
        source_root: None,
        world_members: BTreeMap::new(),
        standard_generated: BTreeMap::new(),
        generated_current: BTreeMap::new(),
        mrustc_source,
        mrustc,
        translator: translation_helper,
        helper: graph_helper,
        native_helper: None,
        native_members: None,
        cfg: None,
        reservation_policy,
        reservation_hints: BTreeMap::new(),
        llvm: llvm_exports.clone(),
        steps: Vec::new(),
        files: BTreeMap::from([(HELPER_PATH.into(), HELPER_SOURCE.into())]),
        exports: BTreeMap::new(),
        package_metadata: BTreeMap::new(),
        runtime,
        native_runtime,
        mounts,
    };
    c.exports.insert("bootstrap.source-utility".into(), c.helper.clone());
    let cfg_id = "minicargo.target-cfg";
    c.exec(
        cfg_id,
        c.mrustc.clone(),
        BTreeMap::new(),
        vec![lit("--target"), lit(TRIPLE), lit("-Z"), lit("print-cfgs")],
        BTreeMap::new(),
        outputs("stdout", ArtifactKind::Text),
    );
    let cfg = product(cfg_id, "stdout");
    c.exports.insert("observed.mrustc-target-cfg".into(), cfg.clone());
    c.cfg = Some(cfg);
    let sources = resolve::load(request)?;
    let std_goals = [
        ("library/std/Cargo.toml", &[][..]),
        ("library/panic_unwind/Cargo.toml", &[][..]),
        ("library/test/Cargo.toml", &[][..]),
        ("library/rustc-std-workspace-core/Cargo.toml", &[][..]),
        ("library/rustc-std-workspace-alloc/Cargo.toml", &[][..]),
        ("library/rustc-std-workspace-std/Cargo.toml", &[][..]),
    ];
    let seed_topology = resolve::resolve(&sources, &std_goals, false)?;
    let std_topology = resolve::resolve(&sources, &std_goals, true)?;
    let compiler_topology = resolve::resolve(&sources, &[("compiler/rustc/Cargo.toml", &["llvm"])], true)?;
    let cargo_topology =
        resolve::resolve(&sources, &[("src/tools/cargo/Cargo.toml", &["vendored-openssl", "all-static"])], true)?;
    let selected = [&seed_topology, &std_topology, &compiler_topology, &cargo_topology]
        .into_iter()
        .flat_map(|topology| topology.packages.keys().map(|key| key.manifest.clone()))
        .collect();
    c.reservation_hints = resolve::reservation_hints(request, &sources, &selected)?;
    // The C source helper is already Produced before any std-dependent Rust
    // helper exists. Apply the admitted exact original-source patch to the seed
    // too, so genuine build-script Command::output/status use the shared seam.
    let process_overlay = "minicargo.std-process-overlay";
    c.exec(
        process_overlay,
        c.helper.clone(),
        BTreeMap::from([
            ("source".into(), c.source.clone()),
            ("descriptor".into(), super::helper_source_ref(STD_PROCESS_DESCRIPTOR_PATH)),
            ("module".into(), super::helper_source_ref(STD_PROCESS_MODULE_PATH)),
        ]),
        vec![
            lit("std-process-overlay"),
            input("source", ""),
            output("tree"),
            input("descriptor", ""),
            input("module", ""),
        ],
        BTreeMap::new(),
        outputs("tree", ArtifactKind::Tree),
    );
    c.source = product(process_overlay, "tree");
    let (seed, _) = graph(&mut c, &sources, &seed_topology, "stdlib-seed", "output", None, true)?;
    let seed_tree = c.merge("minicargo.stdlib-seed", seed.values().map(|v| v.bundle.clone()));
    let mut seed_generated = BTreeMap::new();
    for crate_ in seed.values() {
        merge_generated(&mut seed_generated, &crate_.generated);
    }
    c.standard_generated.insert(seed_tree.clone(), seed_generated);
    c.exports.insert("seed.std-override-provider".into(), seed_tree.clone());
    // Bootstrap the pure-I/O native continuation helper using the source-selected
    // std seed, before any real generator or source boundary overlay is executed.
    let helper_source = super::successors::helper_source();
    let helper_path = "mantle-source-authoring/rust-minicargo-native-helper.rs";
    c.files.insert(helper_path.into(), helper_source);
    let source_ref = super::helper_source_ref(helper_path);
    let source_id = "minicargo.native-helper.source";
    c.steps.push(RecipeStep {
        id: source_id.into(),
        inputs: BTreeMap::from([("source".into(), source_ref)]),
        outputs: BTreeMap::from([("tree".into(), spec(ArtifactKind::Tree))]),
        operation: RecipeOperation::AssembleTree {
            output: "tree".into(),
            entries: BTreeMap::from([("helper.rs".into(), "source".into())]),
        },
    });
    let helper = c.compile(
        "minicargo.native-helper",
        product(source_id, "tree"),
        "helper.rs",
        "minicargo_native_helper",
        "minicargo-native-helper".into(),
        TranslationKind::Bin,
        "2021",
        None,
        &BTreeSet::new(),
        &[],
        Some(&seed_tree),
        BTreeMap::new(),
        None,
        BTreeMap::new(),
        BTreeMap::new(),
        "output",
    )?;
    c.native_helper = Some(helper.artifact.clone());
    let mut cc_sources = BTreeSet::new();
    let mut script_sources = BTreeSet::new();
    for topology in [&std_topology, &compiler_topology, &cargo_topology] {
        for p in topology.packages.values() {
            if p.name == "cc" {
                cc_sources.insert(format!("rustc-1.90.0-src/{}", p.directory));
            }
            if let Some(script) = &p.script {
                script_sources.insert(format!("rustc-1.90.0-src/{script}"));
            }
        }
    }
    let overlay_id = "minicargo.cc-overlay";
    let mut args = vec![lit("--cc-overlay"), input("source", ""), output("tree")];
    for path in cc_sources {
        args.push(input("source", &path));
    }
    c.exec(
        overlay_id,
        helper.artifact,
        BTreeMap::from([("source".into(), c.source.clone())]),
        args,
        BTreeMap::new(),
        outputs("tree", ArtifactKind::Tree),
    );
    c.source = product(overlay_id, "tree");
    let scripts_id = "minicargo.script-entry-overlay";
    let mut args = vec![lit("scripts"), input("source", ""), output("tree")];
    for path in script_sources {
        args.push(input("source", &path));
    }
    c.exec(
        scripts_id,
        c.helper.clone(),
        BTreeMap::from([("source".into(), c.source.clone())]),
        args,
        BTreeMap::new(),
        outputs("tree", ArtifactKind::Tree),
    );
    c.source = product(scripts_id, "tree");
    let (standard, _) = graph(&mut c, &sources, &std_topology, "stdlib", "output", Some(&seed_tree), false)?;
    let mut std_bundles = standard.values().map(|v| v.bundle.clone()).collect::<Vec<_>>();
    let mut standard_generated = BTreeMap::new();
    for compiled in standard.values() {
        merge_generated(&mut standard_generated, &compiled.generated);
    }
    let (proc_manifest, proc_members) = mrustc_proc_manifest(request)?;
    let proc_sources = Sources {
        top: "mrustc-0.12.0".into(),
        manifests: BTreeMap::from([("lib/libproc_macro/Cargo.toml".into(), proc_manifest)]),
        locks: BTreeMap::new(),
        members: proc_members.into_iter().map(|m| format!("lib/libproc_macro/{m}")).collect(),
        policy: BTreeMap::new(),
    };
    let proc_topology = resolve::resolve(&proc_sources, &[("lib/libproc_macro/Cargo.toml", &[])], true)?;
    let p = &proc_topology.packages[&proc_topology.roots[0]];
    let package = c.select(
        "minicargo.proc-macro.source",
        c.mrustc_source.clone(),
        "mrustc-0.12.0/lib/libproc_macro",
        ArtifactKind::Tree,
    );
    let (name, path, kind) = library(p, &proc_sources).ok_or_else(|| error("original proc_macro has no library"))?;
    let proc = c.compile(
        "minicargo.proc-macro",
        package,
        &path,
        &name,
        "libproc_macro.rlib".into(),
        kind,
        &p.edition,
        Some(resolve::suffix(p).trim_start_matches('-')),
        &p.features,
        &[],
        Some(&seed_tree),
        environment(p, ""),
        None,
        BTreeMap::new(),
        BTreeMap::new(),
        "output",
    )?;
    merge_generated(&mut standard_generated, &proc.generated);
    std_bundles.push(proc.bundle);
    let stdlib = c.merge("minicargo.stdlib-provider", std_bundles);
    c.standard_generated.insert(stdlib.clone(), standard_generated);
    c.exports.insert("mrustc.std".into(), stdlib.clone());
    let mut binaries = BTreeMap::new();
    for (label, topology) in [("rustc", compiler_topology), ("cargo", cargo_topology)] {
        let (compiled, scripts) =
            graph(&mut c, &sources, &topology, label, &format!("output/{label}-build"), Some(&stdlib), false)?;
        let root = &topology.roots[0];
        let p = &topology.packages[root];
        let package = c.source.clone();
        let mut targets = Vec::new();
        if let Some(bins) = p.manifest.get("bin").and_then(toml::Value::as_array) {
            for bin in bins {
                let name = bin.get("name").and_then(toml::Value::as_str).unwrap_or(&p.name);
                let path = bin.get("path").and_then(toml::Value::as_str).map(str::to_string).unwrap_or_else(|| {
                    if sources.members.contains(&format!("{}/src/bin/{name}.rs", p.directory)) {
                        format!("src/bin/{name}.rs")
                    } else {
                        "src/main.rs".into()
                    }
                });
                targets.push((name.replace('-', "_"), path, resolve::strings(bin.get("required-features"))));
            }
        }
        if targets.is_empty() && sources.members.contains(&format!("{}/src/main.rs", p.directory)) {
            targets.push((p.name.replace('-', "_"), "src/main.rs".into(), Vec::new()));
        }
        for (name, path, required) in targets {
            if !required.iter().all(|f| p.features.contains(f)) {
                continue;
            }
            let id = format!("minicargo.{label}.binary.{name}");
            let mut deps = transitive(root, &topology, &compiled, false);
            if let Some(own) = compiled.get(root) {
                deps.push((own.name.clone(), own.clone()));
            }
            let linked = p
                .dependencies
                .iter()
                .filter_map(|edge| {
                    topology.packages[&edge.key]
                        .links
                        .as_ref()
                        .and_then(|links| scripts.get(&edge.key).map(|script| (links.as_str(), script)))
                })
                .collect::<Vec<_>>();
            let metadata = c.metadata(&format!("{id}.metadata"), package.clone(), scripts.get(root), &linked)?;
            let (mut env, field_inputs) = c.environment(p, &sources)?;
            if let Some(script) = scripts.get(root) {
                env.insert("OUT_DIR".into(), logical_output(&script.logical_out));
            }
            let source_path = format!("rustc-1.90.0-src/{}/{path}", p.directory);
            let binary = c.compile(
                &id,
                metadata.world,
                &source_path,
                &name,
                name.clone(),
                TranslationKind::Bin,
                &p.edition,
                Some(resolve::suffix(p).trim_start_matches('-')),
                &p.features,
                &deps,
                Some(&stdlib),
                env,
                Some((metadata.arguments, metadata.environment)),
                field_inputs,
                metadata.generated,
                &format!("output/{label}-build"),
            )?;
            if (label == "rustc" && name == "rustc_main") || (label == "cargo" && name == "cargo") {
                binaries.insert(label.to_string(), binary.artifact);
            }
        }
    }
    for name in ["rustc", "cargo"] {
        let r = binaries
            .get(name)
            .ok_or_else(|| error(format!("selected source graph omitted final {name} binary")))?
            .clone();
        c.exports.insert(name.into(), r.clone());
        c.exports.insert(format!("bootstrap.{name}"), r);
    }
    let provider_id = "minicargo.bootstrap-provider";
    let mut provider_inputs = binaries;
    provider_inputs.insert("mrustc-stdlib".into(), stdlib.clone());
    c.steps.push(RecipeStep {
        id: provider_id.into(),
        inputs: provider_inputs,
        outputs: BTreeMap::from([("tree".into(), spec(ArtifactKind::Tree))]),
        operation: RecipeOperation::AssembleTree {
            output: "tree".into(),
            entries: BTreeMap::from([
                ("bin/rustc".into(), "rustc".into()),
                ("bin/cargo".into(), "cargo".into()),
                ("lib/mrustc".into(), "mrustc-stdlib".into()),
            ]),
        },
    });
    let provider = product(provider_id, "tree");
    c.exports.insert("bootstrap.provider".into(), provider.clone());
    c.exports.insert("provider".into(), provider);
    Ok(Fragment {
        steps: c.steps,
        files: c.files,
        exports: c.exports,
    })
}
