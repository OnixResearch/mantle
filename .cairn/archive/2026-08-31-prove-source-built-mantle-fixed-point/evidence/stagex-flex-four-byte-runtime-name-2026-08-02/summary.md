# StageX Flex four-byte runtime-name observation

## Result

Source-built observation v6 failed closed during protected GNU Flex materialization. The exact 49-byte compatibility-runtime section had a four-byte process-derived name. The existing canonicalizer accepted only the retained three-byte variants.

The fresh artifact and retained canonical Flex executable have the same file size, section-header offset, runtime bytes, and non-name content. Their first difference is in `.shstrtab`. The fresh table is one byte longer and shifts all later section names by one byte.

## Bounded repair

Mantle now accepts only three-byte or four-byte names for the exact authenticated runtime section. It rebuilds `.shstrtab` in place with `stx`, shifts later `sh_name` offsets, reduces the table size, and zero-fills the vacated byte. It does not move section headers or change file length.

The fresh four-byte artifact canonicalizes to the retained BLAKE3 identity:

`502324a00e1b35d6fc18a6cf6c3a257d3578b7d5fd8bffc27ce41b9da3c1ebbf`

Positive tests prove three-byte and four-byte inputs converge and remain idempotent. Negative tests reject zero-byte, one-byte, two-byte, and five-byte names, substituted runtime bytes, non-boundary offsets, and out-of-range offsets.

## Non-claims

This repair does not prove TinyCC correctness, arbitrary ELF equivalence, Flex correctness, transition completion, native-provider admission, a Mantle fixed point, or release eligibility.
