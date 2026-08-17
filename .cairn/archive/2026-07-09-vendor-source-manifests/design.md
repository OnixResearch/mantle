## Design

Define a vendor manifest with upstream identity, revision, filter definition, path allowlist, artifact digests, local patch list, refresh command, and non-claims. The pure checker validates parsed manifest rows against measured file identities provided by the shell.

The manifest proves vendored-source identity and freshness only; it does not certify upstream quality or license compatibility unless separate evidence exists.
