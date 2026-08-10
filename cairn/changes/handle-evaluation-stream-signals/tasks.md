## Phase 1: Baseline and policy

- [x] [serial] I1 Record the current default-signal behavior and rerun the focused stream CLI baseline. r[evaluation_streaming.signal_cancellation]
- [x] [serial] I2 Add positive and negative pure tests for first and repeated signal observations. r[evaluation_streaming.signal_cancellation]

## Phase 2: Stream shell

- [x] [serial] I3 Add mode-local `SIGINT` and `SIGTERM` listeners with a portable Ctrl-C fallback. r[evaluation_streaming.signal_cancellation]
- [x] [serial] I4 Connect the first signal to `EvaluationCancellation` while continuing bounded record draining. r[evaluation_streaming.signal_cancellation]
- [x] [serial] I5 Make repeated interruption fail closed with a non-success status. r[evaluation_streaming.signal_cancellation]

## Phase 3: Validation and completion

- [x] [serial] I6 Add bounded subprocess tests for `SIGINT`, `SIGTERM`, repeated interruption, and the final cancelled summary. r[evaluation_streaming.signal_cancellation]
- [x] [serial] I7 Document signal, summary, process-status, and non-claim behavior. r[evaluation_streaming.signal_cancellation]
- [ ] [serial] V1 Run focused tests, formatting, Clippy, stream-contract checks, Cairn validation and gates, and Tracey coverage. r[evaluation_streaming.signal_cancellation]
- [ ] [serial] V2 Sync the accepted requirement, archive the completed change with evidence, and rerun post-archive validation. r[evaluation_streaming.signal_cancellation]
