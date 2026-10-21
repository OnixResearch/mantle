//! Cargo-free Rust source-program admission. The authenticated bootstrap route
//! supplies the independent stage/source/output inventory. It is not executable
//! authority, and its legacy materialization receipts never become events.
use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::io::Read;
use std::path::Path;
use std::path::PathBuf;

use serde::Deserialize;
use sha2::Digest;
use sha2::Sha256;

use crate::errors::RunError;
use crate::source_built_fixed_point::SourceAuthorityRole;
use crate::source_built_fixed_point::SourceBuiltFixedPointPlan;
use crate::source_built_fixed_point_action_trust::ActionExecutionEvent;
use crate::source_built_fixed_point_action_trust::StageActionAdapter;
use crate::source_built_fixed_point_stagex_actions::ExpandedNode;
use crate::source_built_fixed_point_stagex_actions::depends_on;
use crate::source_built_fixed_point_stagex_actions::require_export;
use crate::source_built_fixed_point_stagex_actions::require_node_input;
use crate::source_built_fixed_point_stagex_actions::used_inputs;
use crate::source_built_fixed_point_stagex_actions::validate_partition;
use crate::source_built_recipe::ArtifactKind;
use crate::source_built_recipe::ArtifactRef;
use crate::source_built_recipe::PreparedRecipe;
use crate::source_built_recipe::RecipeOperation;
use crate::source_built_recipe::RecipePaths;
use crate::source_built_recipe::RecipeRun;
use crate::source_built_recipe::{self as recipe};
use crate::source_toolchain_closure::RustProviderRole;
use crate::source_toolchain_closure::RustSourceProviderBootstrapPlan;

const STAGE: &str = "full-source-rust-provider";
const INDEX_SCHEMA: &str = "mantle-rust-source-expansion-v1";
const SOURCE_BYTES_MAX: u64 = 64 * 1024 * 1024;
const NATIVE_TOOLS: [&str; 16] = [
    "native.cc1",
    "native.cc1plus",
    "native.as",
    "native.ld",
    "native.ar",
    "native.nm",
    "native.ranlib",
    "native.objcopy",
    "native.strip",
    "native.readelf",
    "native.loader",
    "host.make",
    "host.cmake",
    "host.python",
    "host.perl",
    "host.busybox",
];
const NATIVE_TREES: [&str; 3] = ["native.provider", "native.sysroot", "linux.headers"];
const NATIVE_LIBRARIES: [&str; 4] = ["native.libc", "native.libgcc", "native.libgcc-eh", "native.libstdcxx"];

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RustExpansionIndex {
    schema: String,
    recipe_path: String,
    /// Original typed route source, inside the authenticated archive-set root.
    route_path: String,
    /// Original source IDs map to archive bytes, not generated/copied binaries.
    archives: BTreeMap<String, ArtifactRef>,
    stages: BTreeMap<String, ExpandedNode>,
}

pub(crate) struct PreparedRustActions {
    prepared: PreparedRecipe,
    compiler_host: String,
}

impl PreparedRustActions {
    pub(crate) fn compiler_host(&self) -> &str {
        &self.compiler_host
    }
    pub(crate) fn recipe(&self) -> &PreparedRecipe {
        &self.prepared
    }
    pub(crate) fn adapter(&self) -> &StageActionAdapter {
        self.prepared.adapter()
    }
}

fn error(message: impl Into<String>) -> RunError {
    RunError::Build(format!("Rust source actions: {}", message.into()))
}

/// Source products required in RustSourceArchiveSet: the selected expanded leaf
/// recipe, `<path>.expansion.json`, the original typed route and its imports,
/// every route archive, and all original generator/compiler sources. A route
/// plan, x.py/make/Cargo launch, or observed build summary alone is not a leaf
/// source program. The protected executor denies undeclared nested execution.
pub(crate) fn prepare_actions(
    proof: &SourceBuiltFixedPointPlan,
    recipe_ref: &ArtifactRef,
    paths: &RecipePaths<'_>,
    prior: &[&PreparedRecipe],
    host_digest: &str,
) -> Result<PreparedRustActions, RunError> {
    let ArtifactRef::Source {
        role: SourceAuthorityRole::RustSourceArchiveSet,
        path,
    } = recipe_ref
    else {
        return Err(error("source program selector must be Source RustSourceArchiveSet with a path"));
    };
    recipe::relative(path, false)?;
    let root = paths
        .source_roots
        .get(&SourceAuthorityRole::RustSourceArchiveSet)
        .ok_or_else(|| error("missing Rust source archive-set root"))?;
    let authority = proof
        .source_inputs
        .iter()
        .find(|input| input.role == SourceAuthorityRole::RustSourceArchiveSet)
        .ok_or_else(|| error("missing Rust source archive-set authority"))?;
    if recipe::digest(root)?.1 != authority.digest_blake3 {
        return Err(error("Rust archive/program sources differ from admitted input"));
    }
    let index_path = format!("{path}.expansion.json");
    let bytes = read_source(root, &index_path).map_err(|failure| error(format!(
        "missing authenticated leaf source-program input {index_path}; requires original mrustc/minicargo, Rust bootstrap/build-script and LLVM/TableGen leaf expansions for every route stage, not a provider summary or Cargo/x.py child launcher: {failure}"
    )))?;
    let index: RustExpansionIndex =
        serde_json::from_slice(&bytes).map_err(|failure| error(format!("Rust expansion source syntax: {failure}")))?;
    if index.schema != INDEX_SCHEMA || index.recipe_path != *path {
        return Err(error("Rust expansion source selects another schema/program"));
    }
    let route_path = source_path(root, &index.route_path)?;
    // Nickel evaluation reads the original typed route, not a lossy report or
    // an invented list of expected stages. Its transitive imports belong in the
    // same authenticated source archive set supplied by the product author.
    if std::fs::metadata(&route_path).map_err(|failure| error(failure.to_string()))?.len() > SOURCE_BYTES_MAX {
        return Err(error("typed Rust route exceeds source-data bound"));
    }
    let search_dirs: Vec<std::ffi::OsString> = Vec::new();
    recipe::validate_source_imports(root, &route_path, &search_dirs)?;
    let route: RustSourceProviderBootstrapPlan = crunch_eval::evaluate_and_deserialize(&route_path, &search_dirs)
        .map_err(|failure| error(format!("typed Rust route: {failure}")))?;
    crate::source_toolchain_closure::validate_rust_source_provider_bootstrap_plan(&route)
        .map_err(|failure| error(failure.message().to_string()))?;
    let native = one_prior(prior, "full-source-native-provider")?;
    for stage in ["stagex-transition", "stagex-provider-publication"] {
        let _ = one_prior(prior, stage)?;
    }
    validate_native(native)?;
    let prepared = recipe::prepare_from_reference(proof, STAGE, recipe_ref, paths, prior, host_digest)?;
    validate_expansion(&route, &index, &prepared, native, root)?;
    Ok(PreparedRustActions {
        prepared,
        compiler_host: route.host_triple,
    })
}

pub(crate) fn run_with_action_observations(
    actions: &PreparedRustActions,
    emit: &mut dyn FnMut(ActionExecutionEvent) -> Result<(), RunError>,
) -> Result<RecipeRun, RunError> {
    recipe::run(&actions.prepared, emit)
}

fn one_prior<'a>(prior: &[&'a PreparedRecipe], stage: &str) -> Result<&'a PreparedRecipe, RunError> {
    let mut matches = prior.iter().copied().filter(|prepared| prepared.adapter().stage_id == stage);
    let selected = matches.next().ok_or_else(|| error(format!("missing actual prepared prior {stage}")))?;
    if matches.next().is_some() {
        return Err(error(format!("ambiguous prepared prior {stage}")));
    }
    Ok(selected)
}

fn validate_native(native: &PreparedRecipe) -> Result<(), RunError> {
    for role in NATIVE_TOOLS {
        require_export(native, role, ArtifactKind::Executable, None)?;
    }
    for role in NATIVE_TREES {
        require_export(
            native,
            role,
            ArtifactKind::Tree,
            if role == "native.provider" {
                Some("full-source-native-provider")
            } else {
                None
            },
        )?;
    }
    for role in NATIVE_LIBRARIES {
        if !matches!(native.export(role)?.kind, ArtifactKind::Archive | ArtifactKind::Object | ArtifactKind::Executable)
        {
            return Err(error(format!("native runtime must be an individually produced binary: {role}")));
        }
    }
    Ok(())
}

fn validate_expansion(
    route: &RustSourceProviderBootstrapPlan,
    index: &RustExpansionIndex,
    prepared: &PreparedRecipe,
    native: &PreparedRecipe,
    root: &Path,
) -> Result<(), RunError> {
    let stage_ids = route.stages.iter().map(|stage| stage.id.clone()).collect::<BTreeSet<_>>();
    let source_ids = route.sources.iter().map(|source| source.id.clone()).collect::<BTreeSet<_>>();
    if index.stages.keys().ne(stage_ids.iter()) || index.archives.keys().ne(source_ids.iter()) {
        return Err(error("source expansion omits/invents original Rust stages or archives"));
    }
    let program = prepared.source_recipe();
    validate_partition(program, index.stages.values())?;
    for source in &route.sources {
        let ArtifactRef::Source {
            role: SourceAuthorityRole::RustSourceArchiveSet,
            path,
        } = &index.archives[&source.id]
        else {
            return Err(error("route archives must be original admitted source bytes"));
        };
        let archive = source_path(root, path)?;
        if sha256_file(&archive)? != source.sha256_hex {
            return Err(error(format!("original Rust source archive identity drift: {}", source.id)));
        }
    }
    for stage in &route.stages {
        let node = &index.stages[&stage.id];
        let expected = stage.outputs.iter().map(|role| role_name(*role).to_string()).collect::<BTreeSet<_>>();
        if node.outputs.keys().ne(expected.iter()) {
            return Err(error(format!("original Rust stage output inventory changed: {}", stage.id)));
        }
        for source in &stage.source_ids {
            require_node_input(program, node, &index.archives[source])?;
        }
        if !stage.bootstrap_stage_id.is_empty() {
            let previous = index
                .stages
                .get(&stage.bootstrap_stage_id)
                .ok_or_else(|| error("missing original bootstrap predecessor"))?;
            for reference in previous.outputs.values() {
                require_node_input(program, node, reference)?;
            }
            let compiler = previous.outputs.get("rustc").ok_or_else(|| error("bootstrap predecessor lacks rustc"))?;
            if !program.steps.iter().filter(|step| node.step_ids.contains(&step.id)).any(|step| {
                matches!(&step.operation, RecipeOperation::Exec { executable_input, .. } if step.inputs.get(executable_input) == Some(compiler))
            }) { return Err(error("Rust stage never executes its actual source-produced bootstrap compiler")); }
        }
        for (role, reference) in &node.outputs {
            let binding = prepared.bindings.get(reference).ok_or_else(|| error("unbound original Rust output"))?;
            let expected = match role.as_str() {
                "rustc" | "cargo" | "rustdoc" => ArtifactKind::Executable,
                "host-rustlib" | "target-rustlib" => ArtifactKind::Tree,
                "provider-receipt" => ArtifactKind::Text,
                _ => return Err(error("unknown original Rust output role")),
            };
            if binding.kind != expected {
                return Err(error("original Rust output kind drift"));
            }
        }
        let cargo = node.outputs.get("cargo");
        if program.steps.iter().any(|step| {
            matches!(&step.operation, RecipeOperation::Exec { executable_input, .. } if cargo.is_some_and(|reference| step.inputs.get(executable_input) == Some(reference)))
        }) {
            return Err(error("Cargo execution is forbidden even when Cargo was genuinely source-built"));
        }
    }
    let product = &prepared.export(STAGE)?.reference;
    require_export(prepared, STAGE, ArtifactKind::Tree, Some(STAGE))?;
    require_export(prepared, "rustc", ArtifactKind::Executable, None)?;
    require_export(prepared, "toolchain-closure", ArtifactKind::Text, None)?;
    let final_stage = route.stages.last().ok_or_else(|| error("empty original Rust route"))?;
    for output in &route.final_outputs {
        let name = role_name(output.role);
        let reference = index.stages[&final_stage.id]
            .outputs
            .get(name)
            .ok_or_else(|| error("final stage omits original provider role"))?;
        if prepared.export(name)?.reference != *reference || !depends_on(program, product, reference) {
            return Err(error(format!("published Rust provider drops or substitutes original final product {name}")));
        }
    }
    // Verify exact individually produced host/compiler/data dependencies really
    // occur in operands, not merely in a semantic label or attached input list.
    for role in NATIVE_TOOLS.into_iter().chain(NATIVE_TREES).chain(NATIVE_LIBRARIES) {
        let reference = &native.export(role)?.reference;
        if !program.steps.iter().any(|step| used_inputs(step).contains(reference)) {
            return Err(error(format!("Rust expansion omits source-built native role {role}")));
        }
    }
    if !depends_on(program, &prepared.export("toolchain-closure")?.reference, &prepared.export("rustc")?.reference) {
        return Err(error("toolchain closure text is not derived from the actual produced Rust compiler"));
    }
    Ok(())
}

fn role_name(role: RustProviderRole) -> &'static str {
    match role {
        RustProviderRole::Rustc => "rustc",
        RustProviderRole::Cargo => "cargo",
        RustProviderRole::Rustdoc => "rustdoc",
        RustProviderRole::HostRustlib => "host-rustlib",
        RustProviderRole::TargetRustlib => "target-rustlib",
        RustProviderRole::ProviderReceipt => "provider-receipt",
    }
}

fn source_path(root: &Path, member: &str) -> Result<PathBuf, RunError> {
    recipe::relative(member, false)?;
    let mut path = root.to_path_buf();
    for component in Path::new(member).components() {
        path.push(component);
        let metadata = std::fs::symlink_metadata(&path).map_err(|failure| error(failure.to_string()))?;
        if metadata.file_type().is_symlink() {
            return Err(error("symlink in Rust source-program input path"));
        }
    }
    if !std::fs::symlink_metadata(&path).map_err(|failure| error(failure.to_string()))?.is_file() {
        return Err(error("Rust source-program input must be a regular file"));
    }
    Ok(path)
}

fn read_source(root: &Path, member: &str) -> Result<Vec<u8>, RunError> {
    let path = source_path(root, member)?;
    let metadata = std::fs::symlink_metadata(&path).map_err(|failure| error(failure.to_string()))?;
    if metadata.len() > SOURCE_BYTES_MAX {
        return Err(error("source program/index exceeds source-data bound"));
    }
    std::fs::read(path).map_err(|failure| error(failure.to_string()))
}

fn sha256_file(path: &Path) -> Result<String, RunError> {
    let mut file = std::fs::File::open(path).map_err(|failure| error(failure.to_string()))?;
    let mut hasher = Sha256::new();
    let mut buffer = [0u8; 65536];
    loop {
        let count = file.read(&mut buffer).map_err(|failure| error(failure.to_string()))?;
        if count == 0 {
            break;
        }
        hasher.update(&buffer[..count]);
    }
    Ok(format!("{:x}", hasher.finalize()))
}
