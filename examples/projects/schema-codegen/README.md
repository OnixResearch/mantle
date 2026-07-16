# Schema-generated C and Rust applications

A checked-in schema and generator produce equivalent C and Rust bindings. Mantle then compiles a C CLI with the bootstrap C toolchain, builds a Rust CLI through offline Cargo, and runs one integration check against both outputs.

```sh
cd examples/projects/schema-codegen
mantle build .#bindings
mantle build .#c-app
mantle build .#rust-app
mantle build .#checks.integration

# Expected to fail before producing bindings:
mantle build .#invalid-schema
mantle build .#unsafe-schema
```

`bindings.manifest` records the normalized schema values. The integration check covers default, explicit-name, and invalid-argument behavior. Matching outputs do not prove general C/Rust semantic equivalence or generator correctness for schemas outside this bounded format.
