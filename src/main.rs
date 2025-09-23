use adw::prelude::*;
use adw::{ActionRow, Application, ApplicationWindow, HeaderBar, SwitchRow, AboutDialog};
use gtk::{gio, Button, FileDialog, FileFilter, Box as GtkBox, ListBox, Orientation, SelectionMode};
use std::fs;
use std::path::Path;
use glib;

fn is_audio_extension(ext: &str) -> bool {
    matches!(
        ext.to_ascii_lowercase().as_str(),
        "mp3" | "wav" | "flac" | "ogg" | "m4a" | "aac" | "opus"
    )
}

fn add_file_to_list(list: &ListBox, path: &Path) {
    let title = path
        .file_name()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_else(|| path.display().to_string());

    let row = ActionRow::builder()
        .activatable(true)
        .title(&title)
        .build();

    let path_buf = path.to_path_buf();
    row.connect_activated(move |_| {
        eprintln!("Ligne activée : {}", path_buf.display());
    });

    list.append(&row);
}

fn main() {
    adw::init().expect("Failed to init adw");

    let application = Application::builder()
        .application_id("com.example.FirstAdwaitaApp")
        .build();

    application.connect_activate(|app| {
        // --- boutons icônes (sans texte) ---
        let convert_btn = Button::builder()
            .label("Convertir")
            .build();

        let file_btn = Button::builder()
            .icon_name("document-open-symbolic")
            .build();
        file_btn.set_tooltip_text(Some("Choisir un fichier audio"));

        let folder_btn = Button::builder()
            .icon_name("folder-symbolic")
            .build();
        folder_btn.set_tooltip_text(Some("Choisir un dossier"));

        let prefs_btn = Button::builder()
            .icon_name("preferences-system-symbolic")
            .build();
        prefs_btn.set_tooltip_text(Some("Préférences"));

        let about_btn = Button::builder()
            .icon_name("help-about-symbolic")
            .build();
        about_btn.set_tooltip_text(Some("À propos"));

        // --- HeaderBar ---
        let header = HeaderBar::new();
        header.pack_start(&convert_btn);
        header.pack_start(&file_btn);
        header.pack_start(&folder_btn);
        header.pack_end(&prefs_btn);
        header.pack_end(&about_btn);

        // --- contenu (LISTE sans CSS personnalisé) ---
        let row = ActionRow::builder().activatable(true).title("Click me").build();
        row.connect_activated(|_| eprintln!("Clicked!"));
        let switch = SwitchRow::new();
        switch.set_title("Switch me");

        // <-- ici : plus de CSS personnalisée, on se repose sur le thème par défaut -->
        let list = ListBox::builder()
            .margin_top(16)
            .margin_end(16)
            .margin_bottom(16)
            .margin_start(16)
            .selection_mode(SelectionMode::None)
            .build();

        list.append(&row);
        list.append(&switch);

        let content = GtkBox::new(Orientation::Vertical, 0);
        content.append(&header);
        content.append(&list);

        // --- fenêtre ---
        let window = ApplicationWindow::builder()
            .application(app)
            .title("First App")
            .default_width(700)
            .default_height(420)
            .content(&content)
            .build();

        // --- filtre audio ---
        let audio_filter = FileFilter::new();
        audio_filter.set_name(Some("Fichiers audio"));
        audio_filter.add_mime_type("audio/*");

        let filters_model = gio::ListStore::new::<FileFilter>();
        filters_model.append(&audio_filter);

        // --- handlers ---
        let win_for_file = window.clone();
        let list_for_file = list.clone();
        let filters_model_for_file = filters_model.clone();
        file_btn.connect_clicked(move |_| {
            let dlg = FileDialog::new();
            dlg.set_title("Ouvrir un fichier audio");
            dlg.set_filters(Some(&filters_model_for_file));
            let list_for_file = list_for_file.clone();
            dlg.open(Some(&win_for_file), None::<&gio::Cancellable>, move |res: Result<gio::File, glib::Error>| {
                match res {
                    Ok(gfile) => {
                        if let Some(path) = gfile.path() {
                            eprintln!("Fichier audio choisi : {}", path.display());
                            add_file_to_list(&list_for_file, &path);
                        } else {
                            eprintln!("Fichier choisi (pas de path) : {:?}", gfile);
                        }
                    }
                    Err(err) => eprintln!("Aucun fichier sélectionné ou erreur: {}", err),
                }
            });
        });

        let win_for_folder = window.clone();
        let list_for_folder = list.clone();
        folder_btn.connect_clicked(move |_| {
            let dlg = FileDialog::new();
            dlg.set_title("Choisir un dossier");
            let list_for_folder = list_for_folder.clone();
            dlg.select_folder(Some(&win_for_folder), None::<&gio::Cancellable>, move |res: Result<gio::File, glib::Error>| {
                match res {
                    Ok(gfile) => {
                        if let Some(path) = gfile.path() {
                            eprintln!("Dossier choisi : {}", path.display());

                            if let Ok(entries) = fs::read_dir(&path) {
                                for entry in entries.flatten() {
                                    let p = entry.path();
                                    if p.is_file() {
                                        if let Some(ext) = p.extension().and_then(|e| e.to_str()) {
                                            if is_audio_extension(ext) {
                                                add_file_to_list(&list_for_folder, &p);
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                    Err(err) => eprintln!("Annulé / erreur sélection dossier: {}", err),
                }
            });
        });

        convert_btn.connect_clicked(|_| eprintln!("Convertir cliqué"));
        prefs_btn.connect_clicked(|_| eprintln!("Préférences cliqué"));

        let win_for_about = window.clone();
        about_btn.connect_clicked(move |_| {
            let about = AboutDialog::new();
            about.set_application_name("First App");
            about.set_version("0.1");
            about.set_comments("Application de conversion audio — exemple");
            about.present(Some(&win_for_about));
        });

        window.present();
    });

    application.run();
}
