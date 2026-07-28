# Multi-output SDK

This project builds one C SDK derivation with four outputs:

- `out`: stripped runtime CLI
- `dev`: public header, static library, and pkg-config metadata
- `doc`: guide and manual page
- `debug`: unstripped executable

Downstream checks consume only the required output. The development consumer compiles against `dev`. The runtime consumer executes `out`.

```sh
cd examples/projects/multi-output-sdk
mantle build
mantle build .#sdk
mantle build .#checks.runtime
mantle build .#checks.development
```

The first build can fetch Mantle's pinned bootstrap C toolchain. Output selection proves the declared dependency projection. It does not prove ABI stability or compiler correctness.
