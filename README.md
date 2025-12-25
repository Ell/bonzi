# bonzi

Rust parser and web viewer for Microsoft Agent character files (.acs).

## Structure

- `acs/` - Core Rust library for parsing ACS files
- `acs-web/` - WASM bindings for browser use
- `acs-web-example/` - Web viewer demo

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
