# Complete example projects

These directories combine source/configuration, package selectors, checks, and project-local runbooks. Run each command from the named project directory.

| Project | Default package | Check | Capability |
|---|---|---|---|
| `generated-site/` | `mantle build` | `mantle build .#checks.site-content` | local + fast |
| `codegen-pipeline/` | `mantle build` | `mantle build .#checks.app` | local + fast |
| `c-library-cli/` | `mantle build` | `mantle build .#checks.test-greet` | heavy + first-build network |
| `rust-workspace/` | `mantle build` | `mantle build .#checks.smoke` | heavy + first-build network |

The C and Rust projects also expose `.#source`, which assembles their BLAKE3-fixed local source files without realizing a compiler toolchain. Their default packages use Mantle-owned bootstrap/toolchain inputs rather than ambient host compilers or Cargo caches.

Project checks cover successful behavior and explicit invalid-input rejection. Build success proves only the selected local build and check; it does not prove compiler correctness, release reproducibility, or complete language ecosystem compatibility.
