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

## Full flake check

```text
nix flake check --no-build -L   -> exit 0, "all checks passed!"
nix flake check -L              -> exit 1 at
  checks.x86_64-linux.durable-file-publication-adoption
```

The failing check is unrelated to the remote entry-point change or the Octet
profile; it runs three Nickel commands on `evidence/radicle/`, `lib/`, and
`tests/` durable-file-publication files. The same three commands pass when run
directly with the pinned Nickel 1.17.0, the flake source contains all three
files, and the derivation produces no builder log. Its failure is recorded as a
separate repository-check issue, not as a remote-entry-point defect.

## Durable-file-publication check follow-up (2026-11-03)

The full flake failure was traced to stale manifest bindings in the adoption
evidence: `Cargo.toml`/`Cargo.lock` changed after the evidence was recorded
(`7f48786b3`), and this change's flake updates changed `flake.nix`/`flake.lock`.
The Nickel source, exported JSON, adjacent checksum, receipt constants, and
prose digests were rebound (`ef099dc6b`). A direct replay of the check's exact
command sequence with pinned Nickel/jq/b3sum/ripgrep passes
(`DURABLE_CHECK_MANUAL_PASS`).

The check derivation itself could not be scheduled on this host: repeated
`nix build` attempts waited behind the daemon/GC lock until timeout (600-900 s)
under a load average above 40, with hundreds of daemon handlers. That is a host
resource limitation, not a source failure.
