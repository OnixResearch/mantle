# Full-bootstrap Cairn scaffold validation — 2026-07-25

Status: scaffold-only. No implementation, provider, lineage, fixed-point, parity, or release claim is made by this document.

## Goal and completion contract

The scaffolds decompose the remaining work into independently reviewable changes whose observable terminal evidence is: current source-built native row receipts, a Rust provider built by the admitted native provider, a real StageX lineage receipt, a clean source-to-provider-to-Mantle v2 fixed point, and independent parity/release verification. Build-witness quorum is not a bootstrap completion criterion. Checked task boxes, historical output directories, scaffold receipts, provider smoke alone, one-shot Mantle builds, imported provider outputs, witness counts, or model agreement are false completions.

## Dependency order

1. `bind-full-source-rust-provider` and `close-early-native-bootstrap-parity` may start independently.
2. `close-final-native-toolchain-parity` depends on the early native parity change.
3. `materialize-stagex-lineage-provider` depends on both native parity changes.
4. `prove-source-built-mantle-fixed-point` depends on the Rust binding and StageX materialization changes.
5. `promote-full-bootstrap-parity` depends on every construction/proof change above.

This graph is acyclic. No predecessor depends on the promotion change.

## Portfolio review

| Family | Mechanism | State | Exact boundary |
|---|---|---|---|
| Rust integration | Rebuild mrustc-to-Rust against admitted native provider | active | musl compiler-host probe first; source-built GNU closure required for any GNU-host alternative |
| Native parity | Complete early and final stage-local artifacts/receipts | active | later provider admission cannot substitute for predecessor evidence |
| StageX lineage | One-way protected hex0-to-provider materialization | active | scaffold receipt and unbound host executable fallback are rejected |
| Fixed point | Fresh source-only provider construction plus stage1-to-stage2 Cargo-free rebuild | active | imported or cache-hit provider outputs cannot satisfy promoted proof |
| Promotion | Independent parity/release checker | blocked on predecessors | producer status and repeated digest text are not authority |

Adversarial model review suggested a dependency cycle, missing generated-source/binutils coverage, host fallback for the configure bridge, and narrower fixed-point scope. Deterministic inspection rejected the first three: the graph above has no cycle; the final-native change explicitly owns generated sources and final binutils; host fallback would violate the accepted bridge confinement and full-source goal. The review did reveal one real false-completion path: provider cache hits in the promoted fixed-point run. The fixed-point design, tasks, and requirement now require empty provider/output authorities and current in-proof provider construction.

## Validation evidence

Baseline before scaffold creation:

- pueue task `377`: `nix run path:/home/brittonr/git/OnixResearch/cairn#cairn -- validate --root .` returned `valid: true`.

After writing and adversarially tightening all six changes:

- pueue task `396`: local current Cairn validation returned `valid: true`.
- pueue task `397`: proposal, design, and tasks gates for all six changes completed with zero command failures.
- pueue task `398`: all 18 individual gate JSON outputs were captured under ignored `target/cairn-scaffold-gates/`; every output records `verdict: PASS`.
- pueue task `399`: canonical Nix-run Cairn validation returned no substance issues/findings and `valid: true`.
- pueue task `400`: `git diff --check` passed; status showed only the new active `cairn/changes/` tree.

## Non-claims

These packages are executable plans, not implementation evidence. They do not prove compiler correctness, seed correctness, StageX materialization, source-built Rust integration, Mantle fixed-point equality, parity completion, release reproducibility, optional-witness validity, witness quorum, independent rebuild agreement, deployment, or full Cargo compatibility.