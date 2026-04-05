## Context

`input_derivations: BTreeMap<StorePath, BTreeSet<String>>` already maps
each dependency derivation to the set of output names the consumer needs.
`collect_input_paths` iterates those names and resolves each to a store
path via `KnownPaths::get_output_path`. The bwrap sandbox mounts exactly
those resolved paths.

The gap is upstream: `resolve_inputs` in convert.rs always puts *all*
output names from the nested derivation into `input_derivations`:

```rust
let output_names: BTreeSet<String> =
    nested_drv.outputs.iter().cloned().collect();
input_derivations.insert(nested_drv_path, output_names);
```

And the Nickel `Input` type is either a bare string or a full derivation
record — no room for "this derivation, but only output X."

## Goals / Non-Goals

**Goals:**
- Consuming derivations can depend on specific outputs.
- Bare derivation in `inputs` keeps current behavior (all outputs) — no
  breaking change.
- Ergonomic Nickel syntax: `crunch.select pkg "dev"` or equivalent.

**Non-Goals:**
- Output path interpolation at eval time (Nix's `${pkg.dev}`). We don't
  know CA output paths during eval. Builders use `$dev`, `$lib` env vars
  at build time — that already works.
- Multiple output selections per dependency in a single input entry. Use
  two entries or depend on all outputs. Simplicity over generality.

## Decisions

### 1. Structured record variant for Input

**Choice:** Add a third `Input` serde variant: a record with `drv` and
`output` fields. Serde's `#[serde(untagged)]` tries each variant in order:
1. `String` → `Source`
2. Record with `drv` + `output` fields → `OutputSelection`
3. Record with `name` + `builder` → `Derivation`

Ordering matters for untagged: `OutputSelection` must come before
`Derivation` because both are records. The distinguishing field is
`drv` (nested record) vs `name` (string).

**Rationale:** Nickel records serialize naturally as JSON objects.
A wrapper `{ drv = pkg, output = "dev" }` is unambiguous and trivial
to destructure on the Rust side. No new syntax or contract magic needed.

**Alternative rejected:** Adding an `output` field to `CrunchDerivation`
itself. This would mean the same derivation record appears different
depending on how it's consumed, which breaks identity — the same package
would hash differently when consumed for `dev` vs `out`. Output selection
is a property of the *edge* (consumer → dependency), not the node.

**Implementation:**

Rust:
```rust
#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum Input {
    Source(String),
    OutputSelection(Box<OutputRef>),
    Derivation(Box<CrunchDerivation>),
}

#[derive(Debug, Clone, Deserialize)]
pub struct OutputRef {
    pub drv: CrunchDerivation,
    pub output: String,
}
```

Nickel:
```nickel
# In contracts.ncl, extend Input:
Input = fun label value =>
  if std.is_string value then value | StorePath
  else if std.is_record value && std.record.has_field "drv" value then
    value  # OutputSelection — validated at build time
  else
    value | import "derivation.ncl"
```

### 2. Stdlib `select` helper

**Choice:** `crunch.select : Derivation -> String -> { drv, output }`

```nickel
let select = fun drv output_name => { drv = drv, output = output_name } in
```

Usage:
```nickel
{
  inputs = [
    crunch.select openssl "dev",  # only headers
    seed.gcc,
  ],
}
```

**Rationale:** A function is simpler than a contract or special syntax.
The returned record is just data — it serializes through `to_serde()`
without any special handling.

**Alternative considered:** `pkg.select "dev"` as a method on derivation
records. Requires adding a `select` field to every derivation, marked
`| not_exported`. Feasible but adds weight to every derivation record
for a feature most won't use. A standalone function is lighter.

### 3. resolve_inputs routing

**Choice:** Pattern match on `Input::OutputSelection` in `resolve_inputs`,
extract the single output name, convert the inner derivation recursively,
then insert into `input_derivations` with `BTreeSet::from([output_name])`.

If the same derivation appears twice (once selecting `dev`, once
selecting `lib`), the `BTreeMap::entry` API merges the output sets:

```rust
input_derivations
    .entry(drv_path)
    .or_default()
    .insert(output_name);
```

This matches how Nix handles multiple output references to the same
derivation — they coalesce in `input_derivations`.

**Assertion:** the selected output name must exist in the derivation's
`outputs` list. Fail at convert time, not build time.

## Risks / Trade-offs

**[Serde ordering sensitivity]** Untagged enum variant ordering is
fragile. A record with both `drv` and `name` fields could match either
`OutputSelection` or `Derivation`. Mitigation: `OutputSelection` tries
first; `CrunchDerivation` requires `name` + `builder` which won't appear
in an `OutputRef`. Tested explicitly.

**[Single output per entry]** Users who want both `dev` and `lib` from
the same dep need two input entries or omit the selection (get all).
Acceptable — multiple selections are rare, and the coalescing logic
in `resolve_inputs` handles duplicates.
