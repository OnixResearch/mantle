# 0163. Bound untrusted acquisition, secret loading, and sandbox diagnostics at their boundaries

Status: Proposed

## Context

Six issue reviews found boundary defects rather than isolated bugs:

- signing-key parse errors could echo encoded secret material, and the ordinary
  loader lacked the bounded, no-follow, private-mode admission already used by
  the shared Rust key loader (#42);
- GC asserted that every retained PathInfo was signed even though
  `--trust-unsigned` imports deliberately persist unsigned records (#43);
- a cancelled build service panic or an unobserved fetch/sandbox replaced the
  worker task with a permanently pending future, so watch shutdown could not
  drain (#44);
- one-shot and watch sandbox execution collected complete stdout/stderr in
  memory before any size filter (#45);
- tarball acquisition decompressed and extracted without separate compressed,
  decompressed, per-entry, or cumulative byte budgets, and XZ/LZMA or unknown
  suffixes buffered whole inputs (#46);
- the flake still resolved a transitive SSH artifact source through a pinned
  upstream manifest, so clean evaluation drifted even after the direct
  manifests moved to HTTPS (#73).

## Decision Drivers

- Untrusted bytes must be consumed under explicit limits, not checked after
  allocation.
- Secrets must not cross error, tracing, or diagnostic boundaries.
- A liveness failure must be classified, not hidden behind an unobservable
  pending future.
- Store maintenance must preserve an operator's explicit trust choice.
- A clean runner must evaluate the flake without SSH credentials or Git URL
  rewrites.

## Decision

1. **Private key files** are admitted only as bounded regular files with no
   group/other permission bits, opened with `O_NOFOLLOW`, revalidated by
   device/inode/size, and read through a zeroizing buffer. The keypair parser
   verifies that the declared public half matches the secret half and never
   retains or renders the encoded input on a decode error. The ordinary and
   shared Rust loaders use the same file-admission helper.
2. **GC preserves stored PathInfo trust status verbatim.** The
   `persist_and_export_signed_output` refusal remains the build-output signing
   boundary; it is not a universal stored-record invariant. An unsigned record
   admitted through explicit policy survives GC without being signed, dropped,
   or silently repaired.
3. **Cancellable build-task panic and unobserved-teardown errors complete the
   worker task with a failed outcome.** The sandbox/fetch service retains its
   quarantine and releases it when teardown is observed; the worker releases
   its scheduling permit and pending metadata so a later generation and normal
   shutdown can proceed. No worker path waits forever for teardown.
4. **Sandbox stdout/stderr uses a shared bounded drain.** Each stream retains a
   bounded prefix (480 KiB), continues draining to EOF to avoid pipe
   deadlock, and appends an explicit truncation marker. The retained prefix and
   marker together stay under the worker's 1 MiB failed-log cap. Unix preserves
   declared environment bytes; NUL is rejected with a typed error before spawn.
5. **Acquisition has separate compressed, decompressed, per-entry, and
   cumulative budgets.** Compressed input is spooled under its budget before a
   decoder runs; XZ/LZMA output is written through a bounded writer; tar
   extraction charges regular entries and hardlink copies; prefix detection
   enforces the entry count. Flat downloads read one byte past the limit so an
   exact-limit response and an overflow are distinguishable.
6. **The transitive artifact source is patched to HTTPS.** The pinned valence
   revision still names onix-artifact over SSH; a Cargo `[patch]` redirects that
   source to the identical HTTPS revision. This is a source replacement, not a
   Git transport rewrite, and it keeps the flake assertion and clean-runner
   fetch credential-free.

## Consequences

- Over-limit acquisition fails with `FetchError::LimitExceeded` and no admitted
  output; existing traversal, symlink, hardlink, and prefix behavior remains.
- Older key files with group/other permission bits fail closed; operators must
  use private modes (0600).
- Truncated sandbox logs are visibly marked and cannot silently look complete.
- An unobserved sandbox reservation remains held by the service, so a retry may
  wait; the worker itself can still shut down and report the failure.
- The vendored `nix-compat` parser and `snix-build` capture path carry small
  local modifications; upstream syncs must retain these regressions.
- These changes reduce resource and diagnostic exposure but do not establish
  arbitrary sandbox, fetch, or key-management correctness.
