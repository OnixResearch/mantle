const STANDARD_LIBRARY_MODULE_COUNT_MAX: usize = 128;
const ARGUMENTS_PER_MODULE: usize = 2;

pub(super) fn expand(
    radiance_source: &std::path::Path,
    profile: &crate::radiance::profile::Definition,
) -> Result<Vec<String>, crate::errors::RunError> {
    let modules = std::fs::read_to_string(radiance_source.join("std.lib"))
        .map_err(|error| crate::errors::RunError::Internal(format!("reading Radiance std.lib: {error}")))?
        .lines()
        .filter(|line| !line.trim().is_empty())
        .map(|line| line.trim().to_string())
        .collect::<Vec<_>>();
    if modules.is_empty() || modules.len() > STANDARD_LIBRARY_MODULE_COUNT_MAX {
        return Err(crate::errors::RunError::Internal(
            "Radiance std.lib module count is outside its bound".to_string(),
        ));
    }
    let module_argument_count_max = modules
        .len()
        .checked_mul(ARGUMENTS_PER_MODULE)
        .ok_or_else(|| crate::errors::RunError::Internal("Radiance module argument count overflow".to_string()))?;
    let argument_count_max =
        profile.compiler_arguments.len().checked_add(module_argument_count_max).ok_or_else(|| {
            crate::errors::RunError::Internal("Radiance compiler argument count overflow".to_string())
        })?;
    let mut arguments = Vec::with_capacity(argument_count_max);
    for argument in &profile.compiler_arguments {
        if argument == "@std.lib" {
            for module in &modules {
                validate_module(module)?;
                arguments.push("-mod".to_string());
                arguments.push(module.clone());
            }
        } else {
            arguments.push(argument.clone());
        }
    }
    debug_assert!(arguments.len() > profile.compiler_arguments.len());
    debug_assert!(arguments.iter().all(|argument| argument != "@std.lib"));
    Ok(arguments)
}

pub(super) fn validate_module(module: &str) -> Result<(), crate::errors::RunError> {
    let path = std::path::Path::new(module);
    if module.is_empty()
        || path.is_absolute()
        || !path.components().all(|component| matches!(component, std::path::Component::Normal(_)))
    {
        return Err(crate::errors::RunError::Internal(format!(
            "unsafe Radiance standard-library module path: {module}"
        )));
    }
    debug_assert!(!module.is_empty());
    debug_assert!(!path.is_absolute());
    Ok(())
}
