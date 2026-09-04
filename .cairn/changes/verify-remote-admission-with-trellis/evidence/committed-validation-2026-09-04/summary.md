# Committed Trellis admission validation

Date: 2026-09-04
Implementation commit: `6ff60dcd91437306180819ffc4da27b8b7c1c00f`

The committed source passed these checks:

- 38 `crunch-build` remote-attempt tests;
- 9 `crunch-release-core` Trellis tests;
- 14 root remote-attempt tests;
- 20 root gateway tests;
- 2 evidence integration tests;
- the evidence checker and its mutation self-test;
- machine contracts with 33 contracted and 67 classified surfaces;
- strict Clippy for `crunch-build` and `crunch-release-core`;
- the documented strict first-party root Clippy command;
- first-party formatting and the commit diff check;
- all three focused Trellis Nix checks;
- the pinned Tiger Style Nix gate;
- Nix flake evaluation with `--no-build`;
- Cairn validation and all three change gates;
- Tracey coverage for all 157 accepted pre-sync requirements.

The current checks preserve Mantle runtime authority. They accept only the named abstract Trellis properties and the recorded evidence linkage.
