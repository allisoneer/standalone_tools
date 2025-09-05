# Whisperer Implementation TODOs

This document lists all known incomplete features and technical debt in the Whisperer implementation.

## Critical Issues

### 1. Linux Audio Recording (Placeholder Only)
**Location**: `src-tauri/src/linux_audio/mod.rs`
**Issue**: CPAL Stream type doesn't implement Send/Sync, preventing proper thread management
**Current State**: Returns empty WAV files
**Solution**: Implement dedicated audio thread that owns the Stream for its lifetime

### 2. Android Plugin Not Connected
**Location**: `src-tauri/src/android_audio/mod.rs`
**Issue**: Kotlin code exists but isn't packaged as a Tauri plugin
**Current State**: Will fail at runtime when trying to register plugin
**Solution**: Create proper tauri-plugin-audio package structure

## Minor Issues

### 3. Audio Duration Not Calculated
**Location**: `src-tauri/src/commands.rs:39`
**Issue**: Recording duration is always None
**Solution**: Calculate from WAV data: `(audio_data.len() - 44) / 2 / 16000`

### 4. Transcription Service Lazy Initialization
**Location**: `src-tauri/src/lib.rs:49`
**Issue**: Not initialized on startup due to AppState clone constraints
**Impact**: First transcription slightly slower
**Solution**: Create initialization command for frontend to call

### 5. Code Duplication
**Location**: `src-tauri/src/main.rs` and `src-tauri/src/lib.rs`
**Issue**: Same initialization logic in both files
**Solution**: Refactor into shared function

## Missing Features

### 6. No Automated Tests
**Impact**: No unit or integration tests exist
**Needed**:
- Storage manager tests
- Settings persistence tests
- Audio format validation tests
- Command handler tests

### 7. Missing Error Handling
**Location**: Various commands
**Issue**: Errors shown as strings to user, not user-friendly
**Solution**: Implement proper error types with user-facing messages

### 8. No Audio Visualization
**Location**: Frontend
**Issue**: No waveform or recording level indicator
**Solution**: Implement Web Audio API visualization

### 9. No Export Functionality
**Location**: Frontend/Backend
**Issue**: Can't export recordings or transcriptions
**Solution**: Add export commands and UI

## Platform-Specific Notes

### Linux
- Requires PulseAudio or ALSA
- CPAL thread safety prevents full implementation
- Consider using GStreamer as alternative

### Android
- Requires RECORD_AUDIO and MODIFY_AUDIO_SETTINGS permissions
- MediaRecorder implementation complete in Kotlin
- Needs proper plugin packaging

### Windows/macOS
- Not supported (compile error)
- Would need platform-specific audio implementations

## Build Commands

```bash
# Development
cd whisperer
bun run tauri dev

# Linux Build
bun run tauri build --bundles deb,appimage

# Android Build (will fail due to missing plugin)
bun run tauri android build
```

## Testing Checklist

When implementation is complete, test:
1. [ ] Linux: Audio recording creates non-empty WAV files
2. [ ] Android: Permissions requested and audio records
3. [ ] API key persistence across restarts
4. [ ] Transcription with valid Groq API key
5. [ ] Error handling for invalid API key
6. [ ] Pause/resume functionality
7. [ ] Recording deletion
8. [ ] Duration calculation
9. [ ] Multiple recordings management
10. [ ] Settings changes take effect immediately
