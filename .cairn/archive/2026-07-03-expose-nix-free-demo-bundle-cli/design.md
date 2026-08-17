## Context

The demo bundle validator should remain a functional core. The CLI should only read files, choose output mode, call the validator or renderer, and translate the result into exit status plus human or JSON output.

## Decisions

### 1. CLI shell around the existing pure core

**Choice:** Add the command as a thin imperative shell over `validate_nix_free_demo_bundle` and `render_nix_free_demo_readme`.

**Rationale:** This keeps validation deterministic and testable without filesystem or terminal mocks.

### 2. JSON diagnostics are stable

**Choice:** Machine output must include stable diagnostic codes and a boolean claimability verdict.

**Rationale:** Release-readiness and demo automation need to consume the same decision that human output displays.

### 3. README generation is explicit

**Choice:** README rendering should be a separate command or flag that writes generated text from the machine summary and validation result.

**Rationale:** Generated documentation should not be confused with validation itself, and users should be able to regenerate it deterministically.

## Risks / Trade-offs

- Naming the CLI surface too broadly could imply a wider proof system than the current demo profile supports.
- JSON compatibility should be kept narrow until more proof profiles exist.
- The command must preserve default JSON stdout cleanliness and put diagnostics on the documented channel.
