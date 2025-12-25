import init, { AcsFile, AnimationData, AnimationInfo } from 'acs-web';

// Initialize WASM module
await init();

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
let branchingEnabled = true; // Probabilistic branching enabled by default

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

// Branching toggle handler
const branchToggle = document.getElementById('branch-toggle') as HTMLInputElement | null;
if (branchToggle) {
  branchToggle.checked = branchingEnabled;
  branchToggle.addEventListener('change', () => {
    branchingEnabled = branchToggle.checked;
  });
}

// Enable audio function (called from HTML onclick)
async function enableAudio() {
  if (audioContext && audioContext.state === 'suspended') {
    await audioContext.resume();
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

// Cache animation info for quick hasSound lookups
let animationInfoCache: Map<string, AnimationInfo> = new Map();

// State cache - maps animation name to state name
let animationToState: Map<string, string> = new Map();
let idleStateAnimations: string[] = [];

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

// Animation select handler - use both change and click to allow re-selecting same animation
let lastSelectedAnim = '';
animationSelect.addEventListener('change', async () => {
  const animName = animationSelect.value;
  if (animName && acsFile) {
    // User interaction - try to resume audio
    if (audioContext && audioContext.state === 'suspended') {
      await audioContext.resume();
      audioNotice.classList.add('hidden');
      pendingAnimationWithSound = null;
    }
    lastSelectedAnim = animName;
    playAnimation(animName);
    updateUrl();
  }
});

// Allow clicking the same animation to restart it
animationSelect.addEventListener('click', async (e) => {
  // Only trigger on option click, not dropdown open
  const animName = animationSelect.value;
  if (animName && acsFile && animName === lastSelectedAnim && e.detail > 0) {
    // User interaction - try to resume audio
    if (audioContext && audioContext.state === 'suspended') {
      await audioContext.resume();
      audioNotice.classList.add('hidden');
      pendingAnimationWithSound = null;
    }
    playAnimation(animName);
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
  animationInfoCache.clear();
  animationToState.clear();
  idleStateAnimations = [];

  // Load new file
  acsFile = new AcsFile(data);

  // Update canvas size
  canvas.width = acsFile.width;
  canvas.height = acsFile.height;

  // Get all animation info in one call (much more efficient)
  const animInfoList = acsFile.getAllAnimationInfo();
  const animations = animInfoList.map(info => info.name);

  // Build cache for quick lookups
  for (const info of animInfoList) {
    animationInfoCache.set(info.name, info);
  }

  // Build state cache - map each animation to its state
  // State animation names may be uppercase while actual names are mixed case,
  // so we need to match case-insensitively
  const actualAnimNames = new Map<string, string>();
  for (const name of animations) {
    actualAnimNames.set(name.toLowerCase(), name);
  }

  const states = acsFile.getStates();
  for (const state of states) {
    const stateName = state.name.toLowerCase();
    const stateAnims = state.animations;
    const isIdleState = stateName.includes('idl');
    for (const stateAnimName of stateAnims) {
      // Find the actual animation name (case-insensitive match)
      const actualName = actualAnimNames.get(stateAnimName.toLowerCase());
      if (actualName) {
        animationToState.set(actualName.toLowerCase(), state.name);
        // Collect idle state animations for fallback (state names use "IDLING")
        if (isIdleState) {
          idleStateAnimations.push(actualName);
        }
      }
    }
    state.free();
  }

  // Populate animation dropdown with playable animations only
  // (excludes Return/Continued helper animations)
  const playableNames = new Set(acsFile.playableAnimationNames().map(n => n.toLowerCase()));
  animationSelect.innerHTML = '<option value="">Select animation...</option>';
  for (const info of animInfoList) {
    if (!playableNames.has(info.name.toLowerCase())) continue;
    const option = document.createElement('option');
    option.value = info.name;
    option.textContent = info.hasSound ? `${info.name} *` : info.name;
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
  for (let i = 0; i < soundCount; i++) {
    try {
      // Use getSoundAsArrayBuffer - no manual copy needed
      const arrayBuffer = acsFile.getSoundAsArrayBuffer(i);
      const audioBuffer = await audioContext.decodeAudioData(arrayBuffer);
      soundCache.set(i, audioBuffer);
    } catch (err) {
      console.warn(`Failed to load sound ${i}:`, err);
    }
  }
}

async function playSound(index: number) {
  if (!audioContext || !gainNode || index < 0) return;

  const buffer = soundCache.get(index);
  if (!buffer) return;

  // Resume AudioContext if suspended (browser autoplay policy)
  if (audioContext.state === 'suspended') {
    await audioContext.resume();
  }

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
    const animInfo = animationInfoCache.get(name);
    if (audioContext && audioContext.state === 'suspended' && animInfo?.hasSound) {
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

    // Clear canvas before drawing to prevent old frame bleed-through
    ctx.clearRect(0, 0, canvas.width, canvas.height);
    ctx.putImageData(canvasImageData, 0, 0);

    imageData.free();
  } catch (err) {
    console.error('Failed to render frame:', err);
  }
}

// Find an idle animation to return to when no return animation is specified
// Uses state-based lookup if states are available, falls back to name matching
function findIdleAnimation(): string | null {
  if (!acsFile) return null;

  // First try: use state-based idle animations if available
  if (idleStateAnimations.length > 0) {
    // Pick a random idle animation from the state for variety
    const idx = Math.floor(Math.random() * idleStateAnimations.length);
    return idleStateAnimations[idx];
  }

  // Fallback: match by common idle animation names
  const idleNames = ['RestPose', 'Idle1_1', 'Idle', 'Stand', 'Neutral'];
  const animations = acsFile.animationNames();
  for (const name of idleNames) {
    const match = animations.find(a => a.toLowerCase() === name.toLowerCase());
    if (match) return match;
  }
  return null;
}

// Check if current animation is in an idle state
function isInIdleState(animName: string): boolean {
  const stateName = animationToState.get(animName.toLowerCase());
  if (!stateName) return false;
  const lowerState = stateName.toLowerCase();
  // State names use "IDLING" not "IDLE"
  return lowerState.includes('idl');
}

// Select next frame using probabilistic branching if enabled
function selectNextFrame(): number {
  if (!currentAnimation || !branchingEnabled) {
    return currentFrame + 1;
  }

  const branches = currentAnimation.getFrameBranches(currentFrame);
  if (branches.length === 0) {
    return currentFrame + 1; // No branches, linear progression
  }

  // Calculate total probability weight
  const total = branches.reduce((sum, b) => sum + b.probability, 0);
  if (total === 0) {
    return currentFrame + 1;
  }

  // Roll and select branch based on probability
  const roll = Math.random() * total;
  let cumulative = 0;
  for (const branch of branches) {
    cumulative += branch.probability;
    if (roll < cumulative) {
      return branch.frameIndex;
    }
  }

  // Fallback to last branch (shouldn't reach here normally)
  return branches[branches.length - 1].frameIndex;
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
    playSound(soundIndex);
  }

  animationTimer = window.setTimeout(() => {
    const nextFrame = selectNextFrame();

    if (nextFrame >= currentAnimation!.frameCount) {
      // Animation finished - determine next action based on transition type
      const currentAnimName = currentAnimation!.name;
      const transitionType = currentAnimation!.transitionType;
      const returnAnim = currentAnimation!.returnAnimation;

      if (transitionType.usesReturnAnimation && returnAnim) {
        // Type 0: Use the specified return animation
        playAnimation(returnAnim);
      } else {
        // Type 1 (exit branch) or Type 2 (none): No automatic next animation
        // Fall back to idle state animations
        const inIdleState = isInIdleState(currentAnimName);
        const idleAnim = findIdleAnimation();

        if (idleAnim && !inIdleState) {
          // Not in idle state - transition to idle
          playAnimation(idleAnim);
        } else if (shouldLoop) {
          // Loop current animation
          currentFrame = 0;
          renderCurrentFrame();
          scheduleNextFrame();
        } else if (idleAnim && idleAnim !== currentAnimName) {
          // Pick a different idle animation for variety
          playAnimation(idleAnim);
        }
        // Otherwise animation stops (agent is at rest)
      }
    } else {
      // Continue to next frame (may be a branch target)
      currentFrame = nextFrame;
      renderCurrentFrame();
      scheduleNextFrame();
    }
  }, duration);
}
