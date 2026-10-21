//! Native original-node setup shared by historical GNU and final host authors.
use std::collections::BTreeMap;

use super::ControllerHooks;
use super::fail;
use super::root::OriginalShellNode;
use super::root::lower_original_shell_node;
use crate::errors::RunError;
use crate::source_built_native_programs::NativeNodeProgram;
use crate::source_built_native_programs::NativeNodeRequest;
use crate::source_built_native_programs::STAGE;
use crate::source_built_recipe::ArtifactKind;
use crate::source_built_recipe::ArtifactRef;
use crate::source_built_recipe::RecipePreparation;
use crate::source_built_recipe::RecipeStep;
use crate::source_built_recipe::RecipeTimeoutGroup;
use crate::source_built_recipe::SourceRecipe;

fn inventory_add(images: &mut Vec<(String, ArtifactRef)>, name: &str, image: &ArtifactRef) {
    if !images.iter().any(|(old, reference)| old == name && reference == image) {
        images.push((name.into(), image.clone()));
    }
}
fn installed_image(path: &str) -> bool {
    let name = path.rsplit('/').next().unwrap_or(path);
    if path.starts_with("libexec/") {
        return matches!(name, "cc1" | "cc1plus" | "collect2" | "cpp");
    }
    path.starts_with("bin/")
        && !name.ends_with(".h")
        && !name.ends_with(".a")
        && !name.ends_with(".la")
        && !name.ends_with(".pc")
}
pub(crate) fn original_node_with(
    request: &NativeNodeRequest<'_, '_>,
    source_overrides: BTreeMap<String, ArtifactRef>,
    additional_tools: BTreeMap<String, ArtifactRef>,
    installed_members: BTreeMap<String, BTreeMap<String, ArtifactKind>>,
    phase_members: BTreeMap<String, (String, String)>,
    extensions: &[ControllerHooks<'_>],
) -> Result<NativeNodeProgram, RunError> {
    original_node_prepared(
        request,
        source_overrides,
        additional_tools,
        installed_members,
        phase_members,
        extensions,
        Vec::new(),
        Vec::new(),
        BTreeMap::new(),
    )
}
pub(crate) fn original_node_prepared(
    request: &NativeNodeRequest<'_, '_>,
    source_overrides: BTreeMap<String, ArtifactRef>,
    additional_tools: BTreeMap<String, ArtifactRef>,
    installed_members: BTreeMap<String, BTreeMap<String, ArtifactKind>>,
    phase_members: BTreeMap<String, (String, String)>,
    extensions: &[ControllerHooks<'_>],
    steps: Vec<RecipeStep>,
    timeout_groups: Vec<RecipeTimeoutGroup>,
    source_files: BTreeMap<String, String>,
) -> Result<NativeNodeProgram, RunError> {
    original_node_prepared_script(
        request,
        source_overrides,
        additional_tools,
        installed_members,
        phase_members,
        extensions,
        steps,
        timeout_groups,
        source_files,
        None,
    )
}
pub(crate) fn original_node_prepared_script(
    request: &NativeNodeRequest<'_, '_>,
    source_overrides: BTreeMap<String, ArtifactRef>,
    additional_tools: BTreeMap<String, ArtifactRef>,
    installed_members: BTreeMap<String, BTreeMap<String, ArtifactKind>>,
    phase_members: BTreeMap<String, (String, String)>,
    extensions: &[ControllerHooks<'_>],
    steps: Vec<RecipeStep>,
    timeout_groups: Vec<RecipeTimeoutGroup>,
    source_files: BTreeMap<String, String>,
    script: Option<String>,
) -> Result<NativeNodeProgram, RunError> {
    let id = format!("{}.host.setup", request.id);
    let mut preparation = SourceRecipe {
        schema: crate::source_built_recipe::SCHEMA.into(),
        stage_id: STAGE.into(),
        preparation: RecipePreparation {
            tool: "native original node source setup".into(),
            provenance:
                "real predecessor TCC compiling shared original source interpreter and transparent source projector"
                    .into(),
        },
        steps,
        timeout_groups,
        roots: Vec::new(),
        exports: BTreeMap::new(),
        semantic_roots: BTreeMap::new(),
    };
    let controller_member = format!("source-built/native/{id}-controller.c");
    let projector_member = format!("source-built/native/{id}-projector.c");
    let controller_source = super::controller_source(extensions)?;
    let mut files = BTreeMap::from([
        (controller_member.clone(), controller_source),
        (projector_member.clone(), include_str!("source_built_native_programs_host_transform.c").into()),
    ]);
    files.extend(source_files);
    let controller = crate::source_built_native_programs::append_stagex_c_helper(
        request.request,
        &mut preparation,
        &format!("{id}.controller"),
        &controller_member,
    )?;
    let projector = crate::source_built_native_programs::append_stagex_c_helper(
        request.request,
        &mut preparation,
        &format!("{id}.projector"),
        &projector_member,
    )?;
    let mut tools = Vec::new();
    let mut command_inputs = BTreeMap::new();
    let mut runtime_mounts = BTreeMap::new();
    for role in [
        "tcc",
        "tcc-selfhost",
        "tcc-0.9.26",
        "ar",
        "as",
        "ld",
        "nm",
        "objcopy",
        "objdump",
        "ranlib",
        "readelf",
        "strip",
    ] {
        if let Some(image) = request.request.predecessor_exports.get(&format!("stagex-provider-publication:{role}")) {
            inventory_add(&mut tools, role, image);
            inventory_add(&mut tools, &format!("x86_64-linux-musl-{role}"), image);
        }
    }
    for (key, image) in request.request.predecessor_exports {
        if let Some(path) = key.strip_prefix("stagex-transition:member:")
            && path.contains("/bin/")
        {
            inventory_add(&mut tools, path.rsplit('/').next().unwrap_or(path), image);
        }
    }
    if let Some(image) = request.request.predecessor_exports.get("stagex-transition:phase-tool:tcc-musl-v2") {
        inventory_add(&mut tools, "tcc", image);
        inventory_add(&mut tools, "x86_64-linux-musl-tcc", image);
    }
    if let Some(image) = request.request.predecessor_exports.get("stagex-transition:phase-tool:binutils-ld") {
        inventory_add(&mut tools, "ld", image);
        inventory_add(&mut tools, "x86_64-linux-musl-ld", image);
    }
    for program in request.dependencies.values() {
        for members in program.members.values() {
            for (path, image) in members {
                if installed_image(path) {
                    inventory_add(&mut tools, path.rsplit('/').next().unwrap_or(path), image);
                }
            }
        }
        for (name, image) in &program.phase_tools {
            if !name.ends_with("root")
                && !name.ends_with("catalog")
                && !name.ends_with("source")
                && !name.ends_with("runtime")
            {
                inventory_add(&mut tools, name.rsplit('/').next().unwrap_or(name), image);
            }
        }
    }
    let prefix = &request.request.proof.logical_store_prefix;
    let literals = request
        .node
        .derivation
        .input_sources
        .iter()
        .map(|path| path.to_absolute_path_with_prefix(prefix))
        .collect::<std::collections::BTreeSet<_>>();
    let mut literal_roots = BTreeMap::new();
    for (name, reference) in additional_tools {
        if name.starts_with('/') {
            if !literals.contains(&name) {
                return Err(fail(format!("explicit literal runtime is not an original input source {name}")));
            }
            literal_roots.insert(name, reference);
        } else {
            inventory_add(&mut tools, &name, &reference);
        }
    }
    for logical in literals {
        let reference = if let Some(reference) = literal_roots.remove(&logical) {
            reference
        } else if logical.ends_with("-mantle-stagex-intermediate-provider") {
            request
                .request
                .predecessor_exports
                .get("stagex-provider-publication:stagex-provider")
                .cloned()
                .ok_or_else(|| fail("missing exact freshly source-built StageX provider tree"))?
        } else if logical.ends_with("-mantle-stagex-transition") {
            request
                .request
                .predecessor_exports
                .get("stagex-transition:stagex-transition")
                .cloned()
                .ok_or_else(|| fail("missing exact freshly source-built StageX transition tree"))?
        } else {
            return Err(fail(format!(
                "original literal input source requires its exact constructed runtime tree {logical}"
            )));
        };
        let name = format!("literal-runtime-{}", runtime_mounts.len());
        command_inputs.insert(name.clone(), reference);
        runtime_mounts.insert(logical, name);
    }
    if tools.is_empty() {
        return Err(fail("original node has no individually source-built command images"));
    }
    let inherited = request.dependencies.values().find_map(|program| {
        Some((
            program.phase_tools.get("bootstrap.busybox")?.clone(),
            program.phase_tools.get("bootstrap.busybox-root")?.clone(),
        ))
    });
    let (bootstrap_image, bootstrap_root) = if let Some(inherited) = inherited {
        inherited
    } else {
        super::bootstrap::append(request, &mut preparation, &controller, &projector, &tools, &mut files)?
    };
    inventory_add(&mut tools, "busybox", &bootstrap_image);
    for name in "awk basename cat chmod cmp comm cp cut date dd dirname echo env expr false find grep egrep fgrep head hostname id install ln ls mkdir mktemp mv od paste printenv printf pwd readlink realpath rm rmdir sed sh sleep sort split tail tee test [ [[ touch tr true uname uniq wc which whoami timeout xargs".split_whitespace(){inventory_add(&mut tools,name,&bootstrap_image);}
    command_inputs.insert("bootstrap-busybox-root".into(), bootstrap_root.clone());
    runtime_mounts.insert("/bin".into(), "bootstrap-busybox-root".into());
    let remaining = crate::source_built_recipe::STEP_COUNT_MAX
        .checked_sub(preparation.steps.len())
        .ok_or_else(|| fail("native preparation exceeds admitted declaration capacity"))?;
    let commands_max = crate::source_built_recipe::controller::reserve_native_commands(
        request.reservations,
        &format!("{}.host.original.controller", request.id),
        request.native_source_files,
        remaining,
    )?;
    let mut program = lower_original_shell_node(request, OriginalShellNode {
        namespace: String::new(),
        script,
        controller,
        projector,
        tools,
        command_inputs,
        runtime_mounts,
        source_overrides,
        members: installed_members,
        phase_members,
        steps: preparation.steps,
        timeout_groups: preparation.timeout_groups,
        files,
        commands_max,
    })?;
    program.phase_tools.insert("bootstrap.busybox".into(), bootstrap_image);
    program.phase_tools.insert("bootstrap.busybox-root".into(), bootstrap_root);
    Ok(program)
}
