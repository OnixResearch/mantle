# Design: fail-closed GCC 4.0 configure bridge

## Context

`bootstrap/gcc-4.0-native.ncl` generates a compiler wrapper used by three explicit configure invocations. Its `-E` branch removes Autoconf's deliberate syntax-error sentinel, compiles the bounded probe, and emits only the configure-facing result. The mechanism is useful predecessor construction, but it is not a conforming preprocessor and must not become ambient compiler authority.

## Decision

### Runtime admission core

Treat each `-E` invocation as facts checked before compiler execution:

- bridge authority directory is present;
- canonical current directory equals that authority;
- source spelling is `conftest.c` or `./conftest.c`;
- canonical source parent equals the authority directory;
- no `-o` output was requested;
- source size is at most the named 64 KiB probe limit;
- accepted invocation count remains at most the named 4,096-invocation limit; and
- the authority directory class is exactly `libiberty`, `libcpp`, or `gcc`.

The shell is the imperative executor. A Rust checker owns deterministic source/evidence validation and pure bridge-fact decisions for positive and negative fixtures.

### Audit

The configure shell supplies a build-local authority directory, class, and audit path only for each configure child. The wrapper appends one bounded tab-separated record after admission and before compiler execution. After all three configure runs, the parent requires a non-empty audit, a bounded count, and only the three declared classes.

### Evidence

`bootstrap/evidence/gcc-4.0-configure-preprocess-bridge.json` records the limits, allowed classes, output policy, provider eligibility, and explicit non-claims. The checker binds that evidence to exact derivation markers and fails closed on stale or broadened policy.

## Rejected Alternatives

- **Treat comments as sufficient:** rejected because comments do not constrain runtime inputs.
- **Preseed every Autoconf result:** rejected because a large cache becomes an opaque substitute for configure behavior.
- **Claim the bridge is a provider preprocessor:** rejected because it intentionally does not implement general preprocessing semantics.
- **Require immediate native GCC replacement:** rejected as a stronger compiler-construction problem; this change narrows the current mechanism without synthesizing that proof.

## Validation

Run checker self-tests, GCC 4.0 bridge unit tests, bootstrap evaluation, Tiger Style, first-party quality, blocker inventory, machine contracts, Cairn gates, Tracey, and `nix flake check -L`. If the broad gate reaches an unrelated infrastructure or content blocker, retain its exact derivation and diagnostic without claiming success.