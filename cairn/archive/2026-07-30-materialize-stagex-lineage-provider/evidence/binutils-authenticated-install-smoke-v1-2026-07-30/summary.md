# Authenticated binutils install and smoke evidence

## Question

Can Mantle stage and validate the authenticated binutils tool set without changing its logical prefix or using the predecessor TinyCC at runtime?

## Inspected evidence

- `full-install-smoke-test.log` records the complete create-new build, install, and smoke run.
- `retained-install-smoke-test.log` records the post-refactor identity and smoke rerun.
- `focused-unit-tests.log` records 19 positive and negative unit tests.
- `lint-format.log` records Clippy, formatting, and diff checks.
- `cairn-validation.log` records validation plus proposal, design, and tasks gates.
- `make-*-install.stdout.txt` and `make-*-install.stderr.txt` retain the eight authenticated install transcripts.
- `installed-tool-identities.log` binds the 11 required native tools.
- `triplet-links.log` records the 11 logical target-prefixed links.
- The positive and negative smoke stream files retain bounded tool diagnostics.
- `sed-invocation.count` records 4,892 total sed bridge calls.
- `BLAKE3SUMS` binds all retained evidence files except itself.

## Decision

Accept this as a bounded authenticated install and runtime-smoke checkpoint.

The complete integration test passed in 197.87 seconds. The post-refactor retained-root test passed in 0.47 seconds.

Authenticated Make installed 11 regular executable ELF tools under a DESTDIR-staged logical prefix. Each tool matched its checked BLAKE3 identity and did not contain the predecessor TinyCC path.

The final tree contains 11 target-prefixed links. Each link points to the matching absolute path under `/mantle/stagex/binutils-probe-output`.

The positive rail assembled and linked a static `_start` program. The program exited with status 42. Archive, symbol, disassembly, ELF-header, and object-copy checks passed.

The negative rail rejected malformed assembly, object, and archive inputs. It left no rejected object or linked binary output.

## Owner

Mantle owns this StageX install and runtime-smoke boundary.

## Next action

Bind generated configure, libtool, Make, compiler, install, and smoke children into the protected transition authorization graph. Then publish the staged tree with its provider receipt.

## Non-claims

This evidence does not prove compiler correctness, general binutils semantics, protected child authorization, provider admission, or release eligibility.
