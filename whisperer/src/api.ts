import { invoke } from "@tauri-apps/api/core";
import type { Recording, AppSettings, RecordingState, AudioDevice } from "./types";

export const audioApi = {
  startRecording: () => invoke("start_recording"),
  stopRecording: () => invoke<Recording>("stop_recording"),
  pauseRecording: () => invoke("pause_recording"),
  resumeRecording: () => invoke("resume_recording"),
  getRecordingState: () => invoke<RecordingState>("get_recording_state"),
  listDevices: () => invoke<AudioDevice[]>("list_audio_devices"),
  
  async uploadFile(fileData: ArrayBuffer, filename: string): Promise<Recording> {
    const uint8Array = new Uint8Array(fileData);
    return invoke<Recording>("upload_audio_file", {
      fileData: Array.from(uint8Array),
      originalFilename: filename,
    });
  },
  
  async getMaxUploadSize(): Promise<number> {
    return invoke<number>("get_max_upload_size");
  },
};

export const recordingsApi = {
  list: () => invoke<Recording[]>("list_recordings"),
  transcribe: (recordingId: string) => 
    invoke<Recording>("transcribe_recording", { recordingId }),
  delete: (recordingId: string) => 
    invoke("delete_recording", { recordingId }),
};

export const settingsApi = {
  get: () => invoke<AppSettings>("get_settings"),
  save: (settings: AppSettings) => 
    invoke("save_settings", { settings }),
};