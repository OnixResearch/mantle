//! Bind original wrapper generators to the admitted Mantle source revision.
use std::collections::BTreeSet;

use super::error;
use crate::errors::RunError;
use crate::source_built_fixed_point::SourceAuthorityRole;
use crate::source_built_program_authoring::AuthoringRequest;
use crate::source_built_recipe::ArtifactRef;
use crate::source_built_recipe::SourceRecipe;

fn function<'a>(source: &'a str, name: &str) -> Result<&'a str, RunError> {
    let needle = format!("fn {name}(");
    let mut matches = source.match_indices(&needle);
    let (start, _) = matches.next().ok_or_else(|| error(format!("missing original wrapper generator {name}")))?;
    if matches.next().is_some() {
        return Err(error(format!("ambiguous original wrapper generator {name}")));
    }
    let tail = &source[start..];
    let end = tail.find("\n}\n").ok_or_else(|| error(format!("unterminated original wrapper generator {name}")))?;
    Ok(&tail[..end + 2])
}
fn constant<'a>(source: &'a str, name: &str) -> Result<&'a str, RunError> {
    let needle = format!("const {name}:");
    let mut matches = source.match_indices(&needle);
    let (start, _) = matches.next().ok_or_else(|| error(format!("missing original wrapper constant {name}")))?;
    if matches.next().is_some() {
        return Err(error(format!("ambiguous original wrapper constant {name}")));
    }
    let tail = &source[start..];
    let end = tail.find(";\n").ok_or_else(|| error(format!("unterminated original wrapper constant {name}")))?;
    Ok(&tail[..end + 1])
}
fn validate(request: &AuthoringRequest<'_>, name: &str) -> Result<(), RunError> {
    let root = request
        .source_roots
        .get(&SourceAuthorityRole::MantleSource)
        .ok_or_else(|| error("missing original wrapper source authority"))?;
    let admitted = std::fs::read_to_string(root.join("src/rust_source_provider.rs"))
        .map_err(|reason| error(reason.to_string()))?;
    let compiled = include_str!("rust_source_provider.rs");
    let body = function(compiled, name)?;
    if function(&admitted, name)? != body {
        return Err(error(format!("admitted original wrapper generator {name} differs from compiled source")));
    }
    let mut pending = vec![body];
    let mut seen = BTreeSet::new();
    while let Some(text) = pending.pop() {
        for token in text
            .split(|c: char| !c.is_ascii_alphanumeric() && c != '_')
            .filter(|token| token.starts_with("FIRST_STAGE_") || token.starts_with("RUSTC_SOURCE_"))
        {
            if !seen.insert(token) {
                continue;
            }
            let declaration = constant(compiled, token)?;
            if constant(&admitted, token)? != declaration {
                return Err(error(format!("admitted original wrapper constant {token} differs from compiled source")));
            }
            pending.push(declaration);
        }
    }
    Ok(())
}
pub(crate) fn original_target_cc_generator(request: &AuthoringRequest<'_>, first: bool) -> Result<String, RunError> {
    let name = if first {
        "push_first_stage_target_cc_wrapper"
    } else {
        "push_rustc_source_target_cc_wrapper"
    };
    validate(request, name)?;
    let mut script = String::new();
    if first {
        crate::rust_source_provider::push_first_stage_target_cc_wrapper(&mut script, true);
    } else {
        crate::rust_source_provider::push_rustc_source_target_cc_wrapper(&mut script);
    }
    Ok(script)
}

fn shell_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\"'\"'"))
}
fn write_script(script: &mut String, path: &str, body: &str) {
    script.push_str("printf '%s\\n'");
    for line in body.lines() {
        script.push(' ');
        script.push_str(&shell_quote(line));
    }
    script.push_str(" > ");
    script.push_str(&shell_quote(path));
    script.push_str("\nchmod +x ");
    script.push_str(&shell_quote(path));
    script.push('\n');
}

/// Run the unchanged original generator. The returned whole workspace contains
/// tmp/target-wrappers; callers project that subtree with their existing helper.
/// Embedded runtime paths bind the final package Tree, not the temporary writer.
#[allow(clippy::too_many_arguments)]
pub(crate) fn append_target_wrappers(
    request: &AuthoringRequest<'_>,
    recipe: &mut SourceRecipe,
    id: &str,
    package: ArtifactRef,
    controller: ArtifactRef,
    runtime_tree: ArtifactRef,
    first: bool,
    cc_path: &str,
    cxx_path: &str,
    initial_workspace: ArtifactRef,
) -> Result<ArtifactRef, RunError> {
    for path in [cc_path, cxx_path] {
        crate::source_built_recipe::relative(path, false)?;
    }
    let generator = original_target_cc_generator(request, first)?;
    // Original companion algorithm: select CXX through MANTLE_TARGET_CC_PATH,
    // export it, then exec the same CC wrapper with the untouched argument list.
    // Its existing source body and constants must be exactly the admitted ones.
    validate(
        request,
        if first {
            "push_first_stage_target_companion_wrappers"
        } else {
            "push_rustc_source_target_companion_wrappers"
        },
    )?;
    let world = super::super::llvm::native_command_world(request)?;
    let shell = world
        .runtime_mounts
        .iter()
        .find(|(_, name)| name.as_str() == "native-command-host-busybox")
        .map(|(root, _)| format!("{root}/bin/sh"))
        .ok_or_else(|| error("original full-source shell root missing"))?;
    let mut script = format!(
        "set -eu\nSHELL_PROGRAM={}\ntarget_alias_dir=/tmp/target-wrappers\ntarget_runtime_dir=/mantle-rust-runtime\ntarget_cc_path=/inputs/package/target-wrappers/sdk-cc\nmkdir -p \"$target_alias_dir\"\n",
        shell_quote(&shell)
    );
    for (name, path, response) in [
        ("sdk-cc", cc_path, "native-sdk-cc"),
        ("sdk-cxx", cxx_path, "native-sdk-cxx"),
    ] {
        let body = format!("#!/bin/sh\nexec \"/inputs/package/{path}\" @\"/inputs/package/{response}\" \"$@\"\n");
        write_script(&mut script, &format!("/tmp/target-wrappers/{name}"), &body);
    }
    script.push_str(&generator);
    write_script(
        &mut script,
        "/tmp/target-wrappers/c++",
        "#!/bin/sh\nset -eu\nMANTLE_TARGET_CC_PATH=\"/inputs/package/target-wrappers/sdk-cxx\"\nexport MANTLE_TARGET_CC_PATH\nexec \"/inputs/package/target-wrappers/cc\" \"$@\"\n",
    );
    validate(request, "provider_local_mrustc_rustc_wrapper")?;
    write_script(
        &mut script,
        "/tmp/target-wrappers/rustc",
        "#!/bin/sh\nset -eu\nhas_sysroot=false\nfor arg in \"$@\"; do\n  case \"$arg\" in --sysroot|--sysroot=*) has_sysroot=true ;; esac\ndone\nif [ \"$has_sysroot\" = false ]; then set -- --sysroot \"$MANTLE_CAIRN_SYSROOT\" \"$@\"; fi\nexec \"/inputs/package/rustc\" \"$@\"\n",
    );
    let mut inputs = world.inputs;
    inputs.insert("package".into(), package);
    inputs.insert("rust-runtime".into(), runtime_tree);
    let readonly_inputs = inputs.keys().cloned().collect();
    let mut runtime_mounts = world.runtime_mounts;
    runtime_mounts.insert("/mantle-rust-runtime".into(), "rust-runtime".into());
    let mut environment = world.environment;
    environment.insert("PATH".into(), world.path_prefix);
    let policy_root = request
        .source_roots
        .get(&SourceAuthorityRole::MantleSource)
        .ok_or_else(|| error("original wrapper reservation authority missing"))?;
    let policy = crate::source_built_program_authoring::read_continuation_policy(policy_root)?;
    let commands_max = crate::source_built_recipe::controller::reserve_native_commands(
        &policy,
        id,
        0,
        crate::source_built_recipe::STEP_COUNT_MAX,
    )?;
    super::super::llvm::append_native_source_script(
        request,
        recipe,
        controller,
        super::super::llvm::NativeSourceScript {
            id: id.into(),
            script,
            initial_workspace,
            initial_cwd: "/tmp".into(),
            initial_cwd_input: None,
            original_zero: "rust-source-provider-target-wrappers".into(),
            environment,
            environment_input: None,
            inputs,
            readonly_inputs,
            runtime_mounts,
            inventory: world.inventory,
            commands_max,
        },
    )
}
