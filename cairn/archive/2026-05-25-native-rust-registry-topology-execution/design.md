# Design: Native Rust registry topology execution

## 1. Source facts become execution inputs

`native_registry_source_planning.ready=true` is a prerequisite for treating a registry-backed package as a native execution input. The execution rail must carry the package's lockfile identity, checksum material, declared vendor/source root, source digest, and source-class evidence into the same deterministic facts that native path packages already use.

If a registry package is referenced by the unit graph but has no ready native registry source fact, the topology executor must produce a deterministic blocker before invoking `rustc` for the affected unit or consumer.

## 2. Native graph integration stays bounded

The initial supported shape is intentionally narrow:

- a local/path root package;
- a vendored registry-backed `lib` dependency with a supported Cargo oracle unit;
- declared `.cargo/config{,.toml}` directory source replacement;
- `Cargo.lock` checksum evidence;
- direct `rustc` execution through the existing topology rail.

The implementation should reuse existing DTO/receipt patterns rather than adding a second execution path. Registry-backed packages should appear as native package/unit/derivation facts only when their source facts are ready and comparable to Cargo oracle material.

## 3. Execution receipts identify registry evidence

The combined CLI JSON receipt should preserve the retained `rust_plan.native_registry_source_planning` summary and the topology execution receipt. The executed registry-backed unit receipts should bind source digest and artifact digest material so reviewers can trace the execution claim from lockfile/vendor evidence to produced `.rlib` outputs.

## 4. Fail closed, no ambient Cargo repair

Blockers must be deterministic and pre-execution for:

- missing lockfile checksum or registry identity;
- missing/unreadable vendor root;
- missing vendored package directory/manifest/source;
- stale or mismatched source digest/oracle material;
- unsupported registry source/layout/target kind;
- any need to consult `$CARGO_HOME`, Cargo registry caches, target directories, git checkouts, or the network.

The rail may retain Cargo metadata/unit-graph as oracle evidence only; it must not invoke Cargo as build orchestrator.

## 5. Verification fixtures

Add one positive CLI fixture with a local root crate depending on a vendored registry-style library through declared directory replacement. Add one negative fixture that removes or corrupts the vendor source material and proves zero affected registry-backed unit executions plus a stable blocker class/message.
