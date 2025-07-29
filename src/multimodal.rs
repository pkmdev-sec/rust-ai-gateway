use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum ModalityType {
    Text,
    Image,
    Audio,
    Video,
    Code,
    Document,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ImageFormat {
    Jpeg,
    Png,
    Webp,
    Gif,
    Svg,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AudioFormat {
    Mp3,
    Wav,
    Flac,
    Ogg,
    M4a,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum VideoFormat {
    Mp4,
    Avi,
    Mov,
    Webm,
    Mkv,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediaContent {
    pub modality: ModalityType,
    pub data: MediaData,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MediaData {
    Text { content: String },
    Image { 
        format: ImageFormat,
        data: Vec<u8>,
        width: Option<u32>,
        height: Option<u32>,
    },
    Audio {
        format: AudioFormat,
        data: Vec<u8>,
        duration_ms: Option<u64>,
        sample_rate: Option<u32>,
    },
    Video {
        format: VideoFormat,
        data: Vec<u8>,
        duration_ms: Option<u64>,
        width: Option<u32>,
        height: Option<u32>,
        fps: Option<f32>,
    },
    Base64 {
        mime_type: String,
        data: String,
    },
    Url {
        url: String,
        mime_type: Option<String>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultiModalRequest {
    pub contents: Vec<MediaContent>,
    pub model_capabilities: ModelCapabilities,
    pub processing_options: ProcessingOptions,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelCapabilities {
    pub supported_modalities: Vec<ModalityType>,
    pub max_image_size: Option<u64>, // in bytes
    pub max_audio_duration: Option<u64>, // in milliseconds
    pub max_video_duration: Option<u64>, // in milliseconds
    pub supported_image_formats: Vec<ImageFormat>,
    pub supported_audio_formats: Vec<AudioFormat>,
    pub supported_video_formats: Vec<VideoFormat>,
    pub max_total_size: Option<u64>, // in bytes
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessingOptions {
    pub resize_images: Option<ImageResizeOptions>,
    pub compress_audio: Option<AudioCompressionOptions>,
    pub extract_frames: Option<VideoFrameExtractionOptions>,
    pub ocr_enabled: bool,
    pub speech_to_text: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageResizeOptions {
    pub max_width: u32,
    pub max_height: u32,
    pub maintain_aspect_ratio: bool,
    pub quality: Option<u8>, // 1-100 for JPEG
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioCompressionOptions {
    pub target_bitrate: Option<u32>,
    pub target_format: Option<AudioFormat>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VideoFrameExtractionOptions {
    pub frame_count: u32,
    pub start_time_ms: Option<u64>,
    pub end_time_ms: Option<u64>,
}

impl Default for ModelCapabilities {
    fn default() -> Self {
        Self {
            supported_modalities: vec![ModalityType::Text],
            max_image_size: Some(10 * 1024 * 1024), // 10MB
            max_audio_duration: Some(10 * 60 * 1000), // 10 minutes
            max_video_duration: Some(5 * 60 * 1000), // 5 minutes
            supported_image_formats: vec![ImageFormat::Jpeg, ImageFormat::Png, ImageFormat::Webp],
            supported_audio_formats: vec![AudioFormat::Mp3, AudioFormat::Wav],
            supported_video_formats: vec![VideoFormat::Mp4],
            max_total_size: Some(50 * 1024 * 1024), // 50MB
        }
    }
}

impl Default for ProcessingOptions {
    fn default() -> Self {
        Self {
            resize_images: Some(ImageResizeOptions {
                max_width: 2048,
                max_height: 2048,
                maintain_aspect_ratio: true,
                quality: Some(85),
            }),
            compress_audio: None,
            extract_frames: None,
            ocr_enabled: false,
            speech_to_text: false,
        }
    }
}

pub struct MultiModalProcessor {
    capabilities: ModelCapabilities,
    options: ProcessingOptions,
}

impl MultiModalProcessor {
    pub fn new(capabilities: ModelCapabilities, options: ProcessingOptions) -> Self {
        Self {
            capabilities,
            options,
        }
    }

    pub fn validate_request(&self, request: &MultiModalRequest) -> Result<(), String> {
        let mut total_size = 0u64;

        for content in &request.contents {
            // Check if modality is supported
            if !self.capabilities.supported_modalities.contains(&content.modality) {
                return Err(format!("Unsupported modality: {:?}", content.modality));
            }

            // Validate content based on type
            match &content.data {
                MediaData::Image { format, data, .. } => {
                    if !self.capabilities.supported_image_formats.contains(format) {
                        return Err(format!("Unsupported image format: {:?}", format));
                    }
                    if let Some(max_size) = self.capabilities.max_image_size {
                        if data.len() as u64 > max_size {
                            return Err(format!("Image size {} exceeds maximum {}", data.len(), max_size));
                        }
                    }
                    total_size += data.len() as u64;
                }
                MediaData::Audio { format, data, duration_ms, .. } => {
                    if !self.capabilities.supported_audio_formats.contains(format) {
                        return Err(format!("Unsupported audio format: {:?}", format));
                    }
                    if let (Some(duration), Some(max_duration)) = (duration_ms, self.capabilities.max_audio_duration) {
                        if *duration > max_duration {
                            return Err(format!("Audio duration {} exceeds maximum {}", duration, max_duration));
                        }
                    }
                    total_size += data.len() as u64;
                }
                MediaData::Video { format, data, duration_ms, .. } => {
                    if !self.capabilities.supported_video_formats.contains(format) {
                        return Err(format!("Unsupported video format: {:?}", format));
                    }
                    if let (Some(duration), Some(max_duration)) = (duration_ms, self.capabilities.max_video_duration) {
                        if *duration > max_duration {
                            return Err(format!("Video duration {} exceeds maximum {}", duration, max_duration));
                        }
                    }
                    total_size += data.len() as u64;
                }
                MediaData::Base64 { data, .. } => {
                    // Estimate decoded size (base64 is ~33% larger than binary)
                    let estimated_size = (data.len() as f64 * 0.75) as u64;
                    total_size += estimated_size;
                }
                MediaData::Text { content } => {
                    total_size += content.len() as u64;
                }
                MediaData::Url { .. } => {
                    // URLs don't count toward size limit
                }
            }
        }

        // Check total size limit
        if let Some(max_total) = self.capabilities.max_total_size {
            if total_size > max_total {
                return Err(format!("Total content size {} exceeds maximum {}", total_size, max_total));
            }
        }

        Ok(())
    }

    pub fn should_process_content(&self, content: &MediaContent) -> bool {
        match &content.data {
            MediaData::Image { .. } => self.options.resize_images.is_some(),
            MediaData::Audio { .. } => self.options.compress_audio.is_some(),
            MediaData::Video { .. } => self.options.extract_frames.is_some(),
            _ => false,
        }
    }

    pub fn estimate_processing_time(&self, request: &MultiModalRequest) -> u64 {
        let mut total_time_ms = 0u64;

        for content in &request.contents {
            match &content.data {
                MediaData::Image { data, .. } => {
                    if self.options.resize_images.is_some() {
                        // Estimate ~1ms per KB for image processing
                        total_time_ms += (data.len() / 1024) as u64;
                    }
                }
                MediaData::Audio { duration_ms, .. } => {
                    if self.options.speech_to_text {
                        // Speech-to-text typically takes ~10% of audio duration
                        if let Some(duration) = duration_ms {
                            total_time_ms += duration / 10;
                        }
                    }
                }
                MediaData::Video { duration_ms, .. } => {
                    if self.options.extract_frames.is_some() {
                        // Frame extraction typically takes ~5% of video duration
                        if let Some(duration) = duration_ms {
                            total_time_ms += duration / 20;
                        }
                    }
                }
                MediaData::Text { content } => {
                    if self.options.ocr_enabled {
                        // OCR estimation based on text length
                        total_time_ms += (content.len() / 1000) as u64;
                    }
                }
                _ => {}
            }
        }

        total_time_ms
    }

    pub fn get_model_routing_hints(&self, request: &MultiModalRequest) -> HashMap<String, String> {
        let mut hints = HashMap::new();

        // Count modalities
        let mut modality_counts = HashMap::new();
        for content in &request.contents {
            *modality_counts.entry(&content.modality).or_insert(0) += 1;
        }

        // Determine primary modality
        let primary_modality = modality_counts
            .iter()
            .max_by_key(|(_, count)| *count)
            .map(|(modality, _)| modality);

        if let Some(modality) = primary_modality {
            hints.insert("primary_modality".to_string(), format!("{:?}", modality));
        }

        // Add complexity hints
        let has_video = modality_counts.contains_key(&ModalityType::Video);
        let has_audio = modality_counts.contains_key(&ModalityType::Audio);
        let has_image = modality_counts.contains_key(&ModalityType::Image);

        if has_video {
            hints.insert("complexity".to_string(), "high".to_string());
            hints.insert("preferred_provider".to_string(), "multimodal_specialist".to_string());
        } else if has_audio && has_image {
            hints.insert("complexity".to_string(), "medium".to_string());
        } else if has_image {
            hints.insert("complexity".to_string(), "low".to_string());
            hints.insert("preferred_provider".to_string(), "vision_model".to_string());
        }

        // Estimate token usage for routing decisions
        let estimated_tokens = self.estimate_token_usage(request);
        hints.insert("estimated_tokens".to_string(), estimated_tokens.to_string());

        hints
    }

    fn estimate_token_usage(&self, request: &MultiModalRequest) -> u64 {
        let mut total_tokens = 0u64;

        for content in &request.contents {
            match &content.data {
                MediaData::Text { content } => {
                    // ~4 characters per token
                    total_tokens += (content.len() / 4) as u64;
                }
                MediaData::Image { width, height, .. } => {
                    // Vision models typically use ~170 tokens per image + resolution factor
                    let base_tokens = 170u64;
                    let resolution_factor = match (width, height) {
                        (Some(w), Some(h)) => {
                            let pixels = (*w as u64) * (*h as u64);
                            if pixels > 2048 * 2048 { 3 }
                            else if pixels > 1024 * 1024 { 2 }
                            else { 1 }
                        }
                        _ => 1,
                    };
                    total_tokens += base_tokens * resolution_factor;
                }
                MediaData::Audio { duration_ms, .. } => {
                    // Audio models typically use ~1 token per second
                    if let Some(duration) = duration_ms {
                        total_tokens += duration / 1000;
                    }
                }
                MediaData::Video { duration_ms, .. } => {
                    // Video models use significantly more tokens
                    if let Some(duration) = duration_ms {
                        total_tokens += (duration / 1000) * 10; // ~10 tokens per second
                    }
                }
                _ => {}
            }
        }

        total_tokens
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_multimodal_validation() {
        let capabilities = ModelCapabilities {
            supported_modalities: vec![ModalityType::Text, ModalityType::Image],
            max_image_size: Some(1024), // 1KB for testing
            ..Default::default()
        };

        let processor = MultiModalProcessor::new(capabilities, ProcessingOptions::default());

        // Test valid request
        let valid_request = MultiModalRequest {
            contents: vec![
                MediaContent {
                    modality: ModalityType::Text,
                    data: MediaData::Text { content: "Hello".to_string() },
                    metadata: HashMap::new(),
                },
            ],
            model_capabilities: ModelCapabilities::default(),
            processing_options: ProcessingOptions::default(),
        };

        assert!(processor.validate_request(&valid_request).is_ok());

        // Test invalid modality
        let invalid_request = MultiModalRequest {
            contents: vec![
                MediaContent {
                    modality: ModalityType::Video, // Not supported
                    data: MediaData::Video {
                        format: VideoFormat::Mp4,
                        data: vec![0; 100],
                        duration_ms: Some(1000),
                        width: Some(640),
                        height: Some(480),
                        fps: Some(30.0),
                    },
                    metadata: HashMap::new(),
                },
            ],
            model_capabilities: ModelCapabilities::default(),
            processing_options: ProcessingOptions::default(),
        };

        assert!(processor.validate_request(&invalid_request).is_err());
    }

    #[test]
    fn test_token_estimation() {
        let processor = MultiModalProcessor::new(
            ModelCapabilities::default(),
            ProcessingOptions::default(),
        );

        let request = MultiModalRequest {
            contents: vec![
                MediaContent {
                    modality: ModalityType::Text,
                    data: MediaData::Text { content: "Hello world".to_string() }, // ~3 tokens
                    metadata: HashMap::new(),
                },
                MediaContent {
                    modality: ModalityType::Image,
                    data: MediaData::Image {
                        format: ImageFormat::Jpeg,
                        data: vec![0; 1000],
                        width: Some(512),
                        height: Some(512),
                    },
                    metadata: HashMap::new(),
                },
            ],
            model_capabilities: ModelCapabilities::default(),
            processing_options: ProcessingOptions::default(),
        };

        let tokens = processor.estimate_token_usage(&request);
        assert!(tokens > 170); // Should include base image tokens + text tokens
    }

    #[test]
    fn test_routing_hints() {
        let processor = MultiModalProcessor::new(
            ModelCapabilities::default(),
            ProcessingOptions::default(),
        );

        let request = MultiModalRequest {
            contents: vec![
                MediaContent {
                    modality: ModalityType::Image,
                    data: MediaData::Image {
                        format: ImageFormat::Jpeg,
                        data: vec![0; 1000],
                        width: Some(1024),
                        height: Some(1024),
                    },
                    metadata: HashMap::new(),
                },
            ],
            model_capabilities: ModelCapabilities::default(),
            processing_options: ProcessingOptions::default(),
        };

        let hints = processor.get_model_routing_hints(&request);
        assert_eq!(hints.get("primary_modality"), Some(&"Image".to_string()));
        assert_eq!(hints.get("preferred_provider"), Some(&"vision_model".to_string()));
    }
}