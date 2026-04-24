## ADDED Requirements

### Requirement: CLI MUST export verifier-ready trusted public keys from signing keys

Crunch MUST provide `crunch attest key-show` to print the verifier-ready
trusted public key string for an existing signing keypair file.
ID: release.verification.tech.trustedkey.export.cli

The command MUST:
- accept an explicit signing-key path,
- support the default config signing-key location when no explicit path is
  given,
- fail clearly when no signing key exists,
- avoid generating or mutating key material as a side effect, and
- print the exact `name:base64` token accepted by `--trusted-public-key`.

#### Scenario: Explicit signing-key path prints verifier token

- GIVEN an existing signing keypair file passed with `--signing-key`
- WHEN the operator runs `crunch attest key-show`
- THEN the command exits successfully
- AND it prints the exact `name:base64` trusted public key token for that key

#### Scenario: Default config signing key prints verifier token

- GIVEN no explicit signing-key path
- AND the default config signing-key file exists
- WHEN the operator runs `crunch attest key-show`
- THEN the command exits successfully
- AND it prints the exact `name:base64` trusted public key token for that key

#### Scenario: Missing signing key fails without key generation

- GIVEN no explicit signing-key path and no default config signing-key file
- WHEN the operator runs `crunch attest key-show`
- THEN the command exits non-zero
- AND the diagnostic says that no signing key was found
- AND the command does not generate a new key file
