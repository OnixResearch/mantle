// SPDX-License-Identifier: AGPL-3.0-or-later
use std::io::Read;
use std::io::Write;
use std::os::unix::net::UnixStream;
use std::path::Path;
use std::time::Duration;

use crate::decisions::complete;
use crate::decisions::reservation;
use crate::quota_store::ResourceQuotaStore;

// Operational handshake bound only; timeout is not a guest invariant verdict.
const CHILD_HANDSHAKE_TIMEOUT_S: u64 = 10;

/// Kill a real accounting process before, after, or racing atomic publication.
/// The parent does not synthesize ledger state or acknowledgements.
pub fn interrupt(directory: &Path, boundary: u32) -> Result<(), String> {
    let (mut parent, mut child) = UnixStream::pair().map_err(|e| e.to_string())?;
    parent
        .set_read_timeout(Some(Duration::from_secs(CHILD_HANDSHAKE_TIMEOUT_S)))
        .map_err(|e| e.to_string())?;
    // Single-threaded harness; child only uses the production store and its pipe,
    // never the inherited SDK transport or assertion mutexes.
    let pid = unsafe { libc::fork() };
    if pid < 0 {
        return Err("accounting fork failed".into());
    }
    if pid == 0 {
        drop(parent);
        let result = (|| -> Result<(), String> {
            child.write_all(b"R").map_err(|e| e.to_string())?;
            child.read_exact(&mut [0u8]).map_err(|e| e.to_string())?;
            let store = ResourceQuotaStore::open(directory).map_err(|e| format!("{e:?}"))?;
            store.reconcile(&reservation("crash"), complete()).map_err(|e| format!("{e:?}"))?;
            child.write_all(b"C").map_err(|e| e.to_string())?;
            child.read_exact(&mut [0u8]).map_err(|e| e.to_string())?;
            Ok(())
        })();
        unsafe { libc::_exit(if result.is_ok() { 0 } else { 71 }) };
    }
    drop(child);
    let exchange = (|| -> Result<(), String> {
        let mut marker = [0u8];
        parent.read_exact(&mut marker).map_err(|e| e.to_string())?;
        if marker != *b"R" {
            return Err("accounting readiness marker invalid".into());
        }
        if boundary != 0 {
            parent.write_all(b"G").map_err(|e| e.to_string())?;
            if boundary == 1 {
                parent.read_exact(&mut marker).map_err(|e| e.to_string())?;
                if marker != *b"C" {
                    return Err("accounting commit marker missing".into());
                }
            }
        }
        Ok(())
    })();
    // Always reap even when the exchange fails. Never kill the harness/PID 1.
    let killed = unsafe { libc::kill(pid, libc::SIGKILL) };
    let mut status = 0;
    let waited = unsafe { libc::waitpid(pid, &mut status, 0) };
    exchange?;
    if killed != 0 || waited != pid || !libc::WIFSIGNALED(status) || libc::WTERMSIG(status) != libc::SIGKILL {
        return Err("accounting child was not observed killed and reaped".into());
    }
    Ok(())
}
