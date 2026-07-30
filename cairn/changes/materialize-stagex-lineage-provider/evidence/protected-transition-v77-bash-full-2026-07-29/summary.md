# Protected transition v77: later full Bash

## Result

The protected transition reached the separate later Bash 2.05b stage.
The plan contained 91 ordered stages.
The execution audit contained 2,547 allowed events and no fallback event.

Pueue task `3947` ran the exact protected transition test from the current source.
The test passed in 1,312.06 seconds.
The complete command output is in `full-transition-test.log`.

The later Bash suffix contained exactly 180 events:

- two TinyCC executions built authenticated `mkbuiltins`;
- 40 protected `mkbuiltins` executions generated retained builtin sources;
- 131 TinyCC executions compiled and linked Bash;
- five Bash or GNU Make smoke parents ran;
- two declared Bash or GNU Make children ran.

## Bound identities

- lineage manifest: `ff13121154ebda99ec4b3887aff52f38584e1a91395a662f8ca3e704ae1bd80a`
- configured Bash source: `dc3fd01b5f34af5e2f3031c10cda243f43b3c9ce45a0fd2181693727fb629a2e`
- `mkbuiltins`: `725e682bc5f91728a92f72a81332dda87537ff0ec27b20134b9644b5f2cd43a7`
- generated builtin tree: `1797cb39c0507d6fbb40afbbd58299ffd9a0eb7d0e983bbe64742499a3fdc9c9`
- full Bash binary: `3f166d5bb28aee29982ec38ba07f8b1dca3c488390737a0e78d35950876e4d4e`
- version observation: `1cb581932f76e9f3870af0a75060141d692acd438ef5663ebb182798642fa20e`
- functional observation: `6687d0952325979e333ab55180b9aea8835ff490e46b7652ea5ec204c94b9c2a`
- malformed-input observation: `f1184a47cefae3e975232299de76d9a96ea5a9354c87f19f77413895583d8da3`
- external GNU Make child observation: `deb003aad7485e87e9a7ec33306fe4e89ca6774d39eeea26371b78d21cd8c292`
- GNU Make shell observation: `97ce45a8f015be46bbbd66c2abd778940a5aed257069c4e9dfd432b751c968d8`

## Checked boundary

The stage compiled 130 Bash sources and ran 133 Bash build commands.
It generated 42 retained builtin files through 40 authenticated generator commands.
It linked against the protected native-musl archive and startup objects.

The compatibility layer is bounded and single-threaded.
It supplies direct socket syscalls and the semaphore behavior observed by this build.
It does not establish a general threading runtime.

The smoke set checked version output, shell functions, malformed input, an exact GNU Make child, and GNU Make using this Bash.
The protected execution policy checked canonical TinyCC, generator, Bash, and GNU Make producer authority.

## Validation

- Task `3939`: 36 transition tests passed.
- Task `3940`: four focused positive and negative full-Bash tests passed.
- Task `3944`: strict first-party Clippy passed.
- Task `3936`: Rust formatting completed.
- Task `3948`: `git diff --check` passed.
- Task `4008`: Cairn validation and proposal, design, and tasks gates passed with the sibling generated policy.
- The checked Nickel and JSON lineage files have exact BLAKE3 parity.
- Bootstrap evaluation, source-pin checking, and the standalone 91-stage plan check passed before this final run.

## Non-claims

This evidence proves only the recorded non-interactive, single-thread Bash boundary and its declared observations.
It does not prove arbitrary shell semantics, general threading, binutils, compiler correctness, or StageX provider admission.
The `scaffold-only` provider receipt remains unchanged.
