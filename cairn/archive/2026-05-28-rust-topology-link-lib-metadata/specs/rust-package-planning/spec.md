## ADDED Requirements

### Requirement: Native link-lib build-script metadata

r[rust_package_planning.native_link_lib_metadata] Mantle MUST parse bounded Cargo-compatible `rustc-link-lib` build-script metadata without treating supported link-kind modifiers as malformed metadata.

#### Scenario: Modifier-bearing static link metadata is accepted

GIVEN a native build script emits `cargo:rustc-link-lib=static:+whole-archive=crypto_core`
WHEN Mantle parses build-script metadata
THEN Mantle MUST accept the directive as `rustc_link_lib` metadata.
AND Mantle MUST preserve the original directive value for deterministic rustc argument replay.

#### Scenario: Ordinary link metadata remains accepted

GIVEN a native build script emits `cargo:rustc-link-lib=static=aws_lc_0_39_1_crypto`
WHEN Mantle parses build-script metadata
THEN Mantle MUST accept the directive as `rustc_link_lib` metadata.
AND Mantle MUST continue to accept safe unqualified link names.

#### Scenario: Unsafe link metadata remains rejected

GIVEN a native build script emits empty, whitespace-containing, path-like, or unsupported-kind `rustc-link-lib` metadata
WHEN Mantle parses build-script metadata
THEN Mantle MUST reject the directive with deterministic `malformed-build-script-metadata` diagnostics.
AND Mantle MUST NOT pass unsafe link metadata through to rustc.

#### Scenario: Link-lib parser frontier moves

GIVEN topology execution currently blocks with `rustc-link-lib name must be a safe token`
WHEN bounded link-lib metadata parsing is applied
THEN self-probe verification MUST show that this parser blocker no longer stops the topology.
AND any remaining blocker MUST be recorded with deterministic class and evidence.
