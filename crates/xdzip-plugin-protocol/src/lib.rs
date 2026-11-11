//! Bounded SDK v1 CBOR framing. This crate has no filesystem, process or Qt authority.
use std::collections::BTreeMap;
use std::io::{Read, Write};

/// Largest encoded envelope, checked before allocation.
pub const MAX_FRAME: usize = 1024 * 1024;
/// Largest individual binary transfer block.
pub const MAX_BLOCK: usize = 256 * 1024;
/// Protocol major shared by the independently implemented peers.
pub const MAJOR: u64 = 1;

/// Stable protocol errors, without peer data or OS messages.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Error {
    /// A pipe closed or failed.
    Io,
    /// Framing, CBOR, schema or state was invalid.
    Invalid,
    /// An allocation, nesting or collection limit was exceeded.
    Limit,
}
impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::Io => "protocol_io",
            Self::Invalid => "protocol_invalid",
            Self::Limit => "protocol_limit",
        })
    }
}
impl std::error::Error for Error {}

/// The admitted CBOR subset excludes negative integers, tags, floats and indefinite values.
#[derive(Clone, Eq, PartialEq)]
pub enum Value {
    /// Unsigned integer.
    UInt(u64),
    /// Bounded opaque bytes.
    Bytes(Vec<u8>),
    /// Strict UTF-8 protocol text, never an output path capability.
    Text(String),
    /// At most 128 elements.
    Array(Vec<Value>),
    /// At most 32 unique unsigned field keys.
    Map(BTreeMap<u64, Value>),
    /// Boolean value.
    Bool(bool),
}
impl std::fmt::Debug for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UInt(v) => f.debug_tuple("UInt").field(v).finish(),
            Self::Bool(v) => f.debug_tuple("Bool").field(v).finish(),
            Self::Bytes(v) => f.debug_struct("Bytes").field("length", &v.len()).finish(),
            Self::Text(v) => f.debug_struct("Text").field("length", &v.len()).finish(),
            Self::Array(v) => f.debug_tuple("Array").field(v).finish(),
            Self::Map(v) => f.debug_tuple("Map").field(v).finish(),
        }
    }
}
impl Value {
    /// Construct an owned fixed-schema map.
    pub fn map(fields: impl IntoIterator<Item = (u64, Value)>) -> Self {
        Self::Map(fields.into_iter().collect())
    }
    /// Read one required map field.
    pub fn field(&self, key: u64) -> Result<&Self, Error> {
        match self {
            Self::Map(m) => m.get(&key).ok_or(Error::Invalid),
            _ => Err(Error::Invalid),
        }
    }
    /// Borrow an unsigned field.
    pub fn uint(&self) -> Result<u64, Error> {
        match self {
            Self::UInt(n) => Ok(*n),
            _ => Err(Error::Invalid),
        }
    }
    /// Borrow an opaque binary field.
    pub fn bytes(&self) -> Result<&[u8], Error> {
        match self {
            Self::Bytes(b) => Ok(b),
            _ => Err(Error::Invalid),
        }
    }
    /// Borrow protocol text.
    pub fn text(&self) -> Result<&str, Error> {
        match self {
            Self::Text(s) => Ok(s),
            _ => Err(Error::Invalid),
        }
    }
    /// Borrow a bounded array.
    pub fn array(&self) -> Result<&[Self], Error> {
        match self {
            Self::Array(a) => Ok(a),
            _ => Err(Error::Invalid),
        }
    }
    /// Reject unknown required keys; keys >=128 are explicitly optional extensions.
    pub fn schema(&self, required: &[u64]) -> Result<(), Error> {
        let Self::Map(fields) = self else {
            return Err(Error::Invalid);
        };
        if required.iter().any(|key| !fields.contains_key(key))
            || fields
                .keys()
                .any(|key| *key < 128 && !required.contains(key))
        {
            return Err(Error::Invalid);
        }
        Ok(())
    }
}

/// Fixed v1 operation kinds. Unsupported optional capabilities remain unadvertised.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u64)]
pub enum Kind {
    /// Peer announces its protocol interval and identity.
    Hello = 1,
    /// Host freezes version and resource limits.
    Welcome = 2,
    /// Host rejects the handshake.
    Reject = 3,
    /// Bounded format detection.
    Probe = 10,
    /// Open one source capability.
    Open = 11,
    /// Enumerate complete metadata.
    Scan = 12,
    /// Read a selected entry into one approved sink capability.
    ReadEntry = 13,
    /// Close a session.
    Close = 14,
    /// Request one bounded source range.
    ReadAt = 20,
    /// Host reply for the requested range.
    ReadData = 21,
    /// Bounded metadata batch.
    EntryBatch = 30,
    /// Successful complete metadata scan.
    ScanFinished = 31,
    /// Sequential output block.
    WriteData = 40,
    /// Peer seals its output before the operation terminal.
    FinishOutput = 41,
    /// Refresh a single output credit after the host accepts a block.
    WriteAck = 42,
    /// Cancel and revoke outstanding output authority.
    Cancel = 50,
    /// Cancellation acknowledgement, not publication permission.
    CancelAck = 51,
    /// End the private peer process.
    Shutdown = 52,
    /// Bounded progress, not a terminal.
    Progress = 60,
    /// Exactly one operation terminal.
    OperationFinished = 61,
    /// Stable peer error code.
    Failure = 62,
}
impl TryFrom<u64> for Kind {
    type Error = Error;
    fn try_from(n: u64) -> Result<Self, Error> {
        Ok(match n {
            1 => Self::Hello,
            2 => Self::Welcome,
            3 => Self::Reject,
            10 => Self::Probe,
            11 => Self::Open,
            12 => Self::Scan,
            13 => Self::ReadEntry,
            14 => Self::Close,
            20 => Self::ReadAt,
            21 => Self::ReadData,
            30 => Self::EntryBatch,
            31 => Self::ScanFinished,
            40 => Self::WriteData,
            41 => Self::FinishOutput,
            42 => Self::WriteAck,
            50 => Self::Cancel,
            51 => Self::CancelAck,
            52 => Self::Shutdown,
            60 => Self::Progress,
            61 => Self::OperationFinished,
            62 => Self::Failure,
            _ => return Err(Error::Invalid),
        })
    }
}
/// Owned envelope. Both directions validate their own monotonic sequence.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Envelope {
    /// Message kind.
    pub kind: Kind,
    /// Request identity, zero only for the handshake.
    pub request: u64,
    /// Operation identity, zero only for the handshake.
    pub operation: u64,
    /// Monotonic, nonzero sequence in this direction.
    pub sequence: u64,
    /// Kind-specific fixed-schema map.
    pub payload: Value,
}
impl Envelope {
    /// Write a bounded frame and flush its private pipe.
    pub fn write(&self, writer: &mut impl Write) -> Result<(), Error> {
        let value = Value::map([
            (1, Value::UInt(self.kind as u64)),
            (2, Value::UInt(self.request)),
            (3, Value::UInt(self.operation)),
            (4, Value::UInt(self.sequence)),
            (5, self.payload.clone()),
        ]);
        let bytes = encode(&value)?;
        writer
            .write_all(&(bytes.len() as u32).to_le_bytes())
            .map_err(|_| Error::Io)?;
        writer.write_all(&bytes).map_err(|_| Error::Io)?;
        writer.flush().map_err(|_| Error::Io)
    }
    /// Read one frame; reject the length before allocating any payload.
    pub fn read(reader: &mut impl Read) -> Result<Self, Error> {
        let mut len = [0; 4];
        reader.read_exact(&mut len).map_err(|_| Error::Io)?;
        let len = u32::from_le_bytes(len) as usize;
        if len == 0 || len > MAX_FRAME {
            return Err(Error::Limit);
        }
        let mut bytes = vec![0; len];
        reader.read_exact(&mut bytes).map_err(|_| Error::Io)?;
        Self::from_value(decode(&bytes)?)
    }
    fn from_value(value: Value) -> Result<Self, Error> {
        value.schema(&[1, 2, 3, 4, 5])?;
        let sequence = value.field(4)?.uint()?;
        if sequence == 0 {
            return Err(Error::Invalid);
        }
        let payload = value.field(5)?.clone();
        if !matches!(payload, Value::Map(_)) {
            return Err(Error::Invalid);
        }
        Ok(Self {
            kind: value.field(1)?.uint()?.try_into()?,
            request: value.field(2)?.uint()?,
            operation: value.field(3)?.uint()?,
            sequence,
            payload,
        })
    }
}
/// Validate direction-wide sequence and the exact current request/operation pair.
pub fn accept(
    envelope: &Envelope,
    next: &mut u64,
    request: u64,
    operation: u64,
) -> Result<(), Error> {
    if envelope.sequence != *next || envelope.request != request || envelope.operation != operation
    {
        return Err(Error::Invalid);
    }
    *next = next.checked_add(1).ok_or(Error::Limit)?;
    Ok(())
}
fn head(output: &mut Vec<u8>, major: u8, n: u64) {
    if n < 24 {
        output.push(major << 5 | n as u8);
    } else if n <= u8::MAX as u64 {
        output.extend([major << 5 | 24, n as u8]);
    } else if n <= u16::MAX as u64 {
        output.push(major << 5 | 25);
        output.extend((n as u16).to_be_bytes());
    } else if n <= u32::MAX as u64 {
        output.push(major << 5 | 26);
        output.extend((n as u32).to_be_bytes());
    } else {
        output.push(major << 5 | 27);
        output.extend(n.to_be_bytes());
    }
}
/// Encode the admitted deterministic CBOR subset, validating depth and sizes.
pub fn encode(value: &Value) -> Result<Vec<u8>, Error> {
    fn visit(v: &Value, out: &mut Vec<u8>, depth: usize) -> Result<(), Error> {
        if depth > 16 {
            return Err(Error::Limit);
        }
        match v {
            Value::UInt(n) => head(out, 0, *n),
            Value::Bool(b) => out.push(if *b { 0xf5 } else { 0xf4 }),
            Value::Bytes(b) => {
                if b.len() > MAX_BLOCK {
                    return Err(Error::Limit);
                }
                head(out, 2, b.len() as u64);
                out.extend(b);
            }
            Value::Text(s) => {
                if s.len() > MAX_BLOCK {
                    return Err(Error::Limit);
                }
                head(out, 3, s.len() as u64);
                out.extend(s.as_bytes());
            }
            Value::Array(a) => {
                if a.len() > 128 {
                    return Err(Error::Limit);
                }
                head(out, 4, a.len() as u64);
                for v in a {
                    visit(v, out, depth + 1)?;
                }
            }
            Value::Map(m) => {
                if m.len() > 32 {
                    return Err(Error::Limit);
                }
                head(out, 5, m.len() as u64);
                for (key, v) in m {
                    head(out, 0, *key);
                    visit(v, out, depth + 1)?;
                }
            }
        }
        if out.len() > MAX_FRAME {
            return Err(Error::Limit);
        }
        Ok(())
    }
    let mut out = Vec::new();
    visit(value, &mut out, 0)?;
    Ok(out)
}
/// Decode without accepting trailing bytes, duplicate keys or nonminimal integers.
pub fn decode(bytes: &[u8]) -> Result<Value, Error> {
    struct Parser<'a> {
        bytes: &'a [u8],
        at: usize,
        nodes: usize,
    }
    impl Parser<'_> {
        fn take(&mut self, len: usize) -> Result<&[u8], Error> {
            let end = self
                .at
                .checked_add(len)
                .filter(|n| *n <= self.bytes.len())
                .ok_or(Error::Invalid)?;
            let value = &self.bytes[self.at..end];
            self.at = end;
            Ok(value)
        }
        fn item(&mut self, depth: usize) -> Result<Value, Error> {
            self.nodes += 1;
            if depth > 16 || self.nodes > 8192 {
                return Err(Error::Limit);
            }
            let first = self.take(1)?[0];
            if first == 0xf4 || first == 0xf5 {
                return Ok(Value::Bool(first == 0xf5));
            }
            let major = first >> 5;
            if !matches!(major, 0 | 2..=5) {
                return Err(Error::Invalid);
            }
            let additional = first & 31;
            let (n, minimum) = match additional {
                n @ 0..=23 => (u64::from(n), 0),
                24 => (u64::from(self.take(1)?[0]), 24),
                25 => (
                    u64::from(u16::from_be_bytes(self.take(2)?.try_into().unwrap())),
                    256,
                ),
                26 => (
                    u64::from(u32::from_be_bytes(self.take(4)?.try_into().unwrap())),
                    65536,
                ),
                27 => (
                    u64::from_be_bytes(self.take(8)?.try_into().unwrap()),
                    1 << 32,
                ),
                _ => return Err(Error::Invalid),
            };
            if n < minimum {
                return Err(Error::Invalid);
            }
            if major == 0 {
                return Ok(Value::UInt(n));
            }
            let len = usize::try_from(n).map_err(|_| Error::Limit)?;
            Ok(match major {
                2 | 3 => {
                    if len > MAX_BLOCK {
                        return Err(Error::Limit);
                    }
                    let b = self.take(len)?;
                    if major == 2 {
                        Value::Bytes(b.to_vec())
                    } else {
                        Value::Text(
                            std::str::from_utf8(b)
                                .map_err(|_| Error::Invalid)?
                                .to_owned(),
                        )
                    }
                }
                4 => {
                    if len > 128 {
                        return Err(Error::Limit);
                    }
                    let mut a = Vec::with_capacity(len);
                    for _ in 0..len {
                        a.push(self.item(depth + 1)?);
                    }
                    Value::Array(a)
                }
                5 => {
                    if len > 32 {
                        return Err(Error::Limit);
                    }
                    let mut m = BTreeMap::new();
                    for _ in 0..len {
                        let key = self.item(depth + 1)?.uint()?;
                        let v = self.item(depth + 1)?;
                        if m.insert(key, v).is_some() {
                            return Err(Error::Invalid);
                        }
                    }
                    Value::Map(m)
                }
                _ => return Err(Error::Invalid),
            })
        }
    }
    if bytes.len() > MAX_FRAME {
        return Err(Error::Limit);
    }
    let mut parser = Parser {
        bytes,
        at: 0,
        nodes: 0,
    };
    let value = parser.item(0)?;
    if parser.at != bytes.len() {
        return Err(Error::Invalid);
    }
    Ok(value)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn wire_vectors_and_adversarial_cbor_are_bounded() {
        // RFC 8949 deterministic vectors, independent of this encoder.
        assert_eq!(decode(&[0x19, 0x03, 0xe8]).unwrap(), Value::UInt(1000));
        assert_eq!(
            encode(&Value::UInt(u64::MAX)).unwrap(),
            [0x1b, 255, 255, 255, 255, 255, 255, 255, 255]
        );
        for bytes in [
            &[0x18, 1][..],
            &[0x9f, 0xff],
            &[0xa2, 1, 0, 1, 1],
            &[0x61, 255],
            &[0xf9, 0, 0],
            &[0, 0],
        ] {
            assert!(decode(bytes).is_err());
        }
        let mut nested = vec![0x81; 17];
        nested.push(0);
        assert_eq!(decode(&nested), Err(Error::Limit));
        assert_eq!(
            Envelope::read(&mut std::io::Cursor::new(u32::MAX.to_le_bytes())),
            Err(Error::Limit)
        );
    }
    #[test]
    fn envelopes_reject_sequence_replay_and_foreign_operations() {
        let message = Envelope {
            kind: Kind::Hello,
            request: 0,
            operation: 0,
            sequence: 1,
            payload: Value::map([(1, Value::UInt(MAJOR))]),
        };
        let mut wire = Vec::new();
        message.write(&mut wire).unwrap();
        let decoded = Envelope::read(&mut std::io::Cursor::new(wire)).unwrap();
        assert_eq!(decoded, message);
        let mut next = 1;
        accept(&decoded, &mut next, 0, 0).unwrap();
        assert_eq!(accept(&decoded, &mut next, 0, 0), Err(Error::Invalid));
        let mut next = 1;
        assert_eq!(accept(&decoded, &mut next, 1, 1), Err(Error::Invalid));
    }
}
