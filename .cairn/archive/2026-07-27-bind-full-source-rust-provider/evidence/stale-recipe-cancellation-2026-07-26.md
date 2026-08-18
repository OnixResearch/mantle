# Stale Rust recipe cancellation

## Scope

This evidence records an operator cancellation before provider publication.
It does not prove provider completion.

## Reason

Detached v5 bound the old `bootstrap/rust-source.ncl` identity.
That file still named the provider `rust-source-provider-blocked`.
It also stated that source-built Rust materialization was not implemented.

The generated first-stage plan preserved the same stale blocker.
Publishing a provider with that recipe identity would make the completion claim inconsistent.

## Cancellation

Pueue task 469 terminated the detached v5 process group before publication.
The planned output directory did not exist after termination.
The incomplete scratch contained no validated final provider receipt.

## Repair

`bootstrap/rust-source.ncl` is now a typed materialization recipe.
It selects `mantle bootstrap rust-source-provider` and the musl-host route plan.
It binds authenticated offline sources and create-new publication.
It explicitly rejects Nix, rustup, prebuilt Rust, ambient discovery, and host-assisted substitution.

The intermediate-stage reason now states that completion is pending until all planned stages pass.
It no longer states that the implemented materializer does not exist.

## Next action

A fresh construction must bind the repaired recipe bytes.
No v5 artifact or scratch state can be resumed as completion evidence.
