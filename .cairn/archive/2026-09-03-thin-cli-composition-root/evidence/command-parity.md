# Command parity

Date: 2026-09-03

The final Mantle binary was built from the current worktree. The checked Rust capture tool then ran the same 12 command cases as the baseline.

Commands are in `evidence/validation-2026-09-03/command-parity.command.txt`.

## Result

- Cases: 12
- Help cases with status `0`: 10
- Invalid-input cases with status `2`: 2
- Manifest BLAKE3 before: `04993d806d317478dd5e4b6d5011e71afa158c2f572149f591605dfcbf9b6fa6`
- Manifest BLAKE3 after: `04993d806d317478dd5e4b6d5011e71afa158c2f572149f591605dfcbf9b6fa6`
- Recursive byte comparison: no differences
- Diff file: `evidence/validation-2026-09-03/cli-family-parity.diff` (zero bytes)

The checked after-files are in `evidence/validation-2026-09-03/cli-family-parity-after/`. They preserve each stdout stream, stderr stream, status code, and BLAKE3 value.

The cases cover build, build-plan, Rust-plan, remote, store, source, release, project, bootstrap, artifact, invalid build input, and conflicting machine-output modes.

This comparison proves byte parity for the checked cases. It does not prove all command behavior or external effect success.
