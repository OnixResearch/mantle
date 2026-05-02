# Design: i386 TinyCC 0.9.26 object-emission repair

## Context

The previous diagnostic showed:

```text
version rc=0
assemble_object rc=139
compile_c_object rc=139
```

Host-side GDB showed a `strlen`-shaped crash with `rdi=0x800`, suggesting a target-sized value is flowing into a host string path.

## Decisions

### 1. Repair the sibling proof first

Keep production bootstrap derivations untouched. Change only the i386 proof/diagnostic derivation until object emission and the i386 `exit42` run work.

### 2. Prefer source-level target/host fixes over Make patches

The failure occurs before Make and before static linking from an object. The target is TinyCC object emission.

### 3. Evidence must include staged diagnostic results

A build exit alone is insufficient. Evidence must show version, object emission, link, and run results.
