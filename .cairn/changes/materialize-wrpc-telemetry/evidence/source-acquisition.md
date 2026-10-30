# Source acquisition only

The source archive was actually acquired using:

`nix store prefetch-file --json https://codeload.github.com/WebAssembly/wasi-otel/tar.gz/e5262a84044aff064c9599eebc05013bb3432795`

It returned `sha256-ZSESQKRr7xZkrkeK53J/TOtc8hQWTJKKg0Aw2tWZHS8=`.
The acquired source WIT confirms tracing's on-start(span-context),
on-end(span-data), current-span-context()->span-context functions and JSON-string
attribute values. Exact WIT files were retained from those archive bytes.

`cargo generate-lockfile --manifest-path packages/wrpc-telemetry-reference/guest/Cargo.toml`
resolved43 packages, retaining wit-bindgen0.36.0. The generated lock is retained
verbatim, not hand-authored or trimmed.

No generated component digest, build/smoke/consumer test result, trusted UCAN
host/export observation, lifecycle gate or immutable publication revision was
observed or claimed. The consumer explicitly accepted a real comparison sink only
as producer smoke, never trusted host evidence.
