# Proposal: Add security and contribution guides

## Why

The repository had no private vulnerability-reporting route and no contribution
guide. Security-relevant findings were filed publicly, and contributors had to
infer the supported shell, gates, Cairn lifecycle, and licensing terms.

## What Changes

- Add `SECURITY.md` with private GitHub advisory reporting, scope, and
  expectations.
- Add `CONTRIBUTING.md` with the dev shell, gates, Cairn workflow, evidence
  rules, and AGPL-3.0-or-later contribution terms.
- Link both from the README.

## Impact

- **Immediate outcome**: reporters and contributors have explicit routes.
- **Durable capability**: repository-owned reporting and contribution policy.
- **Maintenance owner**: Mantle maintainers.
- **Evidence**: file presence, README links, and stated commands matching
  `AGENTS.md`.

## Out of Scope

- Enabling repository settings or branch protection.
- Legal advice or changing the license.
