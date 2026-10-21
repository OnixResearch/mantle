# Nickel stdlib contract shape defect (pre-existing)

Status: open, pre-existing, blocks Nickel evaluation of the standard library.

## Finding

The standard library does not evaluate under Nickel 1.17.0 (the version the
repository pins). `nickel eval --import-path lib lib/lib.ncl` fails, and the
failure is a contract-shape rule, not a syntax error:

- A record contract may have at most one field without a definition. A field
  written as `name | Type` with no `| default = ...` and no `| optional` is
  the single required field; every other field needs a definition. The
  repository's plan records declare many required fields, so Nickel reports
  `missing definition for <field>`.
- A contract field may only be annotated with a builtin or enum type. A
  validator contract (`std.contract.from_validator`) or a nested record used
  as an annotation is rejected with the same `missing definition` error.
- A record type (`{ name : Type }`) is the correct spelling for a plan shape.
  A record type cannot be applied as a contract; the shape check belongs in a
  whole-record validator, with the Rust core remaining the admission
  authority.

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
   working.
3. Apply the field validators inside those validators so the shape and value
   checks are not lost.
4. Keep the Rust core as the admission authority: it consumes the evaluated
   plan and enforces the real contract.

## Non-claims

- This note does not claim that the standard library is correct, complete, or
  release eligible. It records why Nickel evaluation fails today and what the
  repair must cover.
- It does not claim a bootstrap, reproducibility, or parity result.
