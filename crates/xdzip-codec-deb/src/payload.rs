use crate::{
    decode_error,
    envelope::Member,
    index::{Index, Origin},
    source::Root,
};
use std::{cell::Cell, io::Read, rc::Rc};
use xdzip_codec_api::{
    Cancellation, Entry, ScanError,
    payload::{DecodeError, DecodedSink, PayloadReader},
};
pub(crate) struct Reader<'a> {
    pub(crate) root: Root<'a>,
    pub(crate) index: Index,
    pub(crate) readers: Vec<Box<dyn PayloadReader + 'a>>,
    pub(crate) members: Vec<Member>,
    pub(crate) delivery: Rc<Cell<u64>>,
    pub(crate) failure: Option<DecodeError>,
}
impl Reader<'_> {
    fn deliver(
        &mut self,
        entry: &Entry,
        sink: &mut dyn DecodedSink,
        control: &Cancellation,
    ) -> Result<(), DecodeError> {
        control.check().map_err(decode_error)?;
        let at = usize::try_from(entry.id.get() - 1).map_err(|_| DecodeError::Integrity)?;
        if self.index.rows.get(at) != Some(entry) {
            return Err(DecodeError::Integrity);
        }
        if !entry.payload_supported() {
            return Err(DecodeError::Unsupported);
        }
        match self.index.origins.get(at).ok_or(DecodeError::Integrity)? {
            Origin::Inner(group, child) => {
                self.readers[*group].decode(&self.index.children[*group][*child], sink, control)
            }
            Origin::Member(member) => {
                let member = &self.members[*member];
                let mut source = self
                    .root
                    .range(member.offset, member.size)
                    .map_err(decode_error)?;
                let mut remaining = member.size;
                let mut bytes = [0; 64 * 1024];
                while remaining != 0 {
                    control.check().map_err(decode_error)?;
                    let budget = self.delivery.get();
                    if budget == 0 {
                        return Err(DecodeError::Limit);
                    }
                    let take = remaining.min(budget).min(bytes.len() as u64) as usize;
                    let n = source
                        .read(&mut bytes[..take])
                        .map_err(|error| decode_error(ScanError::from_source_error(&error)))?;
                    if n == 0 {
                        return Err(DecodeError::Integrity);
                    }
                    self.delivery.set(budget - n as u64);
                    sink.write_chunk(&bytes[..n])?;
                    remaining -= n as u64;
                }
                Ok(())
            }
        }
    }
}
impl PayloadReader for Reader<'_> {
    fn decode(
        &mut self,
        entry: &Entry,
        sink: &mut dyn DecodedSink,
        control: &Cancellation,
    ) -> Result<(), DecodeError> {
        if let Some(error) = self.failure {
            return Err(error);
        }
        let result = self.deliver(entry, sink, control);
        if let Err(error) = result {
            self.failure = Some(error);
        }
        result
    }
}
