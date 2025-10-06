use crate::components::{about_dialog::AboutDialogComponent, file_list::FileList, header_bar::HeaderBar};
use crate::services::audio_service::AudioService;
use adw::prelude::*;
use adw::{ApplicationWindow, ToolbarView};
use gtk::{gio, FileDialog, FileFilter};

/// Composant MainWindow
/// Responsable de l'assemblage de tous les composants et de la gestion des interactions
pub struct MainWindow {
    window: ApplicationWindow,
    header_bar: HeaderBar,
    file_list: FileList,
    audio_service: AudioService,
}

impl MainWindow {
    /// Crée et configure la fenêtre principale
    pub fn new(app: &adw::Application) -> Self {
        // Création de la fenêtre
        let window = ApplicationWindow::builder()
            .application(app)
            .default_width(700)
            .default_height(420)
            .build();
        
        // Création des composants
        let header_bar = HeaderBar::new();
        let file_list = FileList::new();
        let audio_service = AudioService::new();
        
        // Utilisation de ToolbarView pour la fusion headerbar/contenu
        let toolbar_view = ToolbarView::new();
        toolbar_view.add_top_bar(header_bar.widget());
        toolbar_view.set_content(Some(file_list.widget()));
                
        window.set_content(Some(&toolbar_view));
        
        let mut main_window = Self {
            window,
            header_bar,
            file_list,
            audio_service,
        };
        
        // Configuration des interactions
        main_window.setup_callbacks(app);
        
        main_window
    }

    /// Configure tous les callbacks et interactions
    fn setup_callbacks(&mut self, app: &adw::Application) {
        // Setup actions et raccourcis clavier
        self.header_bar.setup_actions(app, &self.window);
        
        // Bouton Convertir
        let audio_service = self.audio_service.clone();
        self.header_bar.on_convert(move || {
            let files = audio_service.get_files();
            eprintln!("Conversion lancée pour {} fichiers", files.len());
            for (idx, file) in files.iter().enumerate() {
                eprintln!("  {} - {}", idx + 1, file.path.display());
            }
        });
        
        // Bouton Ouvrir fichier
        let window_clone = self.window.clone();
        let audio_service = self.audio_service.clone();
        let file_list = self.file_list.clone();
        self.header_bar.on_open_file(move || {
            Self::open_file_dialog(&window_clone, audio_service.clone(), file_list.clone());
        });
        
        // Bouton Ouvrir dossier
        let window_clone = self.window.clone();
        let audio_service = self.audio_service.clone();
        let file_list = self.file_list.clone();
        self.header_bar.on_open_folder(move || {
            Self::open_folder_dialog(&window_clone, audio_service.clone(), file_list.clone());
        });
        
        // Bouton À propos
        let window_clone = self.window.clone();
        self.header_bar.on_about(move || {
            AboutDialogComponent::show(&window_clone);
        });
        
        // Bouton Préférences
        self.header_bar.on_preferences(|| {
            eprintln!("Préférences cliqué");
        });
    }

    /// Ouvre un dialogue de sélection de fichier
    fn open_file_dialog(
        window: &ApplicationWindow,
        audio_service: AudioService,
        file_list: FileList,
    ) {
        let dialog = FileDialog::new();
        dialog.set_title("Ouvrir un fichier audio");
        
        let audio_filter = FileFilter::new();
        audio_filter.set_name(Some("Fichiers audio"));
        audio_filter.add_mime_type("audio/*");
        
        let filters = gio::ListStore::new::<FileFilter>();
        filters.append(&audio_filter);
        dialog.set_filters(Some(&filters));
        
        let audio_service = audio_service.clone();
        let file_list = file_list.clone();
        let audio_service = audio_service.clone();
        let file_list = file_list.clone();
        dialog.open(
            Some(window),
            gio::Cancellable::NONE,
            move |result| {
                if let Ok(file) = result {
                    if let Some(path) = file.path() {
                        eprintln!("Fichier audio choisi : {}", path.display());
                        audio_service.add_file(path.clone());
                        
                        // Rafraîchir l'affichage
                        let files = audio_service.get_files();
                        if let Some(last_file) = files.last() {
                            file_list.add_file(last_file);
                        }
                    }
                }
            },
        );
    }

    /// Ouvre un dialogue de sélection de dossier
    fn open_folder_dialog(
        window: &ApplicationWindow,
        audio_service: AudioService,
        file_list: FileList,
    ) {
        let dialog = FileDialog::new();
        dialog.set_title("Choisir un dossier");
        
        let audio_service = audio_service.clone();
        let file_list = file_list.clone();
        let audio_service = audio_service.clone();
        let file_list = file_list.clone();
        dialog.select_folder(
            Some(window),
            gio::Cancellable::NONE,
            move |result| {
                if let Ok(file) = result {
                    if let Some(folder) = file.path() {
                        eprintln!("Dossier choisi : {}", folder.display());
                        
                        let count_before = audio_service.count();
                        match audio_service.add_files_from_folder(folder.clone()) {
                            Ok(count) => {
                                eprintln!("{} fichiers audio ajoutés", count);
                                
                                // Rafraîchir l'affichage
                                let files = audio_service.get_files();
                                let new_files = &files[count_before..];
                                file_list.add_files(new_files);
                            }
                            Err(e) => {
                                eprintln!("Impossible de lire le dossier {}: {}", folder.display(), e);
                            }
                        }
                    }
                }
            },
        );
    }

    /// Affiche la fenêtre
    pub fn present(&self) {
        self.window.present();
    }
}
