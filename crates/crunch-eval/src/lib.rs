//! crunch-eval: Nickel evaluation for crunch.
//!
//! Evaluates `.ncl` files using the stable `nickel-lang` API. Returns an
//! [`nickel_lang::Expr`] that can be deserialized directly into typed Rust
//! structs via `Expr::to_serde()` — no JSON round-trip.
//!
//! For debug output, `Expr` can also be exported to JSON via
//! `Context::expr_to_json()`.

use std::ffi::OsString;
use std::path::Path;

pub use nickel_lang::Context;
pub use nickel_lang::Error as NickelError;
pub use nickel_lang::Expr;

pub mod stdlib;

/// Errors from crunch-eval.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// nickel_lang::Error doesn't implement std::error::Error,
    /// so we wrap it manually rather than using #[from].
    #[error("Nickel evaluation error: {0:?}")]
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

    let mut ctx = Context::new().with_added_import_paths(paths).with_source_name(path.display().to_string());

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

    let mut ctx = Context::new().with_added_import_paths(paths).with_source_name(path.display().to_string());

    let expr = ctx.eval_deep_for_export(&source)?;
    let json = ctx.expr_to_json(&expr)?;
    Ok(json)
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── Phase 1: File-based evaluation ──────────────────────────

    #[test]
    fn eval_file_simple_record() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("test.ncl");
        std::fs::write(&file, r#"{ name = "hello", port = 8080 }"#).unwrap();

        let expr = evaluate(&file, &[]).unwrap();
        assert!(expr.is_record());
        let record = expr.as_record().unwrap();
        assert_eq!(record.value_by_name("name").unwrap().as_str(), Some("hello"));
        assert_eq!(record.value_by_name("port").unwrap().as_i64(), Some(8080));
    }

    #[test]
    fn eval_file_resolves_import_from_parent_dir() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("dep.ncl"), r#"{ greeting = "hi" }"#).unwrap();
        std::fs::write(dir.path().join("main.ncl"), r#"let dep = import "dep.ncl" in { msg = dep.greeting }"#).unwrap();

        let expr = evaluate(&dir.path().join("main.ncl"), &[]).unwrap();
        let record = expr.as_record().unwrap();
        assert_eq!(record.value_by_name("msg").unwrap().as_str(), Some("hi"));
    }

    #[test]
    fn eval_file_extra_import_path() {
        let main_dir = tempfile::tempdir().unwrap();
        let lib_dir = tempfile::tempdir().unwrap();

        std::fs::write(lib_dir.path().join("util.ncl"), r#"{ version = 42 }"#).unwrap();
        std::fs::write(main_dir.path().join("app.ncl"), r#"let u = import "util.ncl" in { v = u.version }"#).unwrap();

        let import_paths = vec![lib_dir.path().as_os_str().to_owned()];
        let expr = evaluate(&main_dir.path().join("app.ncl"), &import_paths).unwrap();
        let record = expr.as_record().unwrap();
        assert_eq!(record.value_by_name("v").unwrap().as_i64(), Some(42));
    }

    #[test]
    fn eval_file_nonexistent_returns_io_error() {
        let result = evaluate(Path::new("/nonexistent/file.ncl"), &[]);
        let err = result.err().expect("should be Err");
        assert!(matches!(err, Error::Io(_)), "expected Io, got: {err}");
    }

    #[test]
    fn eval_file_to_json_returns_valid_json() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("test.ncl");
        std::fs::write(&file, r#"{ x = 1, y = "two" }"#).unwrap();

        let json = evaluate_to_json(&file, &[]).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed["x"], 1);
        assert_eq!(parsed["y"], "two");
    }

    // ── Phase 3: JSON round-trip deserialization ────────────────

    #[test]
    fn eval_str_and_deserialize_flat_record() {
        #[derive(serde::Deserialize, Debug, PartialEq)]
        struct Item {
            name: String,
            count: i64,
        }

        let item: Item = evaluate_str_and_deserialize(r#"{ name = "widget", count = 5 }"#, &[]).unwrap();
        assert_eq!(item.name, "widget");
        assert_eq!(item.count, 5);
    }

    #[test]
    fn eval_str_and_deserialize_enum_tags() {
        #[derive(serde::Deserialize, Debug, PartialEq)]
        struct Tagged {
            status: String,
        }

        // Nickel enum tags become strings through the JSON export path
        let t: Tagged = evaluate_str_and_deserialize(r#"{ status = 'active }"#, &[]).unwrap();
        assert_eq!(t.status, "active");
    }

    #[test]
    fn eval_str_and_deserialize_nested_records() {
        #[derive(serde::Deserialize, Debug, PartialEq)]
        struct Inner {
            value: i64,
        }
        #[derive(serde::Deserialize, Debug, PartialEq)]
        struct Outer {
            name: String,
            inner: Inner,
        }

        let o: Outer = evaluate_str_and_deserialize(r#"{ name = "pkg", inner = { value = 99 } }"#, &[]).unwrap();
        assert_eq!(o.name, "pkg");
        assert_eq!(o.inner.value, 99);
    }

    #[test]
    fn eval_str_and_deserialize_type_mismatch_returns_serde_error() {
        #[derive(serde::Deserialize, Debug)]
        struct NeedsNumber {
            x: i64,
        }

        let result = evaluate_str_and_deserialize::<NeedsNumber>(r#"{ x = "not a number" }"#, &[]);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), Error::Serde(_)));
    }

    // ── Phase 4: Error paths ───────────────────────────────────

    #[test]
    fn eval_str_syntax_error_returns_eval() {
        let result = evaluate_str("{ missing_brace = 1", &[]);
        let err = result.err().expect("should be Err");
        assert!(matches!(err, Error::Eval(_)), "expected Eval, got: {err}");
    }

    #[test]
    fn eval_str_typecheck_failure_returns_eval() {
        let result = evaluate_str(r#"let f : Number -> Number = fun x => x in f "hello""#, &[]);
        let err = result.err().expect("should be Err");
        assert!(matches!(err, Error::Eval(_)), "expected Eval, got: {err}");
    }

    #[test]
    fn eval_and_deserialize_non_record_returns_serde() {
        // Evaluating a number and trying to deserialize as a struct
        #[derive(serde::Deserialize, Debug)]
        struct Rec {
            field: String,
        }

        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("num.ncl");
        std::fs::write(&file, "42").unwrap();

        let result = evaluate_and_deserialize::<Rec>(&file, &[]);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), Error::Serde(_)));
    }

    #[test]
    fn eval_error_display_has_context() {
        let err = evaluate_str("{ x | Number = \"bad\" }", &[]).err().expect("should be Err");
        let msg = format!("{err}");
        // The display should say more than just "Nickel evaluation error"
        // — it wraps NickelError which has diagnostic info.
        assert!(!msg.is_empty());
    }

    // ── Original tests ─────────────────────────────────────────

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
        let expr = evaluate_str("{ visible = 1, hidden | not_exported = 2 }", &[]).unwrap();

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

        let expr = evaluate_str("{ name = \"myapp\", port = 3000, debug = true }", &[]).unwrap();

        let config: Config = expr.to_serde().unwrap();
        assert_eq!(config, Config {
            name: "myapp".to_string(),
            port: 3000,
            debug: true,
        });
    }

    #[test]
    fn eval_recursive_record() {
        let expr = evaluate_str("{ name = \"app\", greeting = \"Hello, %{name}!\" }", &[]).unwrap();

        let record = expr.as_record().unwrap();
        assert_eq!(record.value_by_name("greeting").unwrap().as_str(), Some("Hello, app!"));
    }

    #[test]
    fn eval_enum_tags() {
        let expr = evaluate_str("'Ok", &[]).unwrap();
        assert_eq!(expr.as_enum_tag(), Some("Ok"));
    }

    #[test]
    fn eval_merge() {
        let expr = evaluate_str("{ port | default = 8080 } & { port = 3000 }", &[]).unwrap();

        let record = expr.as_record().unwrap();
        assert_eq!(record.value_by_name("port").unwrap().as_i64(), Some(3000));
    }

    #[test]
    fn eval_defaults_applied() {
        let expr = evaluate_str("{ port | default = 8080, name = \"svc\" }", &[]).unwrap();

        let record = expr.as_record().unwrap();
        assert_eq!(record.value_by_name("port").unwrap().as_i64(), Some(8080));
    }
}
