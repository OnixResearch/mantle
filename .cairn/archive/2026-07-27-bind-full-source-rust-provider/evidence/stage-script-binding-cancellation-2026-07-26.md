# Stage-script binding cancellation

## Scope

This evidence records an operator cancellation before provider publication.
It does not prove provider completion.

## Reason

Detached v7 used explicit bounded job counts in every generated stage shell.
Its candidate receipt model still recorded a script path without the script's BLAKE3 digest.
It also did not place the explicit parallel-job limit in the provider receipt.

A path in temporary scratch is not durable construction identity.
A provider receipt must bind the exact stage plan and generated script that produced its artifacts.

## Cancellation

Pueue task 484 terminated the detached v7 process group before publication.
The same command verified that no provider output existed.
The incomplete scratch had no validated final provider receipt.

## Repair

Each stage build record must add these explicit fields:

- stage-plan BLAKE3
- generated-script BLAKE3
- bounded parallel-job count

Each provider receipt must include those values in the stage execution step.
Positive tests must require the bindings.
Negative tests must reject a missing or changed binding.

## Next action

A fresh construction must create receipts with the repaired stage identity fields.
The v7 scratch cannot be resumed as completion evidence.
