use std::collections::BTreeMap;
use std::path::Path;
use std::path::PathBuf;

use serde_json::Value;

use crate::ValidatedModule;
use crate::error::SystemConfigError;
use crate::eval_trait::EvalOptions;
use crate::threading::EvalThreadHandle;
use crate::threading::ValueId;

const DEFAULT_MODULE_LIMIT: usize = 1024;
const DEFAULT_PRIORITY: i64 = 1000;

pub fn discover_module_files(dir: &Path) -> Result<Vec<(String, PathBuf)>, SystemConfigError> {
    discover_module_files_with_limit(dir, DEFAULT_MODULE_LIMIT)
}

pub fn discover_module_files_with_limit(
    dir: &Path,
    module_limit: usize,
) -> Result<Vec<(String, PathBuf)>, SystemConfigError> {
    assert!(module_limit > 0, "module limit must be positive");
    if !dir.exists() {
        return Err(loader_error(
            format!("module directory does not exist: {}", dir.display()),
            None,
            None,
        ));
    }
    if !dir.is_dir() {
        return Err(loader_error(
            format!("module path is not a directory: {}", dir.display()),
            None,
            None,
        ));
    }

    let entries = std::fs::read_dir(dir).map_err(|err| {
        loader_error(
            format!("reading module directory {}", dir.display()),
            None,
            Some(err.to_string()),
        )
    })?;
    let mut discovered = Vec::new();
    let mut stem_paths = BTreeMap::<String, PathBuf>::new();

    for entry_result in entries {
        let entry = entry_result.map_err(|err| {
            loader_error(
                format!("reading module directory entry in {}", dir.display()),
                None,
                Some(err.to_string()),
            )
        })?;
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        if path.extension().and_then(|value| value.to_str()) != Some("ncl") {
            continue;
        }
        if discovered.len() >= module_limit {
            return Err(loader_error(
                format!("module count exceeds configured limit {module_limit}"),
                None,
                None,
            ));
        }
        let stem = path
            .file_stem()
            .and_then(|value| value.to_str())
            .ok_or_else(|| loader_error(format!("module file has invalid utf-8 stem: {}", path.display()), None, None))?
            .to_string();
        if let Some(existing_path) = stem_paths.get(&stem) {
            return Err(loader_error(
                format!("duplicate module stem '{stem}'"),
                Some(stem.clone()),
                Some(format!("{} and {}", existing_path.display(), path.display())),
            ));
        }
        stem_paths.insert(stem.clone(), path.clone());
        discovered.push((stem, path));
    }

    discovered.sort_by(|left, right| left.0.cmp(&right.0));
    Ok(discovered)
}

pub async fn validate_module(
    name: &str,
    value_id: ValueId,
    handle: &EvalThreadHandle,
) -> Result<ValidatedModule, SystemConfigError> {
    let options = EvalOptions {
        timeout: None,
        import_paths: Vec::new(),
    };
    let interface_id = handle
        .get_field(value_id, "interface".to_string(), options.clone())
        .await
        .map_err(|err| loader_error(format!("module '{name}' is missing interface"), Some(name.to_string()), Some(err.to_string())))?;
    let roles_id = handle
        .get_field(interface_id, "roles".to_string(), options.clone())
        .await
        .map_err(|err| loader_error(format!("module '{name}' is missing interface.roles"), Some(name.to_string()), Some(err.to_string())))?;
    let roles_json = handle
        .to_json(roles_id, options.clone())
        .await
        .map_err(|err| loader_error(format!("module '{name}' has invalid interface.roles"), Some(name.to_string()), Some(err.to_string())))?;
    let impl_id = handle
        .get_field(value_id, "impl".to_string(), options.clone())
        .await
        .map_err(|err| loader_error(format!("module '{name}' is missing impl"), Some(name.to_string()), Some(err.to_string())))?;
    let impl_is_function = handle
        .is_function(impl_id, options.clone())
        .await
        .map_err(|err| loader_error(format!("module '{name}' impl validation failed"), Some(name.to_string()), Some(err.to_string())))?;
    if !impl_is_function {
        return Err(loader_error(
            format!("module '{name}' impl is not a function"),
            Some(name.to_string()),
            None,
        ));
    }
    let _ = options;

    Ok(ValidatedModule {
        module_name: name.to_string(),
        role_names: extract_role_names(name, &roles_json)?,
        inputs: extract_optional_string_array(handle, value_id, "inputs", name).await?,
        consumes_providers: extract_optional_string_array(handle, value_id, "consumes_providers", name).await?,
        produces_providers: extract_optional_string_array(handle, value_id, "produces_providers", name).await?,
        priority: extract_optional_priority(handle, value_id, name).await?,
    })
}

fn extract_role_names(name: &str, roles_json: &Value) -> Result<Vec<String>, SystemConfigError> {
    let Value::Object(map) = roles_json else {
        return Err(loader_error(
            format!("module '{name}' interface.roles must be a record"),
            Some(name.to_string()),
            None,
        ));
    };
    let mut role_names: Vec<String> = map.keys().cloned().collect();
    role_names.sort();
    Ok(role_names)
}

async fn extract_optional_string_array(
    handle: &EvalThreadHandle,
    value_id: ValueId,
    field_name: &str,
    module_name: &str,
) -> Result<Vec<String>, SystemConfigError> {
    let options = EvalOptions {
        timeout: None,
        import_paths: Vec::new(),
    };
    let field_id = match handle.get_field(value_id, field_name.to_string(), options.clone()).await {
        Ok(field_id) => field_id,
        Err(_) => return Ok(Vec::new()),
    };
    let value = handle.to_json(field_id, options).await.map_err(|err| {
        loader_error(
            format!("module '{module_name}' field '{field_name}' serialization failed"),
            Some(module_name.to_string()),
            Some(err.to_string()),
        )
    })?;
    let Value::Array(items) = value else {
        return Err(loader_error(
            format!("module field '{field_name}' must be an array"),
            None,
            None,
        ));
    };
    let mut values = Vec::with_capacity(items.len());
    for item in items {
        let Some(string_value) = item.as_str() else {
            return Err(loader_error(
                format!("module field '{field_name}' must contain only strings"),
                None,
                None,
            ));
        };
        values.push(string_value.to_string());
    }
    Ok(values)
}

async fn extract_optional_priority(
    handle: &EvalThreadHandle,
    value_id: ValueId,
    module_name: &str,
) -> Result<i64, SystemConfigError> {
    let options = EvalOptions {
        timeout: None,
        import_paths: Vec::new(),
    };
    let field_id = match handle.get_field(value_id, "priority".to_string(), options.clone()).await {
        Ok(field_id) => field_id,
        Err(_) => return Ok(DEFAULT_PRIORITY),
    };
    let value = handle.to_json(field_id, options).await.map_err(|err| {
        loader_error(
            format!("module '{module_name}' priority serialization failed"),
            Some(module_name.to_string()),
            Some(err.to_string()),
        )
    })?;
    if let Some(priority) = value.as_i64() {
        return Ok(priority);
    }
    if let Some(priority) = value.as_f64()
        && priority.fract() == 0.0
    {
        return Ok(priority as i64);
    }
    Err(loader_error("module field 'priority' must be an integer".to_string(), None, None))
}

fn loader_error(message: String, module_name: Option<String>, detail: Option<String>) -> SystemConfigError {
    SystemConfigError::Loader {
        message,
        detail,
        machine_name: None,
        module_name,
        field_path: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::threading::EvalThread;
    use std::time::Duration;

    fn eval_options() -> EvalOptions {
        EvalOptions {
            timeout: Some(Duration::from_secs(5)),
            import_paths: Vec::new(),
        }
    }

    fn import_paths() -> Vec<PathBuf> {
        vec![crunch_eval::stdlib::stdlib_import_path().unwrap()]
    }

    async fn write_file(dir: &Path, name: &str, body: &str) -> PathBuf {
        let path = dir.join(name);
        tokio::fs::write(&path, body).await.unwrap();
        path
    }

    #[tokio::test(flavor = "current_thread")]
    async fn discover_three_ncl_files_and_verify_stems() {
        let dir = tempfile::tempdir().unwrap();
        write_file(dir.path(), "a.ncl", "{} ").await;
        write_file(dir.path(), "b.ncl", "{} ").await;
        write_file(dir.path(), "c.ncl", "{} ").await;

        let discovered = discover_module_files(dir.path()).unwrap();
        let stems: Vec<String> = discovered.into_iter().map(|item| item.0).collect();

        assert_eq!(stems, vec!["a", "b", "c"]);
    }

    #[tokio::test(flavor = "current_thread")]
    async fn discover_ignores_subdirectories() {
        let dir = tempfile::tempdir().unwrap();
        let nested = dir.path().join("nested");
        tokio::fs::create_dir_all(&nested).await.unwrap();
        write_file(dir.path(), "root.ncl", "{} ").await;
        write_file(&nested, "child.ncl", "{} ").await;

        let discovered = discover_module_files(dir.path()).unwrap();

        assert_eq!(discovered.len(), 1);
        assert_eq!(discovered[0].0, "root");
    }

    #[tokio::test(flavor = "current_thread")]
    async fn duplicate_stem_detection_reports_loader_error() {
        let dir = tempfile::tempdir().unwrap();
        write_file(dir.path(), "dup.ncl", "{} ").await;
        write_file(dir.path(), "other.ncl", "{} ").await;

        let result = discover_module_files_with_limit(dir.path(), 1);

        assert!(matches!(result, Err(SystemConfigError::Loader { message, .. }) if message.contains("limit 1")));
    }

    #[tokio::test(flavor = "current_thread")]
    async fn module_count_limit_exceeded() {
        let dir = tempfile::tempdir().unwrap();
        write_file(dir.path(), "one.ncl", "{} ").await;
        write_file(dir.path(), "two.ncl", "{} ").await;

        let result = discover_module_files_with_limit(dir.path(), 1);

        assert!(matches!(result, Err(SystemConfigError::Loader { message, .. }) if message.contains("limit 1")));
    }

    #[tokio::test(flavor = "current_thread")]
    async fn validate_module_valid_module_returns_validated_module() {
        let dir = tempfile::tempdir().unwrap();
        let path = write_file(
            dir.path(),
            "valid.ncl",
            r#"{
              interface = { roles = { server = { port = 22 }, client = { port = 80 } } },
              impl = fun args => { output = { exports = { port = args.settings.port } } },
              inputs = ["a"],
              consumes_providers = ["firewall"],
              produces_providers = ["ssh"],
              priority = 7,
            }"#,
        )
        .await;
        let handle = EvalThread::spawn(import_paths());
        let value_id = handle.evaluate_file(path, eval_options()).await.unwrap();

        let validated = validate_module("valid", value_id, &handle).await.unwrap();

        assert_eq!(validated.module_name, "valid");
        assert_eq!(validated.role_names, vec!["client", "server"]);
        assert_eq!(validated.inputs, vec!["a"]);
        assert_eq!(validated.consumes_providers, vec!["firewall"]);
        assert_eq!(validated.produces_providers, vec!["ssh"]);
        assert_eq!(validated.priority, 7);
        handle.shutdown().await.unwrap();
    }

    #[tokio::test(flavor = "current_thread")]
    async fn validate_module_missing_interface_returns_loader_error() {
        let dir = tempfile::tempdir().unwrap();
        let path = write_file(dir.path(), "missing_interface.ncl", "{ impl = fun _args => { output = {} } }").await;
        let handle = EvalThread::spawn(import_paths());
        let value_id = handle.evaluate_file(path, eval_options()).await.unwrap();

        let result = validate_module("missing_interface", value_id, &handle).await;

        assert!(matches!(result, Err(SystemConfigError::Loader { message, .. }) if message.contains("missing interface")));
        handle.shutdown().await.unwrap();
    }

    #[tokio::test(flavor = "current_thread")]
    async fn validate_module_missing_impl_returns_loader_error() {
        let dir = tempfile::tempdir().unwrap();
        let path = write_file(dir.path(), "missing_impl.ncl", "{ interface = { roles = { server = {} } } }").await;
        let handle = EvalThread::spawn(import_paths());
        let value_id = handle.evaluate_file(path, eval_options()).await.unwrap();

        let result = validate_module("missing_impl", value_id, &handle).await;

        assert!(matches!(result, Err(SystemConfigError::Loader { message, .. }) if message.contains("missing impl")));
        handle.shutdown().await.unwrap();
    }

    #[tokio::test(flavor = "current_thread")]
    async fn validate_module_impl_not_function_returns_loader_error() {
        let dir = tempfile::tempdir().unwrap();
        let path = write_file(
            dir.path(),
            "bad_impl.ncl",
            "{ interface = { roles = { server = {} } }, impl = { nope = true } }",
        )
        .await;
        let handle = EvalThread::spawn(import_paths());
        let value_id = handle.evaluate_file(path, eval_options()).await.unwrap();

        let result = validate_module("bad_impl", value_id, &handle).await;

        assert!(matches!(result, Err(SystemConfigError::Loader { message, .. }) if message.contains("not a function")));
        handle.shutdown().await.unwrap();
    }
}
