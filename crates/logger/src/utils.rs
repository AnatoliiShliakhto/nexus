use fxhash::FxHashSet;
use serde::Serialize;
use smallvec::SmallVec;
use smol_str::SmolStr;
use std::sync::Arc;
use tracing::{Subscriber, field::Visit};
use tracing_subscriber::Layer;
use tracing_subscriber::field::MakeVisitor;
use tracing_subscriber::fmt::format::{DefaultFields, Writer};
use tracing_subscriber::fmt::{FmtContext, FormatEvent, FormatFields};
use tracing_subscriber::layer::Context;
use tracing_subscriber::registry::LookupSpan;

// --- Models ---

#[derive(Debug, Clone)]
pub(crate) enum FieldValue {
    Static(&'static str),
    String(SmolStr),
    I64(i64),
    U64(u64),
    F64(f64),
    Bool(bool),
}

impl Serialize for FieldValue {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            Self::Static(v) => serializer.serialize_str(v),
            Self::String(v) => serializer.serialize_str(v.as_str()),
            Self::I64(v) => serializer.serialize_i64(*v),
            Self::U64(v) => serializer.serialize_u64(*v),
            Self::F64(v) => serializer.serialize_f64(*v),
            Self::Bool(v) => serializer.serialize_bool(*v),
        }
    }
}

impl std::fmt::Display for FieldValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Static(v) => write!(f, "{v}"),
            Self::String(v) => write!(f, "{v}"),
            Self::I64(v) => write!(f, "{v}"),
            Self::U64(v) => write!(f, "{v}"),
            Self::F64(v) => write!(f, "{v}"),
            Self::Bool(v) => write!(f, "{v}"),
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct IgnoreFields {
    names: Arc<FxHashSet<String>>,
}

impl IgnoreFields {
    #[inline]
    pub(crate) fn contains(&self, field_name: &str) -> bool {
        self.names.contains(field_name)
    }

    pub(crate) fn from_env() -> Self {
        let names = std::env::var("LOG_IGNORE_FIELDS")
            .unwrap_or_else(|_| "password,token,secret,authorization".to_owned())
            .split(',')
            .map(|s| s.trim().to_lowercase())
            .collect::<FxHashSet<_>>();
        Self { names: Arc::new(names) }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct SpanAttributes {
    pub(crate) attrs: SmallVec<(&'static str, FieldValue), 16>,
}

// --- Field Visitor ---

pub(crate) struct FieldVisitor<'a> {
    pub(crate) attrs: &'a mut SmallVec<(&'static str, FieldValue), 16>,
    pub(crate) ignored: &'a IgnoreFields,
}

impl FieldVisitor<'_> {
    fn insert(&mut self, key: &'static str, value: FieldValue) {
        if self.ignored.contains(key) {
            return;
        }
        if let Some(existing) = self.attrs.iter_mut().find(|(k, _)| *k == key) {
            existing.1 = value;
        } else {
            self.attrs.push((key, value));
        }
    }
}

impl Visit for FieldVisitor<'_> {
    fn record_f64(&mut self, field: &tracing::field::Field, value: f64) {
        self.insert(field.name(), FieldValue::F64(value));
    }
    fn record_i64(&mut self, field: &tracing::field::Field, value: i64) {
        self.insert(field.name(), FieldValue::I64(value));
    }
    fn record_u64(&mut self, field: &tracing::field::Field, value: u64) {
        self.insert(field.name(), FieldValue::U64(value));
    }
    fn record_bool(&mut self, field: &tracing::field::Field, value: bool) {
        self.insert(field.name(), FieldValue::Bool(value));
    }
    fn record_str(&mut self, field: &tracing::field::Field, value: &str) {
        self.insert(field.name(), FieldValue::String(SmolStr::new(value)));
    }
    fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn std::fmt::Debug) {
        self.insert(field.name(), FieldValue::String(SmolStr::new(format!("{value:?}"))));
    }
}

// --- Span Collector ---

pub(crate) struct SpanFieldCollector {
    pub(crate) ignored: IgnoreFields,
}

impl<S> Layer<S> for SpanFieldCollector
where
    S: Subscriber + for<'lookup> LookupSpan<'lookup>,
{
    fn on_new_span(
        &self,
        attrs: &tracing::span::Attributes<'_>,
        id: &tracing::span::Id,
        ctx: Context<'_, S>,
    ) {
        let span = ctx.span(id).expect("Span not found");
        let mut fields = SmallVec::new();
        attrs.record(&mut FieldVisitor { attrs: &mut fields, ignored: &self.ignored });
        span.extensions_mut().insert(SpanAttributes { attrs: fields });
    }

    fn on_record(
        &self,
        id: &tracing::span::Id,
        values: &tracing::span::Record<'_>,
        ctx: Context<'_, S>,
    ) {
        let span = ctx.span(id).expect("Span not found");
        let mut ext = span.extensions_mut();
        if let Some(span_attrs) = ext.get_mut::<SpanAttributes>() {
            values
                .record(&mut FieldVisitor { attrs: &mut span_attrs.attrs, ignored: &self.ignored });
        }
    }
}

// --- JSON Formatter ---

pub(crate) struct JsonFormatEvent {
    ignored: IgnoreFields,
    spans: bool,
}

impl JsonFormatEvent {
    pub(crate) const fn new(ignored: IgnoreFields, spans: bool) -> Self {
        Self { ignored, spans }
    }
}

impl<S, N> FormatEvent<S, N> for JsonFormatEvent
where
    S: Subscriber + for<'lookup> LookupSpan<'lookup>,
    N: for<'writer> FormatFields<'writer> + 'static,
{
    fn format_event(
        &self,
        ctx: &FmtContext<'_, S, N>,
        mut writer: Writer<'_>,
        event: &tracing::Event<'_>,
    ) -> std::fmt::Result {
        let metadata = event.metadata();
        let mut fields: SmallVec<(&'static str, FieldValue), 16> = SmallVec::new();
        event.record(&mut FieldVisitor { attrs: &mut fields, ignored: &self.ignored });

        writer.write_str("{")?;

        write_json_kv(&mut writer, "level", &FieldValue::Static(metadata.level().as_str()), true)?;
        write_json_kv(&mut writer, "target", &FieldValue::Static(metadata.target()), false)?;

        let mut has_message = false;
        for (k, v) in &fields {
            if *k == "message" {
                write_json_kv(&mut writer, "message", v, false)?;
                has_message = true;
            }
        }
        if !has_message {
            write_json_kv(&mut writer, "message", &FieldValue::Static(""), false)?;
        }

        for (k, v) in &fields {
            if *k != "message" {
                write_json_kv(&mut writer, k, v, false)?;
            }
        }

        if self.spans
            && let Some(scope) = ctx.event_scope()
        {
            let mut first_span = true;
            let mut write_spans_array = false;

            for span in scope.from_root() {
                if !write_spans_array {
                    writer.write_str(",\"spans\":[")?;
                    write_spans_array = true;
                }
                if !first_span {
                    writer.write_str(",")?;
                }
                first_span = false;
                write_escaped_json_str(&mut writer, span.name())?;
            }

            if write_spans_array {
                writer.write_str("]")?;
            }

            if let Some(scope) = ctx.event_scope() {
                for span in scope.from_root() {
                    if let Some(span_fields) = span.extensions().get::<SpanAttributes>() {
                        for (k, v) in &span_fields.attrs {
                            write_json_kv(&mut writer, k, v, false)?;
                        }
                    }
                }
            }
        }

        writer.write_str("}\n")
    }
}

fn write_escaped_json_str(writer: &mut impl std::fmt::Write, s: &str) -> std::fmt::Result {
    writer.write_char('"')?;
    for c in s.chars() {
        match c {
            '"' => writer.write_str("\\\"")?,
            '\\' => writer.write_str("\\\\")?,
            '\n' => writer.write_str("\\n")?,
            '\r' => writer.write_str("\\r")?,
            '\t' => writer.write_str("\\t")?,
            c if c.is_control() => write!(writer, "\\u{:04x}", c as u32)?,
            c => writer.write_char(c)?,
        }
    }
    writer.write_char('"')
}

#[inline]
fn write_json_kv(
    writer: &mut Writer<'_>,
    key: &str,
    value: &FieldValue,
    is_first: bool,
) -> std::fmt::Result {
    if !is_first {
        writer.write_char(',')?;
    }

    write_escaped_json_str(writer, key)?;
    writer.write_char(':')?;

    match value {
        FieldValue::Static(s) => write_escaped_json_str(writer, s)?,
        FieldValue::String(s) => write_escaped_json_str(writer, s.as_str())?,
        FieldValue::I64(i) => write!(writer, "{i}")?,
        FieldValue::U64(u) => write!(writer, "{u}")?,
        FieldValue::F64(f) => write!(writer, "{f}")?,
        FieldValue::Bool(b) => write!(writer, "{}", if *b { "true" } else { "false" })?,
    }

    Ok(())
}

// // --- Filtered Fields ---

#[derive(Debug, Clone)]
pub(crate) struct FilteredFields {
    ignored: IgnoreFields,
}

impl FilteredFields {
    pub(crate) const fn new(ignored: IgnoreFields) -> Self {
        Self { ignored }
    }
}

impl<'writer> FormatFields<'writer> for FilteredFields {
    fn format_fields<R>(&self, mut writer: Writer<'writer>, fields: R) -> std::fmt::Result
    where
        R: tracing_subscriber::field::RecordFields,
    {
        let visitor = DefaultFields::new().make_visitor(writer.by_ref());
        let mut filtering_visitor = FilteringVisitor { inner: visitor, ignored: &self.ignored };
        fields.record(&mut filtering_visitor);
        Ok(())
    }
}

macro_rules! forward_filtered {
    ($($method:ident($val_ty:ty)),*) => {
        $(
            fn $method(&mut self, field: &tracing::field::Field, value: $val_ty) {
                if !self.ignored.contains(field.name()) {
                    self.inner.$method(field, value);
                }
            }
        )*
    };
}

#[derive(Debug)]
struct FilteringVisitor<'a, V> {
    inner: V,
    ignored: &'a IgnoreFields,
}

impl<V: Visit> Visit for FilteringVisitor<'_, V> {
    forward_filtered!(
        record_str(&str),
        record_f64(f64),
        record_i64(i64),
        record_u64(u64),
        record_i128(i128),
        record_u128(u128),
        record_bool(bool),
        record_debug(&dyn std::fmt::Debug),
        record_error(&(dyn std::error::Error + 'static))
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    impl IgnoreFields {
        pub(crate) fn from_str(input: &str) -> Self {
            let names = input.split(',').map(|s| s.trim().to_lowercase()).collect::<FxHashSet<_>>();
            Self { names: Arc::new(names) }
        }
    }

    #[test]
    fn test_ignore_fields_logic() {
        let ignored = IgnoreFields::from_str("Password, SECRET , Auth_Token");

        assert!(ignored.contains("password"));
        assert!(ignored.contains("secret"));
        assert!(ignored.contains("auth_token"));
        assert!(!ignored.contains("user_id"));
    }

    #[test]
    fn test_json_escaping_manual() {
        let mut buf = String::new();
        write_escaped_json_str(&mut buf, "Hello \"World\"\n\\").unwrap();
        assert_eq!(buf, r#""Hello \"World\"\n\\""#);
    }

    proptest! {
        #[test]
        fn proptest_json_escaping_is_valid(s in "\\PC*") {
            let mut buf = String::new();
            write_escaped_json_str(&mut buf, &s).unwrap();

            let parsed: Result<String, _> = serde_json::from_str(&buf);

            assert!(parsed.is_ok(), "Failed to parse generated JSON: {buf}");
            assert_eq!(parsed.unwrap(), s);
        }
    }
}
