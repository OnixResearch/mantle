# StageX binutils archive-width observation

## Result

Source-built observation v8 passed the Flex identity repair and failed closed at `libiberty.a`. The protected C helper had replaced process-derived local-symbol values but preserved each input decimal width.

The retained archive used local names such as `L.0000012`. The fresh archive used `L.000000012`. They had the same 66-member order. Exactly 33 members differed. ELF inspection showed matching code, symbol indices, bindings, values, and sizes. Differences were confined to local-name bytes, `.strtab` sizes, and later file offsets.

The existing pure Rust ELF core canonicalized real retained and fresh `regex.o` variants to the same BLAKE3 identity `76bbbe9f0a27ff0ba6a3553a51b941215d91215bfd23fa97b9a178ae33607a0b`.

## Bounded repair

`src/stagex_archive_core.rs` accepts only the deterministic GNU archive shape produced by `stagex-binutils-ar-runner.sh`. It validates zero metadata, mode `100644`, bounded short or GNU long names, exact padding, member limits, and ELF-only object members. It applies the existing fixed 10-digit local-symbol core to each member and rebuilds the same member order and names.

Mantle applies this pure core immediately after each of the four protected archive Make targets. It publishes changed bytes through a create-new sibling before later components consume the archive. It retains the protected C helper at every compiler and linker boundary.

Retained and fresh `libiberty.a` converge to `9e054fcfdd504399c978714e8013154c57eb51951ecbf6ae71b52ab550d3af01`. The other canonical archive identities are:

- `libz.a`: `7cfe930119c77ef8bd7f34ec122de1a7f644851183cd7bc82dadc27b6bbb54b4`
- `libbfd.a`: `94a9d681cbacecb5edc26a100fd94b860332115f0adf65d89cffeb5a4038144e`
- `libopcodes.a`: `d75ecd69a0f592d5b8fa4d9189d3af133c98c369ec5f89939e55371eb72a6dda`

The focused observation and enforced full binutils rails passed. All four protected executable identities and all eleven installed-tool identities remained unchanged.

## Non-claims

This repair does not prove TinyCC correctness, arbitrary ELF or archive equivalence, binutils correctness, transition completion, provider admission, a Mantle fixed point, or release eligibility.
