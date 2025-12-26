# bonzi

Rust tools for Microsoft Agent character files (.acs) and SAPI4 text-to-speech.

## Structure

- `acs/` - Rust library for parsing ACS files
- `acs-web/` - WASM bindings for browser
- `acs-web-example/` - Web viewer demo
- `sapi4-rs/` - Rust SAPI4 TTS binary (cross-compiled for Windows)
- `sapi4-api/` - Docker HTTP API for TTS
- `sapi4-tts/` - Docker container for running sapi4-rs

## Building

```bash
# Build the WASM package
cd acs-web
wasm-pack build --target web

# Run the web viewer
cd acs-web-example
bun install
bun run dev
```

## Usage

```typescript
import init, { AcsFile } from 'acs-web';

await init();

const response = await fetch('Bonzi.acs');
const data = new Uint8Array(await response.arrayBuffer());
const acs = new AcsFile(data);

console.log(acs.name, acs.width, acs.height);
console.log(acs.animationNames());
```

## ACS Format

ACS files contain animated characters with:
- Compressed sprite images (RLE + LZ77)
- Sound effects (WAV)
- Animation sequences with branching and transitions
- Character states for grouping related animations

See `notes/NOTES.md` for format documentation.

## Demo

https://acs-viewer.pages.dev
