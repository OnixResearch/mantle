# Protected C/C++ compile-cache build runtime

The C/C++ cache is disabled unless explicitly selected. The driver and its
compiler policy remain declared executable/source inputs. A local daemon socket
is mapped only at execution time; no cache directory, receipt, socket, or runtime
configuration is added to derivation inputs. This path cannot admit rebuild
proofs and is excluded from strict and remote execution.

## Explicit runtime policy

Export a Nickel value through `RuntimePolicy` in
`config/compile-cache/default.ncl`. For example, a repository-local
`compile-runtime.ncl` can contain:

```nickel
let cache = import "config/compile-cache/default.ncl" in
{
  schema = "mantle-compile-driver-runtime-v1",
  host_socket = "/absolute/private/daemon.sock",
  build_id = "local-bootstrap-iteration",
  admitted_builders = ["/absolute/declared/sandbox-shell"],
  receipt_path = "/absolute/private/mapping.ndjson",
  protected_exec = {
    runner = "/absolute/bin/mantle-protected-build-runner",
    inventory = "/absolute/private/inventory.json",
    audit_dir = "/absolute/private/exec-audits",
  },
} | cache.RuntimePolicy
```

```sh
nickel export --format json compile-runtime.ncl > compile-runtime.json
mantle --store /absolute/physical-store --state-dir /absolute/state \
  build bootstrap-recipe.ncl --no-substitute \
  --compile-driver-runtime compile-runtime.json
```

Paths are explicit bounded clean absolute paths. The inventory is the existing
`Stage0Inventory` JSON contract, including actual bounded version evidence and
BLAKE3 executable identities. Select only builders and executables authorized
for the workload. The runner does not install a driver, rewrite PATH, construct a
trusted compiler inventory, or turn test-fixture provenance into source-built
provenance. The recipe must already call its declared driver with its declared
policy. Omitting `protected_exec` retains the ordinary explicit socket mapping.

Use the inventory's exact absolute shell path as the recipe builder and applet
launcher. A BusyBox seed uses `busybox sh -c ...` and `busybox env -i ...`;
implicit `/bin/sh` and `/bin/busybox` mount names do not acquire inventory
authority. Select the declared launcher through `SNIX_BUILD_BWRAP` and the
declared shell through `SNIX_BUILD_SANDBOX_SHELL`. These host-side selections
do not enroll extra executable paths. The native rail makes both selections
explicit rather than relying on PATH searches or incidental shell aliases.

The fixed guest endpoint is `/run/mantle/compile-cache.sock`. Only
`MANTLE_CC_CACHE_SOCKET` and `MANTLE_COMPILE_BUILD_ID` are supplied by the runtime;
these keys cannot be overridden by derivation environment entries. Inherited
compiler-cache or scheduling authority is refused. Missing daemon endpoints are
left unmounted so the driver can compile normally. Runtime policy and audits do
not override strict proof exclusions.

## Materialized executable admission

Mantle invokes the protected runner after populating the sandbox's host input
directory. The runner validates the original inventory, then checks each
corresponding materialized executable under the explicit guest store prefix:

- Canonical path remains beneath the populated host input root.
- File bytes match the exact declared BLAKE3 digest.
- Original declarations plus materialized aliases still pass the existing
  protected-policy checks and combined 4096-entry bound.

Original host-bound paths remain admitted even when the same seed is also a
declared input. Byte-checked aliases retain entry IDs, roles, provenance, digests,
and version evidence; their `required` flag is false so a materialized shell or
launcher cannot create another required-role candidate. Missing input members
add no alias. Other access failures and changed bytes refuse execution before
creating the audit file. Inventory input is capped at exactly 1 MiB; oversized
and malformed values fail before audit creation.

For each intercepted exec, the supervisor hashes the file it opened through the
tracee's root. It does not reopen the descriptor's reported pathname in the host
namespace: a guest-only name may be absent there or select different bytes.
Inventory lookup and audit names are unchanged; exact bytes still have to match.
The same retained-file rule applies to diagnostic observation. It is not atomic
exec-to-file binding or a filesystem-isolation guarantee.

Each sandbox claims a fresh audit file beneath the external `audit_dir`, named
from its unique sandbox work directory. Keep audit and mapping paths outside
build outputs. The observation schema is
`mantle-protected-build-observation-v1`; it contains
`source_inventory_digest_blake3`, effective `inventory_digest_blake3`, exact
`materialized_aliases`, and existing protected supervisor events. Its
`proof_evidence` is always false. It is not rebuild, bootstrap, or filesystem
isolation proof. See [ADR 0137](../adr/0137-validate-protected-executables-after-materialization.md).

## Repeat the bounded native rail

Build the binaries in the checked-in development shell. Use fresh short absolute
artifact directories (`NATIVE_FIXTURE` and `BUILD_SHELL_ARTIFACTS`) and matched
GCC-compatible native executables (`NATIVE_CC` and `NATIVE_CXX`):

```sh
export CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-$PWD/target}"
cargo build --locked -p mantle --bin mantle --bin mantle-protected-build-runner
cargo build --locked -p crunch-rustc-wrapper \
  --bin mantle-cc-driver --bin mantle-rust-cache-daemon
python3 scripts/prove-cc-cache-slots.py \
  --driver "$CARGO_TARGET_DIR/debug/mantle-cc-driver" \
  --daemon "$CARGO_TARGET_DIR/debug/mantle-rust-cache-daemon" \
  --compiler "$NATIVE_CC" --cxx "$NATIVE_CXX" \
  --work-dir "$NATIVE_FIXTURE" \
  --protected-exec-runner "$CARGO_TARGET_DIR/debug/mantle-protected-build-runner" \
  --sandbox-shell "$SNIX_BUILD_SANDBOX_SHELL"
python3 scripts/prove-cc-cache-build-shell.py \
  --fixture-dir "$NATIVE_FIXTURE" \
  --mantle "$CARGO_TARGET_DIR/debug/mantle" \
  --driver "$CARGO_TARGET_DIR/debug/mantle-cc-driver" \
  --daemon "$CARGO_TARGET_DIR/debug/mantle-rust-cache-daemon" \
  --protected-exec-runner "$CARGO_TARGET_DIR/debug/mantle-protected-build-runner" \
  --sandbox-shell "$SNIX_BUILD_SANDBOX_SHELL" \
  --work-dir "$BUILD_SHELL_ARTIFACTS"
```

The second rail uses the real build CLI with fresh per-case state, declared
seed/runtime roots, the real daemon, and the real seccomp supervisor. It checks
cold compilation, object replay without compiler execution, unavailable-daemon
compilation, unchanged logical derivation/output identity and object bytes, and
undeclared/wrong-digest refusals. Dispositions are retained from driver receipts
in normal build logs, not added to outputs. Source staging uses explicit
practical-mode host input fallback; those observations are not an authenticated
source-built cohort. Each command has a recorded 900-second ceiling because
fresh state ingests the complete selected compiler/runtime packages, not pruned
stand-ins. Keep each `evidence.json`, command streams, selected logs, and
protected audits; do not enroll entire store trees as audit artifacts.

Compile-cache maintainers own the driver/runtime rails; protected-exec
maintainers own admission/supervision; sandbox maintainers own the reconstructed
Snix hook. These checks establish the bounded production seam only.
Representative bootstrap-chain parity/timing, a cache-off fixed point, and
release acceptance remain separate obligations.
