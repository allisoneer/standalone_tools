import { invoke } from "@tauri-apps/api/core";
import type { Recording, AppSettings, RecordingState } from "./types";

export const audioApi = {
  startRecording: () => invoke("start_recording"),
  stopRecording: () => invoke<Recording>("stop_recording"),
  pauseRecording: () => invoke("pause_recording"),
  resumeRecording: () => invoke("resume_recording"),
  getRecordingState: () => invoke<RecordingState>("get_recording_state"),
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