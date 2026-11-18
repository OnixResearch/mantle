//! Bounded stdout/stderr capture for sandbox children.
//!
//! Draining must never depend on the child stopping first: a build can emit
//! arbitrarily many bytes, and a full pipe would otherwise deadlock the
//! parent while it waits for exit. Each stream retains a bounded prefix, keeps
//! draining to EOF, and reports truncation explicitly.

use std::io;
use std::process::Output;

use tokio::io::AsyncRead;
use tokio::io::AsyncReadExt;
use tokio::process::Child;

/// Maximum bytes retained from each sandbox stream.
///
/// Two streams plus truncation markers must stay under the worker's 1 MiB
/// retained-log cap so a truncated log is still visible rather than dropped.
pub(crate) const SANDBOX_STREAM_CAPTURE_LIMIT_BYTES: usize = 480 * 1024;

/// Marker appended when a stream exceeded the retained bound.
pub(crate) const SANDBOX_CAPTURE_TRUNCATION_MARKER: &[u8] =
    b"\n[mantle: sandbox output truncated after 491520 bytes; the remainder was drained]\n";

/// Read one child stream, retaining at most the declared bound.
pub(crate) async fn capture_stream(reader: impl AsyncRead + Unpin) -> io::Result<Vec<u8>> {
    assert!(SANDBOX_STREAM_CAPTURE_LIMIT_BYTES > 0, "capture bound must be positive");
    let mut captured = Vec::with_capacity(SANDBOX_STREAM_CAPTURE_LIMIT_BYTES.min(16 * 1024));
    let mut truncated = false;
    let mut buffer = [0_u8; 8 * 1024];
    let mut reader = reader;
    loop {
        let read = reader.read(&mut buffer).await?;
        if read == 0 {
            break;
        }
        let remaining = SANDBOX_STREAM_CAPTURE_LIMIT_BYTES.saturating_sub(captured.len());
        if remaining == 0 {
            truncated = true;
            continue;
        }
        let retained = read.min(remaining);
        captured.extend_from_slice(&buffer[..retained]);
        if retained < read {
            truncated = true;
        }
    }
    debug_assert!(captured.len() <= SANDBOX_STREAM_CAPTURE_LIMIT_BYTES);
    if truncated {
        captured.extend_from_slice(SANDBOX_CAPTURE_TRUNCATION_MARKER);
    }
    Ok(captured)
}

async fn capture_optional<R>(reader: Option<R>) -> io::Result<Vec<u8>>
where R: AsyncRead + Unpin {
    match reader {
        Some(reader) => capture_stream(reader).await,
        None => Ok(Vec::new()),
    }
}

/// Wait for a child while concurrently draining both output pipes.
///
/// The returned vectors are bounded; truncation is self-describing. A capture
/// error still waits for the child before returning so a failed reader cannot
/// leave an unreaped or pipe-blocked process behind.
pub(crate) async fn capture_child_output(mut child: Child) -> io::Result<Output> {
    let stdout = child.stdout.take();
    let stderr = child.stderr.take();
    let (stdout, stderr, status) = tokio::join!(capture_optional(stdout), capture_optional(stderr), child.wait());
    let stdout = stdout?;
    let stderr = stderr?;
    let status = status?;
    debug_assert!(stdout.len() <= SANDBOX_STREAM_CAPTURE_LIMIT_BYTES + SANDBOX_CAPTURE_TRUNCATION_MARKER.len());
    debug_assert!(stderr.len() <= SANDBOX_STREAM_CAPTURE_LIMIT_BYTES + SANDBOX_CAPTURE_TRUNCATION_MARKER.len());
    Ok(Output { status, stdout, stderr })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn capture_exactly_at_the_limit_is_not_truncated() {
        let payload = vec![b'x'; SANDBOX_STREAM_CAPTURE_LIMIT_BYTES];
        let captured = capture_stream(std::io::Cursor::new(payload.clone())).await.unwrap();
        assert_eq!(captured, payload);
    }

    #[tokio::test]
    async fn capture_over_the_limit_retains_prefix_and_marks_truncation() {
        let payload = vec![b'x'; SANDBOX_STREAM_CAPTURE_LIMIT_BYTES + 1];
        let captured = capture_stream(std::io::Cursor::new(payload)).await.unwrap();
        assert_eq!(captured.len(), SANDBOX_STREAM_CAPTURE_LIMIT_BYTES + SANDBOX_CAPTURE_TRUNCATION_MARKER.len());
        assert!(captured.ends_with(SANDBOX_CAPTURE_TRUNCATION_MARKER));
        assert!(captured.starts_with(&vec![b'x'; SANDBOX_STREAM_CAPTURE_LIMIT_BYTES]));
    }
}
