use async_openai::{
    config::OpenAIConfig,
    types::{AudioInput, CreateTranscriptionRequestArgs, CreateTranscriptionResponseVerboseJson},
    Client,
};
use serde_json::Value;
use std::error::Error;

pub struct TranscriptionService {
    client: Client<OpenAIConfig>,
}

impl TranscriptionService {
    pub fn new(api_key: String, base_url: String) -> Self {
        let config = OpenAIConfig::new()
            .with_api_key(api_key)
            .with_api_base(base_url);
        
        let client = Client::with_config(config);
        
        Self { client }
    }

    // Note: This method is unused but kept for API compatibility
    // The transcribe_with_metadata method below is used instead
    pub async fn transcribe_audio(
        &self,
        audio_data: Vec<u8>,
        filename: String,
        model: String,
    ) -> Result<CreateTranscriptionResponseVerboseJson, Box<dyn Error>> {
        let audio_input = AudioInput::from_vec_u8(filename, audio_data);
        
        let request = CreateTranscriptionRequestArgs::default()
            .file(audio_input)
            .model(model)
            .response_format(async_openai::types::AudioResponseFormat::VerboseJson)
            .temperature(0.0)
            .build()?;

        let response = self.client
            .audio()
            .transcribe_verbose_json(request)
            .await?;

        Ok(response)
    }

    pub async fn transcribe_with_metadata(
        &self,
        audio_data: Vec<u8>,
        filename: String,
        model: String,
        include_timestamps: bool,
    ) -> Result<(String, Value), Box<dyn Error>> {
        let audio_input = AudioInput::from_vec_u8(filename, audio_data);
        
        let request = if include_timestamps {
            CreateTranscriptionRequestArgs::default()
                .file(audio_input)
                .model(model)
                .response_format(async_openai::types::AudioResponseFormat::VerboseJson)
                .temperature(0.0)
                .timestamp_granularities(vec![
                    async_openai::types::TimestampGranularity::Segment,
                    async_openai::types::TimestampGranularity::Word,
                ])
                .build()?
        } else {
            CreateTranscriptionRequestArgs::default()
                .file(audio_input)
                .model(model)
                .response_format(async_openai::types::AudioResponseFormat::VerboseJson)
                .temperature(0.0)
                .build()?
        };
        let response = self.client
            .audio()
            .transcribe_verbose_json(request)
            .await?;

        // Extract text and metadata
        let text = response.text.clone();
        let metadata = serde_json::to_value(&response)?;

        Ok((text, metadata))
    }
}

// TODO: These error types are defined but not currently used
// They were intended for better error handling but the current implementation
// just converts errors to strings. These should be integrated into the command
// handlers for better user-facing error messages.
#[derive(Debug)]
pub struct TranscriptionError {
    pub kind: ErrorKind,
    pub message: String,
}

#[derive(Debug)]
pub enum ErrorKind {
    ApiKeyMissing,
    NetworkError,
    QuotaExceeded,
    InvalidAudio,
    Unknown,
}

impl From<async_openai::error::OpenAIError> for TranscriptionError {
    fn from(error: async_openai::error::OpenAIError) -> Self {
        match error {
            async_openai::error::OpenAIError::ApiError(api_error) => {
                if api_error.message.contains("quota") {
                    TranscriptionError {
                        kind: ErrorKind::QuotaExceeded,
                        message: api_error.message,
                    }
                } else {
                    TranscriptionError {
                        kind: ErrorKind::Unknown,
                        message: api_error.message,
                    }
                }
            }
            _ => TranscriptionError {
                kind: ErrorKind::NetworkError,
                message: error.to_string(),
            },
        }
    }
}