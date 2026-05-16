use std::collections::HashMap;
use std::ffi::OsString;
use std::fs;
use std::path::Path;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::AtomicU64;
use std::sync::atomic::Ordering;
use std::thread;
use std::thread::JoinHandle;
use std::time::Duration;

use serde_json::Value;
use tokio::runtime::Builder as RuntimeBuilder;
use tokio::sync::mpsc;
use tokio::sync::oneshot;

use crate::NickelValue;
use crate::eval_trait::EvalError;
use crate::eval_trait::EvalOptions;

const CHANNEL_CAPACITY: usize = 32;
const DEFAULT_TIMEOUT_MS: u64 = 2_000;
const VALUE_ID_START: u64 = 1;

const _: () = assert!(CHANNEL_CAPACITY > 0, "channel capacity must be positive");
const _: () = assert!(VALUE_ID_START > 0, "value id start must be positive");

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ValueId(pub u64);

#[derive(Debug)]
pub enum EvalRequest {
    EvaluateFile {
        path: PathBuf,
        options: EvalOptions,
    },
    GetField {
        value_id: ValueId,
        field_name: String,
        options: EvalOptions,
    },
    IsFunction {
        value_id: ValueId,
        options: EvalOptions,
    },
    Merge {
        left_id: ValueId,
        right_id: ValueId,
        options: EvalOptions,
    },
    Call {
        function_id: ValueId,
        argument_id: ValueId,
        options: EvalOptions,
    },
    ToJson {
        value_id: ValueId,
        options: EvalOptions,
    },
    DropValue {
        value_id: ValueId,
    },
    Shutdown,
}

#[derive(Debug)]
pub enum EvalResponse {
    Value(ValueId),
    Bool(bool),
    Json(Value),
    Unit,
}

#[derive(Debug)]
struct EvalEnvelope {
    request: EvalRequest,
    response_tx: oneshot::Sender<Result<EvalResponse, EvalError>>,
}

pub struct EvalThread;

pub struct EvalThreadHandle {
    request_tx: mpsc::Sender<EvalEnvelope>,
    timeout_default: Duration,
    join_handle: Arc<std::sync::Mutex<Option<JoinHandle<()>>>>,
}

struct OnThreadEvaluator {
    import_paths: Vec<PathBuf>,
    next_value_id: AtomicU64,
    values: HashMap<ValueId, NickelValue>,
}

impl EvalThread {
    pub fn spawn(import_paths: Vec<PathBuf>) -> EvalThreadHandle {
        let (request_tx, mut request_rx) = mpsc::channel::<EvalEnvelope>(CHANNEL_CAPACITY);
        let join_handle = thread::spawn(move || {
            let mut evaluator = OnThreadEvaluator::new(import_paths);
            let runtime = RuntimeBuilder::new_current_thread().enable_all().build().expect("runtime must build");
            runtime.block_on(async move {
                while let Some(envelope) = request_rx.recv().await {
                    let is_shutdown = matches!(envelope.request, EvalRequest::Shutdown);
                    let result = evaluator.handle_request(envelope.request);
                    let _ = envelope.response_tx.send(result);
                    if is_shutdown {
                        break;
                    }
                }
            });
        });
        let join_handle = Arc::new(std::sync::Mutex::new(Some(join_handle)));
        EvalThreadHandle {
            request_tx,
            timeout_default: Duration::from_millis(DEFAULT_TIMEOUT_MS),
            join_handle,
        }
    }
}

impl EvalThreadHandle {
    pub async fn evaluate_file(&self, path: PathBuf, options: EvalOptions) -> Result<ValueId, EvalError> {
        match self.send_request(EvalRequest::EvaluateFile { path, options }).await? {
            EvalResponse::Value(value_id) => Ok(value_id),
            _ => Err(EvalError::NickelError("unexpected response type for evaluate_file".to_string())),
        }
    }

    pub async fn get_field(
        &self,
        value_id: ValueId,
        field_name: String,
        options: EvalOptions,
    ) -> Result<ValueId, EvalError> {
        match self
            .send_request(EvalRequest::GetField {
                value_id,
                field_name,
                options,
            })
            .await?
        {
            EvalResponse::Value(next_value_id) => Ok(next_value_id),
            _ => Err(EvalError::NickelError("unexpected response type for get_field".to_string())),
        }
    }

    pub async fn is_function(&self, value_id: ValueId, options: EvalOptions) -> Result<bool, EvalError> {
        match self.send_request(EvalRequest::IsFunction { value_id, options }).await? {
            EvalResponse::Bool(value) => Ok(value),
            _ => Err(EvalError::NickelError("unexpected response type for is_function".to_string())),
        }
    }

    pub async fn merge(&self, left_id: ValueId, right_id: ValueId, options: EvalOptions) -> Result<ValueId, EvalError> {
        match self
            .send_request(EvalRequest::Merge {
                left_id,
                right_id,
                options,
            })
            .await?
        {
            EvalResponse::Value(value_id) => Ok(value_id),
            _ => Err(EvalError::NickelError("unexpected response type for merge".to_string())),
        }
    }

    pub async fn call(
        &self,
        function_id: ValueId,
        argument_id: ValueId,
        options: EvalOptions,
    ) -> Result<ValueId, EvalError> {
        match self
            .send_request(EvalRequest::Call {
                function_id,
                argument_id,
                options,
            })
            .await?
        {
            EvalResponse::Value(value_id) => Ok(value_id),
            _ => Err(EvalError::NickelError("unexpected response type for call".to_string())),
        }
    }

    pub async fn to_json(&self, value_id: ValueId, options: EvalOptions) -> Result<Value, EvalError> {
        match self.send_request(EvalRequest::ToJson { value_id, options }).await? {
            EvalResponse::Json(value) => Ok(value),
            _ => Err(EvalError::NickelError("unexpected response type for to_json".to_string())),
        }
    }

    pub async fn drop_value(&self, value_id: ValueId) -> Result<(), EvalError> {
        match self.send_request(EvalRequest::DropValue { value_id }).await? {
            EvalResponse::Unit => Ok(()),
            _ => Err(EvalError::NickelError("unexpected response type for drop_value".to_string())),
        }
    }

    pub async fn shutdown(&self) -> Result<(), EvalError> {
        match self.send_request(EvalRequest::Shutdown).await? {
            EvalResponse::Unit => {
                let join_handle = self.join_handle.lock().expect("join handle mutex must lock").take();
                if let Some(handle) = join_handle {
                    handle
                        .join()
                        .map_err(|_| EvalError::NickelError("eval thread panicked during shutdown".to_string()))?;
                }
                Ok(())
            }
            _ => Err(EvalError::NickelError("unexpected response type for shutdown".to_string())),
        }
    }

    async fn send_request(&self, request: EvalRequest) -> Result<EvalResponse, EvalError> {
        let timeout = request_timeout(&request, self.timeout_default);
        let (response_tx, response_rx) = oneshot::channel();
        self.request_tx
            .send(EvalEnvelope { request, response_tx })
            .await
            .map_err(|_| EvalError::NickelError("eval thread is not available".to_string()))?;
        tokio::time::timeout(timeout, response_rx)
            .await
            .map_err(|_| EvalError::Timeout)?
            .map_err(|_| EvalError::NickelError("eval thread dropped response channel".to_string()))?
    }
}

impl OnThreadEvaluator {
    fn new(import_paths: Vec<PathBuf>) -> Self {
        Self {
            import_paths,
            next_value_id: AtomicU64::new(VALUE_ID_START),
            values: HashMap::new(),
        }
    }

    fn handle_request(&mut self, request: EvalRequest) -> Result<EvalResponse, EvalError> {
        match request {
            EvalRequest::EvaluateFile { path, options } => {
                let expr = self.evaluate_file(&path, &options)?;
                let value_id = self.insert_value(expr);
                Ok(EvalResponse::Value(value_id))
            }
            EvalRequest::GetField {
                value_id,
                field_name,
                options: _,
            } => {
                let value = self.require_value(value_id)?;
                let record = value
                    .as_record()
                    .ok_or_else(|| EvalError::NickelError(format!("value {value_id:?} is not a record")))?;
                let field = record
                    .value_by_name(&field_name)
                    .ok_or_else(|| EvalError::NickelError(format!("missing field '{field_name}'")))?
                    .clone();
                let next_value_id = self.insert_value(field);
                Ok(EvalResponse::Value(next_value_id))
            }
            EvalRequest::IsFunction { value_id, options } => {
                let value = self.require_value(value_id)?;
                let json_attempt = value.to_serde::<Value>();
                let is_function = json_attempt.is_err() && options.import_paths.is_empty();
                Ok(EvalResponse::Bool(is_function))
            }
            EvalRequest::Merge {
                left_id: _,
                right_id: _,
                options: _,
            } => Err(EvalError::NickelError("merge not implemented yet".to_string())),
            EvalRequest::Call {
                function_id: _,
                argument_id: _,
                options: _,
            } => Err(EvalError::NickelError("call not implemented yet".to_string())),
            EvalRequest::ToJson { value_id, options: _ } => {
                let value = self.require_value(value_id)?;
                let json = value
                    .to_serde::<Value>()
                    .map_err(|err| EvalError::NickelError(format!("serializing value to json: {err}")))?;
                Ok(EvalResponse::Json(json))
            }
            EvalRequest::DropValue { value_id } => {
                self.values.remove(&value_id);
                Ok(EvalResponse::Unit)
            }
            EvalRequest::Shutdown => Ok(EvalResponse::Unit),
        }
    }

    fn evaluate_file(&self, path: &Path, options: &EvalOptions) -> Result<NickelValue, EvalError> {
        let source = fs::read_to_string(path)
            .map_err(|err| EvalError::NickelError(format!("reading {}: {err}", path.display())))?;
        let import_paths = build_import_paths(path, options, &self.import_paths)?;
        let os_import_paths: Vec<OsString> =
            import_paths.into_iter().map(|path_buf| path_buf.into_os_string()).collect();
        crunch_eval::evaluate_str(&source, &os_import_paths).map_err(|err| map_eval_error(err, path))
    }

    fn insert_value(&mut self, value: NickelValue) -> ValueId {
        let raw_value_id = self.next_value_id.fetch_add(1, Ordering::Relaxed);
        let value_id = ValueId(raw_value_id);
        let prior = self.values.insert(value_id, value);
        assert!(prior.is_none(), "value id must be unique: {raw_value_id}");
        value_id
    }

    fn require_value(&self, value_id: ValueId) -> Result<NickelValue, EvalError> {
        self.values
            .get(&value_id)
            .cloned()
            .ok_or_else(|| EvalError::NickelError(format!("unknown value id {}", value_id.0)))
    }
}

fn request_timeout(request: &EvalRequest, timeout_default: Duration) -> Duration {
    assert!(timeout_default.as_millis() > 0, "default timeout must be positive");
    match request {
        EvalRequest::EvaluateFile { options, .. }
        | EvalRequest::GetField { options, .. }
        | EvalRequest::IsFunction { options, .. }
        | EvalRequest::Merge { options, .. }
        | EvalRequest::Call { options, .. }
        | EvalRequest::ToJson { options, .. } => options.timeout.unwrap_or(timeout_default),
        EvalRequest::DropValue { .. } | EvalRequest::Shutdown => timeout_default,
    }
}

fn build_import_paths(
    path: &Path,
    options: &EvalOptions,
    shared_import_paths: &[PathBuf],
) -> Result<Vec<PathBuf>, EvalError> {
    let mut import_paths = Vec::new();
    if let Some(parent) = path.parent() {
        import_paths.push(parent.to_path_buf());
    }
    import_paths.extend(shared_import_paths.iter().cloned());
    import_paths.extend(options.import_paths.iter().cloned());
    if has_denied_import_path(&import_paths) {
        return Err(EvalError::ImportDenied);
    }
    Ok(import_paths)
}

fn has_denied_import_path(import_paths: &[PathBuf]) -> bool {
    const DENIED_SEGMENT: &str = "..";
    import_paths
        .iter()
        .any(|path| path.components().any(|component| component.as_os_str() == DENIED_SEGMENT))
}

fn map_eval_error(error: crunch_eval::Error, path: &Path) -> EvalError {
    let message = error.to_string();
    if message.contains("import") {
        return EvalError::ImportDenied;
    }
    EvalError::NickelError(format!("{}: {message}", path.display()))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn import_paths() -> Vec<PathBuf> {
        vec![crunch_eval::stdlib::stdlib_import_path().unwrap()]
    }

    fn eval_options() -> EvalOptions {
        EvalOptions {
            timeout: Some(Duration::from_secs(5)),
            import_paths: Vec::new(),
        }
    }

    async fn write_module(dir: &Path, name: &str, body: &str) -> PathBuf {
        let path = dir.join(name);
        tokio::fs::write(&path, body).await.unwrap();
        path
    }

    #[tokio::test(flavor = "current_thread")]
    async fn spawn_eval_thread_and_evaluate_simple_file() {
        let dir = tempfile::tempdir().unwrap();
        let path = write_module(dir.path(), "simple.ncl", "{ value = 42 }").await;
        let handle = EvalThread::spawn(import_paths());

        let value_id = handle.evaluate_file(path, eval_options()).await.unwrap();
        let json = handle.to_json(value_id, eval_options()).await.unwrap();

        assert_eq!(json["value"], 42.0);
        assert!(json.is_object(), "json result should stay a record");
        handle.shutdown().await.unwrap();
    }

    #[tokio::test(flavor = "current_thread")]
    async fn eval_thread_get_field_extracts_record_field() {
        let dir = tempfile::tempdir().unwrap();
        let path = write_module(dir.path(), "record.ncl", "{ alpha = { nested = 7 } }").await;
        let handle = EvalThread::spawn(import_paths());

        let root_id = handle.evaluate_file(path, eval_options()).await.unwrap();
        let alpha_id = handle.get_field(root_id, "alpha".to_string(), eval_options()).await.unwrap();
        let json = handle.to_json(alpha_id, eval_options()).await.unwrap();

        assert_eq!(json["nested"], 7.0);
        handle.shutdown().await.unwrap();
    }

    #[tokio::test(flavor = "current_thread")]
    async fn eval_thread_is_function_distinguishes_functions_from_records() {
        let dir = tempfile::tempdir().unwrap();
        let path =
            write_module(dir.path(), "kinds.ncl", "{ handler = (fun x => x), record_value = { done = true } }").await;
        let handle = EvalThread::spawn(import_paths());

        let root_id = handle.evaluate_file(path, eval_options()).await.unwrap();
        let fun_id = handle.get_field(root_id, "handler".to_string(), eval_options()).await.unwrap();
        let rec_id = handle.get_field(root_id, "record_value".to_string(), eval_options()).await.unwrap();

        assert!(handle.is_function(fun_id, eval_options()).await.unwrap());
        assert!(!handle.is_function(rec_id, eval_options()).await.unwrap());
        handle.shutdown().await.unwrap();
    }

    #[tokio::test(flavor = "current_thread")]
    async fn timeout_enforcement_returns_timeout() {
        let dir = tempfile::tempdir().unwrap();
        let path = write_module(dir.path(), "timeout.ncl", "{ slow = 1 }").await;
        let (request_tx, _request_rx) = mpsc::channel::<EvalEnvelope>(1);
        let handle = EvalThreadHandle {
            request_tx,
            timeout_default: Duration::from_millis(1),
            join_handle: Arc::new(std::sync::Mutex::new(None)),
        };
        let options = EvalOptions {
            timeout: Some(Duration::from_millis(1)),
            import_paths: Vec::new(),
        };

        let result = handle.evaluate_file(path, options).await;

        assert_eq!(result.err(), Some(EvalError::Timeout));
    }

    #[tokio::test(flavor = "current_thread")]
    async fn shutdown_joins_cleanly() {
        let handle = EvalThread::spawn(import_paths());
        let result = handle.shutdown().await;
        assert!(result.is_ok());
    }

    #[tokio::test(flavor = "current_thread")]
    async fn drop_value_frees_table_entry() {
        let dir = tempfile::tempdir().unwrap();
        let path = write_module(dir.path(), "drop.ncl", "{ alpha = 1 }").await;
        let handle = EvalThread::spawn(import_paths());

        let value_id = handle.evaluate_file(path, eval_options()).await.unwrap();
        handle.drop_value(value_id).await.unwrap();
        let result = handle.to_json(value_id, eval_options()).await;

        assert!(matches!(result, Err(EvalError::NickelError(message)) if message.contains("unknown value id")));
        handle.shutdown().await.unwrap();
    }

    #[tokio::test(flavor = "current_thread")]
    async fn eval_thread_panic_returns_error_after_thread_drop() {
        let (request_tx, request_rx) = mpsc::channel::<EvalEnvelope>(1);
        drop(request_rx);
        let handle = EvalThreadHandle {
            request_tx,
            timeout_default: Duration::from_secs(1),
            join_handle: Arc::new(std::sync::Mutex::new(None)),
        };

        let result = handle.shutdown().await;

        assert!(matches!(result, Err(EvalError::NickelError(message)) if message.contains("not available")));
    }

    #[tokio::test(flavor = "current_thread")]
    async fn import_sandboxing_rejects_parent_path() {
        let dir = tempfile::tempdir().unwrap();
        let path = write_module(dir.path(), "sandbox.ncl", "{ ok = true }").await;
        let handle = EvalThread::spawn(import_paths());
        let options = EvalOptions {
            timeout: Some(Duration::from_secs(1)),
            import_paths: vec![PathBuf::from("../escape")],
        };

        let result = handle.evaluate_file(path, options).await;

        assert_eq!(result.err(), Some(EvalError::ImportDenied));
        handle.shutdown().await.unwrap();
    }
}
