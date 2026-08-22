# V37 GNU binutils middle-sed lower closure

## Question

Does V31 show an accepted GNU binutils execution shape outside the middle-sed event floor?

## Inspected evidence

Remote pueue task `277` failed closed during StageX audit validation. It did not start native-provider or Rust construction.

The preserved attempt reports:

```text
bounds [73980, 74066], sed bounds [74001, 74066], observed 73994
```

The complete V31 inventory records 4,892 sed invocations, eight components, four archives, and 11 installed tools. Protected execution stayed enabled, with no fallback events.

All expected component archive and tool identities matched. The complete audit BLAKE3 is `a9853abd851b70d41c08de68353fde075864c8e1b641cc239288e85b66ee847e`.

The inventory BLAKE3 is `5540d65291c56b335a83e3ad3d767603a7851322adb56621b40e92b5c4f221e6`.

Compared with V29, V31 has the same 68 authorization identities. Only five accepted identity counts changed:

- full Bash: minus two;
- coreutils `chmod`: minus one;
- coreutils `cp`: minus one;
- coreutils `mkdir`: minus three.

The total difference is minus seven events. No identity, executable digest, policy decision, source, or fallback rule changed.

## Decision

Lower the 4,892-sed event floor from `74_001` to the observed `73_994`. Keep every upper bound and other sed-specific bound unchanged.

The first complete V31 replay then reached the existing `mkdir` check. It confirmed `5_366` accepted executions, three fewer than V29.

Lower only the `mkdir` floor from `5_369` to `5_366`. Keep its `5_425` upper bound unchanged.

Accept the repair only if one current test binary replays the complete V28, V29, and V31 audits through ordinary production validators.

## Validation

The final current test binary replayed all three complete audits:

```text
preserved-binutils-audit: events=73980 sed_invocations=4891
preserved-binutils-audit: events=74001 sed_invocations=4892
preserved-binutils-audit: events=73994 sed_invocations=4892
```

All three passed the production validators for 68 identities, exact and bounded counts, producer order, and fallback rejection.

Positive and negative total-event and `mkdir` boundary tests passed. Focused strict Clippy, changed-file formatting, and `git diff --check` also passed.

## Owner

The StageX transition validator owns this closed event-count policy.

## Next action

Build and transfer a new release orchestrator. Refresh the paired source/vendor profile, then run another fresh promoted proof.

## Non-claims

This repair does not admit a new executable, denied event, fallback path, or broader StageX behavior. A fresh promoted proof remains required.
