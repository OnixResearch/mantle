## Context

snix-build's `BuildService::do_build` is an async trait method:

```rust
async fn do_build(&self, request: BuildRequest) -> io::Result<BuildResult>;
```

`BuildResult` contains `Vec<BuildOutput>` with output nodes and refscan
matches. No log field. The bwrap implementation runs the builder inside a
namespace and captures its exit status, but stdout/stderr handling depends
on the specific implementation.

`BubblewrapBuildService` spawns a `bwrap` child process. The child's
stdout/stderr can be piped and captured by the calling code.

## Goals / Non-Goals

**Goals:**
- Structured log capture from the build sandbox.
- Persistent log files accessible after the build session.
- Human-readable log display in the CLI.

**Non-Goals:**
- Log streaming (real-time output during build). v0 captures after completion.
- Log rotation or cleanup. Users manage `$CRUNCH_LOG_DIR` themselves.
- Remote log storage. Local filesystem only.

## Decisions

### 1. Capture at the orchestrator level, not the BuildService level

**Choice:** Modify how crunch invokes `do_build` to capture output, rather
than changing the `BuildService` trait signature.

**Rationale:** The `BuildService` trait is from vendored snix-build. Changing
its signature ripples through all implementations. Instead, the orchestrator
can wrap the call with output capture at the crunch layer.

**Implementation:** `BubblewrapBuildService` already configures the child
process. Add a mechanism to pipe stdout/stderr to a buffer. This may require
a small patch to the vendored bwrap code to expose the child's stdio, or
wrapping the call to capture at a higher level.

**Alternative:** Add `log: Option<Vec<u8>>` to `BuildResult`. Rejected —
changes the vendored trait.

### 2. Log file naming: `<drv-store-path-hash>.log`

**Choice:** Use the 32-character nixbase32 digest from the derivation store
path as the filename.

**Rationale:** Unique per derivation. Human-readable with `crunch log`.
No collision risk.

### 3. `crunch log` subcommand

**Choice:** Add a `log` subcommand that takes a derivation hash or path
and prints the stored log.

**Rationale:** Users need access to build logs after the fact. `crunch log
<hash>` is simpler than `cat $XDG_STATE_HOME/crunch/logs/<hash>.log`.

**Implementation:**
```
crunch log <hash-or-path>     Print the build log
crunch log --list             List stored logs
```

## Risks / Trade-offs

**[Bwrap internals]** → Capturing stdout/stderr from the bwrap child
requires understanding how `BubblewrapBuildService` spawns the process.
May need a small vendored patch.

**[Disk usage]** → Build logs accumulate without cleanup. Acceptable for
v0. Document that users should periodically clean `$CRUNCH_LOG_DIR`.
