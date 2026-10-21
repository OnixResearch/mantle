# Reproducible package behavior matrix

This is the executable validation scope for issue #16, not an assertion that
only these packages are supported. No package outcome is checked in as a pass.
Run `scripts/mantlepkgs-matrix.py` against a frozen Mantle binary and retain its
real artifacts. A manifest, a successful import, a process exit, and generic
`crunch-test` results are not package-behavior acceptance.

## Reconciled compatibility inventory

The existing `mantlepkgs` and `mantlepkgs-catalog-structure` Cairn specifications
remain authoritative. In particular, `mantlepkgs.recomputed_rebuild` requires
recomputed output paths and no substitution; `mantlepkgs.validation` requires a
live proof; domain validation remains separate from package identity. This
matrix does not relax any of those requirements or add parser forms.

| Concrete form | Current boundary | Matrix treatment |
| --- | --- | --- |
| Traditional input-addressed outputs with concrete paths | Admitted through the reviewed `/nix/store` adapter | Every selected ordinary root |
| Several named concrete outputs | Admitted, not flattened | zlib/libpng require distinct `dev` and `out`; cold/warm receipts must contain **all** root outputs |
| Flat or NAR fixed outputs with concrete paths | Admitted projection; source readiness still required | Explicit producer-bound seeds and content-checked source bundle |
| Structured `__json`, output placeholders, `passAsFile` | Preserved and translated through existing protocol handling, subject to bounds | Real selected graph, plan, receipt; not a claim about all stdenv recipes |
| Generated sources / parser generators | An ordinary dependency graph, not a dynamic derivation | Bison produces C from the checked-in grammar; compile and evaluate valid/invalid expressions |
| Rust package recipes | Ordinary concrete foreign graph, **not** Mantle Cargo-resolver parity | Pinned ripgrep graph and real `rg` semantics; unsupported dependencies remain failures |
| Floating, deferred, impure outputs | Parser facts; foreign projection rejects | Direct ATerm negatives expect `unsupported-nix-derivation-output` |
| Text-addressed / Git-addressed fixed outputs | Parser facts; foreign projection rejects | Direct ATerm negatives; a Git-addressed output is not the same as an admitted Git-download builtin |
| Recursive dynamic input graph | Parser fact; foreign projection rejects without flattening | Versioned ATerm negative expects `unsupported-nix-dynamic-input` |
| Non-UTF-8 environment, logical-path/environment-name mismatch | Rejected at the string projection / identity boundary | Existing boundary fixtures remain the authoritative rail |
| Unknown builtin, unresolved path, graph cycle, ambiguous alias, opaque embedded source-store assumptions | Existing conversion blockers | Preserve failure dispositions; do not repair by rewriting opaque payloads |
| Other prefixes / Guix | Outside this reviewed Nix adapter | Existing prefix-aware path is unchanged; no matrix parity claim |

See [the compatibility boundary](../../docs/nix-derivation-compatibility-boundary.md)
for parser identity, limits, upstream pin, hash domains and rollback. Parser
admission alone never establishes buildability. The strict translation policy
continues to prohibit embedded source-payload rewriting and undeclared sandbox
capabilities. A package exposing a real unsupported case must stay failed until
that specific case is reviewed and fixed; do not weaken policy to make the row
pass.

## Exact cohort and semantic oracles

All selectors are pinned to Nixpkgs
`dfd9566f82a6e1d55c30f861879186440614696e`, using the source-lock digest already
bound by `live-cohort/manifest.ncl`. No floating flake reference or replacement
resolver is introduced.

| Selector | Representative behavior | Success oracle, after cold build and again after reuse |
| --- | --- | --- |
| hello | Native executable | Exact default greeting and exact custom greeting |
| zlib | Native library, several outputs | Compile against selected `dev`, link selected `out`; binary-containing compression/decompression round-trip and corrupt-stream rejection |
| libpng | Library consuming zlib | Compile/link selected PNG and zlib outputs; exact RGBA pixel round-trip and invalid-signature rejection |
| jq | Native processing tool | Exact JSON aggregation/projection and malformed-input parse rejection |
| ripgrep | Rust executable | Exact numbered matches, exclusion of invalid numeric records, no-match status and empty output |
| bison | Generated source | Generate calculator C, compile it with the separately admitted validation compiler, require `6*7 = 42`, reject `6+*7` |

The C/compiler/parser fixtures are separate consumers, not edits to the package
recipes. The validation compiler, m4 and runtime closure are explicit tool
inputs, not package output substitutes. A compiler or package-process crash
cannot count as a valid semantic rejection merely because it exited nonzero.
Each probe checks the expected output or expected error code/diagnostic.

Only native `x86_64-linux` execution is admitted by this driver. The preparation
report explicitly records `aarch64-linux` as **not-run**, because no native
second-architecture runner has been admitted. A wrong-system catalog request
is a negative, not evidence of executing that architecture. Add a second native
runner and review its pinned manifest, tool root and behavior evidence before
extending the execution driver; do not silently cross-compile or emulate.

## External preparation inputs

Prepare these before scheduling the matrix from immutable final source:

1. One final Mantle binary with its dynamic runtime dependencies. Do not rebuild
   it while a run is in flight. The consumer binary must have exactly the same
   BLAKE3 as the producer binary. The driver itself and the behavior fixture
   tree must also remain unchanged between preparation and consumption.
2. One exact Nix executable, network/cache access to the pinned Nixpkgs lock
   and fixed-output seeds, writable producer store, and producer daemon access
   if that exact Nix installation needs it. Its canonical implementation path,
   version, executable BLAKE3, selectors and graph/source facts are retained in
   ordinary producer receipts. Keep producer GC roots until source preparation
   has completed for every selected package.
3. Python 3.8+ and `b3sum` on the orchestration host; host `bwrap` with working
   Linux user/mount/PID/network namespaces. The Python interpreter must support
   assignment expressions. No Python package download is needed.
4. A **complete, explicit** runtime closure for the consumer tools: final
   Mantle, bwrap, a C compiler, sh, m4, and the exact statically linked sandbox
   shell expected by the compiled snix-build adapter. Include each ELF
   interpreter, dynamic library, compiler wrapper's shell, compiler support
   program and sysroot. The closure can be acquired with the exact producer
   Nix `path-info --recursive` operation, saved as a JSON array of store
   directories. That is an explicit acquisition step, never a consumer query.
   Include no Nix executable or daemon socket. The rootfs assembler does not
   infer missing dependencies from the host. Build profiles, tools or runtime
   closure missing at execution are blockers, not reasons for a host fallback.
5. Spacious scratch. Start with at least 200 GiB free and 32 GiB RAM for the six
   isolated stores, encoded source bundles, producer seeds and Rust toolchain
   graph; **this is a planning allowance, not a measured upper bound**. Inspect
   actual source inventories and free capacity after `prepare`. Increase the
   runner allocation before `consume` when necessary. Source bundles retain
   the existing 1 TiB payload and 4 TiB serialized-artifact admission bounds;
   derivation graphs retain existing 2,048-node/256 MiB limits. A real selected
   graph exceeding an admitted bound is a reported failure, not silent truncation
   or permission to raise limits.

The consumer uses `CRUNCH_NO_FUSE=1` materialization, so FUSE devices or a host
Nix daemon mount are not needed. It still requires the real nested bwrap build
sandbox and the static sandbox shell/runtime closure. Capability preflight
exercises the nested namespace before any package build.

## Complete execution plan

Run these stages on a spacious remote scratch runner under the repository's
long-build supervisor. The driver is opt-in: do not run it from ordinary tests.
Each command is timeout-bounded (default six hours, maximum 24 hours) and
captured to per-command stdout/stderr/argv/result records. The driver polls a
16 MiB log limit and kills the supervised process group on timeout/overflow.
Use a new output directory for each preparation and consumption; neither
phase resumes over earlier stores.

```sh
python3 scripts/mantlepkgs-matrix.py prepare \
  --mantle /absolute/frozen/mantle \
  --nix /absolute/exact/nix \
  --out /scratch/matrix-prepared
```

Preparation first validates the full checked-in manifest through Mantle. It
then derives one selector-only manifest from that normalized manifest per row,
copying the **same** strict policy/profile bytes. Each explicit producer batch
is atomic and complete for its selector. This avoids one unsupported Rust
node suppressing the evidence for five other packages. It is not evidence for
a six-package atomic publication; the existing batch-failure rail covers that
separate contract. All source requirements are prepared through
`mantle mantlepkgs prepare-sources`, including producer-bound fixed-output
seeds. `preparation.json` retains real failures and marks later phases not-run.

Create `runtime-closure.json` as a reviewed JSON array of exact acquired
`/nix/store/...` directories (not a command to execute). Assemble the clean
consumer root without invoking Nix:

```sh
python3 scripts/mantlepkgs-matrix.py rootfs \
  --closure /scratch/runtime-closure.json \
  --tool mantle=/absolute/frozen/mantle \
  --tool bwrap=/absolute/exact/bwrap \
  --tool cc=/absolute/exact/cc \
  --tool sh=/absolute/exact/sh \
  --tool m4=/absolute/exact/m4 \
  --out /scratch/consumer-root

python3 scripts/mantlepkgs-matrix.py consume \
  --prepared /scratch/matrix-prepared \
  --rootfs /scratch/consumer-root \
  --bwrap /absolute/exact/bwrap \
  --jobs 4 --command-seconds 21600 \
  --out /scratch/matrix-result
```

The assembled root contains only the admitted runtime closure/tools plus empty
mount points. Symlinks are resolved relative to that future root, including
absolute store links. Every file/mode/link is inventoried. Actual directories
such as `/nix` are allowed; non-directory entries with known Nix executable
names, sockets, special files and the producer Nix executable's digest under
another filename are rejected. Consumer invocations bind that root read-only,
mount only prepared inputs and private scratch, create fresh `/run`, `/tmp`,
`/proc` and `/dev`, clear the environment, deny networking, and use only
`/tools/bin` in PATH. There is no bind of the producer host root or its daemon
socket. Real preflight commands check Nix command/socket absence, tool versions
and nested bwrap execution.

Each package has its **own empty store/state** for the cold run. The six phases
are recorded separately: acquisition, import/verification, plan, execution,
behavior, reuse. Planning retains the ordinary import receipt and exact plan
identity/root outputs. Cold execution must produce a complete ordinary
realization receipt, with substitution disabled/offline and each selected
root classified `built`. All root output names/paths must agree with the plan.
The warm execution must classify **every unit** `already-present`, preserve
all output NAR hashes/sizes, and pass the semantic consumer again. This is
local cache reuse, not a second independent reproducibility build.

Negative cases use a fresh store rather than successful package stores:
unsupported ATerm output forms and dynamic inputs; wrong-system package
selection; catalog artifact tampering; missing source-bundle file; incorrect
admitted bundle identity; and a valid unrelated source bundle missing the
package's actual required source payloads. They require the actual expected
rejection, not timeout/crash, and reject a complete realization receipt.
Consumer toolchain mismatch is rejected at preflight before package execution.

The final command exits nonzero if any required row phase or negative is
failed/not-run. Preserve `matrix.json`, `preparation.json`, per-package plans,
import/cold/warm receipts, producer receipts/failures, source inventory,
rootfs identity and command transcripts together. No synthetic result or
empty success artifact can fill an unexecuted phase.

## Deterministic updates and regression gate

Change the pinned manifest only after reviewing the exact selected recipe or
source change. Regenerate through `prepare` into a new immutable directory;
never edit a generated catalog or re-pin its digest by hand. Preserve old
and new generation trees and source/tool receipts. Run the complete consumer
matrix against each admitted source/tool set, then compare real reports:

```sh
python3 scripts/mantlepkgs-matrix.py compare \
  --base /scratch/base-result/matrix.json \
  --head /scratch/head-result/matrix.json \
  --out /scratch/matrix-change.json
```

The comparison sorts package/system rows, compares stable catalog/source/
producer/plan/output/receipt facts rather than absolute transcript paths,
reports removed selections and passed-to-failed/not-run phases, and rejects
negative-case regressions. Head must itself be a complete passing matrix;
an already-failing baseline does not waive acceptance. `matrix-change.json`
binds the exact base/head report bytes. Review identities and the detailed
ordinary Mantlepkgs impact report separately for policy/tool/source changes.
No comparison result grants source-update, publication or release authority.

A failed row identifies reachable compatibility work; it must not be relabeled
supported, skipped, or passed to close the issue. Preserve that failure and
repair its actual owner before final acceptance. No result here proves full
Nixpkgs coverage, Nix evaluator parity, global reproducibility, full-source
bootstrap, package correctness or release eligibility.
