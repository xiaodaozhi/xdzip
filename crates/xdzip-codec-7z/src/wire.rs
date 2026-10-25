use xdzip_codec_api::ScanError;
pub(crate) struct Cursor<'a> {
    data: &'a [u8],
    at: usize,
}
impl<'a> Cursor<'a> {
    pub(crate) fn new(data: &'a [u8]) -> Self {
        Self { data, at: 0 }
    }
    pub(crate) fn byte(&mut self) -> Result<u8, ScanError> {
        Ok(self.take(1)?[0])
    }
    pub(crate) fn peek(&self) -> Option<u8> {
        self.data.get(self.at).copied()
    }
    pub(crate) fn take(&mut self, count: usize) -> Result<&'a [u8], ScanError> {
        let end = self.at.checked_add(count).ok_or(ScanError::Invalid)?;
        let value = self.data.get(self.at..end).ok_or(ScanError::Invalid)?;
        self.at = end;
        Ok(value)
    }
    pub(crate) fn end(&self) -> Result<(), ScanError> {
        if self.at == self.data.len() {
            Ok(())
        } else {
            Err(ScanError::Invalid)
        }
    }
    pub(crate) fn id(&mut self, id: u8) -> Result<(), ScanError> {
        if self.byte()? == id {
            Ok(())
        } else {
            Err(ScanError::Invalid)
        }
    }
    pub(crate) fn u32(&mut self) -> Result<u32, ScanError> {
        Ok(u32::from_le_bytes(
            self.take(4)?.try_into().map_err(|_| ScanError::Invalid)?,
        ))
    }
    pub(crate) fn u64(&mut self) -> Result<u64, ScanError> {
        Ok(u64::from_le_bytes(
            self.take(8)?.try_into().map_err(|_| ScanError::Invalid)?,
        ))
    }
    pub(crate) fn number(&mut self) -> Result<u64, ScanError> {
        let first = self.byte()?;
        let mut mask = 0x80u8;
        let mut value = 0u64;
        for index in 0..8 {
            if first & mask == 0 {
                return Ok(value | (u64::from(first & (mask - 1)) << (index * 8)));
            }
            value |= u64::from(self.byte()?) << (index * 8);
            mask >>= 1;
        }
        Ok(value)
    }
    pub(crate) fn count(&mut self, max: usize) -> Result<usize, ScanError> {
        usize::try_from(self.number()?)
            .ok()
            .filter(|n| *n <= max)
            .ok_or(ScanError::Limit)
    }
    pub(crate) fn bits(&mut self, count: usize) -> Result<Vec<bool>, ScanError> {
        let bytes = self.take(count.div_ceil(8))?;
        Ok((0..count)
            .map(|i| bytes[i / 8] & (0x80 >> (i % 8)) != 0)
            .collect())
    }
    pub(crate) fn defined(&mut self, count: usize) -> Result<Vec<bool>, ScanError> {
        if self.byte()? == 0 {
            self.bits(count)
        } else {
            Ok(vec![true; count])
        }
    }
    pub(crate) fn digests(&mut self, count: usize) -> Result<Vec<Option<u32>>, ScanError> {
        let defined = self.defined(count)?;
        let mut values = Vec::with_capacity(count);
        for bit in defined {
            values.push(if bit { Some(self.u32()?) } else { None });
        }
        Ok(values)
    }
}
