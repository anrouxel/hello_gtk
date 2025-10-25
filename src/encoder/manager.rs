use crate::audio::AudioFormat;
use crate::metadata::AlbumDetails;
use super::{AudioEncoder, GStreamerEncoder};
use discid::DiscId;
use std::error::Error;

pub struct TranscodeManager {
    encoder: Box<dyn AudioEncoder>,
}

impl TranscodeManager {
    pub fn new() -> Self {
        Self {
            encoder: Box::new(GStreamerEncoder::new()),
        }
    }

    pub fn transcode_all_tracks(
        &self,
        disc: &DiscId,
        album: &AlbumDetails,
        format: AudioFormat,
    ) -> Result<(), Box<dyn Error>> {
        std::fs::create_dir_all("output")?;
        
        println!("Début du transcodage de l'album : {}", album.title);
        println!("Format d'encodage : {}", format.name());
        println!("Nombre de pistes : {}", album.tracks.len());
        
        for track in &album.tracks {
            let filename = format!("output/{}", GStreamerEncoder::create_output_filename(track, album, format));
            
            match self.encoder.transcode(disc, track, album, &filename, format) {
                Ok(()) => {
                    println!("✓ Piste {} transcodée avec succès", track.number);
                }
                Err(e) => {
                    eprintln!("✗ Erreur lors du transcodage de la piste {}: {}", track.number, e);
                }
            }
        }
        
        Ok(())
    }
}
