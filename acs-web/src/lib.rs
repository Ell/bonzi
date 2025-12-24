//! WASM bindings for the ACS parser.
//!
//! Provides a JavaScript/TypeScript API for loading and rendering ACS files.

use wasm_bindgen::prelude::*;

use acs::Acs;

/// RGBA image data suitable for use with HTML Canvas.
#[wasm_bindgen]
pub struct ImageData {
    #[wasm_bindgen(readonly)]
    pub width: u32,
    #[wasm_bindgen(readonly)]
    pub height: u32,
    data: Vec<u8>,
}

#[wasm_bindgen]
impl ImageData {
    /// Get RGBA pixel data as Uint8Array.
    #[wasm_bindgen(getter)]
    pub fn data(&self) -> js_sys::Uint8Array {
        js_sys::Uint8Array::from(&self.data[..])
    }
}

/// A single frame in an animation.
#[wasm_bindgen]
pub struct FrameData {
    #[wasm_bindgen(readonly, js_name = "durationMs")]
    pub duration_ms: u32,
    #[wasm_bindgen(readonly, js_name = "soundIndex")]
    pub sound_index: i32, // -1 if no sound
    #[wasm_bindgen(readonly, js_name = "imageCount")]
    pub image_count: u32,
}

/// Animation metadata.
#[wasm_bindgen]
pub struct AnimationData {
    name: String,
    return_animation: Option<String>,
    frames: Vec<FrameInfo>,
}

struct FrameInfo {
    duration_ms: u32,
    sound_index: Option<usize>,
    image_count: usize,
}

#[wasm_bindgen]
impl AnimationData {
    /// Animation name.
    #[wasm_bindgen(getter)]
    pub fn name(&self) -> String {
        self.name.clone()
    }

    /// Name of the animation to return to after this one completes.
    #[wasm_bindgen(getter, js_name = "returnAnimation")]
    pub fn return_animation(&self) -> Option<String> {
        self.return_animation.clone()
    }

    /// Number of frames in this animation.
    #[wasm_bindgen(getter, js_name = "frameCount")]
    pub fn frame_count(&self) -> usize {
        self.frames.len()
    }

    /// Get frame metadata by index.
    #[wasm_bindgen(js_name = "getFrame")]
    pub fn get_frame(&self, index: usize) -> Option<FrameData> {
        self.frames.get(index).map(|f| FrameData {
            duration_ms: f.duration_ms,
            sound_index: f.sound_index.map(|i| i as i32).unwrap_or(-1),
            image_count: f.image_count as u32,
        })
    }
}

/// An ACS character file.
#[wasm_bindgen]
pub struct AcsFile {
    inner: Acs,
}

#[wasm_bindgen]
impl AcsFile {
    /// Load an ACS file from a Uint8Array.
    #[wasm_bindgen(constructor)]
    pub fn new(data: &[u8]) -> Result<AcsFile, JsError> {
        let inner = Acs::new(data.to_vec()).map_err(|e| JsError::new(&e.to_string()))?;
        Ok(AcsFile { inner })
    }

    /// Character name.
    #[wasm_bindgen(getter)]
    pub fn name(&self) -> String {
        self.inner.character_info().name.clone()
    }

    /// Character description.
    #[wasm_bindgen(getter)]
    pub fn description(&self) -> String {
        self.inner.character_info().description.clone()
    }

    /// Character width in pixels.
    #[wasm_bindgen(getter)]
    pub fn width(&self) -> u32 {
        self.inner.character_info().width as u32
    }

    /// Character height in pixels.
    #[wasm_bindgen(getter)]
    pub fn height(&self) -> u32 {
        self.inner.character_info().height as u32
    }

    /// List all animation names.
    #[wasm_bindgen(js_name = "animationNames")]
    pub fn animation_names(&self) -> Vec<String> {
        self.inner
            .animation_names()
            .into_iter()
            .map(|s| s.to_string())
            .collect()
    }

    /// Get number of images in the file.
    #[wasm_bindgen(js_name = "imageCount")]
    pub fn image_count(&self) -> usize {
        self.inner.image_count()
    }

    /// Get number of sounds in the file.
    #[wasm_bindgen(js_name = "soundCount")]
    pub fn sound_count(&self) -> usize {
        self.inner.sound_count()
    }

    /// Get a single image by index as RGBA data.
    #[wasm_bindgen(js_name = "getImage")]
    pub fn get_image(&self, index: usize) -> Result<ImageData, JsError> {
        let img = self
            .inner
            .image(index)
            .map_err(|e| JsError::new(&e.to_string()))?;

        Ok(ImageData {
            width: img.width,
            height: img.height,
            data: img.data,
        })
    }

    /// Get animation metadata by name.
    /// Note: This clones the animation data to avoid borrow issues in WASM.
    #[wasm_bindgen(js_name = "getAnimation")]
    pub fn get_animation(&mut self, name: &str) -> Result<AnimationData, JsError> {
        let anim = self
            .inner
            .animation(name)
            .map_err(|e| JsError::new(&e.to_string()))?;

        // Clone the data we need to avoid holding a borrow
        let result = AnimationData {
            name: anim.name.clone(),
            return_animation: anim.return_animation.clone(),
            frames: anim
                .frames
                .iter()
                .map(|f| FrameInfo {
                    duration_ms: f.duration_ms,
                    sound_index: f.sound_index,
                    image_count: f.images.len(),
                })
                .collect(),
        };

        Ok(result)
    }

    /// Render a complete animation frame by compositing all frame images.
    /// Returns RGBA image data at the character's full dimensions.
    #[wasm_bindgen(js_name = "renderFrame")]
    pub fn render_frame(&self, animation: &str, frame_index: usize) -> Result<ImageData, JsError> {
        let img = self
            .inner
            .render_frame(animation, frame_index)
            .map_err(|e| JsError::new(&e.to_string()))?;

        Ok(ImageData {
            width: img.width,
            height: img.height,
            data: img.data,
        })
    }

    /// Get sound data by index as WAV bytes.
    #[wasm_bindgen(js_name = "getSound")]
    pub fn get_sound(&self, index: usize) -> Result<js_sys::Uint8Array, JsError> {
        let sound = self
            .inner
            .sound(index)
            .map_err(|e| JsError::new(&e.to_string()))?;

        Ok(js_sys::Uint8Array::from(&sound.data[..]))
    }
}
