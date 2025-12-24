pub struct Bits {
    pub bytes: Vec<u8>,
    pub idx: usize,
    pub bidx: usize,
}

impl Bits {
    pub fn new(bytes: Vec<u8>) -> Self {
        Self {
            bytes,
            idx: 0,
            bidx: 0,
        }
    }

    pub fn pop_bit(&mut self) -> Option<bool> {
        let w = self.bytes.get(self.idx)?;
        let ret = (w >> self.bidx) & 0b1;

        self.bidx += 1;

        if self.bidx > 7 {
            self.bidx = 0;
            self.idx += 1;
        }

        Some(ret == 1)
    }

    pub fn pop_bits(&mut self, count: usize) -> Option<u32> {
        let mut ret = 0;
        for shift in 0..count {
            ret |= (self.pop_bit()? as u32) << shift;
        }

        Some(ret)
    }

    pub fn pop_byte(&mut self) -> Option<u8> {
        Some(self.pop_bits(8)? as u8)
    }
}
