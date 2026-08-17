# Distributed build Cairn scaffold validation

## Changes
- distributed-route-planning
- remote-build-service-dispatch
- stdio-ssh-remote-builds
- source-bundle-remote-input-sync
- remote-output-trust-admission
- coordinator-worker-scheduling
- remote-build-observability

## Command

```text
nix run path:/home/brittonr/git/cairn#cairn -- validate --root .
for each change: gate proposal, gate design, gate tasks
```

## Output

```text
{
  "change_issues": [],
  "changes": 15,
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "spec_issues": [],
  "specs_validated": 31,
  "valid": true
}
{
  "change": "distributed-route-planning",
  "evidence_summary": {
    "docs_only": 0,
    "fixture": 0,
    "formal": 0,
    "mode": "advisory",
    "model": 0,
    "probe": 0,
    "property": 0
  },
  "input_hash": "e72b59338b151fa91e3bede1052422d7d1a94df5952a7c64db880ad9424505a2",
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "policy_hash": "2ba17ace71e36a2d8f03f0dc5eaa805a6008e970f2e56a53ff72b891601de119",
  "receipt_hash": "410e901e244636f9338e2fbd6cfa03fbfec9df9fac3b6f61f1e8ffab4121741e",
  "stage": "proposal",
  "valid": true,
  "verdict": "PASS"
}
{
  "change": "distributed-route-planning",
  "evidence_summary": {
    "docs_only": 0,
    "fixture": 0,
    "formal": 0,
    "mode": "advisory",
    "model": 0,
    "probe": 0,
    "property": 0
  },
  "input_hash": "cd379e00b149cd4d6f7449d839fbb755b759831e824f0dbe6935a7a6eeeae24d",
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "policy_hash": "2ba17ace71e36a2d8f03f0dc5eaa805a6008e970f2e56a53ff72b891601de119",
  "receipt_hash": "ec8c4ebb46a3779adb444c2604d75b5d7da3c84bf47165198c5cefc40034aeb9",
  "stage": "design",
  "valid": true,
  "verdict": "PASS"
}
{
  "change": "distributed-route-planning",
  "evidence_summary": {
    "docs_only": 0,
    "fixture": 0,
    "formal": 0,
    "mode": "advisory",
    "model": 0,
    "probe": 0,
    "property": 0
  },
  "input_hash": "bce5e237fe8a33d0712f17cde693c225716d68a040a53dd5fcfb9c7cc09988da",
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "policy_hash": "2ba17ace71e36a2d8f03f0dc5eaa805a6008e970f2e56a53ff72b891601de119",
  "receipt_hash": "ad8ad31d0a99b6d509b0aed521d6642befacbe1052758bbdd4b53d103dbfd344",
  "stage": "tasks",
  "valid": true,
  "verdict": "PASS"
}
{
  "change": "remote-build-service-dispatch",
  "evidence_summary": {
    "docs_only": 0,
    "fixture": 0,
    "formal": 0,
    "mode": "advisory",
    "model": 0,
    "probe": 0,
    "property": 0
  },
  "input_hash": "7f7b97c933ecd98b5e261a8920326494851c9e0161b096ece297119128d5046e",
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "policy_hash": "2ba17ace71e36a2d8f03f0dc5eaa805a6008e970f2e56a53ff72b891601de119",
  "receipt_hash": "530c8b5e230d90591a78c67e6713d0307f7a11beb5c7495a8d2d934ee2558c78",
  "stage": "proposal",
  "valid": true,
  "verdict": "PASS"
}
{
  "change": "remote-build-service-dispatch",
  "evidence_summary": {
    "docs_only": 0,
    "fixture": 0,
    "formal": 0,
    "mode": "advisory",
    "model": 0,
    "probe": 0,
    "property": 0
  },
  "input_hash": "b8d305988ead38ed1182f7bde3a8a4d0d575e21f43125d387fdcb1f8c43e72fc",
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "policy_hash": "2ba17ace71e36a2d8f03f0dc5eaa805a6008e970f2e56a53ff72b891601de119",
  "receipt_hash": "903951232be2039fe2b7f93cbfae0f96e95cb0989801f7e742951582f7297c96",
  "stage": "design",
  "valid": true,
  "verdict": "PASS"
}
{
  "change": "remote-build-service-dispatch",
  "evidence_summary": {
    "docs_only": 0,
    "fixture": 0,
    "formal": 0,
    "mode": "advisory",
    "model": 0,
    "probe": 0,
    "property": 0
  },
  "input_hash": "6c8e28d4830b26fbca686862c5ba02bcd6ca0e0f66203da78677d9c2708a5c68",
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "policy_hash": "2ba17ace71e36a2d8f03f0dc5eaa805a6008e970f2e56a53ff72b891601de119",
  "receipt_hash": "9578e19d74b52ed3e5f53eb557e2736fd22b60121b490f15e6b25f8a2df9c0da",
  "stage": "tasks",
  "valid": true,
  "verdict": "PASS"
}
{
  "change": "stdio-ssh-remote-builds",
  "evidence_summary": {
    "docs_only": 0,
    "fixture": 0,
    "formal": 0,
    "mode": "advisory",
    "model": 0,
    "probe": 0,
    "property": 0
  },
  "input_hash": "37e69bb8dcfef731ced851d5c2bca8a735278ab4c958da7de8e91bbae3500773",
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "policy_hash": "2ba17ace71e36a2d8f03f0dc5eaa805a6008e970f2e56a53ff72b891601de119",
  "receipt_hash": "48e3a74bfb20e410c254eeb9d91ec7b643524fcefac457252239de164f2aeeb8",
  "stage": "proposal",
  "valid": true,
  "verdict": "PASS"
}
{
  "change": "stdio-ssh-remote-builds",
  "evidence_summary": {
    "docs_only": 0,
    "fixture": 0,
    "formal": 0,
    "mode": "advisory",
    "model": 0,
    "probe": 0,
    "property": 0
  },
  "input_hash": "813b96803e362c43bb33044ab94b3cb8fa0574f9e44b3beafb01b7abf0bd0dbd",
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "policy_hash": "2ba17ace71e36a2d8f03f0dc5eaa805a6008e970f2e56a53ff72b891601de119",
  "receipt_hash": "fef813f767fb732b9ba287e5f6e1b9d760cbeae7afbe713e787a5828a24b6dd4",
  "stage": "design",
  "valid": true,
  "verdict": "PASS"
}
{
  "change": "stdio-ssh-remote-builds",
  "evidence_summary": {
    "docs_only": 0,
    "fixture": 0,
    "formal": 0,
    "mode": "advisory",
    "model": 0,
    "probe": 0,
    "property": 0
  },
  "input_hash": "886cc65a44a1e4c883a2be9bb050c5c738dfdef377f35ff335e2418620133d72",
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "policy_hash": "2ba17ace71e36a2d8f03f0dc5eaa805a6008e970f2e56a53ff72b891601de119",
  "receipt_hash": "4dcbadecf11a86af01c55100df6e3eac6ad16ffe691423552eb5e7d8b04ade5a",
  "stage": "tasks",
  "valid": true,
  "verdict": "PASS"
}
{
  "change": "source-bundle-remote-input-sync",
  "evidence_summary": {
    "docs_only": 0,
    "fixture": 0,
    "formal": 0,
    "mode": "advisory",
    "model": 0,
    "probe": 0,
    "property": 0
  },
  "input_hash": "5f5b18afb8138328dd4c33431631ac39747d62dfafc6cb2b09e893a925fd2182",
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "policy_hash": "2ba17ace71e36a2d8f03f0dc5eaa805a6008e970f2e56a53ff72b891601de119",
  "receipt_hash": "4911bd1cab76f86530a7ee337e41598f51c45cc9d4c97137a0ec6abe062a2aff",
  "stage": "proposal",
  "valid": true,
  "verdict": "PASS"
}
{
  "change": "source-bundle-remote-input-sync",
  "evidence_summary": {
    "docs_only": 0,
    "fixture": 0,
    "formal": 0,
    "mode": "advisory",
    "model": 0,
    "probe": 0,
    "property": 0
  },
  "input_hash": "86b966b602e15bb05ace54219bef8552205e7ac3cfea2886b3cbb705deaa3ee3",
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "policy_hash": "2ba17ace71e36a2d8f03f0dc5eaa805a6008e970f2e56a53ff72b891601de119",
  "receipt_hash": "e98e910c4961d1962c52df89d66d48311555efd812794ecf26c46a233e78de5f",
  "stage": "design",
  "valid": true,
  "verdict": "PASS"
}
{
  "change": "source-bundle-remote-input-sync",
  "evidence_summary": {
    "docs_only": 0,
    "fixture": 0,
    "formal": 0,
    "mode": "advisory",
    "model": 0,
    "probe": 0,
    "property": 0
  },
  "input_hash": "5b7e30e95581e524aefb7cf3500f0a071a15e365dace9279818fb282ad7172b7",
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "policy_hash": "2ba17ace71e36a2d8f03f0dc5eaa805a6008e970f2e56a53ff72b891601de119",
  "receipt_hash": "8bc58d91054e66b790a57944db25a65ead5ff82f12d71f5f9f3339effb80aab9",
  "stage": "tasks",
  "valid": true,
  "verdict": "PASS"
}
{
  "change": "remote-output-trust-admission",
  "evidence_summary": {
    "docs_only": 0,
    "fixture": 0,
    "formal": 0,
    "mode": "advisory",
    "model": 0,
    "probe": 0,
    "property": 0
  },
  "input_hash": "98ea09c3a546b1eb336224acedd347e5889db45f7268d0636ffc63ffa4d32f3d",
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "policy_hash": "2ba17ace71e36a2d8f03f0dc5eaa805a6008e970f2e56a53ff72b891601de119",
  "receipt_hash": "cd9b9b29406b6ce3b78abd074f05bb4f983740caa9378dca0141a9f47a61bf7e",
  "stage": "proposal",
  "valid": true,
  "verdict": "PASS"
}
{
  "change": "remote-output-trust-admission",
  "evidence_summary": {
    "docs_only": 0,
    "fixture": 0,
    "formal": 0,
    "mode": "advisory",
    "model": 0,
    "probe": 0,
    "property": 0
  },
  "input_hash": "47ccdc7f6a98a764cdfd6cb20a3f33a8bdbc59d97979fb8d29427cca16d8fdf2",
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "policy_hash": "2ba17ace71e36a2d8f03f0dc5eaa805a6008e970f2e56a53ff72b891601de119",
  "receipt_hash": "0ac9fb6a677bcfd4e6c858b836c59dbf55df021f15a109b4aa9d8717e1e354cd",
  "stage": "design",
  "valid": true,
  "verdict": "PASS"
}
{
  "change": "remote-output-trust-admission",
  "evidence_summary": {
    "docs_only": 0,
    "fixture": 0,
    "formal": 0,
    "mode": "advisory",
    "model": 0,
    "probe": 0,
    "property": 0
  },
  "input_hash": "d342e358312586483c649026ec57028dc9af71d98fbf81674f5f74783f44623f",
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "policy_hash": "2ba17ace71e36a2d8f03f0dc5eaa805a6008e970f2e56a53ff72b891601de119",
  "receipt_hash": "663151c8b35723fc6e369c60f0b629cc557deb962fa22e6403cc11be6ea12e1b",
  "stage": "tasks",
  "valid": true,
  "verdict": "PASS"
}
{
  "change": "coordinator-worker-scheduling",
  "evidence_summary": {
    "docs_only": 0,
    "fixture": 0,
    "formal": 0,
    "mode": "advisory",
    "model": 0,
    "probe": 0,
    "property": 0
  },
  "input_hash": "eb4da937780bded3ca0d682e95597b466339c857f59cf00ef76608b9b9fd9e67",
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "policy_hash": "2ba17ace71e36a2d8f03f0dc5eaa805a6008e970f2e56a53ff72b891601de119",
  "receipt_hash": "bb5f4699f47145a3e7880977141f788f90ed60847659e3bee3f727c816d7169a",
  "stage": "proposal",
  "valid": true,
  "verdict": "PASS"
}
{
  "change": "coordinator-worker-scheduling",
  "evidence_summary": {
    "docs_only": 0,
    "fixture": 0,
    "formal": 0,
    "mode": "advisory",
    "model": 0,
    "probe": 0,
    "property": 0
  },
  "input_hash": "a51b098b87346cd9de315d7e879ce130733864e11c8b6d7ea893285938d02242",
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "policy_hash": "2ba17ace71e36a2d8f03f0dc5eaa805a6008e970f2e56a53ff72b891601de119",
  "receipt_hash": "171aeede80f99476836ae58939e72b92def7e12ecc41f597ff4bbbef9e44198e",
  "stage": "design",
  "valid": true,
  "verdict": "PASS"
}
{
  "change": "coordinator-worker-scheduling",
  "evidence_summary": {
    "docs_only": 0,
    "fixture": 0,
    "formal": 0,
    "mode": "advisory",
    "model": 0,
    "probe": 0,
    "property": 0
  },
  "input_hash": "8d34e001e6b500f817d8aa218df38b9ebe9d1cffb1d311b835afaad73bac2df7",
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "policy_hash": "2ba17ace71e36a2d8f03f0dc5eaa805a6008e970f2e56a53ff72b891601de119",
  "receipt_hash": "c39657c5abb11729241b1c390a2e39312016ceea20c26eaad58098179f48ac6e",
  "stage": "tasks",
  "valid": true,
  "verdict": "PASS"
}
{
  "change": "remote-build-observability",
  "evidence_summary": {
    "docs_only": 0,
    "fixture": 0,
    "formal": 0,
    "mode": "advisory",
    "model": 0,
    "probe": 0,
    "property": 0
  },
  "input_hash": "80ece6a56b88de8a0ffc7ccf1fdb87cb376f2467839c98a674b690776d4a2de0",
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "policy_hash": "2ba17ace71e36a2d8f03f0dc5eaa805a6008e970f2e56a53ff72b891601de119",
  "receipt_hash": "40ea28b4718c41e0b6172d65b4b7963869a196d472749c1eb6fe6b92f83c9883",
  "stage": "proposal",
  "valid": true,
  "verdict": "PASS"
}
{
  "change": "remote-build-observability",
  "evidence_summary": {
    "docs_only": 0,
    "fixture": 0,
    "formal": 0,
    "mode": "advisory",
    "model": 0,
    "probe": 0,
    "property": 0
  },
  "input_hash": "076f53325bf2c51578a7fcb92c6c47bc0b8a8a44a7ad9e033496e6d2b4aa85da",
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "policy_hash": "2ba17ace71e36a2d8f03f0dc5eaa805a6008e970f2e56a53ff72b891601de119",
  "receipt_hash": "3893a6eec1be928cf27b92fb4fec4b1a49dd67b865660cff19c1dccf56d44d28",
  "stage": "design",
  "valid": true,
  "verdict": "PASS"
}
{
  "change": "remote-build-observability",
  "evidence_summary": {
    "docs_only": 0,
    "fixture": 0,
    "formal": 0,
    "mode": "advisory",
    "model": 0,
    "probe": 0,
    "property": 0
  },
  "input_hash": "20490b30afbb252cc4290f30dfb72f870c5e92ed0fb49bddfda3881e216f3999",
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "policy_hash": "2ba17ace71e36a2d8f03f0dc5eaa805a6008e970f2e56a53ff72b891601de119",
  "receipt_hash": "8a33d0143e9446b03d54247b4aedb5948d40e83b68cffb0319365b55a494afef",
  "stage": "tasks",
  "valid": true,
  "verdict": "PASS"
}
```
