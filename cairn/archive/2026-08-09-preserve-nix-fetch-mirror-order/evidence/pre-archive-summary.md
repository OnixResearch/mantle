# Pre-archive summary

Mantle now normalizes supported Nix `url`, `urls`, and structured `__json.urls` facts into one bounded ordered candidate list.

The compiler binds the list into target derivation and executable-plan identity. The fetch service receives only canonical `url` and `__mantle_foreign_candidates` values.

Validation rejects empty, malformed, duplicate, oversized, conflicting, unsupported, unresolved `mirror://`, credential-bearing, and reserved-field inputs. Arbitrary fixed-output builders do not become downloads.

Focused final results:

- Producer: 27 passed.
- Compiler: 19 passed.
- Fetch service: 24 passed.
- CLI: 16 passed and one external parity test ignored.
- First-party Clippy: passed with warnings denied.
- Changed-file Rustfmt: passed.
- Foreign import trust-model check and self-test: passed.
- Cairn validation: valid with no findings or issues.
- Proposal, design, and tasks gates: passed.
- Tracey: 155/155 referenced.

The package-wide formatting command retains the unchanged `src/source_built_fixed_point_shell.rs` difference. This change does not modify that excluded fixed-point file.

This evidence does not claim arbitrary Nix fetcher parity, mirror trust, Nix evaluation, Nixpkgs parity, source correctness, build correctness, or release eligibility.
