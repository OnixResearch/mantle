## ADDED Requirements

### Requirement: FRAG-1 Per-machine grouping

The collector MUST group evaluated module output fragments by target machine.
ID: systemconfig.fragment.collector.frag1

Grouping MUST use the inventory's instance-to-machine mapping so that each
machine is merged independently of every other machine.

#### Scenario: Collector groups fragments by machine
ID: systemconfig.fragment.collector.frag1.scenario

- GIVEN four evaluated fragments for two machines
- WHEN the collector groups fragments
- THEN each fragment appears under exactly one target machine
- AND no fragment from one machine is merged into another machine's group

### Requirement: FRAG-2 Deep merge

Fragments targeting the same machine MUST be deep-merged.
ID: systemconfig.fragment.collector.frag2

When scalar values conflict at the same path, the collector MUST prefer the
fragment with higher precedence according to module priority and then module
name. Lower numeric priority values mean higher precedence, matching the
stable-order rule used by EVAL-1. Equal-precedence conflicts MUST be reported
as fragment errors naming the conflicting path and both contributing modules.

#### Scenario: Equal-precedence scalar conflict becomes an error
ID: systemconfig.fragment.collector.frag2.scenario

- GIVEN two fragments for the same machine that set the same scalar path
- AND both fragments have equal merge precedence
- WHEN the collector merges them
- THEN the merge fails with a fragment diagnostic naming the path and both
  modules

### Requirement: FRAG-3 Output namespace preservation

The merged tree MUST preserve output namespaces such as `output.nixos`,
`output.files`, and `output.providers` without interpreting their internal
meaning.
ID: systemconfig.fragment.collector.frag3

The collector is responsible only for structural merge behavior, not namespace-
specific semantics.

#### Scenario: Collector preserves multiple output namespaces
ID: systemconfig.fragment.collector.frag3.scenario

- GIVEN one fragment containing `output.nixos`
- AND another fragment containing `output.files`
- WHEN the collector merges them
- THEN both namespaces are present in the merged output
- AND neither namespace is rewritten by collector-specific logic

### Requirement: FRAG-4 Machine filtering

The collector MUST accept an optional machine filter.
ID: systemconfig.fragment.collector.frag4

When the filter is present, only matching machines are collected. Fragments for
unselected machines MUST be skipped without producing errors.

#### Scenario: Machine filter skips non-selected machines
ID: systemconfig.fragment.collector.frag4.scenario

- GIVEN fragments for `server1` and `server2`
- WHEN the collector runs with a filter selecting only `server1`
- THEN only `server1` fragments appear in the merged output
- AND `server2` fragments are skipped without diagnostics

### Requirement: FRAG-5 Merge audit trail

The merged tree MUST preserve provenance metadata recording which module
contributed each top-level fragment.
ID: systemconfig.fragment.collector.frag5

The provenance data MUST be available to later diagnostics and debugging tools
without re-running the evaluation stage.

#### Scenario: Collector records top-level provenance
ID: systemconfig.fragment.collector.frag5.scenario

- GIVEN a merged machine config built from three modules
- WHEN the collector produces the final `MergedConfig`
- THEN the result includes provenance entries for the top-level keys
- AND each entry names the contributing module

### Requirement: FRAG-6 Fixed limits

The collector MUST enforce a configurable maximum merged-tree depth whose
default value is 128.
ID: systemconfig.fragment.collector.frag6

Trees exceeding the depth limit MUST fail with a fragment diagnostic naming the
deepest path reached.

#### Scenario: Excessive tree depth is rejected
ID: systemconfig.fragment.collector.frag6.scenario

- GIVEN a merged tree deeper than the configured depth limit
- WHEN the collector validates the merged output
- THEN the collector fails with a fragment diagnostic
- AND the diagnostic names the deepest offending path
