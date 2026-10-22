//! Thread-scoped ownership of the test-only fault injection counter.
//!
//! The pinned `fault-injection` crate keeps one process-global counter that
//! every fault site decrements, and one process-global trigger function. A
//! test that reaches a fault site while a fixture is armed therefore consumes
//! the counter the fixture is waiting for, so the injected fault lands in the
//! wrong test and the fixture observes a site it never asked for.
//!
//! This module gives the counter to the fixture's own thread. Only the armed
//! thread reaches the counter; every other thread runs the original I/O
//! expression and annotates a real error exactly as the injector would, so it
//! neither consumes the counter nor changes the observable error text. The
//! scope is test-only, so production never observes it and the vendored crate
//! keeps its process-global contract.

use std::cell::Cell;

thread_local! {
    static ARMED: Cell<bool> = const { Cell::new(false) };
}

/// Whether the calling thread owns the armed fault counter.
pub(crate) fn armed() -> bool {
    ARMED.with(Cell::get)
}

/// Give the calling thread ownership of the armed fault counter.
pub(crate) fn arm() {
    ARMED.with(|flag| flag.set(true));
}

/// Release the calling thread's ownership of the armed fault counter.
pub(crate) fn disarm() {
    ARMED.with(|flag| flag.set(false));
}

/// Run an I/O expression through the pinned fault injector when the calling
/// thread owns the armed counter, and through the original expression
/// otherwise.
///
/// The armed branch keeps the injected annotation contract: the crate, file
/// and line the injector reports are the ones of this call site, because the
/// injector and the annotator both expand at the outermost call. The unarmed
/// branch annotates a real error the same way the injector does and never
/// touches the process counter, so an unrelated test cannot consume the
/// counter a fixture is waiting for. `SLEEPINESS` stays zero in both
/// branches, so the unarmed branch adds no delay.
macro_rules! fallible {
    ($e:expr) => {{
        if $crate::fault_scope::armed() {
            fault_injection::fallible!($e)
        } else {
            match $e {
                Ok(value) => value,
                Err(error) => {
                    let annotated = fault_injection::annotate!(error);
                    Err::<_, std::io::Error>(annotated)?
                }
            }
        }
    }};
}
pub(crate) use fallible;
