import { useState } from "react";
import { recordingsApi } from "../api";
import type { Recording } from "../types";

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

  return (
    <div className="recordings-list">
      {recordings.length === 0 ? (
        <p className="empty-state">No recordings yet. Start recording to begin!</p>
      ) : (
        recordings.map((recording) => (
          <div key={recording.id} className="recording-item">
            <div className="recording-header">
              <div className="recording-info">
                <span className="recording-date">
                  {formatDate(recording.created_at)}
                </span>
                {recording.duration_seconds && (
                  <span className="recording-duration">
                    {Math.round(recording.duration_seconds)}s
                  </span>
                )}
              </div>
              
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