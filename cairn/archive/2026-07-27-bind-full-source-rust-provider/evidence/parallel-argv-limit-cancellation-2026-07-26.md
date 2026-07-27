# Parallel argv-limit cancellation

## Scope

This evidence records an operator cancellation before provider publication.
It does not prove provider completion.

## Reason

Detached v9 recorded an explicit parallel-job limit and passed that limit to mrustc's nested LLVM build.
The top-level first-stage Make invocations still omitted `-j "$PARLEVEL"`.
They therefore used Make's serial default rather than the receipt-bound limit.

The materialization task requires explicit argv and limits.
Recording a limit without placing it in each applicable command argv is incomplete.

## Cancellation

Pueue task 465 terminated the detached v9 process group before publication.
The same command verified that no provider output existed.
The incomplete scratch had no validated final provider receipt.

## Repair

Every first-stage Make invocation must pass the same bounded `PARLEVEL` value through explicit argv.
Generated-script tests must require the bounded argument and reject an ambient or implicit job count.
The receipt continues to record the same parallel-job value.

## Next action

A fresh construction must use the repaired bounded Make argv.
The v9 scratch cannot be resumed as completion evidence.
