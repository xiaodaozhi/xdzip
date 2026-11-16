use serde::de::{self, DeserializeSeed, MapAccess, SeqAccess, Visitor};
use std::{cell::Cell, collections::BTreeSet, fmt};
use xdzip_codec_api::{Cancellation, ScanError};
const MAX_NODES: usize = 600_000;
const MAX_TEXT: usize = 16 * 1024 * 1024;
const MAX_DEPTH: usize = 136;
/// JSON values admitted before ASAR-specific field interpretation.
#[derive(Debug, PartialEq)]
pub enum Value {
    /// Object members retain physical serialization order.
    Object(Vec<(String, Value)>),
    /// Bounded array.
    Array(Vec<Value>),
    /// Strictly decoded Unicode text.
    String(String),
    /// Nonnegative integral value; individual fields impose tighter bounds.
    Integer(u64),
    /// Boolean.
    Bool(bool),
    /// Explicit null, distinct from an absent member.
    Null,
}
struct Context<'a> {
    cancel: &'a Cancellation,
    nodes: Cell<usize>,
    text: Cell<usize>,
    failure: Cell<Option<ScanError>>,
}
impl Context<'_> {
    fn fail<E: de::Error>(&self, error: ScanError) -> E {
        self.failure.set(Some(error));
        E::custom(error)
    }
    fn check<E: de::Error>(&self) -> Result<(), E> {
        self.cancel.check().map_err(|error| self.fail(error))
    }
    fn text<E: de::Error>(&self, count: usize) -> Result<(), E> {
        self.check()?;
        let remaining = self
            .text
            .get()
            .checked_sub(count)
            .ok_or_else(|| self.fail(ScanError::Limit))?;
        self.text.set(remaining);
        Ok(())
    }
}
#[derive(Clone, Copy)]
struct Seed<'a> {
    context: &'a Context<'a>,
    depth: usize,
}
impl<'de> DeserializeSeed<'de> for Seed<'_> {
    type Value = Value;
    fn deserialize<D: de::Deserializer<'de>>(self, deserializer: D) -> Result<Value, D::Error> {
        self.context.check()?;
        if self.depth > MAX_DEPTH {
            return Err(self.context.fail(ScanError::Limit));
        }
        let nodes = self
            .context
            .nodes
            .get()
            .checked_sub(1)
            .ok_or_else(|| self.context.fail(ScanError::Limit))?;
        self.context.nodes.set(nodes);
        deserializer.deserialize_any(self)
    }
}
impl<'de> Visitor<'de> for Seed<'_> {
    type Value = Value;
    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("bounded ASAR JSON")
    }
    fn visit_map<M: MapAccess<'de>>(self, mut map: M) -> Result<Value, M::Error> {
        let mut keys = BTreeSet::new();
        let mut values = Vec::new();
        while let Some(key) = map.next_key::<String>()? {
            self.context.text(key.len())?;
            if !keys.insert(key.clone()) {
                return Err(self.context.fail(ScanError::Invalid));
            }
            let value = map.next_value_seed(Seed {
                depth: self.depth + 1,
                ..self
            })?;
            values.push((key, value));
        }
        Ok(Value::Object(values))
    }
    fn visit_seq<S: SeqAccess<'de>>(self, mut array: S) -> Result<Value, S::Error> {
        let mut values = Vec::new();
        while let Some(value) = array.next_element_seed(Seed {
            depth: self.depth + 1,
            ..self
        })? {
            values.push(value);
        }
        Ok(Value::Array(values))
    }
    fn visit_string<E: de::Error>(self, text: String) -> Result<Value, E> {
        self.context.text(text.len())?;
        Ok(Value::String(text))
    }
    fn visit_str<E: de::Error>(self, text: &str) -> Result<Value, E> {
        self.context.text(text.len())?;
        Ok(Value::String(text.to_owned()))
    }
    fn visit_u64<E: de::Error>(self, value: u64) -> Result<Value, E> {
        Ok(Value::Integer(value))
    }
    fn visit_i64<E: de::Error>(self, value: i64) -> Result<Value, E> {
        u64::try_from(value)
            .map(Value::Integer)
            .map_err(|_| self.context.fail(ScanError::Invalid))
    }
    fn visit_f64<E: de::Error>(self, value: f64) -> Result<Value, E> {
        if !(0.0..=9_007_199_254_740_991.0).contains(&value) || value.fract() != 0.0 {
            return Err(self.context.fail(ScanError::Invalid));
        }
        Ok(Value::Integer(value as u64))
    }
    fn visit_bool<E: de::Error>(self, value: bool) -> Result<Value, E> {
        Ok(Value::Bool(value))
    }
    fn visit_unit<E: de::Error>(self) -> Result<Value, E> {
        Ok(Value::Null)
    }
}
/// Parse one complete bounded header, preserving order and rejecting duplicate keys.
/// No input-derived path is opened and no implicit recursion limit is relaxed
/// without the explicit node, text and depth admission above.
pub fn parse(bytes: &[u8], cancel: &Cancellation) -> Result<Value, ScanError> {
    parse_limited(bytes, cancel, MAX_NODES, MAX_TEXT)
}
fn parse_limited(
    bytes: &[u8],
    cancel: &Cancellation,
    nodes: usize,
    text: usize,
) -> Result<Value, ScanError> {
    if bytes.len() > 8 * 1024 * 1024 {
        return Err(ScanError::Limit);
    }
    let context = Context {
        cancel,
        nodes: Cell::new(nodes),
        text: Cell::new(text),
        failure: Cell::new(None),
    };
    let mut deserializer = serde_json::Deserializer::from_slice(bytes);
    deserializer.disable_recursion_limit();
    let value = Seed {
        context: &context,
        depth: 0,
    }
    .deserialize(&mut deserializer)
    .and_then(|value| {
        deserializer.end()?;
        Ok(value)
    });
    cancel.check()?;
    value.map_err(|_| context.failure.get().unwrap_or(ScanError::Invalid))
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn physical_order_decoded_keys_and_general_json_do_not_hide_duplicate_fields() {
        let cancel = Cancellation::default();
        assert_eq!(
            parse(
                br#"{"z":0,"a":[true,null,1e2],"x":"\u8d44\u6599"}"#,
                &cancel
            )
            .unwrap(),
            Value::Object(vec![
                ("z".into(), Value::Integer(0)),
                (
                    "a".into(),
                    Value::Array(vec![Value::Bool(true), Value::Null, Value::Integer(100)])
                ),
                ("x".into(), Value::String("资料".into()))
            ])
        );
        for bytes in [
            br#"{"files":{},"files":{}}"#.as_slice(),
            br#"{"x":1,"\u0078":2}"#,
            br#"{"a":{"size":0,"size":1}}"#,
            b"{} {}",
            b"[0.5]",
            b"[-1]",
            b"[1e999]",
            b"\"\xed\xa0\x80\"",
            br#""\ud800""#,
        ] {
            assert_eq!(parse(bytes, &cancel), Err(ScanError::Invalid));
        }
    }
    #[test]
    fn exact_node_text_and_explicit_depth_limits_and_cancellation_are_enforced() {
        let cancel = Cancellation::default();
        let bytes = br#"{"x":["abc",0]}"#;
        assert!(parse_limited(bytes, &cancel, 4, 4).is_ok());
        assert_eq!(parse_limited(bytes, &cancel, 3, 4), Err(ScanError::Limit));
        assert_eq!(parse_limited(bytes, &cancel, 4, 3), Err(ScanError::Limit));
        let nested = format!("{}0{}", "[".repeat(MAX_DEPTH), "]".repeat(MAX_DEPTH));
        assert!(parse(nested.as_bytes(), &cancel).is_ok());
        let nested = format!("[{nested}]");
        assert_eq!(parse(nested.as_bytes(), &cancel), Err(ScanError::Limit));
        cancel.cancel();
        assert_eq!(parse(b"{}", &cancel), Err(ScanError::Cancelled));
    }
}
