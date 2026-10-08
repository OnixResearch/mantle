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

Content-addressed units with multiple outputs retain their distinct named
outputs: a dependent v2 unit's placeholder for `dev`, for example, resolves
to the parent's realized `dev` path at dispatch, not its `out` path.

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

## Static derivation inputs bound to plan roots

A static Nickel derivation can consume an exported root from a native
`mantle-plan-v1` or `mantle-plan-v2` without a second evaluation:

```nickel
let mantle = import "lib.ncl" in
let producer = import "producer.ncl" in
{
  name = "consumer",
  builder = "/bin/sh",
  addressing_mode = 'input-addressed,
  inputs = [{
    name = "root_input",
    producer = producer,
    plan_output = "plan",
    root = "unit.root",
    unit_output = "out",
  }],
  env = { ROOT = "{{mantle-plan-output:root_input}}" },
  args = ["-c", "read payload < \"$ROOT\" && echo \"$payload\" > \"$out\""],
} | mantle.Derivation
```

`producer` must be an inline derivation and `plan_output` one of its declared
`dynamic_plan_outputs`. `root` is an exported plan root unit id, not an
internal unit; `unit_output` must be declared among that unit derivation's
outputs. At most 16 differently named references are allowed per consumer.
The exact `{{mantle-plan-output:<name>}}` marker can occur in argument and
environment values. A marker naming no input, or a malformed or unterminated
marker, fails conversion.
Markers in environment keys overwritten by derivation registration are
rejected rather than silently lost. The reserved
`__MANTLE_PLAN_OUTPUT_BINDINGS` environment value is compact JSON ordered by
input name: each record contains `name`, `producer_drv_path`, `plan_output`,
`root`, `unit_output`, and a deterministic `placeholder`. The selected
producer plan output is also a normal derivation input edge. These stable
request facts determine the consumer's derivation and input-addressed output
paths before the plan runs; no output path from a content-addressed root is
needed during evaluation. Derivations without these inputs keep their
existing identities.

After the producer succeeds, the worker accepts and validates its plan,
resolves exported root membership and output membership, waits for the root
goal, then substitutes the *realized* root output path into the consumer's
arguments and environment at dispatch. Bound root output paths enter the
sandbox with their closures and participate in reference scanning. No
unresolved reference can dispatch. Failure reasons in the build report are
`plan-output-invalid`, `plan-output-bounded`,
`plan-output-producer-failed`, `plan-output-plan-rejected`,
`plan-output-undeclared`, `plan-output-root-missing`,
`plan-output-output-missing`, `plan-output-root-failed`, and
`plan-output-unbound`; a failed bound consumer has no successful output.
One failing reference is selected in input-name order. The build JSON
`plan_output_bindings` rows expose each reference's declared request,
accepted plan digest and resolved root/output where available, status, and
failure reason. An empty binding list is omitted.

Successful consumer outputs also retain the *worker-observed* association in
their canonical persisted artifact provenance. The existing
`Claims.extra["mantle.plan_output_bindings"]` value is a JSON array sorted
by `name`, with `producer_drv_path`, `plan_output`, `plan_digest`, `root`,
`unit_output`, `root_drv_path`, and `output_path` for each bound reference.
Other provenance claims are retained; a pre-existing claim under this
reserved key fails the bound consumer rather than replacing observed facts.
The `BuildInput` provenance graph edges include the bound root artifacts,
and signed store PathInfo reference scanning can record the root path when
it appears in output bytes. The artifact provenance JSON has a canonical
BLAKE3 digest checked by `mantle attest verify artifact`, but is **not
cryptographically signed**; the PathInfo signature does **not** bind that
attestation digest. Neither the request identity nor the observation
establishes producer determinism, root content correctness, successful
consumer semantics, source trust, or release eligibility.
