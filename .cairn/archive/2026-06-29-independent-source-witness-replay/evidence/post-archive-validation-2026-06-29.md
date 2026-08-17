# Post-archive validation for independent source witness replay

Date: 2026-06-29

Archive note: `cairn archive --execute` initially created `cairn/archive/1970-01-01-independent-source-witness-replay`. Following the repository guidance for this known Cairn date issue, it was manually renamed to `cairn/archive/2026-06-29-independent-source-witness-replay` before validation.

Sync note: the archive command moved the change package but did not sync the delta requirement into `cairn/specs/verification-evidence/spec.md`; the accepted `Independent source witness replay` requirement was copied manually from the archived delta into the canonical spec before the validation below.

## nix run path:/home/brittonr/git/cairn#cairn -- validate --root .

~~~text
{
  "change_issues": [],
  "changes": 0,
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "spec_issues": [],
  "specs_validated": 6,
  "valid": true
}
~~~

Exit status: 0

## Canonical spec sync check

Command:

~~~text
rg -n "Independent source witness replay|independent_source_witness_replay|source_acquisition" cairn/specs/verification-evidence/spec.md
~~~

Output excerpt:

~~~text
cairn/specs/verification-evidence/spec.md:352:### Requirement: Independent source witness replay [r[verification_evidence.independent_source_witness_replay]]
~~~
