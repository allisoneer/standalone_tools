import { useState } from "react";
import { recordingsApi } from "../api";
import type { Recording } from "../types";

// Icon components
const MicrophoneIcon = () => (
  <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
    <path d="M12 1a3 3 0 0 0-3 3v8a3 3 0 0 0 6 0V4a3 3 0 0 0-3-3z" />
    <path d="M19 10v2a7 7 0 0 1-14 0v-2" />
    <line x1="12" y1="19" x2="12" y2="23" />
    <line x1="8" y1="23" x2="16" y2="23" />
  </svg>
);

const FileIcon = () => (
  <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
    <path d="M13 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V9z" />
    <polyline points="13 2 13 9 20 9" />
  </svg>
);

export function RecordingsList({ 
  recordings, 
  onUpdate 
}: { 
  recordings: Recording[];
  onUpdate: () => void;
}) {
  const [expandedId, setExpandedId] = useState<string | null>(null);
  const [transcribingId, setTranscribingId] = useState<string | null>(null);

  const handleTranscribe = async (recording: Recording) => {
    setTranscribingId(recording.id);
    try {
      await recordingsApi.transcribe(recording.id);
      onUpdate();
    } catch (error) {
      alert(`Transcription failed: ${error}`);
    } finally {
      setTranscribingId(null);
    }
  };

  const handleDelete = async (recording: Recording) => {
    if (confirm(`Delete recording "${recording.id}"?`)) {
      try {
        await recordingsApi.delete(recording.id);
        onUpdate();
      } catch (error) {
        alert(`Failed to delete: ${error}`);
      }
    }
  };

  const formatDate = (dateString: string) => {
    return new Date(dateString).toLocaleString();
  };

  const formatDuration = (seconds: number): string => {
    const mins = Math.floor(seconds / 60);
    const secs = Math.floor(seconds % 60);
    return `${mins}:${secs.toString().padStart(2, '0')}`;
  };

  const formatFileInfo = (recording: Recording): string => {
    if (recording.source === 'uploaded' && recording.original_filename) {
      const name = recording.original_filename;
      const maxLength = 30;
      if (name.length > maxLength) {
        return `${name.substring(0, maxLength - 3)}...`;
      }
      return name;
    }
    return '';
  };

  return (
    <div className="recordings-list">
      {recordings.length === 0 ? (
        <p className="empty-state">No recordings yet. Start recording to begin!</p>
      ) : (
        recordings.map((recording) => (
          <div key={recording.id} className="recording-item">
            <div className="recording-header">
              <div className="recording-info">
                <div className="recording-icon">
                  {recording.source === 'uploaded' ? <FileIcon /> : <MicrophoneIcon />}
                </div>
                <div className="recording-details">
                  <span className="recording-date">
                    {formatDate(recording.created_at)}
                  </span>
                  {recording.source === 'uploaded' && recording.original_format && (
                    <span className="recording-format">{recording.original_format.toUpperCase()}</span>
                  )}
                  {recording.duration_seconds && (
                    <span className="recording-duration">
                      {formatDuration(recording.duration_seconds)}
                    </span>
                  )}
                </div>
              </div>
              {recording.source === 'uploaded' && recording.original_filename && (
                <div className="recording-filename" title={recording.original_filename}>
                  {formatFileInfo(recording)}
                </div>
              )}
              
              <div className="recording-actions">
                {!recording.transcription && (
                  <button
                    onClick={() => handleTranscribe(recording)}
                    disabled={transcribingId === recording.id}
                    className="btn-small"
                  >
                    {transcribingId === recording.id ? "Transcribing..." : "Transcribe"}
                  </button>
                )}
                
                <button
                  onClick={() => setExpandedId(
                    expandedId === recording.id ? null : recording.id
                  )}
                  className="btn-small"
                  disabled={!recording.transcription}
                >
                  {expandedId === recording.id ? "Hide" : "Show"} Text
                </button>
                
                <button
                  onClick={() => handleDelete(recording)}
                  className="btn-small btn-danger"
                >
                  Delete
                </button>
              </div>
            </div>
            
            {expandedId === recording.id && recording.transcription && (
              <div className="transcription-content">
                <div className="transcription-text">
                  {recording.transcription.text}
                </div>
                <div className="transcription-meta">
                  {recording.transcription.language && (
                    <span>Language: {recording.transcription.language}</span>
                  )}
                  <span>Model: {recording.transcription.model_used}</span>
                </div>
              </div>
            )}
          </div>
        ))
      )}
    </div>
  );
}