# ADR 0137: Validate protected executables after sandbox input materialization

## Status

Accepted (2026-10-27) for the bounded local production build-shell seam exercised
by the retained compile-cache observations. Representative bootstrap-chain and
cache-off fixed-point acceptance remain separate.

Numbering: drafted as ADR 0135 and renumbered when main allocated 0135 and 0136
to other decisions. Retained compile-cache evidence names the earlier path; it
is historical and unchanged.

## Context

The explicit local compile-driver runtime already maps one daemon socket after
derivation conversion. The existing protected-exec supervisor authorizes exact
executable paths and BLAKE3 bytes. A real Mantle sandbox materializes declared
store inputs under its private host input directory before binding that directory
into the guest store. The supervisor observes those resolved host paths, not the
original guest paths. A host-root fixture cannot establish this production seam.

Treating arbitrary materialized paths as admitted, dropping executable checks on
cache hits, or mounting the daemon's cache directory would weaken the existing
boundary. Predeclaring a random sandbox work directory is not repeatable.

## Decision Drivers

- Preserve exact executable identity and existing protected-exec decisions.
- Keep daemon, socket, audit, and runtime configuration outside derivation inputs.
- Preserve ordinary local execution and the existing sandbox lifetime owner.
- Reuse the existing supervisor rather than add a second authorization mechanism.
- Exercise the production build CLI, including cache hits and unavailable daemons.

## Decision

Add an optional execution-only command factory to the pinned Mantle-local Snix
fork. Invoke it after the input provider has populated or mounted the host input
directory, retaining the input guard until command completion. Preserve the
executor's umask, stdin, cancellation gate, and process-supervision ownership.
No serialized BuildRequest, SandboxSpec, or derivation field is added.

Construct cancellation-gate arguments before the complete sandbox invocation
is passed to a command factory; configure inherited gate descriptors on the
resulting launcher separately. Appending `--block-fd` after the guest command
was rejected after the existing delayed-PID1 native regression observed a
payload running before PID1 ownership. Plain and wrapped execution share the
same prefixed arguments and executor-owned descriptor setup.

The explicit compile-driver runtime may select a protected runner, inventory, and
external audit directory. The runner first validates the original inventory.
For declared executables beneath the explicit guest store prefix, it adds a
same-identity path alias only when the corresponding materialized file exists,
canonically remains under the input root, and matches the declared BLAKE3 bytes.
Original declarations remain intact: a host-bound BusyBox or launcher may also
be present in the declared inputs. Replacing its path in place was rejected
after the native shared-seed control observed an actual host-bound denial.
Aliases retain the source entry ID and metadata but are not additional required
role candidates. The combined inventory must still satisfy the existing
4096-entry limit and all policy checks. No new executable identity is admitted.
Missing input members add no alias; other access errors and changed bytes fail
closed.

Implicit guest shell mount names do not acquire authority. The production
fixture's original `/bin/sh` builder was refused as undeclared; the corrected
fixture calls its inventoried absolute BusyBox path for the shell and applets.
It also selects the inventoried launcher through `SNIX_BUILD_BWRAP`, rather than
asking the supervised child to search ambient PATH. These are explicit seed
selections, not extra aliases or a digest-only fallback.

The supervisor retains the file opened through the tracee's root and hashes
that handle with the existing bounded reader. The descriptor's reported path
remains the exact inventory lookup/audit name, not a request to reopen that name
in the supervisor's namespace. A guest-only path need not exist there, and the
same path may select different host bytes. Both protected and diagnostic reads
use the retained tracee file. This does not bind the later kernel exec atomically
or establish filesystem isolation.
The binary entrypoints use the library supervisor rather than compile a second
private copy against the library's policy types; the retained-file hash helper
stays crate-private.

`mantle-protected-build-runner` replaces the native-only observer example.
Observations retain original/effective inventory digests, exact materialized
mappings, and existing supervisor events. They remain performance/execution
observations with `proof_evidence: false`, not rebuild admission. Strict and
remote execution exclusions remain in force.

## Consequences

- A cache hit still requires admission of the declared materialized compiler
  bytes before the driver can request an object; cache state grants no authority.
- Runtime policy changes do not enter derivation or store input identity.
- Compile-cache, protected-exec, and sandbox maintainers own the runner, runtime
  integration, and reconstructed fork patch respectively.
- `scripts/prove-cc-cache-build-shell.py` consumes the existing native driver
  fixture and exercises fresh production build states, dispositions, path/digest
  admission, and logical derivation/output identity. This is repeatability for
  the bounded seam, not representative-chain performance or bootstrap proof.
- The fork remains tied to the recorded immutable Mantle revision/tree, not an
  asserted upstream Snix revision. Its tracked patch and importer pin the exact
  reconstructed postimage.
