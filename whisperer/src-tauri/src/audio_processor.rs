use symphonia::core::audio::SampleBuffer;
use symphonia::core::codecs::DecoderOptions;
use symphonia::core::formats::FormatOptions;
use symphonia::core::io::MediaSourceStream;
use symphonia::core::probe::Hint;
use rubato::{Resampler, SincFixedIn, SincInterpolationType, SincInterpolationParameters};

pub struct AudioProcessor;

impl AudioProcessor {
    pub fn convert_to_wav(
        input_data: Vec<u8>,
        filename_hint: Option<&str>,
    ) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        // Create cursor from input data
        let cursor = std::io::Cursor::new(input_data);
        let mss = MediaSourceStream::new(Box::new(cursor), Default::default());
        
        // Probe the format
        let mut hint = Hint::new();
        if let Some(filename) = filename_hint {
            if let Some(ext) = std::path::Path::new(filename)
                .extension()
                .and_then(|ext| ext.to_str()) {
                hint.with_extension(ext);
            }
        }
        
        let probe = symphonia::default::get_probe()
            .format(&hint, mss, &FormatOptions::default(), &Default::default())?;
        
        let mut format = probe.format;
        let track = format.default_track()
            .ok_or("No audio track found")?;
        
        // Extract codec parameters before the loop
        let codec_params = track.codec_params.clone();
        let channels = codec_params.channels
            .map(|ch| ch.count() as u32)
            .unwrap_or(1);
            
        let mut decoder = symphonia::default::get_codecs()
            .make(&codec_params, &DecoderOptions::default())?;
            
        // Collect all audio samples
        let mut all_samples = Vec::new();
        let mut sample_rate = 0u32;
        
        // Decode all packets
        while let Ok(packet) = format.next_packet() {
            match decoder.decode(&packet) {
                Ok(decoded) => {
                    let spec = *decoded.spec();
                    sample_rate = spec.rate;
                    let mut sample_buf = SampleBuffer::<f32>::new(
                        decoded.capacity() as u64,
                        spec,
                    );
                    sample_buf.copy_interleaved_ref(decoded);
                    all_samples.extend_from_slice(sample_buf.samples());
                }
                Err(_) => continue,
            }
        }
        
        // Convert to mono and resample to 16kHz
        let mono_samples = Self::convert_to_mono(&all_samples, channels);
        let resampled = Self::resample_to_16khz(&mono_samples, sample_rate)?;
        
        // Convert to 16-bit PCM WAV
        Ok(Self::create_wav_file(&resampled)?)
    }
    
    fn convert_to_mono(samples: &[f32], channels: u32) -> Vec<f32> {
        if channels == 1 {
            return samples.to_vec();
        }
        
        samples.chunks(channels as usize)
            .map(|chunk| chunk.iter().sum::<f32>() / channels as f32)
            .collect()
    }
    
    fn resample_to_16khz(samples: &[f32], source_rate: u32) -> Result<Vec<f32>, Box<dyn std::error::Error>> {
        if source_rate == 16000 {
            return Ok(samples.to_vec());
        }
        
        let params = SincInterpolationParameters {
            sinc_len: 256,
            f_cutoff: 0.95,
            interpolation: SincInterpolationType::Cubic,
            oversampling_factor: 256,
            window: rubato::WindowFunction::BlackmanHarris2,
        };
        
        let mut resampler = SincFixedIn::<f32>::new(
            16000.0 / source_rate as f64,
            2.0,
            params,
            samples.len(),
            1,
        )?;
        
        let waves = vec![samples.to_vec()];
        let mut output = resampler.process(&waves, None)?;
        Ok(output.remove(0))
    }
    
    fn create_wav_file(samples: &[f32]) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let spec = hound::WavSpec {
            channels: 1,
            sample_rate: 16000,
            bits_per_sample: 16,
            sample_format: hound::SampleFormat::Int,
        };
        
        let mut cursor = std::io::Cursor::new(Vec::new());
        {
            let mut writer = hound::WavWriter::new(&mut cursor, spec)?;
            for &sample in samples {
                let sample_i16 = (sample * 32767.0).clamp(-32768.0, 32767.0) as i16;
                writer.write_sample(sample_i16)?;
            }
            writer.finalize()?;
        }
        
        Ok(cursor.into_inner())
    }
}