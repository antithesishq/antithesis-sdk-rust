use serde::Serialize;
#[cfg(feature = "full")]
use serde::{
    ser::{
        SerializeMap, SerializeSeq, SerializeStruct, SerializeStructVariant, SerializeTuple,
        SerializeTupleStruct, SerializeTupleVariant,
    },
    Serializer,
};
use serde_json::{Map, Value};
#[cfg(feature = "full")]
use std::{
    cell::Cell,
    fmt::Display,
    sync::atomic::{AtomicBool, Ordering},
};

/// The one thing the SDK asks of a details value, behind a vtable: the
/// assertion tracker, catalog, and output machinery then compile once, in
/// this crate, instead of being monomorphized into every caller for every
/// details type — which also keeps the tracker's hash-map probing (seeded
/// per process) out of callers' instrumented code.
pub trait Details {
    fn object(&self, context: &str) -> Option<Map<String, Value>>;
}

impl<T: Serialize + ?Sized> Details for T {
    #[cfg(feature = "full")]
    fn object(&self, context: &str) -> Option<Map<String, Value>> {
        object(self, context)
    }

    // Without `full` nothing is emitted, so details are never serialized.
    #[cfg(not(feature = "full"))]
    fn object(&self, _context: &str) -> Option<Map<String, Value>> {
        None
    }
}

/// `context` names the emission the details belong to — the assertion id,
/// the event name, or "setup_complete" — so the diagnostics emitted for the
/// details point at the offending call site.
#[cfg(feature = "full")]
pub(crate) fn object<T: Serialize + ?Sized>(
    details: &T,
    context: &str,
) -> Option<Map<String, Value>> {
    let diagnostics = Diagnostics::default();
    let value = inspect(details, &diagnostics);
    finish(value, &diagnostics, context)
}

#[cfg(feature = "full")]
#[doc(hidden)]
#[derive(Serialize)]
pub struct NumericOperands<'a, T> {
    pub left: &'a T,
    pub right: &'a T,
}

/// The details of a numeric or boolean guidance assertion: the caller's,
/// merged with the guidance keys. A `Details` rather than a `Serialize`, so
/// that diagnostics for the merged value are reported once.
#[cfg(feature = "full")]
pub struct WithGuidance<'a, T: ?Sized, F> {
    pub details: &'a T,
    pub guidance: F,
}

#[cfg(feature = "full")]
impl<T: Serialize + ?Sized, G: Serialize, F: Fn() -> G> Details for WithGuidance<'_, T, F> {
    fn object(&self, context: &str) -> Option<Map<String, Value>> {
        let diagnostics = Diagnostics::default();
        let merged = (|| {
            let generated = match inspect(&(self.guidance)(), &diagnostics)? {
                Value::Object(generated) => generated,
                _ => Map::new(),
            };
            let details = self.details.serialize(shadow::SkipShadowed {
                shadowed: &generated,
                diagnostics: &diagnostics,
            })?;
            let mut details = match details {
                Value::Object(object) => object,
                Value::Null => Map::new(),
                value => {
                    let mut object = Map::new();
                    object.insert("value".to_owned(), value);
                    object
                }
            };
            details.extend(generated);
            Ok(Value::Object(details))
        })();
        finish(merged, &diagnostics, context)
    }
}

/// Which of the details warnings an inspecting pass has found to apply.
#[cfg(feature = "full")]
#[derive(Default)]
struct Diagnostics {
    non_finite: Cell<bool>,
    beyond_safe_integer: Cell<bool>,
}

/// Serializes `details` to a value, replacing non-finite floats and noting
/// integers beyond ±2^53 on the way. This has to happen during
/// serialization: serde_json turns a non-finite float into null silently.
#[cfg(feature = "full")]
fn inspect<T: Serialize + ?Sized>(
    details: &T,
    diagnostics: &Diagnostics,
) -> serde_json::Result<Value> {
    details.serialize(Inspecting {
        inner: serde_json::value::Serializer,
        diagnostics,
    })
}

/// Emits the diagnostics the pass found — ahead of the packet the details
/// belong to, which the caller dispatches after this returns — and shapes
/// the details for the wire: objects as they are, null as no details,
/// anything else under "value".
#[cfg(feature = "full")]
fn finish(
    value: serde_json::Result<Value>,
    diagnostics: &Diagnostics,
    context: &str,
) -> Option<Map<String, Value>> {
    let value = finish_value(value, diagnostics, context)?;
    match value {
        Value::Object(object) => Some(object),
        Value::Null => None,
        value => {
            let mut object = Map::new();
            object.insert("value".to_owned(), value);
            Some(object)
        }
    }
}

/// Guidance uses the same inspecting pass without the details root
/// normalization. Public for the guidance macros: this is the one step of a
/// guidance emission that depends on the operand type, so it alone is
/// monomorphized into the caller; the packet is built and emitted by
/// `assert::guidance::guidance_impl` in this crate.
#[cfg(feature = "full")]
#[doc(hidden)]
pub fn guidance<T: Serialize + ?Sized>(data: &T, context: &str) -> Option<Value> {
    let diagnostics = Diagnostics::default();
    finish_value(inspect(data, &diagnostics), &diagnostics, context)
}

#[cfg(feature = "full")]
fn finish_value(
    value: serde_json::Result<Value>,
    diagnostics: &Diagnostics,
    context: &str,
) -> Option<Value> {
    // The context (an event name, assertion id, or "setup_complete") is
    // inserted verbatim, like every SDK's message: Debug formatting would
    // escape control characters in a Rust-only dialect.
    let value = match value {
        Ok(value) => value,
        // Bad diagnostic context must not suppress an assertion or lifecycle event.
        Err(error) => {
            report(
                "antithesis_error",
                6001,
                context,
                format!("Rust SDK failed to serialize details for \"{context}\": {error}"),
            );
            return None;
        }
    };
    // Each flag is read before it is swapped: once a warning is out, every
    // later emission that would repeat it must not pay a locked exchange.
    static REPORTED_NON_FINITE: AtomicBool = AtomicBool::new(false);
    if diagnostics.non_finite.get()
        && !REPORTED_NON_FINITE.load(Ordering::Relaxed)
        && !REPORTED_NON_FINITE.swap(true, Ordering::Relaxed)
    {
        report(
            "antithesis_warning",
            6002,
            context,
            format!(
                "Rust SDK replaced non-finite numbers in details for \"{context}\" \
                 with the strings \"NaN\", \"Infinity\" or \"-Infinity\"; \
                 later occurrences in this process are not reported"
            ),
        );
    }
    static REPORTED_UNSAFE_INTEGER: AtomicBool = AtomicBool::new(false);
    if diagnostics.beyond_safe_integer.get()
        && !REPORTED_UNSAFE_INTEGER.load(Ordering::Relaxed)
        && !REPORTED_UNSAFE_INTEGER.swap(true, Ordering::Relaxed)
    {
        report(
            "antithesis_warning",
            6003,
            context,
            format!(
                "Rust SDK emitted integers beyond ±2^53 in details for \"{context}\"; \
                 consumers that parse JSON numbers as doubles will round them; \
                 later occurrences in this process are not reported"
            ),
        );
    }
    Some(value)
}

/// Skipped guidance is never serialized, so unlike the details warnings this
/// one is claimed as it is written.
#[cfg(feature = "full")]
pub(crate) fn report_non_finite_guidance(id: &str) {
    static REPORTED_NON_FINITE_GUIDANCE: AtomicBool = AtomicBool::new(false);
    if !REPORTED_NON_FINITE_GUIDANCE.load(Ordering::Relaxed)
        && !REPORTED_NON_FINITE_GUIDANCE.swap(true, Ordering::Relaxed)
    {
        report(
            "antithesis_warning",
            6004,
            id,
            format!(
                "Rust SDK skipped numeric guidance for \"{id}\" because an operand was \
                 not finite; later occurrences in this process are not reported"
            ),
        );
    }
}

#[cfg(feature = "full")]
fn report(key: &str, code: u32, context: &str, message: String) {
    crate::internal::dispatch_output(&serde_json::json!({
        key: { "code": code, "message": message, "context": context },
    }));
}

/// A serializer forwarding to `S` that inspects the numbers passing through,
/// at any depth: a non-finite float is serialized as the string "NaN",
/// "Infinity" or "-Infinity" (JavaScript's spellings) and an integer of
/// magnitude 2^53 or more is noted. Only serde's integer types count: a
/// large integral float is already a double, and survives a consumer that
/// parses JSON numbers as doubles unchanged.
#[cfg(feature = "full")]
struct Inspecting<'d, S> {
    inner: S,
    diagnostics: &'d Diagnostics,
}

/// A value below the top of the details tree, serialized through `Inspecting`.
#[cfg(feature = "full")]
struct Nested<'a, T: ?Sized> {
    value: &'a T,
    diagnostics: &'a Diagnostics,
}

/// A sequence, map or struct in progress whose elements are `Nested`.
#[cfg(feature = "full")]
struct Compound<'d, C> {
    inner: C,
    diagnostics: &'d Diagnostics,
}

#[cfg(feature = "full")]
impl<T: Serialize + ?Sized> Serialize for Nested<'_, T> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.value.serialize(Inspecting {
            inner: serializer,
            diagnostics: self.diagnostics,
        })
    }
}

#[cfg(feature = "full")]
fn non_finite(value: f64) -> Option<&'static str> {
    if value.is_finite() {
        None
    } else if value.is_nan() {
        Some("NaN")
    } else if value > 0.0 {
        Some("Infinity")
    } else {
        Some("-Infinity")
    }
}

#[cfg(feature = "full")]
impl<S: Serializer> Inspecting<'_, S> {
    fn float(
        self,
        value: f64,
        finite: impl FnOnce(S) -> Result<S::Ok, S::Error>,
    ) -> Result<S::Ok, S::Error> {
        match non_finite(value) {
            Some(spelling) => {
                self.diagnostics.non_finite.set(true);
                self.inner.serialize_str(spelling)
            }
            None => finite(self.inner),
        }
    }

    fn integer(&self, magnitude: u128) {
        if magnitude >= 1 << 53 {
            self.diagnostics.beyond_safe_integer.set(true);
        }
    }
}

#[cfg(feature = "full")]
macro_rules! forward {
    ($($method:ident($($arg:ident: $ty:ty),*)),* $(,)?) => {$(
        fn $method(self, $($arg: $ty),*) -> Result<S::Ok, S::Error> {
            self.inner.$method($($arg),*)
        }
    )*};
}

#[cfg(feature = "full")]
macro_rules! forward_compound {
    ($($method:ident($($arg:ident: $ty:ty),*) -> $compound:ident),* $(,)?) => {$(
        fn $method(self, $($arg: $ty),*) -> Result<Self::$compound, S::Error> {
            let diagnostics = self.diagnostics;
            self.inner.$method($($arg),*).map(|inner| Compound { inner, diagnostics })
        }
    )*};
}

#[cfg(feature = "full")]
impl<'d, S: Serializer> Serializer for Inspecting<'d, S> {
    type Ok = S::Ok;
    type Error = S::Error;
    type SerializeSeq = Compound<'d, S::SerializeSeq>;
    type SerializeTuple = Compound<'d, S::SerializeTuple>;
    type SerializeTupleStruct = Compound<'d, S::SerializeTupleStruct>;
    type SerializeTupleVariant = Compound<'d, S::SerializeTupleVariant>;
    type SerializeMap = Compound<'d, S::SerializeMap>;
    type SerializeStruct = Compound<'d, S::SerializeStruct>;
    type SerializeStructVariant = Compound<'d, S::SerializeStructVariant>;

    fn serialize_f32(self, v: f32) -> Result<S::Ok, S::Error> {
        self.float(v.into(), |inner| inner.serialize_f32(v))
    }

    fn serialize_f64(self, v: f64) -> Result<S::Ok, S::Error> {
        self.float(v, |inner| inner.serialize_f64(v))
    }

    fn serialize_i64(self, v: i64) -> Result<S::Ok, S::Error> {
        self.integer(v.unsigned_abs().into());
        self.inner.serialize_i64(v)
    }

    fn serialize_i128(self, v: i128) -> Result<S::Ok, S::Error> {
        self.integer(v.unsigned_abs());
        self.inner.serialize_i128(v)
    }

    fn serialize_u64(self, v: u64) -> Result<S::Ok, S::Error> {
        self.integer(v.into());
        self.inner.serialize_u64(v)
    }

    fn serialize_u128(self, v: u128) -> Result<S::Ok, S::Error> {
        self.integer(v);
        self.inner.serialize_u128(v)
    }

    // The narrower integers cannot reach 2^53.
    forward! {
        serialize_bool(v: bool),
        serialize_i8(v: i8),
        serialize_i16(v: i16),
        serialize_i32(v: i32),
        serialize_u8(v: u8),
        serialize_u16(v: u16),
        serialize_u32(v: u32),
        serialize_char(v: char),
        serialize_str(v: &str),
        serialize_bytes(v: &[u8]),
        serialize_none(),
        serialize_unit(),
        serialize_unit_struct(name: &'static str),
        serialize_unit_variant(name: &'static str, index: u32, variant: &'static str),
    }

    fn collect_str<T: Display + ?Sized>(self, value: &T) -> Result<S::Ok, S::Error> {
        self.inner.collect_str(value)
    }

    fn is_human_readable(&self) -> bool {
        self.inner.is_human_readable()
    }

    fn serialize_some<T: Serialize + ?Sized>(self, value: &T) -> Result<S::Ok, S::Error> {
        let diagnostics = self.diagnostics;
        self.inner.serialize_some(&Nested { value, diagnostics })
    }

    fn serialize_newtype_struct<T: Serialize + ?Sized>(
        self,
        name: &'static str,
        value: &T,
    ) -> Result<S::Ok, S::Error> {
        let diagnostics = self.diagnostics;
        self.inner
            .serialize_newtype_struct(name, &Nested { value, diagnostics })
    }

    fn serialize_newtype_variant<T: Serialize + ?Sized>(
        self,
        name: &'static str,
        index: u32,
        variant: &'static str,
        value: &T,
    ) -> Result<S::Ok, S::Error> {
        let diagnostics = self.diagnostics;
        self.inner
            .serialize_newtype_variant(name, index, variant, &Nested { value, diagnostics })
    }

    forward_compound! {
        serialize_seq(len: Option<usize>) -> SerializeSeq,
        serialize_tuple(len: usize) -> SerializeTuple,
        serialize_tuple_struct(name: &'static str, len: usize) -> SerializeTupleStruct,
        serialize_tuple_variant(name: &'static str, index: u32, variant: &'static str, len: usize)
            -> SerializeTupleVariant,
        serialize_map(len: Option<usize>) -> SerializeMap,
        serialize_struct(name: &'static str, len: usize) -> SerializeStruct,
        serialize_struct_variant(name: &'static str, index: u32, variant: &'static str, len: usize)
            -> SerializeStructVariant,
    }
}

/// One impl per serde compound trait: the element method wraps each value
/// in `Nested`, `end` forwards, and anything further is given verbatim.
#[cfg(feature = "full")]
macro_rules! compound {
    ($($trait:ident { $method:ident($($arg:ident: $ty:ty),*) $($extra:tt)* })*) => {$(
        impl<C: $trait> $trait for Compound<'_, C> {
            type Ok = C::Ok;
            type Error = C::Error;

            fn $method<T: Serialize + ?Sized>(
                &mut self,
                $($arg: $ty,)*
                value: &T,
            ) -> Result<(), C::Error> {
                let diagnostics = self.diagnostics;
                self.inner.$method($($arg,)* &Nested { value, diagnostics })
            }

            fn end(self) -> Result<C::Ok, C::Error> {
                self.inner.end()
            }

            $($extra)*
        }
    )*};
}

#[cfg(feature = "full")]
compound! {
    SerializeSeq { serialize_element() }
    SerializeTuple { serialize_element() }
    SerializeTupleStruct { serialize_field() }
    SerializeTupleVariant { serialize_field() }
    SerializeMap {
        serialize_value()

        // Keys are strings on the wire whatever their type, so there is
        // nothing to inspect or replace in them.
        fn serialize_key<T: Serialize + ?Sized>(&mut self, key: &T) -> Result<(), C::Error> {
            self.inner.serialize_key(key)
        }
    }
    SerializeStruct {
        serialize_field(key: &'static str)

        fn skip_field(&mut self, key: &'static str) -> Result<(), C::Error> {
            self.inner.skip_field(key)
        }
    }
    SerializeStructVariant {
        serialize_field(key: &'static str)

        fn skip_field(&mut self, key: &'static str) -> Result<(), C::Error> {
            self.inner.skip_field(key)
        }
    }
}

/// Serializes a details value into a map with the entries the generated keys
/// would shadow **skipped**, so a caller value the merge overwrites is never
/// handed to a serializer at all.
#[cfg(feature = "full")]
mod shadow {
    use super::{inspect, Diagnostics, Inspecting};
    use serde::ser::{Serialize, SerializeMap, Serializer};
    use serde_json::{Map, Value};

    type Out = Map<String, Value>;
    type Failure = serde_json::Error;
    type Inner<'a> = Inspecting<'a, serde_json::value::Serializer>;

    pub(super) struct SkipShadowed<'a> {
        pub(super) shadowed: &'a Out,
        pub(super) diagnostics: &'a Diagnostics,
    }

    enum Pending {
        Absent,
        Keep(String),
        Skip,
    }

    pub(super) struct SkipShadowedMap<'a> {
        shadowed: &'a Out,
        diagnostics: &'a Diagnostics,
        out: Out,
        pending: Pending,
    }

    impl<'a> SkipShadowed<'a> {
        fn inner(self) -> Inner<'a> {
            Inspecting {
                inner: serde_json::value::Serializer,
                diagnostics: self.diagnostics,
            }
        }

        fn map(self) -> SkipShadowedMap<'a> {
            SkipShadowedMap {
                shadowed: self.shadowed,
                diagnostics: self.diagnostics,
                out: Out::new(),
                pending: Pending::Absent,
            }
        }
    }

    macro_rules! forward_value {
        ($($method:ident($($arg:ident: $ty:ty),*)),* $(,)?) => {$(
            fn $method(self, $($arg: $ty),*) -> Result<Value, Failure> {
                if self.shadowed.contains_key("value") {
                    Ok(Value::Object(Out::new()))
                } else {
                    self.inner().$method($($arg),*)
                }
            }
        )*};
    }

    impl<'a> Serializer for SkipShadowed<'a> {
        type Ok = Value;
        type Error = Failure;
        type SerializeSeq = Filtered<<Inner<'a> as Serializer>::SerializeSeq>;
        type SerializeTuple = Filtered<<Inner<'a> as Serializer>::SerializeTuple>;
        type SerializeTupleStruct = Filtered<<Inner<'a> as Serializer>::SerializeTupleStruct>;
        type SerializeTupleVariant = Filtered<<Inner<'a> as Serializer>::SerializeTupleVariant>;
        type SerializeMap = SkipShadowedMap<'a>;
        type SerializeStruct = SkipShadowedMap<'a>;
        type SerializeStructVariant = Filtered<<Inner<'a> as Serializer>::SerializeStructVariant>;

        forward_value! {
            serialize_bool(v: bool), serialize_i8(v: i8), serialize_i16(v: i16),
            serialize_i32(v: i32), serialize_i64(v: i64), serialize_i128(v: i128),
            serialize_u8(v: u8), serialize_u16(v: u16), serialize_u32(v: u32),
            serialize_u64(v: u64), serialize_u128(v: u128), serialize_f32(v: f32),
            serialize_f64(v: f64), serialize_char(v: char), serialize_str(v: &str),
            serialize_bytes(v: &[u8]), serialize_none(), serialize_unit(),
            serialize_unit_struct(name: &'static str),
            serialize_unit_variant(name: &'static str, index: u32, variant: &'static str),
        }

        fn collect_str<T: std::fmt::Display + ?Sized>(self, value: &T) -> Result<Value, Failure> {
            if self.shadowed.contains_key("value") {
                Ok(Value::Object(Out::new()))
            } else {
                self.inner().collect_str(value)
            }
        }

        fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap, Failure> {
            Ok(self.map())
        }

        fn serialize_struct(
            self,
            _name: &'static str,
            _len: usize,
        ) -> Result<Self::SerializeStruct, Failure> {
            Ok(self.map())
        }

        // These wrappers do not add a JSON level, so filtering still applies.
        fn serialize_some<T: Serialize + ?Sized>(self, value: &T) -> Result<Value, Failure> {
            value.serialize(self)
        }

        fn serialize_newtype_struct<T: Serialize + ?Sized>(
            self,
            _name: &'static str,
            value: &T,
        ) -> Result<Value, Failure> {
            value.serialize(self)
        }

        fn serialize_newtype_variant<T: Serialize + ?Sized>(
            self,
            name: &'static str,
            index: u32,
            variant: &'static str,
            value: &T,
        ) -> Result<Value, Failure> {
            if self.shadowed.contains_key(variant) {
                Ok(Value::Object(Out::new()))
            } else {
                self.inner()
                    .serialize_newtype_variant(name, index, variant, value)
            }
        }

        fn serialize_seq(self, len: Option<usize>) -> Result<Self::SerializeSeq, Failure> {
            Ok(Filtered {
                inner: if self.shadowed.contains_key("value") {
                    None
                } else {
                    Some(self.inner().serialize_seq(len)?)
                },
            })
        }
        fn serialize_tuple(self, len: usize) -> Result<Self::SerializeTuple, Failure> {
            Ok(Filtered {
                inner: if self.shadowed.contains_key("value") {
                    None
                } else {
                    Some(self.inner().serialize_tuple(len)?)
                },
            })
        }
        fn serialize_tuple_struct(
            self,
            name: &'static str,
            len: usize,
        ) -> Result<Self::SerializeTupleStruct, Failure> {
            Ok(Filtered {
                inner: if self.shadowed.contains_key("value") {
                    None
                } else {
                    Some(self.inner().serialize_tuple_struct(name, len)?)
                },
            })
        }
        fn serialize_tuple_variant(
            self,
            name: &'static str,
            index: u32,
            variant: &'static str,
            len: usize,
        ) -> Result<Self::SerializeTupleVariant, Failure> {
            Ok(Filtered {
                inner: if self.shadowed.contains_key(variant) {
                    None
                } else {
                    Some(
                        self.inner()
                            .serialize_tuple_variant(name, index, variant, len)?,
                    )
                },
            })
        }
        fn serialize_struct_variant(
            self,
            name: &'static str,
            index: u32,
            variant: &'static str,
            len: usize,
        ) -> Result<Self::SerializeStructVariant, Failure> {
            Ok(Filtered {
                inner: if self.shadowed.contains_key(variant) {
                    None
                } else {
                    Some(
                        self.inner()
                            .serialize_struct_variant(name, index, variant, len)?,
                    )
                },
            })
        }
    }

    // Sequences live under "value" and variants under their variant name.
    // When that key is overwritten, their fields must not be serialized.
    pub(super) struct Filtered<C> {
        inner: Option<C>,
    }

    macro_rules! filtered {
        ($trait:ident, $method:ident, $($arg:ident: $ty:ty),*) => {
            impl<C: serde::ser::$trait<Ok = Value, Error = Failure>> serde::ser::$trait for Filtered<C> {
                type Ok = Value;
                type Error = Failure;
                fn $method<T: Serialize + ?Sized>(&mut self, $($arg: $ty,)* value: &T) -> Result<(), Failure> {
                    match &mut self.inner {
                        Some(inner) => inner.$method($($arg,)* value),
                        None => Ok(()),
                    }
                }
                fn end(self) -> Result<Value, Failure> {
                    match self.inner {
                        Some(inner) => inner.end(),
                        None => Ok(Value::Object(Out::new())),
                    }
                }
            }
        };
    }
    filtered!(SerializeSeq, serialize_element,);
    filtered!(SerializeTuple, serialize_element,);
    filtered!(SerializeTupleStruct, serialize_field,);
    filtered!(SerializeTupleVariant, serialize_field,);
    filtered!(SerializeStructVariant, serialize_field, key: &'static str);

    impl SerializeMap for SkipShadowedMap<'_> {
        type Ok = Value;
        type Error = Failure;

        fn serialize_key<T: Serialize + ?Sized>(&mut self, key: &T) -> Result<(), Failure> {
            // Delegate key conversion to serde_json: map keys include numbers,
            // booleans and newtypes, not just values serializing as strings.
            let mut map = serde_json::value::Serializer.serialize_map(Some(1))?;
            map.serialize_entry(key, &())?;
            let key = match map.end()? {
                Value::Object(map) => map.into_iter().next().unwrap().0,
                _ => unreachable!(),
            };
            self.pending = if self.shadowed.contains_key(&key) {
                Pending::Skip
            } else {
                Pending::Keep(key)
            };
            Ok(())
        }

        fn serialize_value<T: Serialize + ?Sized>(&mut self, value: &T) -> Result<(), Failure> {
            match std::mem::replace(&mut self.pending, Pending::Absent) {
                Pending::Skip => Ok(()),
                Pending::Keep(key) => {
                    self.out.insert(key, inspect(value, self.diagnostics)?);
                    Ok(())
                }
                Pending::Absent => Err(serde::ser::Error::custom("details value without a key")),
            }
        }

        fn end(self) -> Result<Value, Failure> {
            Ok(Value::Object(self.out))
        }
    }

    impl serde::ser::SerializeStruct for SkipShadowedMap<'_> {
        type Ok = Value;
        type Error = Failure;

        fn serialize_field<T: Serialize + ?Sized>(
            &mut self,
            key: &'static str,
            value: &T,
        ) -> Result<(), Failure> {
            if !self.shadowed.contains_key(key) {
                self.out
                    .insert(key.to_owned(), inspect(value, self.diagnostics)?);
            }
            Ok(())
        }

        fn end(self) -> Result<Value, Failure> {
            Ok(Value::Object(self.out))
        }
    }
}
