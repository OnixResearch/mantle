//! Lazy evaluation session for root discovery and per-root forcing.
//!
//! An [`EvaluationSession`] holds a Nickel [`Context`] and separates
//! top-level root discovery (shallow) from per-root forcing (deep).
//! Root labels can be discovered without deep-forcing sibling values.
//! Individual roots are forced on demand through
//! [`EvaluationSession::force_root`].

use std::any::Any;
use std::collections::BTreeMap;
use std::ffi::OsString;
use std::path::Path;
use std::sync::Arc;
use std::sync::Condvar;
use std::sync::Mutex;
use std::sync::OnceLock;
use std::sync::atomic::AtomicU32;
use std::sync::atomic::Ordering;
use std::sync::mpsc;

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

/// Host execution policy for isolated multi-root forcing.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RootForceExecutionPolicy {
    /// Run worker assignments inline in the current thread.
    Inline,
    /// Prefer the threaded executor and fall back to inline when it is not selected
    /// or cannot be initialized.
    PreferThreaded,
}

/// Immutable worker input for isolated root forcing.
///
/// This is the only state shared across threads for bounded multi-root
/// forcing. Each worker opens its own `EvaluationSession` from this input
/// instead of sharing a mutable Nickel `Context`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IsolatedWorkerInput {
    source: Arc<str>,
    import_paths: Arc<[OsString]>,
    source_name: Arc<str>,
    shape: RootShape,
    labels: Arc<[RootLabel]>,
}

impl IsolatedWorkerInput {
    /// Force one root through an isolated worker session.
    pub fn force_root<T: DeserializeOwned>(&self, label: &str) -> Result<T, Error> {
        let mut session = EvaluationSession::open_worker_source(
            self.source.clone(),
            self.import_paths.clone(),
            self.source_name.clone(),
            self.shape.clone(),
            self.labels.to_vec(),
        )?;
        let value = session.force_root(label)?;
        Ok(value)
    }

    /// Force a selected label set through the requested host execution policy.
    pub fn force_selected_roots_with_policy<T: DeserializeOwned + Send + 'static>(
        &self,
        labels: &[String],
        max_concurrency: u32,
        policy: RootForceExecutionPolicy,
    ) -> Result<Vec<(String, T)>, Error> {
        force_selected_roots_with_policy(self, labels, max_concurrency, policy)
    }
}

/// A lazy evaluation session that separates root discovery from per-root forcing.
///
/// The session keeps a Nickel `Context` alive so that shared thunks remain
/// evaluated across multiple root-forcing calls.
pub struct EvaluationSession {
    ctx: Context,
    /// Original source text, kept for per-root deep evaluation.
    source: Arc<str>,
    /// Resolved import paths used to open this session.
    import_paths: Arc<[OsString]>,
    /// Source name used for diagnostics.
    source_name: Arc<str>,
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

struct SessionInit {
    ctx: Context,
    source: Arc<str>,
    import_paths: Arc<[OsString]>,
    source_name: Arc<str>,
    shape: RootShape,
    labels: Vec<RootLabel>,
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
        let resolved_scope = import_paths_for_file(path, import_paths);
        Self::open_source(source, &resolved_scope, &path.display().to_string())
    }

    /// Open a session from a Nickel source string.
    pub fn open_str(source: &str, import_paths: &[OsString]) -> Result<Self, Error> {
        Self::open_source(source.to_string(), import_paths, "<input>")
    }

    fn open_source(source: String, import_paths: &[OsString], source_name: &str) -> Result<Self, Error> {
        assert!(!source_name.is_empty(), "source name must not be empty");

        let source: Arc<str> = source.into();
        let scope: Arc<[OsString]> = import_paths.to_vec().into();
        let source_name: Arc<str> = source_name.to_string().into();
        let mut ctx = Context::new()
            .with_added_import_paths(scope.to_vec())
            .with_source_name(source_name.to_string());

        assert!(!source.is_empty(), "source must not be empty");
        assert!(
            !scope.iter().any(|path| path.is_empty()),
            "import paths must not contain empty entries"
        );

        let shallow_expr = ctx.eval_shallow(&source).map_err(Error::Eval)?;
        let shape = classify_shape(&mut ctx, &shallow_expr)?;

        let init = SessionInit {
            ctx,
            source,
            import_paths: scope,
            source_name,
            shape,
            labels: Vec::new(),
        };
        let mut session = Self::new_session(init);
        session.discover_roots()?;
        Ok(session)
    }

    fn open_worker_source(
        source: Arc<str>,
        import_paths: Arc<[OsString]>,
        source_name: Arc<str>,
        shape: RootShape,
        labels: Vec<RootLabel>,
    ) -> Result<Self, Error> {
        assert!(!source_name.is_empty(), "source name must not be empty");
        assert!(!labels.is_empty(), "worker labels must not be empty");

        let ctx = Context::new()
            .with_added_import_paths(import_paths.to_vec())
            .with_source_name(source_name.to_string());
        let init = SessionInit {
            ctx,
            source,
            import_paths,
            source_name,
            shape,
            labels,
        };
        let session = Self::new_session(init);
        Ok(session)
    }

    fn new_session(init: SessionInit) -> Self {
        EvaluationSession {
            ctx: init.ctx,
            source: init.source,
            import_paths: init.import_paths,
            source_name: init.source_name,
            shape: init.shape,
            labels: init.labels,
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
            labels: Arc::from(self.labels.clone()),
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
        self.force_selected_roots_with_policy(labels, 1, RootForceExecutionPolicy::Inline)
    }

    /// Force a selected label set through isolated worker sessions under a bounded cap.
    pub fn force_selected_roots_bounded<T: DeserializeOwned + Send + 'static>(
        &self,
        labels: &[String],
        max_concurrency: u32,
    ) -> Result<Vec<(String, T)>, Error> {
        self.force_selected_roots_with_policy(labels, max_concurrency, RootForceExecutionPolicy::PreferThreaded)
    }

    /// Force a selected label set through an explicit host execution policy.
    pub fn force_selected_roots_with_policy<T: DeserializeOwned + Send + 'static>(
        &self,
        labels: &[String],
        max_concurrency: u32,
        policy: RootForceExecutionPolicy,
    ) -> Result<Vec<(String, T)>, Error> {
        let worker_input = self.isolated_worker_input();
        worker_input.force_selected_roots_with_policy(labels, max_concurrency, policy)
    }

    /// Force all discovered roots through isolated worker sessions under a bounded cap.
    pub fn force_all_roots_bounded<T: DeserializeOwned + Send + 'static>(
        &mut self,
        max_concurrency: u32,
    ) -> Result<Vec<(String, T)>, Error> {
        self.force_all_roots_with_policy(max_concurrency, RootForceExecutionPolicy::PreferThreaded)
    }

    /// Force all discovered roots through an explicit host execution policy.
    pub fn force_all_roots_with_policy<T: DeserializeOwned + Send + 'static>(
        &mut self,
        max_concurrency: u32,
        policy: RootForceExecutionPolicy,
    ) -> Result<Vec<(String, T)>, Error> {
        let requested_root_count = self.labels.len();
        let effective_concurrency = effective_parallel_workers(requested_root_count, max_concurrency);
        if should_reuse_session_for_all_roots(requested_root_count, effective_concurrency, policy) {
            return self.force_all_roots();
        }

        let labels = self.labels.iter().map(|root_label| root_label.label.clone()).collect::<Vec<_>>();
        self.force_selected_roots_with_policy(&labels, max_concurrency, policy)
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
        assert!(count <= usize_limit_from_u32_max(), "array root count exceeds u32::MAX");

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
            self.labels.push(RootLabel {
                label,
                index: u32_from_usize(i),
            });
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
            assert!(i <= usize_limit_from_u32_max(), "record root count exceeds u32::MAX");
            self.labels.push(RootLabel {
                label: key.to_string(),
                index: u32_from_usize(i),
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

#[derive(Clone, Debug)]
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

trait WorkerJob {
    fn run(self: Box<Self>);
}

impl<F> WorkerJob for F
where F: FnOnce()
{
    fn run(self: Box<Self>) {
        (*self)()
    }
}

type BoxedWorkerJob = Box<dyn WorkerJob + Send + 'static>;
type ErasedWorkerResult = Box<dyn Any + Send + 'static>;

struct WorkerResultSlot {
    payload: Mutex<Option<ErasedWorkerResult>>,
    ready: Condvar,
}

impl WorkerResultSlot {
    fn new() -> Self {
        Self {
            payload: Mutex::new(None),
            ready: Condvar::new(),
        }
    }

    fn clear(&self) -> Result<(), Error> {
        let mut payload = self
            .payload
            .lock()
            .map_err(|_| Error::Boundary("bounded worker result slot mutex was poisoned".to_string()))?;
        payload.take();
        Ok(())
    }

    fn store<T: Send + 'static>(&self, result: Result<Vec<IndexedRoot<T>>, WorkerFailure>) {
        let mut payload = match self.payload.lock() {
            Ok(payload) => payload,
            Err(_poisoned) => return,
        };
        *payload = Some(Box::new(result));
        self.ready.notify_one();
    }

    fn take<T: Send + 'static>(&self) -> Result<Result<Vec<IndexedRoot<T>>, WorkerFailure>, Error> {
        let payload = self
            .payload
            .lock()
            .map_err(|_| Error::Boundary("bounded worker result slot mutex was poisoned".to_string()))?;
        let mut payload = self
            .ready
            .wait_while(payload, |slot| slot.is_none())
            .map_err(|_| Error::Boundary("bounded worker result slot wait was poisoned".to_string()))?;
        let erased = payload
            .take()
            .ok_or_else(|| Error::Boundary("bounded worker result slot completed without a payload".to_string()))?;
        erased
            .downcast::<Result<Vec<IndexedRoot<T>>, WorkerFailure>>()
            .map(|boxed| *boxed)
            .map_err(|_| Error::Boundary("bounded worker result slot held an unexpected payload type".to_string()))
    }
}

struct BoundedWorkerSlot {
    sender: mpsc::SyncSender<BoxedWorkerJob>,
    result_slot: Arc<WorkerResultSlot>,
}

struct BoundedWorkerPool {
    workers: Vec<BoundedWorkerSlot>,
    next_worker: AtomicU32,
}

impl BoundedWorkerPool {
    fn submit<T: Send + 'static>(
        &self,
        job: impl FnOnce() -> Result<Vec<IndexedRoot<T>>, WorkerFailure> + Send + 'static,
    ) -> Result<Arc<WorkerResultSlot>, Error> {
        assert!(!self.workers.is_empty(), "worker pool must have at least one worker");

        let worker_count = self.workers.len();
        assert!(worker_count != 0, "worker pool must have at least one worker");
        let next_worker = usize_from_u32(self.next_worker.fetch_add(1, Ordering::Relaxed));
        let worker_index = next_worker % worker_count;
        let worker_slot = &self.workers[worker_index];
        worker_slot.result_slot.clear()?;
        let result_slot = worker_slot.result_slot.clone();
        let result_slot_for_job = result_slot.clone();
        let boxed_job: BoxedWorkerJob = Box::new(move || {
            result_slot_for_job.store(job());
        });
        worker_slot
            .sender
            .send(boxed_job)
            .map_err(|_| Error::Boundary("bounded worker pool thread exited unexpectedly".to_string()))?;
        Ok(result_slot)
    }
}

trait AssignmentExecutor {
    fn execute<T: DeserializeOwned + Send + 'static>(
        &self,
        worker_input: &IsolatedWorkerInput,
        assignments: &[WorkerAssignment],
    ) -> Result<Vec<Result<Vec<IndexedRoot<T>>, WorkerFailure>>, Error>;
}

#[derive(Clone, Copy, Debug, Default)]
struct InlineExecutor;

impl AssignmentExecutor for InlineExecutor {
    fn execute<T: DeserializeOwned + Send + 'static>(
        &self,
        worker_input: &IsolatedWorkerInput,
        assignments: &[WorkerAssignment],
    ) -> Result<Vec<Result<Vec<IndexedRoot<T>>, WorkerFailure>>, Error> {
        let mut results = Vec::with_capacity(assignments.len());
        for assignment in assignments {
            results.push(force_worker_assignment(worker_input, assignment.clone()));
        }
        Ok(results)
    }
}

#[derive(Clone, Copy, Debug, Default)]
struct ThreadedExecutor;

impl AssignmentExecutor for ThreadedExecutor {
    fn execute<T: DeserializeOwned + Send + 'static>(
        &self,
        worker_input: &IsolatedWorkerInput,
        assignments: &[WorkerAssignment],
    ) -> Result<Vec<Result<Vec<IndexedRoot<T>>, WorkerFailure>>, Error> {
        let worker_pool = bounded_worker_pool(u32_from_usize(assignments.len()))?;
        let mut worker_receivers = Vec::with_capacity(assignments.len());
        for assignment in assignments {
            let worker = worker_input.clone();
            let assignment = assignment.clone();
            let receiver = worker_pool.submit::<T>(move || force_worker_assignment(&worker, assignment))?;
            worker_receivers.push(receiver);
        }
        collect_threaded_results(worker_receivers)
    }
}

const MIN_ROOTS_PER_WORKER: usize = 8;
const SESSION_REUSE_ALL_ROOTS_MAX_WORKERS: u32 = 2;
const SESSION_REUSE_ALL_ROOTS_MAX_ROOTS: usize = MIN_ROOTS_PER_WORKER * usize_from_u32_const(SESSION_REUSE_ALL_ROOTS_MAX_WORKERS);

const fn usize_from_u32_const(value: u32) -> usize {
    value as usize
}

fn usize_from_u32(value: u32) -> usize {
    match usize::try_from(value) {
        Ok(converted) => converted,
        Err(_) => usize::MAX,
    }
}

fn u32_from_usize(value: usize) -> u32 {
    match u32::try_from(value) {
        Ok(converted) => converted,
        Err(_) => u32::MAX,
    }
}

fn usize_limit_from_u32_max() -> usize {
    usize_from_u32(u32::MAX)
}

fn force_selected_roots_with_policy<T: DeserializeOwned + Send + 'static>(
    worker_input: &IsolatedWorkerInput,
    labels: &[String],
    max_concurrency: u32,
    policy: RootForceExecutionPolicy,
) -> Result<Vec<(String, T)>, Error> {
    let requested_root_count = labels.len();
    if requested_root_count == 0 {
        return Ok(Vec::new());
    }

    let effective_concurrency = effective_parallel_workers(requested_root_count, max_concurrency);
    let assignments = build_worker_assignments(labels, effective_concurrency);
    let inline_executor = InlineExecutor;
    let threaded_executor = ThreadedExecutor;
    force_selected_roots_with_executors(worker_input, &assignments, policy, &inline_executor, &threaded_executor)
}

fn force_selected_roots_with_executors<T: DeserializeOwned + Send + 'static>(
    worker_input: &IsolatedWorkerInput,
    assignments: &[WorkerAssignment],
    policy: RootForceExecutionPolicy,
    inline_executor: &impl AssignmentExecutor,
    threaded_executor: &impl AssignmentExecutor,
) -> Result<Vec<(String, T)>, Error> {
    let inline_results = || inline_executor.execute(worker_input, assignments);
    let assignment_results = match resolve_execution_backend(policy, assignments.len() as u32) {
        ResolvedExecutionBackend::Inline => inline_results()?,
        ResolvedExecutionBackend::Threaded => match threaded_executor.execute(worker_input, assignments) {
            Ok(results) => results,
            Err(_) => inline_results()?,
        },
    };
    merge_assignment_results(assignment_results)
}

fn collect_threaded_results<T: Send + 'static>(
    worker_slots: Vec<Arc<WorkerResultSlot>>,
) -> Result<Vec<Result<Vec<IndexedRoot<T>>, WorkerFailure>>, Error> {
    let mut results = Vec::with_capacity(worker_slots.len());
    for worker_slot in worker_slots {
        let worker_result = worker_slot.take()?;
        results.push(worker_result);
    }
    Ok(results)
}

fn merge_assignment_results<T: Send + 'static>(
    assignment_results: Vec<Result<Vec<IndexedRoot<T>>, WorkerFailure>>,
) -> Result<Vec<(String, T)>, Error> {
    let mut indexed_results = Vec::new();
    for assignment_result in assignment_results {
        match assignment_result {
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ResolvedExecutionBackend {
    Inline,
    Threaded,
}

fn resolve_execution_backend(policy: RootForceExecutionPolicy, effective_concurrency: u32) -> ResolvedExecutionBackend {
    assert!(effective_concurrency >= 1, "effective_concurrency must be at least 1");
    match policy {
        RootForceExecutionPolicy::Inline => ResolvedExecutionBackend::Inline,
        RootForceExecutionPolicy::PreferThreaded => {
            if effective_concurrency <= 1 {
                return ResolvedExecutionBackend::Inline;
            }
            ResolvedExecutionBackend::Threaded
        }
    }
}

fn bounded_worker_pool(worker_count: u32) -> Result<Arc<BoundedWorkerPool>, Error> {
    static WORKER_POOLS: OnceLock<Mutex<BTreeMap<u32, Arc<BoundedWorkerPool>>>> = OnceLock::new();

    assert!(worker_count >= 1, "worker_count must be at least 1");
    let worker_pools = WORKER_POOLS.get_or_init(|| Mutex::new(BTreeMap::new()));
    let mut worker_pools = worker_pools
        .lock()
        .map_err(|_| Error::Boundary("bounded worker pool mutex was poisoned".to_string()))?;
    if let Some(worker_pool) = worker_pools.get(&worker_count) {
        return Ok(worker_pool.clone());
    }

    let mut workers = Vec::with_capacity(usize_from_u32(worker_count));
    for worker_index in 0..worker_count {
        let (sender, receiver) = mpsc::sync_channel::<BoxedWorkerJob>(1);
        let result_slot = Arc::new(WorkerResultSlot::new());
        std::thread::Builder::new()
            .name(format!("crunch-eval-worker-{worker_count}-{worker_index}"))
            .spawn(move || {
                while let Ok(job) = receiver.recv() {
                    job.run();
                }
            })
            .map_err(|err| Error::Boundary(format!("failed to spawn bounded worker thread: {err}")))?;
        workers.push(BoundedWorkerSlot { sender, result_slot });
    }

    assert_eq!(u32_from_usize(workers.len()), worker_count, "worker pool must create one worker slot per worker");
    let worker_pool = Arc::new(BoundedWorkerPool {
        workers,
        next_worker: AtomicU32::new(0),
    });
    worker_pools.insert(worker_count, worker_pool.clone());
    Ok(worker_pool)
}

fn effective_parallel_workers(requested_root_count: usize, max_concurrency: u32) -> u32 {
    if requested_root_count == 0 {
        return 1;
    }

    let concurrency_cap = normalize_concurrency_cap(max_concurrency, requested_root_count);
    clamp_parallel_workers(requested_root_count, concurrency_cap)
}

fn should_reuse_session_for_all_roots(
    requested_root_count: usize,
    effective_concurrency: u32,
    policy: RootForceExecutionPolicy,
) -> bool {
    if requested_root_count == 0 {
        return true;
    }

    if policy == RootForceExecutionPolicy::Inline {
        return true;
    }

    if requested_root_count > SESSION_REUSE_ALL_ROOTS_MAX_ROOTS {
        return false;
    }

    effective_concurrency <= SESSION_REUSE_ALL_ROOTS_MAX_WORKERS
}

fn clamp_parallel_workers(requested_root_count: usize, concurrency_cap: u32) -> u32 {
    assert!(requested_root_count >= 1, "requested_root_count must not be zero");
    assert!(concurrency_cap >= 1, "concurrency_cap must be at least 1");

    let max_workers_from_chunking = requested_root_count.div_ceil(MIN_ROOTS_PER_WORKER);
    let max_workers_from_chunking = max_workers_from_chunking.max(1);
    let max_workers_from_chunking = u32_from_usize(max_workers_from_chunking);
    concurrency_cap.min(max_workers_from_chunking)
}

fn build_worker_assignments(labels: &[String], concurrency_cap: u32) -> Vec<WorkerAssignment> {
    assert!(concurrency_cap >= 1, "concurrency_cap must be at least 1");
    assert!(labels.len() <= usize_limit_from_u32_max(), "label count must fit in u32");
    if labels.is_empty() {
        return Vec::new();
    }

    let chunk_len = labels.len().div_ceil(usize_from_u32(concurrency_cap));
    assert!(chunk_len >= 1, "chunk_len must be at least 1");

    let assignment_count = labels.len().div_ceil(chunk_len);
    let mut assignments = Vec::with_capacity(assignment_count);
    for (chunk_index, chunk_labels) in labels.chunks(chunk_len).enumerate() {
        let start_index = u32_from_usize(chunk_index.saturating_mul(chunk_len));
        assignments.push(WorkerAssignment {
            start_index,
            labels: chunk_labels.to_vec(),
        });
    }
    assignments
}

fn force_worker_assignment<T: DeserializeOwned + Send>(
    worker_input: &IsolatedWorkerInput,
    assignment: WorkerAssignment,
) -> Result<Vec<IndexedRoot<T>>, WorkerFailure> {
    assert!(!assignment.labels.is_empty(), "worker assignments must not be empty");
    assert!(
        u32::try_from(assignment.labels.len()).is_ok(),
        "worker assignment length must fit in u32"
    );

    let worker_labels =
        select_worker_labels(&worker_input.labels, &assignment.labels).map_err(|detail| WorkerFailure {
            index: assignment.start_index,
            label: assignment.labels.first().cloned().unwrap_or_else(|| "<none>".to_string()),
            detail,
        })?;
    let mut session = EvaluationSession::open_worker_source(
        worker_input.source.clone(),
        worker_input.import_paths.clone(),
        worker_input.source_name.clone(),
        worker_input.shape.clone(),
        worker_labels,
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

    assert_eq!(results.len(), assignment.labels.len(), "worker results must match requested labels");
    Ok(results)
}

fn select_worker_labels(all_labels: &[RootLabel], requested_labels: &[String]) -> Result<Vec<RootLabel>, String> {
    assert!(!requested_labels.is_empty(), "requested_labels must not be empty");
    let mut selected_labels = Vec::with_capacity(requested_labels.len());
    for requested_label in requested_labels {
        let root_label =
            all_labels.iter().find(|root_label| root_label.label == *requested_label).ok_or_else(|| {
                format!(
                    "no root with label '{requested_label}'; available roots: {}",
                    all_labels.iter().map(|root_label| root_label.label.as_str()).collect::<Vec<_>>().join(", ")
                )
            })?;
        selected_labels.push(root_label.clone());
    }
    Ok(selected_labels)
}

fn normalize_concurrency_cap(max_concurrency: u32, requested_root_count: usize) -> u32 {
    assert!(requested_root_count <= usize_limit_from_u32_max(), "requested root count must fit in u32");
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
        assert!(expr.as_array().is_some(), "array expressions must expose array payloads");
        return Ok(RootShape::Array);
    }

    let record = expr.as_record();
    let Some(record) = record else {
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
                .get(usize_from_u32(index))
                .ok_or_else(|| Error::Boundary(format!("array index {index} out of bounds")))
        }
        RootShape::Record => {
            let record = expr
                .as_record()
                .ok_or_else(|| Error::Boundary("expected record expression for record shape".to_string()))?;
            let (_key, value) = record
                .key_value_by_index(usize_from_u32(index))
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
            source: Arc::<str>::from(source.to_string()),
            import_paths: Arc::<[OsString]>::from(Vec::<OsString>::new()),
            source_name: Arc::<str>::from("<test>".to_string()),
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
    fn select_worker_labels_keeps_requested_subset_and_indexes() {
        let all_labels = vec![
            RootLabel {
                label: "alpha".to_string(),
                index: 0,
            },
            RootLabel {
                label: "beta".to_string(),
                index: 7,
            },
            RootLabel {
                label: "gamma".to_string(),
                index: 11,
            },
        ];
        let requested = vec!["gamma".to_string(), "alpha".to_string()];

        let selected = select_worker_labels(&all_labels, &requested).unwrap();

        assert_eq!(selected.len(), 2);
        assert_eq!(selected[0].label, "gamma");
        assert_eq!(selected[0].index, 11);
        assert_eq!(selected[1].label, "alpha");
        assert_eq!(selected[1].index, 0);
    }

    #[test]
    fn normalize_concurrency_cap_clamps_to_requested_roots() {
        assert_eq!(normalize_concurrency_cap(4, 2), 2);
        assert_eq!(normalize_concurrency_cap(2, 4), 2);
        assert_eq!(normalize_concurrency_cap(1, 4), 1);
    }

    #[test]
    fn clamp_parallel_workers_preserves_minimum_chunk_size() {
        assert_eq!(clamp_parallel_workers(1, 4), 1);
        assert_eq!(clamp_parallel_workers(8, 4), 1);
        assert_eq!(clamp_parallel_workers(16, 4), 2);
        assert_eq!(clamp_parallel_workers(24, 4), 3);
        assert_eq!(clamp_parallel_workers(32, 4), 4);
    }

    #[test]
    fn effective_parallel_workers_clamps_requested_concurrency() {
        assert_eq!(effective_parallel_workers(0, 4), 1);
        assert_eq!(effective_parallel_workers(1, 4), 1);
        assert_eq!(effective_parallel_workers(16, 4), 2);
        assert_eq!(effective_parallel_workers(16, 2), 2);
        assert_eq!(effective_parallel_workers(16, 1), 1);
    }

    #[test]
    fn small_all_root_sets_reuse_the_session_even_when_threading_is_allowed() {
        assert!(should_reuse_session_for_all_roots(1, 1, RootForceExecutionPolicy::PreferThreaded));
        assert!(should_reuse_session_for_all_roots(
            SESSION_REUSE_ALL_ROOTS_MAX_ROOTS,
            SESSION_REUSE_ALL_ROOTS_MAX_WORKERS,
            RootForceExecutionPolicy::PreferThreaded,
        ));
        assert!(!should_reuse_session_for_all_roots(
            SESSION_REUSE_ALL_ROOTS_MAX_ROOTS + 1,
            SESSION_REUSE_ALL_ROOTS_MAX_WORKERS,
            RootForceExecutionPolicy::PreferThreaded,
        ));
        assert!(!should_reuse_session_for_all_roots(
            SESSION_REUSE_ALL_ROOTS_MAX_ROOTS,
            SESSION_REUSE_ALL_ROOTS_MAX_WORKERS.saturating_add(1),
            RootForceExecutionPolicy::PreferThreaded,
        ));
        assert!(should_reuse_session_for_all_roots(
            SESSION_REUSE_ALL_ROOTS_MAX_ROOTS + 8,
            SESSION_REUSE_ALL_ROOTS_MAX_WORKERS.saturating_add(1),
            RootForceExecutionPolicy::Inline,
        ));
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
        let mut bounded_session = EvaluationSession::open_str(source, &[]).unwrap();

        let serial = serial_session.force_all_roots::<CrunchDerivation>().unwrap();
        let bounded = bounded_session.force_all_roots_bounded::<CrunchDerivation>(3).unwrap();

        assert_eq!(serial.len(), bounded.len());
        for (serial_item, bounded_item) in serial.iter().zip(bounded.iter()) {
            assert_eq!(serial_item.0, bounded_item.0);
            assert_eq!(serial_item.1.name, bounded_item.1.name);
            assert_eq!(serial_item.1.builder, bounded_item.1.builder);
        }
    }

    #[derive(Clone, Copy, Debug, Default)]
    struct FailingThreadedExecutor;

    impl AssignmentExecutor for FailingThreadedExecutor {
        fn execute<T: DeserializeOwned + Send + 'static>(
            &self,
            _worker_input: &IsolatedWorkerInput,
            _assignments: &[WorkerAssignment],
        ) -> Result<Vec<Result<Vec<IndexedRoot<T>>, WorkerFailure>>, Error> {
            Err(Error::Boundary("synthetic threaded executor unavailable".to_string()))
        }
    }

    #[test]
    fn resolve_execution_backend_uses_inline_for_single_worker_requests() {
        assert_eq!(resolve_execution_backend(RootForceExecutionPolicy::Inline, 3), ResolvedExecutionBackend::Inline);
        assert_eq!(
            resolve_execution_backend(RootForceExecutionPolicy::PreferThreaded, 1),
            ResolvedExecutionBackend::Inline
        );
        assert_eq!(
            resolve_execution_backend(RootForceExecutionPolicy::PreferThreaded, 2),
            ResolvedExecutionBackend::Threaded
        );
    }

    #[test]
    fn force_selected_roots_policy_inline_matches_prefer_threaded() {
        let source = r#"{
  alpha = { name = "alpha", builder = "/bin/sh" },
  beta = { name = "beta", builder = "/bin/sh" },
  gamma = { name = "gamma", builder = "/bin/sh" },
}"#;
        let session = EvaluationSession::open_str(source, &[]).unwrap();
        let labels = vec!["gamma".to_string(), "alpha".to_string(), "beta".to_string()];

        let inline = session
            .force_selected_roots_with_policy::<CrunchDerivation>(&labels, 3, RootForceExecutionPolicy::Inline)
            .unwrap();
        let threaded = session
            .force_selected_roots_with_policy::<CrunchDerivation>(&labels, 3, RootForceExecutionPolicy::PreferThreaded)
            .unwrap();

        assert_eq!(inline.len(), threaded.len());
        for (inline_item, threaded_item) in inline.iter().zip(threaded.iter()) {
            assert_eq!(inline_item.0, threaded_item.0);
            assert_eq!(inline_item.1.name, threaded_item.1.name);
            assert_eq!(inline_item.1.builder, threaded_item.1.builder);
        }
    }

    #[test]
    fn inline_executor_executes_assignments_without_threaded_backend_help() {
        let source = r#"{
  alpha = { name = "alpha", builder = "/bin/sh" },
  beta = { name = "beta", builder = "/bin/sh" },
}"#;
        let session = EvaluationSession::open_str(source, &[]).unwrap();
        let worker_input = session.isolated_worker_input();
        let labels = vec!["beta".to_string(), "alpha".to_string()];
        let assignments = build_worker_assignments(&labels, 2);
        let inline_executor = InlineExecutor;

        let assignment_results = inline_executor.execute::<CrunchDerivation>(&worker_input, &assignments).unwrap();
        let merged = merge_assignment_results(assignment_results).unwrap();
        let observed = merged.into_iter().map(|(label, _)| label).collect::<Vec<_>>();

        assert_eq!(observed, labels);
    }

    #[test]
    fn prefer_threaded_falls_back_to_inline_when_threaded_executor_is_unavailable() {
        let source = r#"{
  alpha = { name = "alpha", builder = "/bin/sh" },
  beta = { name = "beta", builder = "/bin/sh" },
}"#;
        let session = EvaluationSession::open_str(source, &[]).unwrap();
        let worker_input = session.isolated_worker_input();
        let labels = vec!["beta".to_string(), "alpha".to_string()];
        let assignments = build_worker_assignments(&labels, 2);
        let inline_executor = InlineExecutor;
        let fallback = force_selected_roots_with_executors::<CrunchDerivation>(
            &worker_input,
            &assignments,
            RootForceExecutionPolicy::PreferThreaded,
            &inline_executor,
            &FailingThreadedExecutor,
        )
        .unwrap();

        let observed = fallback.into_iter().map(|(label, _)| label).collect::<Vec<_>>();
        assert_eq!(observed, labels);
    }
}
