use super::AudioFormat;
use gstreamer::{ElementFactory, prelude::*};

pub struct FormatValidator;

impl FormatValidator {
    pub fn check_support(format: AudioFormat) -> bool {
        let encodebin = match ElementFactory::make("encodebin").build() {
            Ok(e) => e,
            Err(_) => {
                eprintln!("   ⚠ encodebin non disponible pour {}", format.name());
                return false;
            }
        };
        
        let profile = format.create_encoding_profile();
        encodebin.set_property("profile", &profile);
        
        let pad = encodebin.request_pad_simple("audio_%u");
        let supported = pad.is_some();
        
        if !supported {
            eprintln!("   ✗ {} - plugins manquants", format.name());
        }
        
        if let Some(pad) = pad {
            encodebin.release_request_pad(&pad);
        }
        
        supported
    }

    pub fn get_supported_formats() -> Vec<AudioFormat> {
        AudioFormat::all_formats()
            .into_iter()
            .filter(|&format| Self::check_support(format))
            .collect()
    }
}
