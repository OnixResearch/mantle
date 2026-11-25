//! Bounded stdout/stderr capture for sandbox children.
//!
//! Draining must never depend on the child stopping first: a build can emit
//! arbitrarily many bytes, and a full pipe would otherwise deadlock the
//! parent while it waits for exit. Each stream retains a bounded prefix, keeps
//! draining to EOF, and reports truncation explicitly.

use std::io;
use std::process::Output;
use std::sync::Arc;
use std::sync::atomic::AtomicU64;
use std::sync::atomic::Ordering;
use std::time::Duration;

use tokio::io::AsyncRead;
use tokio::io::AsyncReadExt;
use tokio::process::Child;
use tokio::time::Instant as TokioInstant;

/// Maximum bytes retained from each sandbox stream.
///
/// Two streams plus truncation markers must stay under the worker's 1 MiB
/// retained-log cap so a truncated log is still visible rather than dropped.
pub(crate) const SANDBOX_STREAM_CAPTURE_LIMIT_BYTES: usize = 480 * 1024;

/// Marker appended when a stream exceeded the retained bound.
pub(crate) const SANDBOX_CAPTURE_TRUNCATION_MARKER: &[u8] =
    b"\n[mantle: sandbox output truncated after 491520 bytes; the remainder was drained]\n";

/// Tracks monotonic read activity across child output pipes, including bytes
/// drained after [`SANDBOX_STREAM_CAPTURE_LIMIT_BYTES`] truncation.
#[derive(Debug)]
pub(crate) struct OutputActivityTracker {
    epoch: TokioInstant,
    last_activity_us: AtomicU64,
}

impl OutputActivityTracker {
    pub(crate) fn new() -> Self {
        Self::from_epoch(TokioInstant::now())
    }

    pub(crate) fn from_epoch(epoch: TokioInstant) -> Self {
        Self {
            epoch,
            last_activity_us: AtomicU64::new(0),
        }
    }

    pub(crate) fn record_activity(&self) {
        let elapsed_us = u64::try_from(self.epoch.elapsed().as_micros()).unwrap_or(u64::MAX);
        self.last_activity_us.fetch_max(elapsed_us, Ordering::AcqRel);
    }

    pub(crate) fn last_activity_instant(&self) -> TokioInstant {
        let elapsed_us = self.last_activity_us.load(Ordering::Acquire);
        self.epoch.checked_add(Duration::from_micros(elapsed_us)).unwrap_or_else(TokioInstant::now)
    }
}

/// Read one child stream, retaining at most the declared bound while optionally
/// recording monotonic read-activity timestamps on every non-empty read.
pub(crate) async fn capture_stream_with_activity(
    reader: impl AsyncRead + Unpin,
    activity: Option<&OutputActivityTracker>,
) -> io::Result<Vec<u8>> {
    const {
        assert!(SANDBOX_STREAM_CAPTURE_LIMIT_BYTES > 0, "capture bound must be positive");
    }
    let mut captured = Vec::with_capacity(SANDBOX_STREAM_CAPTURE_LIMIT_BYTES.min(16 * 1024));
    let mut truncated = false;
    let mut buffer = [0_u8; 8 * 1024];
    let mut reader = reader;
    loop {
        let read = reader.read(&mut buffer).await?;
        if read == 0 {
            break;
        }
        if let Some(tracker) = activity {
            tracker.record_activity();
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

/// Read one child stream, retaining at most the declared bound.
#[cfg(test)]
pub(crate) async fn capture_stream(reader: impl AsyncRead + Unpin) -> io::Result<Vec<u8>> {
    capture_stream_with_activity(reader, None).await
}

async fn capture_optional<R>(reader: Option<R>, activity: Option<&OutputActivityTracker>) -> io::Result<Vec<u8>>
where R: AsyncRead + Unpin {
    match reader {
        Some(reader) => capture_stream_with_activity(reader, activity).await,
        None => Ok(Vec::new()),
    }
}

/// Wait for a child while concurrently draining both output pipes and
/// recording read activity.
pub(crate) async fn capture_child_output_with_activity(
    mut child: Child,
    activity: Option<Arc<OutputActivityTracker>>,
) -> io::Result<Output> {
    let stdout = child.stdout.take();
    let stderr = child.stderr.take();
    let (stdout, stderr, status) = tokio::join!(
        capture_optional(stdout, activity.as_deref()),
        capture_optional(stderr, activity.as_deref()),
        child.wait()
    );
    let stdout = stdout?;
    let stderr = stderr?;
    let status = status?;
    debug_assert!(stdout.len() <= SANDBOX_STREAM_CAPTURE_LIMIT_BYTES + SANDBOX_CAPTURE_TRUNCATION_MARKER.len());
    debug_assert!(stderr.len() <= SANDBOX_STREAM_CAPTURE_LIMIT_BYTES + SANDBOX_CAPTURE_TRUNCATION_MARKER.len());
    Ok(Output { status, stdout, stderr })
}

/// Wait for a child while concurrently draining both output pipes.
///
/// The returned vectors are bounded; truncation is self-describing. A capture
/// error still waits for the child before returning so a failed reader cannot
/// leave an unreaped or pipe-blocked process behind.
pub(crate) async fn capture_child_output(child: Child) -> io::Result<Output> {
    capture_child_output_with_activity(child, None).await
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

    #[tokio::test]
    async fn activity_tracker_continues_updating_after_truncation() {
        use tokio::io::AsyncWriteExt;

        let (mut writer, reader) = tokio::io::duplex(64 * 1024);
        let tracker = Arc::new(OutputActivityTracker::new());
        let tracker_clone = Arc::clone(&tracker);
        let capture_task =
            tokio::spawn(async move { capture_stream_with_activity(reader, Some(&tracker_clone)).await });

        let chunk = vec![b'a'; SANDBOX_STREAM_CAPTURE_LIMIT_BYTES + 8192];
        writer.write_all(&chunk).await.unwrap();
        tokio::time::sleep(Duration::from_millis(25)).await;
        let before = tracker.last_activity_instant();
        tokio::time::sleep(Duration::from_millis(25)).await;
        writer.write_all(b"post-truncation-activity\n").await.unwrap();
        writer.shutdown().await.unwrap();

        let captured = capture_task.await.unwrap().unwrap();
        let after = tracker.last_activity_instant();
        assert!(captured.ends_with(SANDBOX_CAPTURE_TRUNCATION_MARKER));
        assert!(after > before, "activity tracker must keep advancing after capture truncation");
    }
}
