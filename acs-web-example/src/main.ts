import { AcsFile, AnimationData } from 'acs-web';

// DOM elements
const fileInput = document.getElementById('file-input') as HTMLInputElement;
const animationSelect = document.getElementById('animation-select') as HTMLSelectElement;
const canvas = document.getElementById('canvas') as HTMLCanvasElement;
const ctx = canvas.getContext('2d')!;
const infoDiv = document.getElementById('info')!;

// State
let acsFile: AcsFile | null = null;
let currentAnimation: AnimationData | null = null;
let currentFrame = 0;
let animationTimer: number | null = null;

// File input handler
fileInput.addEventListener('change', async (e) => {
  const file = (e.target as HTMLInputElement).files?.[0];
  if (!file) return;

  try {
    const buffer = await file.arrayBuffer();
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

function loadAcsFile(data: Uint8Array) {
  // Clean up previous
  if (acsFile) {
    stopAnimation();
    acsFile.free();
  }

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
    option.textContent = name;
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
  if (!currentAnimation) return;

  const frameData = currentAnimation.getFrame(currentFrame);
  if (!frameData) return;

  const duration = frameData.durationMs || 100; // Default to 100ms if 0
  frameData.free();

  animationTimer = window.setTimeout(() => {
    currentFrame = (currentFrame + 1) % currentAnimation!.frameCount;
    renderCurrentFrame();
    scheduleNextFrame();
  }, duration);
}
