# Fragment Collector

## Overview

Merges per-module output fragments into a single configuration tree per
machine. Pure function: no I/O, no store, no Nickel evaluation.

## Requirements

### FRAG-1: Per-machine grouping

The collector MUST group evaluated module output fragments by target
machine, as determined by the inventory's instance-to-machine mapping.

### FRAG-2: Deep merge

Fragments targeting the same machine MUST be deep-merged. For conflicting
scalar values at the same path, the collector MUST apply priority ordering
(module priority, then alphabetical module name as tiebreak). Conflicts
at equal priority MUST be reported as errors naming both modules and the
conflicting path.

### FRAG-3: Output namespace preservation

The merged tree MUST preserve output namespaces from module impls
(e.g., `output.nixos`, `output.files`, `output.providers`). The
collector does not interpret namespace contents — that is the
assembler's job.

### FRAG-4: Machine filtering

The collector MUST accept an optional machine filter. When provided,
only fragments for matching machines are collected. Unmatched modules
are skipped without error.

### FRAG-5: Merge audit trail

The merged tree MUST carry provenance metadata recording which module
contributed each top-level fragment. This enables debugging when merged
configs produce unexpected results.

### FRAG-6: Fixed limits

The collector MUST enforce a maximum merged-tree depth (configurable,
default 128). Trees exceeding the depth limit MUST fail with a
diagnostic naming the deepest path.
