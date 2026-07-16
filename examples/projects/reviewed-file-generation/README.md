# Reviewed file generation

This project declares two generated files in Nickel. Planning is non-mutating; applying requires the exact still-current reviewed plan and records managed identities in `.mantle/filegen-state.json`.

Use a scratch copy so the checked-in example remains unchanged:

```bash
work=$(mktemp -d)
cp -R . "$work/project"
cd "$work/project"
mkdir -p target

mantle --json filegen plan --plan-out target/filegen-plan.json
test ! -e generated/app-config.json
mantle --json filegen apply --plan target/filegen-plan.json
cat generated/app-config.json
cat generated/README.txt
cat .mantle/filegen-state.json
```

The JSON contract requires `schema_version`, `name`, and `mode`. A second plan reports the managed files unchanged.

Negative paths:

- Modify a declaration after writing the plan, then apply the old plan. Mantle rejects the stale reviewed plan before writing changed content.
- Place unrelated content at `generated/app-config.json` before the first plan. Mantle reports an unmanaged conflict.
- Run `mantle filegen plan --manifest fixtures/missing-required-field.ncl`. The JSON contract rejects content without `mode`.
- Run `mantle filegen plan --manifest fixtures/target-escape.ncl`. The `../escaped-config.json` target is rejected before any out-of-root write.

The plan and state establish bounded generated-content identity and ownership. They do not prove the generated configuration is deployable, semantically correct, or accepted by a downstream service.
