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
    #[wasm_bindgen(readonly, js_name = "branchCount")]
    pub branch_count: u32,
}

/// A branch option for probabilistic frame transitions.
#[wasm_bindgen]
pub struct BranchData {
    #[wasm_bindgen(readonly, js_name = "frameIndex")]
    pub frame_index: u32,
    #[wasm_bindgen(readonly)]
    pub probability: u16,
}

/// How an animation transitions when complete.
/// 0 = UseReturnAnimation, 1 = UseExitBranch, 2 = None
#[wasm_bindgen]
#[derive(Clone, Copy)]
pub struct TransitionType(u8);

#[wasm_bindgen]
impl TransitionType {
    /// Type 0: Play the return_animation when complete
    #[wasm_bindgen(getter, js_name = "usesReturnAnimation")]
    pub fn uses_return_animation(&self) -> bool {
        self.0 == 0
    }

    /// Type 1: Uses exit branches (for graceful interruption)
    #[wasm_bindgen(getter, js_name = "usesExitBranch")]
    pub fn uses_exit_branch(&self) -> bool {
        self.0 == 1
    }

    /// Type 2: No automatic transition
    #[wasm_bindgen(getter, js_name = "isNone")]
    pub fn is_none(&self) -> bool {
        self.0 == 2
    }
}

/// Animation metadata.
#[wasm_bindgen]
pub struct AnimationData {
    name: String,
    return_animation: Option<String>,
    transition_type: TransitionType,
    frames: Vec<FrameInfo>,
}

struct FrameInfo {
    duration_ms: u32,
    sound_index: Option<usize>,
    image_count: usize,
    branches: Vec<BranchInfo>,
}

struct BranchInfo {
    frame_index: usize,
    probability: u16,
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

    /// How this animation transitions when complete.
    #[wasm_bindgen(getter, js_name = "transitionType")]
    pub fn transition_type(&self) -> TransitionType {
        self.transition_type
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
            branch_count: f.branches.len() as u32,
        })
    }

    /// Get branches for a frame by index.
    #[wasm_bindgen(js_name = "getFrameBranches")]
    pub fn get_frame_branches(&self, index: usize) -> Vec<BranchData> {
        self.frames
            .get(index)
            .map(|f| {
                f.branches
                    .iter()
                    .map(|b| BranchData {
                        frame_index: b.frame_index as u32,
                        probability: b.probability,
                    })
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Check if any frame in this animation has an associated sound.
    #[wasm_bindgen(getter, js_name = "hasSound")]
    pub fn has_sound(&self) -> bool {
        self.frames.iter().any(|f| f.sound_index.is_some())
    }
}

/// Summary information about an animation (lightweight, no cleanup needed).
#[wasm_bindgen]
pub struct AnimationInfo {
    name: String,
    frame_count: usize,
    has_sound: bool,
    return_animation: Option<String>,
}

/// A character state grouping animations.
#[wasm_bindgen]
pub struct StateInfo {
    name: String,
    animations: Vec<String>,
}

#[wasm_bindgen]
impl StateInfo {
    /// State name (e.g., "Idle", "Speaking", "Greeting").
    #[wasm_bindgen(getter)]
    pub fn name(&self) -> String {
        self.name.clone()
    }

    /// List of animation names in this state.
    #[wasm_bindgen(getter)]
    pub fn animations(&self) -> Vec<String> {
        self.animations.clone()
    }
}

#[wasm_bindgen]
impl AnimationInfo {
    /// Animation name.
    #[wasm_bindgen(getter)]
    pub fn name(&self) -> String {
        self.name.clone()
    }

    /// Number of frames in this animation.
    #[wasm_bindgen(getter, js_name = "frameCount")]
    pub fn frame_count(&self) -> usize {
        self.frame_count
    }

    /// Whether any frame in this animation has an associated sound.
    #[wasm_bindgen(getter, js_name = "hasSound")]
    pub fn has_sound(&self) -> bool {
        self.has_sound
    }

    /// Name of the animation to return to after this one completes.
    #[wasm_bindgen(getter, js_name = "returnAnimation")]
    pub fn return_animation(&self) -> Option<String> {
        self.return_animation.clone()
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

    /// List animation names suitable for direct playback.
    /// Excludes helper animations (Return/Continued variants) that are meant to be chained automatically.
    #[wasm_bindgen(js_name = "playableAnimationNames")]
    pub fn playable_animation_names(&self) -> Vec<String> {
        self.inner
            .animation_names()
            .into_iter()
            .filter(|name| {
                let lower = name.to_lowercase();
                !lower.ends_with("return") && !lower.ends_with("continued")
            })
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
        let transition_type = match anim.transition_type {
            acs::TransitionType::UseReturnAnimation => TransitionType(0),
            acs::TransitionType::UseExitBranch => TransitionType(1),
            acs::TransitionType::None => TransitionType(2),
        };

        let result = AnimationData {
            name: anim.name.clone(),
            return_animation: anim.return_animation.clone(),
            transition_type,
            frames: anim
                .frames
                .iter()
                .map(|f| FrameInfo {
                    duration_ms: f.duration_ms,
                    sound_index: f.sound_index,
                    image_count: f.images.len(),
                    branches: f
                        .branches
                        .iter()
                        .map(|b| BranchInfo {
                            frame_index: b.frame_index,
                            probability: b.probability,
                        })
                        .collect(),
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

    /// Get sound data by index as ArrayBuffer (suitable for decodeAudioData).
    #[wasm_bindgen(js_name = "getSoundAsArrayBuffer")]
    pub fn get_sound_as_array_buffer(&self, index: usize) -> Result<js_sys::ArrayBuffer, JsError> {
        let sound = self
            .inner
            .sound(index)
            .map_err(|e| JsError::new(&e.to_string()))?;

        let buffer = js_sys::ArrayBuffer::new(sound.data.len() as u32);
        let view = js_sys::Uint8Array::new(&buffer);
        view.copy_from(&sound.data);
        Ok(buffer)
    }

    /// Get summary info for all animations (useful for building UI lists).
    #[wasm_bindgen(js_name = "getAllAnimationInfo")]
    pub fn get_all_animation_info(&mut self) -> Vec<AnimationInfo> {
        let names: Vec<String> = self
            .inner
            .animation_names()
            .iter()
            .map(|s| s.to_string())
            .collect();

        names
            .into_iter()
            .filter_map(|name| {
                let anim = self.inner.animation(&name).ok()?;
                let has_sound = anim.frames.iter().any(|f| f.sound_index.is_some());
                Some(AnimationInfo {
                    name: anim.name.clone(),
                    frame_count: anim.frames.len(),
                    has_sound,
                    return_animation: anim.return_animation.clone(),
                })
            })
            .collect()
    }

    /// Get all character states (animation groupings).
    #[wasm_bindgen(js_name = "getStates")]
    pub fn get_states(&self) -> Vec<StateInfo> {
        self.inner
            .states()
            .iter()
            .map(|s| StateInfo {
                name: s.name.clone(),
                animations: s.animations.clone(),
            })
            .collect()
    }
}
