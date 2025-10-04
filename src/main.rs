use adw::prelude::*;
use adw::{ActionRow, AboutDialog, ApplicationWindow};
use gtk::{gio, glib, FileDialog, FileFilter, ListBox, SelectionMode, Orientation};
use std::cell::RefCell;
use std::fs;
use std::path::PathBuf;
use std::rc::Rc;

fn is_audio_extension(ext: &str) -> bool {
    matches!(
        ext.to_ascii_lowercase().as_str(),
        "mp3" | "wav" | "flac" | "ogg" | "m4a" | "aac" | "opus"
    )
}

fn create_audio_row(path: PathBuf) -> ActionRow {
    let row = ActionRow::new();
    row.set_activatable(true);
    
    let title = path
        .file_name()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_else(|| path.display().to_string());
    
    row.set_title(&title);
    
    let path_clone = path.clone();
    row.connect_activated(move |_| {
        eprintln!("Ligne activée : {}", path_clone.display());
    });
    
    row
}

fn build_ui(app: &adw::Application) {
    // Liste des fichiers audio
    let audio_files: Rc<RefCell<Vec<PathBuf>>> = Rc::new(RefCell::new(Vec::new()));
    
    // Création de la fenêtre principale
    let window = ApplicationWindow::builder()
        .application(app)
        .default_width(700)
        .default_height(420)
        .build();
    
    // Container principal
    let main_box = gtk::Box::new(Orientation::Vertical, 0);
    
    // HeaderBar
    let header = adw::HeaderBar::new();
    
    // Bouton Convertir
    let convert_btn = gtk::Button::builder()
        .label("Convertir")
        .tooltip_text("Convertir (Ctrl+R)")
        .build();
    
    // Bouton Ouvrir fichier
    let open_file_btn = gtk::Button::builder()
        .icon_name("document-open-symbolic")
        .tooltip_text("Choisir un fichier audio (Ctrl+O)")
        .build();
    
    // Bouton Ouvrir dossier
    let open_folder_btn = gtk::Button::builder()
        .icon_name("folder-symbolic")
        .tooltip_text("Choisir un dossier (Ctrl+F)")
        .build();
    
    // Bouton À propos
    let about_btn = gtk::Button::builder()
        .icon_name("help-about-symbolic")
        .tooltip_text("À propos")
        .build();
    
    // Bouton Préférences
    let prefs_btn = gtk::Button::builder()
        .icon_name("preferences-system-symbolic")
        .tooltip_text("Préférences")
        .build();
    
    header.pack_start(&convert_btn);
    header.pack_start(&open_file_btn);
    header.pack_start(&open_folder_btn);
    header.pack_end(&about_btn);
    header.pack_end(&prefs_btn);
    
    // Liste des fichiers
    let file_list = ListBox::new();
    file_list.set_margin_top(16);
    file_list.set_margin_bottom(16);
    file_list.set_margin_start(16);
    file_list.set_margin_end(16);
    file_list.set_selection_mode(SelectionMode::None);
    file_list.add_css_class("boxed-list");
    
    let scrolled = gtk::ScrolledWindow::builder()
        .vexpand(true)
        .hexpand(true)
        .child(&file_list)
        .build();
    
    main_box.append(&header);
    main_box.append(&scrolled);
    
    window.set_content(Some(&main_box));
    
    // Callbacks
    
    // Convertir
    let audio_files_clone = audio_files.clone();
    convert_btn.connect_clicked(move |_| {
        let files = audio_files_clone.borrow();
        eprintln!("Conversion lancée pour {} fichiers", files.len());
        for (idx, file) in files.iter().enumerate() {
            eprintln!("  {} - {}", idx + 1, file.display());
        }
    });
    
    // Ouvrir fichier
    let window_clone = window.clone();
    let audio_files_clone = audio_files.clone();
    let file_list_clone = file_list.clone();
    open_file_btn.connect_clicked(move |_| {
        let dialog = FileDialog::new();
        dialog.set_title("Ouvrir un fichier audio");
        
        let audio_filter = FileFilter::new();
        audio_filter.set_name(Some("Fichiers audio"));
        audio_filter.add_mime_type("audio/*");
        
        let filters = gio::ListStore::new::<FileFilter>();
        filters.append(&audio_filter);
        dialog.set_filters(Some(&filters));
        
        let audio_files_clone2 = audio_files_clone.clone();
        let file_list_clone2 = file_list_clone.clone();
        dialog.open(
            Some(&window_clone),
            gio::Cancellable::NONE,
            move |result| {
                if let Ok(file) = result {
                    if let Some(path) = file.path() {
                        eprintln!("Fichier audio choisi : {}", path.display());
                        audio_files_clone2.borrow_mut().push(path.clone());
                        let row = create_audio_row(path);
                        file_list_clone2.append(&row);
                    }
                }
            },
        );
    });
    
    // Ouvrir dossier
    let window_clone = window.clone();
    let audio_files_clone = audio_files.clone();
    let file_list_clone = file_list.clone();
    open_folder_btn.connect_clicked(move |_| {
        let dialog = FileDialog::new();
        dialog.set_title("Choisir un dossier");
        
        let audio_files_clone2 = audio_files_clone.clone();
        let file_list_clone2 = file_list_clone.clone();
        dialog.select_folder(
            Some(&window_clone),
            gio::Cancellable::NONE,
            move |result| {
                if let Ok(file) = result {
                    if let Some(folder) = file.path() {
                        eprintln!("Dossier choisi : {}", folder.display());
                        if let Ok(entries) = fs::read_dir(&folder) {
                            for entry in entries.flatten() {
                                let p = entry.path();
                                if p.is_file() {
                                    if let Some(ext) = p.extension().and_then(|e| e.to_str()) {
                                        if is_audio_extension(ext) {
                                            audio_files_clone2.borrow_mut().push(p.clone());
                                            let row = create_audio_row(p);
                                            file_list_clone2.append(&row);
                                        }
                                    }
                                }
                            }
                        } else {
                            eprintln!("Impossible de lire le dossier: {}", folder.display());
                        }
                    }
                }
            },
        );
    });
    
    // Préférences
    prefs_btn.connect_clicked(move |_| {
        eprintln!("Préférences cliqué");
    });
    
    // À propos
    let window_clone = window.clone();
    about_btn.connect_clicked(move |_| {
        let about = AboutDialog::new();
        about.set_application_name("First App");
        about.set_version("0.1");
        about.set_comments("Application de conversion audio — exemple avec GTK-RS + Adwaita");
        about.set_developers(&["Développeur"]);
        
        about.present(Some(&window_clone));
    });
    
    // Actions et raccourcis clavier
    let action_group = gio::SimpleActionGroup::new();
    
    // Action ouvrir fichier
    let open_file_action = gio::SimpleAction::new("open-file", None);
    let open_file_btn_clone = open_file_btn.clone();
    open_file_action.connect_activate(move |_, _| {
        open_file_btn_clone.activate();
    });
    action_group.add_action(&open_file_action);
    
    // Action ouvrir dossier
    let open_folder_action = gio::SimpleAction::new("open-folder", None);
    let open_folder_btn_clone = open_folder_btn.clone();
    open_folder_action.connect_activate(move |_, _| {
        open_folder_btn_clone.activate();
    });
    action_group.add_action(&open_folder_action);
    
    // Action convertir
    let convert_action = gio::SimpleAction::new("convert", None);
    let convert_btn_clone = convert_btn.clone();
    convert_action.connect_activate(move |_, _| {
        convert_btn_clone.activate();
    });
    action_group.add_action(&convert_action);
    
    window.insert_action_group("win", Some(&action_group));
    
    // Raccourcis clavier
    app.set_accels_for_action("win.open-file", &["<Ctrl>O"]);
    app.set_accels_for_action("win.open-folder", &["<Ctrl>F"]);
    app.set_accels_for_action("win.convert", &["<Ctrl>R"]);
    
    window.present();
}

fn main() -> glib::ExitCode {
    let app = adw::Application::builder()
        .application_id("com.example.FirstAdwaitaApp")
        .build();
    
    app.connect_activate(build_ui);
    
    app.run()
}
