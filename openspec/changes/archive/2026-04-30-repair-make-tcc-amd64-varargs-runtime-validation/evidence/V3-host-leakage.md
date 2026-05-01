Task-ID: V3
Covers: bootstrap.part.make.3.82.amd64.runtime-validation
Status: deferred

The host-leakage scan for the Make output requires either a completed Make output or a completed build transcript that reaches installation. V1 instead stopped at a concrete TinyCC static-link segfault before any `make-3.82-tcc` output was produced.

The available V1 transcripts are preserved and show the failure occurs inside the Crunch sandbox with declared bootstrap inputs. Full output scanning is deferred until the TinyCC link repair produces a Make output.

Deferred to OpenSpec change:

    repair-tinycc-0-9-27-amd64-link
