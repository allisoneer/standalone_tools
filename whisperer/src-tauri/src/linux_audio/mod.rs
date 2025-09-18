#[cfg(target_os = "linux")]
pub mod linux {
    use crate::audio::{AudioRecorder, RecordingState};
    use async_trait::async_trait;
    use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
    use std::sync::{Arc, Mutex};
    use hound;
    
    // SafeStream wrapper to make cpal::Stream Send + Sync
    // This is safe because:
    // 1. We never access the raw pointers directly
    // 2. All Stream access is synchronized through Arc<Mutex<>>
    // 3. The Stream is created and dropped in controlled contexts
    // 4. CPAL manages the actual audio thread internally
    struct SafeStream(cpal::Stream);
    
    // SAFETY: While cpal::Stream doesn't implement Send due to platform-specific
    // raw pointers, we ensure thread safety by:
    // - Only accessing the stream through synchronized Arc<Mutex<>>
    // - Never exposing the raw stream outside our controlled API
    // - Following the same pattern as production tauri-plugin-mic-recorder
    unsafe impl Send for SafeStream {}
    unsafe impl Sync for SafeStream {}
    
    pub struct LinuxAudioRecorder {
        state: Arc<Mutex<RecordingState>>,
        stream: Arc<Mutex<Option<SafeStream>>>,
        audio_samples: Arc<Mutex<Vec<i16>>>,
        sample_rate: u32,
        preferred_device: Option<String>,
    }

    impl LinuxAudioRecorder {
        pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
            Ok(Self {
                state: Arc::new(Mutex::new(RecordingState::Idle)),
                stream: Arc::new(Mutex::new(None)),
                audio_samples: Arc::new(Mutex::new(Vec::new())),
                sample_rate: 16000, // Fixed 16kHz for Groq optimization
                preferred_device: None,
            })
        }
        
        pub fn with_preferred_device(preferred_device: Option<String>) -> Result<Self, Box<dyn std::error::Error>> {
            Ok(Self {
                state: Arc::new(Mutex::new(RecordingState::Idle)),
                stream: Arc::new(Mutex::new(None)),
                audio_samples: Arc::new(Mutex::new(Vec::new())),
                sample_rate: 16000, // Fixed 16kHz for Groq optimization
                preferred_device,
            })
        }
        
        fn find_working_input_device(&self, host: &cpal::Host) -> Result<cpal::Device, Box<dyn std::error::Error>> {
            // First, try user's preferred device if specified
            if let Some(ref preferred) = self.preferred_device {
                eprintln!("Trying user's preferred device: {}", preferred);
                if let Ok(devices) = host.input_devices() {
                    for device in devices {
                        if let Ok(name) = device.name() {
                            if name == *preferred && device.supported_input_configs().is_ok() {
                                eprintln!("Successfully selected preferred device: {}", name);
                                return Ok(device);
                            }
                        }
                    }
                }
                eprintln!("Preferred device '{}' not available, falling back...", preferred);
            }
            
            // Second, try system default (changed from last resort)
            eprintln!("Trying system default input device...");
            if let Some(device) = host.default_input_device() {
                if device.supported_input_configs().is_ok() {
                    if let Ok(name) = device.name() {
                        eprintln!("Successfully selected system default: {}", name);
                    }
                    return Ok(device);
                }
            }
            
            // Then try preferred backends by name
            let preferred_names = vec![
                "pipewire",  // Modern Linux audio
                "pulse",     // PulseAudio
            ];
            
            for pref_name in &preferred_names {
                eprintln!("Trying to use '{}' device...", pref_name);
                if let Ok(devices) = host.input_devices() {
                    for device in devices {
                        if let Ok(name) = device.name() {
                            if name.to_lowercase().contains(pref_name) {
                                // Test if device actually works by getting configs
                                if device.supported_input_configs().is_ok() {
                                    eprintln!("Successfully selected '{}' device", name);
                                    return Ok(device);
                                } else {
                                    eprintln!("Device '{}' exists but can't get configs", name);
                                }
                            }
                        }
                    }
                }
            }
            
            // If preferred devices don't work, try hardware devices
            eprintln!("Preferred devices not working, trying hardware devices...");
            if let Ok(devices) = host.input_devices() {
                for device in devices {
                    if let Ok(name) = device.name() {
                        // Skip surround sound configs and OSS devices
                        if !name.contains("surround") && !name.contains("oss") {
                            // Test if device works
                            if device.supported_input_configs().is_ok() {
                                eprintln!("Found working hardware device: {}", name);
                                return Ok(device);
                            }
                        }
                    }
                }
            }
            
            Err("No working audio input device found. Please check:\n\
                 1. Your microphone is connected\n\
                 2. You have permission to access audio devices (check 'audio' group)\n\
                 3. No other application is using the microphone\n\
                 4. Try: 'systemctl --user restart pipewire' or 'pulseaudio -k'".into())
        }
    }

    #[async_trait]
    impl AudioRecorder for LinuxAudioRecorder {
        async fn start_recording(&mut self) -> Result<(), Box<dyn std::error::Error>> {
            // Check if already recording
            if *self.state.lock().unwrap() == RecordingState::Recording {
                return Err("Already recording".into());
            }

            // Get audio host and device
            let host = cpal::default_host();
            eprintln!("Using audio host: {:?}", host.id());
            
            // Try to find a working input device
            let device = self.find_working_input_device(&host)?;
            
            let device_name = device.name().unwrap_or_else(|_| "Unknown".to_string());
            eprintln!("Selected input device: {}", device_name);

            // Find suitable config for 16kHz mono
            eprintln!("Supported input configs:");
            for (idx, config) in device.supported_input_configs()?.enumerate() {
                eprintln!("  [{}] channels={}, min_rate={}, max_rate={}, format={:?}", 
                    idx, config.channels(), config.min_sample_rate().0, 
                    config.max_sample_rate().0, config.sample_format());
            }
            
            // Collect all valid configs and sort by preference
            let all_configs: Vec<_> = device.supported_input_configs()?
                .filter(|c| {
                    (c.channels() == 1 || c.channels() == 2) &&
                    c.min_sample_rate().0 <= 16000 && 
                    c.max_sample_rate().0 >= 16000
                })
                .collect();
            
            // Prefer: 1) I16 mono, 2) F32 mono, 3) I16 stereo, 4) F32 stereo, 5) Others
            let config = all_configs.iter()
                .min_by_key(|c| {
                    let format_priority = match c.sample_format() {
                        cpal::SampleFormat::I16 => 0,
                        cpal::SampleFormat::F32 => 1,
                        cpal::SampleFormat::U8 => 2,
                        _ => 3,
                    };
                    let channel_priority = if c.channels() == 1 { 0 } else { 1 };
                    (channel_priority, format_priority)
                })
                .cloned()
                .ok_or("No suitable config found - device must support 16kHz recording")?
                .with_sample_rate(cpal::SampleRate(16000));
                
            eprintln!("Selected config: channels={}, rate={}, format={:?}", 
                config.channels(), config.sample_rate().0, config.sample_format());
            
            let is_stereo = config.channels() == 2;

            // Clear the audio samples buffer
            self.audio_samples.lock().unwrap().clear();
            
            // Clone the samples buffer for the audio callback
            let samples_buffer = self.audio_samples.clone();
            let samples_buffer_f32 = self.audio_samples.clone();
            let samples_buffer_u8 = self.audio_samples.clone();

            // Create error callback
            let err_fn = |err| eprintln!("Error in audio stream: {}", err);

            // Build input stream based on sample format
            let stream = match config.sample_format() {
                cpal::SampleFormat::I16 => {
                    eprintln!("Building i16 input stream (stereo: {})...", is_stereo);
                    device.build_input_stream(
                        &config.into(),
                        move |data: &[i16], _: &_| {
                            if let Ok(mut samples) = samples_buffer.try_lock() {
                                if is_stereo {
                                    // Take only left channel (every other sample)
                                    for i in (0..data.len()).step_by(2) {
                                        samples.push(data[i]);
                                    }
                                } else {
                                    samples.extend_from_slice(data);
                                }
                            }
                        },
                        err_fn,
                        None,
                    ).map_err(|e| format!("Failed to build i16 input stream: {}", e))?
                }
                cpal::SampleFormat::F32 => {
                    eprintln!("Building f32 input stream (stereo: {})...", is_stereo);
                    device.build_input_stream(
                        &config.into(),
                        move |data: &[f32], _: &_| {
                            if let Ok(mut samples) = samples_buffer_f32.try_lock() {
                                if is_stereo {
                                    // Take only left channel (every other sample)
                                    for i in (0..data.len()).step_by(2) {
                                        let sample_i16 = (data[i] * i16::MAX as f32) as i16;
                                        samples.push(sample_i16);
                                    }
                                } else {
                                    for &sample in data {
                                        // Convert f32 to i16
                                        let sample_i16 = (sample * i16::MAX as f32) as i16;
                                        samples.push(sample_i16);
                                    }
                                }
                            }
                        },
                        err_fn,
                        None,
                    ).map_err(|e| format!("Failed to build f32 input stream: {}", e))?
                }
                cpal::SampleFormat::U8 => {
                    eprintln!("Building u8 input stream (stereo: {})...", is_stereo);
                    device.build_input_stream(
                        &config.into(),
                        move |data: &[u8], _: &_| {
                            if let Ok(mut samples) = samples_buffer_u8.try_lock() {
                                if is_stereo {
                                    // Take only left channel (every other sample)
                                    for i in (0..data.len()).step_by(2) {
                                        // Convert u8 to i16: u8 ranges 0-255, with 128 as center
                                        // Map to i16 range: -32768 to 32767
                                        let sample_i16 = ((data[i] as i16 - 128) * 256) as i16;
                                        samples.push(sample_i16);
                                    }
                                } else {
                                    for &sample in data {
                                        // Convert u8 to i16
                                        let sample_i16 = ((sample as i16 - 128) * 256) as i16;
                                        samples.push(sample_i16);
                                    }
                                }
                            }
                        },
                        err_fn,
                        None,
                    ).map_err(|e| format!("Failed to build u8 input stream: {}", e))?
                }
                _ => return Err("Unsupported sample format".into()),
            };

            // Start the stream
            eprintln!("Starting audio stream...");
            stream.play()
                .map_err(|e| format!("Failed to start audio stream: {}", e))?;
            eprintln!("Audio stream started successfully!");

            // Store stream and update state
            *self.stream.lock().unwrap() = Some(SafeStream(stream));
            *self.state.lock().unwrap() = RecordingState::Recording;

            Ok(())
        }

        async fn stop_recording(&mut self) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
            // Update state
            *self.state.lock().unwrap() = RecordingState::Idle;
            
            // Drop the stream to stop recording
            if let Some(safe_stream) = self.stream.lock().unwrap().take() {
                drop(safe_stream.0);
            }
            
            // Get the recorded samples
            let samples = self.audio_samples.lock().unwrap().clone();
            
            if samples.is_empty() {
                return Err("No audio data recorded".into());
            }
            
            // Create WAV file from samples
            let spec = hound::WavSpec {
                channels: 1,
                sample_rate: self.sample_rate,
                bits_per_sample: 16,
                sample_format: hound::SampleFormat::Int,
            };
            
            let mut cursor = std::io::Cursor::new(Vec::new());
            {
                let mut writer = hound::WavWriter::new(&mut cursor, spec)?;
                for sample in samples {
                    writer.write_sample(sample)?;
                }
                writer.finalize()?;
            }
            
            Ok(cursor.into_inner())
        }

        async fn pause_recording(&mut self) -> Result<(), Box<dyn std::error::Error>> {
            if let Some(safe_stream) = self.stream.lock().unwrap().as_ref() {
                safe_stream.0.pause()?;
                *self.state.lock().unwrap() = RecordingState::Paused;
                Ok(())
            } else {
                Err("No recording in progress".into())
            }
        }

        async fn resume_recording(&mut self) -> Result<(), Box<dyn std::error::Error>> {
            if let Some(safe_stream) = self.stream.lock().unwrap().as_ref() {
                safe_stream.0.play()?;
                *self.state.lock().unwrap() = RecordingState::Recording;
                Ok(())
            } else {
                Err("No recording in progress".into())
            }
        }

        fn get_state(&self) -> RecordingState {
            *self.state.lock().unwrap()
        }
    }
}