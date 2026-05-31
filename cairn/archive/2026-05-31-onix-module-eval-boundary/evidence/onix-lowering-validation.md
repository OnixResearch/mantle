# Onix-owned module lowering validation

Task-ID: onix-lowering-reference
Covers: r[build_tool_boundary.onix_owns_module_lowering]

## Referenced Onix change

Repository: `/home/brittonr/git/onix-modules`
Change: `cairn/changes/onix-mantle-module-lowering`
Head commit: `6e5681cdd42eca360e23c500a38a12624f79e69c`

Relevant Onix commits inspected:

```text
6e5681cdd42eca360e23c500a38a12624f79e69c prove Mantle lowering before assembler work
b90dc6a65ef6de3db1aa2327966f87ba3b8076a5 keep Nix as reference backend until Mantle parity
0a8210b104987a818572368f94a2e694e3c9fe98 separate Onix and Mantle diagnostic layers
83a4843c2ae3f6f53d0290ece70b5256ff233fb2 reject raw Onix handoff data before Mantle build
51f747fda10c9f6bfe313005620ce4cf6c0e6ae6 order Mantle lowering by Onix service dependencies
9d3c1232a6e6f4f6712d7acba983122653b627d8 preserve Onix settings semantics before Mantle lowering
af180bb421087aa30e45c087dc057ebd9c36638b invoke real Onix service impls before Mantle handoff
```

## What the Onix evidence covers

- Real Nickel service-module `impl` invocation occurs in Onix before Mantle handoff.
- Role settings, tag settings, machine binding settings, defaults, contracts, and enum values are validated/preserved by Onix.
- Upstream exports and provider outputs flow through Onix graph lowering; failed producers block dependents with Onix-layer diagnostics.
- Raw Onix module concepts and synthetic `JsonEvalBoundary` output are rejected before the Mantle build-tool boundary.
- Nix remains the default/reference backend while Mantle remains opt-in/fail-closed until parity and assembler evidence exists.
- Focused positive and negative tests cover real output, normalized Nix-reference fragments, invalid settings, unknown dependencies, failed providers, raw handoff, and synthetic output.

## Validation transcript

Command:

```sh
cd /home/brittonr/git/onix-modules
git rev-parse HEAD
git log --oneline -7 --decorate --no-abbrev-commit
/nix/store/bs92xsdsf6a8bfkrlfc6ryisqh0vx8j8-cairn-0.1.0/bin/cairn validate --root .
/nix/store/bs92xsdsf6a8bfkrlfc6ryisqh0vx8j8-cairn-0.1.0/bin/cairn gate tasks onix-mantle-module-lowering --root .
```

Output:

```text
6e5681cdd42eca360e23c500a38a12624f79e69c
6e5681cdd42eca360e23c500a38a12624f79e69c (HEAD -> main) prove Mantle lowering before assembler work
b90dc6a65ef6de3db1aa2327966f87ba3b8076a5 keep Nix as reference backend until Mantle parity
0a8210b104987a818572368f94a2e694e3c9fe98 separate Onix and Mantle diagnostic layers
83a4843c2ae3f6f53d0290ece70b5256ff233fb2 reject raw Onix handoff data before Mantle build
51f747fda10c9f6bfe313005620ce4cf6c0e6ae6 order Mantle lowering by Onix service dependencies
9d3c1232a6e6f4f6712d7acba983122653b627d8 preserve Onix settings semantics before Mantle lowering
af180bb421087aa30e45c087dc057ebd9c36638b invoke real Onix service impls before Mantle handoff
{
  "change_issues": [],
  "changes": 8,
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "spec_issues": [],
  "specs_validated": 41,
  "valid": true
}
{
  "change": "onix-mantle-module-lowering",
  "input_hash": "9c49434fb3dbe517f25124c3970b098dc5f5a1644a4308c6e1bfe1c2d860991b",
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "policy_hash": "63001c432f74f09c117774afca2952a7d0019b36aae7b65107afc26bc56b44c1",
  "receipt_hash": "833ddfdc3a44b1485c8eba1c983ed34987bc5073651b293e0b71ee717aa7b62b",
  "stage": "tasks",
  "valid": true,
  "verdict": "PASS"
}
```

## Boundary conclusion

Mantle can now reference Onix-owned lowering evidence without claiming Mantle owns module semantics. The remaining Mantle-side boundary stays build-tool-shaped: Mantle receives derivations, build plans, source inputs, store/build requests, build reports, diagnostics, or opaque evaluated frontend data.
