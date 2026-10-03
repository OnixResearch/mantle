# Nominal dynamic-plan types

Mantle keeps the `mantle-plan-v1` JSON contract structural. It admits that wire data into a typed core before graph validation.

```text
JSON bytes
  -> WireDynamicPlanV1
  -> admit_plan_v1(..., store_prefix)
  -> DynamicPlanV1
  -> graph validation and canonicalization
  -> WireDynamicPlanV1 projection
```

## Core types

The admitted model uses these private types:

- `UnitId` for unit and root identities
- `SourceId` for declared source identities
- `StorePathString` for checked logical store paths
- `OutputName` for derivation output names
- `PlanDigest` for canonical plan identity
- `NarDigest` for declared source NAR identity

Each type has a fallible constructor and an `as_str` accessor. The types do not implement `Deref` or unrestricted `From<String>` conversions.

The graph keeps these types in sets, maps, dependencies, roots, and placeholder bindings. It converts them to text only for wire output, diagnostics, and existing scheduler or store adapters.

## Wire and admission APIs

`decode_plan_v1` and `decode_wire_plan_v1` return `WireDynamicPlanV1`. These functions enforce the byte, JSON-depth, field, and required-nullable limits.

`admit_plan_v1` checks semantic scalars and returns `DynamicPlanV1`. Invalid IDs, output names, logical store paths, and NAR digests cannot enter graph validation.

`decode_validated_plan_v1` combines bounded decoding, admission, graph validation, canonicalization, and plan digest calculation.

Canonical serialization always projects the admitted model through `WireDynamicPlanV1`. This preserves the current field names, enum tags, null values, collection shapes, canonical bytes, and BLAKE3 plan identity.

## Rust consumer migration

Code that only reads JSON can continue to call `decode_plan_v1`. Its result is now the explicit wire DTO.

Code that needs graph-ready data must call `admit_plan_v1` with the active logical store prefix. Code that needs the canonical admitted result can call `decode_validated_plan_v1`.

Construct new core values with `UnitId::new`, `SourceId::new`, `StorePathString::new`, `OutputName::new`, and the role-specific digest constructors. Do not convert checked values back to strings inside core graph logic.

## Claim boundary

These types prove local category separation and scalar admission for the supplied values. They do not prove store presence, source trust, build success, sandbox enforcement, compiler correctness, or release eligibility.

## Version 2 source slices

`decode_validated_plan_v2` accepts the separate `mantle-plan-v2` schema and
canonicalizes its source entries by source id. A slice entry has exactly
`id`, `producer_output`, `subpath`, `store_name`, and required lowercase
64-character `nar_blake3` fields. Existing store-path source entries remain
available in v2. The v2 canonical digest covers the declared slice digest.
An identical repeated slice id collapses to one admitted source; conflicting
entries for one id are rejected before graph admission.
The v1 DTO, serializer, golden bytes, and plan digest are unchanged; v1
rejects slice fields.

The pure v2 planner (`dynamic_plan::plan_slices`) takes declared
producer output names and observed subtree facts supplied by its caller.
It orders candidate slices by source id and returns **no candidate batch**
on a rejected slice. Each candidate carries a `publication_source_id`: the
first canonical source id with the same store name and observed NAR digest.
Distinct ids sharing that key select one future publication; the planner
neither derives a logical store path nor performs publication itself.
Named limits: 256 slices, 4,096 UTF-8 bytes per
relative subpath, 32 components, 64 bytes per Nix-compatible store name,
and 1 GiB aggregate admitted NAR bytes per plan. A subpath cannot be
absolute, empty, contain `.` or `..` or empty components, a backslash,
or a NUL. Pure tree-fact rejection kinds are
`slice-output-undeclared`, `slice-absent`, `slice-symlink-traversal`,
`slice-digest-mismatch`, `slice-limit`, and `slice-conflict`.
Invalid subpaths fail scalar admission with `slice-subpath-invalid`.

The native worker recognizes v2 only on declared regular plan outputs. After
bounded decoding, its read-only store view walks each named producer output
subpath through castore without following a symlink, renders the selected NAR,
and observes BLAKE3, SHA256, and size. The plan output itself cannot supply a
slice. The pure planner verifies every declared BLAKE3 and the aggregate
admitted-byte limit before any source publication or unit registration.

The Builder retains its signing key and a separate `SliceAdmission` capability;
`BuildStore` can observe subtrees but cannot publish verified sources. For each
canonical publication owner, the worker projects the content-addressed path
from the observed NAR SHA256 and declared store name; source ids with identical
content and name bind to that one projected path without writing a PathInfo.
Against a read-only view of the live registry plus staged V2 units, it then
resolves every unit's source and unit dependencies, computes its derivation
and output paths, checks complete duplicate identities and registry capacity,
and preflights the complete root-goal closure and its capacity. A rejection at
this stage leaves signed slice PathInfos, the live derivation registry, goals,
scheduler state, and accepted-plan reports unchanged.

Only after that whole-plan preflight does `SliceAdmission` validate the signed
batch and commit its new PathInfos in one backend transaction. The worker then
installs the prepared units and root goals. An ordinary pre-commit failure
rejects the plan with no admitted source paths. A publication-uncertain result or
post-commit invariant failure aborts the worker instead of claiming a clean
rejection; the V1 registration path remains separate.

Native-plan report `source_slices` rows are ordered by source id. Each row
records `source_id`, `producer_output`, `subpath`, `declared_nar_blake3`,
`observed_nar_blake3` (when observed), `admitted_store_path` (only after batch
commit), and `disposition`. V2 admission rejections retain rows for every
decoded declared slice, with no admitted store path. V1 reports omit the empty field.

This is a *logical PathInfo transaction*: Snix Redb atomically writes the
batch, but pre-existing castore blobs can remain unreferenced on failure and
an unrelated concurrent signer may overwrite a preflighted digest before
commit. This contract does not promise physical cleanup or cross-process
no-clobber. Neither the pure planner nor signed source publication proves
source trust, producer correctness, build success, or release eligibility.
