# Security review of the attenuation grant sites, 2026-10-30

Subject: the work on branch `agent/attenuate-sites-20261030`, integrated to main as the nine commits from
`feat(store): gate restricted output lookups…` to `docs: drop stale untested-duplicate-ordinal sentence`.
Reviewer: a read-only security-review agent (no execution; it read the changed files and the pinned Basalt, UCAN
and replay-ledger code they call). Two passes; the structured output of each is retained verbatim:
`security-review-pass1-2026-10-30.json` (BLAKE3 `3b17e3cfbef31ac833f9b871f7e28dca8ffdcc44b81a705e443fd0a4d65f7d68`) and
`security-review-pass2-2026-10-30.json` (BLAKE3 `54ed6795648f885c605c92f01475af316e0f73e952d556611b19bb787c7dd451`).

## Verdicts

- Pass 1: `merge-with-followups`. On the production receiver the new path is mechanically sound and
  fail-closed. Invariants I1 to I7 hold; I8 (nested project goals) is violated by pre-existing code this change did
  not touch; I9 is unverified because Mantle issues no grants.
- Pass 2 (on the follow-up commits): `merge-with-followups`. It found two problems that the first round of
  fixes introduced, both repaired in round 3 and recorded below.

**The headline finding stands: attenuation is advisory.** The presentation is optional and the ticket bearer alone
admits, so a holder who also has the bearer can drop the restriction. Nothing here makes a grant non-bypassable. The
work is merged as implemented, not accepted, groundwork; tasks T3.1 to T3.4 stay unticked.

## Findings and where each stands at the merged tip

Pass 1:

| Id | Severity | Finding | Disposition |
|---|---|---|---|
| F1 | high | Remote attenuation is advisory: the presentation is optional and the bearer alone admits, so any delegatee can drop the restriction (the weakness ADR… | **Open**. Attenuation is advisory. Stated prominently in `docs/remote-credentials.md` and the status record. An enforcing mode (a ticket-level requirement or a provisioned-authority rule) needs an owner decision and a ticket-schema change. |
| F2 | medium | Job identity is a sender-chosen label: caveats see only request_id and payload kind, not the production job_id, declared output names or payload conte… | **Partly**. Superseded by R2-1. The filter-visible identity is now request id, attempt job id, payload class and an output-set id. Payload bodies, inputs and resource requests are bound only by the holder signature, so a grant is label-scoped; documented. |
| F3 | medium | Replay ledger is capped at 8,192 records with no expiry/GC, shared by all holders; exhaustion silently and permanently disables every attenuated admis… | **Open**. The shared replay ledger holds at most 8,192 records with no expiry, and exhaustion silently disables attenuated admission. Documented with a rotation procedure; no GC or quota added. |
| F4 | low | Holder attenuation is operator-gated: parent proofs and every delegate/holder key must be pre-provisioned in the receiver snapshot; the presentation c… | **Documented**. Parent proofs and holder keys must be provisioned in the receiver snapshot; the presentation carries none. |
| F5 | low | Documentation and doc-comment claim the digest covers the whole request; eight request fields are unbound | **Fixed**. The digest covers the whole `ConcreteBuildRequest` (domain v2, endpoint id, `serde_json` encoding). A golden vector and a test per formerly unbound field exist. The ticket id stays unbound and is disclosed. |
| F6 | low | remote serve does not fail at start on a content-invalid snapshot, contrary to the doc comment and docs | **Fixed**. `open_operator_authority` loads the snapshot at start; twelve failure modes are tested at helper level. Consequence documented (R2-3). |
| F7 | low | A presented grant is silently ignored by non-enforcing admission paths; --executor defaults to the fixture path | **Partly**. Loopback, frame-level and fixture paths refuse a presented grant with the constant string. The default executor, the hello capability and older receivers are unchanged and documented. |
| F8 | low | Restricted store view has no production producer; Info opens the store before the gate; admitted PathInfo discloses out-of-view references | **Partly**. `store info` is gated before `open_store`. The store view still has no production producer, and an admitted PathInfo still shows references and deriver; documented. |
| F9 | low | No local diagnostics on denial (contrary to the design) and the audit line is logged before redemption commits | **Fixed**. Reason codes are logged locally and the admitted line now follows the redemption commit. The sink problem is R2-2. |
| F10 | low | UCAN verification, snapshot parsing and an fsync'd ledger rewrite now run under the global ticket-state lock | **Open**. UCAN verification and the ledger write still run under the ticket lock. Recorded only. |
| F11 | low | Test gaps and a tautological assertion limit what the remote fixtures prove | **Fixed**. State and ledger bytes are captured before each refused admission; 29 attributed refusal cases and a serve-path positive. Gaps are listed under R2-4. |
| F12 | low | Project site and chain report: nested goals admitted, no production caller, report overclaims | **Partly**. A zero-caveat chain is labelled `ucan-unrestricted`. Nested goals and the missing production callers are documented, not changed. |
| F13 | info | Snapshot rollback guard is per process, but remote serve is stdio-once; revocation freshness equals file contents | **Open**. The rollback guard is per process and revocation freshness is the snapshot window. Documented. |

Pass 2:

| Id | Severity | Finding | Disposition |
|---|---|---|---|
| R2-1 | medium | Variable-length output-name tail makes reject (denylist) and wildcard caveats shape-dependent: a reject on a job id or class is evaded by padding the… | **Fixed**. Round 3: fixed-shape target with one output-set segment, so a reject on a job id is complete for any output set. Golden vectors and padded-output reject tests (1, 2 and 20 outputs). |
| R2-2 | low | Denial reason codes are warned to stderr at the default level; in stdio-once/ssh-stdio the client relays remote stderr to the sender, so reason classe… | **Fixed**. Round 3: denial reason codes are debug-only. A tracing-capture test shows nothing at warn or info on a serve-path denial. Docs state that debug logging under ssh-stdio is relayed to the peer. |
| R2-3 | low | Start check ties legacy bearer service to snapshot validity, runs late, and gives the operator no reason; docs still say bearer-only requests are 'unc… | **Documented**. A provisioned snapshot that is invalid or expired stops `remote serve`, bearer-only service included. Kept as fail-closed; no CLI added. |
| R2-4 | low | Test gaps: frame-level non-enforcing assertion is vacuous and its control weak; refusal cases cannot attribute the refusing check; key negatives, serv… | **Partly**. The frame-level test now asserts on the real ticket; each refusal asserts its `DenialReason`; seven missing negatives were added. Not added: concurrent presentations, ledger full or write failure, the verified-then-redemption-failed abort, `remote serve` start wiring, rollback-guard seeding. |
| R2-5 | low | The whole-request digest is an implicit serde struct-order contract with no golden vector, no external-signer spec, and an unzeroized request copy | **Fixed**. Golden vector for the whole-request digest, encoding and its fragility documented for external signers, encoded buffer zeroized with the existing `zeroize` dependency. |
| R2-6 | low | F2 residual is stated only in the status record: operator docs still say the caveats 'restrict one job' although payload, inputs, limits and registrat… | **Documented**. The label-scoped nature of a grant is stated in the operator docs. |
| R2-7 | low | Remaining overclaiming or stale sentences in docs, code comments and the core crate | **Documented**. Stale or overclaiming sentences were rewritten. |
| R2-8 | info | Operational consequences of the new target and digest (all fail-closed) | **Documented**. Production reaches only the attempt-shaped target; the 9-output cap is gone with the new target shape. |

## Independent validation by the integrator

- The integrator re-ran the full rail set on the branch tip after each round, in the agent's worktree: the
  counts equal the agent's (13, 2, 7, 5, 20, 8, 1, 1, 20, 2, 3, 9 tests across the twelve rails), `cargo fmt --check` and
  strict Clippy are clean for `mantle` and `crunch-store`. Exact output: `independent-rerun-2026-10-30.json`
  (BLAKE3 `4a81f02031367588c6c084d631d4c945a13b5a8879f84cf60182b49a5d929177`).
- The golden vectors (two output-set ids, one whole-request digest) were recomputed with a separate Python
  implementation of the documented encoding and `b3sum`; all match, which also shows the encoding is implementable
  outside Rust.
- Scope checks: no change to `tasks.md`, ADRs, specs, `Cargo.toml`/`Cargo.lock` or the flake; `zeroize` was already a
  root dependency.

## What this does not establish

- Any enforcement: see the headline. Nothing produces a presentation (no client or CLI), and the restricted store
  view has no production producer.
- The round-3 delta was **not** re-read by the security reviewer. It is covered by the re-run, my reading of
  `job_target` and `output_set_id`, and the recomputed vectors.
- Cairn validation of the branch alone, Tracey, Nix checks, workspace gates, concurrent presentations, ledger-full and
  ledger-write failure, and any real transport.

## Conduct note

The attenuation worker's first edits used paths relative to the canonical checkout and modified
`src/remote_build.rs` and `src/remote_build/authority_caveats.rs` there. It saved the diff, reverted it with
`git apply -R`, and ported the changes to its worktree. The integrator confirmed both files equal HEAD there and that
no other file in that checkout changed in the window; the saved patch held only the worker's own additions.
