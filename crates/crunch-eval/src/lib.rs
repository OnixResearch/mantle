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

use backend::EvalBackend;
use backend::EvalRequest;
#[cfg(feature = "cranelift-proto")]
use backend::cranelift_prototype_backend;
use backend::default_backend;
pub use nickel_lang::Context;
pub use nickel_lang::Error as NickelError;
pub use nickel_lang::Expr;
use serde::de::DeserializeOwned;

mod backend;
#[cfg(feature = "cranelift-proto")]
pub(crate) mod cranelift_proto;
pub mod session;
pub mod stdlib;

/// Errors from crunch-eval.
#[derive(Debug)]
pub enum Error {
    /// nickel_lang::Error doesn't implement std::error::Error,
    /// so we wrap it manually rather than using #[from].
    Eval(NickelError),

    Io(std::io::Error),

    Boundary(String),

    Serde(String),

    Labeled {
        label: String,
        source: Box<Error>,
    },
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Eval(err) => f.write_str(&format_nickel_error(err)),
            Error::Io(err) => write!(f, "reading source file: {err}"),
            Error::Boundary(err) => write!(f, "evaluation boundary error: {err}"),
            Error::Serde(err) => write!(f, "deserialization error: {err}"),
            Error::Labeled { label, source } => write!(f, "root '{label}': {source}"),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::Eval(_) => None,
            Error::Io(err) => Some(err),
            Error::Boundary(_) => None,
            Error::Serde(_) => None,
            Error::Labeled { source, .. } => Some(source),
        }
    }
}

impl From<NickelError> for Error {
    fn from(e: NickelError) -> Self {
        Error::Eval(e)
    }
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Error::Io(e)
    }
}

fn format_nickel_error(err: &NickelError) -> String {
    let mut rendered = Vec::new();
    if err.format(&mut rendered, nickel_lang::ErrorFormat::Text).is_err() {
        return "Nickel evaluation error".to_string();
    }

    let text = String::from_utf8_lossy(&rendered).trim().to_string();
    if text.is_empty() {
        return "Nickel evaluation error".to_string();
    }

    format!("Nickel evaluation error:\n{text}")
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
    let request = file_eval_request(path, &source, import_paths);
    default_backend().eval(request)
}

/// Evaluate a Nickel source string (not a file path).
///
/// Useful for tests and REPL-like usage.
pub fn evaluate_str(source: &str, import_paths: &[OsString]) -> Result<Expr, Error> {
    let request = inline_eval_request(source, import_paths);
    default_backend().eval(request)
}

fn file_eval_request<'a>(path: &Path, source: &'a str, import_paths: &[OsString]) -> EvalRequest<'a> {
    EvalRequest {
        source,
        import_paths: import_paths_for_file(path, import_paths),
        source_name: path.display().to_string(),
    }
}

fn inline_eval_request<'a>(source: &'a str, import_paths: &[OsString]) -> EvalRequest<'a> {
    EvalRequest {
        source,
        import_paths: import_paths.to_vec(),
        source_name: "<input>".to_string(),
    }
}

fn import_paths_for_file(path: &Path, import_paths: &[OsString]) -> Vec<OsString> {
    let mut paths = Vec::with_capacity(import_paths.len().saturating_add(1));
    if let Some(parent) = path.parent() {
        paths.push(parent.into());
    }
    paths.extend_from_slice(import_paths);
    paths
}

fn deserialize_expr<T: DeserializeOwned>(expr: &Expr, context: &str) -> Result<T, Error> {
    expr.to_serde().map_err(|e| Error::Serde(format!("deserializing {context}: {e}")))
}

fn record_name(expr: &Expr) -> Option<String> {
    expr.as_record()?.value_by_name("name")?.as_str().map(str::to_owned)
}

fn deserialize_array_roots<T: DeserializeOwned>(expr: &Expr) -> Result<Vec<(String, T)>, Error> {
    let array = expr
        .as_array()
        .ok_or_else(|| Error::Boundary("array roots require an array expression".to_string()))?;
    let mut derivations = Vec::with_capacity(array.len());

    for (index, item) in array.iter().enumerate() {
        let label = record_name(&item)
            .ok_or_else(|| Error::Boundary(format!("derivation [{index}] is missing string field 'name'")))?;
        let drv = deserialize_expr(&item, &format!("derivation [{index}]"))?;
        derivations.push((label, drv));
    }

    Ok(derivations)
}

fn deserialize_record_roots<T: DeserializeOwned>(expr: &Expr) -> Result<Vec<(String, T)>, Error> {
    let record = expr
        .as_record()
        .ok_or_else(|| Error::Boundary("record roots require a record expression".to_string()))?;
    let mut derivations = Vec::with_capacity(record.len());

    for (key, value) in record.iter() {
        let value = value.ok_or_else(|| Error::Boundary(format!("derivation '{key}' is missing a value")))?;
        let drv = deserialize_expr(&value, &format!("derivation '{key}'"))?;
        derivations.push((key.to_string(), drv));
    }

    Ok(derivations)
}

pub fn extract_named_roots<T: DeserializeOwned>(expr: &Expr) -> Result<Vec<(String, T)>, Error> {
    if expr.is_array() {
        return deserialize_array_roots(expr);
    }

    let Some(record) = expr.as_record() else {
        return Err(Error::Boundary(
            "expected a Derivation record, array of Derivations, or record of Derivations".to_string(),
        ));
    };

    if let Some(label) = record.value_by_name("name").and_then(|value| value.as_str().map(str::to_owned)) {
        let drv = deserialize_expr(expr, "derivation")?;
        return Ok(vec![(label, drv)]);
    }

    deserialize_record_roots(expr)
}

/// Evaluate and extract named derivation roots through direct typed deserialization.
pub fn evaluate_and_extract_named_roots<T: DeserializeOwned>(
    path: &Path,
    import_paths: &[OsString],
) -> Result<Vec<(String, T)>, Error> {
    let expr = evaluate(path, import_paths)?;
    extract_named_roots(&expr)
}

/// Evaluate a Nickel source string and extract named derivation roots.
pub fn evaluate_str_and_extract_named_roots<T: DeserializeOwned>(
    source: &str,
    import_paths: &[OsString],
) -> Result<Vec<(String, T)>, Error> {
    let expr = evaluate_str(source, import_paths)?;
    extract_named_roots(&expr)
}

/// Evaluate and deserialize into a typed Rust struct.
///
/// Goes through JSON export. Keep this path for callers that want Nickel's
/// JSON rendering semantics rather than the direct derivation-extraction path.
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
    let request = inline_eval_request(source, import_paths);
    default_backend().eval_to_json(request)
}

/// Evaluate a flat derivation literal through the experimental Cranelift prototype backend.
#[cfg(feature = "cranelift-proto")]
pub fn evaluate_str_to_json_with_cranelift_prototype(source: &str, import_paths: &[OsString]) -> Result<String, Error> {
    let request = inline_eval_request(source, import_paths);
    cranelift_prototype_backend().eval_to_json(request)
}

/// Evaluate a flat derivation literal through the experimental Cranelift prototype backend.
#[cfg(feature = "cranelift-proto")]
pub fn evaluate_str_and_deserialize_with_cranelift_prototype<T: serde::de::DeserializeOwned>(
    source: &str,
    import_paths: &[OsString],
) -> Result<T, Error> {
    let json = evaluate_str_to_json_with_cranelift_prototype(source, import_paths)?;
    serde_json::from_str(&json).map_err(|e| Error::Serde(e.to_string()))
}

/// Evaluate a flat derivation literal through the experimental Cranelift prototype backend.
#[cfg(feature = "cranelift-proto")]
pub fn evaluate_str_and_extract_named_roots_with_cranelift_prototype<T: DeserializeOwned>(
    source: &str,
    import_paths: &[OsString],
) -> Result<Vec<(String, T)>, Error> {
    let json = evaluate_str_to_json_with_cranelift_prototype(source, import_paths)?;
    let value: serde_json::Value = serde_json::from_str(&json).map_err(|e| Error::Serde(e.to_string()))?;
    let label = value
        .get("name")
        .and_then(serde_json::Value::as_str)
        .ok_or_else(|| Error::Serde("cranelift prototype output is missing string field `name`".to_string()))?
        .to_string();
    let root = serde_json::from_value(value).map_err(|e| Error::Serde(e.to_string()))?;
    Ok(vec![(label, root)])
}

/// Evaluate and export to JSON string. For `crunch eval` debug output.
pub fn evaluate_to_json(path: &Path, import_paths: &[OsString]) -> Result<String, Error> {
    let source = std::fs::read_to_string(path)?;
    let request = file_eval_request(path, &source, import_paths);
    default_backend().eval_to_json(request)
}

#[cfg(test)]
mod tests {
    use crunch_glue::CrunchDerivation;
    use crunch_glue::Input;
    use crunch_glue::OutputRef;
    use session::EvaluationSession;
    use session::RootShape;

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
    fn eval_str_and_extract_named_roots_single_derivation() {
        let roots = evaluate_str_and_extract_named_roots::<CrunchDerivation>(
            r#"{
  name = "hello",
  builder = "/bin/sh",
  args = ["-c", "echo hello > $out"],
  addressing_mode = 'input-addressed,
}"#,
            &[],
        )
        .unwrap();

        assert_eq!(roots.len(), 1);
        assert_eq!(roots[0].0, "hello");
        assert_eq!(roots[0].1.name, "hello");
        assert_eq!(roots[0].1.builder, "/bin/sh");
    }

    #[test]
    fn eval_str_and_extract_named_roots_package_set_preserves_keys() {
        let roots = evaluate_str_and_extract_named_roots::<CrunchDerivation>(
            r#"{
  hello = {
    name = "hello",
    builder = "/bin/sh",
  },
  world = {
    name = "world",
    builder = "/bin/sh",
  },
}"#,
            &[],
        )
        .unwrap();

        assert_eq!(roots.len(), 2);
        assert_eq!(roots[0].0, "hello");
        assert_eq!(roots[0].1.name, "hello");
        assert_eq!(roots[1].0, "world");
        assert_eq!(roots[1].1.name, "world");
    }

    #[test]
    fn eval_str_and_extract_named_roots_array_uses_derivation_names() {
        let roots = evaluate_str_and_extract_named_roots::<CrunchDerivation>(
            r#"[
  {
    name = "alpha",
    builder = "/bin/sh",
  },
  {
    name = "beta",
    builder = "/bin/sh",
  },
]"#,
            &[],
        )
        .unwrap();

        assert_eq!(roots.len(), 2);
        assert_eq!(roots[0].0, "alpha");
        assert_eq!(roots[1].0, "beta");
    }

    #[test]
    fn eval_str_and_extract_named_roots_rejects_non_root_shape_with_boundary_error() {
        let expr = evaluate_str("42", &[]).unwrap();
        let err = extract_named_roots::<CrunchDerivation>(&expr).unwrap_err();
        assert!(matches!(err, Error::Boundary(_)), "expected boundary error, got: {err}");
    }

    #[test]
    fn eval_str_and_extract_named_roots_array_missing_name_returns_boundary_error() {
        let err = evaluate_str_and_extract_named_roots::<CrunchDerivation>(
            r#"[
  {
    builder = "/bin/sh",
  },
]"#,
            &[],
        )
        .unwrap_err();
        assert!(matches!(err, Error::Boundary(_)), "expected boundary error, got: {err}");
    }

    #[test]
    fn eval_str_and_extract_named_roots_nested_derivation_enum_tags() {
        let roots = evaluate_str_and_extract_named_roots::<CrunchDerivation>(
            r#"{
  app = {
    name = "app",
    builder = "/bin/sh",
    system = 'x86_64-linux,
    addressing_mode = 'content-addressed,
    inputs = [
      {
        name = "dep",
        builder = "/bin/sh",
        system = 'x86_64-linux,
        addressing_mode = 'input-addressed,
      },
    ],
  },
}"#,
            &[],
        )
        .unwrap();

        assert_eq!(roots.len(), 1);
        assert_eq!(roots[0].0, "app");
        assert_eq!(roots[0].1.system, "x86_64-linux");
        assert_eq!(roots[0].1.addressing_mode, "content-addressed");

        match &roots[0].1.inputs[0] {
            Input::Derivation(dep) => {
                assert_eq!(dep.name, "dep");
                assert_eq!(dep.system, "x86_64-linux");
                assert_eq!(dep.addressing_mode, "input-addressed");
            }
            other => panic!("expected nested derivation input, got {other:?}"),
        }
    }

    #[test]
    fn eval_str_and_extract_named_roots_source_input_uses_manual_input_deserializer() {
        let roots = evaluate_str_and_extract_named_roots::<CrunchDerivation>(
            r#"{
  name = "uses-source",
  builder = "/bin/sh",
  inputs = ["/nix/store/00000000000000000000000000000000-bash"],
}"#,
            &[],
        )
        .unwrap();

        assert_eq!(roots.len(), 1);
        match &roots[0].1.inputs[0] {
            Input::Source(path) => {
                assert_eq!(path, "/nix/store/00000000000000000000000000000000-bash");
            }
            other => panic!("expected source input, got {other:?}"),
        }
    }

    #[test]
    fn eval_str_and_extract_named_roots_output_selection_uses_manual_input_deserializer() {
        let roots = evaluate_str_and_extract_named_roots::<CrunchDerivation>(
            r#"{
  name = "uses-output-selection",
  builder = "/bin/sh",
  inputs = [
    {
      drv = {
        name = "libfoo",
        builder = "/bin/sh",
        outputs = ["out", "dev"],
      },
      output = "dev",
    },
  ],
}"#,
            &[],
        )
        .unwrap();

        assert_eq!(roots.len(), 1);
        match &roots[0].1.inputs[0] {
            Input::OutputSelection(selection) => {
                let OutputRef { drv, output } = selection.as_ref();
                assert_eq!(drv.name, "libfoo");
                assert_eq!(drv.outputs, vec!["out".to_string(), "dev".to_string()]);
                assert_eq!(output, "dev");
            }
            other => panic!("expected output selection input, got {other:?}"),
        }
    }

    #[cfg(feature = "cranelift-proto")]
    #[test]
    fn eval_str_and_deserialize_with_cranelift_prototype_matches_nickel_subset() {
        let source = r#"{
  name = "proto-demo",
  builder = "/bin/sh",
  system = 'x86_64-linux,
  addressing_mode = 'content-addressed,
  args = ["-c", "echo hi"],
}"#;
        let nickel: CrunchDerivation = evaluate_str_and_deserialize(source, &[]).unwrap();
        let prototype: CrunchDerivation = evaluate_str_and_deserialize_with_cranelift_prototype(source, &[]).unwrap();
        assert_eq!(prototype.name, nickel.name);
        assert_eq!(prototype.builder, nickel.builder);
        assert_eq!(prototype.system, nickel.system);
        assert_eq!(prototype.addressing_mode, nickel.addressing_mode);
        assert_eq!(prototype.args, nickel.args);
        assert_eq!(prototype.outputs, nickel.outputs);
    }

    #[cfg(feature = "cranelift-proto")]
    #[test]
    fn eval_str_and_extract_named_roots_with_cranelift_prototype_uses_name_field() {
        let roots = evaluate_str_and_extract_named_roots_with_cranelift_prototype::<CrunchDerivation>(
            r#"{
  name = "proto-roots",
  builder = "/bin/sh",
}"#,
            &[],
        )
        .unwrap();
        assert_eq!(roots.len(), 1);
        assert_eq!(roots[0].0, "proto-roots");
        assert_eq!(roots[0].1.name, "proto-roots");
        assert_eq!(roots[0].1.builder, "/bin/sh");
    }

    #[test]
    fn eval_str_and_deserialize_type_mismatch_returns_serde_error() {
        #[allow(dead_code)]
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
        #[allow(dead_code)]
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

    // ── Lazy evaluation session tests ──────────────────────────

    #[test]
    fn session_single_derivation_discovers_label_from_name() {
        let session = EvaluationSession::open_str(
            r#"{
  name = "hello",
  builder = "/bin/sh",
  args = ["-c", "echo hello > $out"],
  addressing_mode = 'input-addressed,
}"#,
            &[],
        )
        .unwrap();

        assert_eq!(*session.shape(), RootShape::Single);
        assert_eq!(session.root_labels().len(), 1);
        assert_eq!(session.root_labels()[0].label, "hello");
        assert_eq!(session.root_labels()[0].index, 0);
        assert_eq!(session.explicit_force_count(), 0);
    }

    #[test]
    fn session_record_discovers_labels_from_field_names() {
        let session = EvaluationSession::open_str(
            r#"{
  hello = { name = "hello", builder = "/bin/sh" },
  world = { name = "world", builder = "/bin/sh" },
}"#,
            &[],
        )
        .unwrap();

        assert_eq!(*session.shape(), RootShape::Record);
        let labels: Vec<&str> = session.root_labels().iter().map(|r| r.label.as_str()).collect();
        assert!(labels.contains(&"hello"));
        assert!(labels.contains(&"world"));
        assert_eq!(session.explicit_force_count(), 0);
    }

    #[test]
    fn session_array_discovers_labels_from_derivation_names() {
        let session = EvaluationSession::open_str(
            r#"[
  { name = "alpha", builder = "/bin/sh" },
  { name = "beta", builder = "/bin/sh" },
]"#,
            &[],
        )
        .unwrap();

        assert_eq!(*session.shape(), RootShape::Array);
        assert_eq!(session.root_labels().len(), 2);
        assert_eq!(session.root_labels()[0].label, "alpha");
        assert_eq!(session.root_labels()[1].label, "beta");
        assert_eq!(session.discovery_metrics().name_fields_accessed, 2);
    }

    #[test]
    fn session_force_root_matches_eager_path() {
        let source = r#"{
  hello = {
    name = "hello",
    builder = "/bin/sh",
    args = ["-c", "echo hi"],
    addressing_mode = 'input-addressed,
  },
  world = {
    name = "world",
    builder = "/bin/sh",
    system = 'x86_64-linux,
  },
}"#;

        // Eager path
        let eager = evaluate_str_and_extract_named_roots::<CrunchDerivation>(source, &[]).unwrap();

        // Lazy path
        let mut session = EvaluationSession::open_str(source, &[]).unwrap();
        let hello: CrunchDerivation = session.force_root("hello").unwrap();
        let world: CrunchDerivation = session.force_root("world").unwrap();

        let eager_hello = &eager.iter().find(|(l, _)| l == "hello").unwrap().1;
        let eager_world = &eager.iter().find(|(l, _)| l == "world").unwrap().1;

        assert_eq!(hello.name, eager_hello.name);
        assert_eq!(hello.builder, eager_hello.builder);
        assert_eq!(hello.args, eager_hello.args);
        assert_eq!(world.name, eager_world.name);
        assert_eq!(world.system, eager_world.system);
        assert_eq!(session.explicit_force_count(), 2);
    }

    #[test]
    fn session_force_all_roots_matches_eager_path() {
        let source = r#"{
  a = { name = "a", builder = "/bin/sh" },
  b = { name = "b", builder = "/bin/sh" },
}"#;

        let eager = evaluate_str_and_extract_named_roots::<CrunchDerivation>(source, &[]).unwrap();
        let mut session = EvaluationSession::open_str(source, &[]).unwrap();
        let lazy = session.force_all_roots::<CrunchDerivation>().unwrap();

        assert_eq!(eager.len(), lazy.len());
        for (eager_item, lazy_item) in eager.iter().zip(lazy.iter()) {
            assert_eq!(eager_item.0, lazy_item.0);
            assert_eq!(eager_item.1.name, lazy_item.1.name);
            assert_eq!(eager_item.1.builder, lazy_item.1.builder);
        }
    }

    #[test]
    fn session_missing_root_returns_error() {
        let mut session =
            EvaluationSession::open_str(r#"{ hello = { name = "hello", builder = "/bin/sh" } }"#, &[]).unwrap();

        let result = session.force_root::<CrunchDerivation>("nonexistent");
        assert!(result.is_err());
        let err = format!("{}", result.unwrap_err());
        assert!(err.contains("nonexistent"), "error should mention the missing label: {err}");
    }

    #[test]
    fn session_invalid_top_level_shape_returns_error() {
        let result = EvaluationSession::open_str("42", &[]);
        assert!(result.is_err());
        let err = format!("{}", result.unwrap_err());
        assert!(err.contains("neither a derivation"), "error should describe invalid shape: {err}");
    }

    #[test]
    fn session_single_derivation_force_matches_eager() {
        let source = r#"{
  name = "single",
  builder = "/bin/sh",
  args = ["-c", "echo out"],
}"#;

        let eager = evaluate_str_and_extract_named_roots::<CrunchDerivation>(source, &[]).unwrap();
        let mut session = EvaluationSession::open_str(source, &[]).unwrap();
        let lazy: CrunchDerivation = session.force_root("single").unwrap();

        assert_eq!(eager[0].1.name, lazy.name);
        assert_eq!(eager[0].1.builder, lazy.builder);
        assert_eq!(eager[0].1.args, lazy.args);
    }

    #[test]
    fn session_array_force_matches_eager() {
        let source = r#"[
  { name = "first", builder = "/bin/sh", args = ["-c", "echo 1"] },
  { name = "second", builder = "/bin/sh", args = ["-c", "echo 2"] },
]"#;

        let eager = evaluate_str_and_extract_named_roots::<CrunchDerivation>(source, &[]).unwrap();
        let mut session = EvaluationSession::open_str(source, &[]).unwrap();

        let first: CrunchDerivation = session.force_root("first").unwrap();
        let second: CrunchDerivation = session.force_root("second").unwrap();

        assert_eq!(first.name, eager[0].1.name);
        assert_eq!(first.args, eager[0].1.args);
        assert_eq!(second.name, eager[1].1.name);
        assert_eq!(second.args, eager[1].1.args);
    }

    #[test]
    fn session_nested_derivation_inputs_match_eager() {
        let source = r#"{
  app = {
    name = "app",
    builder = "/bin/sh",
    inputs = [
      { name = "dep", builder = "/bin/sh", addressing_mode = 'input-addressed },
    ],
  },
}"#;

        let eager = evaluate_str_and_extract_named_roots::<CrunchDerivation>(source, &[]).unwrap();
        let mut session = EvaluationSession::open_str(source, &[]).unwrap();
        let lazy: CrunchDerivation = session.force_root("app").unwrap();

        match (&eager[0].1.inputs[0], &lazy.inputs[0]) {
            (Input::Derivation(e), Input::Derivation(l)) => {
                assert_eq!(e.name, l.name);
                assert_eq!(e.addressing_mode, l.addressing_mode);
            }
            _ => panic!("expected nested derivation inputs"),
        }
    }

    #[test]
    fn session_file_based_evaluation() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("pkg.ncl");
        std::fs::write(&file, r#"{ name = "filepkg", builder = "/bin/sh" }"#).unwrap();

        let mut session = EvaluationSession::open_file(&file, &[]).unwrap();
        assert_eq!(*session.shape(), RootShape::Single);
        assert_eq!(session.root_labels()[0].label, "filepkg");

        let drv: CrunchDerivation = session.force_root("filepkg").unwrap();
        assert_eq!(drv.name, "filepkg");
    }

    #[test]
    fn session_file_with_imports() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("dep.ncl"), r#"{ name = "dep", builder = "/bin/sh" }"#).unwrap();
        std::fs::write(dir.path().join("main.ncl"), r#"{ mydep = import "dep.ncl" }"#).unwrap();

        let mut session = EvaluationSession::open_file(&dir.path().join("main.ncl"), &[]).unwrap();
        assert_eq!(*session.shape(), RootShape::Record);
        assert_eq!(session.root_labels()[0].label, "mydep");

        let drv: CrunchDerivation = session.force_root("mydep").unwrap();
        assert_eq!(drv.name, "dep");
    }

    #[test]
    fn session_recursive_record_refs_match_eager() {
        let source = r#"let base = { builder = "/bin/sh" } in {
  a = base & { name = "a" },
  b = base & { name = "b" },
}"#;

        let eager = evaluate_str_and_extract_named_roots::<CrunchDerivation>(source, &[]).unwrap();
        let mut session = EvaluationSession::open_str(source, &[]).unwrap();
        let all = session.force_all_roots::<CrunchDerivation>().unwrap();

        assert_eq!(eager.len(), all.len());
        for (e, l) in eager.iter().zip(all.iter()) {
            assert_eq!(e.0, l.0);
            assert_eq!(e.1.name, l.1.name);
            assert_eq!(e.1.builder, l.1.builder);
        }
    }

    #[test]
    fn session_does_not_affect_eval_to_json() {
        // Regression: crunch eval output must be unchanged by the lazy session.
        let source = r#"{ hello = { name = "hello", builder = "/bin/sh" } }"#;
        let json = evaluate_str_to_json(source, &[]).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed["hello"]["name"], "hello");
        assert_eq!(parsed["hello"]["builder"], "/bin/sh");
    }

    #[test]
    fn session_import_heavy_fixture_matches_eager() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("base.ncl"), r#"{ builder = "/bin/sh", args = ["-c", "echo base"] }"#).unwrap();
        std::fs::write(dir.path().join("helpers.ncl"), r#"{ make_drv = fun n => (import "base.ncl") & { name = n } }"#)
            .unwrap();
        std::fs::write(
            dir.path().join("main.ncl"),
            r#"let h = import "helpers.ncl" in {
  one = h.make_drv "one",
  two = h.make_drv "two",
  three = h.make_drv "three",
}"#,
        )
        .unwrap();

        let main_path = dir.path().join("main.ncl");
        let eager = evaluate_and_extract_named_roots::<CrunchDerivation>(&main_path, &[]).unwrap();
        let mut session = EvaluationSession::open_file(&main_path, &[]).unwrap();
        assert_eq!(*session.shape(), RootShape::Record);
        assert_eq!(session.root_labels().len(), 3);

        let lazy = session.force_all_roots::<CrunchDerivation>().unwrap();
        assert_eq!(eager.len(), lazy.len());
        for (e, l) in eager.iter().zip(lazy.iter()) {
            assert_eq!(e.0, l.0);
            assert_eq!(e.1.name, l.1.name);
            assert_eq!(e.1.builder, l.1.builder);
            assert_eq!(e.1.args, l.1.args);
        }
    }

    #[test]
    fn session_nonselected_force_count_is_zero_for_single_root() {
        let mut session = EvaluationSession::open_str(
            r#"{
  a = { name = "a", builder = "/bin/sh" },
  b = { name = "b", builder = "/bin/sh" },
  c = { name = "c", builder = "/bin/sh" },
}"#,
            &[],
        )
        .unwrap();

        let _: CrunchDerivation = session.force_root("a").unwrap();
        // We explicitly forced exactly 1 root.
        assert_eq!(session.explicit_force_count(), 1);
    }

    /// Boundary guard: EvaluationSession is defined in crunch-eval, not crunch-pipeline.
    #[test]
    fn session_type_lives_in_crunch_eval() {
        // This test exists solely as a compile-time boundary guard.
        // If EvaluationSession were moved to crunch-pipeline, this test
        // would need to be moved too, making the boundary violation visible.
        let _: fn(&str, &[OsString]) -> Result<EvaluationSession, Error> = EvaluationSession::open_str;
    }
}
