# wrpc wasm-profile blocker resolved (2026-11-03)

Scope: the Octet `wrpc-reference` profile that blocked this change's T4.2 Nix
check now exists, and Mantle selects it for the wrpc reference pair.

## Octet

- Branch `cairn/add-wrpc-reference-artifact-profile`, commit
  `f081ed9682d478e585f4cede89bb89b542755ab9`.
- Additive `wrpc-reference` profile: server host import
  `component@3::import-func-hello::Func(0)`, client handler instance import
  `component::wrpc-examples:hello/handler::Instance(0)`, proposals
  `wasm1`/`component-model`/`bulk-memory`/`call-indirect-overlong`, two import
  slots, two tables, recorded-only non-claims.
- Normalized server and client fixtures admitted (`status: passed`, profile
  identity `f7c26c93…`); the portable baseline still denies both.
- Nickel export byte-identical; `cargo test -p cargo-octet --lib wrpc_reference`
  passes; Clippy and formatting pass; pinned `cairn validate` and
  proposal/design/tasks gates pass.

## Mantle

- Octet pin `f081ed9682d478e585f4cede89bb89b542755ab9`.
- The toolchain records and verifies four selectable profiles (`portable`,
  `molten-baseline`, `molten-effects`, `wrpc-reference`), seeds the wrpc fixture,
  and `mantle-wrpc-author` requests the wrpc-reference identity.
- `nix build .#wrpc-reference-bundle` now succeeds for both roles; the Octet
  receipts show `wrpc-reference` `passed`.
- `cargo test -p crunch-wasm-component` (18) and
  `--test wasm_component_cli` (3) pass; Clippy and formatting pass.
- `nix flake check --no-build -L` evaluates all checks ("all checks passed").

## Non-claims

- T4.2's full Nix check result is recorded separately when `nix flake check -L`
  completes; T4.3 acceptance/archive remains open.
