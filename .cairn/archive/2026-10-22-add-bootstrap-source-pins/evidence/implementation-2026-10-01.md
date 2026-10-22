# Bootstrap source pin implementation acceptance — 2026-10-01

The isolated detached `origin/main` worktree and baseline are identified in `baseline-2026-09-30.md`. Follow-up compilation copied only this bootstrap change into that worktree; the shared root was not reset. Scratch runs used `TMPDIR=/home/brittonr/scratch` and `/home/brittonr/scratch/mantle-pins-bin-target/debug/mantle`. All source URLs, external payloads, and hashes below are observed values, not proposed values.

## Real CLI: pending CMake, failed controls, reviewed mutation

The isolated real binary ran `bootstrap-pin check --root <scratch-CMake-root> --cache-dir <scratch-cache> --plan <scratch-plan>`; exit **3**, exact stdout:

```json
{
  "schema": "mantle-bootstrap-pin-plan-v1",
  "entries": [
    {
      "source": "cmake",
      "preimage": "5ce71abf2ad00adc99e7b2346edb992bcd36dfa96c5eac07393c3de0c293c207",
      "current": "3.31.8",
      "decision": {
        "status": "candidate",
        "candidate": {
          "version": "4.4.3",
          "release_date": "2026-08-25",
          "hashes": {
            "source": "sha256-WK5Ba6oZ8aQRYuWs5sEqeG1/6xgq3k578W3dBLFmTec="
          }
        }
      }
    }
  ],
  "digest": "b6ec63c6498c04cbc9b90b99ebd9b3d60d6ba9e09cb5f507437aaeaab5a278dc"
}
```

`b3sum` of the original 613-byte CMake TOML was `5ce71abf2ad00adc99e7b2346edb992bcd36dfa96c5eac07393c3de0c293c207`. Independently BLAKE3-hashing compact canonical JSON `[plan.schema,plan.entries]` produced `b6ec63c6498c04cbc9b90b99ebd9b3d60d6ba9e09cb5f507437aaeaab5a278dc`, equal to the stored plan digest. The source URL rendered from the candidate version is `https://github.com/Kitware/CMake/releases/download/v4.4.3/cmake-4.4.3.tar.gz`.

Each negative command used `bootstrap-pin apply --reviewed` against that scratch root; each exited **3** before pin/reader mutation:

| Changed input | Exact stderr |
|---|---|
| Unresealed edited plan | `error: mutated or incomplete bootstrap pin plan` |
| TOML changed after check while JSON reader stayed valid | `error: cmake: source pin changed since check` |
| Resealed plan naming a different source | `error: unknown source unknown-source` |
| Resealed plan whose expected source hash was changed to the old hash | `error: cmake: upstream hash mismatch for source` |

For the external-payload hash mismatch, the script compared all scratch-root file bytes against the before snapshot and observed equality. For the stale preimage, it confirmed the only interim edit was its own appended TOML comment, the derived reader was unchanged, then restored the original TOML. The focused binary test `unresolved_source_blocks_entire_mixed_candidate_plan_without_writes` built a sealed plan with one valid candidate and one upstream error: `apply` rejected the entire plan before downloads/publication; both TOML files and both readers remained byte-identical.

Reviewed apply of the original unchanged CMake plan exited **0**, exact stdout `{"schema":"mantle-bootstrap-pin-apply-v1","applied_sources":1}`. Exactly `bootstrap/pins/cmake.toml` and `bootstrap/pins/generated/cmake.json` changed; the Nickel recipe bytes did not. Original TOML/JSON lengths were **613/735** bytes, applied lengths **610/732** bytes. Applied TOML BLAKE3 was `1cb38f0b786bd3d9ab116fea868e9c65cce9baf3a2e2245a70fe684777ea0383`. The evaluated fixed-output input afterward was exactly URL `https://github.com/Kitware/CMake/releases/download/v4.4.3/cmake-4.4.3.tar.gz`, tree hash `sha256-WK5Ba6oZ8aQRYuWs5sEqeG1/6xgq3k578W3dBLFmTec=`, mode `recursive`. The original source input was URL `https://github.com/Kitware/CMake/releases/download/v3.31.8/cmake-3.31.8.tar.gz`, tree hash `sha256-1sQqWhxydhTeQ0PUpW/wToX/u4kVmd6YE97O40G2sV8=`. A Nickel-export comparison before/after application verified builder and addressing mode, non-source inputs, and builder arguments other than the versioned script unchanged; the builder script afterward equaled its predecessor after exactly `3.31.8` → `4.4.3` substitution. Separately, exporting complete original `origin/main` and migrated CMake derivations before the bump showed JSON equality, including the URL/hash and all builder phases/inputs.

## Real CLI: unchanged Picolibc without Nickel executable

The isolated real binary ran with `PATH` set to an empty scratch directory and the fixture asserted `nickel` was absent. Two `bootstrap-pin check` invocations each exited **0** and produced identical stdout:

```json
{
  "schema": "mantle-bootstrap-pin-plan-v1",
  "entries": [
    {
      "source": "picolibc",
      "preimage": "f114eda07b6da85c1481e3c039da6b4e60caffad82cd7dcb712ee1bee66a5c47",
      "current": "1.8.12",
      "decision": {
        "status": "current"
      }
    }
  ],
  "digest": "3213c87ba32a09647ae6e7606c97c15c1e750d5897e290cd40dd713fc94262fe"
}
```

The upstream conditional request returned 304: the cache file's modification time was unchanged and no new body was parsed. Reviewed apply exited **0**, exact stdout `{"schema":"mantle-bootstrap-pin-apply-v1","applied_sources":0}`, with TOML bytes unchanged. A tampered plan exited **3**, stderr `error: mutated or incomplete bootstrap pin plan`, still without TOML mutation.

## Independent recipe, rollback, operator and policy rails

`python3 bootstrap/pins/generate_readers.py --check-recipes` passed byte-exact reader checks and bounded Nickel export of CMake plus both Picolibc recipes. A temporary *actual `crunch.fetchTarball` Nickel recipe* using `pin.artifacts.source.url` evaluated and passed; changing only its fetch URL to the literal `https://example.org/hardcoded-source.tar.gz` produced the exact rejection `cmake-3.31.8-gcc10.ncl: evaluated source fetch URL, sha256 hash, or flat/tree mode differs from pin`. Independent evaluated fixtures also rejected a wrong SHA256 and a flat mode for the tree artifact. No source-text matching was used. The rail bounds each Nickel export to 45 seconds, a 30-second CPU limit, and an 8-MiB output-file limit. Runtime `check/apply` checks TOML and reader bytes but does **not** claim recipe-binding validation without this separate rail or require Nickel on PATH.

Focused isolated Mantle tests passed **3/3**: URL authority bounds, first-file committed/second-file publication fault with successful restoration of both originals, and mixed candidate+upstream-error no-write rejection. Publication first stages all replacements then renames under a directory advisory lock. Rollback is best effort: a second failure during restoration produces the explicit `ROLLBACK UNCERTAIN` error; a power loss between renames or a noncooperating external editor is not a cross-file transaction and is not claimed atomic. Runtime re-verifies plan preimages while holding its advisory lock before publication.
After replacing the staged-publication loop counter with an enumerated
index, the isolated binary was rebuilt and the three focused pin tests
passed again (**3/3**). The rebuilt binary repeated the real CMake pending
check, all four no-write controls (including stale preimage), and reviewed
apply: the same exit codes, old/new versions, observed SHA256, and exactly
two changed data files were observed. This is the final exercised
publication implementation, not merely the earlier build.

The research-only Picolibc diagnostic's output name and fact-manifest
`picolibc_version` were also changed to read `pin.version` rather than
hardcode `1.8.12`. At the checked pin, a complete Nickel export of the
previous and updated diagnostic derivations produced identical **4545-byte**
JSON results, including output name
`picolibc-1.8.12-x86_64-linux-diagnostic` and every builder argument.
After that edit, `generate_readers.py --check-recipes` passed all three
declared consumers and Nickel typecheck passed both Picolibc files.

An isolated **synthetic** `1.8.13` reader projection (no source download)
evaluated the unchanged diagnostic recipe: the output name changed exactly
from `picolibc-1.8.12-x86_64-linux-diagnostic` to
`picolibc-1.8.13-x86_64-linux-diagnostic`, the generated builder script
contained `picolibc_version=1.8.13` instead of `1.8.12`, and the entire
script was byte-equal to the old one after substituting `1.8.12` →
`1.8.13`. This checks data-derived receipt behavior, not existence of a
published 1.8.13 release or an accepted diagnostic lineage. CMake is the
migrated live source-built family.

`nickel typecheck config/operator-surfaces.ncl` and the three changed bootstrap recipes passed. Re-exporting the policy NCL yielded **210203 bytes**, byte-identical to `config/operator-surfaces.json`. The coherent shared Mantle binary generated combined bootstrap-pin and remote-live descriptors/catalog/reference/workflow, then `__operator-contract --mode check` printed `operator command contract: PASS (commands=182)`. Core focused pin tests passed **2/2** and strict core Clippy passed; portable-client-core tests passed **7/7** and strict Clippy passed. Cairn `gate proposal`, `gate design`, and `gate tasks` for this change each reported `verdict: PASS`, `issues: []` as *advisory structural checks*, not lifecycle acceptance.

This **210203-byte / 182-command** inventory receipt describes the
coherent combined snapshot before the separate attestation owner corrected
attest-specific policy rows and workflow wording. That owner took control
of the shared NCL/JSON and generated operator files afterward and must
repeat generation/freshness checks for their final snapshot; this pin
change does not silently claim those later bytes.
After the attestation owner revised nine attestation policy rows, an
independent read-only `nickel export --format json
config/operator-surfaces.ncl` exited **0** and returned **210252 bytes**,
byte-identical to the checked `config/operator-surfaces.json`. That owner
acknowledged this source/JSON freshness result and retains sole ownership
of final binary-generated descriptors, catalog, reference, and workflow;
this policy-only check does not establish freshness of those four artifacts.

After evidence-only task checkoff, the scoped Cairn tasks gate again exited
**0**, `verdict: PASS`, showing **11 done / 2 open**. Workspace Cairn
`validate --root .` exited **0** with `change_issues: []`; its sole reported
finding remained the unrelated thin-cli I7-before-I6 task-order issue. The
tasks gate and validation are advisory structural results, not source-sync or
archive receipts.

The obsolete `scripts/check-bootstrap-source-pins.rs` was demonstrated to fail the valid migrated CMake `pin.artifacts.source.hash` expression (`bad-hash-format`, one file/one fetch block/one issue); a targeted search found no active callers in scripts, Nix, CI, or active changes. It was removed instead of being special-cased; one current bootstrap inventory statement was corrected, historical archived Cairn records left intact. The workspace-wide Cairn validation separately reported an unrelated existing task-ordering issue at `.cairn/changes/thin-cli-composition-root/tasks.md:88` (I7 marked done before I6), which this change does not modify. No archive/sync is claimed here.

The documented `bootstrap-pin check/apply` shell example in
`docs/bootstrap-stage0-inventory.md` now quotes `$HOME` normally instead
of emitting literal escaped quote characters. A throwaway shell argument
parse using `HOME="/tmp/pin smoke"` produced exactly four flag/value
arguments, with cache and plan paths preserving the embedded space; no
networked check/apply invocation was repeated for this documentation fix.

Scoped `nix build --offline --no-link .#checks.x86_64-linux.bootstrap-blocker-inventory` exited **0**; Nix first reported an unavailable remote `aspen1.local` builder and built locally. `rustfmt --edition 2024 --check` passed the two owned Rust pin modules. Strict `cargo clippy -p crunch-project-core --lib -- -D warnings` and strict `cargo clippy -p mantle-portable-client-core --lib -- -D warnings` passed. Strict whole-Mantle binary Clippy is **not** claimed: the first isolated run failed with 36 pre-existing vendored `fuse-backend-rs` lints; `--no-deps -D warnings` then reached Mantle and reported 35 findings across pre-existing modules (for example `src/ambient_env.rs` and `src/ast_grep_evidence.rs`) plus one owned `explicit_counter_loop`, which was fixed by enumerating staged writes. The remaining global lint backlog belongs to the wider repository, not this pin implementation.

The first attempted *shared-root* focused `cargo test -p mantle --bin
mantle bootstrap_pin_cmd::tests::` did not reach pin tests because a
concurrent Rust-plan trait cutover made `src/rust_plan.rs` fail compilation
with five unrelated errors: a removed `UnitExecutor::execute_unit` method,
missing `Prepared`/`Receipt` and
`prepare_unit`/`execute_miss`/`finish_unit`, missing
`UnitExecutionResult.process_attempts`, and removed
`ExecutionError.code/detail` fields. The owner subsequently reported
`cargo check -p mantle --bin mantle --locked` green after restoring the
Rust-plan adapter. Only then was the scoped root pin test retried with
`TMPDIR=/home/brittonr/scratch` and `--locked`; it again stopped before
pin tests, now with unrelated `E0382` in `src/build_cmd.rs`: live publisher
construction moved `run_id` before the worker snapshot reused it. The
live owner fixed that move by borrowing `run_id.as_str()` and is awaiting
a coherent shared build amid other active changes. No identical failed
root test has been rerun without an owner's coherence signal. The
isolated baseline+bootstrap overlay's final **3/3** pin tests and real
pending/check/apply evidence above remain the scoped proof; T4.2 stays
unchecked for this integration interruption and the strict lint backlog.

Afterward, the Rust-plan/deny owners reported a *new prerequisite*, not a
bootstrap-pin test failure: Cargo.lock selected rustls **0.23.45** while the
then-current Nix offline vendored registry had only **0.23.37**. This
worker did **not** repeat the pin-root test under that known stale closure.
The backend owner subsequently reported repo-owned `vendor-deps.py generate`
and `check` passing on the new lock, locked offline `cargo metadata
--no-deps` exiting 0, the vendored rustls **0.23.45** and h2 **0.4.16**
manifests present, and a new-lock Crane closure built. The vendor owner then
reported an actual default Nix dev-shell
`cargo check --locked --offline -p crunch-store --lib` exit **0**, compiling
rustls **0.23.45**, h2 **0.4.16**, Casita, and crunch-store. This resolved
the vendored compile prerequisite, not lasting Mantle-root coherence.
After the S0/dynamic/Nickel repairs, the Rust-plan owner reported a fresh
integrated `cargo check -p mantle --bin mantle --locked --offline` GREEN
(artifact `7424`). During the single scoped shared-root pin test,
wasm-receipt and dynamic-glue production edits changed the shared source,
making that earlier root check historical. The cold-target test nevertheless
compiled and ran against its observed source snapshot:
`TMPDIR=/home/brittonr/scratch
CARGO_TARGET_DIR=/home/brittonr/scratch/mantle-pins-shared-final-target
NIX_CONFIG='eval-cache = false' nix develop --offline --no-write-lock-file
-c cargo test --locked --offline -p mantle --bin mantle
bootstrap_pin_cmd::tests:: -- --nocapture` exited **0**, **3 passed,
0 failed, 0 ignored, 0 measured, 2637 filtered out**. The test-profile
build took **12m 55s**, the tests **0.00s**; the complete command/output
transcript is `evidence/shared-root-pin-tests-2026-10-01.txt` (also tool
artifact `7680`).
Six unrelated dead-code/test-helper warnings did not fail compilation.
The exact counts and exit were sent immediately to the Rust-plan owner.
That owner correctly retained the core/app API freeze: this historical pass
preceded final wasm/dynamic syntax readiness and did not re-certify later
shared-source edits.

After those owners signaled final syntax readiness, the Rust-plan owner
reported a **fresh current-tree** `cargo check -p mantle --bin mantle
--locked --offline` exit **0** after **3m 13s** (artifact `7904`; 16
nonfatal warnings). Only then was the same focused `cargo test --locked
--offline -p mantle --bin mantle bootstrap_pin_cmd::tests:: --
--nocapture` run once using the warmed private target. It exited **0**:
**3 passed, 0 failed, 0 ignored, 0 measured, 2637 filtered out**;
test-profile build **3m 41s**, tests **0.01s**, wall **260.47s**. All
three named tests passed, with six unrelated dead-code/test-helper
warnings. The command/output is retained in
`evidence/shared-root-pin-tests-current-tree-2026-10-01.txt`; the exact
counts and exit were sent immediately to the Rust-plan owner. This
releases the bootstrap-pin half of the frozen core/app API gate, not the
independent Android half, strict whole-binary Clippy, accepted-spec sync,
or archive. T4.2 stays open on the strict lint/final quality backlog.

Separately, after the historical evidence update, the scoped Cairn
`gate tasks` using the
canonical `/home/brittonr/git/OnixResearch/cairn` path exited **0** with
`verdict: PASS`, `valid: true`, `issues: []`, **11 done / 2 open**. Its
complete command/output is retained in `evidence/cairn-tasks-gate-2026-10-01.txt`.
The first Nix command using the documented `/home/brittonr/git/cairn`
symlink exited **1** before Cairn ran (`file ... is a symlink`); canonical
path retry succeeded despite a nonfatal Git server HTTP **530** warning.
This is an advisory structural gate, not proof of implementation truth,
accepted-spec sync, current-tree root stability, or archive.

## Isolated integration ownership handoff

Inspected owned paths in the shared checkout; copy *only these bootstrap
changes* into a future isolated integration branch after root quality
blockers are resolved:

- Cairn: `.cairn/changes/add-bootstrap-source-pins/design.md`,
  `tasks.md`, `specs/bootstrap-source-pins/spec.md`,
  `evidence/baseline-2026-09-30.md`,
  `evidence/implementation-2026-10-01.md`,
  `evidence/shared-root-pin-tests-2026-10-01.txt`,
  `evidence/shared-root-pin-tests-current-tree-2026-10-01.txt`, and
  `evidence/cairn-tasks-gate-2026-10-01.txt`.
- Boundary ADR: `adr/0086-separate-bootstrap-source-pins-from-catalog-update-policy.md`.
- Pin data/rail: `bootstrap/pins/cmake.toml`,
  `bootstrap/pins/picolibc.toml`,
  `bootstrap/pins/generated/cmake.json`,
  `bootstrap/pins/generated/picolibc.json`, and
  `bootstrap/pins/generate_readers.py`.
- Consumers: `bootstrap/cmake-3.31.8-gcc10.ncl`,
  `bootstrap/picolibc-1.8.12-src.ncl`, and
  `bootstrap/picolibc-1.8.12-diagnostic.ncl`.
- Core/shell/platform: `crates/crunch-project-core/src/bootstrap_pins.rs`,
  the `pub mod bootstrap_pins` insertion in its `lib.rs`,
  `src/bootstrap_pin_cmd.rs`, and the `bootstrap-pin` compiled profile/count
  insertion in `crates/mantle-portable-client-core/src/lib.rs`.
- Existing documentation: `docs/bootstrap-stage0-inventory.md`;
  **remove** obsolete `scripts/check-bootstrap-source-pins.rs`.

The following paths are **shared with other owners** and cannot be copied
whole from this worktree: `Cargo.toml` (bootstrap dependency plus other
workspace changes), `Cargo.lock` (concurrent updates), `src/main.rs`
(bootstrap-pin registration plus remote-live and other CLI work),
`adr/README.md` (0086 row plus other ADR rows), and
`config/operator-surfaces.ncl`, `config/operator-surfaces.json`,
`config/operator-command-descriptors.json`,
`config/operator-command-catalog.json`,
`docs/generated/operator-command-reference.md`, and
`docs/generated/canonical-operator-workflow.md` (combined operator rows,
then transferred to the attestation owner for final regeneration).
Preserve the `crunch-project-core` root dependency, bootstrap-pin CLI
dispatch, both command families' inventory rows, and all other owners'
changes when integrating. Four scratch-only Python proof/overlay scripts,
their generated Python bytecode, and the two disposable CLI smoke fixture
roots were removed after the above receipts were retained; the detached
baseline worktree itself was not reset or archived.
