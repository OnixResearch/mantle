# Specification: Dynamic-plan source slices

## ADDED Requirements

### Requirement: Source slices use a versioned plan schema

r[mantle.dynamic_plan_source_slices.versioned_schema] Mantle MUST accept source slices only in plans whose schema is `mantle-plan-v2`. A slice MUST declare a source id, a producer output name, a relative subpath, a store name, and an expected NAR BLAKE3. Mantle MUST keep `mantle-plan-v1` decoding, canonical bytes, and plan digests unchanged.

#### Scenario: Version 2 plan declares a slice

- GIVEN a producer declares a dynamic-plan output containing a `mantle-plan-v2` plan with one slice of its `sources` output
- WHEN the worker decodes and validates the plan
- THEN the plan MUST be admitted with the slice as a typed source and a canonical BLAKE3 plan digest that covers the slice's expected digest

#### Scenario: Version 1 plan uses slice fields

- GIVEN a `mantle-plan-v1` plan contains a slice-shaped source entry
- WHEN the worker decodes it
- THEN decoding MUST fail with a typed invalid-plan rejection
- AND no unit MUST be registered from that plan

#### Scenario: Accepted v1 fixture is unchanged

- GIVEN an accepted `mantle-plan-v1` golden fixture
- WHEN it is decoded and canonicalized after this change
- THEN its canonical bytes and plan digest MUST equal the recorded values

### Requirement: Slices are admitted before unit registration

r[mantle.dynamic_plan_source_slices.content_admission] The worker MUST admit every slice of an accepted plan before registering any of that plan's units. Admission MUST locate the subtree inside the named producer output, MUST verify that its observed NAR BLAKE3 equals the declared value, and MUST publish it with signed PathInfo through the existing verified-source admission capability. Mantle MUST NOT add a second source admission path or a store interface inside the sandbox.

#### Scenario: Slice becomes a source

- GIVEN an accepted v2 plan whose slice names an existing directory in a declared producer output with a matching digest
- WHEN the worker handles the producer's completion
- THEN the slice MUST have signed PathInfo before any unit of the plan is registered
- AND a unit that references the slice through `{{mantle-source:ID}}` MUST see that store path in its sandbox

#### Scenario: Digest mismatch

- GIVEN a slice whose observed NAR BLAKE3 differs from the declared value
- WHEN the worker plans admission
- THEN it MUST reject the plan with `slice-digest-mismatch`
- AND it MUST NOT publish any slice or register any unit from that plan

### Requirement: Slice identity depends only on content and name

r[mantle.dynamic_plan_source_slices.content_identity] A slice's logical store path MUST be derived from its NAR content and declared store name under the configured logical prefix. The producing derivation, the producer output name, and the subpath MUST NOT affect the path.

#### Scenario: Unchanged slice across producer reruns

- GIVEN a producer rerun whose output changed only outside one slice
- WHEN both plans are admitted
- THEN that slice MUST resolve to the same store path in both runs
- AND units whose derivations depend only on unchanged slices and unchanged inputs MUST keep their derivation paths

#### Scenario: Identical content at two subpaths

- GIVEN two slices with the same store name and identical content at different subpaths
- WHEN the worker admits them
- THEN both MUST resolve to one store path that is published once

### Requirement: Slice admission is bounded and fails closed

r[mantle.dynamic_plan_source_slices.bounded_rejection] Mantle MUST enforce named limits on slice count, subpath bytes, subpath depth, and admitted bytes per plan. It MUST reject undeclared producer outputs, absolute or escaping subpaths, empty path components, absent subpaths, symlinks on the walk to a slice root, and one source id declared with conflicting content. It MUST evaluate every slice before the first publication and MUST publish either all slices of a plan or none.

#### Scenario: Escaping subpath

- GIVEN a slice subpath containing `..`
- WHEN the pure core validates the plan
- THEN it MUST reject the plan with `slice-subpath-invalid` before any store effect

#### Scenario: Symlink on the walk path

- GIVEN a slice subpath whose intermediate component is a symlink in the producer output
- WHEN the worker plans admission
- THEN it MUST reject the plan with `slice-symlink-traversal` without following the link

#### Scenario: Limit exceeded

- GIVEN a plan whose slices exceed the named admitted-byte limit
- WHEN the worker plans admission
- THEN it MUST reject the plan with `slice-limit` naming the limit
- AND registry, goal, and scheduler state MUST remain unchanged

### Requirement: Slice admission is reported

r[mantle.dynamic_plan_source_slices.provenance] Native dynamic-plan report rows MUST record, for every declared slice, the source id, producer output, subpath, declared and observed NAR BLAKE3, admitted store path when present, and admitted or rejected disposition, in canonical order.

#### Scenario: Accepted plan with slices

- GIVEN a v2 plan with two admitted slices
- WHEN the build report is written
- THEN the plan's native dynamic-plan row MUST list both slices with their store paths in source-id order

#### Scenario: Rejected slice

- GIVEN a v2 plan rejected for one absent slice
- WHEN the build report is written
- THEN the row MUST record the rejection kind and subpath
- AND it MUST NOT list an admitted store path for any slice of that plan
