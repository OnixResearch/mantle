## Phase 1: Evidence-backed Partial Promotion

- [x] [serial] Replace the explicit `placeholder` marker in `bootstrap/binutils-tcc.ncl` without changing the bounded omitted-member behavior. ✅ 4m (started: 2026-05-12T19:43:00Z → completed: 2026-05-12T19:47:00Z)
- [x] [depends:marker-removal] Add a targeted parity regression that `binutils.tcc` reports `partial` with the checked transcript and remains a parity blocker. ✅ 5m (started: 2026-05-12T19:47:00Z → completed: 2026-05-12T19:52:00Z)
- [x] [depends:regression] Run targeted parity tests, parity-report JSON, and whitespace checks. ✅ 5m (started: 2026-05-12T19:52:00Z → completed: 2026-05-12T19:57:00Z; `cargo fmt --check`; `cargo test --bin crunch bootstrap_parity::tests -- --nocapture`; parity JSON shows `binutils.tcc` status `partial`; `git diff --check`)
- [x] [depends:verification] Sync/archive this OpenSpec change after verification. ✅ 1m (started: 2026-05-12T19:57:00Z → completed: 2026-05-12T19:58:00Z)
