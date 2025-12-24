//! Low-level ACS file reader.
//!
//! Provides zero-copy parsing of raw ACS file structures.

use std::fmt;
use std::io::{Cursor, Read};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReaderError {
    UnexpectedEof,
    InvalidSignature(u32),
    InvalidOffset { offset: u32, size: u32 },
    InvalidUtf16,
}

impl fmt::Display for ReaderError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnexpectedEof => write!(f, "unexpected end of file"),
            Self::InvalidSignature(sig) => write!(f, "invalid signature: 0x{:08X}", sig),
            Self::InvalidOffset { offset, size } => {
                write!(f, "invalid offset {} with size {}", offset, size)
            }
            Self::InvalidUtf16 => write!(f, "invalid UTF-16 string"),
        }
    }
}

impl std::error::Error for ReaderError {}

pub const ACS_SIGNATURE: u32 = 0xABCDABC3;

#[derive(Debug, Clone)]
pub struct Locator {
    pub offset: u32,
    pub size: u32,
}

#[derive(Debug, Clone)]
pub struct AcsHeader {
    pub signature: u32,
    pub character_info: Locator,
    pub animation_info: Locator,
    pub image_info: Locator,
    pub audio_info: Locator,
}

#[derive(Debug, Clone)]
pub struct LocalizedInfo {
    pub lang_id: u16,
    pub name: String,
    pub description: String,
    pub extra_data: String,
}

#[derive(Debug, Clone)]
pub struct VoiceInfo {
    pub tts_engine_id: [u8; 16],
    pub tts_mode_id: [u8; 16],
    pub speed: u32,
    pub pitch: u16,
    pub extra_data_exists: bool,
    pub extra_data: Option<VoiceExtraData>,
}

#[derive(Debug, Clone)]
pub struct VoiceExtraData {
    pub lang_id: u16,
    pub lang_dialect: String,
    pub gender: u16,
    pub age: u16,
    pub style: String,
}

#[derive(Debug, Clone)]
pub struct BalloonInfo {
    pub num_lines: u8,
    pub chars_per_line: u8,
    pub fg_color: [u8; 3],
    pub bg_color: [u8; 3],
    pub border_color: [u8; 3],
    pub font_name: String,
    pub font_height: i32,
    pub font_weight: i32,
    pub font_italic: bool,
    pub font_charset: u8,
}

#[derive(Debug, Clone)]
pub struct TrayIcon {
    pub mono_bitmap: Vec<u8>,
    pub color_bitmap: Vec<u8>,
}

#[derive(Debug, Clone)]
pub struct StateInfo {
    pub name: String,
    pub animations: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct RawCharacterInfo {
    pub minor_version: u16,
    pub major_version: u16,
    pub localized_info: Vec<LocalizedInfo>,
    pub guid: [u8; 16],
    pub width: u16,
    pub height: u16,
    pub transparent_color: u8,
    pub flags: u32,
    pub anim_set_major_version: u16,
    pub anim_set_minor_version: u16,
    pub voice_info: Option<VoiceInfo>,
    pub balloon_info: BalloonInfo,
    pub palette: Vec<[u8; 3]>,
    pub tray_icon: Option<TrayIcon>,
    pub states: Vec<StateInfo>,
}

#[derive(Debug, Clone)]
pub struct AnimationEntry {
    pub name: String,
    pub locator: Locator,
}

#[derive(Debug, Clone)]
pub struct RawAnimationInfo {
    pub name: String,
    pub transition_type: u8,
    pub return_animation: String,
    pub frames: Vec<RawFrameInfo>,
}

#[derive(Debug, Clone)]
pub struct RawFrameInfo {
    pub images: Vec<RawFrameImage>,
    pub sound_index: i16,
    pub duration: u16,
    pub exit_branch: i16,
    pub branches: Vec<RawBranchInfo>,
    pub overlays: Vec<RawOverlayInfo>,
}

#[derive(Debug, Clone)]
pub struct RawFrameImage {
    pub image_index: u32,
    pub x_offset: i16,
    pub y_offset: i16,
}

#[derive(Debug, Clone)]
pub struct RawBranchInfo {
    pub frame_index: u16,
    pub probability: u16,
}

#[derive(Debug, Clone)]
pub struct RawOverlayInfo {
    pub overlay_type: u8,
    pub replace_enabled: bool,
    pub image_index: u16,
    pub x_offset: i16,
    pub y_offset: i16,
    pub width: u16,
    pub height: u16,
    pub region_data: Option<Vec<u8>>,
}

#[derive(Debug, Clone)]
pub struct ImageEntry {
    pub locator: Locator,
    pub checksum: u32,
}

#[derive(Debug, Clone)]
pub struct RawImageInfo {
    pub width: u16,
    pub height: u16,
    pub is_compressed: bool,
    pub data: Vec<u8>,
    pub region_data: Option<Vec<u8>>,
}

#[derive(Debug, Clone)]
pub struct AudioEntry {
    pub locator: Locator,
    pub checksum: u32,
}

pub struct AcsReader<'a> {
    cursor: Cursor<&'a [u8]>,
}

impl<'a> AcsReader<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        Self {
            cursor: Cursor::new(data),
        }
    }

    pub fn position(&self) -> u64 {
        self.cursor.position()
    }

    pub fn seek(&mut self, pos: u64) {
        self.cursor.set_position(pos);
    }

    pub fn len(&self) -> usize {
        self.cursor.get_ref().len()
    }

    pub fn is_empty(&self) -> bool {
        self.cursor.get_ref().is_empty()
    }

    pub fn read_u8(&mut self) -> Result<u8, ReaderError> {
        let mut buf = [0u8; 1];
        self.cursor
            .read_exact(&mut buf)
            .map_err(|_| ReaderError::UnexpectedEof)?;
        Ok(buf[0])
    }

    pub fn read_u16(&mut self) -> Result<u16, ReaderError> {
        let mut buf = [0u8; 2];
        self.cursor
            .read_exact(&mut buf)
            .map_err(|_| ReaderError::UnexpectedEof)?;
        Ok(u16::from_le_bytes(buf))
    }

    pub fn read_i16(&mut self) -> Result<i16, ReaderError> {
        let mut buf = [0u8; 2];
        self.cursor
            .read_exact(&mut buf)
            .map_err(|_| ReaderError::UnexpectedEof)?;
        Ok(i16::from_le_bytes(buf))
    }

    pub fn read_u32(&mut self) -> Result<u32, ReaderError> {
        let mut buf = [0u8; 4];
        self.cursor
            .read_exact(&mut buf)
            .map_err(|_| ReaderError::UnexpectedEof)?;
        Ok(u32::from_le_bytes(buf))
    }

    pub fn read_i32(&mut self) -> Result<i32, ReaderError> {
        let mut buf = [0u8; 4];
        self.cursor
            .read_exact(&mut buf)
            .map_err(|_| ReaderError::UnexpectedEof)?;
        Ok(i32::from_le_bytes(buf))
    }

    pub fn read_bytes(&mut self, len: usize) -> Result<Vec<u8>, ReaderError> {
        let mut buf = vec![0u8; len];
        self.cursor
            .read_exact(&mut buf)
            .map_err(|_| ReaderError::UnexpectedEof)?;
        Ok(buf)
    }

    pub fn read_guid(&mut self) -> Result<[u8; 16], ReaderError> {
        let mut guid = [0u8; 16];
        self.cursor
            .read_exact(&mut guid)
            .map_err(|_| ReaderError::UnexpectedEof)?;
        Ok(guid)
    }

    /// Read a length-prefixed UTF-16LE string.
    ///
    /// ACS format: length (character count, not including terminator) followed by
    /// that many UTF-16LE characters plus a null terminator (0x0000).
    pub fn read_string(&mut self) -> Result<String, ReaderError> {
        let len = self.read_u32()? as usize;
        if len == 0 {
            return Ok(String::new());
        }
        // Read len characters + 1 null terminator
        let bytes = self.read_bytes((len + 1) * 2)?;
        // Parse only the actual characters (exclude the null terminator)
        let utf16: Vec<u16> = bytes[..len * 2]
            .chunks_exact(2)
            .map(|chunk| u16::from_le_bytes([chunk[0], chunk[1]]))
            .collect();
        String::from_utf16(&utf16).map_err(|_| ReaderError::InvalidUtf16)
    }

    fn read_locator(&mut self) -> Result<Locator, ReaderError> {
        Ok(Locator {
            offset: self.read_u32()?,
            size: self.read_u32()?,
        })
    }

    pub fn read_header(&mut self) -> Result<AcsHeader, ReaderError> {
        let signature = self.read_u32()?;
        if signature != ACS_SIGNATURE {
            return Err(ReaderError::InvalidSignature(signature));
        }

        Ok(AcsHeader {
            signature,
            character_info: self.read_locator()?,
            animation_info: self.read_locator()?,
            image_info: self.read_locator()?,
            audio_info: self.read_locator()?,
        })
    }

    pub fn read_character_info(&mut self, offset: u32) -> Result<RawCharacterInfo, ReaderError> {
        self.seek(offset as u64);

        let minor_version = self.read_u16()?;
        let major_version = self.read_u16()?;

        // Localized info is stored at a separate location, referenced by a locator
        let localized_info_locator = self.read_locator()?;

        let guid = self.read_guid()?;
        let width = self.read_u16()?;
        let height = self.read_u16()?;
        let transparent_color = self.read_u8()?;
        let flags = self.read_u32()?;

        // Animation set version
        let anim_set_major_version = self.read_u16()?;
        let anim_set_minor_version = self.read_u16()?;

        // Voice info is present only if bit 5 is set (not bit 4 as spec says)
        // Bit 5 = 0x20
        let voice_info = if flags & 0x20 != 0 {
            Some(self.read_voice_info()?)
        } else {
            None
        };

        // Balloon info is always present
        let balloon_info = self.read_balloon_info()?;

        // Palette (count is ULONG, each color is RGBQUAD = 4 bytes)
        // RGBQUAD in Windows is stored as: Blue, Green, Red, Reserved (BGR order)
        let palette_count = self.read_u32()? as usize;
        let mut palette = Vec::with_capacity(palette_count);
        for _ in 0..palette_count {
            let b = self.read_u8()?;
            let g = self.read_u8()?;
            let r = self.read_u8()?;
            let _reserved = self.read_u8()?;
            palette.push([r, g, b]);
        }

        // Tray icon flag (BYTE)
        let has_tray_icon = self.read_u8()? != 0;

        // Tray icon data (if flag is set)
        let tray_icon = if has_tray_icon {
            Some(self.read_tray_icon()?)
        } else {
            None
        };

        // States
        let state_count = self.read_u16()? as usize;
        let mut states = Vec::with_capacity(state_count);
        for _ in 0..state_count {
            states.push(self.read_state_info()?);
        }

        // Now read localized info from its location
        let localized_info = self.read_localized_info_list(localized_info_locator)?;

        Ok(RawCharacterInfo {
            minor_version,
            major_version,
            localized_info,
            guid,
            width,
            height,
            transparent_color,
            flags,
            anim_set_major_version,
            anim_set_minor_version,
            voice_info,
            balloon_info,
            palette,
            tray_icon,
            states,
        })
    }

    fn read_localized_info_list(
        &mut self,
        locator: Locator,
    ) -> Result<Vec<LocalizedInfo>, ReaderError> {
        if locator.size == 0 {
            return Ok(Vec::new());
        }

        self.seek(locator.offset as u64);
        let count = self.read_u16()? as usize;
        let mut list = Vec::with_capacity(count);
        for _ in 0..count {
            let lang_id = self.read_u16()?;
            let name = self.read_string()?;
            let description = self.read_string()?;
            let extra_data = self.read_string()?;
            list.push(LocalizedInfo {
                lang_id,
                name,
                description,
                extra_data,
            });
        }
        Ok(list)
    }

    fn read_voice_info(&mut self) -> Result<VoiceInfo, ReaderError> {
        let tts_engine_id = self.read_guid()?;
        let tts_mode_id = self.read_guid()?;
        let speed = self.read_u32()?;
        let pitch = self.read_u16()?;
        let extra_data_exists = self.read_u8()? != 0;

        let extra_data = if extra_data_exists {
            let lang_id = self.read_u16()?;
            let lang_dialect = self.read_string()?;
            let gender = self.read_u16()?;
            let age = self.read_u16()?;
            let style = self.read_string()?;
            Some(VoiceExtraData {
                lang_id,
                lang_dialect,
                gender,
                age,
                style,
            })
        } else {
            None
        };

        Ok(VoiceInfo {
            tts_engine_id,
            tts_mode_id,
            speed,
            pitch,
            extra_data_exists,
            extra_data,
        })
    }

    fn read_balloon_info(&mut self) -> Result<BalloonInfo, ReaderError> {
        let num_lines = self.read_u8()?;
        let chars_per_line = self.read_u8()?;
        // Colors are RGBQUAD (4 bytes each: R, G, B, Reserved)
        let fg_color = [self.read_u8()?, self.read_u8()?, self.read_u8()?];
        let _fg_reserved = self.read_u8()?;
        let bg_color = [self.read_u8()?, self.read_u8()?, self.read_u8()?];
        let _bg_reserved = self.read_u8()?;
        let border_color = [self.read_u8()?, self.read_u8()?, self.read_u8()?];
        let _border_reserved = self.read_u8()?;
        let font_name = self.read_string()?;
        let font_height = self.read_i32()?;
        let font_weight = self.read_i32()?;
        let font_italic = self.read_u8()? != 0;
        let font_charset = self.read_u8()?;

        Ok(BalloonInfo {
            num_lines,
            chars_per_line,
            fg_color,
            bg_color,
            border_color,
            font_name,
            font_height,
            font_weight,
            font_italic,
            font_charset,
        })
    }

    fn read_tray_icon(&mut self) -> Result<TrayIcon, ReaderError> {
        let mono_size = self.read_u32()? as usize;
        let mono_bitmap = self.read_bytes(mono_size)?;
        let color_size = self.read_u32()? as usize;
        let color_bitmap = self.read_bytes(color_size)?;

        Ok(TrayIcon {
            mono_bitmap,
            color_bitmap,
        })
    }

    fn read_state_info(&mut self) -> Result<StateInfo, ReaderError> {
        let name = self.read_string()?;
        let animation_count = self.read_u16()? as usize;
        let mut animations = Vec::with_capacity(animation_count);
        for _ in 0..animation_count {
            animations.push(self.read_string()?);
        }
        Ok(StateInfo { name, animations })
    }

    pub fn read_animation_list(
        &mut self,
        locator: &Locator,
    ) -> Result<Vec<AnimationEntry>, ReaderError> {
        self.seek(locator.offset as u64);
        let count = self.read_u32()? as usize;
        let mut entries = Vec::with_capacity(count);

        for _ in 0..count {
            let name = self.read_string()?;
            let entry_locator = self.read_locator()?;
            entries.push(AnimationEntry {
                name,
                locator: entry_locator,
            });
        }

        Ok(entries)
    }

    pub fn read_animation_info(&mut self, offset: u32) -> Result<RawAnimationInfo, ReaderError> {
        self.seek(offset as u64);

        let name = self.read_string()?;
        let transition_type = self.read_u8()?;
        let return_animation = self.read_string()?;

        let frame_count = self.read_u16()? as usize;
        let mut frames = Vec::with_capacity(frame_count);

        for _ in 0..frame_count {
            frames.push(self.read_frame_info()?);
        }

        Ok(RawAnimationInfo {
            name,
            transition_type,
            return_animation,
            frames,
        })
    }

    fn read_frame_info(&mut self) -> Result<RawFrameInfo, ReaderError> {
        // Frame images
        let image_count = self.read_u16()? as usize;
        let mut images = Vec::with_capacity(image_count);
        for _ in 0..image_count {
            images.push(RawFrameImage {
                image_index: self.read_u32()?,
                x_offset: self.read_i16()?,
                y_offset: self.read_i16()?,
            });
        }

        let sound_index = self.read_i16()?;
        let duration = self.read_u16()?;
        let exit_branch = self.read_i16()?;

        // Branches (count is BYTE)
        let branch_count = self.read_u8()? as usize;
        let mut branches = Vec::with_capacity(branch_count);
        for _ in 0..branch_count {
            branches.push(RawBranchInfo {
                frame_index: self.read_u16()?,
                probability: self.read_u16()?,
            });
        }

        // Overlays (count is BYTE)
        let overlay_count = self.read_u8()? as usize;
        let mut overlays = Vec::with_capacity(overlay_count);
        for _ in 0..overlay_count {
            overlays.push(self.read_overlay_info()?);
        }

        Ok(RawFrameInfo {
            images,
            sound_index,
            duration,
            exit_branch,
            branches,
            overlays,
        })
    }

    fn read_overlay_info(&mut self) -> Result<RawOverlayInfo, ReaderError> {
        let overlay_type = self.read_u8()?;
        let replace_enabled = self.read_u8()? != 0;
        let image_index = self.read_u16()?;
        let _unknown = self.read_u8()?; // Unknown byte (observed: 0x00)
        let has_region = self.read_u8()? != 0;
        let x_offset = self.read_i16()?;
        let y_offset = self.read_i16()?;
        let width = self.read_u16()?;
        let height = self.read_u16()?;

        let region_data = if has_region {
            let size = self.read_u32()? as usize;
            Some(self.read_bytes(size)?)
        } else {
            None
        };

        Ok(RawOverlayInfo {
            overlay_type,
            replace_enabled,
            image_index,
            x_offset,
            y_offset,
            width,
            height,
            region_data,
        })
    }

    pub fn read_image_list(&mut self, locator: &Locator) -> Result<Vec<ImageEntry>, ReaderError> {
        self.seek(locator.offset as u64);
        let count = self.read_u32()? as usize;
        let mut entries = Vec::with_capacity(count);

        for _ in 0..count {
            entries.push(ImageEntry {
                locator: self.read_locator()?,
                checksum: self.read_u32()?,
            });
        }

        Ok(entries)
    }

    pub fn read_image_info(&mut self, offset: u32) -> Result<RawImageInfo, ReaderError> {
        self.seek(offset as u64);

        let _unknown = self.read_u8()?;
        let width = self.read_u16()?;
        let height = self.read_u16()?;
        let is_compressed = self.read_u8()? != 0;

        // Calculate padded row width (DWORD aligned)
        let row_width = (width as usize + 3) & !3;
        let data_size = row_width * height as usize;

        let data = if is_compressed {
            let compressed_size = self.read_u32()? as usize;
            self.read_bytes(compressed_size)?
        } else {
            self.read_bytes(data_size)?
        };

        // Region data
        let region_compressed_size = self.read_u32()? as usize;
        let _region_uncompressed_size = self.read_u32()?;

        let region_data = if region_compressed_size > 0 {
            Some(self.read_bytes(region_compressed_size)?)
        } else {
            None
        };

        Ok(RawImageInfo {
            width,
            height,
            is_compressed,
            data,
            region_data,
        })
    }

    pub fn read_audio_list(&mut self, locator: &Locator) -> Result<Vec<AudioEntry>, ReaderError> {
        self.seek(locator.offset as u64);
        let count = self.read_u32()? as usize;
        let mut entries = Vec::with_capacity(count);

        for _ in 0..count {
            entries.push(AudioEntry {
                locator: self.read_locator()?,
                checksum: self.read_u32()?,
            });
        }

        Ok(entries)
    }

    pub fn read_audio_data(&mut self, entry: &AudioEntry) -> Result<Vec<u8>, ReaderError> {
        self.seek(entry.locator.offset as u64);
        self.read_bytes(entry.locator.size as usize)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_read_primitives() {
        let data = [0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08];
        let mut reader = AcsReader::new(&data);

        assert_eq!(reader.read_u8().unwrap(), 0x01);
        assert_eq!(reader.read_u16().unwrap(), 0x0302); // bytes [0x02, 0x03] -> 0x0302 LE
        assert_eq!(reader.read_u32().unwrap(), 0x07060504); // bytes [0x04, 0x05, 0x06, 0x07] -> 0x07060504 LE
    }

    #[test]
    fn test_read_string() {
        // Length (4 bytes LE) + UTF-16LE "Hi" + null terminator
        let data = [
            0x02, 0x00, 0x00, 0x00, // length = 2 characters
            0x48, 0x00, // 'H'
            0x69, 0x00, // 'i'
            0x00, 0x00, // null terminator
        ];
        let mut reader = AcsReader::new(&data);
        assert_eq!(reader.read_string().unwrap(), "Hi");
    }

    #[test]
    fn test_unexpected_eof() {
        let data = [0x01, 0x02];
        let mut reader = AcsReader::new(&data);
        assert!(reader.read_u32().is_err());
    }
}
