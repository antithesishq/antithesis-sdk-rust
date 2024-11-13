use std::collections::BTreeMap;

use serde::{ser::SerializeMap, Serialize, Serializer};
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
        #[derive(Serialize)]
        #[serde(bound = "R: Subscriber + for<'l> LookupSpan<'l>")]
        #[serde(rename_all = "snake_case")]
        enum SerializeEvent<'a, 'b, R> {
            TracingEvent {
                level: &'a str,
                target: &'a str,
                spans: SerializeScope<'a, 'b, R>,
                fields: SerializeValues<'a, 'b>,
            }
        }
        let meta = event.metadata();
        let level = meta.level().as_str();
        let target = meta.target();
        let spans = SerializeScope(ctx, event);
        let fields = SerializeValues(event);
        crate::internal::dispatch_output(&SerializeEvent::TracingEvent { level, target, fields, spans });

        struct SerializeValues<'a, 'b>(&'a tracing_core::Event<'b>);
        impl<'a, 'b> Serialize for SerializeValues<'a, 'b> {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where S: Serializer {
                let mut serializer = serializer.serialize_map(None)?;
                self.0.record(&mut SerializeVisitor(&mut serializer));
                serializer.end()
            }
        }

        struct SerializeVisitor<'a, S>(&'a mut S);
        impl<'a, S: SerializeMap> field::Visit for SerializeVisitor<'a, S> {
            fn record_f64(&mut self, field: &field::Field, value: f64) {
                let _ = self.0.serialize_entry(field.name(), &value);
            }

            fn record_i64(&mut self, field: &field::Field, value: i64) {
                let _ = self.0.serialize_entry(field.name(), &value);
            }

            fn record_u64(&mut self, field: &field::Field, value: u64) {
                let _ = self.0.serialize_entry(field.name(), &value);
            }

            fn record_i128(&mut self, field: &field::Field, value: i128) {
                let _ = self.0.serialize_entry(field.name(), &value);
            }

            fn record_u128(&mut self, field: &field::Field, value: u128) {
                let _ = self.0.serialize_entry(field.name(), &value);
            }

            fn record_bool(&mut self, field: &field::Field, value: bool) {
                let _ = self.0.serialize_entry(field.name(), &value);
            }

            fn record_str(&mut self, field: &field::Field, value: &str) {
                let _ = self.0.serialize_entry(field.name(), &value);
            }

            fn record_debug(&mut self, field: &field::Field, value: &dyn std::fmt::Debug) {
                let _ = self.0.serialize_entry(field.name(), &format!("{:?}", value));
            }
        }

        struct SerializeScope<'a, 'b, R>(Context<'a, R>, &'a tracing_core::Event<'b>);
        impl<'a, 'b, R> Serialize for SerializeScope<'a, 'b, R>
        where for<'l> R: Subscriber + LookupSpan<'l> {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where S: Serializer {
                use serde::ser::SerializeSeq;
                let SerializeScope(ctx, event) = self;
                let mut serializer = serializer.serialize_seq(None)?;
                if let Some(scope) = ctx.event_scope(event) {
                    for span in scope.from_root() {
                        if let Some(SpanValues(values)) = span.extensions().get() {
                            serializer.serialize_element(values)?;
                        }
                    }
                }
                serializer.end()
            }
        }
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
