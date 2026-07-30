# Binutils `ylwrap` generated-source probe v1

Date: 2026-07-30

## Result

Mantle closed the authenticated `ylwrap` sed performance blocker with a bounded native runner.

The final create-new probe completed in 140.16 seconds. Authenticated GNU Make, Bison 2.3, Flex 2.6.4, M4, Bash, and `ylwrap` generated all declared binutils and ld parser and scanner files.

## Bounded runner

`stagex-ylwrap-sed-runner` accepts only the exact authenticated four-program `ylwrap` shape. It requires:

- a `ylwrap<digits>` current directory;
- the exact parent source prefix;
- either the Bison three-file mapping or the Flex one-file mapping;
- matching deterministic header guards;
- one regular, non-symlink input no larger than 8 MiB;
- literal patterns without general regex authority.

The runner source BLAKE3 is `372a51aef1c1d06bef3ec1963bc61e89598b2912c8e709dce747637ffe49587d`.

The TinyCC musl-v2 and native-musl binary BLAKE3 is `d675a75869cba2e3c7cdb932ee75106a4a6094161e4d95fc196c08aa7b9723f8`.

A diagnostic comparison produced byte-identical 47,824-byte output from the runner and host GNU sed for the original blocked `arparse.c` rewrite. Host GNU sed was a comparator only. It was not execution authority for the accepted probe.

## Negative coverage

The runner and Rust shell reject:

- a broad regex pattern;
- a nested output name;
- a mismatched guard mapping;
- a wrong source prefix;
- a wrong current directory;
- a symlink input;
- an input larger than 8 MiB;
- an incomplete `ylwrap` audit suffix.

The alternative full GNU sed route did not produce an accepted artifact. TinyCC musl-v2 exited with status 139 while preprocessing authenticated GNU sed `lib/utils.c` against native musl.

## Generated outputs

Mantle validated producer markers in 14 Bison files and four Flex files.

The generated `binutils/arparse.c` BLAKE3 is `ec179b3977cc538050bb489823bfa6c43a311ec67335ffe4cb4111b322de7cc3`.

The generated `ld/ldlex.c` BLAKE3 is `d02762fcb534e9ba26b97defacc3bc3aa53dad984f42df99b296d6012f5d0272`.

`identities.log` records every generated output identity.

Expected generator diagnostics remain visible:

- Bison reports the authenticated grammar conflicts.
- Flex reports the existing unmatched `ldlex.l` rule.

Neither diagnostic caused a producer or marker failure.

## Audit closure

- Configure-preprocessor probes: 145
- Total sed bridge invocations: 3,488
- Bounded `ylwrap` runner invocations: 18
- Canonical sed audit BLAKE3: `c3fcb7af220adc52a4ed74820a8e6de796794784f2b298ca63fcfc976f317cea`

Each accepted `ylwrap` audit entry has empty standard input, one file input, and nonempty output.

## Validation

- Final generated-source test, pueue task 1039: passed, 1 test, 140.16 seconds.
- Focused tests, pueue task 1036: passed, 12 tests.
- Strict first-party Clippy, pueue task 1044: passed. The existing vendor warning remained.
- Strict C syntax and warning check, pueue task 1045: passed.
- Rust formatting and `git diff --check`, pueue task 1046: passed.
- Cached Cairn validation: passed.
- Cached Cairn proposal, design, and tasks gates: passed.

## Boundary

This probe authenticates source generation and bounded transformation observations only.

It does not add the runner, Bison, Flex, `ylwrap`, Make, or their children to the protected transition authorization graph. It does not prove the full binutils build, installed tools, binutils behavior, compiler correctness, or provider admission.
