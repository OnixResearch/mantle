//! Lazy evaluation session for root discovery and per-root forcing.
//!
//! An [`EvaluationSession`] holds a Nickel [`Context`] and separates
//! top-level root discovery (shallow) from per-root forcing (deep).
//! Root labels can be discovered without deep-forcing sibling values.
//! Individual roots are forced on demand through
//! [`EvaluationSession::force_root`].

use std::ffi::OsString;
use std::path::Path;

use nickel_lang::Context;
use nickel_lang::Expr;
use serde::de::DeserializeOwned;

use crate::Error;
use crate::deserialize_expr;
use crate::import_paths_for_file;

/// The shape discovered at the top level of a Nickel evaluation.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RootShape {
    /// A single derivation record (has a `name` field).
    Single,
    /// An array of derivation records.
    Array,
    /// A record of named derivation fields.
    Record,
}

/// A discovered root label with its position in the top-level value.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RootLabel {
    /// The label for this root (field name for records, derivation `name` for arrays/single).
    pub label: String,
    /// Index into the top-level value (0 for single, array index, or record field index).
    pub index: u32,
}

/// Metrics from the discovery phase.
#[derive(Clone, Debug, Default)]
pub struct DiscoveryMetrics {
    /// Number of top-level roots whose name field was explicitly accessed during discovery.
    pub name_fields_accessed: u32,
}

/// Immutable worker input for isolated root forcing.
///
/// This is the only state shared across threads for bounded multi-root
/// forcing. Each worker opens its own `EvaluationSession` from this input
/// instead of sharing a mutable Nickel `Context`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IsolatedWorkerInput {
    source: String,
    import_paths: Vec<OsString>,
    source_name: String,
    shape: RootShape,
    labels: Vec<RootLabel>,
}

impl IsolatedWorkerInput {
    /// Force one root through an isolated worker session.
    pub fn force_root<T: DeserializeOwned>(&self, label: &str) -> Result<T, Error> {
        let mut session = EvaluationSession::open_worker_source(
            self.source.clone(),
            &self.import_paths,
            &self.source_name,
            self.shape.clone(),
            self.labels.clone(),
        )?;
        let value = session.force_root(label)?;
        Ok(value)
    }
}

/// A lazy evaluation session that separates root discovery from per-root forcing.
///
/// The session keeps a Nickel `Context` alive so that shared thunks remain
/// evaluated across multiple root-forcing calls.
pub struct EvaluationSession {
    ctx: Context,
    /// Original source text, kept for per-root deep evaluation.
    source: String,
    /// Resolved import paths used to open this session.
    import_paths: Vec<OsString>,
    /// Source name used for diagnostics.
    source_name: String,
    /// Discovered root shape.
    shape: RootShape,
    /// Discovered root labels, populated during construction.
    labels: Vec<RootLabel>,
    /// The deep-exported expression, lazily populated on first force.
    deep_expr: Option<Expr>,
    /// How many roots have been explicitly forced through the session API.
    forced_root_count: u32,
    /// Discovery-phase metrics.
    discovery_metrics: DiscoveryMetrics,
}

impl std::fmt::Debug for EvaluationSession {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EvaluationSession")
            .field("source_name", &self.source_name)
            .field("shape", &self.shape)
            .field("labels", &self.labels)
            .field("forced_root_count", &self.forced_root_count)
            .field("has_deep_expr", &self.deep_expr.is_some())
            .finish_non_exhaustive()
    }
}

impl EvaluationSession {
    /// Open a session from a `.ncl` file path.
    ///
    /// Performs a shallow evaluation to determine the top-level shape
    /// and discover root labels without deep-forcing.
    pub fn open_file(path: &Path, import_paths: &[OsString]) -> Result<Self, Error> {
        let source = std::fs::read_to_string(path)?;
        let resolved_imports = import_paths_for_file(path, import_paths);
        Self::open_source(source, &resolved_imports, &path.display().to_string())
    }

    /// Open a session from a Nickel source string.
    pub fn open_str(source: &str, import_paths: &[OsString]) -> Result<Self, Error> {
        Self::open_source(source.to_string(), import_paths, "<input>")
    }

    fn open_source(source: String, import_paths: &[OsString], source_name: &str) -> Result<Self, Error> {
        assert!(!source_name.is_empty(), "source name must not be empty");

        let mut ctx = Context::new()
            .with_added_import_paths(import_paths.to_vec())
            .with_source_name(source_name.to_string());

        let shallow_expr = ctx.eval_shallow(&source).map_err(Error::Eval)?;
        let shape = classify_shape(&mut ctx, &shallow_expr)?;

        let mut session =
            Self::new_session(ctx, source, import_paths.to_vec(), source_name.to_string(), shape, Vec::new());
        session.discover_roots()?;
        Ok(session)
    }

    fn open_worker_source(
        source: String,
        import_paths: &[OsString],
        source_name: &str,
        shape: RootShape,
        labels: Vec<RootLabel>,
    ) -> Result<Self, Error> {
        assert!(!source_name.is_empty(), "source name must not be empty");
        assert!(!labels.is_empty(), "worker labels must not be empty");

        let ctx = Context::new()
            .with_added_import_paths(import_paths.to_vec())
            .with_source_name(source_name.to_string());
        let session = Self::new_session(ctx, source, import_paths.to_vec(), source_name.to_string(), shape, labels);
        Ok(session)
    }

    fn new_session(
        ctx: Context,
        source: String,
        import_paths: Vec<OsString>,
        source_name: String,
        shape: RootShape,
        labels: Vec<RootLabel>,
    ) -> Self {
        EvaluationSession {
            ctx,
            source,
            import_paths,
            source_name,
            shape,
            labels,
            deep_expr: None,
            forced_root_count: 0,
            discovery_metrics: DiscoveryMetrics::default(),
        }
    }

    /// The top-level shape of the evaluated program.
    pub fn shape(&self) -> &RootShape {
        &self.shape
    }

    /// The discovered root labels, in order.
    pub fn root_labels(&self) -> &[RootLabel] {
        &self.labels
    }

    /// How many top-level roots have been explicitly forced through this session.
    pub fn explicit_force_count(&self) -> u32 {
        self.forced_root_count
    }

    /// Discovery-phase metrics.
    pub fn discovery_metrics(&self) -> &DiscoveryMetrics {
        &self.discovery_metrics
    }

    /// Clone immutable worker input for isolated per-root forcing.
    pub fn isolated_worker_input(&self) -> IsolatedWorkerInput {
        IsolatedWorkerInput {
            source: self.source.clone(),
            import_paths: self.import_paths.clone(),
            source_name: self.source_name.clone(),
            shape: self.shape.clone(),
            labels: self.labels.clone(),
        }
    }

    /// Force one root through an isolated worker session.
    pub fn force_root_isolated<T: DeserializeOwned>(&self, label: &str) -> Result<T, Error> {
        let worker_input = self.isolated_worker_input();
        match worker_input.force_root(label) {
            Ok(value) => Ok(value),
            Err(err) => Err(label_error(label, err)),
        }
    }

    /// Force a selected label set serially through isolated worker sessions.
    pub fn force_selected_roots<T: DeserializeOwned + Send + 'static>(
        &self,
        labels: &[String],
    ) -> Result<Vec<(String, T)>, Error> {
        self.force_selected_roots_bounded(labels, 1)
    }

    /// Force a selected label set through isolated worker sessions under a bounded cap.
    pub fn force_selected_roots_bounded<T: DeserializeOwned + Send + 'static>(
        &self,
        labels: &[String],
        max_concurrency: u32,
    ) -> Result<Vec<(String, T)>, Error> {
        let worker_input = self.isolated_worker_input();
        force_selected_roots_with_workers(&worker_input, labels, max_concurrency)
    }

    /// Force all discovered roots through isolated worker sessions under a bounded cap.
    pub fn force_all_roots_bounded<T: DeserializeOwned + Send + 'static>(
        &self,
        max_concurrency: u32,
    ) -> Result<Vec<(String, T)>, Error> {
        let labels = self.labels.iter().map(|root_label| root_label.label.clone()).collect::<Vec<_>>();
        self.force_selected_roots_bounded(&labels, max_concurrency)
    }

    /// Force one root by label, deeply evaluating and deserializing into `T`.
    ///
    /// The first call triggers a deep export of the whole program (Nickel's
    /// sharing means previously-evaluated thunks are not re-forced). Subsequent
    /// calls reuse the cached deep expression.
    ///
    /// Returns an error if the label does not match any discovered root.
    pub fn force_root<T: DeserializeOwned>(&mut self, label: &str) -> Result<T, Error> {
        let root_label = self.labels.iter().find(|r| r.label == label).ok_or_else(|| {
            Error::Boundary(format!(
                "no root with label '{label}'; available roots: {}",
                self.labels.iter().map(|r| r.label.as_str()).collect::<Vec<_>>().join(", ")
            ))
        })?;
        let index = root_label.index;

        let shape = self.shape.clone();
        let field_expr = {
            let deep = self.ensure_deep_expr()?;
            extract_field_expr(deep, &shape, index)?
        };

        self.forced_root_count = self.forced_root_count.saturating_add(1);
        deserialize_expr(&field_expr, &format!("root '{label}'"))
    }

    /// Force all roots, returning `(label, value)` pairs.
    pub fn force_all_roots<T: DeserializeOwned>(&mut self) -> Result<Vec<(String, T)>, Error> {
        let labels: Vec<RootLabel> = self.labels.clone();
        let mut results = Vec::with_capacity(labels.len());
        for root_label in &labels {
            let value: T = self.force_root(&root_label.label)?;
            results.push((root_label.label.clone(), value));
        }
        Ok(results)
    }

    /// Ensure the deep expression is available, performing deep eval if needed.
    fn ensure_deep_expr(&mut self) -> Result<&Expr, Error> {
        if self.deep_expr.is_none() {
            let expr = self.ctx.eval_deep_for_export(&self.source).map_err(Error::Eval)?;
            self.deep_expr = Some(expr);
        }
        self.deep_expr
            .as_ref()
            .ok_or_else(|| Error::Boundary("deep evaluation cache was not populated".to_string()))
    }

    fn discover_roots(&mut self) -> Result<(), Error> {
        match &self.shape {
            RootShape::Single => {
                let label = self.discover_single_label()?;
                self.labels.push(RootLabel { label, index: 0 });
            }
            RootShape::Array => {
                self.discover_array_labels()?;
            }
            RootShape::Record => {
                self.discover_record_labels()?;
            }
        }
        Ok(())
    }

    fn discover_single_label(&mut self) -> Result<String, Error> {
        // For a single derivation, shallow eval gives us a record at WHNF.
        // We shallow-eval the `name` field to get the label.
        let shallow_expr = self.ctx.eval_shallow(&self.source).map_err(Error::Eval)?;
        let record = shallow_expr
            .as_record()
            .ok_or_else(|| Error::Boundary("single shape requires a record expression".to_string()))?;
        let name_expr = record
            .value_by_name("name")
            .ok_or_else(|| Error::Boundary("single derivation is missing 'name' field".to_string()))?;
        let name_value = self.ctx.eval_expr_shallow(name_expr).map_err(Error::Eval)?;
        self.discovery_metrics.name_fields_accessed = self.discovery_metrics.name_fields_accessed.saturating_add(1);
        name_value
            .as_str()
            .map(str::to_owned)
            .ok_or_else(|| Error::Boundary("derivation 'name' field is not a string".to_string()))
    }

    fn discover_array_labels(&mut self) -> Result<(), Error> {
        // Re-shallow-eval to get the array (we need to shallow-eval each element).
        let shallow_expr = self.ctx.eval_shallow(&self.source).map_err(Error::Eval)?;
        let array = shallow_expr
            .as_array()
            .ok_or_else(|| Error::Boundary("array shape requires an array expression".to_string()))?;
        let count = array.len();
        assert!(count <= u32::MAX as usize, "array root count exceeds u32::MAX");

        for (i, item) in array.iter().enumerate() {
            let item_value = self.ctx.eval_expr_shallow(item).map_err(Error::Eval)?;
            let name_expr = item_value
                .as_record()
                .and_then(|r| r.value_by_name("name"))
                .ok_or_else(|| Error::Boundary(format!("array element [{i}] is missing record 'name' field")))?;
            let name_value = self.ctx.eval_expr_shallow(name_expr).map_err(Error::Eval)?;
            self.discovery_metrics.name_fields_accessed = self.discovery_metrics.name_fields_accessed.saturating_add(1);
            let label = name_value
                .as_str()
                .map(str::to_owned)
                .ok_or_else(|| Error::Boundary(format!("array element [{i}] 'name' field is not a string")))?;
            self.labels.push(RootLabel { label, index: i as u32 });
        }
        Ok(())
    }

    fn discover_record_labels(&mut self) -> Result<(), Error> {
        // Re-shallow-eval not needed: we stored the shape from the first shallow eval.
        // But we need record field names. Re-shallow-eval is a noop thanks to Nickel sharing.
        let shallow_expr = self.ctx.eval_shallow(&self.source).map_err(Error::Eval)?;
        let record = shallow_expr
            .as_record()
            .ok_or_else(|| Error::Boundary("record shape requires a record expression".to_string()))?;
        for (i, (key, _value)) in record.iter().enumerate() {
            assert!(i <= u32::MAX as usize, "record root count exceeds u32::MAX");
            self.labels.push(RootLabel {
                label: key.to_string(),
                index: i as u32,
            });
        }
        Ok(())
    }
}

#[derive(Debug)]
struct IndexedRoot<T> {
    index: u32,
    label: String,
    value: T,
}

#[derive(Debug)]
struct WorkerAssignment {
    start_index: u32,
    labels: Vec<String>,
}

#[derive(Debug)]
struct WorkerFailure {
    index: u32,
    label: String,
    detail: String,
}

fn force_selected_roots_with_workers<T: DeserializeOwned + Send + 'static>(
    worker_input: &IsolatedWorkerInput,
    labels: &[String],
    max_concurrency: u32,
) -> Result<Vec<(String, T)>, Error> {
    let requested_root_count = labels.len();
    let concurrency_cap = normalize_concurrency_cap(max_concurrency, requested_root_count);
    let worker_assignments = build_worker_assignments(labels, concurrency_cap);
    let mut indexed_results = Vec::with_capacity(requested_root_count);
    if requested_root_count == 0 {
        return Ok(Vec::new());
    }

    let mut handles = Vec::with_capacity(worker_assignments.len());
    for assignment in worker_assignments {
        let worker = worker_input.clone();
        handles.push(std::thread::spawn(move || -> Result<Vec<IndexedRoot<T>>, WorkerFailure> {
            force_worker_assignment(&worker, assignment)
        }));
    }

    for handle in handles {
        let worker_result =
            handle.join().map_err(|_| Error::Boundary("isolated root forcing worker panicked".to_string()))?;
        match worker_result {
            Ok(mut worker_values) => indexed_results.append(&mut worker_values),
            Err(failure) => {
                return Err(Error::Boundary(format!(
                    "root '{}' at request index {}: {}",
                    failure.label, failure.index, failure.detail
                )));
            }
        }
    }

    indexed_results.sort_by(|left, right| left.index.cmp(&right.index));
    let ordered_results =
        indexed_results.into_iter().map(|indexed_root| (indexed_root.label, indexed_root.value)).collect();
    Ok(ordered_results)
}

fn build_worker_assignments(labels: &[String], concurrency_cap: u32) -> Vec<WorkerAssignment> {
    assert!(concurrency_cap >= 1, "concurrency_cap must be at least 1");
    assert!(labels.len() <= u32::MAX as usize, "label count must fit in u32");
    if labels.is_empty() {
        return Vec::new();
    }

    let chunk_len = labels.len().div_ceil(concurrency_cap as usize);
    assert!(chunk_len >= 1, "chunk_len must be at least 1");

    let mut assignments = Vec::new();
    for (chunk_index, chunk_labels) in labels.chunks(chunk_len).enumerate() {
        let start_index = chunk_index
            .checked_mul(chunk_len)
            .and_then(|value| u32::try_from(value).ok())
            .expect("worker assignment start index must fit in u32");
        assignments.push(WorkerAssignment {
            start_index,
            labels: chunk_labels.to_vec(),
        });
    }
    assignments
}

fn force_worker_assignment<T: DeserializeOwned + Send + 'static>(
    worker_input: &IsolatedWorkerInput,
    assignment: WorkerAssignment,
) -> Result<Vec<IndexedRoot<T>>, WorkerFailure> {
    let mut session = EvaluationSession::open_source(
        worker_input.source.clone(),
        &worker_input.import_paths,
        &worker_input.source_name,
    )
    .map_err(|err| WorkerFailure {
        index: assignment.start_index,
        label: assignment.labels.first().cloned().unwrap_or_else(|| "<none>".to_string()),
        detail: err.to_string(),
    })?;
    let mut results = Vec::with_capacity(assignment.labels.len());

    for (offset, label) in assignment.labels.iter().enumerate() {
        let request_index = assignment.start_index.saturating_add(offset as u32);
        let value = session.force_root(label).map_err(|err| WorkerFailure {
            index: request_index,
            label: label.clone(),
            detail: err.to_string(),
        })?;
        results.push(IndexedRoot {
            index: request_index,
            label: label.clone(),
            value,
        });
    }

    Ok(results)
}

fn normalize_concurrency_cap(max_concurrency: u32, requested_root_count: usize) -> u32 {
    assert!(requested_root_count <= u32::MAX as usize, "requested root count must fit in u32");
    if requested_root_count == 0 {
        return 1;
    }

    let requested_root_count_u32 = requested_root_count as u32;
    let normalized_cap = if max_concurrency == 0 { 1 } else { max_concurrency };
    normalized_cap.min(requested_root_count_u32)
}

fn label_error(label: &str, err: Error) -> Error {
    Error::Labeled {
        label: label.to_string(),
        source: Box::new(err),
    }
}

/// Classify the top-level shape from a shallowly-evaluated expression.
fn classify_shape(ctx: &mut Context, expr: &Expr) -> Result<RootShape, Error> {
    if expr.is_array() {
        return Ok(RootShape::Array);
    }

    let Some(record) = expr.as_record() else {
        return Err(Error::Boundary(
            "top-level value is neither a derivation, an array of derivations, \
             nor a record of derivations"
                .to_string(),
        ));
    };

    // A record with a `name` field is treated as a single derivation.
    // For a record-of-derivations, individual fields don't have `name` at the top level.
    if let Some(name_expr) = record.value_by_name("name") {
        // Shallow-eval the name to check if it's a string.
        if let Ok(name_value) = ctx.eval_expr_shallow(name_expr)
            && name_value.as_str().is_some()
        {
            return Ok(RootShape::Single);
        }
    }

    Ok(RootShape::Record)
}

/// Extract a field expression from the deeply-evaluated top-level value.
fn extract_field_expr(expr: &Expr, shape: &RootShape, index: u32) -> Result<Expr, Error> {
    match shape {
        RootShape::Single => {
            assert!(index == 0, "single root index must be 0");
            Ok(expr.clone())
        }
        RootShape::Array => {
            let array = expr
                .as_array()
                .ok_or_else(|| Error::Boundary("expected array expression for array shape".to_string()))?;
            array
                .get(index as usize)
                .ok_or_else(|| Error::Boundary(format!("array index {index} out of bounds")))
        }
        RootShape::Record => {
            let record = expr
                .as_record()
                .ok_or_else(|| Error::Boundary("expected record expression for record shape".to_string()))?;
            let (_key, value) = record
                .key_value_by_index(index as usize)
                .ok_or_else(|| Error::Boundary(format!("record field index {index} out of bounds")))?;
            value.ok_or_else(|| Error::Boundary(format!("record field at index {index} has no value")))
        }
    }
}

#[cfg(test)]
mod tests {
    use std::panic::AssertUnwindSafe;

    use crunch_glue::CrunchDerivation;

    use super::*;

    fn eval_deep_expr(source: &str) -> Expr {
        let mut ctx = Context::new().with_source_name("<test>".to_string());
        ctx.eval_deep_for_export(source).unwrap()
    }

    fn test_session(
        shape: RootShape,
        source: &str,
        labels: Vec<RootLabel>,
        deep_expr: Option<Expr>,
    ) -> EvaluationSession {
        EvaluationSession {
            ctx: Context::new().with_source_name("<test>".to_string()),
            source: source.to_string(),
            import_paths: Vec::new(),
            source_name: "<test>".to_string(),
            shape,
            labels,
            deep_expr,
            forced_root_count: 0,
            discovery_metrics: DiscoveryMetrics::default(),
        }
    }

    #[test]
    fn normalize_concurrency_cap_defaults_zero_to_one() {
        assert_eq!(normalize_concurrency_cap(0, 0), 1);
        assert_eq!(normalize_concurrency_cap(0, 1), 1);
        assert_eq!(normalize_concurrency_cap(0, 3), 1);
    }

    #[test]
    fn normalize_concurrency_cap_clamps_to_requested_roots() {
        assert_eq!(normalize_concurrency_cap(4, 2), 2);
        assert_eq!(normalize_concurrency_cap(2, 4), 2);
        assert_eq!(normalize_concurrency_cap(1, 4), 1);
    }

    #[test]
    fn isolated_worker_input_is_send() {
        fn assert_send<T: Send>() {}

        assert_send::<IsolatedWorkerInput>();
    }

    #[test]
    fn force_root_returns_boundary_error_for_inconsistent_cached_shape_without_panic() {
        let mut session = test_session(
            RootShape::Record,
            r#"{ demo = { name = "demo", builder = "/bin/sh" } }"#,
            vec![RootLabel {
                label: "demo".to_string(),
                index: 0,
            }],
            Some(eval_deep_expr(
                r#"[
  { name = "demo", builder = "/bin/sh" },
]"#,
            )),
        );

        let result = std::panic::catch_unwind(AssertUnwindSafe(|| session.force_root::<CrunchDerivation>("demo")));
        assert!(result.is_ok(), "force_root should not panic on inconsistent cached shape");
        let err = result.unwrap().unwrap_err();
        assert!(matches!(err, Error::Boundary(_)), "expected boundary error, got: {err}");
    }

    #[test]
    fn discover_record_labels_returns_boundary_error_for_stale_shape_without_panic() {
        let mut session = test_session(
            RootShape::Record,
            r#"[
  { name = "demo", builder = "/bin/sh" },
]"#,
            Vec::new(),
            None,
        );

        let result = std::panic::catch_unwind(AssertUnwindSafe(|| session.discover_roots()));
        assert!(result.is_ok(), "discover_roots should not panic on stale record shape");
        let err = result.unwrap().unwrap_err();
        assert!(matches!(err, Error::Boundary(_)), "expected boundary error, got: {err}");
    }

    #[test]
    fn force_root_isolated_matches_same_session_force_root() {
        let source = r#"{
  alpha = { name = "alpha", builder = "/bin/sh" },
  beta = { name = "beta", builder = "/bin/sh" },
}"#;
        let mut session = EvaluationSession::open_str(source, &[]).unwrap();

        let serial: CrunchDerivation = session.force_root("alpha").unwrap();
        let isolated: CrunchDerivation = session.force_root_isolated("alpha").unwrap();

        assert_eq!(serial.name, isolated.name);
        assert_eq!(serial.builder, isolated.builder);
    }

    #[test]
    fn force_selected_roots_bounded_preserves_requested_order() {
        let source = r#"{
  alpha = { name = "alpha", builder = "/bin/sh" },
  beta = { name = "beta", builder = "/bin/sh" },
  gamma = { name = "gamma", builder = "/bin/sh" },
}"#;
        let session = EvaluationSession::open_str(source, &[]).unwrap();
        let labels = vec!["gamma".to_string(), "alpha".to_string(), "beta".to_string()];

        let roots = session.force_selected_roots_bounded::<CrunchDerivation>(&labels, 3).unwrap();
        let observed = roots.into_iter().map(|(label, _)| label).collect::<Vec<_>>();

        assert_eq!(observed, labels);
    }

    #[test]
    fn force_selected_roots_bounded_reports_failed_label() {
        let source = r#"{
  good = { name = "good", builder = "/bin/sh" },
  bad = { builder = "/bin/sh" },
}"#;
        let session = EvaluationSession::open_str(source, &[]).unwrap();
        let labels = vec!["good".to_string(), "missing".to_string()];

        let err = session.force_selected_roots_bounded::<CrunchDerivation>(&labels, 2).unwrap_err();
        let rendered = err.to_string();

        assert!(matches!(err, Error::Boundary(_)), "expected boundary error, got: {rendered}");
        assert!(rendered.contains("missing"), "error should mention failed label: {rendered}");
    }

    #[test]
    fn force_all_roots_bounded_matches_serial_path() {
        let source = r#"{
  alpha = { name = "alpha", builder = "/bin/sh" },
  beta = { name = "beta", builder = "/bin/sh" },
  gamma = { name = "gamma", builder = "/bin/sh" },
}"#;
        let mut serial_session = EvaluationSession::open_str(source, &[]).unwrap();
        let bounded_session = EvaluationSession::open_str(source, &[]).unwrap();

        let serial = serial_session.force_all_roots::<CrunchDerivation>().unwrap();
        let bounded = bounded_session.force_all_roots_bounded::<CrunchDerivation>(3).unwrap();

        assert_eq!(serial.len(), bounded.len());
        for (serial_item, bounded_item) in serial.iter().zip(bounded.iter()) {
            assert_eq!(serial_item.0, bounded_item.0);
            assert_eq!(serial_item.1.name, bounded_item.1.name);
            assert_eq!(serial_item.1.builder, bounded_item.1.builder);
        }
    }
}
