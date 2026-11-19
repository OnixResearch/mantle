# Security policy

## Reporting a vulnerability

Please report suspected vulnerabilities privately through GitHub's
**Security → Advisories → Report a vulnerability** flow for
[`OnixResearch/mantle`](https://github.com/OnixResearch/mantle/security/advisories/new).
Do not open a public issue, discussion, or pull request for an exploitable
finding before a fix or coordinated disclosure is agreed.

If the private advisory flow is unavailable, contact the repository owners
through their GitHub profiles and ask for a private channel; do not include
exploit details in that first message.

## Scope

Mantle is research software with no supported release line. Reports are still
welcome for:

- the build scheduler, store capabilities, sandbox and command execution paths;
- signing-key loading, PathInfo publication, substitution trust, and GC;
- bounded acquisition, archive extraction, and remote worker/message handling;
- protected-exec supervision and audit/receipt integrity;
- dependency or CI supply-chain issues in the checked-in build inputs.

Out of scope for this policy: findings that require compromising the host
kernel, the Nix daemon, a upstream dependency's own infrastructure, or a
third-party cache; social engineering; and reports about unsupported local
modifications. A finding in a dependency can still be reported here when
Mantle's use of it creates the exposure.

Do not test against systems you do not own or have permission to test. Use
local fixtures and temporary state directories. Never include real signing
keys, credentials, or private store contents in a report.

## What to expect

- Acknowledgement of the report, normally within five business days.
- An initial triage decision and a request for a minimal reproduction where
  needed.
- Coordination on a fix and disclosure timing for confirmed issues.
- Public credit if you want it, unless you ask to remain anonymous.

There is no bug bounty or guaranteed response time. Reports are handled on a
best-effort basis by the maintainers. A fix is validated against the
repository's scoped evidence and gates; it is not a claim that arbitrary
deployments or upstream components are secure.
