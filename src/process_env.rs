//! Test-only lock for the shared process environment.
//!
//! Tests that mutate `PATH` or other process-wide variables must hold this
//! lock, and so must tests that spawn children which resolve utilities from the
//! environment. The lock crosses module boundaries on purpose: a poisoning test
//! in one module must not break a spawning test in another.

use std::sync::Mutex;
use std::sync::MutexGuard;
use std::sync::OnceLock;

/// Acquire the process-environment lock for the current test.
pub(crate) fn lock_process_env() -> MutexGuard<'static, ()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    let mutex = LOCK.get_or_init(|| Mutex::new(()));
    let guard = match mutex.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    };
    debug_assert!(LOCK.get().is_some());
    debug_assert!(std::ptr::eq(mutex as *const _, LOCK.get().expect("initialized") as *const _));
    guard
}
