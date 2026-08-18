# Final validation — source-built provider native static-PIE CRT

Date: 2026-06-26

## Whitespace

Command:

```sh
git diff --check
```

Evidence: pueue task 128 completed successfully.

## Cairn validation

Command:

```sh
/home/brittonr/git/cairn/target/debug/cairn validate --root .
```

Evidence: pueue task 132.

```json
{
  "change_issues": [],
  "changes": 1,
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "spec_issues": [],
  "specs_validated": 7,
  "valid": true
}
```

## Cairn gates

Commands:

```sh
/home/brittonr/git/cairn/target/debug/cairn gate proposal source-built-provider-native-static-pie-crt --root .
/home/brittonr/git/cairn/target/debug/cairn gate design source-built-provider-native-static-pie-crt --root .
/home/brittonr/git/cairn/target/debug/cairn gate tasks source-built-provider-native-static-pie-crt --root .
```

Evidence: pueue tasks 133, 134, and final post-checklist tasks rerun 138.

```text
proposal: valid=true verdict=PASS issues=[] receipt_hash=adef863efd28d4bfabc5bcbf184d14bb7074dd925a77a681fb581341ec593f89
design: valid=true verdict=PASS issues=[] receipt_hash=ae2ac888a77ac2f413e8602ddec9a429ce5baa40bab40592d391f9f3f10cba37
tasks: valid=true verdict=PASS issues=[] receipt_hash=1299cb53d8c68e74bf53870e4935ae56f5ce60ba0378e54555f94e6b6990e68d
```

## Post-archive validation

The archive command moved the change to `cairn/archive/2026-06-26-source-built-provider-native-static-pie-crt/`. The accepted spec requirement was manually synced afterward because archive execution did not insert the delta into `cairn/specs/rust-package-planning/spec.md`.

Command:

```sh
/home/brittonr/git/cairn/target/debug/cairn validate --root .
```

Evidence: pueue task 156 (rerun after appending this post-archive evidence).

```json
{
  "change_issues": [],
  "changes": 0,
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "spec_issues": [],
  "specs_validated": 6,
  "valid": true
}
```

Post-archive whitespace validation also passed in pueue task 156 after the final evidence edit:

```sh
git diff --check
```
