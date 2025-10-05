use crate::models::audio_file::AudioFile;
use adw::prelude::*;
use adw::ActionRow;
use gtk::{ListBox, SelectionMode};

/// Composant FileList
/// Responsable de l'affichage de la liste des fichiers audio
#[derive(Clone)]
pub struct FileList {
    widget: gtk::ScrolledWindow,
    list_box: ListBox,
}

impl FileList {
    /// Crée une nouvelle liste de fichiers
    pub fn new() -> Self {
        let list_box = ListBox::new();
        list_box.set_margin_top(16);
        list_box.set_margin_bottom(16);
        list_box.set_margin_start(16);
        list_box.set_margin_end(16);
        list_box.set_selection_mode(SelectionMode::None);
        list_box.add_css_class("boxed-list");
        
        let widget = gtk::ScrolledWindow::builder()
            .vexpand(true)
            .hexpand(true)
            .child(&list_box)
            .build();
        
        Self { widget, list_box }
    }

    /// Retourne le widget GTK
    pub fn widget(&self) -> &gtk::ScrolledWindow {
        &self.widget
    }

    /// Ajoute un fichier à la liste
    pub fn add_file(&self, audio_file: &AudioFile) {
        let row = Self::create_row(audio_file);
        self.list_box.append(&row);
    }

    /// Ajoute plusieurs fichiers à la liste
    pub fn add_files(&self, audio_files: &[AudioFile]) {
        for audio_file in audio_files {
            self.add_file(audio_file);
        }
    }

    /// Vide la liste
    pub fn clear(&self) {
        while let Some(child) = self.list_box.first_child() {
            self.list_box.remove(&child);
        }
    }

    /// Crée une ligne pour un fichier audio
    fn create_row(audio_file: &AudioFile) -> ActionRow {
        let row = ActionRow::new();
        row.set_activatable(true);
        row.set_title(&audio_file.filename());
        
        let path_clone = audio_file.path.clone();
        row.connect_activated(move |_| {
            eprintln!("Ligne activée : {}", path_clone.display());
        });
        
        row
    }
}

impl Default for FileList {
    fn default() -> Self {
        Self::new()
    }
}
