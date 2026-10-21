# Design: Repair the Nickel standard-library contract shapes

## The pinned interpreter's rules

All three rules were confirmed directly against Nickel 1.17.0, the version the
repository pins.

1. A record contract may carry at most one field without a definition. A
   field written as `field | Type` with no `| default = ...`, no
   `| optional`, and no `= value` shorthand is undefined. A second undefined
   field is rejected with `missing definition for <field>`, and the error
   names the first undefined field.
2. A contract field may only be annotated with a builtin or enum type. A
   validator contract or a nested record used as an annotation is rejected
   with the same error, so a nested shape cannot be expressed inside a record
   contract.
3. A record type (`{ field : Type }`) admits any number of required fields,
   is closed, and can be applied as a contract. It cannot carry `| doc`
   annotations, and it rejects both missing and extra fields.

Nickel records are recursive, so a record literal whose field value is the
same identifier as the field name refers to the field itself. That is why the
reconciled adapter modules failed with `infinite recursion` on their export
records.

## The repair

Two spellings are used, chosen per shape.

A shape whose fields are all required and carry no defaults becomes a record
type. The conversion is a field-separator change: `field |` becomes
`field :`, including inside nested record literals.

A shape with defaults or optional fields becomes a sequence of two contracts:

```nickel
let XDefaults = {
  <defaulted and optional fields> | <type> | <default or optional>,
  ..
} in
let XRequired = {
  <previously required fields> : <type>,
} in
let X = std.contract.Sequence [
  XDefaults,
  Shape [ <every field name that is not in XRequired> ] XRequired,
] in
```

`contracts.Shape` is the single place that knows the rule. It removes the
listed fields from the value and applies the required record type to the
remainder, so the required fields must be present with their declared types
and no other field may be present. The defaults contract is open so that it
can fill the defaulted fields; the required type is closed, so the shape
stays closed exactly as the pre-repair contract was.

`std.contract.Sequence` applies its contracts left to right and returns the
last result, so the defaults are filled before the shape is checked.

## Why the shape check is a contract, not a type

The required record type cannot be applied to the whole value, because the
value also carries the defaulted fields, and a record type rejects extra
fields. Stripping the defaulted fields first makes the two forms agree: the
stripped value carries exactly the required fields, and the type check then
enforces their presence, their types, and the absence of anything else.

## Preserved behaviour

- Every field name, field order, default value, validator, and check message
  is unchanged.
- A required field stays required: it is listed in the required type, so a
  value without it is rejected.
- A defaulted field keeps its default, and an optional field stays optional.
- A shape stays closed: an undeclared field is rejected.
- The exported names of every module are unchanged, and the per-field
  documentation is preserved wherever the shape has any.
- The Rust glue layer remains the admission authority for every evaluated
  plan. The Nickel shape is an authoring-time check.

## Non-claims

- The repair does not claim that the standard library is correct, complete, or
  release eligible.
- It does not claim a bootstrap, reproducibility, or parity result.
- A passing gate is evidence for the recorded revision only.
