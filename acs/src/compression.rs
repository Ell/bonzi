//! Decompressor for the Microsoft Agent (.acs) compression format.
//!
//! This implements an LZ77-style compression scheme used in Microsoft Agent files.
//! See: https://uploads.s.zeid.me/ms-agent-format-spec.html#Compression

use std::fmt;

use crate::bit_reader::Bits;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DecompressionError {
    UnexpectedEof,
    MissingLeadingZero,
    MalformedLengthEncoding,
    InvalidBackReference,
}

impl fmt::Display for DecompressionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnexpectedEof => write!(f, "unexpected end of input"),
            Self::MissingLeadingZero => write!(f, "missing leading zero byte"),
            Self::MalformedLengthEncoding => write!(f, "malformed length encoding"),
            Self::InvalidBackReference => write!(f, "invalid back-reference offset"),
        }
    }
}

impl std::error::Error for DecompressionError {}

pub fn decompress(bytes: Vec<u8>) -> Result<Vec<u8>, DecompressionError> {
    let mut ret = Vec::new();

    let mut bits = Bits::new(bytes);

    // Compressed data must start with a 0x00 byte
    if bits.pop_byte().ok_or(DecompressionError::UnexpectedEof)? != 0 {
        return Err(DecompressionError::MissingLeadingZero);
    }

    // Process bit stream: each sequence starts with a control bit
    while let Some(bty) = bits.pop_bit() {
        match bty {
            // 1-bit: Back-reference (copy from earlier in output buffer)
            true => {
                // Minimum copy length is 2 bytes
                let mut bytes_to_read = 2;

                // Count sequential 1-bits (max 3) to determine offset encoding tier
                let mut off_sequential_ones = 0;
                for _ in 0..3 {
                    if bits.pop_bit().ok_or(DecompressionError::UnexpectedEof)? {
                        off_sequential_ones += 1
                    } else {
                        break;
                    }
                }

                // Offset encoding tiers:
                //   0 sequential 1s:  6-bit offset + 1    (offsets 1-64)
                //   1 sequential 1s:  9-bit offset + 65   (offsets 65-576)
                //   2 sequential 1s: 12-bit offset + 577  (offsets 577-4672)
                //   3 sequential 1s: 20-bit offset + 4673 (offsets 4673+)
                let (bitcount, addend) = match off_sequential_ones {
                    0 => (6, 1),
                    1 => (9, 65),
                    2 => (12, 577),
                    3 => (20, 4673),
                    _ => unreachable!(),
                };

                let mut num = bits
                    .pop_bits(bitcount)
                    .ok_or(DecompressionError::UnexpectedEof)?;

                // End-of-stream marker: 20-bit offset with value 0xFFFFF (before adding 4673)
                if bitcount == 20 {
                    if num == 0x000fffff {
                        break;
                    } else {
                        bytes_to_read += 1;
                    }
                }

                num += addend;
                if (num as usize) > ret.len() {
                    return Err(DecompressionError::InvalidBackReference);
                }
                let idx = ret.len() - num as usize;

                // Length encoding: count sequential 1-bits (max 11), terminated by 0-bit
                let mut sequential_ones = 0;
                for i in 0..12 {
                    if i == 11 {
                        // 11th bit must be 0 (terminator)
                        if bits.pop_bit().ok_or(DecompressionError::UnexpectedEof)? {
                            return Err(DecompressionError::MalformedLengthEncoding);
                        }
                    } else {
                        match bits.pop_bit().ok_or(DecompressionError::UnexpectedEof)? {
                            true => {
                                sequential_ones += 1;
                            }
                            false => break,
                        }
                    }
                }

                // Final length = base (2) + (2^sequential_ones - 1) + next N bits
                bytes_to_read += (1 << sequential_ones) - 1;
                bytes_to_read +=
                    bits.pop_bits(sequential_ones)
                        .ok_or(DecompressionError::UnexpectedEof)? as usize;

                // Copy bytes from back-reference position (may overlap with destination)
                for i in 0..bytes_to_read {
                    ret.push(ret[idx + i]);
                }
            }
            // 0-bit: Literal byte (next 8 bits are raw data)
            false => {
                let b = bits.pop_byte().ok_or(DecompressionError::UnexpectedEof)?;
                ret.push(b);
            }
        }
    }
    Ok(ret)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Test case from the MS Agent format spec:
    /// https://uploads.s.zeid.me/ms-agent-format-spec.html#Compression
    #[test]
    fn test_decompress_spec_example() {
        let compressed: Vec<u8> = vec![
            0x00, 0x40, 0x00, 0x04, 0x10, 0xD0, 0x90, 0x80, 0x42, 0xED, 0x98, 0x01, 0xB7, 0xFF,
            0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
        ];

        let expected: Vec<u8> = vec![
            0x20, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xA8, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00,
        ];

        let result = decompress(compressed).expect("decompression failed");
        assert_eq!(result, expected);
    }
}
