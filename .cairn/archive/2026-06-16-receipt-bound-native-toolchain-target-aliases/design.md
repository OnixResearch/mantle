## Context

`cargo_free_self_build::execution_path_env(...)` creates a guard PATH containing wrapper scripts for declared toolchain closure members. It currently adds the executable basename and role aliases (`cc`, `ld`, `pkg-config`, `ar`, `ranlib`). Target build scripts such as `cc-rs` can derive a target-prefixed compiler name like `x86_64-linux-musl-gcc`; if the physical compiler path does not already have that basename, the lookup falls through to a missing ambient tool and the proof blocks.

## Decisions

### 1. Member-name aliases are receipt-bound PATH entries

**Choice:** Add each executable member's manifest `name` as a PATH alias, after validating it is a safe single path component and before executing any Rust unit.

**Rationale:** The manifest is the receipt-bound authority for tool identity. Letting it declare target-prefixed names keeps target build-script lookup deterministic without weakening the Cargo guard or inheriting ambient PATH.

### 2. Alias conflicts stay fail-closed

**Choice:** Reuse the existing alias conflict check for member-name aliases.

**Rationale:** If two manifest members claim the same executable name with different targets, Mantle cannot know which tool a build script would use. Failing before execution is safer than silently choosing one.

### 3. Generic host aliases stay host-oriented

**Choice:** Keep generic aliases such as `cc`, `ld`, and `ld.lld` bound to host tools in the closure manifest, and expose musl target compilers/linkers through target-prefixed member-name aliases such as `x86_64-linux-musl-gcc`.

**Rationale:** Rust host units and build-script executables still link for `x86_64-unknown-linux-gnu` before target units run. Mapping generic `cc` to the musl compiler makes host units fail and hides the real target-tool boundary. Target build scripts can request the target-prefixed aliases when compiling target C objects.

### 4. Full native closure remains separate from Rust provider closure

**Choice:** This change only fixes target-prefixed closure aliasing and validates a musl-target proof attempt. It does not claim release reproducibility or a minimized bootstrap trust root.

**Rationale:** The broader native closure still needs source-built linker/C/C++/sysroot/crt/runtime/pkg-config/helper evidence. The proof should improve determinism without overstating scope.

## Risks / Trade-offs

- A member name becomes part of the executable surface; validation must reject separators, empty names, and `cargo`.
- The immediate proof can still block on missing source-built native tool material. That blocker must be recorded as evidence rather than hidden by ambient PATH.
