use std::collections::BTreeMap;

use serde::Serialize;
use serde_json::Value;
use tracing_core::{field, span, Event, Subscriber};
use tracing_subscriber::{layer::Context, registry::LookupSpan, Layer};

pub struct AntithesisLayer;

struct SpanValues(BTreeMap<&'static str, Value>);

// TODO: Also emit events at span creation/enter/leave/drop times, or allow configuration
// options to make that possible?
impl<S> Layer<S> for AntithesisLayer
where S: Subscriber + for<'a> LookupSpan<'a> {
    fn on_new_span(&self, attrs: &span::Attributes<'_>, id: &span::Id, ctx: Context<'_, S>) {
        let span = ctx.span(id).expect("Span not found");
        let mut values = BTreeMap::new();
        attrs.record(&mut JsonVisitor::with(&mut values));
        span.extensions_mut().insert(SpanValues(values));
    }

    fn on_record(&self, span: &span::Id, record: &span::Record<'_>, ctx: Context<'_, S>) {
        let span = ctx.span(span).expect("Span not found");
        let mut extensions = span.extensions_mut();
        if let Some(SpanValues(values)) = extensions.get_mut() {
            record.record(&mut JsonVisitor::with(values));
        } else {
            let mut values = BTreeMap::new();
            record.record(&mut JsonVisitor::with(&mut values));
            extensions.insert(SpanValues(values));
        }
    }

    fn on_event(&self, event: &Event<'_>, ctx: Context<'_, S>) {
        // TODO: Use streaming JSON serialization to eliminate intermediate data structures
        // and cloning? Might complicates implementation.
        let mut values = BTreeMap::new();
        event.record(&mut JsonVisitor::with(&mut values));
        let spans = ctx.event_scope(event).map(|scope| {
            scope.from_root().filter_map(|span| {
                let extensions = span.extensions();
                let SpanValues(values) = extensions.get()?;
                Some(values.clone())
            }).collect::<Vec<_>>()
        });

        #[derive(Serialize)]
        #[serde(rename_all = "snake_case")]
        enum Event {
            TracingEvent {
                spans: Option<Vec<BTreeMap<&'static str, Value>>>,
                #[serde(flatten)]
                values: BTreeMap<&'static str, Value>,
            }
        }
        crate::internal::dispatch_output(&Event::TracingEvent { values, spans });
    }
}

struct JsonVisitor<'a> {
    values: &'a mut BTreeMap<&'static str, Value>,
}

impl<'a> JsonVisitor<'a> {
    fn with(values: &'a mut BTreeMap<&'static str, Value>) -> Self {
        Self { values }
    }
}

impl<'a> field::Visit for JsonVisitor<'a> {
    fn record_f64(&mut self, field: &field::Field, value: f64) {
        self.values.insert(field.name(), Value::from(value));
    }

    fn record_i64(&mut self, field: &field::Field, value: i64) {
        self.values.insert(field.name(), Value::from(value));
    }

    fn record_u64(&mut self, field: &field::Field, value: u64) {
        self.values.insert(field.name(), Value::from(value));
    }

    fn record_bool(&mut self, field: &field::Field, value: bool) {
        self.values.insert(field.name(), Value::from(value));
    }

    fn record_str(&mut self, field: &field::Field, value: &str) {
        self.values.insert(field.name(), Value::from(value));
    }

    fn record_debug(&mut self, field: &field::Field, value: &dyn std::fmt::Debug) {
        self.values.insert(field.name(), Value::from(format!("{:?}", value)));
    }
}
