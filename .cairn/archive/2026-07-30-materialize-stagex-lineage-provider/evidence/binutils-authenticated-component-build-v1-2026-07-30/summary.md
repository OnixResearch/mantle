# Authenticated binutils component-build evidence

## Question

Can authenticated binutils 2.30 Make build its eight declared components without ambient `ar` authority or false-green archive outputs?

## Inspected evidence

- `full-component-test.log` records the final create-new integration run.
- `focused-unit-tests.log` records positive and negative Rust tests.
- `ar-audit.tsv` records four successful archive operations.
- `ar-audit.canonical.tsv` records the path-bounded archive facts.
- `sed-invocation.count` records 4,772 sed bridge calls.
- `sed-audit.canonical.tsv` records each accepted sed producer call.
- `component-identities.log` records the eight required output identities.
- `producer-identities.log` records the archive and sed producer identities.
- `lint-format-shell.log` records Clippy, formatting, shell syntax, and diff checks.
- `cairn-validation.log` records validation plus proposal, design, and tasks gates.
- `make-*-build.stdout.txt` and `make-*-build.stderr.txt` retain each component transcript.
- `ar-*.stderr.txt` retains the archive producer smoke diagnostics.
- `BLAKE3SUMS` binds all retained evidence files except itself.

## Decision

Accept this as a bounded authenticated component-build checkpoint.

The final integration test passed in 193.38 seconds. Make produced four required archives:

1. `libiberty.a`: 66 members and 704,408 bytes;
2. `libz.a`: 15 members and 164,042 bytes;
3. `.libs/libbfd.a`: 59 members and 2,383,682 bytes;
4. `.libs/libopcodes.a`: five members and 2,160,260 bytes.

The build produced all eight required component identities. The test compares each output with its checked BLAKE3 value.

The archive producer used create-new staging and left no staged archive behind. Negative smokes rejected empty members, an existing output, unsupported mode, path traversal, and a symlink member.

## Owner

Mantle owns this StageX archive and component-build boundary.

## Next action

Add bounded runtime smokes and install the authenticated tool set into a new output tree. Then add generated-child execution authority to the protected transition graph.

## Non-claims

This evidence does not prove installation, general binutils behavior, compiler correctness, protected child authorization, provider admission, or release eligibility.
