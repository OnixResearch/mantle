# Nickel stdlib contract shape defect (pre-existing)

Status: open, pre-existing, blocks Nickel evaluation of the standard library.

## Finding

The standard library does not evaluate under Nickel 1.17.0 (the version the
repository pins). `nickel eval --import-path lib lib/lib.ncl` fails, and the
failure is a contract-shape rule, not a syntax error. The rules were
confirmed directly against the pinned interpreter:

- A record contract may carry **at most one field without a definition**. A
  field written as `name | Type` with no `| default = ...`, no `| optional`,
  and no `= value` shorthand is undefined; a second undefined field is
  rejected with `missing definition for <field>`, and the error names the
  first undefined field.
- A contract field may only be annotated with a **builtin or enum type**. A
  validator contract (`std.contract.from_validator`) or a nested record used
  as an annotation is rejected with the same error, so a nested shape cannot
  be expressed inside a record contract at all.
- A **record type** (`{ name : Type }`) admits any number of required
  fields, is closed, and can be applied as a contract. It cannot carry `| doc`
  annotations.
- A record **contract** is closed as well, and a catch-all field must be
  written `{ _ : Type }`; `{ _ | Type }` is a syntax error.
- Nickel records are **recursive**, so an export record such as
  `{ Plan = Plan }` refers to the field itself and fails with
  `infinite recursion`.

The repository's plan records break the first two rules: `Derivation`,
`ManifestInput`, the WebAssembly component records, the kernel-script
experiment records, and the reconciled adapter plan records all declare
several required fields and annotate fields with validator contracts or
nested records.

Two further defects are in the same family:

- `lib/android.ncl` embedded a shell `printf` format that contained a double
  quote immediately followed by a percent sign. In a Nickel multiline string
  that sequence ends the string, so the module failed to parse. Fixed in
  commit `fb2f8b438` by quoting the operand with single quotes.
- The reconciled adapter modules (`lib/android.ncl`, `lib/android/sources.ncl`,
  `lib/cmake.ncl`, `lib/esp_idf.ncl`, `lib/fpga.ncl`, `lib/gpu.ncl`,
  `lib/zephyr.ncl`) exported bindings whose field name shadowed the exported
  value, for example `{ Plan = Plan }`. Nickel reports that as an infinite
  recursion.

## Scope

The defect exists in the published baseline `43ccd6884`, not only in the
reconciled tree: `git archive 43ccd6884 lib` fails the same way, first at
`lib/android.ncl` and then at the same rule in the remaining modules. Every
Nickel-evaluating test in the ordinary gate fails for this reason, which is
why the gate has never passed on this tree.

Affected modules and the number of contract annotations that Nickel rejects:

| Module | Invalid annotations |
|---|---|
| `lib/derivation.ncl` | 1 required-field record (`Derivation`) plus nested shapes |
| `lib/kernelscript_experiment.ncl` | 35 |
| `lib/project.ncl` | 3 |
| `lib/wasm_component.ncl` | 38 |
| `lib/android.ncl` | 7 |
| `lib/android/sources.ncl` | 9 |
| `lib/cmake.ncl` | 8 |
| `lib/esp_idf.ncl` | 8 |
| `lib/fpga.ncl` | 17 |
| `lib/gpu.ncl` | 7 |

## Required repair

1. Restate the plan records as record types (`{ field : Type }`), keeping the
   validator contracts as field types.
2. Provide a whole-record validator per plan shape and export it under the
   published name, so `value | Derivation` and the adapter equivalents keep
   working. A record type cannot be applied as a contract, so the validator
   performs the field-presence and field-type checks itself and delegates each
   field to its own field contract where one exists.
3. Apply the field validators inside those validators so the shape and value
   checks are not lost.
4. Keep the Rust core as the admission authority: it consumes the evaluated
   plan and enforces the real contract.

## Repair

The repair is recorded in
`adr/0124-express-nickel-plan-shapes-as-record-types.md` and tracked by the
Cairn change `repair-nickel-contract-shapes`. A shape whose fields are all
required and carry no defaults becomes a record type. A shape with defaults
or optional fields becomes `std.contract.Sequence` of an open defaults
contract and a whole-shape check built on `contracts.Shape`, which strips the
defaulted and optional fields and applies the required record type to the
remainder. Every field name, default, requiredness, closedness rule,
validator, and exported name is preserved.

## Observed effect in the ordinary gate

The authoritative gate transcript for revision
`13627a62f4293ec42fab6971cbe67d1350da7f0e` is retained in
`.cairn/changes/reconcile-main-api-lineages-2026-10-17/evidence/final-gate-v1/`.
The bounded GCC 4.0 configure bridge, the canonical first-party rustfmt, and
strict Clippy all pass; the serialized workspace lib/tests phase fails first in
`crunch-esp-idf --test nickel_contract`, whose two cases reject even their
baseline plan because the plan contract cannot be evaluated.

## Non-claims

- This note does not claim that the standard library is correct, complete, or
  release eligible. It records why Nickel evaluation fails today and what the
  repair must cover.
- It does not claim a bootstrap, reproducibility, or parity result.
