import { useState, useEffect } from "react";
import { settingsApi, audioApi } from "../api";
import type { AppSettings, AudioDevice } from "../types";

export function Settings({ onClose }: { onClose: () => void }) {
  const [settings, setSettings] = useState<AppSettings>({
    api_key: "",
    base_url: "https://api.groq.com/openai/v1",
    model: "whisper-large-v3-turbo",
    selected_audio_device: undefined,
  });
  const [audioDevices, setAudioDevices] = useState<AudioDevice[]>([]);
  const [isSaving, setIsSaving] = useState(false);
  const [isLoadingDevices, setIsLoadingDevices] = useState(true);

  useEffect(() => {
    loadSettings();
    loadAudioDevices();
  }, []);

  const loadSettings = async () => {
    try {
      const loaded = await settingsApi.get();
      setSettings(loaded);
    } catch (error) {
      console.error("Failed to load settings:", error);
    }
  };

  const loadAudioDevices = async () => {
    try {
      setIsLoadingDevices(true);
      const devices = await audioApi.listDevices();
      setAudioDevices(devices);
    } catch (error) {
      console.error("Failed to load audio devices:", error);
      // Continue without device selection if enumeration fails
    } finally {
      setIsLoadingDevices(false);
    }
  };

  const handleSave = async () => {
    if (!settings.api_key) {
      alert("API key is required");
      return;
    }

    setIsSaving(true);
    try {
      await settingsApi.save(settings);
      alert("Settings saved successfully");
      onClose();
    } catch (error) {
      alert(`Failed to save settings: ${error}`);
    } finally {
      setIsSaving(false);
    }
  };

  return (
    <div className="settings-modal">
      <div className="settings-content">
        <h2>Settings</h2>
        
        <div className="form-group">
          <label htmlFor="api-key">API Key:</label>
          <input
            id="api-key"
            type="password"
            value={settings.api_key || ""}
            onChange={(e) => setSettings({ ...settings, api_key: e.target.value })}
            placeholder="Enter your Groq API key"
          />
          <small>Get your API key from https://console.groq.com/keys</small>
        </div>
        
        <div className="form-group">
          <label htmlFor="base-url">Base URL:</label>
          <input
            id="base-url"
            type="text"
            value={settings.base_url}
            onChange={(e) => setSettings({ ...settings, base_url: e.target.value })}
          />
        </div>
        
        <div className="form-group">
          <label htmlFor="audio-device">Audio Input Device:</label>
          <select
            id="audio-device"
            value={settings.selected_audio_device || "system-default"}
            onChange={(e) => setSettings({ 
              ...settings, 
              selected_audio_device: e.target.value === "system-default" ? undefined : e.target.value 
            })}
            disabled={isLoadingDevices}
          >
            <option value="system-default">System Default</option>
            {audioDevices.map(device => (
              <option key={device.id} value={device.id}>
                {device.name} {device.is_default ? "(System Default)" : ""}
              </option>
            ))}
          </select>
          {isLoadingDevices && <small>Loading audio devices...</small>}
        </div>
        
        <div className="form-group">
          <label htmlFor="model">Model:</label>
          <select
            id="model"
            value={settings.model}
            onChange={(e) => setSettings({ ...settings, model: e.target.value })}
          >
            <option value="whisper-large-v3-turbo">
              Whisper Large V3 Turbo (Faster, $0.04/hour)
            </option>
            <option value="whisper-large-v3">
              Whisper Large V3 (More Accurate, $0.111/hour)
            </option>
          </select>
        </div>
        
        <div className="settings-actions">
          <button onClick={onClose} className="btn-secondary">
            Cancel
          </button>
          <button 
            onClick={handleSave} 
            disabled={isSaving}
            className="btn-primary"
          >
            {isSaving ? "Saving..." : "Save"}
          </button>
        </div>
      </div>
    </div>
  );
}