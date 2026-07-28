// Mantle durable-file-publication focused Tracey bridge.
//
// r[impl mantle.durable_file_publication.source]
// r[impl mantle.durable_file_publication.mapping]
// r[impl mantle.durable_file_publication.outcomes]
// r[impl mantle.durable_file_publication.adapter]
// r[impl mantle.durable_file_publication.authority]
// r[impl mantle.durable_file_publication.validation]
// r[impl mantle.durable_file_publication.evidence]
// r[verify mantle.durable_file_publication.source]
// r[verify mantle.durable_file_publication.mapping]
// r[verify mantle.durable_file_publication.outcomes]
// r[verify mantle.durable_file_publication.adapter]
// r[verify mantle.durable_file_publication.authority]
// r[verify mantle.durable_file_publication.validation]
// r[verify mantle.durable_file_publication.evidence]
//
// The production adapter and Linux integration tests live in
// `src/remote_attempt_log_store.rs`. Cargo/Nix identity checks, typed Nickel
// evidence, and negative source fixtures cover immutable-source admission.
// Mantle retains manifests, content equivalence, retention, deletion, receipts,
// diagnostics, and explicit rollback. The shared crate grants no product
// authority and does not publish release-bundle directories.
