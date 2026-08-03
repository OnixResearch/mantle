# Mantlepkgs source-update plans

Mantlepkgs update plans replay explicit source, advisory, and validation evidence. The pure core selects one candidate and plans exact structured edits.

## Policy

Author policy with `contracts.ncl`. Export it to JSON before you run the CLI.

Policy binds:

- package and system selectors;
- source adapter, authority, query, and current lock facts;
- version, prerelease, development, and patch rules;
- OSV and Repology requirements;
- catalog, build, validation-root, and impact requirements;
- one safe relative JSON target and exact JSON pointers;
- response, candidate, finding, effect, diagnostic, artifact, time, redirect, retry, document, and plan limits.

Version one rejects ambient credentials and proxies. It does not accept shell fragments or arbitrary package update scripts.

## Seal the policy

Export the typed Nickel policy. Then seal its canonical BLAKE3 identity:

```console
nickel export policy.ncl --format json > policy-unsealed.json
mantle mantlepkgs update-policy-seal \
  --policy policy-unsealed.json \
  --out policy.json
```

The output path must differ from the input path.

## Record source evidence

Use a saved response for deterministic offline replay:

```console
mantle mantlepkgs update-source-observe \
  --policy policy.json \
  --response git-tags-response.json \
  --out source-observation.json
```

The response schema is selected by `source_kind`. Git tags, release indexes, directory indexes, and ecosystem registries use separate schema names.

The configured URL is a credential-free adapter endpoint. It must return the selected Mantle response envelope. Do not point it at an upstream API that returns a different schema.

You can use `--url` only when it exactly matches the configured HTTPS query. The HTTP shell disables ambient proxies and credentials. It applies policy time and byte limits and does not follow redirects.

Record a failed request explicitly:

```console
mantle mantlepkgs update-source-observe \
  --policy policy.json \
  --status unavailable \
  --reason source-timeout \
  --out source-observation.json
```

An unavailable or failed observation cannot contain candidates.

## Record advisory evidence

```console
mantle mantlepkgs update-advisory-observe \
  --policy policy.json \
  --service osv \
  --version v1.3.0 \
  --response osv-response.json \
  --out osv-observation.json
```

Use the same command with `--service repology`. Each response must include the exact package coordinate and version from the request. A successful empty finding set is different from unavailable or failed evidence.

## Build a dry-run plan

```console
mantle mantlepkgs update-plan \
  --policy policy.json \
  --source-observation source-observation.json \
  --advisory-observation osv-observation.json \
  --advisory-observation repology-observation.json \
  --validation-evidence validation.json \
  --source-root ./package-policy \
  --out update-plan.json
```

The command reads only explicit local files. It does not run Nix, a producer, a build, or a network request.

The plan binds the exact input file digest, old field values, new field values, canonical output digest, candidate, observations, validation artifacts, reasons, and policy.

## Publish an immutable update tree

```console
mantle mantlepkgs update-execute \
  --plan update-plan.json \
  --source-root ./package-policy \
  --output-root ./candidate-update \
  --denial-receipt-out ./update-denied.json
```

Execution does not edit the source tree. It performs these actions:

1. Reject symlinks and unsafe path components.
2. Re-read every source file.
3. Verify each input digest and old field value.
4. Write all outputs to one private stage.
5. Verify every canonical output digest.
6. Add the execution receipt to the stage.
7. Publish the complete directory with no-clobber rename.

A failure removes the stage, leaves source files unchanged, and writes a typed denial receipt. The receipt includes the stable execution failure code when one is available. An existing output directory is never replaced.

## Claim boundary

Update plans report evidence under one exact policy. They do not prove source trust, advisory completeness, package correctness, reproducibility, deployment safety, or release eligibility.
