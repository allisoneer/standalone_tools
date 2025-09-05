import { useState, useEffect } from "react";
import { audioApi } from "../api";
import type { RecordingState } from "../types";

export function RecordingControls({ onRecordingComplete }: { 
  onRecordingComplete: () => void 
}) {
  const [state, setState] = useState<RecordingState>("Idle");
  const [isLoading, setIsLoading] = useState(false);

  useEffect(() => {
    // Poll recording state
    const interval = setInterval(async () => {
      try {
        const currentState = await audioApi.getRecordingState();
        setState(currentState);
      } catch (error) {
        console.error("Failed to get recording state:", error);
      }
    }, 1000);

    return () => clearInterval(interval);
  }, []);

  const handleStart = async () => {
    setIsLoading(true);
    try {
      await audioApi.startRecording();
      setState("Recording");
    } catch (error) {
      alert(`Failed to start recording: ${error}`);
    } finally {
      setIsLoading(false);
    }
  };

  const handleStop = async () => {
    setIsLoading(true);
    try {
      await audioApi.stopRecording();
      setState("Idle");
      onRecordingComplete();
    } catch (error) {
      alert(`Failed to stop recording: ${error}`);
    } finally {
      setIsLoading(false);
    }
  };

  const handlePause = async () => {
    setIsLoading(true);
    try {
      await audioApi.pauseRecording();
      setState("Paused");
    } catch (error) {
      alert(`Failed to pause recording: ${error}`);
    } finally {
      setIsLoading(false);
    }
  };

  const handleResume = async () => {
    setIsLoading(true);
    try {
      await audioApi.resumeRecording();
      setState("Recording");
    } catch (error) {
      alert(`Failed to resume recording: ${error}`);
    } finally {
      setIsLoading(false);
    }
  };

  return (
    <div className="recording-controls">
      {state === "Idle" && (
        <button 
          onClick={handleStart} 
          disabled={isLoading}
          className="btn-primary"
        >
          Start Recording
        </button>
      )}
      
      {state === "Recording" && (
        <div className="controls-group">
          <button 
            onClick={handlePause} 
            disabled={isLoading}
            className="btn-secondary"
          >
            Pause
          </button>
          <button 
            onClick={handleStop} 
            disabled={isLoading}
            className="btn-danger"
          >
            Stop
          </button>
        </div>
      )}
      
      {state === "Paused" && (
        <div className="controls-group">
          <button 
            onClick={handleResume} 
            disabled={isLoading}
            className="btn-secondary"
          >
            Resume
          </button>
          <button 
            onClick={handleStop} 
            disabled={isLoading}
            className="btn-danger"
          >
            Stop
          </button>
        </div>
      )}
      
      <div className="status">
        Status: {state}
        {state === "Recording" && <span className="recording-indicator">●</span>}
      </div>
    </div>
  );
}