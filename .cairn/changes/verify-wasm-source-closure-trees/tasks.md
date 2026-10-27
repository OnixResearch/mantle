# Tasks: Verify Wasm source-closure trees

Implementation items describe authored changes. Pending execution, lifecycle,
and publication items MUST NOT be checked from code inspection alone.

## Provider repair

- [x] [serial] Reuse bounded-tree observation and Mantle v1 BFS/native-name framing for directory source-closure measurement; preserve schemas, fixed non-claims, regular-file roles, and duplicate digest/length agreement. r[mantle.wasm_source_closure_trees.identity] r[mantle.wasm_source_closure_trees.compatibility]
  - Implementation: `crates/mantle-wasm-consumer-verifier/src/source_tree.rs`, `src/shell.rs`; pure core and producer unchanged.
- [x] [serial] Retain opened source/destination capabilities and implement source-only create-new copy with no-follow nonblocking file streaming, empty-directory preservation, and revalidation. r[mantle.wasm_source_closure_trees.capabilities] r[mantle.wasm_source_closure_trees.copy]
  - Implementation: `ConsumerRoot::copy_source_closure`, `source_tree::{open_parent,open_regular,stream_file,prepare,measure}`; `MaterializationBundle` directly re-exported.
- [x] [serial] Author consumer-visible producer parity, drift/type/path/bound/copy regressions and extend the real pinned pipeline rail without rewriting bundle identities. r[mantle.wasm_source_closure_trees.bounds] r[mantle.wasm_source_closure_trees.pipeline]
  - Verification markers: `crates/mantle-wasm-consumer-verifier/tests/consumer_fixtures.rs`, `tests/wasm_component_cli.rs`; not yet execution evidence.
- [x] [serial] Document unchanged framing, supported path policy, bounds, no-atomic-snapshot limitation, current Molten intake consumer, and Mantle maintenance ownership. r[mantle.wasm_source_closure_trees.compatibility]
  - Documentation: crate README and `examples/projects/wasm-component-hello/README.md`.

## Execution and publication

- [x] [serial] Publish/pin the bounded-tree nonblocking member-open fix and resolve dependency locks before provider gates. r[mantle.wasm_source_closure_trees.capabilities]
  - Evidence: [immutable prerequisite and observed remote archive publication](evidence/2026-10-27-verification.md#immutable-bounded-tree-prerequisite); Cargo/Nix retain tested implementation `250f3b25d0933c2fa106aa82d1cc17a496716f06`.
- [x] [serial] Run focused verifier tests, no-std check, changed-package formatting/Clippy and the actual CLI/pipeline fixture; retain exact commands, source identities, output, and any limits. r[mantle.wasm_source_closure_trees.pipeline] r[mantle.wasm_source_closure_trees.bounds]
  - Evidence: [executed focused checks](evidence/2026-10-27-verification.md#executed-focused-provider-checks): 26 verifier fixtures, no-std, strict package Clippy and formatting passed; one actual production CLI/pipeline case passed. Broader-gate attempts remain separately recorded and are not claimed passing.
- [x] [serial] Exercise published API against the unchanged retained real producer handoff, copy its source closure, and admit the same pinned bytes into Molten's private snapshot with complete report agreement. Record only intake success, not runtime adoption. r[mantle.wasm_source_closure_trees.copy] r[mantle.wasm_source_closure_trees.pipeline]
  - Evidence: `evidence/development-producer.json` replays the unchanged retained bundle and a fresh real pipeline; all 37 members match. `evidence/development-intake.json` exercises actual Molten snapshot acceptance, empty-directory drift denial and lease cleanup with an explicitly unadmitted intake-only profile. The supported production-world pipeline separately remains denied by its unchanged Octet artifact policy; this is not production runtime adoption or an immutable-consumer-pin claim.
- [ ] [serial] Run Cairn validate and proposal/design/tasks gates; record evidence and commit implementation before authorized sync/archive. r[mantle.wasm_source_closure_trees.compatibility]

Archive/publication and the exact immutable Molten consumer pin remain required
delivery closeout operations, recorded from their executed command results after
the implementation commit. They are not prerequisites to their own archive;
checking the implementation gate does not claim publication or consumer pinning.
