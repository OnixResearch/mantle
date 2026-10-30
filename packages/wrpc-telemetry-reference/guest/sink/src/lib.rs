//! Source-built comparison sink, never a trusted consumer host or exporter.
#![no_std]
extern crate alloc;

#[path = "sink.rs"]
mod bindings;
#[path = "../../guest_rt.rs"]
pub mod guest_rt;

use alloc::string::ToString;
use alloc::vec::Vec;
use core::sync::atomic::AtomicBool;
use core::sync::atomic::Ordering;

use bindings::exports::wasi::otel::tracing::SpanContext;
use bindings::exports::wasi::otel::tracing::SpanData;
use bindings::exports::wasi::otel::tracing::SpanKind;
use bindings::exports::wasi::otel::tracing::Status;
use bindings::exports::wasi::otel::tracing::TraceFlags;

static ACTIVE: AtomicBool = AtomicBool::new(false);
struct Sink;

impl bindings::exports::wasi::otel::tracing::Guest for Sink {
    fn current_span_context() -> SpanContext {
        SpanContext {
            trace_id: "11111111111111111111111111111111".to_string(),
            span_id: "2222222222222222".to_string(),
            trace_flags: TraceFlags::SAMPLED,
            is_remote: false,
            trace_state: Vec::new(),
        }
    }

    fn on_start(context: SpanContext) {
        assert_eq!(context.trace_id, "11111111111111111111111111111111");
        assert_eq!(context.span_id, "2222222222222222");
        assert!(!ACTIVE.swap(true, Ordering::Relaxed));
    }

    fn on_end(span: SpanData) {
        assert!(ACTIVE.swap(false, Ordering::Relaxed));
        assert_eq!(span.name, "component-rpc-call");
        assert_eq!(span.instrumentation_scope.name, "molten-component-rpc");
        assert!(span.parent_span_id.is_empty());
        assert!(matches!(span.span_kind, SpanKind::Client));
        assert!(span.attributes.is_empty());
        assert!(span.instrumentation_scope.attributes.is_empty());
        assert!(span.events.is_empty());
        assert!(span.links.is_empty());
        assert!(matches!(span.status, Status::Unset));
    }
}

bindings::export!(Sink with_types_in bindings);
