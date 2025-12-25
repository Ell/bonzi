//! High-level ACS file API.
//!
//! Provides lazy extraction of images, animations, and audio from ACS files.

use std::fmt;

use crate::compression::{DecompressionError, decompress};
use crate::reader::{
    AcsHeader, AcsReader, AudioEntry, ImageEntry, RawAnimationInfo, RawCharacterInfo, RawImageInfo,
    ReaderError,
};

#[derive(Debug)]
pub enum AcsError {
    Reader(ReaderError),
    Decompression(DecompressionError),
    InvalidImageIndex(usize),
    InvalidSoundIndex(usize),
    AnimationNotFound(String),
}

impl fmt::Display for AcsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Reader(e) => write!(f, "reader error: {}", e),
            Self::Decompression(e) => write!(f, "decompression error: {}", e),
            Self::InvalidImageIndex(i) => write!(f, "invalid image index: {}", i),
            Self::InvalidSoundIndex(i) => write!(f, "invalid sound index: {}", i),
            Self::AnimationNotFound(name) => write!(f, "animation not found: {}", name),
        }
    }
}

impl std::error::Error for AcsError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Reader(e) => Some(e),
            Self::Decompression(e) => Some(e),
            _ => None,
        }
    }
}

impl From<ReaderError> for AcsError {
    fn from(e: ReaderError) -> Self {
        Self::Reader(e)
    }
}

impl From<DecompressionError> for AcsError {
    fn from(e: DecompressionError) -> Self {
        Self::Decompression(e)
    }
}

/// Raw RGBA image data (WASM-friendly, no dependencies)
#[derive(Debug, Clone)]
pub struct Image {
    pub width: u32,
    pub height: u32,
    /// RGBA pixel data, row-major order
    pub data: Vec<u8>,
}

#[derive(Debug, Clone)]
pub struct Animation {
    pub name: String,
    pub frames: Vec<Frame>,
    pub return_animation: Option<String>,
    pub transition_type: TransitionType,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransitionType {
    None,
    ReturnAnimation,
    ExitBranch,
}

impl From<u8> for TransitionType {
    fn from(val: u8) -> Self {
        match val {
            1 => Self::ReturnAnimation,
            2 => Self::ExitBranch,
            _ => Self::None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Frame {
    pub images: Vec<FrameImage>,
    /// Duration in milliseconds (original is in 1/100 sec, we convert)
    pub duration_ms: u32,
    pub sound_index: Option<usize>,
    pub exit_branch: Option<usize>,
    pub branches: Vec<Branch>,
    pub overlays: Vec<Overlay>,
}

#[derive(Debug, Clone)]
pub struct FrameImage {
    pub image_index: usize,
    pub x: i16,
    pub y: i16,
}

#[derive(Debug, Clone)]
pub struct Branch {
    pub frame_index: usize,
    pub probability: u16,
}

#[derive(Debug, Clone)]
pub struct Overlay {
    pub overlay_type: OverlayType,
    pub replace_enabled: bool,
    pub image_index: usize,
    pub x: i16,
    pub y: i16,
    pub width: u16,
    pub height: u16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OverlayType {
    MouthClosed,
    MouthWide1,
    MouthWide2,
    MouthWide3,
    MouthWide4,
    MouthMedium,
    MouthNarrow,
    Unknown(u8),
}

impl From<u8> for OverlayType {
    fn from(val: u8) -> Self {
        match val {
            0 => Self::MouthClosed,
            1 => Self::MouthWide1,
            2 => Self::MouthWide2,
            3 => Self::MouthWide3,
            4 => Self::MouthWide4,
            5 => Self::MouthMedium,
            6 => Self::MouthNarrow,
            n => Self::Unknown(n),
        }
    }
}

#[derive(Debug, Clone)]
pub struct CharacterInfo {
    pub name: String,
    pub description: String,
    pub width: u16,
    pub height: u16,
    pub transparent_color: u8,
    /// RGBA palette (256 entries max)
    pub palette: Vec<[u8; 4]>,
    pub guid: [u8; 16],
}

#[derive(Debug, Clone)]
pub struct Sound {
    /// Raw WAV data
    pub data: Vec<u8>,
}

/// A character state grouping animations.
#[derive(Debug, Clone)]
pub struct State {
    pub name: String,
    pub animations: Vec<String>,
}

struct AnimationCacheEntry {
    name: String,
    offset: u32,
    cached: Option<Animation>,
}

pub struct Acs {
    data: Vec<u8>,
    #[allow(dead_code)]
    header: AcsHeader,
    character_info: CharacterInfo,
    #[allow(dead_code)]
    raw_character_info: RawCharacterInfo,
    animation_list: Vec<AnimationCacheEntry>,
    image_list: Vec<ImageEntry>,
    audio_list: Vec<AudioEntry>,
    states: Vec<State>,
}

impl Acs {
    /// Parse an ACS file from a byte buffer.
    pub fn new(data: Vec<u8>) -> Result<Self, AcsError> {
        let mut reader = AcsReader::new(&data);

        let header = reader.read_header()?;

        let raw_character_info = reader.read_character_info(header.character_info.offset)?;

        let (name, description) = if let Some(info) = raw_character_info.localized_info.first() {
            (info.name.clone(), info.description.clone())
        } else {
            (String::new(), String::new())
        };

        let palette: Vec<[u8; 4]> = raw_character_info
            .palette
            .iter()
            .map(|[r, g, b]| [*r, *g, *b, 255])
            .collect();

        let character_info = CharacterInfo {
            name,
            description,
            width: raw_character_info.width,
            height: raw_character_info.height,
            transparent_color: raw_character_info.transparent_color,
            palette,
            guid: raw_character_info.guid,
        };

        let raw_animations = reader.read_animation_list(&header.animation_info)?;
        let animation_list: Vec<AnimationCacheEntry> = raw_animations
            .into_iter()
            .map(|entry| AnimationCacheEntry {
                name: entry.name,
                offset: entry.locator.offset,
                cached: None,
            })
            .collect();

        let image_list = reader.read_image_list(&header.image_info)?;

        let audio_list = reader.read_audio_list(&header.audio_info)?;

        // Convert states from raw format
        let states: Vec<State> = raw_character_info
            .states
            .iter()
            .map(|s| State {
                name: s.name.clone(),
                animations: s.animations.clone(),
            })
            .collect();

        Ok(Self {
            data,
            header,
            character_info,
            raw_character_info,
            animation_list,
            image_list,
            audio_list,
            states,
        })
    }

    /// Get character metadata.
    pub fn character_info(&self) -> &CharacterInfo {
        &self.character_info
    }

    /// List all animation names.
    pub fn animation_names(&self) -> Vec<&str> {
        self.animation_list
            .iter()
            .map(|e| e.name.as_str())
            .collect()
    }

    /// Get all states (animation groupings).
    pub fn states(&self) -> &[State] {
        &self.states
    }

    /// Get animation by name (lazy load).
    pub fn animation(&mut self, name: &str) -> Result<&Animation, AcsError> {
        let idx = self
            .animation_list
            .iter()
            .position(|e| e.name.eq_ignore_ascii_case(name))
            .ok_or_else(|| AcsError::AnimationNotFound(name.to_string()))?;

        if self.animation_list[idx].cached.is_some() {
            return Ok(self.animation_list[idx].cached.as_ref().unwrap());
        }

        // Load the animation
        let offset = self.animation_list[idx].offset;
        let mut reader = AcsReader::new(&self.data);
        let raw = reader.read_animation_info(offset)?;

        let animation = self.convert_animation(&raw);
        self.animation_list[idx].cached = Some(animation);

        Ok(self.animation_list[idx].cached.as_ref().unwrap())
    }

    fn convert_animation(&self, raw: &RawAnimationInfo) -> Animation {
        let frames: Vec<Frame> = raw
            .frames
            .iter()
            .map(|f| Frame {
                images: f
                    .images
                    .iter()
                    .map(|img| FrameImage {
                        image_index: img.image_index as usize,
                        x: img.x_offset,
                        y: img.y_offset,
                    })
                    .collect(),
                duration_ms: f.duration as u32 * 10, // Convert 1/100s to ms
                sound_index: if f.sound_index >= 0 {
                    Some(f.sound_index as usize)
                } else {
                    None
                },
                exit_branch: if f.exit_branch >= 0 {
                    Some(f.exit_branch as usize)
                } else {
                    None
                },
                branches: f
                    .branches
                    .iter()
                    .map(|b| Branch {
                        frame_index: b.frame_index as usize,
                        probability: b.probability,
                    })
                    .collect(),
                overlays: f
                    .overlays
                    .iter()
                    .map(|o| Overlay {
                        overlay_type: OverlayType::from(o.overlay_type),
                        replace_enabled: o.replace_enabled,
                        image_index: o.image_index as usize,
                        x: o.x_offset,
                        y: o.y_offset,
                        width: o.width,
                        height: o.height,
                    })
                    .collect(),
            })
            .collect();

        Animation {
            name: raw.name.clone(),
            frames,
            return_animation: if raw.return_animation.is_empty() {
                None
            } else {
                Some(raw.return_animation.clone())
            },
            transition_type: TransitionType::from(raw.transition_type),
        }
    }

    /// Get the number of images in the file.
    pub fn image_count(&self) -> usize {
        self.image_list.len()
    }

    /// Get image by index (lazy decompress + palette apply).
    pub fn image(&self, index: usize) -> Result<Image, AcsError> {
        if index >= self.image_list.len() {
            return Err(AcsError::InvalidImageIndex(index));
        }

        let entry = &self.image_list[index];
        let mut reader = AcsReader::new(&self.data);
        let raw = reader.read_image_info(entry.locator.offset)?;

        self.decode_image(&raw)
    }

    fn decode_image(&self, raw: &RawImageInfo) -> Result<Image, AcsError> {
        let pixel_data = if raw.is_compressed {
            decompress(raw.data.clone())?
        } else {
            raw.data.clone()
        };

        let row_width = (raw.width as usize + 3) & !3;
        let _expected_size = row_width * raw.height as usize;

        // ACS images are stored bottom-up, we need to flip them
        let mut rgba = Vec::with_capacity(raw.width as usize * raw.height as usize * 4);

        for y in (0..raw.height as usize).rev() {
            for x in 0..raw.width as usize {
                let idx = y * row_width + x;
                if idx < pixel_data.len() {
                    let color_index = pixel_data[idx] as usize;
                    if color_index == self.character_info.transparent_color as usize {
                        rgba.extend_from_slice(&[0, 0, 0, 0]);
                    } else if color_index < self.character_info.palette.len() {
                        rgba.extend_from_slice(&self.character_info.palette[color_index]);
                    } else {
                        rgba.extend_from_slice(&[0, 0, 0, 255]);
                    }
                } else {
                    rgba.extend_from_slice(&[0, 0, 0, 0]);
                }
            }
        }

        Ok(Image {
            width: raw.width as u32,
            height: raw.height as u32,
            data: rgba,
        })
    }

    /// Get the number of sounds in the file.
    pub fn sound_count(&self) -> usize {
        self.audio_list.len()
    }

    /// Get sound by index.
    pub fn sound(&self, index: usize) -> Result<Sound, AcsError> {
        if index >= self.audio_list.len() {
            return Err(AcsError::InvalidSoundIndex(index));
        }

        let entry = &self.audio_list[index];
        let mut reader = AcsReader::new(&self.data);
        let data = reader.read_audio_data(entry)?;

        Ok(Sound { data })
    }

    /// Render a complete animation frame by compositing all frame images.
    pub fn render_frame(
        &self,
        animation_name: &str,
        frame_index: usize,
    ) -> Result<Image, AcsError> {
        let anim_idx = self
            .animation_list
            .iter()
            .position(|e| e.name.eq_ignore_ascii_case(animation_name))
            .ok_or_else(|| AcsError::AnimationNotFound(animation_name.to_string()))?;

        let frame = if let Some(ref cached) = self.animation_list[anim_idx].cached {
            cached.frames.get(frame_index)
        } else {
            let offset = self.animation_list[anim_idx].offset;
            let mut reader = AcsReader::new(&self.data);
            let raw = reader.read_animation_info(offset)?;
            let animation = self.convert_animation(&raw);

            if frame_index < animation.frames.len() {
                return self.composite_frame(&animation.frames[frame_index]);
            } else {
                return Err(AcsError::InvalidImageIndex(frame_index));
            }
        };

        let frame = frame.ok_or(AcsError::InvalidImageIndex(frame_index))?;
        self.composite_frame(frame)
    }

    fn composite_frame(&self, frame: &Frame) -> Result<Image, AcsError> {
        let width = self.character_info.width as u32;
        let height = self.character_info.height as u32;

        let mut canvas = vec![0u8; (width * height * 4) as usize];

        for frame_img in frame.images.iter().rev() {
            let img = self.image(frame_img.image_index)?;

            // Blit the image onto the canvas
            for y in 0..img.height {
                for x in 0..img.width {
                    let dst_x = frame_img.x as i32 + x as i32;
                    let dst_y = frame_img.y as i32 + y as i32;

                    if dst_x >= 0 && dst_x < width as i32 && dst_y >= 0 && dst_y < height as i32 {
                        let src_idx = ((y * img.width + x) * 4) as usize;
                        let dst_idx = ((dst_y as u32 * width + dst_x as u32) * 4) as usize;

                        let alpha = img.data[src_idx + 3];
                        if alpha > 0 {
                            canvas[dst_idx] = img.data[src_idx];
                            canvas[dst_idx + 1] = img.data[src_idx + 1];
                            canvas[dst_idx + 2] = img.data[src_idx + 2];
                            canvas[dst_idx + 3] = alpha;
                        }
                    }
                }
            }
        }

        Ok(Image {
            width,
            height,
            data: canvas,
        })
    }
}
