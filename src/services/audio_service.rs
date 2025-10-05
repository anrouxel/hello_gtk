use crate::models::audio_file::AudioFile;
use std::cell::RefCell;
use std::fs;
use std::path::PathBuf;
use std::rc::Rc;

/// Service de gestion des fichiers audio
/// Responsable de la logique métier liée aux fichiers audio
#[derive(Clone)]
pub struct AudioService {
    files: Rc<RefCell<Vec<AudioFile>>>,
}

impl AudioService {
    /// Crée un nouveau service audio
    pub fn new() -> Self {
        Self {
            files: Rc::new(RefCell::new(Vec::new())),
        }
    }

    /// Ajoute un fichier audio à la liste
    pub fn add_file(&self, path: PathBuf) {
        let audio_file = AudioFile::new(path);
        self.files.borrow_mut().push(audio_file);
    }

    /// Ajoute tous les fichiers audio d'un dossier
    pub fn add_files_from_folder(&self, folder: PathBuf) -> Result<usize, std::io::Error> {
        let mut count = 0;
        
        let entries = fs::read_dir(&folder)?;
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() && AudioFile::is_audio_file(&path) {
                self.add_file(path);
                count += 1;
            }
        }
        
        Ok(count)
    }

    /// Retourne la liste des fichiers audio
    pub fn get_files(&self) -> Vec<AudioFile> {
        self.files.borrow().clone()
    }

    /// Retourne le nombre de fichiers
    pub fn count(&self) -> usize {
        self.files.borrow().len()
    }

    /// Vide la liste des fichiers
    pub fn clear(&self) {
        self.files.borrow_mut().clear();
    }
}

impl Default for AudioService {
    fn default() -> Self {
        Self::new()
    }
}
