# Complete make 3.82 runtime validation

## Why

Parent change `live-part-make-3-82` completed upstream ordering, derivation audit, and source-pin validation for `bootstrap/make-tcc.ncl`. Runtime validation remains blocked by `repair-make-tcc-amd64-varargs`: diagnostic transcripts show make can report its version, but simple Makefile execution still segfaults on amd64.

## What Changes

- Resume `bootstrap/make-tcc.ncl` build validation after the amd64 varargs repair is complete.
- Record output path or failure class, provider selection, fallback status, and placeholder rejection result.
- Smoke-test GNU Make 3.82 with a simple Makefile, not just `--version`.
- Scan the derivation and runtime transcript for undeclared host-tool/path/environment leakage.

## Scope

In scope: runtime validation evidence and narrowly scoped fixes required to prove make 3.82 execution.

Out of scope: broader TinyCC codegen repairs beyond what `repair-make-tcc-amd64-varargs` owns.
