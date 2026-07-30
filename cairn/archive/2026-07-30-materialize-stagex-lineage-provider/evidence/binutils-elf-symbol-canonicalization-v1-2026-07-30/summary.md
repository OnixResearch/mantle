# StageX binutils ELF symbol canonicalization evidence

Date: 2026-07-30

## Scope

This checkpoint removes process-derived TinyCC local symbol names from StageX binutils object files. It does not grant protected execution authority or admit a provider.

The bounded canonicalizer changes only local `SHT_SYMTAB` names that exactly match `L.[0-9]+`. It preserves file length, section layout, symbol facts, code, and non-matching names.

## Authenticated identities

- canonicalizer source: `eed4dcf5e348d6b77317243a394ad2effb76ac112186eaf71e7115a1c5dc4a20`
- stripped canonicalizer binary: `1a7a10d6ce97f3cea18ffc4f956fe28bdb10d94568146d89670016a80d1de06a`
- repeated canonical smoke object: `188411279a7f243e7caff51f368d1d5bfd748fa4b3e8a9b00a6c9c5b5a2397d4`
- `libiberty.a`: `d19e0d15373a237fe94b0d915f3edba43182343739da15f837c1a9a1ea426236`
- `libz.a`: `b922a8528a2491568debfc87c017514986a44516f43c3865c10a72884bb4eb53`
- `libbfd.a`: `f45500a900c2294cd23bcd0c7fd013e31a7b4028bce48f765108d7f309169e53`
- `libopcodes.a`: `1f9d0deabd406f82dcabaaddf6faff6b5fdb7475b3e59d000b27b70251200fac`

The eight component outputs, DESTDIR install, target-prefixed links, and positive and negative runtime smokes completed under the strict checked identities. See `strict-build-test.log`.

The canonicalizer rejected malformed non-ELF input during probe setup. The repeated compiler fixture produced byte-identical canonical object files. See `identities-command.log` for the retained strict-run identities.

## Claim boundary

This evidence proves the checked bounded build and smoke checkpoint only. It does not prove compiler correctness, arbitrary ELF equivalence, complete protected child authorization, provider publication, or provider admission.
