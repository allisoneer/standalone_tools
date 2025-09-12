export type RecordingSource = "recorded" | "uploaded";

export interface Recording {
  id: string;
  filename: string;
  duration_seconds?: number;
  created_at: string;
  transcription?: Transcription;
  source: RecordingSource;
  original_filename?: string;
  original_format?: string;
}

export interface Transcription {
  text: string;
  language?: string;
  model_used: string;
  created_at: string;
  metadata?: any;
}

export interface AudioDevice {
  id: string;
  name: string;
  is_default: boolean;
}

export interface AppSettings {
  api_key?: string;
  base_url: string;
  model: string;
  selected_audio_device?: string;
}

export type RecordingState = "Idle" | "Recording" | "Paused";