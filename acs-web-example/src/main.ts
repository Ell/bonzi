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
let pendingAnimationWithSound: string | null = null; // Animation to replay when audio is enabled

// Audio notice element
const audioNotice = document.getElementById('audio-notice')!;

// Loop toggle handler
loopToggle.addEventListener('change', () => {
  shouldLoop = loopToggle.checked;
});

// Enable audio function (called from HTML onclick)
async function enableAudio() {
  if (audioContext && audioContext.state === 'suspended') {
    await audioContext.resume();
    console.log('AudioContext resumed after user click');
    audioNotice.classList.add('hidden');

    // Replay the animation that had sound
    if (pendingAnimationWithSound && acsFile) {
      playAnimation(pendingAnimationWithSound);
      pendingAnimationWithSound = null;
    }
  }
}

// Export to window for HTML onclick
(window as any).enableAudio = enableAudio;

// Check if an animation has any sounds
function animationHasSound(animName: string): boolean {
  if (!acsFile) return false;
  try {
    const anim = acsFile.getAnimation(animName);
    for (let i = 0; i < anim.frameCount; i++) {
      const frame = anim.getFrame(i);
      if (frame && frame.soundIndex >= 0) {
        frame.free();
        anim.free();
        return true;
      }
      frame?.free();
    }
    anim.free();
  } catch (e) {
    // Ignore errors
  }
  return false;
}

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

  // User interaction - try to resume audio
  if (audioContext && audioContext.state === 'suspended') {
    await audioContext.resume();
    audioNotice.classList.add('hidden');
    pendingAnimationWithSound = null;
  }

  await loadAgentFromPath(agentPath);
  updateUrl();
});

async function loadAgentFromPath(agentPath: string, initialAnimation?: string) {
  try {
    const response = await fetch(agentPath);
    if (!response.ok) throw new Error(`Failed to fetch: ${response.status}`);
    const buffer = await response.arrayBuffer();
    fileInput.value = ''; // Clear file input
    agentSelect.value = agentPath;
    await loadAcsFile(new Uint8Array(buffer), initialAnimation);
  } catch (err) {
    console.error('Failed to load agent:', err);
    alert('Failed to load agent: ' + err);
  }
}

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
animationSelect.addEventListener('change', async () => {
  const animName = animationSelect.value;
  if (animName && acsFile) {
    // User interaction - try to resume audio
    if (audioContext && audioContext.state === 'suspended') {
      await audioContext.resume();
      audioNotice.classList.add('hidden');
      pendingAnimationWithSound = null;
    }
    playAnimation(animName);
    updateUrl();
  }
});

// Update URL with current state
function updateUrl() {
  const params = new URLSearchParams();
  if (agentSelect.value) {
    // Extract agent name from path (e.g., "agents/Bonzi.acs" -> "Bonzi")
    const agentName = agentSelect.value.replace('agents/', '').replace('.acs', '');
    params.set('agent', agentName);
  }
  if (animationSelect.value) {
    params.set('anim', animationSelect.value);
  }
  const newUrl = params.toString() ? `?${params.toString()}` : window.location.pathname;
  window.history.replaceState({}, '', newUrl);
}

// Load from URL on page load
async function loadFromUrl() {
  const params = new URLSearchParams(window.location.search);
  const agent = params.get('agent');
  const anim = params.get('anim');

  if (agent) {
    // Find matching agent path
    const options = Array.from(agentSelect.options);
    const match = options.find(opt =>
      opt.value.toLowerCase().includes(agent.toLowerCase())
    );
    if (match && match.value) {
      // Pass the animation to loadAgentFromPath so it plays after sounds are loaded
      await loadAgentFromPath(match.value, anim || undefined);
    }
  }
}

// Initialize from URL
loadFromUrl();

async function loadAcsFile(data: Uint8Array, initialAnimation?: string) {
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

  // Determine which animation to play
  let animToPlay: string | null = null;

  if (initialAnimation && animations.includes(initialAnimation)) {
    // Use the specified initial animation
    animToPlay = initialAnimation;
  } else if (animations.length > 0) {
    // Try to find a good default animation
    const defaultAnims = ['Greet', 'Show', 'RestPose', 'Idle1_1'];
    animToPlay = animations[0];
    for (const name of defaultAnims) {
      if (animations.some(a => a.toLowerCase() === name.toLowerCase())) {
        animToPlay = animations.find(a => a.toLowerCase() === name.toLowerCase())!;
        break;
      }
    }
  }

  if (animToPlay) {
    animationSelect.value = animToPlay;
    playAnimation(animToPlay);
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

async function playSound(index: number) {
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
    await audioContext.resume();
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

    // Check if audio is blocked and this animation has sound
    if (audioContext && audioContext.state === 'suspended' && animationHasSound(name)) {
      pendingAnimationWithSound = name;
      audioNotice.classList.remove('hidden');
    }

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
