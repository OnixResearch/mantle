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

/// A lazy evaluation session that separates root discovery from per-root forcing.
///
/// The session keeps a Nickel `Context` alive so that shared thunks remain
/// evaluated across multiple root-forcing calls.
pub struct EvaluationSession {
    ctx: Context,
    /// Original source text, kept for per-root deep evaluation.
    source: String,
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

        let mut session = EvaluationSession {
            ctx,
            source,
            shape,
            labels: Vec::new(),
            deep_expr: None,
            forced_root_count: 0,
            discovery_metrics: DiscoveryMetrics::default(),
        };
        session.discover_roots()?;
        Ok(session)
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

    /// Force one root by label, deeply evaluating and deserializing into `T`.
    ///
    /// The first call triggers a deep export of the whole program (Nickel's
    /// sharing means previously-evaluated thunks are not re-forced). Subsequent
    /// calls reuse the cached deep expression.
    ///
    /// Returns an error if the label does not match any discovered root.
    pub fn force_root<T: DeserializeOwned>(&mut self, label: &str) -> Result<T, Error> {
        let root_label = self.labels.iter().find(|r| r.label == label).ok_or_else(|| {
            Error::Serde(format!(
                "no root with label '{label}'; available roots: {}",
                self.labels.iter().map(|r| r.label.as_str()).collect::<Vec<_>>().join(", ")
            ))
        })?;
        let index = root_label.index;

        self.ensure_deep_expr()?;
        let deep = self.deep_expr.as_ref().expect("deep_expr set by ensure_deep_expr");
        let shape = self.shape.clone();
        let field_expr = extract_field_expr(deep, &shape, index)?;

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
        Ok(self.deep_expr.as_ref().expect("deep_expr just set"))
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
                self.discover_record_labels();
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
            .ok_or_else(|| Error::Serde("single shape requires record expression".to_string()))?;
        let name_expr = record
            .value_by_name("name")
            .ok_or_else(|| Error::Serde("single derivation is missing 'name' field".to_string()))?;
        let name_value = self.ctx.eval_expr_shallow(name_expr).map_err(Error::Eval)?;
        self.discovery_metrics.name_fields_accessed = self.discovery_metrics.name_fields_accessed.saturating_add(1);
        name_value
            .as_str()
            .map(str::to_owned)
            .ok_or_else(|| Error::Serde("derivation 'name' field is not a string".to_string()))
    }

    fn discover_array_labels(&mut self) -> Result<(), Error> {
        // Re-shallow-eval to get the array (we need to shallow-eval each element).
        let shallow_expr = self.ctx.eval_shallow(&self.source).map_err(Error::Eval)?;
        let array = shallow_expr
            .as_array()
            .ok_or_else(|| Error::Serde("array shape requires array expression".to_string()))?;
        let count = array.len();
        assert!(count <= u32::MAX as usize, "array root count exceeds u32::MAX");

        for (i, item) in array.iter().enumerate() {
            let item_value = self.ctx.eval_expr_shallow(item).map_err(Error::Eval)?;
            let name_expr = item_value
                .as_record()
                .and_then(|r| r.value_by_name("name"))
                .ok_or_else(|| Error::Serde(format!("array element [{i}] is missing record 'name' field")))?;
            let name_value = self.ctx.eval_expr_shallow(name_expr).map_err(Error::Eval)?;
            self.discovery_metrics.name_fields_accessed = self.discovery_metrics.name_fields_accessed.saturating_add(1);
            let label = name_value
                .as_str()
                .map(str::to_owned)
                .ok_or_else(|| Error::Serde(format!("array element [{i}] 'name' field is not a string")))?;
            self.labels.push(RootLabel { label, index: i as u32 });
        }
        Ok(())
    }

    fn discover_record_labels(&mut self) {
        // Re-shallow-eval not needed: we stored the shape from the first shallow eval.
        // But we need record field names. Re-shallow-eval is a noop thanks to Nickel sharing.
        let shallow_expr = self.ctx.eval_shallow(&self.source).expect("re-shallow-eval should not fail");
        let record = shallow_expr.as_record().expect("record shape requires record");
        for (i, (key, _value)) in record.iter().enumerate() {
            assert!(i <= u32::MAX as usize, "record root count exceeds u32::MAX");
            self.labels.push(RootLabel {
                label: key.to_string(),
                index: i as u32,
            });
        }
    }
}

/// Classify the top-level shape from a shallowly-evaluated expression.
fn classify_shape(ctx: &mut Context, expr: &Expr) -> Result<RootShape, Error> {
    if expr.is_array() {
        return Ok(RootShape::Array);
    }

    let Some(record) = expr.as_record() else {
        return Err(Error::Serde(
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
                .ok_or_else(|| Error::Serde("expected array expression for array shape".to_string()))?;
            array.get(index as usize).ok_or_else(|| Error::Serde(format!("array index {index} out of bounds")))
        }
        RootShape::Record => {
            let record = expr
                .as_record()
                .ok_or_else(|| Error::Serde("expected record expression for record shape".to_string()))?;
            let (_key, value) = record
                .key_value_by_index(index as usize)
                .ok_or_else(|| Error::Serde(format!("record field index {index} out of bounds")))?;
            value.ok_or_else(|| Error::Serde(format!("record field at index {index} has no value")))
        }
    }
}
