# ADR 0039: Relocate StageX configure helpers to a bounded utility

## Status

Accepted (2026-07-29)

## Context

Seven authenticated binutils 2.30 configure scripts invoke `sleep` through `PATH`. They also contain direct `/usr/bin/file` calls.

The protected tool namespace did not contain `sleep`. The direct file path was ambient authority and was unavailable in the bounded environment. Configure continued after both failures, but that result could not close the child-execution boundary.

The regular-file sed bridge also used coreutils `cat` to deliver output. A consumer that closed its pipe early could leave that Mes-linked delivery child blocked.

## Decision Drivers

- Do not authorize or discover ambient `/usr/bin/file`.
- Keep authenticated configure scripts in control of compiler and file-format probes.
- Preserve the established coreutils and sed artifacts.
- Give each helper a bounded input shape and exact BLAKE3 identity.
- Make early pipe closure a bounded successful delivery result.
- Reject path escape, oversized files, unsupported sleep values, and unknown invocation names.

## Decision

Mantle builds one native multicall configure utility with TinyCC musl-v2.

The `sleep` name accepts one decimal value from 0 through 10 seconds. It uses the declared native syscall and rejects all other argument shapes.

The `file` name accepts one single-component relative regular file, with optional `-L`. It rejects absolute paths, nested paths, symlinks, and files larger than 64 MiB. It reports only bounded ELF, archive, or data classifications needed by the configure probes.

The `emit` name copies one single-component relative regular file to standard output with raw bounded I/O. It ignores `SIGPIPE` and treats `EPIPE` as an expected closed consumer. The sed bridge uses this mode instead of the Mes-linked `cat` delivery path.

Mantle replaces exactly 10 `/usr/bin/file` occurrences in each of seven configure scripts. The 70 replacements use `$MANTLE_STAGE_X_FILE`, which points to the exact `file` binding. Mantle does not regenerate configure scripts.

The utility source BLAKE3 is `b150327f4ef9256764e8024466dd706ee87012f70554f1c7a01a0d8bc08975b1`. The compiled utility BLAKE3 is `a0d4f306ed84086cb0cebff1dffb0f5fea0a93e9ee4085e6e5e0acc3e4df201f`.

## Alternatives Considered

### Authorize the host `/usr/bin/file`

Rejected because the host file is ambient and is not part of the authenticated StageX closure.

### Bind-mount a helper at `/usr/bin/file`

Rejected because it adds namespace setup authority and keeps an ambient absolute interface.

### Add `sleep` to the established coreutils stage

Rejected because it would change a closed predecessor and all later transition identities.

### Preseed all configure cache results

Rejected because broad cache seeding would hide compiler and file-format observations.

## Consequences

- All nine declared configure scripts complete without missing-command, missing-file, ambient-file, or broken-pipe diagnostics.
- The matrix records 126 bounded compiler-preprocessor probes and 2,880 bounded sed bridge invocations.
- The established coreutils, sed, and full Bash identities remain unchanged.
- The direct configure source transformation has an exact count and a negative test.
- This result does not prove source-generation steps, Make execution, binutils outputs, compiler correctness, or provider admission.
