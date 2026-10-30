#![no_std]
extern crate alloc;

#[path = "emitter.rs"]
mod bindings;
#[path = "../../guest_rt.rs"]
pub mod guest_rt;

use alloc::string::String;
use alloc::string::ToString;
use alloc::vec::Vec;

use bindings::wasi::clocks::wall_clock::Datetime;
use bindings::wasi::otel::tracing;
use bindings::wasi::otel::types::InstrumentationScope;
use bindings::wasi::otel::types::KeyValue;

struct Emitter;

// r[impl materialize_wrpc_telemetry.emitter]
impl bindings::Guest for Emitter {
    fn run(mode: u32) -> u32 {
        assert!(mode <= 2, "unsupported telemetry mode");
        let count = if mode == 2 { 33_u32 } else { 1_u32 };
        assert!(count > 0 && count <= 33);
        let mut span = span_data(mode, tracing::current_span_context());
        for index in 0..count {
            if index > 0 {
                span.span_context = tracing::current_span_context();
            }
            assert_eq!(span.span_context.trace_id.len(), 32);
            assert_eq!(span.span_context.span_id.len(), 16);
            tracing::on_start(&span.span_context);
            tracing::on_end(&span);
        }
        42
    }
}

fn span_data(mode: u32, context: tracing::SpanContext) -> tracing::SpanData {
    let attributes = if mode == 1 {
        alloc::vec![KeyValue { key: "authorization".to_string(), value: "\"secret\"".to_string() }]
    } else { Vec::new() };
    tracing::SpanData {
        parent_span_id: String::new(),
        span_context: context,
        span_kind: tracing::SpanKind::Client,
        name: "component-rpc-call".to_string(),
        start_time: Datetime { seconds: 0, nanoseconds: 0 },
        end_time: Datetime { seconds: 0, nanoseconds: 0 },
        attributes,
        events: Vec::new(),
        links: Vec::new(),
        status: tracing::Status::Unset,
        instrumentation_scope: InstrumentationScope {
            name: "molten-component-rpc".to_string(),
            version: None,
            schema_url: None,
            attributes: Vec::new(),
        },
        dropped_attributes: 0,
        dropped_events: 0,
        dropped_links: 0,
    }
}

bindings::export!(Emitter with_types_in bindings);
