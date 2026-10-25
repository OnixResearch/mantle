# Failed remote builds report their execution failure

A production remote build whose executor failed reached its client as a
transport error (`reading remote frame header: failed to fill whole buffer`).
The worker's effect session closed on the failed executor launch and refused to
send the worker's own error frame. The worker now sends exactly one terminal
failure report (ADR 0131), and the client reports
`remote-worker-execution-failed`.

The message carries the worker's failure-debug reference (`worker_bundle_ref`,
`worker_capture_outcome`, `worker_cleanup_status`), so `mantle remote status`
links the failure to the worker's debug bundle. The executor's own failure text
stays on the worker; read build detail from that bundle with
`mantle remote debug inspect`.

The client records the worker's signed failed-usage report only after
authenticating it. Otherwise the usage stays unrecorded and the message ends
with `failed-usage-unrecorded=<reason>`; the build still fails as an execution
failure.
