# Whisperer

Desktop audio transcription app powered by Groq's Whisper API. Record audio from your microphone or upload audio files for fast, accurate speech-to-text transcription.

## Features

- **Record Audio** - Direct microphone recording with pause/resume support
- **Upload Files** - Drag & drop or select audio files (MP3, M4A, AAC, WAV, OGG, FLAC)
- **Transcribe** - Fast speech-to-text using Groq's Whisper models:
  - `whisper-large-v3-turbo` - Faster, lower cost ($0.04/hour)
  - `whisper-large-v3` - More accurate ($0.111/hour)
- **Manage Recordings** - View all recordings with metadata, transcribe on-demand, delete when done
- **Configure Settings** - Set Groq API key, select audio device, choose transcription model

## Technical Details

**Stack**: Tauri (Rust backend) + React (TypeScript frontend)

**Requirements**:
- Groq API key (get one at https://console.groq.com)
- Bun (JavaScript runtime)
- Rust toolchain

**Quick Start**:
```bash
cd whisperer
bun install
bun run tauri dev
```

**Build for Production**:
```bash
bun run tauri build --bundles deb,appimage
```

**Platform Support**:
- ✅ Linux - Full audio recording and transcription support
- ⚠️ Android - Partial support (plugin integration needed)
- ❌ Windows/macOS - Not yet supported

**Audio Details**:
- Records at 16kHz mono for optimal Groq compatibility
- Automatically converts uploaded files to WAV format
- 25MB maximum file size for uploads
- Supports multiple audio formats through Symphonia

## License

MIT License

Copyright (c) 2024 Allison

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.