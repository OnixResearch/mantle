# Cairn validation after Aspen witness

Date: 2026-06-29

## nix run path:/home/brittonr/git/cairn#cairn -- validate --root .

~~~text
{
  "change_issues": [],
  "changes": 1,
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "spec_issues": [],
  "specs_validated": 8,
  "valid": true
}
~~~

exit status: 0

## nix run path:/home/brittonr/git/cairn#cairn -- gate proposal stabilize-self-hosting-witness-replay --root .

~~~text
{
  "change": "stabilize-self-hosting-witness-replay",
  "evidence_summary": {
    "docs_only": 0,
    "fixture": 0,
    "formal": 0,
    "mode": "advisory",
    "model": 0,
    "probe": 0,
    "property": 0
  },
  "input_hash": "55ba910d8b46d6adbc01fa9d9534ff246b0ab5429e7dad020c15ca69ce1302b9",
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "policy_hash": "2ba17ace71e36a2d8f03f0dc5eaa805a6008e970f2e56a53ff72b891601de119",
  "receipt_hash": "13a19a4a78034ad1c9415d3622afa08770445e7c7a23ca59d153a9cd09936e8f",
  "stage": "proposal",
  "valid": true,
  "verdict": "PASS"
}
~~~

exit status: 0

## nix run path:/home/brittonr/git/cairn#cairn -- gate design stabilize-self-hosting-witness-replay --root .

~~~text
{
  "change": "stabilize-self-hosting-witness-replay",
  "evidence_summary": {
    "docs_only": 0,
    "fixture": 0,
    "formal": 0,
    "mode": "advisory",
    "model": 0,
    "probe": 0,
    "property": 0
  },
  "input_hash": "61d0aff6c53be526209838a87b431fc2d6be0c0ccbf4e65a1f46313c21dea37d",
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "policy_hash": "2ba17ace71e36a2d8f03f0dc5eaa805a6008e970f2e56a53ff72b891601de119",
  "receipt_hash": "38b43f3fb01f97e7afa6152d8aa97db24998b3a2b1448a446f98ab04baab15ec",
  "stage": "design",
  "valid": true,
  "verdict": "PASS"
}
~~~

exit status: 0

## nix run path:/home/brittonr/git/cairn#cairn -- gate tasks stabilize-self-hosting-witness-replay --root .

~~~text
{
  "change": "stabilize-self-hosting-witness-replay",
  "evidence_summary": {
    "docs_only": 0,
    "fixture": 0,
    "formal": 0,
    "mode": "advisory",
    "model": 0,
    "probe": 0,
    "property": 0
  },
  "input_hash": "9e57385a81a56a5e1886e9873f7286bab635005d5c7c6290a5d6ba731408dca6",
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "policy_hash": "2ba17ace71e36a2d8f03f0dc5eaa805a6008e970f2e56a53ff72b891601de119",
  "receipt_hash": "eb192c20f15ca940a3d60d6ff4bc69d7b4fd66b524a02c6ba24b6b0b639463a9",
  "stage": "tasks",
  "valid": true,
  "verdict": "PASS"
}
~~~

exit status: 0

