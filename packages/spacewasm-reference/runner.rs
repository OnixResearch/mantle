use std::alloc::Layout;
use std::fs;
use std::io::Write;
use std::path::Path;
use std::ptr::NonNull;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;

use spacewasm::AllocError;
use spacewasm::Allocator;
use spacewasm::CodeBuilder;
use spacewasm::CompilerOptions;
use spacewasm::ExportDesc;
use spacewasm::InnerVec;
use spacewasm::Interpreter;
use spacewasm::InterpreterResult;
use spacewasm::InterpreterRunner;
use spacewasm::MemoryStatistics;
use spacewasm::Module;
use spacewasm::ModuleRef;
use spacewasm::Rc;
use spacewasm::Ref;
use spacewasm::Store;
use spacewasm::TrapReason;
use spacewasm::WasmMemoryAllocator;
use spacewasm::WasmRef;
use spacewasm::WasmStream;

const SOURCE_REVISION: &str = "e24cf09355a90497148eb5029fdb8e3400bd63e3";
const REPORT_SCHEMA: &str = "mantle-spacewasm-runner-report-v1";
const MAX_CODE_PAGES: usize = 64;
const MAX_CONTROL_FRAMES: usize = 64;
const MAX_STACK_WORDS: usize = 1_024;
const DEFAULT_FUEL: usize = 1_024;
const OUT_OF_FUEL_BUDGET: usize = 1;
const STREAM_CHUNK_BYTES: usize = 1;
const SINGLE_CHUNK_BYTES: usize = usize::MAX;
const MAX_MODULES: usize = 1;

static FAIL_SPACEWASM_ALLOCATIONS: AtomicBool = AtomicBool::new(false);

#[derive(Clone, Copy)]
struct RunnerAllocator;

unsafe impl Allocator for RunnerAllocator {
    unsafe fn alloc(&self, layout: Layout) -> Result<*mut u8, AllocError> {
        if FAIL_SPACEWASM_ALLOCATIONS.load(Ordering::SeqCst) {
            return Err(AllocError::AllocationFailed);
        }
        let pointer = unsafe { std::alloc::alloc(layout) };
        if pointer.is_null() {
            return Err(AllocError::AllocationFailed);
        }
        Ok(pointer)
    }

    unsafe fn dealloc(&self, pointer: *mut u8, layout: Layout) {
        if !pointer.is_null() {
            unsafe { std::alloc::dealloc(pointer, layout) };
        }
    }

    fn memory_statistics(&self) -> MemoryStatistics {
        MemoryStatistics {
            total_bytes: 0,
            pad_bytes: 0,
        }
    }
}

impl WasmMemoryAllocator for RunnerAllocator {
    fn allocate(&self, layout: Layout) -> Result<NonNull<u8>, AllocError> {
        if FAIL_SPACEWASM_ALLOCATIONS.load(Ordering::SeqCst) {
            return Err(AllocError::AllocationFailed);
        }
        unsafe { NonNull::new(std::alloc::alloc(layout)).ok_or(AllocError::AllocationFailed) }
    }

    fn reallocate(
        &self,
        pointer: NonNull<u8>,
        old_layout: Layout,
        layout: Layout,
    ) -> Result<NonNull<u8>, AllocError> {
        if FAIL_SPACEWASM_ALLOCATIONS.load(Ordering::SeqCst) {
            return Err(AllocError::AllocationFailed);
        }
        unsafe {
            NonNull::new(std::alloc::realloc(pointer.as_ptr(), old_layout, layout.size()))
                .ok_or(AllocError::AllocationFailed)
        }
    }

    fn deallocate(&self, pointer: NonNull<u8>, layout: Layout) {
        unsafe { std::alloc::dealloc(pointer.as_ptr(), layout) };
    }
}

spacewasm::global_allocator!(RunnerAllocator, RunnerAllocator);

struct ChunkStream {
    chunks: Vec<Vec<u8>>,
    next: usize,
}

impl ChunkStream {
    fn new(bytes: &[u8], chunk_bytes: usize) -> Self {
        let chunk_bytes = chunk_bytes.max(STREAM_CHUNK_BYTES);
        let chunks = bytes.chunks(chunk_bytes).map(<[u8]>::to_vec).collect();
        Self { chunks, next: 0 }
    }
}

impl WasmStream for ChunkStream {
    fn read(&mut self) -> Result<Option<InnerVec<u8>>, u8> {
        let Some(chunk) = self.chunks.get_mut(self.next) else {
            return Ok(None);
        };
        self.next = self.next.saturating_add(1);
        Ok(Some(InnerVec {
            ptr: chunk.as_mut_ptr(),
            capacity: u32::try_from(chunk.len()).unwrap_or(u32::MAX),
            len: u32::try_from(chunk.len()).unwrap_or(u32::MAX),
        }))
    }

    fn return_(&mut self, _chunk: InnerVec<u8>) {}
}

#[derive(Debug)]
struct FixtureResult {
    fixture_id: &'static str,
    status: &'static str,
    result_code: &'static str,
}

fn main() {
    let arguments: Vec<String> = std::env::args().collect();
    if arguments.len() != 3 {
        eprintln!("usage: mantle-spacewasm-diagnostic-runner <fixture-dir> <report.json>");
        std::process::exit(2);
    }
    let fixture_root = Path::new(&arguments[1]);
    let report_path = Path::new(&arguments[2]);
    let results = run_all(fixture_root);
    let all_passed = results.iter().all(|result| result.status == "passed");
    let report = render_report(&results);
    let mut output = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(report_path)
        .expect("create runner report");
    output.write_all(report.as_bytes()).expect("write runner report");
    if !all_passed {
        std::process::exit(1);
    }
}

fn run_all(root: &Path) -> Vec<FixtureResult> {
    vec![
        expect_execution(root, "mvp-positive", "mvp-positive.wasm", ExecutionExpectation::Finished),
        expect_decode_error(root, "mvp-negative", "mvp-negative.wasm", SINGLE_CHUNK_BYTES, "malformed-magic"),
        expect_execution_with_chunks(
            root,
            "streaming-positive",
            "streaming-positive.wasm",
            STREAM_CHUNK_BYTES,
            ExecutionExpectation::Finished,
            "stream-finished",
        ),
        expect_decode_error(
            root,
            "streaming-negative",
            "streaming-negative.wasm",
            STREAM_CHUNK_BYTES,
            "stream-unexpected-eof",
        ),
        expect_allocation_failure(root),
        expect_decode_error(
            root,
            "unsupported-feature",
            "unsupported-bulk-memory.wasm",
            SINGLE_CHUNK_BYTES,
            "unsupported-bulk-memory",
        ),
        expect_execution(root, "trap", "trap-unreachable.wasm", ExecutionExpectation::TrapUnreachable),
        expect_execution(root, "out-of-fuel", "out-of-fuel.wasm", ExecutionExpectation::OutOfFuel),
    ]
}

#[derive(Clone, Copy)]
enum ExecutionExpectation {
    Finished,
    TrapUnreachable,
    OutOfFuel,
}

fn expect_execution(
    root: &Path,
    fixture_id: &'static str,
    file_name: &str,
    expectation: ExecutionExpectation,
) -> FixtureResult {
    expect_execution_with_chunks(root, fixture_id, file_name, SINGLE_CHUNK_BYTES, expectation, expectation_code(expectation))
}

fn expect_execution_with_chunks(
    root: &Path,
    fixture_id: &'static str,
    file_name: &str,
    chunk_bytes: usize,
    expectation: ExecutionExpectation,
    expected_code: &'static str,
) -> FixtureResult {
    let bytes = read_fixture(root, file_name);
    let result = decode_and_run(&bytes, chunk_bytes, expectation).map(|_| expected_code);
    fixture_result(fixture_id, result, expected_code)
}

fn decode_and_run(
    bytes: &[u8],
    chunk_bytes: usize,
    expectation: ExecutionExpectation,
) -> Result<&'static str, &'static str> {
    FAIL_SPACEWASM_ALLOCATIONS.store(false, Ordering::SeqCst);
    let mut stream = ChunkStream::new(bytes, chunk_bytes);
    let mut store = Store::new(MAX_MODULES, []).map_err(|_| "store-allocation-failed")?;
    let mut code_builder = CodeBuilder::<MAX_CODE_PAGES>::default();
    let allocator = Rc::new(RunnerAllocator)
        .map_err(|_| "allocator-handle-failed")?
        .into_wasm_memory_allocator();
    let module = Module::new::<MAX_CODE_PAGES, MAX_CONTROL_FRAMES, MAX_STACK_WORDS>(
        "fixture",
        &mut stream,
        &mut store,
        &mut code_builder,
        allocator,
        CompilerOptions::default(),
    )
    .map_err(|_| "decode-failed")?;
    let (text, _) = code_builder.finish().map_err(|_| "code-finalize-failed")?;
    let mut state = store.allocate(MAX_STACK_WORDS).map_err(|_| "state-allocation-failed")?;
    match state.initialize_module(module, &text, DEFAULT_FUEL) {
        InterpreterResult::Finished => {}
        _ => return Err("initialize-failed"),
    }
    let function = {
        let module = state.store.modules().last().ok_or("module-missing")?;
        let export = module.exports.iter().find(|export| export.name == "run").ok_or("run-export-missing")?;
        let ExportDesc::Func(index) = export.desc else {
            return Err("run-export-not-function");
        };
        let Ref::Module(index) = module.get_func_ref(index).ok_or("run-function-missing")? else {
            return Err("run-function-not-local");
        };
        WasmRef {
            module: ModuleRef(0),
            index,
        }
    };
    state.invoke(function, &[]).map_err(|_| "invoke-failed")?;
    let fuel = match expectation {
        ExecutionExpectation::OutOfFuel => OUT_OF_FUEL_BUDGET,
        _ => DEFAULT_FUEL,
    };
    let result = Interpreter::default().run(&text, &mut state, fuel);
    match (expectation, result) {
        (ExecutionExpectation::Finished, InterpreterResult::Finished) => Ok("finished"),
        (ExecutionExpectation::TrapUnreachable, InterpreterResult::Trap(TrapReason::Unreachable)) => Ok("trap-unreachable"),
        (ExecutionExpectation::OutOfFuel, InterpreterResult::OutOfFuel) => Ok("out-of-fuel"),
        _ => Err("unexpected-execution-result"),
    }
}

fn expect_decode_error(
    root: &Path,
    fixture_id: &'static str,
    file_name: &str,
    chunk_bytes: usize,
    expected_code: &'static str,
) -> FixtureResult {
    let bytes = read_fixture(root, file_name);
    let result = decode_only(&bytes, chunk_bytes).map_or(Ok(expected_code), |_| Err("unexpected-decode-success"));
    fixture_result(fixture_id, result, expected_code)
}

fn decode_only(bytes: &[u8], chunk_bytes: usize) -> Result<(), ()> {
    FAIL_SPACEWASM_ALLOCATIONS.store(false, Ordering::SeqCst);
    let mut stream = ChunkStream::new(bytes, chunk_bytes);
    let mut store = Store::new(MAX_MODULES, []).map_err(|_| ())?;
    let mut code_builder = CodeBuilder::<MAX_CODE_PAGES>::default();
    let allocator = Rc::new(RunnerAllocator).map_err(|_| ())?.into_wasm_memory_allocator();
    Module::new::<MAX_CODE_PAGES, MAX_CONTROL_FRAMES, MAX_STACK_WORDS>(
        "fixture",
        &mut stream,
        &mut store,
        &mut code_builder,
        allocator,
        CompilerOptions::default(),
    )
    .map(|_| ())
    .map_err(|_| ())
}

fn expect_allocation_failure(root: &Path) -> FixtureResult {
    let bytes = read_fixture(root, "allocation-failure.wasm");
    let result = std::panic::catch_unwind(|| {
        FAIL_SPACEWASM_ALLOCATIONS.store(false, Ordering::SeqCst);
        let mut stream = ChunkStream::new(&bytes, SINGLE_CHUNK_BYTES);
        let mut store = Store::new(MAX_MODULES, []).map_err(|_| ())?;
        let mut code_builder = CodeBuilder::<MAX_CODE_PAGES>::default();
        let allocator = Rc::new(RunnerAllocator).map_err(|_| ())?.into_wasm_memory_allocator();
        FAIL_SPACEWASM_ALLOCATIONS.store(true, Ordering::SeqCst);
        let decoded = Module::new::<MAX_CODE_PAGES, MAX_CONTROL_FRAMES, MAX_STACK_WORDS>(
            "fixture",
            &mut stream,
            &mut store,
            &mut code_builder,
            allocator,
            CompilerOptions::default(),
        );
        FAIL_SPACEWASM_ALLOCATIONS.store(false, Ordering::SeqCst);
        if decoded.is_err() { Ok(()) } else { Err(()) }
    });
    FAIL_SPACEWASM_ALLOCATIONS.store(false, Ordering::SeqCst);
    let observed = match result {
        Ok(Ok(())) => Ok("allocation-failed-without-panic"),
        Ok(Err(())) => Err("allocation-unexpectedly-succeeded"),
        Err(_) => Err("allocation-panicked"),
    };
    fixture_result("allocation-failure", observed, "allocation-failed-without-panic")
}

fn read_fixture(root: &Path, file_name: &str) -> Vec<u8> {
    fs::read(root.join(file_name)).unwrap_or_else(|error| panic!("read fixture {file_name}: {error}"))
}

fn expectation_code(expectation: ExecutionExpectation) -> &'static str {
    match expectation {
        ExecutionExpectation::Finished => "finished",
        ExecutionExpectation::TrapUnreachable => "trap-unreachable",
        ExecutionExpectation::OutOfFuel => "out-of-fuel",
    }
}

fn fixture_result(
    fixture_id: &'static str,
    result: Result<&'static str, &'static str>,
    expected_code: &'static str,
) -> FixtureResult {
    match result {
        Ok(code) if code == expected_code => FixtureResult {
            fixture_id,
            status: "passed",
            result_code: code,
        },
        Ok(_) => FixtureResult {
            fixture_id,
            status: "failed",
            result_code: "wrong-result-code",
        },
        Err(code) => FixtureResult {
            fixture_id,
            status: "failed",
            result_code: code,
        },
    }
}

fn render_report(results: &[FixtureResult]) -> String {
    let rows = results
        .iter()
        .map(|result| {
            format!(
                "{{\"fixture_id\":\"{}\",\"result_code\":\"{}\",\"status\":\"{}\"}}",
                result.fixture_id, result.result_code, result.status
            )
        })
        .collect::<Vec<_>>()
        .join(",");
    format!(
        "{{\"results\":[{rows}],\"schema\":\"{REPORT_SCHEMA}\",\"source_revision\":\"{SOURCE_REVISION}\"}}\n"
    )
}
