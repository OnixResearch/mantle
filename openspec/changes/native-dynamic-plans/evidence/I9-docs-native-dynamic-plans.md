# I9 native dynamic-plan documentation

Task-ID: I9
Covers: defaults.dynamic.derivations, build.engine.dynamic.plans.provenance
Date: 2026-05-31T05:58:10Z

## Coverage summary

- `README.md` now documents `mantle-plan-v1` as Mantle's core dynamic-build API.
- `README.md` documents declared `dynamic_plan_outputs`, validation, root-only scheduling, JSON report rows, and `.drv` discovery as compatibility/debug behavior.
- `adr/0011-native-dynamic-plans.md` is marked Accepted and records report labeling plus `.drv` compatibility separation.

## openspec validate native-dynamic-plans

Command:

```sh
openspec validate native-dynamic-plans
```

Output:

```text
Change 'native-dynamic-plans' is valid

exit status: 0
```

## git diff --check

Command:

```sh
git diff --check
```

Output:

```text

exit status: 0
```
