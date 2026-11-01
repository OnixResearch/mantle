# Cairn validation and readiness (2026-11-01)

Scope: record the pinned Cairn validation of this change and its current
lifecycle readiness.

## Tooling workaround

`nix develop --offline -c cairn validate --root .` produces no output and is
killed after 15 minutes, and the flake wrapper
`/nix/store/180irnms1yr45xpfz4g0six3kdr8grnq-cairn/bin/cairn` hangs because it
`exec`s `nix run`. Running the wrapped binary directly works:

```bash
CAIRN=/nix/store/1z6bhr2cl43z4kpsvjkxy9b47nqr9pkg-cairn-0.1.0/bin/.cairn-wrapped
POLICY=/nix/store/iwgvqkymjzgiwy34vqh425w1p2z5b2n1-source/cairn-policy/generated/cairn-policy.json
$CAIRN validate --root . --policy "$POLICY"
$CAIRN readiness schedule-native-goals-across-workers --root . --policy "$POLICY" --format summary
$CAIRN workflow status schedule-native-goals-across-workers --root . --policy "$POLICY"
```

## Results

- `validate`: exit 0, `issues: []`, `change_issues: []`, 34 changes, policy
  `cairn-default`, receipt hash
  `77d772bbaf21912ca515d32b8661bc1e8ba9299fe8ef295034d2f697cbc17cb9`.
- `workflow status`: exit 0, `issues: []`, empty blockers.
- `readiness` (summary): artifact, proposal, design, and tasks are `ready`;
  completion is `unavailable`; sync and archive are `blocked`; the remaining
  dependency waits are `resolve-remote-entry-points-through-one-binding` and
  `separate-remote-build-hexagon`.

## Consequence

The change tree validates and its authored artifacts are ready, but it cannot
be synced or archived while those dependent changes are unresolved. `tasks.md`
stays unchecked: the change's own rule requires validation and acceptance
before archiving, and the evidence index maps the implemented slices without
substituting for that lifecycle step.
