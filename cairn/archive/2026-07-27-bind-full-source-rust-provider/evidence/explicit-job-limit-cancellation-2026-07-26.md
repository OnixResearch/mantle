# Explicit Rust bootstrap job-limit cancellation

## Scope

This evidence records an operator cancellation before provider publication.
It does not prove provider completion.

## Reason

Detached v6 bound the repaired materialization recipe.
Its generated first-stage script still accepted mrustc's implicit `PARLEVEL = 1` default.
Later x.py stages also lacked an explicit job count.

The change requires explicit limits for every stage shell.
An implicit build-system default is not sufficient evidence for that requirement.

## Cancellation

Pueue task 478 terminated the detached v6 process group before publication.
No provider output existed after termination.
The incomplete scratch had no validated final provider receipt.

## Repair

The materializer now sets a bounded `RUST_BOOTSTRAP_JOB_COUNT`.
It exports that value as first-stage `PARLEVEL`.
It writes the same value as x.py's `[build].jobs` setting.
Compile-time assertions keep the value positive and below the declared maximum.
Positive tests require both settings.
Negative tests reject ambient variable substitution in each generated script.

## Next action

A fresh construction must bind the repaired generated scripts and stage receipts.
The v6 scratch cannot be resumed as completion evidence.
