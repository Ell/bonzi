import { AcsFile, AnimationData } from 'acs-web';

// DOM elements
const agentSelect = document.getElementById('agent-select') as HTMLSelectElement;
const fileInput = document.getElementById('file-input') as HTMLInputElement;
const animationSelect = document.getElementById('animation-select') as HTMLSelectElement;
const loopToggle = document.getElementById('loop-toggle') as HTMLInputElement;
const volumeSlider = document.getElementById('volume-slider') as HTMLInputElement;
const canvas = document.getElementById('canvas') as HTMLCanvasElement;
const ctx = canvas.getContext('2d')!;
const infoDiv = document.getElementById('info')!;

// State
let acsFile: AcsFile | null = null;
let currentAnimation: AnimationData | null = null;
let currentFrame = 0;
let animationTimer: number | null = null;
let shouldLoop = false;
let volume = 0.5;

// Audio cache - preload sounds as AudioBuffers
let audioContext: AudioContext | null = null;
let gainNode: GainNode | null = null;
let soundCache: Map<number, AudioBuffer> = new Map();

// Loop toggle handler
loopToggle.addEventListener('change', () => {
  shouldLoop = loopToggle.checked;
});

// Volume slider handler
volumeSlider.addEventListener('input', () => {
  volume = parseInt(volumeSlider.value) / 100;
  if (gainNode) {
    gainNode.gain.value = volume;
  }
});

// Agent select handler
agentSelect.addEventListener('change', async () => {
  const agentPath = agentSelect.value;
  if (!agentPath) return;

  try {
    const response = await fetch(agentPath);
    if (!response.ok) throw new Error(`Failed to fetch: ${response.status}`);
    const buffer = await response.arrayBuffer();
    fileInput.value = ''; // Clear file input
    loadAcsFile(new Uint8Array(buffer));
  } catch (err) {
    console.error('Failed to load agent:', err);
    alert('Failed to load agent: ' + err);
  }
});

// File input handler
fileInput.addEventListener('change', async (e) => {
  const file = (e.target as HTMLInputElement).files?.[0];
  if (!file) return;

  try {
    const buffer = await file.arrayBuffer();
    agentSelect.value = ''; // Clear agent dropdown
    loadAcsFile(new Uint8Array(buffer));
  } catch (err) {
    console.error('Failed to load file:', err);
    alert('Failed to load ACS file: ' + err);
  }
});

// Animation select handler
animationSelect.addEventListener('change', () => {
  const animName = animationSelect.value;
  if (animName && acsFile) {
    playAnimation(animName);
  }
});

async function loadAcsFile(data: Uint8Array) {
  // Clean up previous
  if (acsFile) {
    stopAnimation();
    acsFile.free();
  }
  soundCache.clear();

  // Load new file
  acsFile = new AcsFile(data);

  // Update canvas size
  canvas.width = acsFile.width;
  canvas.height = acsFile.height;

  // Populate animation dropdown
  const animations = acsFile.animationNames();
  animationSelect.innerHTML = '<option value="">Select animation...</option>';
  for (const name of animations) {
    const option = document.createElement('option');
    option.value = name;

    // Check if animation has any sounds
    let hasSound = false;
    try {
      const anim = acsFile.getAnimation(name);
      for (let i = 0; i < anim.frameCount; i++) {
        const frame = anim.getFrame(i);
        if (frame && frame.soundIndex >= 0) {
          hasSound = true;
          frame.free();
          break;
        }
        frame?.free();
      }
      anim.free();
    } catch (e) {
      // Ignore errors
    }

    option.textContent = hasSound ? `${name} *` : name;
    animationSelect.appendChild(option);
  }
  animationSelect.disabled = false;

  // Update info panel
  document.getElementById('char-name')!.textContent = acsFile.name || '(unnamed)';
  document.getElementById('char-desc')!.textContent = acsFile.description || '(no description)';
  document.getElementById('char-size')!.textContent = `${acsFile.width} x ${acsFile.height}`;
  document.getElementById('char-anims')!.textContent = animations.length.toString();
  document.getElementById('char-images')!.textContent = acsFile.imageCount().toString();
  document.getElementById('char-sounds')!.textContent = acsFile.soundCount().toString();
  infoDiv.classList.remove('hidden');

  // Preload all sounds
  await preloadSounds();

  // Play first animation if available
  if (animations.length > 0) {
    // Try to find a good default animation
    const defaultAnims = ['Greet', 'Show', 'RestPose', 'Idle1_1'];
    let defaultAnim = animations[0];
    for (const name of defaultAnims) {
      if (animations.some(a => a.toLowerCase() === name.toLowerCase())) {
        defaultAnim = animations.find(a => a.toLowerCase() === name.toLowerCase())!;
        break;
      }
    }
    animationSelect.value = defaultAnim;
    playAnimation(defaultAnim);
  }
}

async function preloadSounds() {
  if (!acsFile) return;

  // Initialize AudioContext on first use (requires user interaction)
  if (!audioContext) {
    audioContext = new AudioContext();
    gainNode = audioContext.createGain();
    gainNode.gain.value = volume;
    gainNode.connect(audioContext.destination);
  }

  const soundCount = acsFile.soundCount();
  console.log(`Preloading ${soundCount} sounds...`);
  for (let i = 0; i < soundCount; i++) {
    try {
      const wavData = acsFile.getSound(i);
      // Copy the data to a new ArrayBuffer (required for decodeAudioData)
      const arrayBuffer = new ArrayBuffer(wavData.byteLength);
      new Uint8Array(arrayBuffer).set(wavData);
      const audioBuffer = await audioContext.decodeAudioData(arrayBuffer);
      soundCache.set(i, audioBuffer);
      console.log(`Loaded sound ${i}: ${audioBuffer.duration.toFixed(2)}s`);
    } catch (err) {
      console.warn(`Failed to load sound ${i}:`, err);
    }
  }
  console.log(`Preloaded ${soundCache.size} sounds successfully`);
}

function playSound(index: number) {
  console.log(`playSound(${index}) called`);
  if (!audioContext || !gainNode || index < 0) {
    console.log(`  skipping: audioContext=${!!audioContext}, gainNode=${!!gainNode}, index=${index}`);
    return;
  }

  const buffer = soundCache.get(index);
  if (!buffer) {
    console.log(`  no buffer for sound ${index}`);
    return;
  }

  // Resume AudioContext if suspended (browser autoplay policy)
  if (audioContext.state === 'suspended') {
    console.log('  resuming suspended AudioContext');
    audioContext.resume();
  }

  console.log(`  playing sound ${index}, duration=${buffer.duration.toFixed(2)}s, volume=${volume}`);
  const source = audioContext.createBufferSource();
  source.buffer = buffer;
  source.connect(gainNode);
  source.start();
}

function playAnimation(name: string) {
  if (!acsFile) return;

  stopAnimation();

  try {
    currentAnimation = acsFile.getAnimation(name);
    currentFrame = 0;
    renderCurrentFrame();
    scheduleNextFrame();
  } catch (err) {
    console.error('Failed to play animation:', err);
  }
}

function stopAnimation() {
  if (animationTimer !== null) {
    clearTimeout(animationTimer);
    animationTimer = null;
  }
  currentAnimation = null;
  currentFrame = 0;
}

function renderCurrentFrame() {
  if (!acsFile || !currentAnimation) return;

  try {
    const imageData = acsFile.renderFrame(currentAnimation.name, currentFrame);
    const clampedArray = new Uint8ClampedArray(imageData.data);
    const canvasImageData = new ImageData(clampedArray, imageData.width, imageData.height);

    ctx.clearRect(0, 0, canvas.width, canvas.height);
    ctx.putImageData(canvasImageData, 0, 0);

    imageData.free();
  } catch (err) {
    console.error('Failed to render frame:', err);
  }
}

function scheduleNextFrame() {
  if (!currentAnimation || !acsFile) return;

  const frameData = currentAnimation.getFrame(currentFrame);
  if (!frameData) return;

  const duration = frameData.durationMs || 100; // Default to 100ms if 0
  const soundIndex = frameData.soundIndex;
  frameData.free();

  // Play sound if this frame has one
  if (soundIndex >= 0) {
    console.log(`Frame ${currentFrame} has soundIndex=${soundIndex}`);
    playSound(soundIndex);
  }

  animationTimer = window.setTimeout(() => {
    const nextFrame = currentFrame + 1;

    if (nextFrame >= currentAnimation!.frameCount) {
      // Animation finished - check for return animation
      const returnAnim = currentAnimation!.returnAnimation;

      if (returnAnim) {
        // Update dropdown to show new animation
        animationSelect.value = returnAnim;
        playAnimation(returnAnim);
      } else if (shouldLoop) {
        // Loop current animation
        currentFrame = 0;
        renderCurrentFrame();
        scheduleNextFrame();
      }
      // If not looping and no return animation, animation stops
    } else {
      // Continue to next frame
      currentFrame = nextFrame;
      renderCurrentFrame();
      scheduleNextFrame();
    }
  }, duration);
}
