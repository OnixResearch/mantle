# V27 root-tool source closure repair

## Goal

Continue the source-built fixed-point proof after the receipt repair and retain
the next exact blocker.

## V27 authority

- Source commit: `bb31cef7`.
- Release orchestrator BLAKE3:
  `88bf29c484904a58d065d154be71ce49af12798b5b4094cacd81f47bb0c1ae20`.
- Source transfer: checksum-identical after rsync.
- Refreshed source-profile BLAKE3:
  `15ee4e3fbbd041eb6b134c0d39213fead45ab5e67c0af52a3be75f3c79a62ea9`.
- Replacement Mantle source BLAKE3:
  `8a4ce7a486c2ea924ed4f3e131601ceb1ad754e36556dad037a4300bef3b9a18`.
- Profile verification: Ready with zero missing, stale, unsupported, or
  untrusted records.

## Result

V27 ran on leviathan from 06:15 to 20:47 EDT. It completed:

- protected StageX;
- full-source native-provider admission with digest
  `63d9bc23cfcc232726527c141132ea35bc7ef9adf4ed7a952b5baa62bd466ed9`;
- all six source-built host tools;
- mrustc to Rust 1.90 and Cargo;
- Rust 1.91, 1.92, 1.93.1, and final Rust 1.94;
- source-built closure enforcement with 17 members and zero seed exceptions.

Stage1 native planning then failed closed:

```text
Cargo-free native target planning failed: target `generate-operator-command-contract` source .../inputs/mantle-source/tools/generate_operator_command_contract.rs is not readable
```

The hydrated Mantle source had no `tools/` directory. The transferred source
contained the file with mode `0755` and size `13,535` bytes.

## Root cause

`source_built_mantle_source_record()` reuses
`self_build::STAGED_SOURCE_TOP_LEVEL_ENTRIES`. That fixed allowlist retained the
root `Cargo.toml`, including its explicit tool target, but omitted the `tools/`
directory that owns the target source.

## Repair

- Added `tools` to the shared staged-source allowlist.
- Updated ordinary self-build staging coverage to require the root tool file.
- Updated source-profile generation coverage to require the tool file.
- Updated source-profile refresh coverage to require the replacement tool file.
- Kept `vendor-deps/` and unrelated top-level files excluded from the Mantle
  source record.
- Updated `AGENTS.md` so future root Cargo target directories stay aligned with
  the shared allowlist.

## Validation

Before the repair, both focused baseline tests passed but did not check the
root tool target.

After the repair:

- self-build staging test: `1 passed; 0 failed`;
- source-built profile test: `1 passed; 0 failed`;
- source refresh tests: `4 passed; 0 failed`;
- isolated focused Clippy: passed;
- `git diff --check`: passed.

See `local-validation.log` for the exact commands and output.

## Non-claims

V27 did not reach stage1 execution, stage2, the fixed-point comparison, or final
receipt creation. A refreshed profile and fresh proof are required after this
repair. This evidence does not claim compiler correctness, release eligibility,
or full Cargo compatibility.
