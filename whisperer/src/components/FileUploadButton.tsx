import React, { useRef } from 'react';
import { audioApi } from '../api';

interface FileUploadButtonProps {
  onUploadComplete: () => void;
  disabled?: boolean;
}

const ACCEPTED_FORMATS = ['.mp3', '.m4a', '.aac', '.wav', '.ogg', '.flac'];

const ERROR_MESSAGES: Record<string, string> = {
  UNSUPPORTED_FORMAT: "This audio format isn't supported. Try MP3, WAV, M4A, OGG, or FLAC.",
  FILE_TOO_LARGE: "File is too large. Maximum size is 25MB.",
  CONVERSION_FAILED: "Couldn't process this audio file. Try a different format.",
  NETWORK_ERROR: "Upload failed. Please check your connection.",
};

const getErrorMessage = (error: any): string => {
  const errorString = error.toString();
  for (const [key, message] of Object.entries(ERROR_MESSAGES)) {
    if (errorString.includes(key)) {
      return message;
    }
  }
  return errorString;
};

export function FileUploadButton({ onUploadComplete, disabled }: FileUploadButtonProps) {
  const fileInputRef = useRef<HTMLInputElement>(null);
  const [isUploading, setIsUploading] = React.useState(false);

  const handleFileSelect = async (file: File) => {
    if (isUploading) return;

    // Check file size
    const maxSize = await audioApi.getMaxUploadSize();
    if (file.size > maxSize) {
      alert(`File too large. Maximum size is ${Math.round(maxSize / 1024 / 1024)}MB`);
      return;
    }

    setIsUploading(true);
    try {
      const buffer = await file.arrayBuffer();
      await audioApi.uploadFile(buffer, file.name);
      onUploadComplete();
      
      // Clear file input
      if (fileInputRef.current) {
        fileInputRef.current.value = '';
      }
    } catch (error) {
      console.error('Upload failed:', error);
      alert(getErrorMessage(error));
    } finally {
      setIsUploading(false);
    }
  };

  const handleChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    const file = e.target.files?.[0];
    if (file) {
      handleFileSelect(file);
    }
  };

  return (
    <label className={`btn btn-secondary ${disabled || isUploading ? 'disabled' : ''}`}>
      <input
        ref={fileInputRef}
        type="file"
        accept={ACCEPTED_FORMATS.join(',')}
        onChange={handleChange}
        disabled={disabled || isUploading}
        style={{ display: 'none' }}
      />
      {isUploading ? 'Uploading...' : 'Upload Audio File'}
    </label>
  );
}