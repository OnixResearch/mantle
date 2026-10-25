# Streamed builds: dependencies requested as roots later

`mantle build` evaluates roots as a stream. A later root can name a derivation
that an earlier root already dispatched as a dependency. Mantle now treats root
status as current when each goal finishes:

- A dependency requested as a root while it is building is exported to the
  physical `--store` when it completes.
- A dependency requested as a root after it completed is exported from its
  persisted signed PathInfo and castore content, and reported once as a root
  outcome. A configured retained-root source registers it as for any root.
- A dependency requested as a root after it failed is reported once as a failed
  root with its own failure.

Previously, the first two cases could fail the build with `reported build output
is missing`, and a root requested after its dependency finished could be omitted
from the result. Signed PathInfo, output paths and cache identities are
unchanged; this change adds no new build, trust or cache authority.
