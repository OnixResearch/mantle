# ra-jit feasibility report

Status: feasibility complete, adoption declined. This report answers issue #23
("Evaluate ra-jit as an isolated opt-in Rust development backend") with the
measured evidence; it is not a Mantle integration, a performance claim about
Mantle, or a release claim.

## Pinned inputs

| Input | Identity |
|---|---|
| ra-jit | revision `6a7d245e47830f26475e98db6e01905f117a2971` |
| Mantle | revision `43ccd68840b748e03773fc1bff5eb1cfca99dffa` |
| ra-jit executable | 72,401,720 bytes, SHA-256 `59d391164e6e8667c5456174aca945dd7a3ec7acd2956601691b160884f3f150` |
| `cargo-ra-jit` | 506,856 bytes, SHA-256 `f2aeb2d46836b57912cec317dcfdb57eb6acbabc4df91bed24024c284ca0224d` |
| `ra-jit-rustc` | 509,128 bytes, SHA-256 `dca6da9002bc10fa43889d6391a4c5580d0b714bf63a939fff8db6a72f569690` |
| host toolchain | `rustc 1.96.0-nightly (c75612477 2026-04-07)` |

Bounds for the measured runs: 16 GiB memory maximum, 1 GiB mutable work tmpfs,
64 MiB temporary tmpfs, 600 s outer guard, 16,384 block per-file limit, 128
task maximum, CPU affinity `[0, 1, 2, 3]`.

## Boundary results

### Section 1: feasibility against the supported fragment

- The upstream support contract is Rust 1.99.0 only and excludes nightly
  features. Mantle's checked-in toolchain is `nightly`
  (`rust-toolchain.toml`), and the cohort measured here is
  1.96.0-nightly. Mantle's own workspace is therefore outside ra-jit's
  supported compiler range; compatibility is not established, and the issue
  requires a supported source fragment rather than a best-effort one.
- The experiment ran on a small stable-compatible Linux x86-64 fixture with a
  pinned, bounded environment: no ambient compiler, Cargo home, network, or
  host-executed fallback was used, and the daemon/worker ran under the
  declared memory, tmpfs, task, and wall-clock limits.
- Native live patching is documented for macOS on Apple silicon. Mantle's watch
  executor is Linux-only, so on Linux the available behavior is a bounded
  rebuild-and-splice, not in-process hot reload.

### Section 2: correctness and trust boundaries

Measured on the pinned fixture, with the ordinary pinned `rustc` check of the
same input snapshot executed before any edited program was exposed:

| Case | Observed outcome |
|---|---|
| cold build | check 32.2 ms; check+build+execute 298.6 ms; `answer=42` |
| identical warm 1-3 | 66.0 ms, 49.8 ms, 49.8 ms; identical stdout/stderr/exit |
| leaf body edit | 148.3 ms; `answer=43` |
| signature edit | 198.2 ms; `answer=42` |
| borrow-check rejection | rejected before backend build or edited-executable exposure |
| compiler failure | rejected before backend build or edited-executable exposure |
| unsupported lowering | refused: `asm!` with a `sym`; no backend executable exposed |
| stale fallback object | refused; no backend executable exposed |
| explicit ordinary fallback | executed once through ordinary `rustc` |
| client cancellation | candidate discarded, no publication; retained generation unchanged |
| edit during build | candidate discarded because the input changed |
| worker crash (SIGKILL) | last complete generation retained; restart rebuilt in 5.77 s |
| failed publication | read-only bind denial, errno 30; no mixed generation exposed |
| daemon restart | retained generation reused; 5.77 s to the next successful build |

Backend refusal accounting is retained from the worker log: four refusal
events across two unique function names, reported as events rather than as
unique machine-code instances.

Shared-dependency coverage was completed in a separate five-package
compilation-only diamond run: shared cold, identical warm, and shared-dependency
edit each produced `shared-dependency-reuse = 42` with two daemon builds and no
Cargo builds.

### Section 3: comparison against existing alternatives

Not measured in this campaign. The issue requires the accepted #15 final-source
baseline and the #2 source-bound harness for a like-for-like comparison of
ordinary Cargo, Mantle's compiler cache, the unit-plan lane, and this backend.
Those subjects did not exist when the feasibility runs executed, so no
comparison numbers are claimed here. The decision below does not depend on them.

## Decision

Do not integrate ra-jit as a Mantle development backend.

Reasons, in order of decisiveness:

1. The supported fragment is empty for this repository: ra-jit supports Rust
   1.99.0 without nightly features, while Mantle builds on nightly. Integrating
   it would require a second compiler identity and a second trust path.
2. The offered speedup is a rebuild-and-splice bound, not a Mantle-equivalent
   edit/run loop, and the measured end-to-end numbers (49.8-198.2 ms on a tiny
   fixture) cannot be transferred to Mantle's own workspace.
3. Adopting it would introduce a second scheduler and a second reuse-key space
   next to the existing compiler cache and the Cargo unit-plan lane, which the
   issue explicitly excludes.
4. The trust boundary that matters for Mantle - a signed, sandboxed, reproducible
   artifact - is not provided by a JIT cache hit; a JIT signature confers no
   Mantle trust and no release eligibility.

A negative feasibility result is the issue's own accepted outcome when the
boundaries cannot be met, so no backend, default switch, cache service, or
integration work follows from this report.

## Evidence locations

- `runs/ra-jit-experiment-20261020-v8/summary.json` and
  `runs/ra-jit-experiment-20261020-v8/experiment/summary.json` - bounded fixture
  run, refusal accounting, crash/cancellation/publication cases.
- `runs/ra-jit-diamond-20261020-v1/summary.json` and
  `runs/ra-jit-diamond-20261020-v1/experiment/summary.json` - shared-dependency
  diamond.
- `runs/ra-jit-teardown-20261020-v1/` - bounded process teardown evidence.
- `runs/ra-jit-isolation-20261020-v1/` and
  `runs/ra-jit-source-identities-20261020-v1/` - runtime closure and source
  identities.

## Non-claims

- These runs are a private, isolated experiment on a tiny fixture. They are not
  Mantle integration evidence, not a Mantle performance measurement, and not a
  release or bootstrap claim.
- No Mantle build path, store admission, signed publication, or replay path was
  exercised through ra-jit.
- The absence of a Section 3 comparison is not a performance result; it is an
  unmeasured comparison.
- Upstream's own den-native save-to-splice figures are upstream numbers and
  are not Mantle measurements.
