# Tasks: Repair i386 TinyCC 0.9.26 emission proof

## Diagnostics

- [x] D1 Add a sibling diagnostic derivation that splits `tcc26-i386` version, assemble-only, link-from-assembly, link-from-object, and run-output stages. [covers=bootstrap.i386-tinycc26-emission.diagnostics]
- [x] D2 Run the diagnostic derivation and save the transcript/evidence. [covers=bootstrap.i386-tinycc26-emission.diagnostics]
- [x] D3 Interpret the failure boundary and record the next repair target. [covers=bootstrap.i386-tinycc26-emission.decision]
- [x] D4 Run OpenSpec helper verification before commit. [covers=bootstrap.i386-tinycc26-emission.diagnostics]
