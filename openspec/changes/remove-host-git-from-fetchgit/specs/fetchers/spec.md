## MODIFIED Requirements

### Requirement: fetchGit — clone a git repository

The Nickel stdlib MUST provide `crunch.fetchGit` that materializes a git
repository at a specific revision and stores the working tree without `.git/`.

```nickel
let src = crunch.fetchGit {
  url = "https://github.com/user/repo.git",
  rev = "abc123def456...",
  hash = "sha256-XXXX...",
} in
```

The resulting derivation MUST have:

- `builder = "builtin:fetchurl"`
- `system = "builtin"`
- `fixed_output.mode = 'recursive` (NAR hash of checkout)
- `env.url` set to the URL
- `env.type = "git"`
- `env.rev` set to the commit hash

The implementation MUST use a crunch-controlled git materialization path. It
MUST NOT discover or depend on an arbitrary host `git` from `PATH`.

The `.git/` directory MUST NOT appear in the output.

#### Scenario: Clone at specific rev without host git discovery

- GIVEN `crunch.fetchGit { url = "...", rev = "abc123...", hash = "..." }`
- WHEN `crunch build` runs
- THEN the repo is fetched and checked out at the specified revision
- AND the fetch path does not depend on an arbitrary host `git` found through `PATH`
- AND the output tree excludes `.git/`

#### Scenario: Host PATH git is irrelevant

- GIVEN the host has no `git` on `PATH`, or has a different `git` version than another host
- WHEN the same `fetchGit` derivation is built on both hosts
- THEN crunch uses the same crunch-controlled fetch implementation on each host
- AND host `git` availability or version does not change the fetch semantics

#### Scenario: Invalid rev remains a fetch error

- GIVEN a rev that does not exist in the requested repository
- WHEN the fetch runs
- THEN the error reports that the requested revision could not be materialized
- AND the error does not depend on host `git` stderr formatting
