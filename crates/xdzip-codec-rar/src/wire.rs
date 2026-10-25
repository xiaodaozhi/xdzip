use xdzip_codec_api::ScanError;

pub(crate) struct Wire<'a> {
    bytes: &'a [u8],
    pub at: usize,
}
impl<'a> Wire<'a> {
    pub fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, at: 0 }
    }
    pub fn remaining(&self) -> usize {
        self.bytes.len() - self.at
    }
    pub fn rest(&self) -> &'a [u8] {
        &self.bytes[self.at..]
    }
    pub fn take(&mut self, count: usize) -> Result<&'a [u8], ScanError> {
        let end = self
            .at
            .checked_add(count)
            .filter(|end| *end <= self.bytes.len())
            .ok_or(ScanError::Invalid)?;
        let result = &self.bytes[self.at..end];
        self.at = end;
        Ok(result)
    }
    pub fn byte(&mut self) -> Result<u8, ScanError> {
        Ok(self.take(1)?[0])
    }
    pub fn u16(&mut self) -> Result<u16, ScanError> {
        Ok(u16::from_le_bytes(
            self.take(2)?.try_into().map_err(|_| ScanError::Invalid)?,
        ))
    }
    pub fn u32(&mut self) -> Result<u32, ScanError> {
        Ok(u32::from_le_bytes(
            self.take(4)?.try_into().map_err(|_| ScanError::Invalid)?,
        ))
    }
    pub fn u64(&mut self) -> Result<u64, ScanError> {
        Ok(u64::from_le_bytes(
            self.take(8)?.try_into().map_err(|_| ScanError::Invalid)?,
        ))
    }
    pub fn vint(&mut self) -> Result<u64, ScanError> {
        let mut value = 0;
        for shift in 0..10 {
            let byte = self.byte()?;
            if shift == 9 && byte > 1 {
                return Err(ScanError::Invalid);
            }
            value |= u64::from(byte & 127) << (shift * 7);
            if byte & 128 == 0 {
                return Ok(value);
            }
        }
        Err(ScanError::Invalid)
    }
    pub fn end(&self) -> Result<(), ScanError> {
        if self.remaining() == 0 {
            Ok(())
        } else {
            Err(ScanError::Invalid)
        }
    }
}
