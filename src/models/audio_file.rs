use std::path::PathBuf;

/// Représente un fichier audio dans l'application
#[derive(Debug, Clone)]
pub struct AudioFile {
    pub path: PathBuf,
}

impl AudioFile {
    /// Crée un nouveau fichier audio à partir d'un chemin
    pub fn new(path: PathBuf) -> Self {
        Self { path }
    }

    /// Retourne le nom du fichier
    pub fn filename(&self) -> String {
        self.path
            .file_name()
            .map(|s| s.to_string_lossy().into_owned())
            .unwrap_or_else(|| self.path.display().to_string())
    }

    /// Vérifie si l'extension du fichier est audio
    pub fn is_audio_file(path: &PathBuf) -> bool {
        if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
            Self::is_audio_extension(ext)
        } else {
            false
        }
    }

    /// Vérifie si une extension est une extension audio valide
    fn is_audio_extension(ext: &str) -> bool {
        matches!(
            ext.to_ascii_lowercase().as_str(),
            "mp3" | "wav" | "flac" | "ogg" | "m4a" | "aac" | "opus"
        )
    }
}
