export interface Recording {
  id: string;
  filename: string;
  duration_seconds?: number;
  created_at: string;
  transcription?: Transcription;
}

export interface Transcription {
  text: string;
  language?: string;
  model_used: string;
  created_at: string;
  metadata?: any;
}

export interface AppSettings {
  api_key?: string;
  base_url: string;
  model: string;
}

export type RecordingState = "Idle" | "Recording" | "Paused";