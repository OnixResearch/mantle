//! crunch-eval: Nickel evaluation for crunch.
//!
//! Evaluates `.ncl` files using the stable `nickel-lang` API. Returns an
//! [`nickel_lang::Expr`] that can be deserialized directly into typed Rust
//! structs via `Expr::to_serde()` — no JSON round-trip.
//!
//! For debug output, `Expr` can also be exported to JSON via
//! `Context::expr_to_json()`.

use std::ffi::OsString;
use std::path::{Path, PathBuf};

pub use nickel_lang::{Context, Error as NickelError, Expr};

pub mod stdlib;

/// Errors from crunch-eval.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// nickel_lang::Error doesn't implement std::error::Error,
    /// so we wrap it manually rather than using #[from].
    #[error("Nickel evaluation error")]
    Eval(NickelError),

    #[error("reading source file: {0}")]
    Io(#[from] std::io::Error),

    #[error("deserialization error: {0}")]
    Serde(String),
}

impl From<NickelError> for Error {
    fn from(e: NickelError) -> Self {
        Error::Eval(e)
    }
}

/// Evaluate a `.ncl` file, returning the fully-reduced `Expr`.
///
/// The expression has all thunks forced, `not_exported` fields stripped,
/// and all contracts validated. It can be deserialized into typed Rust
/// structs via `expr.to_serde::<T>()`.
///
/// `import_paths` are additional directories to search for imports.
pub fn evaluate(path: &Path, import_paths: &[OsString]) -> Result<Expr, Error> {
    let source = std::fs::read_to_string(path)?;

    // Set up import resolution: the file's parent directory + any extra paths
    let mut paths: Vec<OsString> = Vec::new();
    if let Some(parent) = path.parent() {
        paths.push(parent.into());
    }
    paths.extend_from_slice(import_paths);

    let mut ctx = Context::new()
        .with_added_import_paths(paths)
        .with_source_name(path.display().to_string());

    let expr = ctx.eval_deep_for_export(&source)?;
    Ok(expr)
}

/// Evaluate a Nickel source string (not a file path).
///
/// Useful for tests and REPL-like usage.
pub fn evaluate_str(source: &str, import_paths: &[OsString]) -> Result<Expr, Error> {
    let mut ctx = Context::new()
        .with_added_import_paths(import_paths.to_vec())
        .with_source_name("<input>".to_string());

    let expr = ctx.eval_deep_for_export(source)?;
    Ok(expr)
}

/// Evaluate and deserialize into a typed Rust struct.
///
/// Goes through JSON export to handle Nickel enum tags (which become
/// strings in JSON but not through direct `to_serde()`). This is the
/// primary way the build pipeline consumes Nickel output.
pub fn evaluate_and_deserialize<T: serde::de::DeserializeOwned>(
    path: &Path,
    import_paths: &[OsString],
) -> Result<T, Error> {
    let json = evaluate_to_json(path, import_paths)?;
    serde_json::from_str(&json).map_err(|e| Error::Serde(e.to_string()))
}

/// Evaluate a string and deserialize into a typed Rust struct.
pub fn evaluate_str_and_deserialize<T: serde::de::DeserializeOwned>(
    source: &str,
    import_paths: &[OsString],
) -> Result<T, Error> {
    let json = evaluate_str_to_json(source, import_paths)?;
    serde_json::from_str(&json).map_err(|e| Error::Serde(e.to_string()))
}

/// Evaluate a Nickel source string and export to JSON.
pub fn evaluate_str_to_json(source: &str, import_paths: &[OsString]) -> Result<String, Error> {
    let mut ctx = Context::new()
        .with_added_import_paths(import_paths.to_vec())
        .with_source_name("<input>".to_string());

    let expr = ctx.eval_deep_for_export(source)?;
    let json = ctx.expr_to_json(&expr)?;
    Ok(json)
}

/// Evaluate and export to JSON string. For `crunch eval` debug output.
pub fn evaluate_to_json(path: &Path, import_paths: &[OsString]) -> Result<String, Error> {
    let source = std::fs::read_to_string(path)?;

    let mut paths: Vec<OsString> = Vec::new();
    if let Some(parent) = path.parent() {
        paths.push(parent.into());
    }
    paths.extend_from_slice(import_paths);

    let mut ctx = Context::new()
        .with_added_import_paths(paths)
        .with_source_name(path.display().to_string());

    let expr = ctx.eval_deep_for_export(&source)?;
    let json = ctx.expr_to_json(&expr)?;
    Ok(json)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn eval_simple_record() {
        let expr = evaluate_str("{ name = \"hello\", port = 8080 }", &[]).unwrap();
        assert!(expr.is_record());

        let record = expr.as_record().unwrap();
        assert_eq!(record.value_by_name("name").unwrap().as_str(), Some("hello"));
        assert_eq!(record.value_by_name("port").unwrap().as_i64(), Some(8080));
    }

    #[test]
    fn eval_not_exported_stripped() {
        let expr = evaluate_str(
            "{ visible = 1, hidden | not_exported = 2 }",
            &[],
        )
        .unwrap();

        let record = expr.as_record().unwrap();
        assert!(record.value_by_name("visible").is_some());
        assert!(record.value_by_name("hidden").is_none());
    }

    #[test]
    fn eval_contract_violation() {
        let result = evaluate_str("{ x | Number = \"not a number\" }", &[]);
        assert!(result.is_err());
    }

    #[test]
    fn eval_serde_deserialize() {
        #[derive(::serde::Deserialize, Debug, PartialEq)]
        struct Config {
            name: String,
            port: i64,
            debug: bool,
        }

        let expr = evaluate_str(
            "{ name = \"myapp\", port = 3000, debug = true }",
            &[],
        )
        .unwrap();

        let config: Config = expr.to_serde().unwrap();
        assert_eq!(config, Config {
            name: "myapp".to_string(),
            port: 3000,
            debug: true,
        });
    }

    #[test]
    fn eval_recursive_record() {
        let expr = evaluate_str(
            "{ name = \"app\", greeting = \"Hello, %{name}!\" }",
            &[],
        )
        .unwrap();

        let record = expr.as_record().unwrap();
        assert_eq!(
            record.value_by_name("greeting").unwrap().as_str(),
            Some("Hello, app!")
        );
    }

    #[test]
    fn eval_enum_tags() {
        let expr = evaluate_str("'Ok", &[]).unwrap();
        assert_eq!(expr.as_enum_tag(), Some("Ok"));
    }

    #[test]
    fn eval_merge() {
        let expr = evaluate_str(
            "{ port | default = 8080 } & { port = 3000 }",
            &[],
        )
        .unwrap();

        let record = expr.as_record().unwrap();
        assert_eq!(record.value_by_name("port").unwrap().as_i64(), Some(3000));
    }

    #[test]
    fn eval_defaults_applied() {
        let expr = evaluate_str(
            "{ port | default = 8080, name = \"svc\" }",
            &[],
        )
        .unwrap();

        let record = expr.as_record().unwrap();
        assert_eq!(record.value_by_name("port").unwrap().as_i64(), Some(8080));
    }
}
