use gstreamer_pbutils::{EncodingAudioProfile, EncodingContainerProfile};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AudioFormat {
    Opus,
    Vorbis,
    Flac,
    Mp3,
    Aac,
    Wavpack,
}

impl AudioFormat {
    pub fn file_extension(&self) -> &str {
        match self {
            AudioFormat::Opus => "opus",
            AudioFormat::Vorbis => "ogg",
            AudioFormat::Flac => "flac",
            AudioFormat::Mp3 => "mp3",
            AudioFormat::Aac => "m4a",
            AudioFormat::Wavpack => "wv",
        }
    }

    pub fn name(&self) -> &str {
        match self {
            AudioFormat::Opus => "Opus",
            AudioFormat::Vorbis => "Ogg Vorbis",
            AudioFormat::Flac => "FLAC",
            AudioFormat::Mp3 => "MP3",
            AudioFormat::Aac => "AAC",
            AudioFormat::Wavpack => "WavPack",
        }
    }

    pub fn is_lossless(&self) -> bool {
        matches!(self, AudioFormat::Flac | AudioFormat::Wavpack)
    }

    pub fn create_encoding_profile(&self) -> EncodingContainerProfile {
        match self {
            AudioFormat::Opus => {
                let opus_caps = gstreamer::Caps::builder("audio/x-opus").build();
                let audio_profile = EncodingAudioProfile::builder(&opus_caps).build();
                
                let ogg_caps = gstreamer::Caps::builder("application/ogg").build();
                EncodingContainerProfile::builder(&ogg_caps)
                    .add_profile(audio_profile)
                    .build()
            }
            AudioFormat::Vorbis => {
                let vorbis_caps = gstreamer::Caps::builder("audio/x-vorbis").build();
                let audio_profile = EncodingAudioProfile::builder(&vorbis_caps).build();
                
                let ogg_caps = gstreamer::Caps::builder("application/ogg").build();
                EncodingContainerProfile::builder(&ogg_caps)
                    .add_profile(audio_profile)
                    .build()
            }
            AudioFormat::Flac => {
                let flac_caps = gstreamer::Caps::builder("audio/x-flac").build();
                let audio_profile = EncodingAudioProfile::builder(&flac_caps).build();
                
                let flac_container_caps = gstreamer::Caps::builder("audio/x-flac").build();
                EncodingContainerProfile::builder(&flac_container_caps)
                    .add_profile(audio_profile)
                    .build()
            }
            AudioFormat::Mp3 => {
                let mp3_caps = gstreamer::Caps::builder("audio/mpeg")
                    .field("mpegversion", 1)
                    .field("layer", 3)
                    .build();
                let audio_profile = EncodingAudioProfile::builder(&mp3_caps).build();
                
                let id3_caps = gstreamer::Caps::builder("application/x-id3").build();
                EncodingContainerProfile::builder(&id3_caps)
                    .add_profile(audio_profile)
                    .build()
            }
            AudioFormat::Aac => {
                let aac_caps = gstreamer::Caps::builder("audio/mpeg")
                    .field("mpegversion", 4)
                    .build();
                let audio_profile = EncodingAudioProfile::builder(&aac_caps).build();
                
                let mp4_caps = gstreamer::Caps::builder("video/quicktime")
                    .field("variant", "iso")
                    .build();
                EncodingContainerProfile::builder(&mp4_caps)
                    .add_profile(audio_profile)
                    .build()
            }
            AudioFormat::Wavpack => {
                let wavpack_caps = gstreamer::Caps::builder("audio/x-wavpack").build();
                let audio_profile = EncodingAudioProfile::builder(&wavpack_caps).build();
                
                let wv_caps = gstreamer::Caps::builder("audio/x-wavpack").build();
                EncodingContainerProfile::builder(&wv_caps)
                    .add_profile(audio_profile)
                    .build()
            }
        }
    }

    pub fn all_formats() -> Vec<AudioFormat> {
        vec![
            AudioFormat::Opus,
            AudioFormat::Vorbis,
            AudioFormat::Flac,
            AudioFormat::Mp3,
            AudioFormat::Aac,
            AudioFormat::Wavpack,
        ]
    }
}
