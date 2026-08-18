# Design

## Functional core

Add a pure filter over candidate parent environment values:

- input: candidate `(key, value)` pairs
- output: only keys in a fixed allowlist
- first allowlisted key: `SNIX_BUILD_SANDBOX_SHELL`

`rust_topology_child_env(...)` should accept already-filtered compile env plus inherited PATH and explicit derivation env. Apply order:

1. inherited PATH convenience
2. allowlisted compile env
3. explicit derivation env

Explicit derivation env remains authoritative if a key overlaps.

## Imperative shell

`apply_rust_topology_child_env(...)` remains the only place that reads process environment. It collects allowlisted values from `std::env`, passes them through the pure filter, clears the child env, and writes the resulting map onto the command.

## Risk

Forwarding too much ambient env would hide missing derivation inputs. Keep the allowlist tiny and test rejection of unrelated variables.
