## Context

`cmd_release_verify` verifies manifest artifacts and computes several evidence results, then immediately calls the JSON renderer or prints `release evidence verified`. Only afterward does it enforce required reproducibility, deterministic-release, provider fixed-point, stack-provenance, external-role, and StageX checks. The JSON object has no top-level final validity field, so payload presence can also be mistaken for acceptance.

The command needs one final policy decision that is independent of presentation.

## Decisions

### Aggregate a complete pure decision

A pure core receives normalized verification facts and invocation policy and returns a `ReleaseVerificationDecision` containing final validity, a closed disposition, ordered contributor results, and ordered diagnostics. Contributors include manifest integrity, reproducibility, deterministic proof, provider fixed-point proof, stack provenance, required external roles, StageX policy, and any function-address or Cairn handoff policies enabled by their owning changes.

The contributor enum and aggregation match are exhaustive. Adding a required policy contributor without mapping it into the final decision fails compilation or focused contract tests rather than silently bypassing final validity.

### Distinguish evidence facts from policy requirements

An absent optional contribution can remain absent without invalidating the command. The same fact becomes a blocker when selected policy requires it. Manifest integrity success is one contributor and cannot itself set final command success.

The core has no filesystem, output, serialization, or process-exit behavior. The shell loads artifacts and invokes specialized evaluators, then passes their normalized results to the aggregator.

### Render only the final decision

Human and JSON renderers consume the completed decision and return a complete output buffer. The shell writes the terminal payload only after aggregation. Human output uses `release evidence verified` only for a valid decision; a rejected decision uses explicit rejection language and cannot contain that success marker.

Operational failures that prevent a decision, such as unreadable or malformed input, remain errors. Policy rejection after facts are available is a first-class decision with a nonzero exit.

### Make JSON validity explicit

The versioned JSON result includes top-level `valid`, `disposition`, ordered `checks`, and `diagnostics` fields. Accepted and policy-rejected results use the same documented shape. JSON mode reserves stdout for exactly one JSON value; human diagnostics and duplicate error text do not follow it on stdout.

Callers must use both process status and `valid`; the presence of a parsed payload is not success.

### Test every late gate

Integration tests independently reject missing reproducibility, ineligible deterministic proof, invalid or absent required provider proof, invalid required stack provenance, missing required external role, unsatisfied StageX policy, and—when active wiring lands—required function-address or Cairn handoff evidence. Each test asserts a nonzero exit, `valid: false` in JSON mode, deterministic diagnostics, and absence of the human success marker. Positive tests assert that success appears exactly once after all selected checks pass.

## Risks / Trade-offs

- Adding explicit JSON validity is a schema-visible change; versioned output and documentation reduce ambiguity for consumers.
- Evaluating all safe contributors can report more than one blocker, but operational errors may still stop fact collection when later checks cannot be evaluated soundly.
- This change orders and reports existing policy decisions; it does not strengthen the semantics of individual evidence validators.
